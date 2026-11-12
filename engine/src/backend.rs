//! The port to the Backend. The engine applies the user's local changes to the replica at once and
//! reports them; work only the Backend does and every change it decides (sending and its timing,
//! snoozes, permissions and limits, agent runs and what wakes them, model output, provider sign-in
//! and sync, export, account deletion) is requested, and its results come back as `Update`s that
//! the engine applies to the replica, as sync does.

use crate::{
    action::Action,
    date::Date,
    live::Target,
    proto::ui::v1::{InboxCategory, MailProvider, ModelProvider, NoteKind, PermissionLevel},
    world::{
        Ai, Approval, ApprovalStatus, Cause, Edition, Identity, Levels, Outcome, Owner, Place,
        ResultItem, SendState, Snooze, Step, TurnState, World,
    },
};

pub trait Backend: crate::platform::Threaded {
    /// The Backend's clock, Unix seconds: Postgres `now()` is the only source of now.
    fn now(&self) -> i64;
    /// The engine opened the replica: the Backend picks up its own work from here.
    fn start(&self, replica: Link);
    /// A change the engine already applied to the replica.
    fn report(&self, action: &Action, replica: Link);
    /// Work only the Backend does, or a change it decides. Its results reach the replica in order.
    fn request(&self, work: Work, replica: Link);
}

/// How the Backend reaches the replica: its updates go in as sync's do, and it reads the data as
/// it stands, as the Backend reads its own database. Neither may be called while the engine is
/// handling an input.
pub trait Replica: crate::platform::Threaded {
    fn deliver(&self, update: Update);
    /// `None` once the engine is gone or the replica cannot be read.
    fn world(&self) -> Option<World>;
}

pub type Link = std::sync::Arc<dyn Replica>;

/// Work in flight the user waits on, which the Backend answers or fails, and the user may stop.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    SignIn,
    Summary(String),
    Reply(String),
    DryRun,
    Export,
}

/// When a send goes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SendAt {
    /// After the undo window.
    Now,
    /// At this wall time.
    Later(Date, u32),
}

pub enum Work {
    SignIn(Identity),
    /// The first sync after sign-in, or again after the local data was dropped.
    Sync,
    /// Connects an account: an OAuth provider after its own sign-in page, or IMAP with the
    /// address and app password, which go to the Backend only.
    Connect {
        provider: MailProvider,
        address: String,
        password: String,
    },
    Reconnect(String),
    Send {
        draft: String,
        at: SendAt,
    },
    /// Takes back a send the Backend still holds.
    CancelSend(String),
    Snooze {
        thread: String,
        until: Snooze,
    },
    /// Scribe's summary of a thread.
    Summarize(String),
    WriteReply {
        thread: String,
        instruction: String,
    },
    /// Run now.
    Run(String),
    /// Answers a chat turn.
    Chat {
        agent: String,
        turn: String,
    },
    /// The user approved (sent or unsubscribed) or declined an item; the agent resumes.
    Resolve {
        approval: String,
        approved: bool,
    },
    /// A permission's level.
    Permission {
        owner: Owner,
        action: String,
        level: PermissionLevel,
    },
    /// The accounts an agent or the builder sees.
    Accounts {
        owner: Owner,
        accounts: Vec<String>,
    },
    AddTopic(String),
    /// The builder's custom interval, in seconds.
    Interval(u32),
    SetKey(ModelProvider, String),
    RemoveKey(ModelProvider),
    /// A daily token budget, or none.
    Budget(Option<u64>),
    /// Pause or resume every agent.
    Pause(bool),
    DryRun,
    Export,
    DeleteUser,
    /// The user stopped waiting for this.
    Cancel(Request),
}

/// A thread new to the replica.
#[derive(Debug, PartialEq)]
pub struct NewThread {
    pub id: String,
    pub account: String,
    pub place: Place,
    pub category: InboxCategory,
    pub from: String,
    pub subject: String,
    pub others: Vec<String>,
    pub note: Option<(String, NoteKind)>,
    /// A folder the user made that holds it, by id.
    pub folder: Option<String>,
}

/// A message new to the replica.
#[derive(Debug, PartialEq)]
pub struct NewMessage {
    pub id: String,
    pub from_you: bool,
    pub sender: String,
    pub sent_at: i64,
    pub body: String,
    /// (name, size in bytes)
    pub attachments: Vec<(String, i64)>,
}

/// Mail kept out of the inbox.
#[derive(Debug, PartialEq)]
pub struct NewQuarantined {
    pub id: String,
    pub account: String,
    pub sender: String,
    pub received_at: i64,
    pub subject: String,
    pub body: String,
    /// "Scam", "Blocked sender"
    pub reason: String,
    /// "Security Guard", "Your block list"
    pub caught_by: String,
    pub why: String,
    pub outcome: String,
    /// The thread it was, now gone from the mail.
    pub thread: Option<String>,
    /// When the Backend drops its copy.
    pub drops_at: i64,
    /// The sender's address to block, and who blocks it: "Security Guard".
    pub block: Option<(String, String)>,
}

#[derive(Debug, PartialEq)]
pub struct NewAccount {
    pub id: String,
    pub name: String,
    pub address: String,
    pub provider: MailProvider,
    pub provider_name: String,
}

/// One line of a dry run: what the agent would do with a mail.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DryRunLine {
    pub from: String,
    pub action: String,
    pub decision: Decision,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Ask,
    Auto,
    Skip,
}

#[derive(Debug, PartialEq)]
pub enum Update {
    SignedIn {
        user_name: String,
        provider: Identity,
    },
    /// The first sync: every row but the session and the device's preferences, as SQL inserts in
    /// the replica's own shape, the stand-in for the snapshot frames until sync is built.
    Snapshot(String),
    /// The provider refused the address or password.
    ConnectRejected(String),
    /// An account was connected; its first sync starts.
    AccountAdded(NewAccount),
    SyncProgress {
        account: String,
        percent: u32,
    },
    AccountSynced(String),
    /// The account was checked for new mail.
    Fetched(String),
    /// Mail arrived: a message in a thread, new when `thread` is given.
    Mail {
        thread: Option<NewThread>,
        thread_id: String,
        message: NewMessage,
    },
    /// A draft handed over, as the Backend holds it.
    Outbox {
        draft: String,
        state: SendState,
    },
    /// A send was taken back: the draft is a draft again.
    SendCancelled(String),
    /// A draft was sent as this message.
    Sent {
        draft: String,
        message: String,
        at: i64,
    },
    Snoozed {
        thread: String,
        until: i64,
    },
    /// A snooze came due: the thread is back on top, unread.
    SnoozeEnded(String),
    /// When each snooze ends, as of now.
    SnoozeTimes(Vec<(Snooze, i64)>),
    Summary {
        thread: String,
        text: String,
    },
    Reply {
        thread: String,
        text: String,
    },
    /// The Backend gave up on a request.
    Failed {
        request: Request,
        reason: String,
    },
    RunStarted {
        agent: String,
        run: String,
        cause: Cause,
    },
    /// The Backend would not do what the user asked.
    Refused {
        target: Target,
        reason: String,
    },
    /// An agent's schedule woke it at this time, whether it ran or not.
    Woke {
        agent: String,
        at: i64,
    },
    RunStep {
        run: String,
        step: Step,
    },
    RunFinished {
        run: String,
        outcome: Outcome,
        summary: String,
        /// How long it ran.
        seconds: i64,
        tokens: i64,
    },
    /// Agents were paused: the run finished its step and waits.
    RunPaused(String),
    RunResumed(String),
    Result {
        agent: String,
        item: ResultItem,
    },
    Approval(Approval),
    /// An approval was carried out or declined, by the user or because it ran out.
    Resolved {
        approval: String,
        status: ApprovalStatus,
    },
    /// An agent moved a thread into a folder the user made.
    Filed {
        thread: String,
        folder: String,
    },
    Quarantined(NewQuarantined),
    /// The Backend dropped its quarantined copy.
    QuarantineDropped(String),
    Turn {
        turn: String,
        state: TurnState,
    },
    Edition(Edition),
    DryRun(Vec<DryRunLine>),
    ExportProgress(u32),
    ExportReady {
        text: String,
        file: String,
    },
    Permission {
        owner: Owner,
        action: String,
        level: PermissionLevel,
        levels: Levels,
    },
    Accounts {
        owner: Owner,
        accounts: Vec<String>,
    },
    TopicAdded(String),
    Interval(u32),
    Key {
        provider: ModelProvider,
        set: bool,
        ai: Ai,
    },
    Budget(Option<i64>),
    AgentsPaused(bool),
    UserDeleted,
}
