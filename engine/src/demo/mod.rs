//! A stand-in for the Backend until it is built: the sample world as the first sync, scripted
//! outcomes for the work only the Backend does, its answers to the changes it decides, and its own
//! jobs: the mail the sample keeps receiving, sends and snoozes coming due, quarantine retention,
//! the event matcher that wakes agents, schedules, approaching events, approval expiry, the safety
//! caps, pausing and the time limit on requests. Results reach the replica through the same updates
//! real sync applies. No model and no network; nothing outside this module knows it is not the real
//! Backend.
//!
//! Every job runs in a timer callback, never inside the engine's handling of an input, and one at
//! a time under `Jobs`, so each decision reads the replica as the decisions before it left it.

mod matcher;
mod pipeline;
mod policy;
mod sample;
mod script;

use std::{
    collections::VecDeque,
    rc::Rc,
    sync::{Arc, Mutex},
};

use matcher::{Event, Fired, Gate, Hold, Wake};
use pipeline::Incoming;
use script::Answer;

use crate::{
    action::{Action, MoveTo},
    backend::{Backend, Link, Request, Update, Work},
    live::Target,
    platform,
    schedule::Schedule,
    world::{ApprovalStatus, Cause, Outcome, Ref, SendState, TurnState, World},
};

/// A connected account gets its first new mail this long after its first sync.
const WELCOME_MS: u64 = 15_000;
/// The Backend gives up on a request the user waits on after this long.
const REQUEST_SECONDS: i64 = 60;

pub struct Demo(Arc<Mutex<Jobs>>);

/// The Backend's own state between jobs. In the real Backend this is the jobs table.
struct Jobs {
    /// Wakes held until agents are resumed, their agent's run ends or a slot among the runs at
    /// once frees.
    waiting: Vec<Wake>,
    /// The Backend time the timer is set for.
    armed: Option<i64>,
    /// Lead times of events were looked at up to this Backend time.
    approaching_from: i64,
    /// The sample's mail arrives counted from this Backend time: the last sync, or the start.
    delivering_from: i64,
    /// Run ids: the device clock when the Backend started, counted up.
    serial: i64,
    /// Requests the user waits on, with when the Backend gives up on each.
    pending: Vec<(Request, i64)>,
    /// Agents are paused: a run's next step waits.
    paused: bool,
    /// Runs the pause holds, with the steps they have left.
    parked: Vec<(String, VecDeque<(u64, Step)>)>,
}

enum Job {
    /// The Backend started: pick up runs a closed app left.
    Start,
    /// The first sync after sign-in.
    Sync,
    Work(Work),
    Report(Action),
    /// The timer went off.
    Tick,
    Wake(Wake),
    Arrive(Incoming),
    /// A run's last update landed; `wake` is what started it.
    Ended {
        run: String,
        wake: Wake,
    },
    Fire(Vec<Fired>),
}

enum Step {
    Update(Update),
    Then(Job),
}

impl Default for Demo {
    fn default() -> Self {
        Self::new()
    }
}

impl Demo {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(Jobs {
            waiting: Vec::new(),
            armed: None,
            approaching_from: 0,
            delivering_from: 0,
            serial: platform::wall_ms(),
            pending: Vec::new(),
            paused: false,
            parked: Vec::new(),
        })))
    }

    fn later(&self, replica: Link, job: Job) {
        play(
            self.0.clone(),
            replica,
            None,
            VecDeque::from([(0, Step::Then(job))]),
        );
    }
}

impl Backend for Demo {
    /// The wall clock.
    fn now(&self) -> i64 {
        platform::wall_ms().div_euclid(1000)
    }

    fn start(&self, replica: Link) {
        self.later(replica, Job::Start);
    }

    fn report(&self, action: &Action, replica: Link) {
        self.later(replica, Job::Report(action.clone()));
    }

    fn request(&self, work: Work, replica: Link) {
        self.later(replica, Job::Work(work));
    }
}

/// Takes the steps in order, each its delay after the one before. The steps of a run wait while
/// agents are paused: the step before finished, the rest goes on when they are resumed.
fn play(
    jobs: Arc<Mutex<Jobs>>,
    replica: Link,
    run: Option<String>,
    mut steps: VecDeque<(u64, Step)>,
) {
    let Some((delay, step)) = steps.pop_front() else {
        return;
    };
    platform::after(
        delay,
        Box::new(move || {
            if let Some(run) = &run
                && let Ok(mut j) = jobs.lock()
                && j.paused
            {
                steps.push_front((0, step));
                j.parked.push((run.clone(), steps));
                drop(j);
                replica.deliver(Update::RunPaused(run.clone()));
                return;
            }
            match step {
                Step::Update(update) => {
                    if answers(&jobs, &update) {
                        replica.deliver(update);
                    }
                }
                Step::Then(job) => handle(&jobs, &replica, job),
            }
            play(jobs, replica, run, steps);
        }),
    );
}

/// Whether an answer still goes out: one the user stopped waiting for, or that came too late, is
/// dropped. The final answer ends the wait.
fn answers(jobs: &Mutex<Jobs>, update: &Update) -> bool {
    let (request, last) = match update {
        Update::SignedIn { .. } | Update::Snapshot(_) => (Request::SignIn, true),
        Update::Reply { thread, .. } => (Request::Reply(thread.clone()), true),
        Update::DryRun(_) => (Request::DryRun, true),
        Update::ExportProgress(_) => (Request::Export, false),
        Update::ExportReady { .. } => (Request::Export, true),
        Update::Failed { request, .. } => (request.clone(), true),
        // A summary is Scribe's work whether or not the user waits on it.
        Update::Summary { thread, .. } => {
            if let Ok(mut j) = jobs.lock() {
                j.pending
                    .retain(|(r, _)| *r != Request::Summary(thread.clone()));
            }
            return true;
        }
        _ => return true,
    };
    let Ok(mut j) = jobs.lock() else {
        return false;
    };
    let waiting = j.pending.iter().any(|(r, _)| *r == request);
    if waiting && last {
        j.pending.retain(|(r, _)| *r != request);
    }
    waiting
}

fn updates(updates: Vec<(u64, Update)>) -> VecDeque<(u64, Step)> {
    updates
        .into_iter()
        .map(|(delay, update)| (delay, Step::Update(update)))
        .collect()
}

/// One job, then whatever waits that may start now, then the timer for what comes due next.
fn handle(shared: &Arc<Mutex<Jobs>>, replica: &Link, job: Job) {
    let Ok(mut jobs) = shared.lock() else {
        return;
    };
    let mut cx = Cx {
        shared,
        replica,
        jobs: &mut jobs,
        world: None,
    };
    match job {
        Job::Start => cx.start(),
        Job::Sync => cx.sync(),
        Job::Work(work) => cx.work(work),
        Job::Report(action) => {
            if let Some(world) = cx.world() {
                cx.fire(reported(&action, &world));
            }
        }
        Job::Tick => cx.tick(),
        Job::Wake(wake) => cx.wake(wake),
        Job::Arrive(mail) => cx.arrive(&mail),
        Job::Ended { run, wake } => cx.ended(&run, wake),
        Job::Fire(fired) => cx.fire(fired),
    }
    cx.resume();
    cx.arm();
}

/// The events a change the user made raises.
fn reported(action: &Action, world: &World) -> Vec<Fired> {
    let event = match action {
        Action::Move(thread, MoveTo::Folder(folder)) => world
            .folders
            .iter()
            .find(|f| f.id == *folder)
            .map(|f| Event::Sorted {
                thread: thread.clone(),
                into: f.id.clone(),
            }),
        Action::Star(message, true) => world
            .message(message)
            .map(|(t, _)| Event::Starred(t.id.clone())),
        _ => None,
    };
    event.into_iter().map(Fired::outside).collect()
}

/// What the user is told when the Backend gives up on a request.
fn timed_out(request: &Request) -> &'static str {
    match request {
        Request::SignIn => "The sign-in did not finish in time. Try again",
        Request::Summary(_) => "Scribe did not answer in time",
        Request::Reply(_) => "The model did not answer in time",
        Request::DryRun => "The dry run did not finish in time",
        Request::Export => "The export did not finish in time",
    }
}

struct Cx<'a> {
    shared: &'a Arc<Mutex<Jobs>>,
    replica: &'a Link,
    jobs: &'a mut Jobs,
    /// The replica as last read; a delivery makes it stale.
    world: Option<Rc<World>>,
}

impl Cx<'_> {
    fn play(&self, run: Option<String>, steps: VecDeque<(u64, Step)>) {
        play(self.shared.clone(), self.replica.clone(), run, steps);
    }

    /// The replica, read once until the next delivery.
    fn world(&mut self) -> Option<Rc<World>> {
        if self.world.is_none() {
            self.world = self.replica.world().map(Rc::new);
        }
        self.world.clone()
    }

    fn deliver(&mut self, update: Update) {
        self.replica.deliver(update);
        self.world = None;
    }

    /// The Backend started: the sample's clock counts from now, and it picks up what was left.
    fn start(&mut self) {
        let Some(world) = self.world() else {
            return;
        };
        self.jobs.approaching_from = world.now;
        self.jobs.delivering_from = world.now;
        self.jobs.paused = world.settings.agents_paused;
        self.pick_up(&world);
        self.tick();
    }

    /// The first sync, once signed in: the sample world as of now.
    fn sync(&mut self) {
        let Some(world) = self.world() else {
            return;
        };
        if world.user.is_none() {
            return;
        }
        self.deliver(Update::Snapshot(sample::snapshot(world.now)));
        self.deliver(Update::SnoozeTimes(policy::snooze_times(world.now)));
        self.jobs.approaching_from = world.now;
        self.jobs.delivering_from = world.now;
        self.jobs.paused = false;
        self.jobs.parked.clear();
        if let Some(synced) = self.world() {
            self.pick_up(&synced);
        }
    }

    /// Runs left unfinished: a chat turn is answered from where it was, any other run failed.
    fn pick_up(&mut self, world: &World) {
        for agent in &world.agents {
            for turn in agent
                .turns
                .iter()
                .filter(|t| matches!(t.state, TurnState::Running(_)))
            {
                let running = agent.runs.iter().find(|r| {
                    matches!(r.outcome, Outcome::Running | Outcome::Paused)
                        && r.cause == Cause::Message(turn.id.clone())
                });
                let wake = Wake::new(&agent.id, Cause::Message(turn.id.clone()));
                match running {
                    Some(r) => {
                        if r.outcome == Outcome::Paused {
                            self.deliver(Update::RunResumed(r.id.clone()));
                        }
                        self.continue_run(&r.id, wake);
                    }
                    None => self.wake(wake),
                }
            }
            for run in agent.runs.iter().filter(|r| {
                matches!(r.outcome, Outcome::Running | Outcome::Paused)
                    && !matches!(r.cause, Cause::Message(_))
            }) {
                self.deliver(Update::RunFinished {
                    run: run.id.clone(),
                    outcome: Outcome::Failed(
                        "EveryMail closed during this run. Nothing more was done".into(),
                    ),
                    summary: "Stopped when EveryMail closed".into(),
                    seconds: run.seconds.unwrap_or(world.now - run.started_at),
                    tokens: run.tokens,
                });
            }
        }
    }

    /// Waits on a request until it is answered, stopped or runs out of time.
    fn wait_on(&mut self, request: Request, now: i64) {
        self.jobs.pending.retain(|(r, _)| *r != request);
        self.jobs.pending.push((request, now + REQUEST_SECONDS));
    }

    fn work(&mut self, work: Work) {
        let Some(world) = self.world() else {
            return;
        };
        let now = world.now;
        match work {
            Work::Run(agent) => self.wake(Wake::new(&agent, Cause::RunNow)),
            Work::Chat { agent, turn } => self.wake(Wake::new(&agent, Cause::Message(turn))),
            Work::SignIn(provider) => {
                self.wait_on(Request::SignIn, now);
                let mut steps = updates(script::script(&Work::SignIn(provider), &world));
                steps.push_back((0, Step::Then(Job::Sync)));
                self.play(None, steps);
            }
            Work::Sync => self.sync(),
            Work::Connect { .. } => {
                let mut steps = updates(script::script(&work, &world));
                let added = steps.iter().find_map(|(_, s)| match s {
                    Step::Update(Update::AccountAdded(a)) => Some((a.id.clone(), a.name.clone())),
                    _ => None,
                });
                if let Some((account, name)) = added {
                    steps.push_back((
                        0,
                        Step::Then(Job::Wake(Wake::new("gatekeeper", Cause::Connected(name)))),
                    ));
                    steps.push_back((
                        WELCOME_MS,
                        Step::Then(Job::Arrive(script::welcome(
                            &account,
                            now + (WELCOME_MS / 1000) as i64,
                        ))),
                    ));
                }
                self.play(None, steps);
            }
            Work::Resolve { ref approval, .. } => {
                let mut steps = updates(script::script(&work, &world));
                if let Some(a) = world.approval(approval) {
                    steps.push_back((0, Step::Then(Job::Fire(vec![resolved(a)]))));
                }
                self.play(None, steps);
            }
            Work::Send { draft, at } => {
                let account = world.draft(&draft).and_then(|d| world.account(&d.account));
                let answer = match (account, policy::send(&at, now)) {
                    (Some(a), _)
                        if matches!(a.state, crate::world::AccountState::SignInExpired(_)) =>
                    {
                        Err(format!("Sign in to {} again first", a.name))
                    }
                    (_, Ok(state)) => Ok(state),
                    (_, Err(reason)) => Err(reason.into()),
                };
                self.deliver(match answer {
                    Ok(state) => Update::Outbox { draft, state },
                    Err(reason) => Update::Refused {
                        target: Target::Draft(draft),
                        reason,
                    },
                });
            }
            Work::CancelSend(draft) => {
                let held = world
                    .draft(&draft)
                    .and_then(|d| d.send.as_ref())
                    .is_some_and(SendState::held);
                self.deliver(if held {
                    Update::SendCancelled(draft)
                } else {
                    Update::Refused {
                        target: Target::Draft(draft),
                        reason: "It is already on its way".into(),
                    }
                });
            }
            Work::Snooze { thread, until } => self.deliver(Update::Snoozed {
                thread,
                until: policy::snooze_until(until, now),
            }),
            Work::Summarize(ref thread) => {
                self.wait_on(Request::Summary(thread.clone()), now);
                self.play(None, updates(script::script(&work, &world)));
            }
            Work::WriteReply { ref thread, .. } => {
                self.wait_on(Request::Reply(thread.clone()), now);
                self.play(None, updates(script::script(&work, &world)));
            }
            Work::DryRun => {
                self.wait_on(Request::DryRun, now);
                self.play(None, updates(script::script(&work, &world)));
            }
            Work::Export => {
                self.wait_on(Request::Export, now);
                self.play(None, updates(script::script(&work, &world)));
            }
            Work::Cancel(request) => self.jobs.pending.retain(|(r, _)| *r != request),
            Work::Permission {
                owner,
                action,
                level,
            } => self.deliver(policy::permission(owner, &action, level, &world)),
            Work::Accounts { owner, accounts } => {
                for update in policy::accounts(owner, accounts, &world) {
                    self.deliver(update);
                }
            }
            Work::AddTopic(topic) => self.deliver(policy::topic(&topic, &world.news_topics)),
            Work::Interval(seconds) => self.deliver(policy::interval(seconds)),
            Work::SetKey(provider, key) => {
                self.deliver(policy::key(provider, Some(&key), &world.settings.keys));
            }
            Work::RemoveKey(provider) => {
                self.deliver(policy::key(provider, None, &world.settings.keys));
            }
            Work::Budget(tokens) => self.deliver(policy::budget(tokens)),
            Work::Pause(paused) => self.pause(paused),
            Work::Reconnect(_) | Work::DeleteUser => {
                self.play(None, updates(script::script(&work, &world)));
            }
        }
    }

    /// Pausing holds each run after its current step; resuming lets the held runs go on.
    fn pause(&mut self, paused: bool) {
        self.jobs.paused = paused;
        self.deliver(Update::AgentsPaused(paused));
        if !paused {
            for (run, steps) in std::mem::take(&mut self.jobs.parked) {
                self.deliver(Update::RunResumed(run.clone()));
                self.play(Some(run), steps);
            }
        }
    }

    /// What came due by now: the sample's mail, schedules, events coming up, approvals that ran
    /// out, sends and snoozes, quarantined copies to drop, the snooze menu's times and requests
    /// that ran out of time.
    fn tick(&mut self) {
        self.jobs.armed = None;
        let Some(world) = self.world() else {
            return;
        };
        let now = world.now;
        let mail = pipeline::due(&world, self.jobs.delivering_from);
        for m in &mail {
            self.arrive(m);
        }
        let mut checked: Vec<&str> = mail.iter().map(|m| m.account.as_str()).collect();
        checked.sort_unstable();
        checked.dedup();
        for account in checked {
            self.deliver(Update::Fetched(account.into()));
        }
        for agent in world.agents.iter().filter(|a| a.active) {
            let Some(slot) = agent.next_wake().filter(|at| *at <= now) else {
                continue;
            };
            self.deliver(Update::Woke {
                agent: agent.id.clone(),
                at: now,
            });
            let label = agent.schedule.slot_label(slot);
            self.wake(Wake::new(&agent.id, Cause::Schedule(label)));
        }
        let fired = matcher::approaching(&world, self.jobs.approaching_from);
        self.jobs.approaching_from = now;
        self.fire(fired);
        for approval in matcher::expired(&world) {
            self.deliver(Update::Resolved {
                approval: approval.id.clone(),
                status: ApprovalStatus::Declined("Nobody answered for 7 days".into()),
            });
            let mut steps = updates(script::resumed(approval, Answer::Expired, &world));
            steps.push_back((0, Step::Then(Job::Fire(vec![resolved(approval)]))));
            self.play(None, steps);
        }
        for draft in world
            .drafts
            .iter()
            .filter(|d| d.send.as_ref().and_then(due).is_some_and(|at| at <= now))
        {
            self.deliver(Update::Outbox {
                draft: draft.id.clone(),
                state: SendState::Sending,
            });
            self.send(&draft.id);
        }
        for thread in world
            .threads
            .iter()
            .filter(|t| t.snoozed_until.is_some_and(|until| until <= now))
        {
            self.deliver(Update::SnoozeEnded(thread.id.clone()));
        }
        for q in world.quarantine.iter().filter(|q| q.drops_at <= now) {
            self.deliver(Update::QuarantineDropped(q.id.clone()));
        }
        let times = policy::snooze_times(now);
        if world.user.is_some() && world.snoozes != times {
            self.deliver(Update::SnoozeTimes(times));
        }
        let (late, pending): (Vec<_>, Vec<_>) = std::mem::take(&mut self.jobs.pending)
            .into_iter()
            .partition(|(_, deadline)| *deadline <= now);
        self.jobs.pending = pending;
        for (request, _) in late {
            let reason = timed_out(&request).into();
            self.deliver(Update::Failed { request, reason });
        }
    }

    /// A send comes due: it goes out, and the approval it answers resolves.
    fn send(&mut self, draft: &str) {
        let Some(world) = self.world() else {
            return;
        };
        let mut steps = updates(script::send(draft, &world));
        if let Some(a) = world
            .draft(draft)
            .and_then(|d| d.approval.as_deref())
            .and_then(|a| world.approval(a))
        {
            steps.push_back((0, Step::Then(Job::Fire(vec![resolved(a)]))));
        }
        self.play(None, steps);
    }

    /// New mail: it lands as rules, the block list and Gatekeeper say, then Gatekeeper runs on it
    /// and its events fire when it has finished, or at once without it. Mail already in, or in
    /// Quarantine, does not come again.
    fn arrive(&mut self, mail: &Incoming) {
        let Some(world) = self.world() else {
            return;
        };
        let quarantined = format!("q-{}", mail.thread);
        if world.thread(&mail.thread).is_some()
            || world.quarantine.iter().any(|q| q.id == quarantined)
        {
            return;
        }
        let arrival = pipeline::arrive(mail, &world, false);
        self.deliver(arrival.update);
        if arrival.gatekeeper {
            let mut wake = Wake::new("gatekeeper", Cause::Mail(Ref::Thread(mail.thread.clone())));
            wake.then = arrival.then;
            self.wake(wake);
        } else {
            self.fire(arrival.then);
        }
    }

    fn fire(&mut self, fired: Vec<Fired>) {
        if fired.is_empty() {
            return;
        }
        let Some(world) = self.world() else {
            return;
        };
        for wake in matcher::wakes(&fired, &world) {
            self.wake(wake);
        }
    }

    /// Starts the run if the gate lets it, holds it, or says why not.
    fn wake(&mut self, wake: Wake) {
        let Some(world) = self.world() else {
            return;
        };
        let Some(agent) = world.agent(&wake.agent) else {
            return;
        };
        let refused = match matcher::gate(agent, &wake.cause, &world) {
            Gate::Start => {
                self.run(wake);
                return;
            }
            Gate::Wait(hold) => match (&wake.cause, hold) {
                (Cause::RunNow, hold) => matcher::hold_text(hold, agent),
                // A schedule that comes due while its last run is still going is skipped.
                (Cause::Schedule(_), Hold::Running) => return,
                _ => {
                    self.jobs.waiting.push(wake);
                    return;
                }
            },
            Gate::Refuse(reason) => reason,
        };
        match &wake.cause {
            Cause::RunNow => self.deliver(Update::Refused {
                target: Target::Agent(wake.agent.clone()),
                reason: refused,
            }),
            Cause::Message(turn) => self.deliver(Update::Turn {
                turn: turn.clone(),
                state: TurnState::Stopped(refused),
            }),
            // Woken but not run: the log says why, and what Gatekeeper's run would fire fires.
            _ if agent.schedule != Schedule::OnMessage => {
                let run = self.next_run();
                self.deliver(Update::RunStarted {
                    agent: wake.agent.clone(),
                    run: run.clone(),
                    cause: wake.cause.clone(),
                });
                self.deliver(Update::RunFinished {
                    run,
                    outcome: Outcome::Stopped(refused),
                    summary: "Not run".into(),
                    seconds: 0,
                    tokens: 0,
                });
                self.fire(wake.then);
            }
            _ => self.fire(wake.then),
        }
    }

    fn next_run(&mut self) -> String {
        self.jobs.serial += 1;
        format!("run-{}", self.jobs.serial)
    }

    fn run(&mut self, wake: Wake) {
        let run = self.next_run();
        self.deliver(Update::RunStarted {
            agent: wake.agent.clone(),
            run: run.clone(),
            cause: wake.cause.clone(),
        });
        self.continue_run(&run, wake);
    }

    fn continue_run(&mut self, run: &str, mut wake: Wake) {
        let Some(world) = self.world() else {
            return;
        };
        let Some(agent) = world.agent(&wake.agent) else {
            return;
        };
        let script = script::run(agent, &wake, run, &world);
        // Mail it files is mail moved into a folder: the folder's watchers wake, never itself.
        for (_, update) in &script {
            if let Update::Filed { thread, folder } = update
                && let Some(f) = world.folders.iter().find(|f| f.id == *folder)
            {
                wake.then.push(Fired {
                    event: Event::Sorted {
                        thread: thread.clone(),
                        into: f.id.clone(),
                    },
                    by: Some(agent.id.clone()),
                    depth: wake.depth,
                });
            }
        }
        let mut steps = updates(script);
        steps.push_back((
            0,
            Step::Then(Job::Ended {
                run: run.into(),
                wake,
            }),
        ));
        self.play(Some(run.into()), steps);
    }

    /// A run ended: what it was to fire fires, and if it finished, the agents waiting for it wake
    /// with its deliverable.
    fn ended(&mut self, run: &str, wake: Wake) {
        let Some(world) = self.world() else {
            return;
        };
        let mut fired = wake.then;
        if let Some((agent, r)) = world.run(run)
            && matches!(r.outcome, Outcome::Done | Outcome::Waiting)
        {
            let mut refs: Vec<Ref> = Vec::new();
            for r in r.steps.iter().flat_map(|s| &s.refs) {
                if !refs.contains(r) {
                    refs.push(r.clone());
                }
            }
            fired.push(Fired {
                event: Event::Finished {
                    agent: agent.id.clone(),
                    run: run.into(),
                    refs,
                },
                by: Some(agent.id.clone()),
                depth: wake.depth,
            });
        }
        self.fire(fired);
    }

    /// Wakes held back start once nothing holds them.
    fn resume(&mut self) {
        let waiting = std::mem::take(&mut self.jobs.waiting);
        for wake in waiting {
            self.wake(wake);
        }
    }

    /// Sets the timer for the first thing to come due.
    fn arm(&mut self) {
        let Some(world) = self.world() else {
            return;
        };
        let now = world.now;
        let next = [
            pipeline::next(&world, self.jobs.delivering_from),
            world
                .agents
                .iter()
                .filter(|a| a.active)
                .filter_map(|a| a.next_wake())
                .min(),
            matcher::next_approaching(&world),
            matcher::next_expiry(&world),
            world
                .drafts
                .iter()
                .filter_map(|d| d.send.as_ref().and_then(due))
                .min(),
            world.threads.iter().filter_map(|t| t.snoozed_until).min(),
            world.quarantine.iter().map(|q| q.drops_at).min(),
            world
                .user
                .is_some()
                .then(|| policy::snooze_times_change(now)),
            self.jobs.pending.iter().map(|(_, at)| *at).min(),
        ]
        .into_iter()
        .flatten()
        .min()
        .map(|at| at.max(now));
        let Some(at) = next else {
            return;
        };
        if self.jobs.armed.is_some_and(|armed| armed <= at) {
            return;
        }
        self.jobs.armed = Some(at);
        let ms = u64::try_from(at - now).unwrap_or(0) * 1000 + 50;
        self.play(None, VecDeque::from([(ms, Step::Then(Job::Tick))]));
    }
}

/// When a held send goes.
fn due(send: &SendState) -> Option<i64> {
    match send {
        SendState::Undoable(at) | SendState::Scheduled(at) => Some(*at),
        SendState::Sending | SendState::Failed(_) => None,
    }
}

/// The user answered (or let run out) an approval of this agent.
fn resolved(approval: &crate::world::Approval) -> Fired {
    Fired::outside(Event::Resolved {
        approval: approval.id.clone(),
        agent: approval.agent.clone(),
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{date::Date, store::Store, time, world::Identity};

    /// The instant the sample was written as of: Wednesday, November 11 2026, 9:50.
    pub const SAMPLE_NOW: i64 = 1_794_390_600;

    /// The sample world as first synced, signed in, at the sample's now.
    pub fn world() -> (Store, World) {
        let store = Store::open(None).expect("an in-memory database");
        for update in [
            Update::SignedIn {
                user_name: "Ali".into(),
                provider: Identity::Google,
            },
            Update::Snapshot(sample::snapshot(SAMPLE_NOW)),
        ] {
            store.apply(&update, SAMPLE_NOW).expect("the sample");
        }
        let world = store.load(SAMPLE_NOW, None).expect("the sample world");
        (store, world)
    }

    #[test]
    fn schedules_wake_at_their_next_slot_once() {
        let (_, w) = world();
        let at = |day: &str, minute: u32| time::at(Date::parse(day).unwrap(), minute);
        let next = |id: &str| w.agent(id).unwrap().next_wake();
        assert_eq!(next("news-bearer"), Some(at("2026-11-11", 18 * 60)));
        assert_eq!(next("follow-up-nudger"), Some(at("2026-11-12", 9 * 60)));
        assert_eq!(next("job-tracker"), Some(at("2026-11-11", 10 * 60 + 45)));
        assert_eq!(next("receipts-clerk"), None);
        // Weekdays skip the weekend: Friday 9:00, then Monday 9:00.
        let mut nudger = w.agent("follow-up-nudger").unwrap().clone();
        nudger.woken_at = at("2026-11-13", 9 * 60);
        assert_eq!(nudger.next_wake(), Some(at("2026-11-16", 9 * 60)));
        // Days missed come due once: the first slot after it last woke.
        nudger.woken_at = at("2026-11-02", 10 * 60);
        assert_eq!(nudger.next_wake(), Some(at("2026-11-03", 9 * 60)));
    }

    /// A replica that keeps what it is sent and holds no world.
    #[derive(Default)]
    struct Kept(Mutex<Vec<Update>>);

    impl crate::backend::Replica for Kept {
        fn deliver(&self, update: Update) {
            self.0.lock().unwrap().push(update);
        }
        fn world(&self) -> Option<World> {
            None
        }
    }

    fn jobs() -> Arc<Mutex<Jobs>> {
        Demo::new().0
    }

    #[test]
    fn only_answers_waited_on_go_out_and_the_last_ends_the_wait() {
        let shared = jobs();
        let export = |percent| Update::ExportProgress(percent);
        let ready = Update::ExportReady {
            text: "Ready".into(),
            file: "everymail.zip".into(),
        };
        assert!(!answers(&shared, &export(10)), "nobody waits on it");
        shared.lock().unwrap().pending.push((Request::Export, 60));
        assert!(answers(&shared, &export(10)));
        assert!(answers(&shared, &ready));
        assert!(!answers(&shared, &ready), "answered once");
        assert!(
            answers(&shared, &Update::AgentsPaused(true)),
            "what nobody requested always goes out"
        );
    }

    #[test]
    fn resuming_lets_held_runs_go_on() {
        let shared = jobs();
        let kept = Arc::new(Kept::default());
        let replica: Link = kept.clone();
        let mut j = shared.lock().unwrap();
        j.paused = true;
        // Far off, so the step does not land while the test reads what was sent.
        j.parked.push((
            "run-1".into(),
            VecDeque::from([(600_000, Step::Update(Update::AgentsPaused(true)))]),
        ));
        let mut cx = Cx {
            shared: &shared,
            replica: &replica,
            jobs: &mut j,
            world: None,
        };
        cx.pause(true);
        assert_eq!(cx.jobs.parked.len(), 1, "pausing again holds them");
        cx.pause(false);
        assert!(!cx.jobs.paused && cx.jobs.parked.is_empty());
        assert_eq!(
            *kept.0.lock().unwrap(),
            vec![
                Update::AgentsPaused(true),
                Update::AgentsPaused(false),
                Update::RunResumed("run-1".into()),
            ]
        );
    }
}
