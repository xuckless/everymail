//! The sample world the app skeleton shows: plain static data, no randomness and no clock. "Now" is
//! Wednesday, November 11 2026, shortly before 10:00. Only the view-building layer reads it, through
//! these types and `Catalog`; the local replica replaces it later.

mod world;

pub use world::world;

use crate::{
    navigation::{Catalog, Listing, Mailbox},
    proto::ui::v1::{self as ui, InboxCategory, MailProvider, NoteKind},
};

pub struct World {
    pub user_name: &'static str,
    pub accounts: Vec<Account>,
    pub folders: Vec<Folder>,
    /// Newest first.
    pub threads: Vec<Thread>,
    /// System agents first.
    pub agents: Vec<Agent>,
    pub approvals: Vec<Approval>,
    pub compose: Compose,
    pub calendar: Calendar,
    pub quarantine: Vec<Quarantined>,
    /// Newest first.
    pub editions: Vec<Edition>,
    pub briefing_unread: bool,
    pub settings: Settings,
    pub builder: ui::AgentBuilderView,
    pub connect: Connect,
}

pub struct Account {
    pub id: &'static str,
    pub name: &'static str,
    pub provider: MailProvider,
    pub provider_name: &'static str,
    pub status: ui::account_status::State,
    pub signature: ui::settings_account::Signature,
}

pub struct Folder {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: ui::nav_folder::Kind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Place {
    Inbox(InboxCategory),
    Archive,
}

pub struct Thread {
    pub id: &'static str,
    pub account: &'static str,
    pub place: Place,
    /// Folders the user created that hold it.
    pub folders: &'static [&'static str],
    pub starred: bool,
    pub unread: bool,
    pub from: &'static str,
    pub subject: &'static str,
    pub time: &'static str,
    pub note: (&'static str, NoteKind),
    pub summary: &'static str,
    pub messages: Vec<Message>,
    pub quick_replies: &'static [&'static str],
}

pub enum Author {
    You,
    Contact(&'static str),
}

pub struct Message {
    pub id: &'static str,
    pub author: Author,
    pub preview: &'static str,
    pub time: &'static str,
    /// Paragraphs of plain text; a line break inside one is kept.
    pub body: &'static [&'static str],
    /// The calendar event this message invites to.
    pub invitation: Option<&'static str>,
}

pub struct Agent {
    /// "On every new mail"
    pub next: &'static str,
    /// The sidebar note: "2", "18:00"
    pub note: &'static str,
    pub last_run: &'static str,
    /// The provider of its model, whose key chat needs.
    pub provider: ui::ModelProvider,
    pub chat: Option<Chat>,
    pub detail: ui::AgentDetail,
}

/// The conversation of an agent that can chat.
pub struct Chat {
    /// The pane the agent opens on.
    pub opens_on: ui::AgentDetailPane,
    /// Oldest first.
    pub turns: Vec<ui::ChatTurn>,
}

pub struct Approval {
    pub item: ui::ApprovalItem,
    /// The drafted mail of a send approval.
    pub draft: &'static [&'static str],
}

pub struct Compose {
    pub models: Vec<ui::Model>,
    pub model: &'static str,
    pub tones: &'static [&'static str],
    pub tone: &'static str,
}

pub struct Calendar {
    pub month: &'static str,
    pub range: &'static str,
    pub next_event: &'static str,
    pub first_hour: u32,
    pub last_hour: u32,
    pub sources: Vec<ui::CalendarSource>,
    pub days: Vec<Day>,
    pub events: Vec<CalendarEvent>,
}

pub struct Day {
    /// The date: "2026-11-09"
    pub id: &'static str,
    pub weekday: &'static str,
    pub day: &'static str,
    /// "Nov"
    pub month: &'static str,
    pub today: bool,
}

pub struct CalendarEvent {
    pub id: &'static str,
    pub calendar: &'static str,
    /// Index into `Calendar::days`.
    pub day: usize,
    pub start: u32,
    pub end: u32,
    pub title: &'static str,
    pub place: &'static str,
    pub guests: &'static [&'static str],
    pub invitation: Option<ui::Invitation>,
}

pub struct Quarantined {
    pub id: &'static str,
    pub from: &'static str,
    pub account: &'static str,
    pub time: &'static str,
    pub subject: &'static str,
    pub reason: &'static str,
    pub caught_by: &'static str,
    pub why: &'static str,
    pub outcome: &'static str,
    /// When EveryMail drops its copy.
    pub drop: &'static str,
    pub body: &'static [&'static str],
}

pub struct Edition {
    pub id: &'static str,
    pub label: &'static str,
    pub byline: &'static str,
    pub activity: Vec<ui::BriefingLine>,
    pub caught_mail: bool,
    pub waiting: Vec<ui::BriefingWaiting>,
    pub newsletters: Vec<ui::BriefingLine>,
    pub updates: Vec<ui::BriefingLine>,
    pub calendar_title: &'static str,
    pub calendar: Vec<ui::BriefingEvent>,
}

pub struct Settings {
    pub defaults: Vec<ui::Permission>,
    pub notify_when_waiting: bool,
    pub agents_paused: bool,
    pub rules: Vec<ui::Rule>,
    pub blocked: Vec<ui::BlockedSender>,
    pub keys: Vec<(ui::ModelProvider, &'static str, ui::provider_key::State)>,
    pub tasks: Vec<ui::TaskModel>,
    pub budget: ui::models_settings::Budget,
}

pub struct Connect {
    pub providers: Vec<ConnectProvider>,
    pub sync_percent: u32,
    pub sync_stats: Vec<ui::Stat>,
}

pub struct ConnectProvider {
    pub provider: MailProvider,
    pub name: &'static str,
    pub method: &'static str,
    pub how: How,
}

pub enum How {
    SignIn {
        host: &'static str,
    },
    Imap {
        password_hint: &'static str,
        server: ui::connect_imap_form::Server,
    },
}

impl World {
    pub fn account(&self, id: &str) -> Option<&Account> {
        self.accounts.iter().find(|a| a.id == id)
    }

    pub fn thread(&self, id: &str) -> Option<&Thread> {
        self.threads.iter().find(|t| t.id == id)
    }

    /// Some model key is set, so summaries, AI compose and user agents run.
    pub fn has_key(&self) -> bool {
        self.settings
            .keys
            .iter()
            .any(|(_, _, state)| matches!(state, ui::provider_key::State::Set(_)))
    }

    pub fn event(&self, id: &str) -> Option<&CalendarEvent> {
        self.calendar.events.iter().find(|e| e.id == id)
    }

    /// The approvals that carry a draft answering this thread.
    pub fn drafts(&self, thread: &str) -> impl Iterator<Item = &Approval> {
        self.approvals.iter().filter(move |a| {
            matches!(&a.item.action, Some(ui::approval_item::Action::Send(send)) if send.thread_id == thread)
        })
    }

    /// The plain-text paragraphs of a message or a quarantined mail.
    pub fn body(&self, id: &str) -> Option<&'static [&'static str]> {
        self.threads
            .iter()
            .flat_map(|t| &t.messages)
            .find(|m| m.id == id)
            .map(|m| m.body)
            .or_else(|| self.quarantine.iter().find(|q| q.id == id).map(|q| q.body))
    }

    fn shown(&self, calendar: &str) -> bool {
        self.calendar
            .sources
            .iter()
            .any(|s| s.id == calendar && s.shown)
    }
}

impl Catalog for World {
    fn listing(&self, listing: &Listing) -> Option<Vec<&str>> {
        let holds: Box<dyn Fn(&Thread) -> bool> = match listing {
            Listing::Mailbox(mailbox, category) => {
                let account = match mailbox {
                    Mailbox::All => None,
                    Mailbox::Account(id) => Some(self.account(id)?.id),
                };
                let place = Place::Inbox(*category);
                Box::new(move |t| t.place == place && account.is_none_or(|a| a == t.account))
            }
            Listing::Folder(id) => {
                let folder = self.folders.iter().find(|f| f.id == id)?;
                let folder_id = folder.id;
                use ui::nav_folder::Kind;
                match folder.kind {
                    Kind::Starred(_) => Box::new(|t| t.starred),
                    Kind::Sent(_) => {
                        Box::new(|t| t.messages.iter().any(|m| matches!(m.author, Author::You)))
                    }
                    Kind::Archive(_) => Box::new(|t| t.place == Place::Archive),
                    Kind::User(_) => Box::new(move |t| t.folders.contains(&folder_id)),
                    // Quarantine is its own screen, not a listing of threads.
                    Kind::Quarantine(_) => return None,
                }
            }
        };
        Some(
            self.threads
                .iter()
                .filter(|t| holds(t))
                .map(|t| t.id)
                .collect(),
        )
    }

    fn messages(&self, thread: &str) -> Option<Vec<&str>> {
        Some(self.thread(thread)?.messages.iter().map(|m| m.id).collect())
    }

    /// The latest message and every invitation.
    fn opened(&self, thread: &str) -> Vec<&str> {
        let Some(thread) = self.thread(thread) else {
            return vec![];
        };
        let last = thread.messages.len().saturating_sub(1);
        thread
            .messages
            .iter()
            .enumerate()
            .filter(|(i, m)| *i == last || m.invitation.is_some())
            .map(|(_, m)| m.id)
            .collect()
    }

    fn agents(&self) -> Vec<&str> {
        self.agents.iter().map(|a| a.detail.id.as_str()).collect()
    }

    fn chat(&self, agent: &str) -> Option<ui::AgentDetailPane> {
        self.agents
            .iter()
            .find(|a| a.detail.id == agent)?
            .chat
            .as_ref()
            .map(|c| c.opens_on)
    }

    fn day_of(&self, event: &str) -> Option<&str> {
        let event = self.calendar.events.iter().find(|e| e.id == event)?;
        self.calendar.days.get(event.day).map(|d| d.id)
    }

    fn events(&self) -> Vec<&str> {
        self.calendar
            .events
            .iter()
            .filter(|e| self.shown(e.calendar))
            .map(|e| e.id)
            .collect()
    }

    fn quarantined(&self) -> Vec<&str> {
        self.quarantine.iter().map(|q| q.id).collect()
    }

    fn editions(&self) -> Vec<&str> {
        self.editions.iter().map(|e| e.id).collect()
    }

    fn days(&self) -> Vec<&str> {
        self.calendar.days.iter().map(|d| d.id).collect()
    }

    fn today(&self) -> Option<&str> {
        self.calendar.days.iter().find(|d| d.today).map(|d| d.id)
    }
}
