//! Builds the views the Frontend renders: pure functions from the navigation state, the data and
//! the short-lived state.

mod agents;
mod calendar;
mod compose;
mod mail;
mod settings;

use crate::{
    action::selected,
    connection::{self, Connection},
    live::{self, Field, Live, Resolution, Target},
    navigation::{self, ConnectStep, Disclosure, Entry, Listing, Mailbox, Navigation, Screen},
    path,
    preferences::Preferences,
    proto::ui::v1::{
        self as ui, Destination, MailProvider, action_target::Target as To,
        connect_imap_form::Server, connect_view::Step, nav_folder::Kind,
        shell_view::Screen as View,
    },
    time,
    world::{Account, AccountState, FolderKind, Identity, SendState, World, mail_provider_name},
};

/// The engine's own state a view is built from, besides the data and the short-lived state.
pub struct Shell<'a> {
    pub navigation: &'a Navigation,
    pub connection: Connection,
    pub preferences: Preferences,
    pub entry: Entry,
    /// The new mail shortcut as the device writes it.
    pub shortcut: &'static str,
}

pub fn shell(shell: &Shell, world: &World, live: &Live, now_ms: i64) -> ui::ShellView {
    let navigation = shell.navigation;
    ui::ShellView {
        connection: Some(connection::view(shell.connection)),
        theme: shell.preferences.theme.into(),
        density: shell.preferences.density.into(),
        sidebar: shell.preferences.sidebar.into(),
        navigation: Some(chrome(navigation, world, live, shell.shortcut)),
        path: path::path(&navigation::route(navigation)),
        history: match shell.entry {
            Entry::Push => ui::HistoryStep::Push,
            Entry::Replace => ui::HistoryStep::Replace,
        }
        .into(),
        screen: Some(screen(navigation, shell.preferences, world, live)),
        outcomes: outcomes(live),
        notice: Some(notice(live, now_ms)),
        composer: Some(compose::composer(&navigation.composer, world)),
    }
}

/// Open or closed, as the navigation holds it.
fn disclosure(navigation: &Navigation, disclosure: &Disclosure) -> ui::Disclosure {
    use ui::disclosure::State;
    ui::Disclosure {
        state: Some(if navigation.open.contains(disclosure) {
            State::Open(ui::DisclosureOpen {})
        } else {
            State::Closed(ui::DisclosureClosed {})
        }),
    }
}

fn screen(navigation: &Navigation, preferences: Preferences, world: &World, live: &Live) -> View {
    let theme = preferences.theme;
    match &navigation.screen {
        Screen::SignIn => View::SignIn(sign_in(live)),
        Screen::Connect(step) => View::Connect(connect(*step, world, live)),
        Screen::Inbox => View::Inbox(mail::inbox(navigation, theme, world, live)),
        // Navigation only opens threads that exist.
        Screen::Thread { id, expanded } => match world.thread(id) {
            Some(thread) => View::Thread(mail::thread(
                thread, expanded, navigation, theme, world, live,
            )),
            None => View::Inbox(mail::inbox(navigation, theme, world, live)),
        },
        Screen::Agents => View::Agents(agents::agents(None, navigation, world, live)),
        // Navigation only opens agents that exist.
        Screen::Agent { id, pane } => View::Agents(agents::agents(
            world.agent(id).map(|a| (a, *pane)),
            navigation,
            world,
            live,
        )),
        Screen::Approvals => View::Approvals(agents::approvals(world, live)),
        Screen::AgentBuilder => View::AgentBuilder(agents::builder(navigation.step, world, live)),
        Screen::Settings => {
            View::Settings(settings::settings(navigation, preferences, world, live))
        }
        Screen::Calendar => View::Calendar(calendar::calendar(navigation, world)),
        Screen::Quarantine => {
            View::Quarantine(mail::quarantine(&navigation.quarantined, theme, world))
        }
        Screen::Briefing => View::Briefing(agents::briefing(&navigation.edition, world)),
    }
}

/// A count as the sidebar prints it: nothing for zero.
fn count(n: usize) -> String {
    if n == 0 { String::new() } else { n.to_string() }
}

/// "1 message", "4 messages"
fn plural(n: usize, noun: &str) -> String {
    if n == 1 {
        format!("1 {noun}")
    } else {
        format!("{n} {noun}s")
    }
}

/// "182,400"
fn grouped(n: i64) -> String {
    let digits = n.unsigned_abs().to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    if n < 0 { format!("-{out}") } else { out }
}

fn target(target: &Target) -> ui::ActionTarget {
    let to = match target {
        Target::Thread(id) => To::ThreadId(id.clone()),
        Target::Approval(id) => To::ApprovalId(id.clone()),
        Target::Event(id) => To::EventId(id.clone()),
        Target::Agent(id) => To::AgentId(id.clone()),
        Target::Draft(id) => To::DraftId(id.clone()),
        Target::Account(id) => To::AccountId(id.clone()),
        Target::Key(provider) => To::Key((*provider).into()),
        Target::Form(form) => To::Form((*form).into()),
    };
    ui::ActionTarget { target: Some(to) }
}

fn outcomes(live: &Live) -> Vec<ui::ActionOutcome> {
    use ui::action_outcome::State;
    live.outcomes
        .iter()
        .map(|o| ui::ActionOutcome {
            target: Some(target(&o.target)),
            state: Some(match &o.result {
                Resolution::Done(text) => State::Done(ui::OutcomeDone { text: text.clone() }),
                Resolution::Failed(reason) => State::Failed(ui::OutcomeFailed {
                    reason: reason.clone(),
                }),
            }),
        })
        .collect()
}

fn notice(live: &Live, now_ms: i64) -> ui::Notice {
    use ui::notice::State;
    ui::Notice {
        state: Some(match &live.notice {
            live::Notice::None => State::None(ui::NoNotice {}),
            live::Notice::Undo { text, until, .. } => State::Undo(ui::UndoNotice {
                text: text.clone(),
                action: "Undo".into(),
                remaining_ms: u32::try_from((until - now_ms).max(0)).unwrap_or(u32::MAX),
            }),
            live::Notice::Info { text, .. } => State::Info(ui::InfoNotice { text: text.clone() }),
        }),
    }
}

fn sign_in(live: &Live) -> ui::SignInView {
    use ui::sign_in_view::State;
    ui::SignInView {
        state: Some(match &live.sign_in {
            live::SignIn::Idle => State::Idle(ui::SignInIdle {}),
            live::SignIn::Waiting(identity) => {
                let (provider, name) = match identity {
                    Identity::Google => (ui::SignInProvider::Google, "Google"),
                    Identity::Microsoft => (ui::SignInProvider::Microsoft, "Microsoft"),
                };
                State::SigningIn(ui::SigningIn {
                    provider: provider.into(),
                    text: format!("Waiting for {name}"),
                })
            }
            live::SignIn::Failed(reason) => State::Failed(ui::SignInFailed {
                reason: reason.clone(),
            }),
        }),
    }
}

/// "Synced 2 min ago"
fn synced(account: &Account, now: i64) -> String {
    match (now - account.synced_at) / 60 {
        ..=0 => "Synced just now".into(),
        minutes @ 1..=59 => format!("Synced {minutes} min ago"),
        _ => format!("Synced {}", time::long(account.synced_at, now)),
    }
}

/// The account's line: "Synced 2 min ago", "App password expired", "Syncing · 64%"
fn state_label(account: &Account, now: i64) -> String {
    match &account.state {
        AccountState::Synced => synced(account, now),
        AccountState::SignInExpired(reason) => reason.clone(),
        AccountState::Syncing(percent) => format!("Syncing · {percent}%"),
    }
}

/// An account's state, with its line.
fn account_state(account: &Account, now: i64) -> ui::AccountState {
    use ui::account_state::State;
    let label = state_label(account, now);
    ui::AccountState {
        state: Some(match account.state {
            AccountState::Synced => State::Synced(ui::AccountSynced { label }),
            AccountState::SignInExpired(_) => {
                State::SignInExpired(ui::AccountSignInExpired { label })
            }
            AccountState::Syncing(_) => State::Syncing(ui::AccountSyncing { label }),
        }),
    }
}

/// Every half hour of the day, for a time menu.
fn time_choices() -> Vec<ui::TimeChoice> {
    (0..48)
        .map(|i| ui::TimeChoice {
            minute: i * 30,
            label: time::clock(i * 30),
        })
        .collect()
}

fn chrome(navigation: &Navigation, world: &World, live: &Live, shortcut: &str) -> ui::Navigation {
    let inbox = match navigation.screen {
        Screen::Inbox => Some(navigation.listing.scope()),
        _ => None,
    };
    let destination = match navigation.screen {
        Screen::SignIn | Screen::Connect(_) => Destination::None,
        Screen::Inbox | Screen::Thread { .. } | Screen::Quarantine => Destination::Inbox,
        Screen::Calendar => Destination::Calendar,
        Screen::Briefing => Destination::Briefing,
        Screen::Agents | Screen::Agent { .. } | Screen::AgentBuilder | Screen::Approvals => {
            Destination::Agents
        }
        Screen::Settings => Destination::Settings,
    };
    let agents = |system: bool| {
        world
            .agents
            .iter()
            .filter(|a| a.system() == system)
            .map(|a| ui::NavAgent {
                id: a.id.clone(),
                name: a.name.clone(),
                status: Some(agents::status(a, world)),
                note: agents::note(a, world),
                current: matches!(&navigation.screen, Screen::Agent { id, .. } if *id == a.id),
            })
            .collect()
    };
    let listed = |listing: &Listing| world.listing(listing).map_or(0, |rows| rows.len());
    let now = world.now;
    let trash = world.folders.iter().find(|f| f.kind == FolderKind::Trash);
    ui::Navigation {
        user_name: world.user.clone().unwrap_or_default(),
        destination: destination.into(),
        all_inboxes: Some(ui::NavMailbox {
            unread: count(world.unread(None, None)),
            current: matches!(inbox, Some(Listing::Mailbox(Mailbox::All, _))),
        }),
        accounts: world
            .accounts
            .iter()
            .map(|a| ui::NavAccount {
                id: a.id.clone(),
                name: a.name.clone(),
                provider: a.provider.into(),
                unread: count(world.unread(Some(&a.id), None)),
                current: matches!(inbox, Some(Listing::Mailbox(Mailbox::Account(id), _)) if *id == a.id),
            })
            .collect(),
        calendar: Some(ui::NavCalendar {
            next_event: calendar::next_event(world),
            current: navigation.screen == Screen::Calendar,
        }),
        briefing: Some(ui::NavBriefing {
            unread: world.editions.iter().any(|e| e.unread),
            current: navigation.screen == Screen::Briefing,
        }),
        folders: world
            .folders
            .iter()
            .filter_map(|f| {
                let listing = Listing::Folder(f.id.clone());
                let (kind, count, current) = match f.kind {
                    FolderKind::Quarantine => (
                        Kind::Quarantine(ui::QuarantineFolder {}),
                        count(world.quarantine.len()),
                        navigation.screen == Screen::Quarantine,
                    ),
                    FolderKind::Archive => (
                        Kind::Archive(ui::ArchiveFolder {}),
                        count(listed(&listing)),
                        inbox == Some(&listing),
                    ),
                    FolderKind::Starred => (
                        Kind::Starred(ui::StarredFolder {}),
                        String::new(),
                        inbox == Some(&listing),
                    ),
                    FolderKind::Sent => (
                        Kind::Sent(ui::SentFolder {}),
                        String::new(),
                        inbox == Some(&listing),
                    ),
                    FolderKind::User => (
                        Kind::User(ui::UserFolder {}),
                        String::new(),
                        inbox == Some(&listing),
                    ),
                    // Trash has its own entry.
                    FolderKind::Trash => return None,
                };
                Some(ui::NavFolder {
                    id: f.id.clone(),
                    name: f.name.clone(),
                    count,
                    current,
                    kind: Some(kind),
                })
            })
            .collect(),
        system_agents: agents(true),
        user_agents: agents(false),
        approvals: Some(agents::nav_approvals(
            navigation.screen == Screen::Approvals,
            world,
        )),
        drafts: world
            .drafts
            .iter()
            .map(|d| {
                use ui::nav_draft::Send;
                let saved = || {
                    if d.subject.trim().is_empty() {
                        time::short(d.updated_at, now)
                    } else {
                        format!("{} · {}", d.subject.trim(), time::short(d.updated_at, now))
                    }
                };
                let (detail, send) = match &d.send {
                    None => (saved(), Send::Unsent(ui::DraftUnsent {})),
                    Some(SendState::Undoable(_)) => (
                        "Sending".into(),
                        Send::Held(ui::DraftHeld {
                            action: "Undo".into(),
                        }),
                    ),
                    Some(SendState::Scheduled(at)) => (
                        format!("Sends {}", time::long(*at, now)),
                        Send::Held(ui::DraftHeld {
                            action: "Cancel".into(),
                        }),
                    ),
                    Some(SendState::Sending) => ("Sending".into(), Send::Sending(ui::DraftSending {})),
                    Some(SendState::Failed(failure)) => (
                        format!("Not sent: {failure}"),
                        Send::NotSent(ui::DraftNotSent {}),
                    ),
                };
                ui::NavDraft {
                    draft_id: d.id.clone(),
                    title: if d.recipients.trim().is_empty() {
                        "No recipient".into()
                    } else {
                        format!("To {}", d.recipients.trim())
                    },
                    detail,
                    send: Some(send),
                }
            })
            .collect(),
        trash: trash.map(|f| {
            let listing = Listing::Folder(f.id.clone());
            ui::NavTrash {
                folder_id: f.id.clone(),
                name: f.name.clone(),
                count: count(listed(&listing)),
                current: inbox == Some(&listing),
            }
        }),
        new_folder: Some(disclosure(navigation, &Disclosure::NewFolder)),
        new_folder_revision: live.revisions.of(&Field::NewFolder),
        new_mail_shortcut: shortcut.into(),
    }
}

/// The providers the connect screen offers, and how each connects.
enum How {
    SignIn {
        host: &'static str,
    },
    Imap {
        password_hint: &'static str,
        server_entry: bool,
    },
}

const PROVIDERS: [(MailProvider, &str, How); 6] = [
    (
        MailProvider::Gmail,
        "Sign in with Google",
        How::SignIn { host: "Google" },
    ),
    (
        MailProvider::Outlook,
        "Sign in with Microsoft",
        How::SignIn { host: "Microsoft" },
    ),
    (
        MailProvider::Icloud,
        "IMAP, app password",
        How::Imap {
            password_hint: "Make one in your Apple Account under Sign-In and Security",
            server_entry: false,
        },
    ),
    (
        MailProvider::Yahoo,
        "IMAP, app password",
        How::Imap {
            password_hint: "Make one in Yahoo Account Security",
            server_entry: false,
        },
    ),
    (
        MailProvider::Fastmail,
        "IMAP, app password",
        How::Imap {
            password_hint: "Make one in Fastmail under Privacy and Security",
            server_entry: false,
        },
    ),
    (
        MailProvider::OtherImap,
        "IMAP, app password",
        How::Imap {
            password_hint: "Your provider’s app password, or the mailbox password if it has none",
            server_entry: true,
        },
    ),
];

fn connect(step: ConnectStep, world: &World, live: &Live) -> ui::ConnectView {
    let picking = || Step::Picking(ui::ConnectPicking {});
    let find = |provider: MailProvider| PROVIDERS.iter().find(|p| p.0 == provider);
    let step = match step {
        ConnectStep::Picking => picking(),
        ConnectStep::Connecting(provider) => match find(provider) {
            Some((_, _, How::SignIn { host })) => Step::SigningIn(ui::ConnectSigningIn {
                provider: provider.into(),
                title: format!("Finish signing in with {host}"),
                note: format!(
                    "We opened {host}’s own sign-in page in your browser. EveryMail never sees your password"
                ),
                waiting: format!("Waiting for {host} · {}", mail_provider_name(provider)),
            }),
            Some((
                _,
                _,
                How::Imap {
                    password_hint,
                    server_entry,
                },
            )) => {
                use ui::connect_imap_form::Check;
                let name = mail_provider_name(provider);
                Step::ImapForm(ui::ConnectImapForm {
                    provider: provider.into(),
                    password_hint: (*password_hint).into(),
                    server: Some(if *server_entry {
                        Server::Entry(ui::ServerEntry {
                            default_port: "993".into(),
                        })
                    } else {
                        Server::Known(ui::ServerKnown {})
                    }),
                    title: format!("Connect {name}"),
                    check: Some(match &live.imap {
                        live::ImapCheck::Idle => Check::Idle(ui::ImapIdle {}),
                        live::ImapCheck::Checking => Check::Checking(ui::ImapChecking {
                            label: format!("Checking with {name}"),
                        }),
                        live::ImapCheck::Rejected(reason) => Check::Rejected(ui::ImapRejected {
                            reason: reason.clone(),
                        }),
                    }),
                })
            }
            // Navigation only holds providers the Frontend could pick.
            None => picking(),
        },
        ConnectStep::Syncing(provider) => {
            let name = mail_provider_name(provider);
            let account =
                world.accounts.iter().rev().find(|a| {
                    a.provider == provider && matches!(a.state, AccountState::Syncing(_))
                });
            let percent = match account.map(|a| &a.state) {
                Some(AccountState::Syncing(percent)) => *percent,
                _ => 0,
            };
            let messages = account.map_or(0, |a| {
                world
                    .threads
                    .iter()
                    .filter(|t| t.account == a.id)
                    .map(|t| t.messages.len())
                    .sum()
            });
            Step::Syncing(ui::ConnectSyncing {
                provider: provider.into(),
                percent,
                stats: vec![
                    ui::Stat {
                        value: messages.to_string(),
                        label: "messages".into(),
                    },
                    ui::Stat {
                        value: format!("{percent}%"),
                        label: "synced".into(),
                    },
                    ui::Stat {
                        value: if percent < 100 { "Indexing" } else { "Ready" }.into(),
                        label: "search".into(),
                    },
                ],
                title: format!("Syncing {name}"),
            })
        }
    };
    ui::ConnectView {
        connected: world
            .accounts
            .iter()
            .map(|a| ui::ConnectedAccount {
                id: a.id.clone(),
                name: a.name.clone(),
                provider: a.provider.into(),
                provider_name: a.provider_name.clone(),
                state: Some(account_state(a, world.now)),
            })
            .collect(),
        step: Some(step),
        providers: PROVIDERS
            .iter()
            .map(|(provider, method, _)| ui::ConnectProvider {
                provider: (*provider).into(),
                name: mail_provider_name(*provider).into(),
                method: (*method).into(),
            })
            .collect(),
    }
}

/// The selected id, for views.
fn is(selection: &navigation::Selection, id: &str) -> bool {
    selected(selection) == Some(id)
}
