//! What the stand-in Backend answers to each piece of work and what each agent does when it runs:
//! pure functions from the work and the data to the updates and the delay before each, in
//! milliseconds after the one before.

use super::{
    matcher::{Event, Wake, code_stage},
    pipeline::{self, Incoming, Kind},
    policy::quarantine_ends,
};
use crate::{
    backend::{
        Decision, DryRunLine, NewAccount, NewMessage, NewQuarantined, Request, Update, Work,
    },
    proto::ui::v1::{InboxCategory, MailProvider, PermissionLevel},
    time,
    world::{
        Agent, Approval, ApprovalAction, ApprovalStatus, Cause, Edition, Folder, FolderKind, Line,
        Outcome, Place, Ref, ResultItem, SendState, Step, StepPermission, Thread, TurnState, World,
        category_name, paragraphs,
    },
};

/// A beat between stages, so each state shows.
const BEAT: u64 = 900;

pub fn script(work: &Work, world: &World) -> Vec<(u64, Update)> {
    match work {
        Work::SignIn(provider) => vec![(
            1_500,
            Update::SignedIn {
                user_name: "Ali".into(),
                provider: *provider,
            },
        )],
        Work::Connect {
            provider,
            address,
            password,
        } => connect(*provider, address, password, world),
        Work::Reconnect(account) => vec![(1_500, Update::AccountSynced(account.clone()))],
        Work::Summarize(thread) => vec![(
            1_800,
            match world.thread(thread) {
                Some(t) => Update::Summary {
                    thread: thread.clone(),
                    text: summary(t),
                },
                None => Update::Failed {
                    request: Request::Summary(thread.clone()),
                    reason: "That thread is gone".into(),
                },
            },
        )],
        Work::WriteReply {
            thread,
            instruction,
        } => vec![(
            1_600,
            match world.thread(thread) {
                Some(t) => Update::Reply {
                    thread: thread.clone(),
                    text: reply(t, instruction, world),
                },
                None => Update::Failed {
                    request: Request::Reply(thread.clone()),
                    reason: "That thread is gone".into(),
                },
            },
        )],
        Work::Resolve { approval, approved } => world
            .approval(approval)
            .map(|a| {
                let answer = if *approved {
                    Answer::Approved
                } else {
                    Answer::Declined
                };
                resolve(a, answer, world)
            })
            .unwrap_or_default(),
        Work::DryRun => vec![(1_800, Update::DryRun(dry_run(world)))],
        Work::Export => {
            let mut steps: Vec<(u64, Update)> = [20, 45, 70, 95]
                .into_iter()
                .map(|percent| (600, Update::ExportProgress(percent)))
                .collect();
            let messages: usize = world.threads.iter().map(|t| t.messages.len()).sum();
            steps.push((
                500,
                Update::ExportReady {
                    text: format!(
                        "Your export is ready: {messages} messages, {} events, {} agents",
                        world.events.len(),
                        world.agents.len()
                    ),
                    file: format!("everymail-export-{}.zip · 48 MB", world.today().id()),
                },
            ));
            steps
        }
        Work::DeleteUser => vec![(1_500, Update::UserDeleted)],
        // Runs start as wakes, through the gate; the rest the Backend answers at once.
        Work::Run(_)
        | Work::Chat { .. }
        | Work::Sync
        | Work::Send { .. }
        | Work::CancelSend(_)
        | Work::Snooze { .. }
        | Work::Permission { .. }
        | Work::Accounts { .. }
        | Work::AddTopic(_)
        | Work::Interval(_)
        | Work::SetKey(..)
        | Work::RemoveKey(_)
        | Work::Budget(_)
        | Work::Pause(_)
        | Work::Cancel(_) => Vec::new(),
    }
}

/// New mail to an account just connected, a little after its first sync.
pub fn welcome(account: &str, at: i64) -> Incoming {
    Incoming {
        thread: format!("{account}-new"),
        account: account.into(),
        sender: "Dana Whitfield".into(),
        address: "dana@whitfield.studio".into(),
        subject: "Coffee next week?".into(),
        body:
            "Hey Ali,\n\nI am in town next week. Coffee on Tuesday or Wednesday afternoon?\n\nDana"
                .into(),
        kind: Kind::Person,
        at,
    }
}

fn first_name(name: &str) -> &str {
    name.split_whitespace().next().unwrap_or(name)
}

fn connect(
    provider: MailProvider,
    address: &str,
    password: &str,
    world: &World,
) -> Vec<(u64, Update)> {
    let name = crate::world::mail_provider_name(provider);
    let (provider_name, oauth_address) = match provider {
        MailProvider::Gmail => (name.to_string(), "ali.hassan.work@gmail.com"),
        MailProvider::Outlook => (name.to_string(), "ali.hassan@outlook.com"),
        MailProvider::Icloud
        | MailProvider::Yahoo
        | MailProvider::Fastmail
        | MailProvider::OtherImap => (format!("{name} over IMAP"), ""),
        MailProvider::Unspecified => {
            return vec![(0, Update::ConnectRejected("Pick a provider".into()))];
        }
    };
    let imap = oauth_address.is_empty();
    // App passwords are 16 letters; spaces and dashes between groups are fine.
    let letters = password
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .count();
    if imap && letters != 16 {
        return vec![(
            1_500,
            Update::ConnectRejected(format!(
                "{name} did not accept that app password. Make a new one and paste it whole"
            )),
        )];
    }
    let address = if imap { address } else { oauth_address };
    if world.accounts.iter().any(|a| a.address == address) {
        return vec![(
            1_200,
            Update::ConnectRejected(format!("{address} is connected already")),
        )];
    }
    let n = (1..)
        .find(|n| world.account(&format!("account-{n}")).is_none())
        .unwrap_or(1);
    let id = format!("account-{n}");
    let taken = |name: &str| world.accounts.iter().any(|a| a.name == name);
    let name = if taken(name) {
        format!("{name} {}", world.accounts.len() + 1)
    } else {
        name.into()
    };
    // The last 90 days, which Gatekeeper sweeps: sorted, waking nobody.
    let mails = [
        (
            "Harbour Climbing Gym",
            "hello@harbourclimbing.com",
            "Your membership renews on Dec 1",
            "Hi Ali,\n\nYour monthly membership renews on December 1. Nothing to do if you want to keep climbing with us.\n\nSee you on the wall,\nHarbour Climbing Gym",
        ),
        (
            "Juniper Library",
            "holds@juniperlibrary.org",
            "A book you reserved is ready",
            "Designing Data-Intensive Applications is waiting for you at the front desk until November 18.",
        ),
    ];
    let mut steps = vec![(
        1_200,
        Update::AccountAdded(NewAccount {
            id: id.clone(),
            name,
            address: address.into(),
            provider,
            provider_name,
        }),
    )];
    for (i, (from, address, subject, body)) in mails.iter().enumerate() {
        steps.push((
            BEAT,
            Update::SyncProgress {
                account: id.clone(),
                percent: 33 * (i as u32 + 1),
            },
        ));
        let mail = Incoming {
            thread: format!("{id}-t{i}"),
            account: id.clone(),
            sender: (*from).into(),
            address: (*address).into(),
            subject: (*subject).into(),
            body: (*body).into(),
            kind: Kind::Bulk,
            at: world.now - 3 * time::DAY * (i as i64 + 1),
        };
        steps.push((BEAT / 2, pipeline::arrive(&mail, world, true).update));
    }
    steps.push((BEAT, Update::AccountSynced(id)));
    steps
}

/// A send coming due: it goes, or bounces.
pub fn send(draft: &str, world: &World) -> Vec<(u64, Update)> {
    let Some(d) = world.draft(draft) else {
        return Vec::new();
    };
    // An address without a domain bounces.
    let bad = d.recipients.split(',').map(str::trim).find(|r| {
        r.contains('@')
            && !r
                .split('@')
                .nth(1)
                .is_some_and(|domain| domain.contains('.'))
    });
    let mut steps = vec![(
        1_000,
        match bad {
            Some(address) => Update::Outbox {
                draft: draft.into(),
                state: SendState::Failed(format!("{address} does not exist. Nothing was sent")),
            },
            None => Update::Sent {
                draft: draft.into(),
                message: format!("m-{draft}"),
                at: world.now,
            },
        },
    )];
    if bad.is_none()
        && let Some(approval) = d.approval.as_deref().and_then(|a| world.approval(a))
    {
        steps.extend(resumed(approval, Answer::Approved, world));
    }
    steps
}

/// "Prof. Okafor writes about “COSC 4456 exam room moved to FA-224”: Tomorrow’s parallel …"
fn summary(thread: &Thread) -> String {
    let latest = thread.messages.last().map_or("", |m| m.body.as_str());
    let first = paragraphs(latest)
        .into_iter()
        .find(|p| !p.trim_end().ends_with(',') && p.split_whitespace().count() > 3)
        .unwrap_or("")
        .split(". ")
        .next()
        .unwrap_or("")
        .trim_end_matches('.');
    format!(
        "{} writes about {}. {first}.",
        thread.from,
        thread.subject.to_lowercase()
    )
}

fn reply(thread: &Thread, instruction: &str, world: &World) -> String {
    let name = first_name(&thread.from);
    let (greeting, closing) = match world.settings.compose_tone.as_str() {
        "Warm" => (format!("Hi {name}!"), "Warmly,"),
        "Formal" => (format!("Dear {name},"), "Kind regards,"),
        _ => (format!("Hi {name},"), "Best,"),
    };
    let ask = instruction.trim().trim_end_matches('.');
    let body = if ask.is_empty() {
        "Thanks for the note. I will get back to you on this today".to_string()
    } else {
        let mut chars = ask.chars();
        chars
            .next()
            .map_or(String::new(), |c| c.to_uppercase().chain(chars).collect())
    };
    // The account's signature follows the reply; without one, the user's name does.
    let signed = world
        .account(&thread.account)
        .is_some_and(|a| a.signature.is_some());
    let user = if signed {
        String::new()
    } else {
        format!("\n{}", world.user.as_deref().unwrap_or("Ali"))
    };
    format!("{greeting}\n\n{body}.\n\n{closing}{user}")
}

fn step(verb: &str, what: &str, permission: StepPermission, refs: Vec<Ref>) -> Step {
    Step {
        verb: verb.into(),
        what: what.into(),
        permission,
        refs,
    }
}

/// The latest threads in the inbox.
fn latest(world: &World, n: usize) -> Vec<&Thread> {
    world
        .threads
        .iter()
        .filter(|t| t.in_inbox(world.now))
        .take(n)
        .collect()
}

fn level(agent: &Agent, action: &str) -> PermissionLevel {
    agent
        .permissions
        .iter()
        .find(|p| p.id == action)
        .map_or(PermissionLevel::Off, |p| p.level)
}

/// Makes the run's result item: (group, title, detail, references).
type Item<'a> = &'a dyn Fn(&str, String, String, Vec<Ref>) -> Update;

/// What a run leaves: its steps, what it changed, how it ended and what it cost.
struct Plan {
    steps: Vec<Step>,
    updates: Vec<Update>,
    outcome: Outcome,
    summary: String,
    tokens: i64,
}

impl Plan {
    /// A run that read and decided: the model's tokens unless it has a code stage.
    fn new(agent: &Agent, steps: Vec<Step>, updates: Vec<Update>, summary: String) -> Self {
        let tokens = if code_stage(agent) {
            0
        } else {
            3_000 + 700 * steps.len() as i64
        };
        Self {
            steps,
            updates,
            outcome: Outcome::Done,
            summary,
            tokens,
        }
    }
}

/// The thread a cause is about.
fn thread_of(cause: &Cause) -> Option<&str> {
    match cause {
        Cause::Mail(Ref::Thread(t))
        | Cause::Opened(Ref::Thread(t))
        | Cause::Handoff(_, Ref::Thread(t))
        | Cause::Sorted(_, Ref::Thread(t))
        | Cause::Flagged(Ref::Thread(t))
        | Cause::Starred(Ref::Thread(t)) => Some(t),
        _ => None,
    }
}

/// A run once started: its steps, what it leaves behind and its end. `RunStarted` is not part of
/// it: the Backend records that the moment it starts the run.
pub fn run(agent: &Agent, wake: &Wake, run: &str, world: &World) -> Vec<(u64, Update)> {
    if let Cause::Message(turn) = &wake.cause {
        return chat(agent, turn, run, world);
    }
    let item = |group: &str, title: String, detail: String, refs: Vec<Ref>| Update::Result {
        agent: agent.id.clone(),
        item: ResultItem {
            id: format!("item-{run}"),
            group: group.into(),
            at: world.now,
            title,
            detail,
            refs,
        },
    };
    let thread = thread_of(&wake.cause).and_then(|t| world.thread(t));
    let plan = match (agent.id.as_str(), &wake.cause, thread) {
        ("gatekeeper", Cause::Connected(account), _) => Plan::new(
            agent,
            vec![
                step(
                    "Swept",
                    &format!("the last 90 days of {account}"),
                    StepPermission::Read,
                    vec![],
                ),
                step(
                    "Sorted",
                    "what was there, waking no other agent",
                    StepPermission::Auto,
                    vec![],
                ),
            ],
            vec![],
            format!("Swept 90 days of {account}"),
        ),
        ("gatekeeper", _, Some(t)) => sort(agent, wake, t),
        ("security-guard", _, Some(t)) => guard(agent, t, world),
        (_, Cause::Finished(by, refs), _) => {
            let (from, summary) = world.run(by).map_or(
                ("an agent you deleted".to_string(), String::new()),
                |(a, r)| (a.name.clone(), r.summary.clone()),
            );
            Plan::new(
                agent,
                vec![step(
                    "Read",
                    &format!("what {from} handed over"),
                    StepPermission::Read,
                    refs.clone(),
                )],
                vec![item(
                    "Handovers",
                    format!("From {from}: {summary}"),
                    "Read what it handed over".into(),
                    refs.clone(),
                )],
                format!("Took over from {from}"),
            )
        }
        (_, Cause::Resolved(r @ Ref::Approval(id)), _) => {
            let (title, answer) = world.approval(id).map_or(("", "Answered"), |a| {
                let answer = match a.status {
                    ApprovalStatus::Done => "You approved it",
                    ApprovalStatus::Declined(_)
                    | ApprovalStatus::Waiting
                    | ApprovalStatus::Resolving => "It was declined",
                };
                (a.title.as_str(), answer)
            });
            Plan::new(
                agent,
                vec![step(
                    "Read",
                    "your answer",
                    StepPermission::Read,
                    vec![r.clone()],
                )],
                vec![item(
                    "Approvals",
                    title.into(),
                    answer.into(),
                    vec![r.clone()],
                )],
                "Noted your answer".into(),
            )
        }
        (_, Cause::Approaching(lead, r @ Ref::Event(id)), _) => {
            let title = world.event(id).map_or("", |e| e.title.as_str());
            Plan::new(
                agent,
                vec![step("Read", title, StepPermission::Read, vec![r.clone()])],
                vec![item(
                    "Coming up",
                    title.into(),
                    format!("Woken {}", crate::builder::lead_label(lead)),
                    vec![r.clone()],
                )],
                format!("Noted {title}"),
            )
        }
        (_, _, Some(t)) if !agent.system() => mail_run(agent, t, &item, world),
        _ => scheduled(agent, &wake.cause, run, &item, world),
    };
    let mut out: Vec<(u64, Update)> = plan
        .steps
        .into_iter()
        .map(|step| {
            (
                BEAT,
                Update::RunStep {
                    run: run.into(),
                    step,
                },
            )
        })
        .collect();
    out.extend(plan.updates.into_iter().map(|u| (BEAT / 2, u)));
    let seconds = 1 + out.len() as i64;
    out.push((
        BEAT / 2,
        Update::RunFinished {
            run: run.into(),
            outcome: plan.outcome,
            summary: plan.summary,
            seconds,
            tokens: plan.tokens,
        },
    ));
    out
}

/// Gatekeeper on new mail: it was sorted on arrival; this is its record, or the hand-over of a
/// likely scam to Security Guard.
fn sort(agent: &Agent, wake: &Wake, t: &Thread) -> Plan {
    let refs = vec![Ref::Thread(t.id.clone())];
    let scam = wake
        .then
        .iter()
        .any(|f| matches!(&f.event, Event::Handoff { to, .. } if to == "security-guard"));
    let flagged = wake
        .then
        .iter()
        .any(|f| matches!(f.event, Event::Flagged(_)));
    let into = category_name(t.category);
    let mut steps = vec![step(
        "Read",
        "the headers and the provider’s verdict",
        StepPermission::Read,
        refs.clone(),
    )];
    let summary = if scam {
        steps.push(step(
            "Handed",
            "a likely scam to Security Guard",
            StepPermission::Auto,
            refs,
        ));
        "Handed 1 to Security Guard".to_string()
    } else {
        steps.push(step(
            "Sorted",
            &format!("into {into}"),
            StepPermission::Auto,
            refs.clone(),
        ));
        if flagged {
            steps.push(step("Flagged", "it for you", StepPermission::Auto, refs));
            format!("Sorted 1 into {into} · flagged")
        } else {
            format!("Sorted 1 into {into}")
        }
    };
    Plan::new(agent, steps, vec![], summary)
}

/// Security Guard on mail Gatekeeper handed over: into Quarantine, the sender blocked.
fn guard(agent: &Agent, t: &Thread, world: &World) -> Plan {
    let refs = vec![Ref::Thread(t.id.clone())];
    let quarantined = format!("q-{}", t.id);
    Plan::new(
        agent,
        vec![
            step("Read", &t.subject, StepPermission::Read, refs),
            step(
                "Checked",
                "where its links go and who sent it",
                StepPermission::Read,
                vec![],
            ),
            step(
                "Moved",
                "it to Quarantine",
                StepPermission::Auto,
                vec![Ref::Quarantined(quarantined.clone())],
            ),
            step("Blocked", &t.from, StepPermission::Auto, vec![]),
        ],
        vec![Update::Quarantined(NewQuarantined {
            id: quarantined,
            account: t.account.clone(),
            sender: t.from.clone(),
            received_at: t.latest_at,
            subject: t.subject.clone(),
            body: t.messages.first().map_or(String::new(), |m| m.body.clone()),
            reason: "Scam".into(),
            caught_by: "Security Guard".into(),
            why: "It asks you to follow a link to a site that is not the sender’s".into(),
            outcome: "Reported as spam and the sender blocked".into(),
            thread: Some(t.id.clone()),
            drops_at: quarantine_ends(t.latest_at, world),
            block: Some((t.from.clone(), "Security Guard".into())),
        })],
        "Quarantined 1 · sender blocked and reported".into(),
    )
}

/// The folder an agent files this mail in: one of the user's folders its instructions name, when
/// the mail is about what the folder holds and the agent may move mail.
fn files_into<'a>(agent: &Agent, t: &Thread, world: &'a World) -> Option<&'a Folder> {
    let may_move = level(agent, "move-folders") == PermissionLevel::Auto;
    let instructions = agent.instructions.to_lowercase();
    let about = format!("{} {}", t.subject, t.from).to_lowercase();
    world.folders.iter().find(|f| {
        let name = f.name.to_lowercase();
        may_move
            && f.kind == FolderKind::User
            && instructions.contains(&name)
            && about.contains(name.trim_end_matches('s'))
            && !t.folders.contains(&f.id)
    })
}

/// An agent the user made, woken by one mail: it files it, logs it in a sheet, or reads it.
fn mail_run(agent: &Agent, t: &Thread, item: Item, world: &World) -> Plan {
    let refs = vec![Ref::Thread(t.id.clone())];
    let sheet = agent
        .permissions
        .iter()
        .any(|p| p.level == PermissionLevel::Auto && p.id.contains("sheet"));
    let files = world.folders.iter().any(|f| {
        f.kind == FolderKind::User
            && agent
                .instructions
                .to_lowercase()
                .contains(&f.name.to_lowercase())
    });
    let mut steps = vec![step("Read", &t.subject, StepPermission::Read, refs.clone())];
    let mut updates = Vec::new();
    let (title, summary) = match files_into(agent, t, world) {
        Some(folder) => {
            steps.push(step(
                "Moved",
                &format!("to {}", folder.name),
                StepPermission::Auto,
                refs.clone(),
            ));
            updates.push(Update::Filed {
                thread: t.id.clone(),
                folder: folder.id.clone(),
            });
            (
                format!("Moved to {}", folder.name),
                format!("Filed 1 in {}", folder.name),
            )
        }
        None if files => ("Nothing to file".into(), "Nothing to file".into()),
        None => ("Read".into(), "Read 1".into()),
    };
    if sheet && summary != "Nothing to file" {
        steps.push(step(
            "Updated",
            "your sheet",
            StepPermission::Auto,
            refs.clone(),
        ));
    }
    updates.push(item(
        "Latest mail",
        format!("{} · {}", t.subject, t.from),
        title,
        refs,
    ));
    Plan::new(agent, steps, updates, summary)
}

/// Run now, or a schedule came due.
fn scheduled(agent: &Agent, cause: &Cause, run: &str, item: Item, world: &World) -> Plan {
    let thread_refs = |threads: &[&Thread]| -> Vec<Ref> {
        threads.iter().map(|t| Ref::Thread(t.id.clone())).collect()
    };
    let recent = latest(world, 3);
    match agent.id.as_str() {
        "gatekeeper" => Plan::new(
            agent,
            vec![
                step(
                    "Read",
                    "3 newest mails",
                    StepPermission::Read,
                    thread_refs(&recent),
                ),
                step(
                    "Sorted",
                    "3 mails into For you and Updates",
                    StepPermission::Auto,
                    thread_refs(&recent),
                ),
            ],
            vec![item(
                "Recent decisions",
                "Sorted 3 mails".into(),
                "Nothing new needed you".into(),
                thread_refs(&recent),
            )],
            "Sorted 3".into(),
        ),
        "security-guard" => Plan::new(
            agent,
            vec![
                step(
                    "Read",
                    "3 newest mails",
                    StepPermission::Read,
                    thread_refs(&recent),
                ),
                step("Checked", "links and senders", StepPermission::Read, vec![]),
            ],
            vec![item(
                "Checked",
                "Nothing suspicious".into(),
                "3 mails checked, every link went where it said".into(),
                thread_refs(&recent),
            )],
            "Checked 3 · nothing caught".into(),
        ),
        "scribe" => {
            let unsummarised: Vec<&Thread> = world
                .threads
                .iter()
                .filter(|t| t.summary.is_none())
                .take(2)
                .collect();
            let mut updates: Vec<Update> = unsummarised
                .iter()
                .map(|t| Update::Summary {
                    thread: t.id.clone(),
                    text: summary(t),
                })
                .collect();
            updates.push(item(
                "Summaries",
                format!("Summarised {}", unsummarised.len()),
                "Threads you have not opened yet".into(),
                thread_refs(&unsummarised),
            ));
            Plan::new(
                agent,
                vec![step(
                    "Summarised",
                    "threads without a summary",
                    StepPermission::Auto,
                    thread_refs(&unsummarised),
                )],
                updates,
                format!("Summarised {}", unsummarised.len()),
            )
        }
        "news-bearer" => Plan::new(
            agent,
            vec![
                step(
                    "Read",
                    "newsletters and updates",
                    StepPermission::Read,
                    vec![],
                ),
                step("Wrote", "a briefing", StepPermission::Auto, vec![]),
            ],
            vec![Update::Edition(edition(run, world))],
            "Wrote a briefing".into(),
        ),
        "follow-up-nudger" => {
            let unanswered = world
                .threads
                .iter()
                .find(|t| t.messages.last().is_some_and(|m| m.from_you) && t.place != Place::Trash);
            match unanswered {
                Some(t) => {
                    let approval = format!("approval-{run}");
                    let mut plan = Plan::new(
                        agent,
                        vec![
                            step(
                                "Found",
                                &format!("no answer from {}", t.from),
                                StepPermission::Read,
                                vec![Ref::Thread(t.id.clone())],
                            ),
                            step(
                                "Drafted",
                                &format!("a nudge to {}", t.from),
                                StepPermission::Asked,
                                vec![Ref::Approval(approval.clone())],
                            ),
                        ],
                        vec![Update::Approval(Approval {
                            id: approval,
                            agent: agent.id.clone(),
                            agent_name: agent.name.clone(),
                            created_at: world.now,
                            title: format!("Nudge {}", t.from),
                            detail: t.subject.clone(),
                            reason: "Sending always asks for this agent".into(),
                            action: ApprovalAction::Send(t.id.clone()),
                            status: ApprovalStatus::Waiting,
                            draft: format!(
                                "Hi {},\n\nJust checking in on this. Any news?\n\nThanks,\n{}",
                                first_name(&t.from),
                                world.user.as_deref().unwrap_or("Ali")
                            ),
                        })],
                        "Drafted 1 · waiting for you".into(),
                    );
                    plan.outcome = Outcome::Waiting;
                    plan
                }
                None => Plan::new(
                    agent,
                    vec![step("Read", "what you sent", StepPermission::Read, vec![])],
                    vec![],
                    "Nobody to nudge".into(),
                ),
            }
        }
        _ => {
            // A scheduled wake first checks, in code, for anything new: an idle wake costs nothing.
            let since = agent
                .runs
                .iter()
                .find(|r| r.outcome != Outcome::Running)
                .map_or(0, |r| r.started_at);
            let new: Vec<&Thread> = world
                .threads
                .iter()
                .filter(|t| t.latest_at > since && agent.accounts.contains(&t.account))
                .take(3)
                .collect();
            if matches!(cause, Cause::Schedule(_)) && new.is_empty() {
                let mut plan = Plan::new(
                    agent,
                    vec![step(
                        "Checked",
                        &format!("for mail since {}", time::long(since, world.now)),
                        StepPermission::Read,
                        vec![],
                    )],
                    vec![],
                    "Nothing new".into(),
                );
                plan.tokens = 0;
                return plan;
            }
            let read = if new.is_empty() { recent } else { new };
            let waiting = world.waiting().count();
            let unread = world.unread(None, None);
            Plan::new(
                agent,
                vec![step(
                    "Read",
                    &format!("{} mails in your accounts", read.len()),
                    StepPermission::Read,
                    thread_refs(&read),
                )],
                vec![item(
                    "Latest run",
                    format!("{unread} unread, {waiting} waiting for you"),
                    if level(agent, "update-sheets") == PermissionLevel::Auto {
                        "Added a row to your sheet".into()
                    } else {
                        "Nothing needed doing".into()
                    },
                    thread_refs(&read),
                )],
                format!("Read {}", read.len()),
            )
        }
    }
}

fn edition(run: &str, world: &World) -> Edition {
    let today = world.today();
    let calendar: Vec<(String, String, String)> = world
        .events
        .iter()
        .filter(|e| e.date == today)
        .map(|e| (time::clock(e.start), e.title.clone(), e.place.clone()))
        .collect();
    Edition {
        id: format!("edition-{run}"),
        written_at: world.now,
        byline: format!(
            "{} {} · written by News Bearer",
            crate::date::WEEKDAYS[today.weekday()],
            time::clock(time::minute(world.now))
        ),
        unread: true,
        activity: world
            .agents
            .iter()
            .filter_map(|a| a.runs.first().map(|r| (a, r)))
            .take(4)
            .map(|(a, r)| Line {
                source: a.name.clone(),
                text: r.summary.clone(),
            })
            .collect(),
        caught_mail: !world.quarantine.is_empty(),
        waiting: world
            .waiting()
            .map(|a| {
                (
                    a.id.clone(),
                    a.title.clone(),
                    format!("{} · {}", a.agent_name, a.detail),
                )
            })
            .collect(),
        newsletters: world
            .threads
            .iter()
            .filter(|t| t.category == InboxCategory::Reading && t.in_inbox(world.now))
            .map(|t| Line {
                source: t.from.clone(),
                text: t.subject.clone(),
            })
            .collect(),
        updates: world
            .threads
            .iter()
            .filter(|t| t.category == InboxCategory::Updates && t.in_inbox(world.now))
            .map(|t| Line {
                source: t.from.clone(),
                text: t.subject.clone(),
            })
            .collect(),
        calendar_title: "Today".into(),
        calendar,
        news: world
            .news_topics
            .iter()
            .map(|topic| {
                (
                    topic.clone(),
                    vec![(
                        format!("What changed in {} this week", topic.to_lowercase()),
                        "The Weekly Digest".to_string(),
                    )],
                )
            })
            .collect(),
    }
}

/// A chat turn's answer, from a small set keyed on what the user asked about.
pub fn chat(agent: &Agent, turn: &str, run: &str, world: &World) -> Vec<(u64, Update)> {
    let Some(message) = agent
        .turns
        .iter()
        .find(|t| t.id == turn)
        .map(|t| t.message.to_lowercase())
    else {
        return Vec::new();
    };
    let run = run.to_string();
    let has = |words: &[&str]| words.iter().any(|w| message.contains(w));
    let user = world.user.as_deref().unwrap_or("Ali");
    let mut updates = Vec::new();
    let read = step(
        "Read",
        "your mail and calendar",
        StepPermission::Read,
        vec![],
    );
    let (state, outcome, summary, extra_step) = if has(&[
        "reply", "send", "email", "write to", "tell",
    ]) {
        let thread = world
            .threads
            .iter()
            .filter(|t| t.place != Place::Trash)
            .find(|t| message.contains(&first_name(&t.from).to_lowercase()))
            .or(latest(world, 1).into_iter().next());
        match thread {
            Some(t) => {
                let approval = format!("approval-{turn}");
                let gmail = world
                    .account(&t.account)
                    .is_some_and(|a| a.provider == MailProvider::Gmail);
                updates.push(Update::Approval(Approval {
                    id: approval.clone(),
                    agent: agent.id.clone(),
                    agent_name: agent.name.clone(),
                    created_at: world.now,
                    title: format!("Reply to {}", t.from),
                    detail: t.subject.clone(),
                    reason: if gmail {
                        "Gmail accounts always ask before sending for now".into()
                    } else {
                        format!("Sending asks first for {}", agent.name)
                    },
                    action: ApprovalAction::Send(t.id.clone()),
                    status: ApprovalStatus::Waiting,
                    draft: format!(
                        "Hi {},\n\nThanks for your message. I will have it ready by tomorrow.\n\nBest,\n{user}",
                        first_name(&t.from)
                    ),
                }));
                (
                    TurnState::Waiting(approval.clone()),
                    Outcome::Waiting,
                    "Drafted 1 · waiting for you".to_string(),
                    Some(step(
                        "Drafted",
                        &format!("a reply to {}", t.from),
                        StepPermission::Asked,
                        vec![Ref::Approval(approval)],
                    )),
                )
            }
            None => (
                TurnState::Answered("I could not find a thread to answer.".into(), vec![]),
                Outcome::Done,
                "Answered".into(),
                None,
            ),
        }
    } else if let Some(event) = has(&["exam", "meeting", "when", "where", "calendar"])
        .then(|| best_event(&message, world))
        .flatten()
    {
        let place = if event.place.is_empty() {
            String::new()
        } else {
            format!(" in {}", event.place)
        };
        (
            TurnState::Answered(
                format!(
                    "{} is on {}, {} {} at {}{place}.",
                    event.title,
                    crate::date::WEEKDAYS[event.date.weekday()],
                    event.date.month_short(),
                    event.date.day,
                    time::clock(event.start)
                ),
                vec![Ref::Event(event.id.clone())],
            ),
            Outcome::Done,
            "Answered".into(),
            None,
        )
    } else if has(&["unread", "waiting", "today", "summary", "catch up"]) {
        let recent = latest(world, 3);
        (
            TurnState::Answered(
                format!(
                    "You have {} unread mails and {} items waiting for your approval.\n\nThe newest: {}.",
                    world.unread(None, None),
                    world.waiting().count(),
                    recent
                        .iter()
                        .map(|t| t.subject.as_str())
                        .collect::<Vec<_>>()
                        .join("; ")
                ),
                recent.iter().map(|t| Ref::Thread(t.id.clone())).collect(),
            ),
            Outcome::Done,
            "Answered".into(),
            None,
        )
    } else {
        let words: Vec<&str> = message
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 3)
            .collect();
        let found: Vec<&Thread> = world
            .threads
            .iter()
            .filter(|t| {
                let text = format!("{} {}", t.subject, t.from).to_lowercase();
                words.iter().any(|w| text.contains(w))
            })
            .take(3)
            .collect();
        let text = match found.len() {
            0 => "I could not find anything about that in your mail.".to_string(),
            n => format!(
                "I found {n} {} about that: {}.",
                if n == 1 { "thread" } else { "threads" },
                found
                    .iter()
                    .map(|t| t.subject.as_str())
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        };
        (
            TurnState::Answered(
                text,
                found.iter().map(|t| Ref::Thread(t.id.clone())).collect(),
            ),
            Outcome::Done,
            "Answered".into(),
            None,
        )
    };
    let mut out = vec![(
        BEAT,
        Update::RunStep {
            run: run.clone(),
            step: read,
        },
    )];
    if let Some(step) = extra_step {
        out.push((
            BEAT,
            Update::RunStep {
                run: run.clone(),
                step,
            },
        ));
    }
    out.extend(updates.into_iter().map(|u| (BEAT / 2, u)));
    out.push((
        BEAT / 2,
        Update::Turn {
            turn: turn.into(),
            state,
        },
    ));
    out.push((
        0,
        Update::RunFinished {
            run,
            outcome,
            summary,
            seconds: 3,
            tokens: 2_400,
        },
    ));
    out
}

/// The event whose title shares the most words with the message, the earliest on a tie.
fn best_event<'a>(message: &str, world: &'a World) -> Option<&'a crate::world::CalendarEvent> {
    let message = message.to_lowercase();
    let words: Vec<&str> = message
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 3)
        .collect();
    let score = |e: &crate::world::CalendarEvent| {
        let title = e.title.to_lowercase();
        title
            .split(|c: char| !c.is_alphanumeric())
            .filter(|t| words.contains(t))
            .count()
    };
    world
        .events
        .iter()
        .map(|e| (score(e), e))
        .filter(|(n, _)| *n > 0)
        .fold(
            None,
            |best: Option<(usize, &crate::world::CalendarEvent)>, (n, e)| match best {
                Some((m, _)) if m >= n => best,
                _ => Some((n, e)),
            },
        )
        .map(|(_, e)| e)
}

/// How an approval was answered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    Approved,
    Declined,
    /// Nobody answered for 7 days: it ends as declined.
    Expired,
}

/// The agent resumes after the user answered an approval: it resolves, a send goes out, and the
/// waiting turn and run finish.
fn resolve(approval: &Approval, answer: Answer, world: &World) -> Vec<(u64, Update)> {
    let declined = match &approval.action {
        ApprovalAction::Send(_) => "You declined. Nothing was sent",
        ApprovalAction::Unsubscribe => "Kept. You stay subscribed",
    };
    let mut out = vec![(
        BEAT,
        Update::Resolved {
            approval: approval.id.clone(),
            status: match answer {
                Answer::Approved => ApprovalStatus::Done,
                Answer::Declined | Answer::Expired => ApprovalStatus::Declined(declined.into()),
            },
        },
    )];
    if answer == Answer::Approved
        && let ApprovalAction::Send(thread) = &approval.action
    {
        out.push((
            0,
            Update::Mail {
                thread: None,
                thread_id: thread.clone(),
                message: NewMessage {
                    id: format!("m-{}", approval.id),
                    from_you: true,
                    sender: world.user.clone().unwrap_or_default(),
                    sent_at: world.now,
                    body: approval.draft.clone(),
                    attachments: Vec::new(),
                },
            },
        ));
    }
    out.extend(resumed(approval, answer, world));
    out
}

pub fn resumed(approval: &Approval, answer: Answer, world: &World) -> Vec<(u64, Update)> {
    let Some(agent) = world.agent(&approval.agent) else {
        return Vec::new();
    };
    let (answer, summary) = match (&approval.action, answer) {
        (ApprovalAction::Send(_), Answer::Approved) => {
            (format!("Sent: {}.", approval.title), "Sent 1")
        }
        (ApprovalAction::Unsubscribe, Answer::Approved) => {
            (format!("Done: {}.", approval.title), "Unsubscribed")
        }
        (_, Answer::Declined) => ("You declined, so nothing was done.".to_string(), "Declined"),
        (_, Answer::Expired) => (
            "Nobody answered for 7 days, so it ended as declined.".to_string(),
            "Expired",
        ),
    };
    let mut out: Vec<(u64, Update)> = agent
        .turns
        .iter()
        .filter(|t| t.state == TurnState::Waiting(approval.id.clone()))
        .map(|t| {
            (
                BEAT / 2,
                Update::Turn {
                    turn: t.id.clone(),
                    state: TurnState::Answered(
                        answer.clone(),
                        vec![Ref::Approval(approval.id.clone())],
                    ),
                },
            )
        })
        .collect();
    out.extend(
        agent
            .runs
            .iter()
            .filter(|r| r.outcome == Outcome::Waiting)
            .filter(|r| {
                r.steps
                    .iter()
                    .any(|s| s.refs.contains(&Ref::Approval(approval.id.clone())))
            })
            .map(|r| {
                (
                    0,
                    Update::RunFinished {
                        run: r.id.clone(),
                        outcome: Outcome::Done,
                        summary: summary.into(),
                        seconds: r.seconds.unwrap_or(world.now - r.started_at),
                        tokens: r.tokens,
                    },
                )
            }),
    );
    out
}

/// What the builder's agent would do with the newest mail: act on For you, skip the rest.
fn dry_run(world: &World) -> Vec<DryRunLine> {
    let b = &world.builder;
    let auto = b
        .permissions
        .iter()
        .find(|p| p.level == PermissionLevel::Auto && p.id != "read-label");
    let ask = b
        .permissions
        .iter()
        .find(|p| p.level == PermissionLevel::Ask);
    world
        .threads
        .iter()
        .filter(|t| t.in_inbox(world.now) && b.accounts.contains(&t.account))
        .take(5)
        .map(|t| {
            let (action, decision) = match (t.category, auto, ask) {
                (InboxCategory::ForYou, Some(p), _) => {
                    (format!("{}: {}", p.label, t.subject), Decision::Auto)
                }
                (InboxCategory::ForYou, None, Some(p)) => {
                    (format!("{}: {}", p.label, t.subject), Decision::Ask)
                }
                _ => ("Not something this agent handles".into(), Decision::Skip),
            };
            DryRunLine {
                from: t.from.clone(),
                action,
                decision,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::demo::{matcher::Fired, tests::world};

    #[test]
    fn imap_needs_a_whole_app_password() {
        let (_, w) = world();
        let rejected = connect(MailProvider::Fastmail, "ali@fastmail.com", "short", &w);
        assert!(matches!(
            rejected.as_slice(),
            [(_, Update::ConnectRejected(_))]
        ));
        let accepted = connect(
            MailProvider::Fastmail,
            "ali@fastmail.com",
            "abcd-efgh-ijkl-mnop",
            &w,
        );
        assert!(matches!(
            accepted.first(),
            Some((_, Update::AccountAdded(_)))
        ));
        assert!(matches!(
            accepted.last(),
            Some((_, Update::AccountSynced(_)))
        ));
    }

    #[test]
    fn a_chat_asking_to_reply_waits_for_approval() {
        let (store, w) = world();
        store.clear_turns("assistant").expect("no turns");
        let turn = store
            .add_turn(
                "assistant",
                "Reply to Mira that I will be there",
                "Reading",
                w.now,
            )
            .expect("a turn");
        let w = store.load(w.now, None).expect("the world");
        let script = chat(w.agent("assistant").expect("Assistant"), &turn, "run-1", &w);
        let approval = script.iter().find_map(|(_, u)| match u {
            Update::Approval(a) => Some(a),
            _ => None,
        });
        let approval = approval.expect("an approval");
        assert_eq!(approval.title, "Reply to Mira");
        assert!(script.iter().any(|(_, u)| matches!(
            u,
            Update::Turn { state: TurnState::Waiting(id), .. } if *id == approval.id
        )));
        assert!(matches!(
            script.last(),
            Some((
                _,
                Update::RunFinished {
                    outcome: Outcome::Waiting,
                    ..
                }
            ))
        ));
    }

    fn arrived(store: &crate::store::Store, now: i64, mail: &Incoming) -> World {
        let arrival = pipeline::arrive(mail, &store.load(now, None).unwrap(), false);
        store.apply(&arrival.update, now).unwrap();
        store.load(now, None).unwrap()
    }

    #[test]
    fn receipts_clerk_files_a_receipt_handed_to_it() {
        let (store, w) = world();
        let mail = Incoming {
            thread: "t-new".into(),
            account: "personal".into(),
            sender: "Ridgeway Hosting".into(),
            address: "billing@ridgewayhosting.com".into(),
            subject: "Receipt for your November invoice".into(),
            body: "Thanks".into(),
            kind: Kind::Bulk,
            at: w.now,
        };
        let w = arrived(&store, w.now, &mail);
        let clerk = w.agent("receipts-clerk").unwrap();
        let wake = Wake::new(
            "receipts-clerk",
            Cause::Handoff("your rule".into(), Ref::Thread("t-new".into())),
        );
        let script = run(clerk, &wake, "run-1", &w);
        assert!(script.iter().any(|(_, u)| matches!(
            u,
            Update::Filed { thread, folder } if thread == "t-new" && folder == "receipts"
        )));
        assert!(matches!(
            script.last(),
            Some((_, Update::RunFinished { summary, .. })) if summary == "Filed 1 in Receipts"
        ));
    }

    #[test]
    fn security_guard_quarantines_what_gatekeeper_hands_it() {
        let (store, w) = world();
        let mail = Incoming {
            thread: "t-scam".into(),
            account: "personal".into(),
            sender: "claims@prize-center-win.top".into(),
            address: "claims@prize-center-win.top".into(),
            subject: "You have won".into(),
            body: "https://prize-center-win.top".into(),
            kind: Kind::Scam,
            at: w.now,
        };
        let w = arrived(&store, w.now, &mail);
        let gatekeeper = w.agent("gatekeeper").unwrap();
        let mut sorting = Wake::new("gatekeeper", Cause::Mail(Ref::Thread("t-scam".into())));
        sorting.then = vec![Fired::outside(Event::Handoff {
            to: "security-guard".into(),
            by: "Gatekeeper".into(),
            thread: "t-scam".into(),
        })];
        let sorted = run(gatekeeper, &sorting, "run-1", &w);
        assert!(matches!(
            sorted.last(),
            Some((_, Update::RunFinished { summary, tokens: 0, .. })) if summary == "Handed 1 to Security Guard"
        ));
        let guard = w.agent("security-guard").unwrap();
        let wake = Wake::new(
            "security-guard",
            Cause::Handoff("Gatekeeper".into(), Ref::Thread("t-scam".into())),
        );
        for (_, update) in run(guard, &wake, "run-2", &w) {
            store.apply(&update, w.now).unwrap();
        }
        let after = store.load(w.now, None).unwrap();
        assert!(after.thread("t-scam").is_none());
        assert!(after.quarantine.iter().any(|q| q.id == "q-t-scam"));
        assert!(
            after
                .settings
                .blocked
                .iter()
                .any(|b| b.address == "claims@prize-center-win.top")
        );
    }

    #[test]
    fn a_question_finds_the_event_sharing_most_words() {
        let (_, w) = world();
        let exam = best_event("When and where is my COSC 4456 exam?", &w);
        assert_eq!(exam.map(|e| e.id.as_str()), Some("e-exam"));
        let lecture = best_event("When is the COSC 4456 lecture?", &w);
        assert_eq!(
            lecture.map(|e| e.id.as_str()),
            Some("e-lecture-mon"),
            "the first on a tie"
        );
    }
}
