use super::{
    Account, Agent, Approval, Author, Calendar, CalendarEvent, Chat, Compose, Connect,
    ConnectProvider, Day, Edition, Folder, How, Message, Place, Quarantined, Settings, Thread,
    World,
};
use crate::proto::ui::v1::{
    self as ui, AgentDetailPane, CalendarColor, CalendarProvider, InboxCategory, MailProvider,
    ModelProvider, NoteKind, PermissionLevel,
};

use Author::{Contact, You};
use InboxCategory::{ForYou, Reading, Updates};
use PermissionLevel::{Ask, Auto, Off};

pub fn world() -> World {
    World {
        user_name: "Ali",
        accounts: accounts(),
        folders: folders(),
        threads: threads(),
        agents: agents(),
        approvals: approvals(),
        compose: Compose {
            models: vec![
                model("Claude Sonnet", ModelProvider::Anthropic),
                model("Claude Haiku", ModelProvider::Anthropic),
                model("Gemini Pro", ModelProvider::GoogleGemini),
                model("GPT-5", ModelProvider::Openai),
                model("OpenRouter models", ModelProvider::Openrouter),
            ],
            model: "Claude Sonnet",
            tones: &["Brief", "Warm", "Formal"],
            tone: "Brief",
        },
        calendar: calendar(),
        quarantine: quarantine(),
        editions: editions(),
        briefing_unread: true,
        settings: settings(),
        builder: builder(),
        connect: connect(),
    }
}

fn accounts() -> Vec<Account> {
    use ui::{account_status::State, settings_account::Signature};
    let synced = || {
        State::Synced(ui::AccountSynced {
            label: "Synced 1 min ago".into(),
        })
    };
    vec![
        Account {
            id: "personal",
            name: "Personal",
            provider: MailProvider::Gmail,
            provider_name: "Gmail",
            status: synced(),
            signature: Signature::Text("Ali\nsent with EveryMail".into()),
        },
        Account {
            id: "laurentian",
            name: "Laurentian",
            provider: MailProvider::Outlook,
            provider_name: "Outlook",
            status: synced(),
            signature: Signature::Text("Ali\nB.Sc. Computer Science, Laurentian University".into()),
        },
        Account {
            id: "icloud",
            name: "iCloud",
            provider: MailProvider::Icloud,
            provider_name: "iCloud Mail over IMAP",
            status: State::SignInExpired(ui::AccountSignInExpired {
                label: "App password expired".into(),
            }),
            signature: Signature::None(ui::NoSignature {}),
        },
    ]
}

fn folders() -> Vec<Folder> {
    use ui::nav_folder::Kind;
    vec![
        Folder {
            id: "starred",
            name: "Starred",
            kind: Kind::Starred(ui::StarredFolder {}),
        },
        Folder {
            id: "sent",
            name: "Sent",
            kind: Kind::Sent(ui::SentFolder {}),
        },
        Folder {
            id: "archive",
            name: "Archive",
            kind: Kind::Archive(ui::ArchiveFolder {}),
        },
        Folder {
            id: "receipts",
            name: "Receipts",
            kind: Kind::User(ui::UserFolder {}),
        },
        Folder {
            id: "quarantine",
            name: "Quarantine",
            kind: Kind::Quarantine(ui::QuarantineFolder {}),
        },
    ]
}

fn message(
    id: &'static str,
    author: Author,
    preview: &'static str,
    time: &'static str,
    body: &'static [&'static str],
) -> Message {
    Message {
        id,
        author,
        preview,
        time,
        body,
        invitation: None,
    }
}

fn threads() -> Vec<Thread> {
    const MIRA: Author = Contact("Mira");
    vec![
        Thread {
            id: "t-okafor",
            account: "laurentian",
            place: Place::Inbox(ForYou),
            folders: &[],
            starred: true,
            unread: true,
            from: "Prof. Okafor",
            subject: "COSC 4456 exam room moved to FA-224",
            time: "9:41",
            note: ("Gatekeeper flagged this for you", NoteKind::Flagged),
            summary: "Tomorrow’s parallel computing exam is in FA-224 instead of the usual room. The time is unchanged.",
            messages: vec![message(
                "m-okafor",
                Contact("Prof. Okafor"),
                "Please note the exam will be held in FA-224",
                "9:41",
                &[
                    "Hello all,",
                    "Please note the exam for COSC 4456 will be held in FA-224. The time and format are unchanged.",
                    "Good luck with your studying.",
                ],
            )],
            quick_replies: &[
                "Thanks for letting us know",
                "Is the exam still at 14:00?",
                "Will office hours be in FA-302 as usual?",
            ],
        },
        Thread {
            id: "t-northline",
            account: "personal",
            place: Place::Inbox(ForYou),
            folders: &[],
            starred: false,
            unread: true,
            from: "Northline Talent",
            subject: "Next steps: ML infrastructure interview loop",
            time: "9:12",
            note: ("Job application tracker drafted a reply", NoteKind::Waiting),
            summary: "They want to book a three-part loop next week (systems design, coding, team chat) and ask for two time windows.",
            messages: vec![message(
                "m-northline",
                Contact("Dana, Northline Talent"),
                "Could you send two windows next week that work for you?",
                "9:12",
                &[
                    "Hi Ali,",
                    "Thanks for the chat last week. We’d love to move you to the full loop for the ML Infrastructure role: a systems design session, a coding session and a conversation with the team.",
                    "Could you send two windows next week that work for you?",
                    "Best,\nDana",
                ],
            )],
            quick_replies: &[
                "Tuesday afternoon works best",
                "Thursday morning works best",
                "Could we do the week after?",
            ],
        },
        Thread {
            id: "t-slides",
            account: "laurentian",
            place: Place::Inbox(ForYou),
            folders: &[],
            starred: false,
            unread: true,
            from: "Mira",
            subject: "Slides for Thursday, can you take section 3?",
            time: "8:50",
            note: ("Scribe suggested a reply", NoteKind::Flagged),
            summary: "The group agreed on four sections on Monday and Mira booked a rehearsal for tomorrow morning. She now asks you to own section 3 (results) by tonight.",
            messages: vec![
                message(
                    "m-slides-1",
                    MIRA,
                    "Kicking off the deck",
                    "Mon 16:12",
                    &[
                        "Kicking off the deck for Thursday. I made a shared file with four sections: problem, method, results, next steps. Claim one when you can!",
                    ],
                ),
                message(
                    "m-slides-2",
                    You,
                    "Happy to help where needed",
                    "Mon 18:30",
                    &[
                        "Happy to help where needed. I have the benchmark numbers if anyone wants them.",
                    ],
                ),
                Message {
                    invitation: Some("e-rehearsal"),
                    ..message(
                        "m-slides-3",
                        MIRA,
                        "Invitation: Deck rehearsal",
                        "Tue 11:20",
                        &["Booked FA-110 so we can run through it once together."],
                    )
                },
                message(
                    "m-slides-4",
                    MIRA,
                    "Could you take section 3, the results?",
                    "8:50",
                    &[
                        "Hey! I split the deck into four parts. Could you take section 3, the results? Would be great to have it by tonight so we can rehearse tomorrow morning.",
                    ],
                ),
            ],
            quick_replies: &[
                "Yes, I’ll have section 3 in the deck tonight",
                "Can I take it but need until tomorrow 8:00?",
                "I can’t this week, could someone swap?",
            ],
        },
        Thread {
            id: "t-harbour",
            account: "personal",
            place: Place::Inbox(ForYou),
            folders: &[],
            starred: false,
            unread: false,
            from: "Harbour Labs",
            subject: "We received your application",
            time: "8:41",
            note: (
                "Job application tracker logged this to Applications",
                NoteKind::Filed,
            ),
            summary: "Harbour Labs confirms your application for the Platform Engineer role and will answer within two weeks.",
            messages: vec![message(
                "m-harbour",
                Contact("Harbour Labs"),
                "Thanks for applying to the Platform Engineer role",
                "8:41",
                &[
                    "Hi Ali,",
                    "Thanks for applying to the Platform Engineer role at Harbour Labs. Our team will review your application and get back to you within two weeks.",
                    "Harbour Labs recruiting",
                ],
            )],
            quick_replies: &[
                "Thanks, looking forward to it",
                "Happy to share more work samples",
                "Could you share the team’s stack?",
            ],
        },
        Thread {
            id: "t-ridgeway",
            account: "personal",
            place: Place::Inbox(Updates),
            folders: &["receipts"],
            starred: false,
            unread: false,
            from: "Ridgeway Hosting",
            subject: "Invoice #4471 for November",
            time: "8:30",
            note: ("Receipts clerk filed this", NoteKind::Filed),
            summary: "Monthly hosting invoice. Already filed to Receipts and logged in your budget sheet.",
            messages: vec![message(
                "m-ridgeway",
                Contact("Ridgeway Hosting"),
                "Your invoice for November is ready",
                "8:30",
                &["Your invoice for November is ready. It will be charged to the card on file."],
            )],
            quick_replies: &[
                "Thanks, received",
                "Could you send it as a PDF?",
                "Please update my billing address",
            ],
        },
        Thread {
            id: "t-campus",
            account: "laurentian",
            place: Place::Inbox(Updates),
            folders: &[],
            starred: false,
            unread: false,
            from: "Campus Rec",
            subject: "Intramural sign-ups close Friday",
            time: "8:02",
            note: ("Gatekeeper sorted this into Updates", NoteKind::Filed),
            summary: "Winter intramural registration closes Friday at 17:00.",
            messages: vec![message(
                "m-campus",
                Contact("Campus Rec"),
                "Registration closes this Friday",
                "8:02",
                &["Registration for winter intramurals closes this Friday at 17:00."],
            )],
            quick_replies: &[
                "Sign me up for climbing",
                "Is there a waitlist?",
                "Thanks for the reminder",
            ],
        },
        Thread {
            id: "t-pragmatic-digest",
            account: "icloud",
            place: Place::Inbox(Reading),
            folders: &[],
            starred: false,
            unread: false,
            from: "The Pragmatic Digest",
            subject: "This week: queues, caches and backpressure",
            time: "7:05",
            note: (
                "Gatekeeper sorted this into Reading · in the 7:30 briefing",
                NoteKind::Filed,
            ),
            summary: "Three long reads on queueing and backpressure in distributed systems.",
            messages: vec![message(
                "m-pragmatic-digest",
                Contact("The Pragmatic Digest"),
                "Queues, caches and backpressure",
                "7:05",
                &[
                    "This week we look at queues, caches and how backpressure keeps systems upright.",
                ],
            )],
            quick_replies: &["Thanks", "Unsubscribe me", "More like this"],
        },
        Thread {
            id: "t-parcel",
            account: "personal",
            place: Place::Inbox(Updates),
            folders: &[],
            starred: false,
            unread: false,
            from: "Parcelway",
            subject: "Your package is out for delivery",
            time: "Yesterday",
            note: ("Gatekeeper sorted this into Updates", NoteKind::Filed),
            summary: "The package was due before 20:00 yesterday.",
            messages: vec![message(
                "m-parcel",
                Contact("Parcelway"),
                "Your package is out for delivery",
                "Yesterday",
                &["Your package is out for delivery and should arrive today before 20:00."],
            )],
            quick_replies: &[
                "Please leave it at the door",
                "Can I pick it up instead?",
                "Thanks",
            ],
        },
        Thread {
            id: "t-dev-weekly",
            account: "icloud",
            place: Place::Inbox(Reading),
            folders: &[],
            starred: false,
            unread: false,
            from: "Dev Weekly",
            subject: "Zig 0.15 notes, async I/O revisited",
            time: "Yesterday",
            note: (
                "Gatekeeper sorted this into Reading · in the Tue 18:00 briefing",
                NoteKind::Filed,
            ),
            summary: "A release notes round-up and a long read on async I/O.",
            messages: vec![message(
                "m-dev-weekly",
                Contact("Dev Weekly"),
                "Release notes round-up",
                "Yesterday",
                &["Release notes round-up and a long read on async I/O."],
            )],
            quick_replies: &["Thanks", "Unsubscribe me", "More like this"],
        },
        Thread {
            id: "t-ostra",
            account: "personal",
            place: Place::Archive,
            folders: &[],
            starred: false,
            unread: false,
            from: "Ostra Robotics",
            subject: "Application: Platform Engineer",
            time: "Sat",
            note: ("Follow-up nudger drafted a nudge", NoteKind::Waiting),
            summary: "You applied to Ostra Robotics on Saturday and nobody has answered yet.",
            messages: vec![message(
                "m-ostra",
                You,
                "My application for the Platform Engineer role",
                "Sat 10:15",
                &[
                    "Hi Sam,",
                    "Please find my application for the Platform Engineer role attached. I’d be glad to talk about how I could help the team.",
                    "Best,\nAli",
                ],
            )],
            quick_replies: &[
                "Just checking in",
                "Happy to share more",
                "Is the role still open?",
            ],
        },
        Thread {
            id: "t-lunch",
            account: "icloud",
            place: Place::Archive,
            folders: &[],
            starred: false,
            unread: false,
            from: "Mira",
            subject: "Lunch Monday?",
            time: "Sun",
            note: ("Gatekeeper sorted this into For you", NoteKind::Filed),
            summary: "Mira suggests lunch at the student centre on Monday at 12:00.",
            messages: vec![message(
                "m-lunch",
                MIRA,
                "Lunch on Monday at the student centre?",
                "Sun 19:04",
                &["Lunch on Monday at the student centre? 12:00 works for me."],
            )],
            quick_replies: &["See you there", "Can we do 12:30?", "Next week instead?"],
        },
    ]
}

fn model(name: &str, provider: ModelProvider) -> ui::Model {
    ui::Model {
        name: name.into(),
        provider: provider.into(),
    }
}

fn stats(pairs: &[(&str, &str)]) -> Vec<ui::Stat> {
    pairs
        .iter()
        .map(|(value, label)| ui::Stat {
            value: (*value).into(),
            label: (*label).into(),
        })
        .collect()
}

fn permissions(levels: &[(&str, &str, PermissionLevel)]) -> Vec<ui::Permission> {
    levels
        .iter()
        .map(|(action, hint, level)| ui::Permission {
            action: (*action).into(),
            hint: (*hint).into(),
            level: (*level).into(),
            levels: Some(ui::permission::Levels::All(ui::AllLevels {})),
        })
        .collect()
}

fn status(state: ui::agent_status::State) -> Option<ui::AgentStatus> {
    Some(ui::AgentStatus { state: Some(state) })
}

fn run(
    id: &str,
    time: &str,
    waiting: bool,
    summary: &str,
    duration: &str,
    steps: &[(&str, &str)],
) -> ui::AgentRun {
    use ui::run_outcome::State;
    ui::AgentRun {
        id: id.into(),
        time: time.into(),
        outcome: Some(ui::RunOutcome {
            state: Some(if waiting {
                State::Waiting(ui::RunWaiting {})
            } else {
                State::Done(ui::RunDone {})
            }),
        }),
        summary: summary.into(),
        duration: duration.into(),
        steps: steps
            .iter()
            .map(|(verb, what)| ui::RunStep {
                verb: (*verb).into(),
                what: (*what).into(),
            })
            .collect(),
    }
}

struct AgentSpec {
    id: &'static str,
    name: &'static str,
    status: ui::agent_status::State,
    next: &'static str,
    note: &'static str,
    last_run: &'static str,
    schedule: &'static str,
    model: ui::Model,
    kind: ui::agent_detail::Kind,
    stats: Vec<ui::Stat>,
    runs: Vec<ui::AgentRun>,
    chat: Option<Chat>,
}

fn agent(spec: AgentSpec) -> Agent {
    Agent {
        next: spec.next,
        note: spec.note,
        last_run: spec.last_run,
        provider: spec.model.provider(),
        chat: spec.chat,
        detail: ui::AgentDetail {
            id: spec.id.into(),
            name: spec.name.into(),
            status: status(spec.status),
            kind: Some(match spec.kind {
                ui::agent_detail::Kind::System(system) => {
                    ui::agent_detail::Kind::System(ui::SystemAgent {
                        your_instructions_label: format!("Your instructions for {}", spec.name),
                        ..system
                    })
                }
                user @ ui::agent_detail::Kind::User(_) => user,
            }),
            stats: spec.stats,
            runs: spec.runs,
            subtitle: format!("{} · {}", spec.schedule, spec.model.name),
            // Composed by the view, which knows the keys.
            chat: None,
        },
    }
}

fn system(
    built_in: &str,
    yours: &str,
    schedule: ui::system_agent::Schedule,
    levels: &[(&str, &str, PermissionLevel)],
    key_note: &str,
) -> ui::agent_detail::Kind {
    ui::agent_detail::Kind::System(ui::SystemAgent {
        built_in_instructions: built_in.into(),
        your_instructions: yours.into(),
        schedule: Some(schedule),
        permissions: permissions(levels),
        key_note: key_note.into(),
        // Set by `agent`, which knows the name.
        your_instructions_label: String::new(),
    })
}

fn on_event(when: &str) -> ui::system_agent::Schedule {
    ui::system_agent::Schedule::OnEvent(ui::EventSchedule { when: when.into() })
}

/// A conversation that opens on activity, as the agents the user made do.
fn conversation(turns: Vec<ui::ChatTurn>) -> Option<Chat> {
    Some(Chat {
        opens_on: AgentDetailPane::Activity,
        turns,
    })
}

fn turn(id: &str, time: &str, message: &str, state: ui::chat_turn::State) -> ui::ChatTurn {
    ui::ChatTurn {
        id: id.into(),
        message: message.into(),
        time: time.into(),
        state: Some(state),
    }
}

fn answered(paragraphs: &[&str], references: Vec<ui::ChatReference>) -> ui::chat_turn::State {
    ui::chat_turn::State::Answered(ui::TurnAnswered {
        paragraphs: paragraphs.iter().map(|p| (*p).into()).collect(),
        references,
    })
}

fn thread_ref(thread_id: &str, label: &str) -> ui::ChatReference {
    ui::ChatReference {
        label: label.into(),
        target: Some(ui::chat_reference::Target::Thread(ui::ThreadReference {
            thread_id: thread_id.into(),
        })),
    }
}

fn event_ref(event_id: &str, label: &str) -> ui::ChatReference {
    ui::ChatReference {
        label: label.into(),
        target: Some(ui::chat_reference::Target::Event(ui::EventReference {
            event_id: event_id.into(),
        })),
    }
}

const SLIDES: &str = "Slides for Thursday, can you take section 3?";
const NORTHLINE: &str = "Next steps: ML infrastructure interview loop";
const EXAM_ROOM: &str = "COSC 4456 exam room moved to FA-224";

fn user(instructions: &str) -> ui::agent_detail::Kind {
    ui::agent_detail::Kind::User(ui::UserAgent {
        instructions: instructions.into(),
    })
}

fn agents() -> Vec<Agent> {
    use ui::agent_status::State;
    const CHECKS_ONLY: &str = "Without a model key it runs its built-in checks only";
    const OFF_WITHOUT_KEY: &str = "Off until you add a model key";
    let sonnet = || model("Claude Sonnet", ModelProvider::Anthropic);
    let idle = || State::Idle(ui::AgentIdle {});
    let waiting = || State::Waiting(ui::AgentWaiting {});
    [
        AgentSpec {
            id: "gatekeeper",
            chat: None,
            name: "Gatekeeper",
            status: State::Running(ui::AgentRunning {}),
            next: "On every new mail",
            note: "2",
            last_run: "9:41 · Flagged 1",
            schedule: "On every new mail",
            model: sonnet(),
            kind: system(
                "Sort every new mail into For you, Updates, Reading or one of your folders, and flag what needs you.\nCheck bulk headers, the provider’s spam verdict, SPF, DKIM and DMARC, the block list and whether you have written to the sender.\nHand likely scams to Security Guard.\nWhen an account is connected, sort its last 90 days and ask once per bulk sender whether to unsubscribe.",
                "Anything from my project group goes to For you.",
                on_event("On every new mail, after your rules"),
                &[
                    ("Sort into categories", "", Auto),
                    ("Flag", "", Auto),
                    ("Move to your folders", "", Auto),
                    ("Unsubscribe", "", Ask),
                ],
                CHECKS_ONLY,
            ),
            stats: stats(&[
                ("21", "sorted today"),
                ("2", "waiting for you"),
                ("9:41", "last run"),
            ]),
            runs: vec![
                run(
                    "r-gatekeeper-3",
                    "9:41",
                    false,
                    "Flagged 1",
                    "2 s",
                    &[("Flagged", "Prof. Okafor: exam room change")],
                ),
                run(
                    "r-gatekeeper-2",
                    "9:12",
                    false,
                    "Sorted 1 into For you",
                    "1 s",
                    &[("Sorted", "Northline Talent into For you")],
                ),
                run(
                    "r-gatekeeper-1",
                    "Mon",
                    true,
                    "Swept 90 days of Laurentian · 2 questions",
                    "38 s",
                    &[
                        ("Sorted", "2,140 messages, no model calls"),
                        ("Asked", "Unsubscribe from Jobstream weekly?"),
                        ("Asked", "Unsubscribe from Campus Store deals?"),
                    ],
                ),
            ],
        },
        AgentSpec {
            id: "security-guard",
            chat: None,
            name: "Security Guard",
            status: idle(),
            next: "When mail is flagged as a scam",
            note: "",
            last_run: "7:12 · Quarantined 1",
            schedule: "When Gatekeeper flags a scam",
            model: sonnet(),
            kind: system(
                "Move scam and malicious mail to Quarantine.\nBlock the sender and report the mail to the provider as spam.\nTell me what it caught.\nNever follow a link. Never unsubscribe.",
                "",
                on_event("When Gatekeeper flags mail as a scam or malicious"),
                &[
                    ("Quarantine", "", Auto),
                    ("Block sender", "", Auto),
                    ("Report spam", "", Auto),
                    ("Notify you", "", Auto),
                ],
                CHECKS_ONLY,
            ),
            stats: stats(&[
                ("3", "in Quarantine"),
                ("3", "senders blocked"),
                ("7:12", "last catch"),
            ]),
            runs: vec![
                run(
                    "r-security-guard-2",
                    "7:12",
                    false,
                    "Quarantined 1 · sender blocked and reported",
                    "2 s",
                    &[
                        ("Caught", "“Your mailbox will be closed today”"),
                        ("Blocked", "security@acc0unt-verify.net"),
                        ("Reported", "To Gmail as spam"),
                    ],
                ),
                run(
                    "r-security-guard-1",
                    "Tue",
                    false,
                    "Quarantined 1 · sender blocked and reported",
                    "2 s",
                    &[
                        ("Caught", "“Customs fee unpaid for your parcel”"),
                        ("Blocked", "noreply@shipping-hold.info"),
                        ("Reported", "To Outlook as spam"),
                    ],
                ),
            ],
        },
        AgentSpec {
            id: "scribe",
            chat: None,
            name: "Scribe",
            status: idle(),
            next: "When a summary is needed",
            note: "",
            last_run: "8:52 · Summarised Mira’s thread",
            schedule: "When a summary or reply is needed",
            model: model("Gemini Pro", ModelProvider::GoogleGemini),
            kind: system(
                "Write a summary of a thread the first time it is needed.\nSuggest short replies in your voice.",
                "Keep summaries to two sentences.",
                on_event(
                    "The first time you open a thread or another agent needs it, never on arrival",
                ),
                &[("Write summaries", "", Auto), ("Suggest replies", "", Auto)],
                OFF_WITHOUT_KEY,
            ),
            stats: stats(&[
                ("63", "summaries this week"),
                ("18", "replies suggested"),
                ("8:52", "last run"),
            ]),
            runs: vec![run(
                "r-scribe-1",
                "8:52",
                false,
                "Summarised Mira’s thread · 3 replies",
                "4 s",
                &[
                    ("Summarised", "Slides for Thursday"),
                    ("Suggested", "3 quick replies"),
                ],
            )],
        },
        AgentSpec {
            id: "news-bearer",
            chat: None,
            name: "News Bearer",
            status: idle(),
            next: "Daily at 7:30 and 18:00",
            note: "18:00",
            last_run: "7:30 · Morning briefing ready",
            schedule: "Daily at 7:30 and 18:00",
            model: sonnet(),
            kind: system(
                "Write a briefing at the set times: what agents did, what waits for you, newsletters and updates condensed, and today’s calendar.\nShow it in Briefing and notify me.",
                "",
                ui::system_agent::Schedule::AtTimes(ui::TimesSchedule {
                    when: "Daily at".into(),
                    times: vec!["7:30".into(), "18:00".into()],
                }),
                &[
                    ("Write the briefing", "", Auto),
                    ("Mail the briefing to you", "", Off),
                ],
                OFF_WITHOUT_KEY,
            ),
            stats: stats(&[
                ("1", "briefing today"),
                ("2", "newsletters condensed"),
                ("7:30", "last run"),
            ]),
            runs: vec![
                run(
                    "r-news-bearer-2",
                    "7:30",
                    false,
                    "Morning briefing ready",
                    "10 s",
                    &[
                        ("Read", "1 newsletter and 1 update"),
                        ("Wrote", "Briefing for Wednesday morning"),
                    ],
                ),
                run(
                    "r-news-bearer-1",
                    "Tue",
                    false,
                    "Evening briefing ready",
                    "12 s",
                    &[
                        ("Read", "1 newsletter and 1 update"),
                        ("Wrote", "Briefing for Tuesday evening"),
                    ],
                ),
            ],
        },
        AgentSpec {
            id: "assistant",
            chat: Some(Chat {
                opens_on: AgentDetailPane::Chat,
                turns: vec![
                    turn(
                        "c-assistant-1",
                        "Mon",
                        "How much did I spend on everything this year?",
                        ui::chat_turn::State::Stopped(ui::TurnStopped {
                            text: "Stopped at the limit on mails one turn may read. Ask about a shorter span, such as this month".into(),
                        }),
                    ),
                    turn(
                        "c-assistant-2",
                        "8:12",
                        "What do I still owe people this week?",
                        answered(
                            &[
                                "Two things. Mira asked you to take section 3 of the deck, the results, by tonight so the group can rehearse tomorrow morning.",
                                "Dana at Northline Talent wants two time windows for the interview loop. Job application tracker drafted a reply, and it waits for your approval.",
                            ],
                            vec![thread_ref("t-slides", SLIDES), thread_ref("t-northline", NORTHLINE)],
                        ),
                    ),
                    turn(
                        "c-assistant-3",
                        "8:20",
                        "When and where is my COSC 4456 exam?",
                        answered(
                            &["Thursday, Nov 12, 14:00 to 16:00 in FA-224. Prof. Okafor moved it from FA-110."],
                            vec![
                                event_ref("e-exam", "COSC 4456 exam · Thu 14:00"),
                                thread_ref("t-okafor", EXAM_ROOM),
                            ],
                        ),
                    ),
                    turn(
                        "c-assistant-4",
                        "9:48",
                        "Tell Mira I’ll have section 3 done by tonight",
                        ui::chat_turn::State::Waiting(ui::TurnWaiting {
                            approval_id: "a-assistant-mira".into(),
                            approval_title: "Reply to Mira".into(),
                            label: "Waiting for your approval".into(),
                        }),
                    ),
                ],
            }),
            name: "Assistant",
            status: waiting(),
            next: "Answers when you ask · 1 waiting",
            note: "1",
            last_run: "9:48 · Drafted 1",
            schedule: "Answers when you ask",
            model: sonnet(),
            kind: system(
                "Answer my questions about my mail and calendar across every account. Read and search on your own.\nPropose any change and follow my permissions for it.",
                "",
                ui::system_agent::Schedule::OnMessage(ui::MessageSchedule {
                    when: "Only when you write to it".into(),
                }),
                &[
                    ("Read and search mail and calendars", "", Auto),
                    ("Draft replies", "", Auto),
                    ("Send", "", Ask),
                    ("Add calendar events", "", Ask),
                    ("Delete", "", Off),
                ],
                OFF_WITHOUT_KEY,
            ),
            stats: stats(&[
                ("4", "questions this week"),
                ("1", "waiting for you"),
                ("9:48", "last run"),
            ]),
            runs: vec![
                run(
                    "r-assistant-4",
                    "9:48",
                    true,
                    "Drafted a reply to Mira · waiting for you",
                    "6 s",
                    &[("Read", SLIDES), ("Drafted", "Reply to Mira")],
                ),
                run(
                    "r-assistant-3",
                    "8:20",
                    false,
                    "Answered when and where your exam is",
                    "3 s",
                    &[("Searched", "Calendars and mail for COSC 4456 exam")],
                ),
                run(
                    "r-assistant-2",
                    "8:12",
                    false,
                    "Answered what you owe this week",
                    "8 s",
                    &[("Read", "9 threads from this week")],
                ),
                run(
                    "r-assistant-1",
                    "Mon",
                    false,
                    "Stopped at the read limit",
                    "41 s",
                    &[("Read", "Receipts from this year")],
                ),
            ],
        },
        AgentSpec {
            id: "job-tracker",
            chat: conversation(vec![]),
            name: "Job application tracker",
            status: waiting(),
            next: "Mail sorted into For you · 1 waiting",
            note: "1",
            last_run: "9:45 · Drafted 1",
            schedule: "Mail sorted into For you, and every hour",
            model: sonnet(),
            kind: user(
                "Watch for mail about my job applications: confirmations, interview invites, rejections and offers.\nLabel each one with the company and stage, and keep my Applications sheet up to date.\nFor interview invites, draft a reply with two time windows from my calendar.",
            ),
            stats: stats(&[
                ("24", "labelled this month"),
                ("1", "waiting for you"),
                ("9:45", "last run"),
            ]),
            runs: vec![
                run(
                    "r-job-tracker-2",
                    "9:45",
                    true,
                    "Drafted 1 · waiting for you",
                    "9 s",
                    &[
                        ("Labelled", "Northline · Interview"),
                        ("Drafted", "Reply to Northline Talent"),
                    ],
                ),
                run(
                    "r-job-tracker-1",
                    "8:41",
                    false,
                    "Logged Harbour Labs application",
                    "5 s",
                    &[("Logged", "Row added to Applications")],
                ),
            ],
        },
        AgentSpec {
            id: "receipts-clerk",
            chat: conversation(vec![]),
            name: "Receipts clerk",
            status: idle(),
            next: "New mail arrived",
            note: "",
            last_run: "8:30 · Filed 1",
            schedule: "New mail arrived",
            model: model("Claude Haiku", ModelProvider::Anthropic),
            kind: user(
                "Find receipts and invoices in every account.\nMove them to Receipts and add vendor, amount and date to my budget sheet.\nNever reply or forward.",
            ),
            stats: stats(&[
                ("86", "filed this month"),
                ("100%", "runs with no questions"),
                ("8:30", "last run"),
            ]),
            runs: vec![run(
                "r-receipts-clerk-1",
                "8:30",
                false,
                "Filed Ridgeway Hosting invoice",
                "6 s",
                &[
                    ("Filed", "Invoice #4471 to Receipts"),
                    ("Logged", "Row added to budget sheet"),
                ],
            )],
        },
        AgentSpec {
            id: "follow-up-nudger",
            chat: conversation(vec![]),
            name: "Follow-up nudger",
            status: waiting(),
            next: "Weekdays 9:00 · 1 waiting",
            note: "1",
            last_run: "9:00 · Drafted 1",
            schedule: "Weekdays at 9:00",
            model: sonnet(),
            kind: user(
                "If I sent something that needs an answer and nobody replied in 3 days, draft a short, friendly nudge.\nAsk before sending to anyone I haven’t emailed before.",
            ),
            stats: stats(&[
                ("12", "nudges sent"),
                ("1", "waiting for you"),
                ("9:00", "last run"),
            ]),
            runs: vec![run(
                "r-follow-up-nudger-1",
                "9:00",
                true,
                "Drafted 1 nudge · waiting for you",
                "7 s",
                &[("Drafted", "Nudge to Ostra Robotics recruiting")],
            )],
        },
        AgentSpec {
            id: "day-summary",
            chat: conversation(vec![
                turn(
                    "c-day-summary-1",
                    "7:02",
                    "Summarize last night’s mail",
                    ui::chat_turn::State::Failed(ui::TurnFailed {
                        reason: "Anthropic did not answer in time. Nothing was changed".into(),
                    }),
                ),
                turn(
                    "c-day-summary-2",
                    "9:30",
                    "Summarize this morning so far",
                    answered(
                        &[
                            "Three things need you: Mira wants section 3 of the deck by tonight, Dana at Northline Talent is waiting on interview times, and Prof. Okafor moved Thursday’s exam to FA-224.",
                            "The rest were updates and newsletters, already sorted.",
                        ],
                        vec![
                            thread_ref("t-slides", SLIDES),
                            thread_ref("t-northline", NORTHLINE),
                            thread_ref("t-okafor", EXAM_ROOM),
                        ],
                    ),
                ),
                turn(
                    "c-day-summary-3",
                    "9:50",
                    "Anything new since then?",
                    ui::chat_turn::State::Running(ui::TurnRunning {
                        label: "Reading your mail".into(),
                    }),
                ),
            ]),
            name: "Day summary",
            status: State::Running(ui::AgentRunning {}),
            next: "Weekdays 17:00",
            note: "17:00",
            last_run: "9:30 · Answered you",
            schedule: "Weekdays at 17:00",
            model: sonnet(),
            kind: user("Give me a brief summary of what came in today and what still needs an answer."),
            stats: stats(&[
                ("3", "summaries this week"),
                ("9:30", "last run"),
            ]),
            runs: vec![
                run(
                    "r-day-summary-2",
                    "9:30",
                    false,
                    "Summarized this morning",
                    "7 s",
                    &[("Read", "14 mails since midnight")],
                ),
                run(
                    "r-day-summary-1",
                    "Tue",
                    false,
                    "Summary ready",
                    "9 s",
                    &[("Read", "41 mails from Tuesday")],
                ),
            ],
        },
    ]
    .into_iter()
    .map(agent)
    .collect()
}

fn approval(
    id: &str,
    agent: &str,
    time: &str,
    title: &str,
    detail: &str,
    reason: &str,
    action: ui::approval_item::Action,
) -> ui::ApprovalItem {
    ui::ApprovalItem {
        id: id.into(),
        agent: agent.into(),
        time: time.into(),
        title: title.into(),
        detail: detail.into(),
        reason: reason.into(),
        action: Some(action),
        state: Some(ui::ApprovalState {
            state: Some(ui::approval_state::State::Waiting(ui::ApprovalWaiting {})),
        }),
    }
}

fn approvals() -> Vec<Approval> {
    use ui::approval_item::Action;
    let send = |thread: &str| {
        Action::Send(ui::SendApproval {
            thread_id: thread.into(),
        })
    };
    let unsubscribe = || Action::Unsubscribe(ui::UnsubscribeApproval {});
    vec![
        Approval {
            item: approval(
                "a-assistant-mira",
                "Assistant",
                "9:48",
                "Reply to Mira",
                "“Hi Mira, I’ll have section 3 done by tonight…”",
                "Assistant asks before sending",
                send("t-slides"),
            ),
            draft: &[
                "Hi Mira,",
                "I’ll have section 3, the results, done by tonight, so we’re set for the rehearsal tomorrow morning.",
                "Ali",
            ],
        },
        Approval {
            item: approval(
                "a-northline",
                "Job application tracker",
                "9:45",
                "Reply to Northline Talent",
                "Offers Tuesday 13:00–16:00 or Thursday 10:00–13:00 for the interview loop.",
                "Gmail accounts always ask before sending for now",
                send("t-northline"),
            ),
            draft: &[
                "Hi Dana,",
                "That sounds great. Tuesday 13:00–16:00 or Thursday 10:00–13:00 (Eastern) both work for me.",
                "Thanks,\nAli",
            ],
        },
        Approval {
            item: approval(
                "a-ostra",
                "Follow-up nudger",
                "9:00",
                "Nudge Ostra Robotics recruiting",
                "“Hi Sam, just checking in on my application from last week…”",
                "No reply in 4 days",
                send("t-ostra"),
            ),
            draft: &[
                "Hi Sam,",
                "Just checking in on my application for the Platform Engineer role from last week. Happy to share anything else that helps.",
                "Thanks,\nAli",
            ],
        },
        Approval {
            item: approval(
                "a-jobstream",
                "Gatekeeper",
                "Mon",
                "Unsubscribe from Jobstream weekly?",
                "Sent 46 mails to Laurentian in the last 90 days. You opened 2.",
                "Found when Laurentian was connected",
                unsubscribe(),
            ),
            draft: &[],
        },
        Approval {
            item: approval(
                "a-campus-store",
                "Gatekeeper",
                "Mon",
                "Unsubscribe from Campus Store deals?",
                "Sent 31 mails to Laurentian in the last 90 days. You opened none.",
                "Found when Laurentian was connected",
                unsubscribe(),
            ),
            draft: &[],
        },
    ]
}

fn calendar() -> Calendar {
    let source = |id: &str,
                  name: &str,
                  provider: CalendarProvider,
                  provider_name: &str,
                  color: CalendarColor| ui::CalendarSource {
        id: id.into(),
        name: name.into(),
        provider: provider.into(),
        provider_name: provider_name.into(),
        shown: true,
        color: color.into(),
    };
    let day = |id, weekday, day, today| Day {
        id,
        weekday,
        day,
        month: "Nov",
        today,
    };
    let event = |id, calendar, day, start: u32, end: u32, title, place| CalendarEvent {
        id,
        calendar,
        day,
        start,
        end,
        title,
        place,
        guests: &[],
        invitation: None,
    };
    let h = |hour: u32, minute: u32| hour * 60 + minute;
    Calendar {
        month: "November 2026",
        range: "Nov 9 – 15",
        next_event: "10:00",
        first_hour: 8,
        last_hour: 20,
        sources: vec![
            source(
                "cal-personal",
                "Personal",
                CalendarProvider::GoogleCalendar,
                "Google Calendar",
                CalendarColor::Violet,
            ),
            source(
                "cal-laurentian",
                "Laurentian",
                CalendarProvider::OutlookCalendar,
                "Outlook calendar",
                CalendarColor::Cyan,
            ),
            source(
                "cal-icloud",
                "iCloud",
                CalendarProvider::Caldav,
                "iCloud through CalDAV",
                CalendarColor::Amber,
            ),
        ],
        days: vec![
            day("2026-11-09", "Mon", "9", false),
            day("2026-11-10", "Tue", "10", false),
            day("2026-11-11", "Wed", "11", true),
            day("2026-11-12", "Thu", "12", false),
            day("2026-11-13", "Fri", "13", false),
            day("2026-11-14", "Sat", "14", false),
            day("2026-11-15", "Sun", "15", false),
        ],
        events: vec![
            event(
                "e-lecture-mon",
                "cal-laurentian",
                0,
                h(10, 0),
                h(11, 30),
                "COSC 4456 lecture",
                "FA-110",
            ),
            CalendarEvent {
                guests: &["Mira"],
                ..event(
                    "e-lunch",
                    "cal-icloud",
                    0,
                    h(12, 0),
                    h(13, 0),
                    "Lunch with Mira",
                    "Student centre",
                )
            },
            event(
                "e-lecture-wed",
                "cal-laurentian",
                2,
                h(10, 0),
                h(11, 30),
                "COSC 4456 lecture",
                "FA-110",
            ),
            event(
                "e-slides",
                "cal-personal",
                2,
                h(19, 0),
                h(20, 30),
                "Section 3 slides",
                "Focus time",
            ),
            CalendarEvent {
                guests: &["Mira", "Priya", "Tomas"],
                invitation: Some(ui::Invitation {
                    event_id: "e-rehearsal".into(),
                    invited_by: "Mira invited you".into(),
                    title: "Deck rehearsal".into(),
                    when: "Thu, Nov 12 · 9:00–10:00 · FA-110".into(),
                    details: "Laurentian · Outlook calendar · Mira, you and 2 others · nothing else on your calendars then".into(),
                    status: "Invitation · Laurentian calendar · waiting for your answer".into(),
                    answer: Some(ui::InvitationAnswer {
                        state: Some(ui::invitation_answer::State::Unanswered(
                            ui::InvitationUnanswered {},
                        )),
                    }),
                }),
                ..event(
                    "e-rehearsal",
                    "cal-laurentian",
                    3,
                    h(9, 0),
                    h(10, 0),
                    "Deck rehearsal",
                    "FA-110",
                )
            },
            event(
                "e-office-hours",
                "cal-laurentian",
                3,
                h(10, 0),
                h(11, 0),
                "Office hours, Prof. Okafor",
                "FA-302",
            ),
            event(
                "e-exam",
                "cal-laurentian",
                3,
                h(14, 0),
                h(16, 0),
                "COSC 4456 exam",
                "FA-224",
            ),
            event(
                "e-brightwave",
                "cal-personal",
                4,
                h(10, 0),
                h(11, 30),
                "Brightwave AI assessment",
                "Online, 90 min",
            ),
            event(
                "e-intramural",
                "cal-laurentian",
                4,
                h(17, 0),
                h(17, 30),
                "Intramural sign-ups close",
                "Campus Rec",
            ),
            CalendarEvent {
                guests: &["Jordan"],
                ..event(
                    "e-climbing",
                    "cal-icloud",
                    5,
                    h(11, 0),
                    h(12, 30),
                    "Climbing with Jordan",
                    "Boulder gym",
                )
            },
            event(
                "e-call-home",
                "cal-icloud",
                6,
                h(18, 0),
                h(19, 0),
                "Call home",
                "",
            ),
        ],
    }
}

fn quarantine() -> Vec<Quarantined> {
    vec![
        Quarantined {
            id: "q-mailbox",
            from: "security@acc0unt-verify.net",
            account: "personal",
            time: "7:12",
            subject: "Your mailbox will be closed today",
            reason: "Phishing: asks you to sign in on a look-alike page",
            caught_by: "Caught by Security Guard",
            why: "The sender pretends to be your mail provider and links to a sign-in page on another domain. SPF and DKIM both failed.",
            outcome: "Sender blocked and reported to Gmail",
            drop: "Dec 11",
            body: &[
                "Dear user,",
                "Your mailbox has exceeded its limit and will be closed today. Verify your account now to keep your messages.",
                "[link removed]",
                "Mail Support Team",
            ],
        },
        Quarantined {
            id: "q-customs",
            from: "noreply@shipping-hold.info",
            account: "laurentian",
            time: "Tue",
            subject: "Customs fee unpaid for your parcel",
            reason: "Scam: asks for a payment to release a parcel",
            caught_by: "Caught by Security Guard",
            why: "Asks for a small card payment to release a parcel. The domain was registered four days ago and you have no order with this sender.",
            outcome: "Sender blocked and reported to Outlook",
            drop: "Dec 10",
            body: &[
                "Your parcel is held at customs. Pay the outstanding fee of $2.99 within 24 hours or it will be returned.",
                "[link removed]",
            ],
        },
        Quarantined {
            id: "q-deals",
            from: "deals@megasavings.shop",
            account: "icloud",
            time: "Nov 3",
            subject: "Final hours: 80% off everything",
            reason: "Blocked sender",
            caught_by: "Blocked sender",
            why: "You blocked this sender on Oct 2, so it came straight here without any agent reading it.",
            outcome: "Already on your block list",
            drop: "Dec 3",
            body: &[
                "Final hours! Everything 80% off, today only.",
                "[link removed]",
            ],
        },
    ]
}

fn lines(pairs: &[(&str, &str)]) -> Vec<ui::BriefingLine> {
    pairs
        .iter()
        .map(|(source, text)| ui::BriefingLine {
            source: (*source).into(),
            text: (*text).into(),
        })
        .collect()
}

fn unsubscribe_waits() -> Vec<ui::BriefingWaiting> {
    [
        (
            "Unsubscribe from Jobstream weekly?",
            "46 mails in 90 days, you opened 2",
        ),
        (
            "Unsubscribe from Campus Store deals?",
            "31 mails in 90 days, you opened none",
        ),
    ]
    .iter()
    .map(|(title, reason)| ui::BriefingWaiting {
        title: (*title).into(),
        detail: format!("Gatekeeper · {reason}"),
    })
    .collect()
}

fn todays_events() -> Vec<ui::BriefingEvent> {
    [
        ("10:00", "COSC 4456 lecture", "FA-110 · Laurentian"),
        ("19:00", "Section 3 slides", "Focus time · Personal"),
    ]
    .iter()
    .map(|(time, title, detail)| ui::BriefingEvent {
        time: (*time).into(),
        title: (*title).into(),
        detail: (*detail).into(),
    })
    .collect()
}

fn editions() -> Vec<Edition> {
    vec![
        Edition {
            id: "b-wed-morning",
            label: "7:30",
            byline: "Wednesday 7:30 · written by News Bearer",
            activity: lines(&[
                (
                    "Gatekeeper",
                    "sorted 12 mails that came in overnight: 2 for you, 4 updates, 6 to read.",
                ),
                (
                    "Security Guard",
                    "caught 1 phishing mail pretending to be your mail provider, blocked the sender and reported it to Gmail.",
                ),
            ]),
            caught_mail: true,
            waiting: unsubscribe_waits(),
            newsletters: lines(&[(
                "The Pragmatic Digest",
                "Three long reads on queues, caches and backpressure. The short version: bound every queue, and let slow consumers push back instead of dropping work.",
            )]),
            updates: lines(&[(
                "Parcelway",
                "your package was delivered yesterday at 18:40.",
            )]),
            calendar_title: "Today",
            calendar: todays_events(),
        },
        Edition {
            id: "b-tue-evening",
            label: "Tue 18:00",
            byline: "Tuesday 18:00 · written by News Bearer",
            activity: lines(&[
                (
                    "Gatekeeper",
                    "sorted 41 new mails: 6 for you, 14 updates, 21 to read.",
                ),
                (
                    "Security Guard",
                    "caught 1 scam asking for a customs fee, blocked the sender and reported it to Outlook.",
                ),
            ]),
            caught_mail: true,
            waiting: unsubscribe_waits(),
            newsletters: lines(&[(
                "Dev Weekly",
                "Zig 0.15 notes are out. The long read revisits async I/O and argues for explicit event loops over hidden ones.",
            )]),
            updates: lines(&[(
                "Parcelway",
                "your package is out for delivery, expected before 20:00.",
            )]),
            calendar_title: "Tomorrow",
            calendar: todays_events(),
        },
    ]
}

fn settings() -> Settings {
    use ui::provider_key::State;
    let rule = |id: &str, when: &str, then: &str, enabled: bool| ui::Rule {
        id: id.into(),
        when: when.into(),
        then: then.into(),
        enabled,
    };
    let blocked = |address: &str, blocked_by: &str| ui::BlockedSender {
        address: address.into(),
        blocked_by: blocked_by.into(),
    };
    let task = |task: &str, hint: &str, model: ui::Model| ui::TaskModel {
        task: task.into(),
        hint: hint.into(),
        model: Some(model),
    };
    Settings {
        defaults: permissions(&[
            ("Read and label", "", Auto),
            ("Draft replies", "", Auto),
            ("Archive", "", Auto),
            ("Send", "", Ask),
            ("Unsubscribe", "", Ask),
            ("Delete", "", Off),
        ]),
        notify_when_waiting: true,
        agents_paused: false,
        rules: vec![
            rule(
                "rule-laurentian",
                "sender domain is laurentian.ca",
                "mark important and notify me",
                true,
            ),
            rule(
                "rule-receipts",
                "subject contains “receipt” or “invoice”",
                "hand to Receipts clerk",
                true,
            ),
            rule(
                "rule-newsletters",
                "mail is a newsletter",
                "skip the inbox, into the briefing",
                true,
            ),
            rule(
                "rule-new-link",
                "sender is new and mail has a link",
                "hand to Security Guard",
                false,
            ),
        ],
        blocked: vec![
            blocked(
                "security@acc0unt-verify.net",
                "Blocked by Security Guard · today",
            ),
            blocked(
                "noreply@shipping-hold.info",
                "Blocked by Security Guard · Nov 10",
            ),
            blocked(
                "prizes@luckywinner-claims.biz",
                "Blocked by Security Guard · Nov 4",
            ),
            blocked("deals@megasavings.shop", "Blocked by you · Oct 2"),
        ],
        keys: vec![
            (
                ModelProvider::Anthropic,
                "Anthropic",
                State::Set(ui::KeySet {}),
            ),
            (
                ModelProvider::Openai,
                "OpenAI",
                State::Missing(ui::KeyMissing {}),
            ),
            (
                ModelProvider::GoogleGemini,
                "Google Gemini",
                State::Set(ui::KeySet {}),
            ),
            (
                ModelProvider::Openrouter,
                "OpenRouter",
                State::Missing(ui::KeyMissing {}),
            ),
        ],
        tasks: vec![
            task(
                "Writing replies",
                "AI compose and drafts",
                model("Claude Sonnet", ModelProvider::Anthropic),
            ),
            task(
                "Summaries",
                "Scribe",
                model("Gemini Pro", ModelProvider::GoogleGemini),
            ),
            task(
                "Agents",
                "System agents and new agents",
                model("Claude Sonnet", ModelProvider::Anthropic),
            ),
        ],
        budget: ui::models_settings::Budget::Daily(ui::DailyBudget {
            tokens: "500,000".into(),
            used: "182,400 used today".into(),
        }),
    }
}

fn builder() -> ui::AgentBuilderView {
    let trigger = |id: &str, label: &str, chosen: bool, filters: &[(&str, &str)], summary: &str| {
        ui::BuilderTrigger {
            id: id.into(),
            label: label.into(),
            chosen,
            filters: filters
                .iter()
                .map(|(key, value)| ui::TriggerFilter {
                    key: (*key).into(),
                    value: (*value).into(),
                })
                .collect(),
            filter_summary: summary.into(),
        }
    };
    const MAIL_FILTERS: &[(&str, &str)] =
        &[("Sender", "Anyone"), ("Account", "All"), ("Folder", "Any")];
    let account = |id: &str, name: &str, provider: MailProvider, provider_name: &str, can_see| {
        ui::BuilderAccount {
            id: id.into(),
            name: name.into(),
            provider: provider.into(),
            provider_name: provider_name.into(),
            can_see,
        }
    };
    let mut permissions = permissions(&[
        ("Read and label", "Sort, tag and summarise mail", Auto),
        ("Update sheets", "Add rows to sheets you name", Off),
        ("Calendar", "Read free time, add events", Auto),
        ("Draft replies", "Saved to Drafts in your voice", Ask),
        ("Send", "Mail leaves your account", Ask),
        ("Archive", "Clear handled mail", Off),
        ("Delete", "Moves to trash", Off),
    ]);
    // It can see a Gmail account, so Send cannot be Auto yet.
    permissions[4].levels = Some(ui::permission::Levels::AskAtMost(ui::AskAtMost {
        reason: "Can’t be Auto on Gmail yet, Google is still reviewing EveryMail".into(),
    }));
    let summary = |label: &str, value: &str| ui::SummaryLine {
        label: label.into(),
        value: value.into(),
    };
    ui::AgentBuilderView {
        name: "Deadline keeper".into(),
        instructions: "When a professor emails about a deadline, add it to my calendar and remind me the day before.".into(),
        ideas: vec![
            "Track my job applications".into(),
            "File receipts to my budget sheet".into(),
            "Nudge threads nobody answered".into(),
            "Unsubscribe from lists I ignore".into(),
        ],
        triggers: vec![
            trigger(
                "new-mail",
                "New mail arrived",
                false,
                MAIL_FILTERS,
                "Any sender, all accounts, any folder",
            ),
            trigger(
                "sorted",
                "Mail sorted into a category or folder",
                true,
                &[
                    ("Into", "For you"),
                    ("Sender", "Anyone"),
                    ("Account", "Personal, Laurentian"),
                ],
                "Into For you, from anyone, Personal and Laurentian",
            ),
            trigger(
                "flagged",
                "Mail flagged",
                false,
                MAIL_FILTERS,
                "Any sender, all accounts, any folder",
            ),
            trigger(
                "labelled",
                "Label or star added",
                false,
                MAIL_FILTERS,
                "Any sender, all accounts, any folder",
            ),
            trigger(
                "agent-finished",
                "Another agent finished",
                false,
                &[("Agent", "Any agent")],
                "Any agent",
            ),
            trigger(
                "approval-resolved",
                "Approval resolved",
                false,
                &[("From", "Any agent")],
                "Any agent",
            ),
            trigger(
                "event-approaching",
                "Calendar event approaching",
                true,
                &[("Lead time", "1 day before"), ("Calendar", "All")],
                "1 day before, all calendars",
            ),
            trigger(
                "file-changed",
                "File changed",
                false,
                &[("File", "Pick a file")],
                "Pick a file",
            ),
        ],
        schedule: Some(ui::BuilderSchedule {
            choices: [
                "No schedule",
                "Every 15 minutes",
                "Every hour",
                "Daily at 8:00",
                "Weekdays at 9:00",
                "Custom schedule",
            ]
            .iter()
            .map(|label| ui::ScheduleChoice {
                label: (*label).into(),
                selected: *label == "Daily at 8:00",
            })
            .collect(),
            state: Some(ui::builder_schedule::State::Scheduled(ui::Scheduled {
                label: "Daily at 8:00".into(),
            })),
        }),
        accounts: vec![
            account("personal", "Personal", MailProvider::Gmail, "Gmail", true),
            account("laurentian", "Laurentian", MailProvider::Outlook, "Outlook", true),
            account("icloud", "iCloud", MailProvider::Icloud, "iCloud Mail", false),
        ],
        permissions,
        ask_before_new_recipient: true,
        test_model: Some(model("Claude Sonnet", ModelProvider::Anthropic)),
        dry_run: Some(ui::agent_builder_view::DryRun::Idle(ui::DryRunIdle {})),
        summary: vec![
            summary(
                "Does",
                "Add professors’ deadlines to your calendar, remind you the day before",
            ),
            summary(
                "Wakes",
                "Mail sorted into a category or folder, Calendar event approaching, Daily at 8:00",
            ),
            summary("Auto", "Read and label, Calendar"),
            summary("Asks first", "Draft replies, Send"),
        ],
        // The steps are navigation, added by the view.
        steps: vec![],
    }
}

fn connect() -> Connect {
    use ui::connect_imap_form::Server;
    let imap = |provider, name, password_hint| ConnectProvider {
        provider,
        name,
        method: "IMAP, app password",
        how: How::Imap {
            password_hint,
            server: Server::Known(ui::ServerKnown {}),
        },
    };
    Connect {
        providers: vec![
            ConnectProvider {
                provider: MailProvider::Gmail,
                name: "Gmail",
                method: "Sign in with Google",
                how: How::SignIn { host: "Google" },
            },
            ConnectProvider {
                provider: MailProvider::Outlook,
                name: "Outlook",
                method: "Sign in with Microsoft",
                how: How::SignIn { host: "Microsoft" },
            },
            imap(
                MailProvider::Icloud,
                "iCloud Mail",
                "Make one in your Apple Account under Sign-In and Security",
            ),
            imap(
                MailProvider::Yahoo,
                "Yahoo Mail",
                "Make one in Yahoo Account Security",
            ),
            imap(
                MailProvider::Fastmail,
                "Fastmail",
                "Make one in Fastmail under Privacy and Security",
            ),
            ConnectProvider {
                provider: MailProvider::OtherImap,
                name: "Other IMAP",
                method: "IMAP, app password",
                how: How::Imap {
                    password_hint: "Your provider’s app password, or the mailbox password if it has none",
                    server: Server::Entry(ui::ServerEntry {
                        default_port: "993".into(),
                    }),
                },
            },
        ],
        sync_percent: 64,
        sync_stats: stats(&[
            ("2,140", "messages"),
            ("38", "folders"),
            ("Indexing", "search"),
        ]),
    }
}
