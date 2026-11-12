use super::{count, disclosure, grouped, is, mail, plural, settings::permissions, time_choices};
use crate::{
    backend::Decision,
    builder::{self, Pick, SCHEDULES, TRIGGERS},
    date::WEEKDAYS,
    live::{DryRun, Field, Live},
    navigation::{self, Disclosure, Navigation, Pane, PaneChoice, STEPS, Selection, Step},
    proto::ui::v1::{
        self as ui, PermissionLevel, agent_page::Pane as PaneView, agent_status::State as Status,
        agents_view::Page, approvals_view::Queue, briefing_view::State as Briefing,
    },
    schedule::{self, Schedule},
    time,
    world::{
        Agent, ApprovalAction, Cause, Edition, Line, Outcome, Ref, ResultsKind, StepPermission,
        TurnState, World, paragraphs,
    },
};

/// The narrow builder's step labels.
fn step_label(step: Step) -> &'static str {
    match step {
        Step::Instructions => "What should it do",
        Step::Triggers => "When does it wake up",
        Step::Permissions => "What it may do",
        Step::Summary => "Review",
    }
}

const IDEAS: [&str; 4] = [
    "Track my job applications",
    "File receipts to my budget sheet",
    "Nudge threads nobody answered",
    "Unsubscribe from lists I ignore",
];

pub fn builder(step: Step, world: &World, live: &Live) -> ui::AgentBuilderView {
    use ui::{
        agent_builder_view::{Back, Creation, DryRun as View, Forward},
        builder_trigger::Readiness,
        dry_run_result::Decision as Choice,
    };
    let messages = |accounts: &[String]| {
        let all: usize = world
            .threads
            .iter()
            .filter(|t| accounts.contains(&t.account))
            .map(|t| t.messages.len())
            .sum();
        all.min(usize::try_from(world.settings.dry_run_messages).unwrap_or(0))
    };
    let b = &world.builder;
    let model = world
        .task_model(crate::world::TASK_AGENTS)
        .or(world.models.first());
    ui::AgentBuilderView {
        name: b.name.clone(),
        instructions: b.instructions.clone(),
        ideas: IDEAS.iter().map(|i| (*i).into()).collect(),
        triggers: TRIGGERS
            .iter()
            .map(|t| {
                let chosen = b.wakes.triggers.iter().any(|c| c == t.id);
                ui::BuilderTrigger {
                    id: t.id.into(),
                    label: t.label.into(),
                    chosen,
                    filters: t
                        .filters
                        .iter()
                        .map(|f| {
                            use ui::trigger_filter::Pick as View;
                            let pick = builder::pick(&b.wakes, t.id, f, world);
                            let picked = match &pick {
                                Pick::Picked { id, .. } => Some(id.as_str()),
                                Pick::NotPicked => None,
                            };
                            ui::TriggerFilter {
                                id: f.id.into(),
                                label: f.label.into(),
                                choices: builder::options(f, world)
                                    .into_iter()
                                    .map(|(id, label)| ui::FilterChoice {
                                        selected: picked == Some(id.as_str()),
                                        id,
                                        label,
                                    })
                                    .collect(),
                                pick: Some(match pick {
                                    Pick::Picked { label, .. } => {
                                        View::Picked(ui::FilterPicked { label })
                                    }
                                    Pick::NotPicked => View::NotPicked(ui::FilterNotPicked {
                                        placeholder: builder::required(f)
                                            .map_or("", |(placeholder, _)| placeholder)
                                            .into(),
                                    }),
                                }),
                            }
                        })
                        .collect(),
                    filter_summary: builder::filter_summary(t, &b.wakes, world),
                    readiness: Some(match builder::missing(t, &b.wakes, world) {
                        Some(text) if chosen => {
                            Readiness::Incomplete(ui::TriggerIncomplete { text: text.into() })
                        }
                        Some(_) | None => Readiness::Ready(ui::TriggerReady {}),
                    }),
                }
            })
            .collect(),
        schedule: Some(ui::BuilderSchedule {
            choices: SCHEDULES
                .iter()
                .map(|option| ui::ScheduleChoice {
                    id: option.id.into(),
                    label: builder::option_label(option),
                    selected: option.id == b.schedule.id,
                })
                .collect(),
            state: Some(match builder::schedule(b).label() {
                None => ui::builder_schedule::State::None(ui::Unscheduled {}),
                Some(label) => ui::builder_schedule::State::Scheduled(ui::Scheduled { label }),
            }),
            interval_minutes: (b.interval_seconds / 60).to_string(),
            interval_hint: schedule::INTERVAL_HINT.into(),
        }),
        accounts: world
            .accounts
            .iter()
            .map(|a| ui::BuilderAccount {
                id: a.id.clone(),
                name: a.name.clone(),
                provider: a.provider.into(),
                provider_name: a.provider_name.clone(),
                can_see: b.accounts.contains(&a.id),
            })
            .collect(),
        permissions: permissions(&b.permissions),
        ask_before_new_recipient: b.ask_before_new_recipient,
        test_model: model.map(mail::model),
        dry_run: Some(match &live.dry_run {
            DryRun::Idle => View::Idle(ui::DryRunIdle {
                text: format!(
                    "A dry run on your last {}. Nothing is sent, moved or changed",
                    plural(
                        usize::try_from(world.settings.dry_run_messages).unwrap_or(0),
                        "message"
                    )
                ),
            }),
            DryRun::Reading => View::Reading(ui::DryRunReading {
                progress: format!(
                    "Reading {} across {}",
                    plural(messages(&b.accounts), "message"),
                    plural(b.accounts.len(), "account")
                ),
            }),
            DryRun::Results(lines) => View::Results(ui::DryRunResults {
                results: lines
                    .iter()
                    .map(|l| ui::DryRunResult {
                        from: l.from.clone(),
                        action: l.action.clone(),
                        decision: Some(match l.decision {
                            Decision::Ask => Choice::Ask(ui::DecisionAsk {}),
                            Decision::Auto => Choice::Auto(ui::DecisionAuto {}),
                            Decision::Skip => Choice::Skip(ui::DecisionSkip {}),
                        }),
                    })
                    .collect(),
            }),
            DryRun::Failed(reason) => View::Failed(ui::DryRunFailed {
                reason: reason.clone(),
            }),
        }),
        summary: [
            ("Does", b.instructions.trim().to_string()),
            ("Wakes", builder::wakes(b)),
            (
                "Auto",
                builder::at_level(&b.permissions, PermissionLevel::Auto),
            ),
            (
                "Asks first",
                builder::at_level(&b.permissions, PermissionLevel::Ask),
            ),
        ]
        .into_iter()
        .map(|(label, value)| ui::SummaryLine {
            label: label.into(),
            value,
        })
        .collect(),
        steps: STEPS
            .iter()
            .map(|s| ui::BuilderStepTab {
                step: ui::BuilderStep::from(*s).into(),
                label: step_label(*s).into(),
                current: *s == step,
            })
            .collect(),
        creation: Some(match builder::incomplete(b, world) {
            Some(text) => Creation::Incomplete(ui::BuilderIncomplete { text }),
            None => Creation::Ready(ui::BuilderReady {}),
        }),
        revision: live.revisions.of(&Field::Builder),
        back: Some(match navigation::step_by(step, -1) {
            Some(before) => Back::BackStep(ui::BuilderStep::from(before).into()),
            None => Back::First(ui::BuilderFirstStep {}),
        }),
        forward: Some(match navigation::step_by(step, 1) {
            Some(after) => Forward::NextStep(ui::BuilderStep::from(after).into()),
            None => Forward::Last(ui::BuilderLastStep {}),
        }),
        progress_percent: STEPS
            .iter()
            .position(|s| *s == step)
            .map_or(0, |at| (100 * (at + 1) / STEPS.len()) as u32),
    }
}

/// When it wakes: "On every new mail, after your rules", "Daily at 7:30 and 18:00"
fn when(agent: &Agent) -> String {
    schedule::wakes(&builder::trigger_labels(&agent.wakes), &agent.schedule)
}

/// The approvals waiting on an agent.
fn waiting_on(agent: &Agent, world: &World) -> usize {
    world.waiting().filter(|a| a.agent == agent.id).count()
}

pub fn status(agent: &Agent, world: &World) -> ui::AgentStatus {
    let state = if !agent.active {
        Status::Deactivated(ui::AgentDeactivated {})
    } else if agent.running() {
        Status::Running(ui::AgentRunning {})
    } else if waiting_on(agent, world) > 0 {
        Status::Waiting(ui::AgentWaiting {})
    } else {
        Status::Idle(ui::AgentIdle {})
    };
    ui::AgentStatus { state: Some(state) }
}

/// When its schedule wakes it next.
enum NextWake {
    /// The slot has come and the Backend has yet to start it.
    Due,
    /// As a menu says it: "18:00", "Thu 9:00"
    At(String),
}

impl NextWake {
    fn short(self) -> String {
        match self {
            NextWake::Due => "due now".into(),
            NextWake::At(at) => at,
        }
    }
}

fn next_wake(agent: &Agent, world: &World) -> Option<NextWake> {
    agent.next_wake().map(|at| {
        if at <= world.now {
            NextWake::Due
        } else {
            NextWake::At(crate::time::long(at, world.now))
        }
    })
}

/// The sidebar note: the waiting count, or when it runs next.
pub fn note(agent: &Agent, world: &World) -> String {
    let waiting = waiting_on(agent, world);
    if waiting > 0 {
        return waiting.to_string();
    }
    next_wake(agent, world).map_or(String::new(), NextWake::short)
}

/// The list row's line: "On every new mail", "Weekdays at 9:00 · next Thu 9:00 · 1 waiting"
fn next(agent: &Agent, world: &World) -> String {
    let mut parts = vec![when(agent)];
    parts.extend(next_wake(agent, world).map(|at| format!("next {}", at.short())));
    match waiting_on(agent, world) {
        0 => {}
        n => parts.push(format!("{n} waiting")),
    }
    parts.join(" · ")
}

/// The last run: "9:45 · Sorted 6 · flagged 1"
fn last_run(agent: &Agent, world: &World) -> String {
    agent.runs.first().map_or(String::new(), |r| {
        format!("{} · {}", time::short(r.started_at, world.now), r.summary)
    })
}

/// The agents list, or the page of the agent `page` names with its pane.
pub fn agents(
    page: Option<(&Agent, PaneChoice)>,
    navigation: &Navigation,
    world: &World,
    live: &Live,
) -> ui::AgentsView {
    let page = match page {
        Some((agent, pane)) => Page::Agent(self::page(agent, pane, navigation, world, live)),
        None => {
            let rows = |system: bool| {
                world
                    .agents
                    .iter()
                    .filter(|a| a.system() == system)
                    .map(|a| ui::AgentRow {
                        id: a.id.clone(),
                        name: a.name.clone(),
                        status: Some(status(a, world)),
                        next: next(a, world),
                        last_run: last_run(a, world),
                    })
                    .collect()
            };
            Page::List(ui::AgentList {
                system_agents: rows(true),
                user_agents: rows(false),
                approvals: Some(nav_approvals(false, world)),
            })
        }
    };
    ui::AgentsView { page: Some(page) }
}

/// The sidebar entry, and the phone's row on the agents list.
pub fn nav_approvals(current: bool, world: &World) -> ui::NavApprovals {
    ui::NavApprovals {
        label: "Approvals".into(),
        waiting: count(world.waiting().count()),
        current,
    }
}

/// The queue: what waits or resolves, and what was just carried out or declined while its outcome
/// lasts.
pub fn approvals(world: &World, live: &Live) -> ui::ApprovalsView {
    use ui::approval_item::Action;
    let items: Vec<ui::ApprovalItem> = world
        .approvals
        .iter()
        .filter_map(|a| {
            let state = mail::approval_state(a, live)?;
            Some(ui::ApprovalItem {
                id: a.id.clone(),
                agent: a.agent_name.clone(),
                time: time::short(a.created_at, world.now),
                title: a.title.clone(),
                detail: a.detail.clone(),
                reason: a.reason.clone(),
                action: Some(match &a.action {
                    ApprovalAction::Send(thread) => Action::Send(ui::SendApproval {
                        thread_id: thread.clone(),
                    }),
                    ApprovalAction::Unsubscribe => Action::Unsubscribe(ui::UnsubscribeApproval {}),
                }),
                state: Some(state),
            })
        })
        .collect();
    let waiting = world.waiting().count();
    let queue = if items.is_empty() {
        Queue::Clear(ui::QueueClear {
            text: "All clear. Nothing waits for you".into(),
        })
    } else {
        Queue::Waiting(ui::ApprovalQueue {
            count: plural(waiting, "item"),
            items,
        })
    };
    ui::ApprovalsView { queue: Some(queue) }
}

const PANE_LABELS: [(Pane, &str); 4] = [
    (Pane::Results, "Results"),
    (Pane::Activity, "Activity"),
    (Pane::Chat, "Chat"),
    (Pane::Settings, "Settings"),
];

fn page(
    agent: &Agent,
    pane: PaneChoice,
    navigation: &Navigation,
    world: &World,
    live: &Live,
) -> ui::AgentPage {
    let panes = agent.panes();
    let current = match pane {
        PaneChoice::Default => panes.default,
        PaneChoice::Other(pane) => pane,
    };
    let content = match current {
        Pane::Activity => PaneView::Activity(activity(agent, navigation, world)),
        Pane::Chat => PaneView::Chat(chat(agent, world, live)),
        Pane::Settings => PaneView::Settings(settings(agent, world, live)),
        Pane::Results => PaneView::Results(results(agent, navigation, world)),
    };
    ui::AgentPage {
        id: agent.id.clone(),
        name: agent.name.clone(),
        subtitle: format!("{} · {}", when(agent), agent.model.name),
        status: Some(status(agent, world)),
        can_run_now: agent.schedule != Schedule::OnMessage,
        panes: PANE_LABELS
            .iter()
            .filter(|(p, _)| panes.panes.contains(p))
            .map(|(p, label)| ui::AgentDetailPaneTab {
                pane: ui::AgentDetailPane::from(*p).into(),
                label: (*label).into(),
                current: *p == current,
            })
            .collect(),
        pane: Some(content),
    }
}

/// "COSC 4456 exam · Thu 14:00"
fn event_label(id: &str, world: &World) -> String {
    world.event(id).map_or(String::new(), |e| {
        format!(
            "{} · {} {}",
            e.title,
            WEEKDAYS[e.date.weekday()],
            time::clock(e.start)
        )
    })
}

fn reference(target: &Ref, world: &World) -> ui::Reference {
    use ui::reference::Target;
    let (label, target) = match target {
        Ref::Thread(id) => (
            world
                .thread(id)
                .map_or(String::new(), |t| t.subject.clone()),
            Target::Thread(ui::ThreadReference {
                thread_id: id.clone(),
            }),
        ),
        Ref::Event(id) => (
            event_label(id, world),
            Target::Event(ui::EventReference {
                event_id: id.clone(),
            }),
        ),
        Ref::Approval(id) => (
            world
                .approval(id)
                .map_or(String::new(), |a| a.title.clone()),
            Target::Approval(ui::ApprovalReference {
                approval_id: id.clone(),
            }),
        ),
        Ref::Quarantined(id) => (
            world
                .quarantine
                .iter()
                .find(|q| q.id == *id)
                .map_or(String::new(), |q| q.subject.clone()),
            Target::Quarantined(ui::QuarantinedReference {
                quarantined_id: id.clone(),
            }),
        ),
    };
    ui::Reference {
        label,
        target: Some(target),
    }
}

fn references(refs: &[Ref], world: &World) -> Vec<ui::Reference> {
    refs.iter().map(|r| reference(r, world)).collect()
}

fn stats(stats: &[crate::world::Stat]) -> Vec<ui::Stat> {
    stats
        .iter()
        .map(|s| ui::Stat {
            value: s.value.clone(),
            label: s.label.clone(),
        })
        .collect()
}

fn results(agent: &Agent, navigation: &Navigation, world: &World) -> ui::AgentResults {
    use ui::agent_results::State;
    let state = match agent.results {
        ResultsKind::Briefing => State::Briefing(briefing(&navigation.edition, world)),
        ResultsKind::Groups if !agent.result_items.is_empty() => {
            let mut groups: Vec<ui::ResultGroup> = Vec::new();
            for item in &agent.result_items {
                let shown = ui::ResultItem {
                    id: item.id.clone(),
                    title: item.title.clone(),
                    detail: item.detail.clone(),
                    time: time::short(item.at, world.now),
                    references: references(&item.refs, world),
                };
                match groups.last_mut() {
                    Some(group) if group.title == item.group => group.items.push(shown),
                    _ => groups.push(ui::ResultGroup {
                        title: item.group.clone(),
                        items: vec![shown],
                    }),
                }
            }
            State::Groups(ui::ResultGroups {
                stats: stats(&agent.result_stats),
                groups,
            })
        }
        ResultsKind::Groups | ResultsKind::Conversation => State::Empty(ui::ResultsEmpty {
            text: "Nothing yet. Results appear after its first run".into(),
        }),
    };
    ui::AgentResults { state: Some(state) }
}

fn cause(agent: &Agent, cause: &Cause, world: &World) -> ui::RunCause {
    let about = |text: String, r: &Ref| ui::RunCause {
        text: format!("{text}: {}", reference(r, world).label),
        references: vec![reference(r, world)],
    };
    let plain = |text: String| ui::RunCause {
        text,
        references: vec![],
    };
    match cause {
        Cause::Mail(r) => about("New mail".into(), r),
        Cause::Opened(r) => about("You opened".into(), r),
        Cause::Handoff(by, r) => about(format!("Handed over by {by}"), r),
        Cause::Schedule(when) => plain(format!("Scheduled: {when}")),
        Cause::Message(id) => plain(format!(
            "Your message: “{}”",
            agent
                .turns
                .iter()
                .find(|t| t.id == *id)
                .map_or("", |t| t.message.as_str())
        )),
        Cause::Connected(account) => plain(format!("{account} was connected")),
        Cause::RunNow => plain("You pressed Run now".into()),
        Cause::Sorted(into, r) => about(format!("Sorted into {}", world.place_name(into)), r),
        Cause::Flagged(r) => about("Flagged".into(), r),
        Cause::Starred(r) => about("You starred".into(), r),
        Cause::Resolved(r) => about("You answered".into(), r),
        Cause::Approaching(lead, r) => {
            about(format!("Coming up, {}", builder::lead_label(lead)), r)
        }
        Cause::Finished(run, refs) => ui::RunCause {
            text: match world.run(run) {
                Some((by, r)) => format!("{} finished: {}", by.name, r.summary),
                None => "An agent you deleted finished".into(),
            },
            references: references(refs, world),
        },
    }
}

fn activity(agent: &Agent, navigation: &Navigation, world: &World) -> ui::AgentActivity {
    use ui::run_outcome::State;
    ui::AgentActivity {
        stats: stats(&agent.stats),
        runs: agent
            .runs
            .iter()
            .map(|run| ui::AgentRun {
                id: run.id.clone(),
                start: time::short(run.started_at, world.now),
                cause: Some(cause(agent, &run.cause, world)),
                duration: match run.seconds {
                    Some(seconds) => format!("{seconds} s"),
                    None => format!("{} s so far", (world.now - run.started_at).max(0)),
                },
                model: agent.model.name.clone(),
                step_count: plural(run.steps.len(), "step"),
                tokens: if run.tokens == 0 {
                    "No model calls".into()
                } else {
                    format!("{} tokens", grouped(run.tokens))
                },
                outcome: Some(ui::RunOutcome {
                    state: Some(match &run.outcome {
                        Outcome::Done => State::Done(ui::RunDone {}),
                        Outcome::Waiting => State::Waiting(ui::RunWaiting {}),
                        Outcome::Running => State::Running(ui::RunRunning {}),
                        Outcome::Paused => State::Paused(ui::RunPaused {}),
                        Outcome::Failed(reason) => State::Failed(ui::RunFailed {
                            reason: reason.clone(),
                        }),
                        Outcome::Stopped(text) => {
                            State::Stopped(ui::RunStopped { text: text.clone() })
                        }
                    }),
                }),
                summary: run.summary.clone(),
                // Steps are only ever added after the last, so a position names one for good.
                steps: run
                    .steps
                    .iter()
                    .enumerate()
                    .map(|(i, step)| {
                        let (permission, label) = match step.permission {
                            StepPermission::Read => (ui::StepPermission::Read, ""),
                            StepPermission::Auto => (ui::StepPermission::Auto, "Ran on Auto"),
                            StepPermission::Asked => (ui::StepPermission::Asked, "Asked you"),
                            StepPermission::Off => (ui::StepPermission::Off, "Off: not done"),
                        };
                        ui::RunStep {
                            id: format!("{}-{i}", run.id),
                            verb: step.verb.clone(),
                            what: step.what.clone(),
                            permission: permission.into(),
                            permission_label: label.into(),
                            references: references(&step.refs, world),
                        }
                    })
                    .collect(),
                detail: Some(disclosure(navigation, &Disclosure::Run(run.id.clone()))),
            })
            .collect(),
    }
}

fn settings(agent: &Agent, world: &World, live: &Live) -> ui::AgentSettings {
    use ui::agent_settings::Schedule as View;
    use ui::agent_settings::{Instructions, News, NextRun, Removal};
    ui::AgentSettings {
        instructions: Some(match &agent.built_in {
            Some(built_in) => Instructions::System(ui::SystemInstructions {
                built_in: built_in.clone(),
                yours: agent.instructions.clone(),
                yours_label: format!("Your instructions for {}", agent.name),
            }),
            None => Instructions::User(ui::UserInstructions {
                text: agent.instructions.clone(),
            }),
        }),
        schedule: Some(match &agent.schedule {
            Schedule::Triggers | Schedule::BuiltIn(_) => {
                View::OnEvent(ui::EventSchedule { when: when(agent) })
            }
            Schedule::Interval(_) => View::Every(ui::IntervalSchedule { when: when(agent) }),
            Schedule::Times { weekdays, minutes } => View::AtTimes(ui::TimesSchedule {
                when: schedule::times_prefix(*weekdays).into(),
                set_times: minutes
                    .iter()
                    .map(|m| ui::TimeChoice {
                        minute: *m,
                        label: time::clock(*m),
                    })
                    .collect(),
            }),
            Schedule::OnMessage => View::OnMessage(ui::MessageSchedule { when: when(agent) }),
        }),
        model: Some(mail::model(&agent.model)),
        models: mail::model_choices(&agent.model.name, world),
        permissions: permissions(&agent.permissions),
        accounts: world
            .accounts
            .iter()
            .map(|a| ui::AgentAccount {
                id: a.id.clone(),
                name: a.name.clone(),
                provider: a.provider.into(),
                included: agent.accounts.contains(&a.id),
            })
            .collect(),
        news: Some(match agent.results {
            ResultsKind::Briefing => News::Topics(ui::NewsTopicSettings {
                topics: world.news_topics.clone(),
                cap: plural(world.news_topics.len(), "topic"),
                revision: live.revisions.of(&Field::NewsTopic),
            }),
            ResultsKind::Groups | ResultsKind::Conversation => News::None(ui::NoNewsTopics {}),
        }),
        key_note: agent.key_note.clone(),
        removal: Some(if agent.system() {
            Removal::DeactivateOnly(ui::DeactivateOnly {
                text: "Built-in agents can be turned off, not deleted".into(),
            })
        } else {
            Removal::Delete(ui::DeleteAgent {
                label: "Delete agent".into(),
            })
        }),
        time_choices: time_choices(),
        triggers: TRIGGERS
            .iter()
            .filter(|t| agent.wakes.triggers.iter().any(|id| id == t.id))
            .map(|t| ui::AgentTrigger {
                label: t.label.into(),
                filters: builder::filter_summary(t, &agent.wakes, world),
            })
            .collect(),
        next_run: Some(match next_wake(agent, world) {
            Some(NextWake::Due) => NextRun::Next(ui::NextRun {
                text: "Due now".into(),
            }),
            Some(NextWake::At(at)) => NextRun::Next(ui::NextRun {
                text: format!("Next run {at}"),
            }),
            None => NextRun::Unscheduled(ui::NoNextRun {}),
        }),
    }
}

pub fn briefing(selection: &Selection, world: &World) -> ui::BriefingView {
    ui::BriefingView {
        editions: world
            .editions
            .iter()
            .map(|e| ui::BriefingEditionTab {
                id: e.id.clone(),
                label: time::long(e.written_at, world.now),
                selected: is(selection, &e.id),
            })
            .collect(),
        state: Some(
            world
                .editions
                .iter()
                .find(|e| is(selection, &e.id))
                .map_or(Briefing::None(ui::NoBriefing {}), |e| {
                    Briefing::Edition(edition(e))
                }),
        ),
    }
}

fn section_empty(text: &str) -> ui::SectionEmpty {
    ui::SectionEmpty { text: text.into() }
}

fn lines(lines: &[Line], empty: &str) -> ui::LineSection {
    use ui::line_section::State;
    ui::LineSection {
        state: Some(if lines.is_empty() {
            State::Empty(section_empty(empty))
        } else {
            State::Lines(ui::BriefingLines {
                lines: lines
                    .iter()
                    .map(|l| ui::BriefingLine {
                        source: l.source.clone(),
                        text: l.text.clone(),
                    })
                    .collect(),
            })
        }),
    }
}

fn edition(edition: &Edition) -> ui::BriefingEdition {
    use ui::{
        calendar_section::State as Calendar, news_section::State as News,
        waiting_section::State as Waiting,
    };
    ui::BriefingEdition {
        id: edition.id.clone(),
        byline: edition.byline.clone(),
        activity: Some(lines(&edition.activity, "Your agents had nothing to do")),
        caught_mail: edition.caught_mail,
        waiting: Some(ui::WaitingSection {
            state: Some(if edition.waiting.is_empty() {
                Waiting::Empty(section_empty("Nothing waits for you"))
            } else {
                Waiting::Waiting(ui::BriefingWaitingList {
                    items: edition
                        .waiting
                        .iter()
                        .map(|(approval, title, detail)| ui::BriefingWaiting {
                            title: title.clone(),
                            detail: detail.clone(),
                            approval_id: approval.clone(),
                        })
                        .collect(),
                })
            }),
        }),
        newsletters: Some(lines(&edition.newsletters, "No newsletters")),
        updates: Some(lines(&edition.updates, "No updates")),
        calendar: Some(ui::CalendarSection {
            title: edition.calendar_title.clone(),
            state: Some(if edition.calendar.is_empty() {
                Calendar::Empty(section_empty("Nothing planned"))
            } else {
                Calendar::Events(ui::BriefingEvents {
                    events: edition
                        .calendar
                        .iter()
                        .map(|(time, title, detail)| ui::BriefingEvent {
                            time: time.clone(),
                            title: title.clone(),
                            detail: detail.clone(),
                        })
                        .collect(),
                })
            }),
        }),
        news: Some(ui::NewsSection {
            state: Some(if edition.news.is_empty() {
                News::None(section_empty(
                    "No topics followed. Add some in News Bearer’s settings",
                ))
            } else {
                News::Topics(ui::NewsTopics {
                    topics: edition
                        .news
                        .iter()
                        .map(|(name, items)| ui::NewsTopic {
                            name: name.clone(),
                            items: items
                                .iter()
                                .map(|(headline, source)| ui::NewsItem {
                                    headline: headline.clone(),
                                    source: source.clone(),
                                    link_label: format!("Read at {source}"),
                                })
                                .collect(),
                        })
                        .collect(),
                })
            }),
        }),
    }
}

fn chat(agent: &Agent, world: &World, live: &Live) -> ui::AgentChat {
    use ui::{agent_chat::Availability, agent_chat::Conversation, chat_turn::State};
    let name = &agent.name;
    let missing_key = world
        .settings
        .keys
        .iter()
        .find(|k| k.provider == agent.model.provider && !k.set)
        .map(|k| k.name.as_str());
    let busy = |text: String| Availability::Busy(ui::ChatBusy { text });
    let availability = match (!agent.active, missing_key, agent.running_turn()) {
        (true, _, _) => Availability::Deactivated(ui::ChatDeactivated {
            text: format!("{name} is off. Turn it on to chat"),
        }),
        (false, Some(key), _) => Availability::Off(ui::ChatOff {
            text: format!("Chat is off until you add a {key} key"),
            action: "Add a key".into(),
        }),
        (false, None, Some(_)) => busy(format!("{name} is answering")),
        (false, None, None) => Availability::Ready(ui::ChatReady {
            placeholder: format!("Message {name}"),
        }),
    };
    let conversation = if agent.turns.is_empty() {
        Conversation::Empty(ui::ChatEmpty {
            hint: format!("Ask {name} about its work, or give it a one-off task"),
        })
    } else {
        Conversation::Turns(ui::ChatTurns {
            turns: agent
                .turns
                .iter()
                .map(|t| ui::ChatTurn {
                    id: t.id.clone(),
                    message: t.message.clone(),
                    time: time::short(t.sent_at, world.now),
                    state: Some(match &t.state {
                        TurnState::Running(label) => State::Running(ui::TurnRunning {
                            label: label.clone(),
                        }),
                        TurnState::Waiting(approval) => State::Waiting(ui::TurnWaiting {
                            approval_id: approval.clone(),
                            approval_title: reference(&Ref::Approval(approval.clone()), world)
                                .label,
                            label: "Waiting for your approval".into(),
                        }),
                        TurnState::Answered(text, refs) => State::Answered(ui::TurnAnswered {
                            paragraphs: paragraphs(text).into_iter().map(String::from).collect(),
                            references: references(refs, world),
                        }),
                        TurnState::Failed(reason) => State::Failed(ui::TurnFailed {
                            reason: reason.clone(),
                        }),
                        TurnState::Stopped(text) => {
                            State::Stopped(ui::TurnStopped { text: text.clone() })
                        }
                    }),
                })
                .collect(),
        })
    };
    ui::AgentChat {
        availability: Some(availability),
        conversation: Some(conversation),
        revision: live.revisions.of(&Field::Chat(agent.id.clone())),
    }
}
