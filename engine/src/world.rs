//! What the query layer returns: the user's data as domain rows, read from the local replica. The
//! views are pure functions over it, and navigation reads it through `Catalog`.

use crate::{
    builder::ScheduleOption,
    date::Date,
    navigation::{AgentPanes, Catalog, Disclosure, Listing, Mailbox, Pane},
    proto::ui::v1::{
        AttachmentKind, CalendarColor, CalendarProvider, InboxCategory, MailProvider,
        ModelProvider, NoteKind, PermissionLevel,
    },
    schedule::Schedule,
};

pub struct World {
    /// The Backend's "now", Unix seconds.
    pub now: i64,
    /// The signed-in user's name; `None` when signed out.
    pub user: Option<String>,
    pub accounts: Vec<Account>,
    pub folders: Vec<Folder>,
    /// Newest first.
    pub threads: Vec<Thread>,
    /// System agents first.
    pub agents: Vec<Agent>,
    /// Newest first.
    pub approvals: Vec<Approval>,
    pub models: Vec<Model>,
    pub calendars: Vec<CalendarSource>,
    /// By day and start.
    pub events: Vec<CalendarEvent>,
    /// Newest first.
    pub quarantine: Vec<Quarantined>,
    /// Newest first.
    pub editions: Vec<Edition>,
    pub news_topics: Vec<String>,
    /// Newest first.
    pub drafts: Vec<Draft>,
    pub settings: Settings,
    pub builder: Builder,
    /// The query searched, with the threads matching it, best first.
    pub search: Option<(String, Vec<String>)>,
    /// When each snooze the Backend offers ends, in menu order.
    pub snoozes: Vec<(Snooze, i64)>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AccountState {
    Synced,
    /// Why: "App password expired"
    SignInExpired(String),
    /// The first sync, this far in percent.
    Syncing(u32),
}

pub struct Account {
    pub id: String,
    pub name: String,
    pub address: String,
    pub provider: MailProvider,
    pub provider_name: String,
    pub state: AccountState,
    pub synced_at: i64,
    pub signature: Option<String>,
}

/// The name the app gives a mail provider: "iCloud Mail".
pub fn mail_provider_name(provider: MailProvider) -> &'static str {
    match provider {
        MailProvider::Gmail => "Gmail",
        MailProvider::Outlook => "Outlook",
        MailProvider::Icloud => "iCloud Mail",
        MailProvider::Yahoo => "Yahoo Mail",
        MailProvider::Fastmail => "Fastmail",
        MailProvider::OtherImap => "Other IMAP",
        MailProvider::Unspecified => "",
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FolderKind {
    Starred,
    Sent,
    Archive,
    Quarantine,
    User,
    Trash,
}

pub struct Folder {
    pub id: String,
    pub name: String,
    pub kind: FolderKind,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Place {
    Inbox,
    Archive,
    Trash,
    /// Only the user's own mail: a conversation the user started.
    Sent,
}

pub struct Thread {
    pub id: String,
    pub account: String,
    pub place: Place,
    pub category: InboxCategory,
    /// Folders the user created that hold it.
    pub folders: Vec<String>,
    /// The people besides the user and the sender on every message.
    pub others: Vec<String>,
    pub unread: bool,
    /// The other party: the sender, or the first recipient of mail the user started.
    pub from: String,
    pub subject: String,
    pub latest_at: i64,
    pub note: Option<(String, NoteKind)>,
    /// Scribe's summary, once written.
    pub summary: Option<String>,
    pub snoozed_until: Option<i64>,
    /// Oldest first.
    pub messages: Vec<Message>,
    pub quick_replies: Vec<String>,
}

impl Thread {
    /// Some message of it is starred.
    pub fn starred(&self) -> bool {
        self.messages.iter().any(|m| m.starred)
    }

    /// Snoozed past `now`.
    pub fn snoozed(&self, now: i64) -> Option<i64> {
        self.snoozed_until.filter(|until| *until > now)
    }

    /// In the inbox, and not snoozed past `now`.
    pub fn in_inbox(&self, now: i64) -> bool {
        self.place == Place::Inbox && self.snoozed(now).is_none()
    }
}

pub struct Message {
    pub id: String,
    pub from_you: bool,
    pub sender: String,
    pub preview: String,
    pub sent_at: i64,
    /// Paragraphs separated by a blank line.
    pub body: String,
    /// The calendar event this message invites to.
    pub invitation: Option<String>,
    pub starred: bool,
    pub attachments: Vec<Attachment>,
}

/// "to you, Priya and Tomas", "to Mira": the user's mail goes to the other party, the rest to the
/// user, and both to everyone else on the thread.
pub fn recipients(other_party: &str, others: &[String], from_you: bool) -> String {
    let first = if from_you { other_party } else { "you" };
    let mut names = vec![first];
    names.extend(others.iter().map(String::as_str));
    let line = match names.split_last() {
        Some((last, rest)) if !rest.is_empty() => format!("{} and {last}", rest.join(", ")),
        _ => first.into(),
    };
    format!("to {line}")
}

pub fn paragraphs(text: &str) -> Vec<&str> {
    text.split("\n\n")
        .filter(|p| !p.trim().is_empty())
        .collect()
}

pub struct Attachment {
    pub id: String,
    pub name: String,
    pub size_bytes: i64,
    pub kind: AttachmentKind,
}

/// The icon a file name gets.
pub fn attachment_kind(name: &str) -> AttachmentKind {
    let extension = name
        .rsplit_once('.')
        .map_or("", |(_, e)| e)
        .to_ascii_lowercase();
    match extension.as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "heic" | "svg" => AttachmentKind::Image,
        "pdf" => AttachmentKind::Pdf,
        "doc" | "docx" | "odt" | "txt" | "md" | "rtf" => AttachmentKind::Document,
        "xls" | "xlsx" | "ods" | "csv" => AttachmentKind::Spreadsheet,
        "ppt" | "pptx" | "odp" | "key" => AttachmentKind::Presentation,
        "zip" | "tar" | "gz" | "7z" | "rar" => AttachmentKind::Archive,
        "ics" => AttachmentKind::Calendar,
        _ => AttachmentKind::Other,
    }
}

/// "2.4 MB", "86 KB"
pub fn size_label(bytes: i64) -> String {
    const MB: i64 = 1_048_576;
    if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else {
        format!("{} KB", (bytes + 1023) / 1024)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DraftKind {
    New,
    Reply,
    ReplyAll,
    Forward,
}

/// A send as the Backend reports it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SendState {
    /// The undo window is open until then.
    Undoable(i64),
    /// Sends then.
    Scheduled(i64),
    Sending,
    /// Why it was not sent.
    Failed(String),
}

impl SendState {
    /// The Backend still holds it, so it can be taken back.
    pub fn held(&self) -> bool {
        matches!(self, SendState::Undoable(_) | SendState::Scheduled(_))
    }
}

pub struct Draft {
    pub id: String,
    pub kind: DraftKind,
    pub thread: Option<String>,
    /// The message replied to or forwarded.
    pub message: Option<String>,
    /// The approval whose drafted reply this is.
    pub approval: Option<String>,
    pub account: String,
    pub recipients: String,
    pub subject: String,
    pub body: String,
    pub updated_at: i64,
    /// (id, name, size in bytes)
    pub attachments: Vec<(String, String, i64)>,
    /// Handed to the Backend.
    pub send: Option<SendState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    pub name: String,
    pub provider: ModelProvider,
}

#[derive(Clone)]
pub struct Stat {
    pub value: String,
    pub label: String,
}

/// The levels the Backend offers for an action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Levels {
    All,
    /// Off and Ask only, for this reason.
    AskAtMost(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Permission {
    /// The action's id: "send"
    pub id: String,
    /// "Send"
    pub label: String,
    pub hint: Option<String>,
    pub level: PermissionLevel,
    pub levels: Levels,
}

/// Whose permissions and accounts: an agent's, the defaults for new agents, or the builder's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Owner {
    Agent(String),
    Defaults,
    Builder,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResultsKind {
    Groups,
    Briefing,
    /// Assistant: its conversation is its result, so it has no Results pane.
    Conversation,
}

#[derive(Clone)]
pub struct Agent {
    pub id: String,
    pub name: String,
    /// A system agent's fixed instructions; `None` for an agent the user made.
    pub built_in: Option<String>,
    pub active: bool,
    /// Appended to a system agent's built-in instructions; the whole of a user agent's.
    pub instructions: String,
    pub model: Model,
    pub schedule: Schedule,
    /// When its schedule last woke it, or when it was made.
    pub woken_at: i64,
    /// What else wakes it.
    pub wakes: Wakes,
    pub key_note: String,
    pub results: ResultsKind,
    pub has_chat: bool,
    pub permissions: Vec<Permission>,
    /// The accounts it can see, by id.
    pub accounts: Vec<String>,
    pub stats: Vec<Stat>,
    pub result_stats: Vec<Stat>,
    /// In group order, then item order.
    pub result_items: Vec<ResultItem>,
    /// Newest first.
    pub runs: Vec<Run>,
    /// Oldest first.
    pub turns: Vec<Turn>,
}

impl Agent {
    pub fn panes(&self) -> AgentPanes {
        use Pane::{Activity, Chat, Results, Settings};
        match (self.results, self.has_chat) {
            (ResultsKind::Conversation, _) => AgentPanes {
                panes: &[Activity, Chat, Settings],
                default: Chat,
            },
            (ResultsKind::Groups | ResultsKind::Briefing, true) => AgentPanes {
                panes: &[Results, Activity, Chat, Settings],
                default: Results,
            },
            (ResultsKind::Groups | ResultsKind::Briefing, false) => AgentPanes {
                panes: &[Results, Activity, Settings],
                default: Results,
            },
        }
    }

    pub fn system(&self) -> bool {
        self.built_in.is_some()
    }

    pub fn running(&self) -> bool {
        self.runs.iter().any(|r| r.outcome == Outcome::Running)
    }

    /// When its schedule wakes it next.
    pub fn next_wake(&self) -> Option<i64> {
        self.schedule.next_wake(self.woken_at)
    }

    /// The running chat turn, if any: one running turn at a time. A turn parked on an approval
    /// does not hold the conversation.
    pub fn running_turn(&self) -> Option<&Turn> {
        self.turns
            .iter()
            .find(|t| matches!(t.state, TurnState::Running(_)))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResultItem {
    pub id: String,
    pub group: String,
    pub at: i64,
    pub title: String,
    pub detail: String,
    pub refs: Vec<Ref>,
}

/// Something a result, run or answer is about.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Ref {
    Thread(String),
    Event(String),
    Approval(String),
    Quarantined(String),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Cause {
    /// New mail: a thread or a quarantined mail.
    Mail(Ref),
    /// Mail sorted into a category or moved into a folder, by filter choice id ("for-you", a
    /// folder's id).
    Sorted(String, Ref),
    /// Mail flagged for the user.
    Flagged(Ref),
    /// The user starred mail.
    Starred(Ref),
    /// Another agent finished this run; its deliverable is that run's summary and these.
    Finished(String, Vec<Ref>),
    /// The user answered this approval.
    Resolved(Ref),
    /// An event is this close, by lead time id: "1-hour".
    Approaching(String, Ref),
    /// The user opened this thread.
    Opened(Ref),
    /// "Weekdays at 9:00"
    Schedule(String),
    /// The user's chat message, by turn id.
    Message(String),
    /// Another agent, by name, handed it this mail.
    Handoff(String, Ref),
    /// An account, by name, was connected.
    Connected(String),
    /// The user pressed Run now.
    RunNow,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Outcome {
    Done,
    /// It left something waiting for the user.
    Waiting,
    Running,
    /// Agents were paused: it finished its step and waits to go on.
    Paused,
    Failed(String),
    /// It reached a safety cap or the token budget.
    Stopped(String),
}

#[derive(Clone)]
pub struct Run {
    pub id: String,
    pub started_at: i64,
    pub cause: Cause,
    /// How long it ran; `None` while it runs.
    pub seconds: Option<i64>,
    pub tokens: i64,
    pub outcome: Outcome,
    pub summary: String,
    pub steps: Vec<Step>,
}

/// How the agent's permissions applied to a step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepPermission {
    /// A read or a search: no permission applies.
    Read,
    /// An action that ran on Auto.
    Auto,
    /// An action that resolved to Ask.
    Asked,
    /// An action that is Off: not done.
    Off,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Step {
    pub verb: String,
    pub what: String,
    pub permission: StepPermission,
    pub refs: Vec<Ref>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TurnState {
    Running(String),
    /// Parked on this approval.
    Waiting(String),
    Answered(String, Vec<Ref>),
    Failed(String),
    Stopped(String),
}

#[derive(Clone)]
pub struct Turn {
    pub id: String,
    pub sent_at: i64,
    pub message: String,
    pub state: TurnState,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ApprovalAction {
    /// A reply to this thread.
    Send(String),
    Unsubscribe,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ApprovalStatus {
    Waiting,
    /// Answered; the Backend is carrying it out.
    Resolving,
    Done,
    /// Why: "You declined. Nothing was sent", "Nobody answered for 7 days"
    Declined(String),
}

#[derive(Debug, PartialEq)]
pub struct Approval {
    pub id: String,
    pub agent: String,
    pub agent_name: String,
    pub created_at: i64,
    pub title: String,
    pub detail: String,
    pub reason: String,
    pub action: ApprovalAction,
    pub status: ApprovalStatus,
    /// The drafted mail of a send, paragraphs separated by a blank line.
    pub draft: String,
}

pub struct CalendarSource {
    pub id: String,
    pub name: String,
    pub provider: CalendarProvider,
    pub provider_name: String,
    pub color: CalendarColor,
    pub shown: bool,
}

/// A snooze the user can pick; the Backend says when each ends.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Snooze {
    LaterToday,
    Tomorrow,
    ThisWeekend,
    NextWeek,
}

/// Who the user signs in with.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Identity {
    Google,
    Microsoft,
}

/// Whether model work runs, as the Backend says: summaries, AI compose and the user's agents are
/// off without a key.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ai {
    On,
    NoKey,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Answer {
    Unanswered,
    Accepted,
    Tentative,
    Declined,
}

pub struct Invitation {
    pub invited_by: String,
    /// "Mira, you and 2 others"
    pub people: String,
    pub answer: Answer,
}

pub struct CalendarEvent {
    pub id: String,
    pub calendar: String,
    pub date: Date,
    pub start: u32,
    pub end: u32,
    pub title: String,
    pub place: String,
    pub guests: Vec<String>,
    pub invitation: Option<Invitation>,
}

pub struct Quarantined {
    pub id: String,
    pub sender: String,
    pub account: String,
    pub received_at: i64,
    pub subject: String,
    pub reason: String,
    pub caught_by: String,
    pub why: String,
    pub outcome: String,
    pub body: String,
    /// When the Backend drops its copy.
    pub drops_at: i64,
}

#[derive(Debug, PartialEq)]
pub struct Line {
    pub source: String,
    pub text: String,
}

#[derive(Debug, PartialEq)]
pub struct Edition {
    pub id: String,
    pub written_at: i64,
    pub byline: String,
    pub unread: bool,
    pub activity: Vec<Line>,
    pub caught_mail: bool,
    /// (approval id, title, detail)
    pub waiting: Vec<(String, String, String)>,
    pub newsletters: Vec<Line>,
    pub updates: Vec<Line>,
    pub calendar_title: String,
    /// (time, title, detail)
    pub calendar: Vec<(String, String, String)>,
    /// Per topic, its items: (headline, source).
    pub news: Vec<(String, Vec<(String, String)>)>,
}

/// What a rule checks on new mail.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Condition {
    /// The sender's name or address contains this, in lower case.
    SenderContains(String),
    /// The sender's domain or a subdomain of it.
    SenderDomain(String),
    /// The subject contains any of these words, in lower case.
    SubjectContains(Vec<String>),
    Newsletter,
    /// The user has not written to the sender, and the mail has a link.
    NewSenderWithLink,
}

/// What a rule does to mail it holds for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuleAction {
    /// Filed in a folder the user made, by id, out of the inbox.
    MoveTo(String),
    Archive,
    Trash,
    MarkImportant,
    /// Wakes the agent, by id.
    HandTo(String),
    /// Kept out of the inbox for News Bearer's newsletters section.
    KeepForBriefing,
}

pub struct Rule {
    pub id: String,
    pub condition: Condition,
    pub actions: Vec<RuleAction>,
    pub enabled: bool,
}

pub struct Blocked {
    pub address: String,
    /// Who blocked it: "Security Guard", "you"
    pub by: String,
    pub at: i64,
}

pub struct Key {
    pub provider: ModelProvider,
    pub name: String,
    pub set: bool,
}

pub struct TaskModel {
    /// "replies"
    pub id: String,
    /// "Writing replies"
    pub label: String,
    pub hint: String,
    pub model: String,
}

pub struct Settings {
    /// The permissions a new agent starts with.
    pub defaults: Vec<Permission>,
    pub notify_when_waiting: bool,
    pub agents_paused: bool,
    pub rules: Vec<Rule>,
    pub blocked: Vec<Blocked>,
    pub keys: Vec<Key>,
    pub tasks: Vec<TaskModel>,
    /// Daily token budget, if one is set.
    pub budget_tokens: Option<i64>,
    /// Tokens today's runs used.
    pub used_tokens: i64,
    /// The tone AI compose writes in.
    pub compose_tone: String,
    pub ai: Ai,
    /// How many days Quarantine keeps a copy, as the Backend says.
    pub quarantine_days: i64,
    /// How many of the latest messages a dry run reads, as the Backend says.
    pub dry_run_messages: i64,
}

/// The inbox categories Gatekeeper sorts into, with their ids and names.
pub const CATEGORIES: [(InboxCategory, &str, &str); 3] = [
    (InboxCategory::ForYou, "for-you", "For you"),
    (InboxCategory::Updates, "updates", "Updates"),
    (InboxCategory::Reading, "reading", "Reading"),
];

fn category(category: InboxCategory) -> (&'static str, &'static str) {
    CATEGORIES
        .iter()
        .find(|(c, ..)| *c == category)
        .map_or(("", ""), |(_, id, name)| (*id, *name))
}

pub fn category_name(c: InboxCategory) -> &'static str {
    category(c).1
}

pub fn category_id(c: InboxCategory) -> &'static str {
    category(c).0
}

pub const TONES: [&str; 3] = ["Brief", "Warm", "Formal"];

/// The tasks a model is picked for, by id.
pub const TASK_REPLIES: &str = "replies";
pub const TASK_AGENTS: &str = "agents";

/// What wakes an agent besides its schedule: the triggers chosen, by id, and the filter choices
/// picked: (trigger id, filter id, choice id). An optional filter not picked takes its default.
#[derive(Clone, Default)]
pub struct Wakes {
    pub triggers: Vec<String>,
    pub filters: Vec<(String, String, String)>,
}

/// The agent being built.
pub struct Builder {
    pub name: String,
    pub instructions: String,
    /// The chosen schedule.
    pub schedule: &'static ScheduleOption,
    /// The custom schedule's interval.
    pub interval_seconds: u32,
    pub ask_before_new_recipient: bool,
    pub wakes: Wakes,
    pub accounts: Vec<String>,
    pub permissions: Vec<Permission>,
}

impl World {
    pub fn today(&self) -> Date {
        crate::time::date(self.now)
    }

    pub fn account(&self, id: &str) -> Option<&Account> {
        self.accounts.iter().find(|a| a.id == id)
    }

    pub fn thread(&self, id: &str) -> Option<&Thread> {
        self.threads.iter().find(|t| t.id == id)
    }

    /// A message with its thread.
    pub fn message(&self, id: &str) -> Option<(&Thread, &Message)> {
        self.threads
            .iter()
            .find_map(|t| t.messages.iter().find(|m| m.id == id).map(|m| (t, m)))
    }

    pub fn agent(&self, id: &str) -> Option<&Agent> {
        self.agents.iter().find(|a| a.id == id)
    }

    /// A run of any agent, with its agent.
    pub fn run(&self, id: &str) -> Option<(&Agent, &Run)> {
        self.agents
            .iter()
            .find_map(|a| a.runs.iter().find(|r| r.id == id).map(|r| (a, r)))
    }

    /// The user has written to this sender.
    pub fn known(&self, sender: &str) -> bool {
        self.threads
            .iter()
            .any(|t| t.from == sender && t.messages.iter().any(|m| m.from_you))
    }

    /// A category's or folder's name by its filter id: "For you", "Receipts".
    pub fn place_name<'a>(&'a self, id: &'a str) -> &'a str {
        CATEGORIES
            .iter()
            .find(|(_, c, _)| *c == id)
            .map(|(_, _, name)| *name)
            .or_else(|| {
                self.folders
                    .iter()
                    .find(|f| f.id == id)
                    .map(|f| f.name.as_str())
            })
            .unwrap_or(id)
    }

    pub fn event(&self, id: &str) -> Option<&CalendarEvent> {
        self.events.iter().find(|e| e.id == id)
    }

    pub fn calendar(&self, id: &str) -> Option<&CalendarSource> {
        self.calendars.iter().find(|c| c.id == id)
    }

    pub fn approval(&self, id: &str) -> Option<&Approval> {
        self.approvals.iter().find(|a| a.id == id)
    }

    pub fn draft(&self, id: &str) -> Option<&Draft> {
        self.drafts.iter().find(|d| d.id == id)
    }

    pub fn model(&self, name: &str) -> Option<&Model> {
        self.models.iter().find(|m| m.name == name)
    }

    /// Model work runs: the Backend has a key.
    pub fn ai_on(&self) -> bool {
        self.settings.ai == Ai::On
    }

    pub fn key_set(&self, provider: ModelProvider) -> bool {
        self.settings
            .keys
            .iter()
            .any(|k| k.provider == provider && k.set)
    }

    /// The model a task uses, by task id.
    pub fn task_model(&self, task: &str) -> Option<&Model> {
        let name = &self.settings.tasks.iter().find(|t| t.id == task)?.model;
        self.model(name)
    }

    pub fn waiting(&self) -> impl Iterator<Item = &Approval> {
        self.approvals
            .iter()
            .filter(|a| a.status == ApprovalStatus::Waiting)
    }

    /// The approvals that carry a draft answering this thread.
    pub fn drafts_for(&self, thread: &str) -> impl Iterator<Item = &Approval> {
        self.approvals
            .iter()
            .filter(move |a| matches!(&a.action, ApprovalAction::Send(t) if t == thread))
    }

    /// The providers of these accounts.
    pub fn providers(&self, accounts: &[String]) -> Vec<MailProvider> {
        accounts
            .iter()
            .filter_map(|id| self.account(id))
            .map(|a| a.provider)
            .collect()
    }

    /// The accounts and permissions an owner has.
    pub fn grants(&self, owner: &Owner) -> Option<(&[String], &[Permission])> {
        match owner {
            Owner::Agent(id) => self
                .agent(id)
                .map(|a| (a.accounts.as_slice(), a.permissions.as_slice())),
            Owner::Defaults => Some((&[], &self.settings.defaults)),
            Owner::Builder => Some((&self.builder.accounts, &self.builder.permissions)),
        }
    }

    /// The text of a message or a quarantined mail.
    pub fn body(&self, id: &str) -> Option<&str> {
        self.message(id).map(|(_, m)| m.body.as_str()).or_else(|| {
            self.quarantine
                .iter()
                .find(|q| q.id == id)
                .map(|q| q.body.as_str())
        })
    }

    fn shown(&self, calendar: &str) -> bool {
        self.calendars.iter().any(|s| s.id == calendar && s.shown)
    }

    /// The threads a listing holds, in order; `None` for an unknown account or folder.
    pub fn listing(&self, listing: &Listing) -> Option<Vec<&Thread>> {
        let now = self.now;
        let holds: Box<dyn Fn(&Thread) -> bool + '_> = match listing {
            Listing::Mailbox(mailbox, category) => {
                let account = match mailbox {
                    Mailbox::All => None,
                    Mailbox::Account(id) => Some(self.account(id)?.id.as_str()),
                };
                let category = *category;
                Box::new(move |t| {
                    t.in_inbox(now)
                        && t.category == category
                        && account.is_none_or(|a| a == t.account)
                })
            }
            Listing::Folder(id) => {
                let folder = self.folders.iter().find(|f| f.id == *id)?;
                let id = folder.id.as_str();
                match folder.kind {
                    FolderKind::Starred => Box::new(|t| t.starred() && t.place != Place::Trash),
                    FolderKind::Sent => Box::new(|t| {
                        t.place != Place::Trash && t.messages.iter().any(|m| m.from_you)
                    }),
                    FolderKind::Archive => Box::new(|t| t.place == Place::Archive),
                    FolderKind::Trash => Box::new(|t| t.place == Place::Trash),
                    FolderKind::User => Box::new(move |t| {
                        t.place != Place::Trash && t.folders.iter().any(|f| f == id)
                    }),
                    // Quarantine is its own screen, not a listing of threads.
                    FolderKind::Quarantine => return None,
                }
            }
            Listing::Search { query, scope } => {
                World::listing(self, scope)?;
                return Some(match &self.search {
                    Some((searched, ids)) if searched == query => {
                        ids.iter().filter_map(|id| self.thread(id)).collect()
                    }
                    _ => Vec::new(),
                });
            }
        };
        Some(self.threads.iter().filter(|t| holds(t)).collect())
    }

    /// Unread threads in the inbox, of one account or all, of one category or all.
    pub fn unread(&self, account: Option<&str>, category: Option<InboxCategory>) -> usize {
        self.threads
            .iter()
            .filter(|t| {
                t.unread
                    && t.in_inbox(self.now)
                    && account.is_none_or(|a| a == t.account)
                    && category.is_none_or(|c| c == t.category)
            })
            .count()
    }
}

impl Catalog for World {
    fn signed_in(&self) -> bool {
        self.user.is_some()
    }

    fn listing(&self, listing: &Listing) -> Option<Vec<&str>> {
        World::listing(self, listing).map(|threads| threads.iter().map(|t| t.id.as_str()).collect())
    }

    fn messages(&self, thread: &str) -> Option<Vec<&str>> {
        Some(
            self.thread(thread)?
                .messages
                .iter()
                .map(|m| m.id.as_str())
                .collect(),
        )
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
            .map(|(_, m)| m.id.as_str())
            .collect()
    }

    fn panes(&self, agent: &str) -> Option<AgentPanes> {
        self.agent(agent).map(Agent::panes)
    }

    fn day_of(&self, event: &str) -> Option<Date> {
        self.event(event).map(|e| e.date)
    }

    fn events(&self) -> Vec<&str> {
        self.events
            .iter()
            .filter(|e| self.shown(&e.calendar))
            .map(|e| e.id.as_str())
            .collect()
    }

    fn has_calendar(&self) -> bool {
        !self.calendars.is_empty()
    }

    fn discloses(&self, disclosure: &Disclosure) -> bool {
        match disclosure {
            Disclosure::NewFolder | Disclosure::NewRule | Disclosure::Budget => true,
            Disclosure::Key(provider) => self.key_set(*provider),
            Disclosure::Invitation(event) => {
                self.event(event).is_some_and(|e| e.invitation.is_some())
            }
            Disclosure::Run(run) => self.run(run).is_some(),
        }
    }

    fn quarantined(&self) -> Vec<&str> {
        self.quarantine.iter().map(|q| q.id.as_str()).collect()
    }

    fn editions(&self) -> Vec<&str> {
        self.editions.iter().map(|e| e.id.as_str()).collect()
    }

    /// Drafts the composer can open: not handed to the Backend.
    fn drafts(&self) -> Vec<&str> {
        self.drafts
            .iter()
            .filter(|d| d.send.is_none())
            .map(|d| d.id.as_str())
            .collect()
    }

    fn today(&self) -> Date {
        World::today(self)
    }
}
