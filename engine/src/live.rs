//! The short-lived state the engine keeps in memory only: outcomes of the user's actions, the undo
//! notice, and Backend work in flight that the views show. None of it survives a relaunch, and
//! each piece is one explicit field.

use std::collections::BTreeMap;

use crate::{
    backend::{DryRunLine, Request, Update},
    proto::ui::v1::{Form, ModelProvider},
    world::{Identity, Place},
};

/// How long an outcome lasts, and a notice with nothing to undo.
pub const OUTCOME_MS: i64 = 2_000;
pub const INFO_MS: i64 = 5_000;
/// Archive, delete, move and snooze can be undone this long.
pub const UNDO_MS: i64 = 6_000;

/// What an outcome is about.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    Thread(String),
    Approval(String),
    Event(String),
    Agent(String),
    Draft(String),
    Account(String),
    Key(ModelProvider),
    Form(Form),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Resolution {
    Done(String),
    Failed(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub target: Target,
    pub result: Resolution,
    /// Device milliseconds.
    pub until: i64,
}

/// What Undo takes back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Undo {
    /// Puts a thread back where it was.
    Thread {
        thread: String,
        place: Place,
        snoozed_until: Option<i64>,
        folders: Vec<String>,
    },
    /// Takes a send back while the Backend holds it.
    Send(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Notice {
    None,
    Undo {
        text: String,
        undo: Undo,
        until: i64,
    },
    Info {
        text: String,
        until: i64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SignIn {
    Idle,
    Waiting(Identity),
    Failed(String),
}

/// A summary Scribe was asked for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Summary {
    Reading,
    Failed(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Generation {
    Writing,
    Written(String),
    Failed(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImapCheck {
    Idle,
    Checking,
    Rejected(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DryRun {
    Idle,
    Reading,
    Results(Vec<DryRunLine>),
    Failed(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Export {
    Idle,
    Running(u32),
    Ready { text: String, file: String },
    Failed(String),
}

/// A typed field or form whose text the engine clears or replaces.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Field {
    NewFolder,
    NewRule,
    NewsTopic,
    Budget,
    Key(ModelProvider),
    /// An agent's chat box.
    Chat(String),
    /// A thread's reply instruction.
    Reply(String),
    /// The builder's name and instructions.
    Builder,
}

/// When each field's text was last cleared or replaced, from one counter that only grows, so a
/// field's revision never comes back to a value the Frontend saw.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Revisions {
    counter: u32,
    /// Every field was cleared at this count, as when the data was dropped.
    all: u32,
    fields: BTreeMap<Field, u32>,
}

impl Revisions {
    pub fn revise(&mut self, field: Field) {
        self.counter += 1;
        self.fields.insert(field, self.counter);
    }

    pub fn revise_all(&mut self) {
        self.counter += 1;
        self.all = self.counter;
        self.fields.clear();
    }

    pub fn of(&self, field: &Field) -> u32 {
        self.fields.get(field).copied().unwrap_or(self.all)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Live {
    pub outcomes: Vec<Outcome>,
    pub notice: Notice,
    pub sign_in: SignIn,
    pub imap: ImapCheck,
    /// Summaries asked for and not written, by thread.
    pub summaries: BTreeMap<String, Summary>,
    /// Replies with AI, by thread.
    pub replies: BTreeMap<String, Generation>,
    pub dry_run: DryRun,
    pub export: Export,
    pub revisions: Revisions,
}

pub const START: Live = Live {
    outcomes: Vec::new(),
    notice: Notice::None,
    sign_in: SignIn::Idle,
    imap: ImapCheck::Idle,
    summaries: BTreeMap::new(),
    replies: BTreeMap::new(),
    dry_run: DryRun::Idle,
    export: Export::Idle,
    revisions: Revisions {
        counter: 0,
        all: 0,
        fields: BTreeMap::new(),
    },
};

impl Live {
    /// Drops everything with the data, and every typed field's text with it.
    pub fn clear(&mut self) {
        let mut revisions = std::mem::replace(self, START).revisions;
        revisions.revise_all();
        self.revisions = revisions;
    }

    pub fn done(&mut self, target: Target, text: &str, now_ms: i64) {
        self.outcome(target, Resolution::Done(text.into()), now_ms);
    }

    pub fn failed(&mut self, target: Target, reason: &str, now_ms: i64) {
        self.outcome(target, Resolution::Failed(reason.into()), now_ms);
    }

    fn outcome(&mut self, target: Target, result: Resolution, now_ms: i64) {
        self.outcomes.retain(|o| o.target != target);
        self.outcomes.push(Outcome {
            target,
            result,
            until: now_ms + OUTCOME_MS,
        });
    }

    pub fn undo(&mut self, text: String, undo: Undo, until: i64) {
        self.notice = Notice::Undo { text, undo, until };
    }

    pub fn info(&mut self, text: String, now_ms: i64) {
        self.notice = Notice::Info {
            text,
            until: now_ms + INFO_MS,
        };
    }

    /// An outcome on this target lasts.
    pub fn lasting(&self, target: &Target) -> Option<&Resolution> {
        self.outcomes
            .iter()
            .find(|o| &o.target == target)
            .map(|o| &o.result)
    }

    /// Drops what has run out by `now_ms`.
    pub fn expire(&mut self, now_ms: i64) {
        self.outcomes.retain(|o| o.until > now_ms);
        let ended = match &self.notice {
            Notice::Undo { until, .. } | Notice::Info { until, .. } => *until <= now_ms,
            Notice::None => false,
        };
        if ended {
            self.notice = Notice::None;
        }
    }

    /// When the next outcome or notice runs out.
    pub fn next_expiry(&self) -> Option<i64> {
        let notice = match &self.notice {
            Notice::Undo { until, .. } | Notice::Info { until, .. } => Some(*until),
            Notice::None => None,
        };
        self.outcomes.iter().map(|o| o.until).chain(notice).min()
    }

    /// The user stopped waiting for a request.
    pub fn cancel(&mut self, request: &Request) {
        match request {
            Request::SignIn => self.sign_in = SignIn::Idle,
            Request::Summary(thread) => {
                self.summaries.remove(thread);
            }
            Request::Reply(thread) => {
                self.replies.remove(thread);
            }
            Request::DryRun => self.dry_run = DryRun::Idle,
            Request::Export => self.export = Export::Idle,
        }
    }

    /// Whether a request is still waited on, so its answer or failure lands.
    fn waiting(&self, request: &Request) -> bool {
        match request {
            Request::SignIn => matches!(self.sign_in, SignIn::Waiting(_)),
            Request::Summary(thread) => self.summaries.get(thread) == Some(&Summary::Reading),
            Request::Reply(thread) => self.replies.get(thread) == Some(&Generation::Writing),
            Request::DryRun => self.dry_run == DryRun::Reading,
            Request::Export => matches!(self.export, Export::Running(_)),
        }
    }

    /// The part of an update that lands in memory rather than in the replica.
    pub fn apply(&mut self, update: &Update) {
        match update {
            Update::SignedIn { .. } => self.sign_in = SignIn::Idle,
            Update::ConnectRejected(reason) => self.imap = ImapCheck::Rejected(reason.clone()),
            Update::AccountAdded(_) => self.imap = ImapCheck::Idle,
            Update::Summary { thread, .. } => {
                self.summaries.remove(thread);
            }
            Update::Reply { thread, text } if self.waiting(&Request::Reply(thread.clone())) => {
                self.replies
                    .insert(thread.clone(), Generation::Written(text.clone()));
            }
            Update::DryRun(lines) if self.waiting(&Request::DryRun) => {
                self.dry_run = DryRun::Results(lines.clone());
            }
            Update::ExportProgress(percent) if self.waiting(&Request::Export) => {
                self.export = Export::Running(*percent);
            }
            Update::ExportReady { text, file } if self.waiting(&Request::Export) => {
                self.export = Export::Ready {
                    text: text.clone(),
                    file: file.clone(),
                }
            }
            Update::Failed { request, reason } if self.waiting(request) => {
                let reason = reason.clone();
                match request {
                    Request::SignIn => self.sign_in = SignIn::Failed(reason),
                    Request::Summary(thread) => {
                        self.summaries
                            .insert(thread.clone(), Summary::Failed(reason));
                    }
                    Request::Reply(thread) => {
                        self.replies
                            .insert(thread.clone(), Generation::Failed(reason));
                    }
                    Request::DryRun => self.dry_run = DryRun::Failed(reason),
                    Request::Export => self.export = Export::Failed(reason),
                }
            }
            Update::UserDeleted => self.clear(),
            Update::Reply { .. }
            | Update::DryRun(_)
            | Update::ExportProgress(_)
            | Update::ExportReady { .. }
            | Update::Failed { .. }
            | Update::Snapshot(_)
            | Update::SyncProgress { .. }
            | Update::AccountSynced(_)
            | Update::Mail { .. }
            | Update::Outbox { .. }
            | Update::SendCancelled(_)
            | Update::Sent { .. }
            | Update::Snoozed { .. }
            | Update::SnoozeEnded(_)
            | Update::SnoozeTimes(_)
            | Update::RunStarted { .. }
            | Update::RunStep { .. }
            | Update::RunFinished { .. }
            | Update::RunPaused(_)
            | Update::RunResumed(_)
            | Update::Result { .. }
            | Update::Approval(_)
            | Update::Resolved { .. }
            | Update::Filed { .. }
            | Update::Quarantined(_)
            | Update::QuarantineDropped(_)
            | Update::Fetched(_)
            | Update::Refused { .. }
            | Update::Woke { .. }
            | Update::Turn { .. }
            | Update::Edition(_)
            | Update::Permission { .. }
            | Update::Accounts { .. }
            | Update::TopicAdded(_)
            | Update::Interval(_)
            | Update::Key { .. }
            | Update::Budget(_)
            | Update::AgentsPaused(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcomes_and_notices_run_out() {
        let mut live = START;
        live.done(Target::Thread("t".into()), "Archived", 1_000);
        live.failed(Target::Form(Form::NewFolder), "Name it", 1_500);
        live.info("Sent".into(), 1_000);
        assert_eq!(live.next_expiry(), Some(1_000 + OUTCOME_MS));
        live.expire(1_000 + OUTCOME_MS);
        assert_eq!(live.outcomes.len(), 1, "the later outcome lasts");
        live.expire(1_000 + INFO_MS);
        assert_eq!((live.outcomes.len(), &live.notice), (0, &Notice::None));
        assert_eq!(live.next_expiry(), None);
    }

    #[test]
    fn a_late_answer_or_failure_lands_only_while_it_is_waited_on() {
        let mut live = START;
        live.dry_run = DryRun::Reading;
        live.cancel(&Request::DryRun);
        live.apply(&Update::DryRun(Vec::new()));
        assert_eq!(live.dry_run, DryRun::Idle, "stopped: the answer is dropped");
        live.replies.insert("t".into(), Generation::Writing);
        let failed = Update::Failed {
            request: Request::Reply("t".into()),
            reason: "No answer".into(),
        };
        live.apply(&failed);
        assert_eq!(
            live.replies.get("t"),
            Some(&Generation::Failed("No answer".into()))
        );
        live.apply(&Update::Reply {
            thread: "t".into(),
            text: "Late".into(),
        });
        assert_eq!(
            live.replies.get("t"),
            Some(&Generation::Failed("No answer".into())),
            "an answer after the failure is dropped"
        );
    }

    #[test]
    fn a_revision_changes_only_for_its_field_and_never_repeats() {
        let mut live = START;
        let chat = Field::Chat("a".into());
        assert_eq!(live.revisions.of(&chat), 0);
        live.revisions.revise(Field::NewFolder);
        assert_eq!(live.revisions.of(&chat), 0, "another field");
        live.revisions.revise(chat.clone());
        let first = live.revisions.of(&chat);
        live.clear();
        let cleared = live.revisions.of(&chat);
        assert!(cleared > first, "dropping the data clears every field");
        assert_eq!(live.revisions.of(&Field::Builder), cleared);
        live.revisions.revise(Field::Builder);
        assert!(live.revisions.of(&Field::Builder) > cleared);
        assert_eq!(live.revisions.of(&chat), cleared);
    }

    #[test]
    fn a_new_outcome_replaces_the_one_on_the_same_target() {
        let mut live = START;
        let target = Target::Agent("a".into());
        live.failed(target.clone(), "Paused", 0);
        live.done(target.clone(), "Started", 10);
        assert_eq!(
            live.lasting(&target),
            Some(&Resolution::Done("Started".into()))
        );
        assert_eq!(live.outcomes.len(), 1);
    }
}
