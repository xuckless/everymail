//! The Backend's event matcher and its limits, as pure functions: which agents an event wakes,
//! whether a woken agent may run now, which approvals ran out and which events come up.

use crate::{
    builder::{self, ALL, ANY, LEAD_TIMES, TRIGGERS, Trigger},
    schedule::Schedule,
    time,
    world::{Agent, Approval, Cause, Outcome, Ref, World},
};

/// A chain of agents waking agents stops at this depth.
pub const MAX_DEPTH: u8 = 3;
/// Fixed safety caps for everyone.
pub const RUNS_PER_DAY: usize = 200;
pub const CONCURRENT_RUNS: usize = 3;
/// An approval nobody answers ends as declined after this long.
pub const APPROVAL_SECONDS: i64 = 7 * time::DAY;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// Gatekeeper finished with new mail, or there is no Gatekeeper.
    NewMail(String),
    /// Mail sorted into a category or moved into a folder, by filter choice id.
    Sorted {
        thread: String,
        into: String,
    },
    Flagged(String),
    Starred(String),
    /// A run of this agent finished with this deliverable.
    Finished {
        agent: String,
        run: String,
        refs: Vec<Ref>,
    },
    /// The user answered an approval of this agent.
    Resolved {
        approval: String,
        agent: String,
    },
    /// By lead time id.
    Approaching {
        event: String,
        lead: &'static str,
    },
    /// A rule or Gatekeeper handed mail straight to an agent, by id.
    Handoff {
        to: String,
        by: String,
        thread: String,
    },
}

/// An event with what caused it: the agent whose action it was, and that run's depth (0 for the
/// user, a rule, Gatekeeper's sorting or the outside world).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fired {
    pub event: Event,
    pub by: Option<String>,
    pub depth: u8,
}

impl Fired {
    pub fn outside(event: Event) -> Self {
        Self {
            event,
            by: None,
            depth: 0,
        }
    }
}

/// An agent to run, why, at what depth, and the events to fire once it has run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Wake {
    pub agent: String,
    pub cause: Cause,
    pub depth: u8,
    pub then: Vec<Fired>,
}

impl Wake {
    pub fn new(agent: &str, cause: Cause) -> Self {
        Self {
            agent: agent.into(),
            cause,
            depth: 1,
            then: Vec::new(),
        }
    }
}

/// The agents these events wake, each once, in order.
pub fn wakes(fired: &[Fired], world: &World) -> Vec<Wake> {
    let mut out: Vec<Wake> = Vec::new();
    for f in fired {
        for agent in &world.agents {
            if out.iter().any(|w| w.agent == agent.id) {
                continue;
            }
            if let Some(cause) = cause(f, agent, world) {
                out.push(Wake {
                    agent: agent.id.clone(),
                    cause,
                    depth: f.depth + 1,
                    then: Vec::new(),
                });
            }
        }
    }
    out
}

fn cause(fired: &Fired, agent: &Agent, world: &World) -> Option<Cause> {
    if fired.depth >= MAX_DEPTH || !agent.active || fired.by.as_deref() == Some(&agent.id) {
        return None;
    }
    if let Event::Handoff { to, by, thread } = &fired.event {
        return (*to == agent.id).then(|| Cause::Handoff(by.clone(), Ref::Thread(thread.clone())));
    }
    TRIGGERS
        .iter()
        .filter(|t| agent.wakes.triggers.iter().any(|id| id == t.id))
        .find_map(|t| matches(t, &fired.event, agent, world))
}

fn matches(trigger: &Trigger, event: &Event, agent: &Agent, world: &World) -> Option<Cause> {
    // The id picked for a filter the trigger has; `None` when it has no such filter.
    let value = |filter: &str| builder::picked(&agent.wakes, trigger, filter, world);
    // The sender, account and folder filters, on mail the agent can see.
    let mail = |thread: &str| {
        let Some(t) = world.thread(thread) else {
            return false;
        };
        agent.accounts.contains(&t.account)
            && value("sender").is_none_or(|s| match s.as_str() {
                "known" => world.known(&t.from),
                "new" => !world.known(&t.from),
                _ => true,
            })
            && value("account").is_none_or(|a| a == ALL || a == t.account)
            && value("folder").is_none_or(|f| f == ANY || t.folders.contains(&f))
    };
    let thread = |id: &str| Ref::Thread(id.into());
    match (trigger.id, event) {
        ("new-mail", Event::NewMail(t)) if mail(t) => Some(Cause::Mail(thread(t))),
        ("sorted", Event::Sorted { thread: t, into })
            if mail(t) && value("into").as_ref() == Some(into) =>
        {
            Some(Cause::Sorted(into.clone(), thread(t)))
        }
        ("flagged", Event::Flagged(t)) if mail(t) => Some(Cause::Flagged(thread(t))),
        ("labelled", Event::Starred(t)) if mail(t) => Some(Cause::Starred(thread(t))),
        (
            "agent-finished",
            Event::Finished {
                agent: finished,
                run,
                refs,
            },
        ) if value("agent").as_ref() == Some(finished) => {
            Some(Cause::Finished(run.clone(), refs.clone()))
        }
        (
            "approval-resolved",
            Event::Resolved {
                approval,
                agent: owner,
            },
        ) if value("from").is_some_and(|f| f == ANY || f == *owner) => {
            Some(Cause::Resolved(Ref::Approval(approval.clone())))
        }
        ("event-approaching", Event::Approaching { event, lead })
            if value("lead-time").as_deref() == Some(*lead)
                && world.event(event).is_some_and(|e| {
                    value("calendar").is_none_or(|c| c == ALL || c == e.calendar)
                }) =>
        {
            Some(Cause::Approaching(
                (*lead).into(),
                Ref::Event(event.clone()),
            ))
        }
        _ => None,
    }
}

/// Why a woken agent waits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hold {
    Paused,
    /// Its own run is still going: one run at a time per agent.
    Running,
    /// The user's runs at once are at the cap.
    Full,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Gate {
    Start,
    Wait(Hold),
    Refuse(String),
}

/// Gatekeeper and Security Guard have a code stage that runs without a model and costs nothing.
pub fn code_stage(agent: &Agent) -> bool {
    matches!(agent.id.as_str(), "gatekeeper" | "security-guard")
}

/// Runs of the agent that started today and were not stopped before they ran.
pub fn runs_today(agent: &Agent, now: i64) -> usize {
    let today = time::start_of_day(now);
    agent
        .runs
        .iter()
        .filter(|r| r.started_at >= today && !matches!(r.outcome, Outcome::Stopped(_)))
        .count()
}

/// Whether the agent may run for this cause now.
pub fn gate(agent: &Agent, cause: &Cause, world: &World) -> Gate {
    let name = &agent.name;
    let settings = &world.settings;
    let running = world.agents.iter().filter(|a| a.running()).count();
    if agent.schedule == Schedule::OnMessage && !matches!(cause, Cause::Message(_)) {
        Gate::Refuse(format!("{name} answers when you write to it"))
    } else if !agent.active {
        Gate::Refuse(format!("{name} is off. Turn it on first"))
    } else if !code_stage(agent) && !world.key_set(agent.model.provider) {
        Gate::Refuse(format!("{name} needs a model key"))
    } else if runs_today(agent, world.now) >= RUNS_PER_DAY {
        Gate::Refuse(format!(
            "{name} ran {RUNS_PER_DAY} times today, the most an agent may. It runs again tomorrow"
        ))
    } else if !code_stage(agent)
        && settings
            .budget_tokens
            .is_some_and(|budget| settings.used_tokens >= budget)
    {
        Gate::Refuse(
            "Today’s token budget is used up. Raise it in Settings or wait for tomorrow".into(),
        )
    } else if settings.agents_paused {
        Gate::Wait(Hold::Paused)
    } else if agent.running() {
        Gate::Wait(Hold::Running)
    } else if running >= CONCURRENT_RUNS {
        Gate::Wait(Hold::Full)
    } else {
        Gate::Start
    }
}

/// What the user is told when Run now meets a hold.
pub fn hold_text(hold: Hold, agent: &Agent) -> String {
    match hold {
        Hold::Paused => "Agents are paused. Resume them in Settings".into(),
        Hold::Running => format!("{} is running", agent.name),
        Hold::Full => format!(
            "{CONCURRENT_RUNS} agents are running, the most at once. Try again when one finishes"
        ),
    }
}

/// Waiting approvals that have waited too long.
pub fn expired(world: &World) -> Vec<&Approval> {
    world
        .waiting()
        .filter(|a| a.created_at + APPROVAL_SECONDS <= world.now)
        .collect()
}

pub fn next_expiry(world: &World) -> Option<i64> {
    world
        .waiting()
        .map(|a| a.created_at + APPROVAL_SECONDS)
        .min()
}

/// Every (event, lead time) and the instant it comes due.
fn leads(world: &World) -> impl Iterator<Item = (&str, &'static str, i64)> {
    world.events.iter().flat_map(|e| {
        let start = time::at(e.date, e.start);
        LEAD_TIMES
            .iter()
            .map(move |(lead, _, seconds)| (e.id.as_str(), *lead, start - seconds))
    })
}

fn watched(world: &World) -> bool {
    world
        .agents
        .iter()
        .any(|a| a.active && a.wakes.triggers.iter().any(|t| t == "event-approaching"))
}

/// Events whose lead times came due after `from`, up to now.
pub fn approaching(world: &World, from: i64) -> Vec<Fired> {
    if !watched(world) {
        return Vec::new();
    }
    leads(world)
        .filter(|(_, _, at)| *at > from && *at <= world.now)
        .map(|(event, lead, _)| {
            Fired::outside(Event::Approaching {
                event: event.into(),
                lead,
            })
        })
        .collect()
}

pub fn next_approaching(world: &World) -> Option<i64> {
    watched(world)
        .then(|| {
            leads(world)
                .map(|(_, _, at)| at)
                .filter(|at| *at > world.now)
                .min()
        })
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{backend::Update, demo::tests::world, store::Store};

    fn finished(agent: &str, depth: u8) -> Fired {
        Fired {
            event: Event::Finished {
                agent: agent.into(),
                run: "run-x".into(),
                refs: vec![Ref::Thread("t-okafor".into())],
            },
            by: Some(agent.into()),
            depth,
        }
    }

    /// Two agents that each wait for the other.
    fn chained(store: &Store) {
        store
            .connection()
            .execute_batch(
                "INSERT INTO agent_triggers VALUES ('day-summary', 'agent-finished'), ('follow-up-nudger', 'agent-finished');
                 INSERT INTO agent_filters VALUES ('day-summary', 'agent-finished', 'agent', 'follow-up-nudger'), ('follow-up-nudger', 'agent-finished', 'agent', 'day-summary');",
            )
            .unwrap();
    }

    #[test]
    fn another_agent_finishing_wakes_the_agent_naming_it_until_depth_three() {
        let (store, w) = world();
        chained(&store);
        let w = store.load(w.now, None).unwrap();
        let woken = wakes(&[finished("follow-up-nudger", 1)], &w);
        assert_eq!(woken.len(), 1);
        assert_eq!(woken[0].agent, "day-summary");
        assert_eq!(woken[0].depth, 2);
        assert_eq!(
            woken[0].cause,
            Cause::Finished("run-x".into(), vec![Ref::Thread("t-okafor".into())])
        );
        assert_eq!(wakes(&[finished("day-summary", 2)], &w)[0].depth, 3);
        assert!(
            wakes(&[finished("day-summary", 3)], &w).is_empty(),
            "the chain stops at depth 3"
        );
    }

    #[test]
    fn an_agent_never_wakes_itself() {
        let (store, w) = world();
        store
            .connection()
            .execute_batch(
                "INSERT INTO agent_triggers VALUES ('day-summary', 'agent-finished');
                 INSERT INTO agent_filters VALUES ('day-summary', 'agent-finished', 'agent', 'day-summary');
                 INSERT INTO agent_filters VALUES ('job-tracker', 'sorted', 'into', 'receipts');",
            )
            .unwrap();
        let w2 = store.load(w.now, None).unwrap();
        assert!(wakes(&[finished("day-summary", 1)], &w2).is_empty());
        // Receipts clerk filing into Receipts wakes the watcher of Receipts, never itself.
        let moved = Fired {
            event: Event::Sorted {
                thread: "t-northline".into(),
                into: "receipts".into(),
            },
            by: Some("job-tracker".into()),
            depth: 1,
        };
        assert!(wakes(std::slice::from_ref(&moved), &w2).is_empty());
        let by_user = Fired::outside(moved.event.clone());
        let woken = wakes(&[by_user], &w2);
        assert_eq!(woken.len(), 1);
        assert_eq!(woken[0].agent, "job-tracker");
    }

    #[test]
    fn mail_triggers_follow_their_filters() {
        let (_, w) = world();
        let sorted = |into: &str| {
            Fired::outside(Event::Sorted {
                thread: "t-northline".into(),
                into: into.into(),
            })
        };
        // Job application tracker watches For you; Receipts clerk any new mail.
        let woken: Vec<String> = wakes(&[sorted("for-you")], &w)
            .into_iter()
            .map(|w| w.agent)
            .collect();
        assert_eq!(woken, vec!["job-tracker".to_string()]);
        assert!(wakes(&[sorted("updates")], &w).is_empty());
        let new_mail = Fired::outside(Event::NewMail("t-northline".into()));
        let both = wakes(&[new_mail, sorted("for-you")], &w);
        assert_eq!(
            both.iter().map(|w| w.agent.as_str()).collect::<Vec<_>>(),
            vec!["receipts-clerk", "job-tracker"],
            "each agent once"
        );
    }

    #[test]
    fn the_gate_holds_and_refuses() {
        let (store, w) = world();
        let agent = |w: &World| -> Agent {
            let a = w.agent("receipts-clerk").unwrap();
            Agent {
                runs: Vec::new(),
                turns: Vec::new(),
                ..a.clone()
            }
        };
        assert_eq!(gate(&agent(&w), &Cause::RunNow, &w), Gate::Start);
        let assistant = w.agent("assistant").unwrap();
        assert!(matches!(
            gate(assistant, &Cause::RunNow, &w),
            Gate::Refuse(_)
        ));
        store.apply(&Update::AgentsPaused(true), w.now).unwrap();
        let paused = store.load(w.now, None).unwrap();
        assert_eq!(
            gate(&agent(&paused), &Cause::RunNow, &paused),
            Gate::Wait(Hold::Paused)
        );
        store.apply(&Update::AgentsPaused(false), w.now).unwrap();
        store.apply(&Update::Budget(Some(1_000)), w.now).unwrap();
        let spent = store.load(w.now, None).unwrap();
        assert!(spent.settings.used_tokens >= 1_000);
        assert!(matches!(
            gate(&agent(&spent), &Cause::RunNow, &spent),
            Gate::Refuse(r) if r.starts_with("Today’s token budget")
        ));
        let gatekeeper = spent.agent("gatekeeper").unwrap();
        assert_ne!(
            gate(gatekeeper, &Cause::RunNow, &spent),
            Gate::Refuse(
                "Today’s token budget is used up. Raise it in Settings or wait for tomorrow".into()
            ),
            "a code stage costs no tokens"
        );
    }

    #[test]
    fn the_daily_cap_counts_runs_that_ran() {
        let (_, w) = world();
        let mut a = w.agent("receipts-clerk").unwrap().clone();
        let run = a.runs[0].clone();
        a.runs = (0..RUNS_PER_DAY)
            .map(|i| crate::world::Run {
                id: format!("r{i}"),
                started_at: w.now - 60,
                outcome: Outcome::Done,
                ..run.clone()
            })
            .collect();
        assert!(matches!(gate(&a, &Cause::RunNow, &w), Gate::Refuse(_)));
        a.runs[0].outcome = Outcome::Stopped("Not run".into());
        assert_eq!(runs_today(&a, w.now), RUNS_PER_DAY - 1);
    }

    #[test]
    fn approvals_expire_after_seven_days() {
        let (_, mut w) = world();
        assert!(expired(&w).is_empty());
        let oldest = w.waiting().map(|a| a.created_at).min().unwrap();
        assert_eq!(next_expiry(&w), Some(oldest + APPROVAL_SECONDS));
        w.now = oldest + APPROVAL_SECONDS;
        let gone = expired(&w);
        assert!(!gone.is_empty() && gone.iter().all(|a| a.created_at == oldest));
    }

    #[test]
    fn approaching_events_fire_once_per_lead_time() {
        let (store, w) = world();
        store
            .connection()
            .execute_batch(
                "INSERT INTO agent_triggers VALUES ('day-summary', 'event-approaching');
                 INSERT INTO agent_filters VALUES ('day-summary', 'event-approaching', 'lead-time', '1-hour');",
            )
            .unwrap();
        // The exam is Thursday 14:00: an hour before is 13:00.
        let mut w = store.load(w.now, None).unwrap();
        let thirteen = time::at(crate::date::Date::parse("2026-11-12").unwrap(), 13 * 60);
        assert_eq!(next_approaching(&w).map(|at| at <= thirteen), Some(true));
        w.now = thirteen;
        let fired = approaching(&w, thirteen - 60);
        let woken = wakes(&fired, &w);
        assert_eq!(woken.len(), 1);
        assert_eq!(
            woken[0].cause,
            Cause::Approaching("1-hour".into(), Ref::Event("e-exam".into()))
        );
        assert!(approaching(&w, thirteen).is_empty(), "not twice");
    }
}
