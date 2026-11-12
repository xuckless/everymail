//! Builds the views the Frontend renders: pure functions from the navigation state and the data.

mod agents;
mod calendar;
mod mail;
mod settings;

pub use settings::settings;

use crate::{
    connection::{self, Connection},
    demo::{How, Place, World},
    navigation::{self, ConnectStep, Listing, Mailbox, Navigation, Screen},
    path,
    preferences::Preferences,
    proto::ui::v1::{
        self as ui, Destination, MailProvider, connect_view::Step, nav_folder::Kind,
        shell_view::Screen as View,
    },
};

pub fn shell(
    navigation: &Navigation,
    connection: Connection,
    preferences: Preferences,
    world: &World,
) -> ui::ShellView {
    ui::ShellView {
        connection: Some(connection::view(connection)),
        theme: preferences.theme.into(),
        density: preferences.density.into(),
        navigation: Some(chrome(navigation, world)),
        path: path::path(&navigation::route(navigation)),
        screen: Some(screen(navigation, preferences, world)),
    }
}

fn screen(navigation: &Navigation, preferences: Preferences, world: &World) -> View {
    let theme = preferences.theme;
    match &navigation.screen {
        Screen::SignIn => View::SignIn(ui::SignInView {}),
        Screen::Connect(step) => View::Connect(connect(*step, world)),
        Screen::Inbox => View::Inbox(mail::inbox(navigation, theme, world)),
        // Navigation only opens threads that exist.
        Screen::Thread { id, expanded } => match world.thread(id) {
            Some(thread) => View::Thread(mail::thread(thread, expanded, theme, world)),
            None => View::Inbox(mail::inbox(navigation, theme, world)),
        },
        Screen::Agents => View::Agents(agents::agents(navigation, world)),
        Screen::AgentBuilder => View::AgentBuilder(agents::builder(navigation.step, world)),
        Screen::Settings => View::Settings(settings(navigation.section, preferences, world)),
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

fn chrome(navigation: &Navigation, world: &World) -> ui::Navigation {
    let inbox = match navigation.screen {
        Screen::Inbox => Some(&navigation.listing),
        _ => None,
    };
    let unread = |account: Option<&str>| {
        count(
            world
                .threads
                .iter()
                .filter(|t| {
                    t.unread
                        && matches!(t.place, Place::Inbox(_))
                        && account.is_none_or(|a| a == t.account)
                })
                .count(),
        )
    };
    let destination = match navigation.screen {
        Screen::SignIn | Screen::Connect(_) => Destination::None,
        Screen::Inbox | Screen::Thread { .. } | Screen::Quarantine => Destination::Inbox,
        Screen::Calendar => Destination::Calendar,
        Screen::Briefing => Destination::Briefing,
        Screen::Agents | Screen::AgentBuilder => Destination::Agents,
        Screen::Settings => Destination::Settings,
    };
    let agents = |system: bool| {
        world
            .agents
            .iter()
            .filter(|a| matches!(a.detail.kind, Some(ui::agent_detail::Kind::System(_))) == system)
            .map(|a| ui::NavAgent {
                id: a.detail.id.clone(),
                name: a.detail.name.clone(),
                status: a.detail.status,
                note: a.note.into(),
            })
            .collect()
    };
    ui::Navigation {
        user_name: world.user_name.into(),
        destination: destination.into(),
        all_inboxes: Some(ui::NavMailbox {
            unread: unread(None),
            current: matches!(inbox, Some(Listing::Mailbox(Mailbox::All, _))),
        }),
        accounts: world
            .accounts
            .iter()
            .map(|a| ui::NavAccount {
                id: a.id.into(),
                name: a.name.into(),
                provider: a.provider.into(),
                unread: unread(Some(a.id)),
                current: matches!(inbox, Some(Listing::Mailbox(Mailbox::Account(id), _)) if id == a.id),
            })
            .collect(),
        calendar: Some(ui::NavCalendar {
            next_event: world.calendar.next_event.into(),
            current: navigation.screen == Screen::Calendar,
        }),
        briefing: Some(ui::NavBriefing {
            unread: world.briefing_unread,
            current: navigation.screen == Screen::Briefing,
        }),
        folders: world
            .folders
            .iter()
            .map(|f| {
                let listing = Listing::Folder(f.id.into());
                let (count, current) = match f.kind {
                    Kind::Quarantine(_) => (
                        count(world.quarantine.len()),
                        navigation.screen == Screen::Quarantine,
                    ),
                    Kind::Archive(_) => (
                        count(
                            navigation::Catalog::listing(world, &listing)
                                .unwrap_or_default()
                                .len(),
                        ),
                        inbox == Some(&listing),
                    ),
                    Kind::Starred(_) | Kind::Sent(_) | Kind::User(_) => {
                        (String::new(), inbox == Some(&listing))
                    }
                };
                ui::NavFolder {
                    id: f.id.into(),
                    name: f.name.into(),
                    count,
                    current,
                    kind: Some(f.kind),
                }
            })
            .collect(),
        system_agents: agents(true),
        user_agents: agents(false),
        agents_waiting: world.approvals.iter().any(|a| {
            matches!(
                a.item.state.as_ref().and_then(|s| s.state.as_ref()),
                Some(ui::approval_state::State::Waiting(_))
            )
        }),
    }
}

fn connect(step: ConnectStep, world: &World) -> ui::ConnectView {
    let providers = &world.connect.providers;
    let list: Vec<ui::ConnectProvider> = providers
        .iter()
        .map(|p| ui::ConnectProvider {
            provider: p.provider.into(),
            name: p.name.into(),
            method: p.method.into(),
        })
        .collect();
    let picking = || Step::Picking(ui::ConnectPicking {});
    let find = |provider: MailProvider| providers.iter().find(|p| p.provider == provider);
    let step = match step {
        ConnectStep::Picking => picking(),
        ConnectStep::Connecting(provider) => match find(provider) {
            Some(p) => match &p.how {
                How::SignIn { host } => Step::SigningIn(ui::ConnectSigningIn {
                    provider: provider.into(),
                    title: format!("Finish signing in with {host}"),
                    note: format!(
                        "We opened {host}’s own sign-in page in your browser. EveryMail never sees your password"
                    ),
                    waiting: format!("Waiting for {host}"),
                }),
                How::Imap {
                    password_hint,
                    server,
                } => Step::ImapForm(ui::ConnectImapForm {
                    provider: provider.into(),
                    password_hint: (*password_hint).into(),
                    server: Some(server.clone()),
                    title: format!("Connect {}", p.name),
                }),
            },
            // Navigation only holds providers the Frontend could pick.
            None => picking(),
        },
        ConnectStep::Syncing(provider) => {
            let name = find(provider).map_or("", |p| p.name);
            Step::Syncing(ui::ConnectSyncing {
                provider: provider.into(),
                percent: world.connect.sync_percent,
                stats: world.connect.sync_stats.clone(),
                title: format!("Syncing {name}"),
            })
        }
    };
    ui::ConnectView {
        connected: world
            .accounts
            .iter()
            .map(|a| ui::ConnectedAccount {
                id: a.id.into(),
                name: a.name.into(),
                provider: a.provider.into(),
                provider_name: a.provider_name.into(),
                status: Some(ui::AccountStatus {
                    state: Some(a.status.clone()),
                }),
            })
            .collect(),
        step: Some(step),
        providers: list,
    }
}
