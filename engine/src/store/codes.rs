//! The one definition of each stored code. Every enum the replica keeps as a number reads and
//! writes through these `ToSql`/`FromSql` impls, so loader and writer share them and SQL text
//! carries no state numbers. An unknown code is an error, never a default.

use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};

use crate::{
    proto::ui::v1::{
        AttachmentKind, CalendarColor, CalendarProvider, Density, InboxCategory, MailProvider,
        ModelProvider, NoteKind, PermissionLevel, Sidebar, Theme,
    },
    world::{
        Ai, Answer, DraftKind, FolderKind, Identity, Place, ResultsKind, Snooze, StepPermission,
    },
};

/// Lists each variant with its code once; reading any other code fails.
macro_rules! codes {
    ($($ty:ident { $($variant:ident = $code:literal),+ $(,)? })+) => {$(
        impl ToSql for $ty {
            fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
                let code: i64 = match self { $($ty::$variant => $code,)+ };
                Ok(ToSqlOutput::from(code))
            }
        }

        impl FromSql for $ty {
            fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
                match value.as_i64()? {
                    $($code => Ok($ty::$variant),)+
                    other => Err(FromSqlError::OutOfRange(other)),
                }
            }
        }
    )+};
}

/// ui.v1 enums are stored as their proto numbers; unspecified and unknown numbers fail.
macro_rules! ui_codes {
    ($($ty:ident),+) => {$(
        impl ToSql for $ty {
            fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
                Ok(ToSqlOutput::from(i64::from(*self as i32)))
            }
        }

        impl FromSql for $ty {
            fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
                let code = value.as_i64()?;
                i32::try_from(code)
                    .ok()
                    .and_then(|c| $ty::try_from(c).ok())
                    .filter(|e| *e as i32 != 0)
                    .ok_or(FromSqlError::OutOfRange(code))
            }
        }
    )+};
}

ui_codes!(
    AttachmentKind,
    CalendarColor,
    CalendarProvider,
    Density,
    InboxCategory,
    MailProvider,
    ModelProvider,
    NoteKind,
    PermissionLevel,
    Sidebar,
    Theme
);

/// accounts.state; `world::AccountState` with its data in other columns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccountKind {
    Synced,
    SignInExpired,
    Syncing,
}

/// outbox.state; `world::SendState` with its failure in another column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SendKind {
    Undoable,
    Scheduled,
    Sending,
    Failed,
}

/// agents.schedule; `schedule::Schedule` with its data in other columns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScheduleKind {
    Triggers,
    BuiltIn,
    Times,
    Interval,
    OnMessage,
}

/// runs.cause; `world::Cause`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CauseKind {
    Mail,
    Opened,
    Schedule,
    Message,
    Handoff,
    Connected,
    RunNow,
    Sorted,
    Flagged,
    Starred,
    Finished,
    Resolved,
    Approaching,
}

/// runs.outcome; `world::Outcome`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutcomeKind {
    Done,
    Waiting,
    Running,
    Failed,
    Stopped,
    Paused,
}

/// approvals.status; `world::ApprovalStatus` with its reason in another column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusKind {
    Waiting,
    Resolving,
    Done,
    Declined,
}

/// The kind column of the refs tables; `world::Ref`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefKind {
    Thread,
    Event,
    Approval,
    Quarantined,
}

/// turns.state; `world::TurnState`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TurnKind {
    Running,
    Waiting,
    Answered,
    Failed,
    Stopped,
}

/// approvals.action; `world::ApprovalAction`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApprovalKind {
    Send,
    Unsubscribe,
}

/// edition_lines.section: which list of `world::Edition` a line is in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    Activity,
    Newsletters,
    Updates,
}

/// rules.condition; `world::Condition`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConditionKind {
    SenderContains,
    SenderDomain,
    SubjectContains,
    Newsletter,
    NewSenderWithLink,
}

/// rule_actions.action; `world::RuleAction`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionKind {
    MoveTo,
    Archive,
    Trash,
    MarkImportant,
    HandTo,
    KeepForBriefing,
}

codes! {
    FolderKind { Starred = 1, Sent = 2, Archive = 3, Quarantine = 4, User = 5, Trash = 6 }
    Place { Inbox = 1, Archive = 2, Trash = 3, Sent = 4 }
    DraftKind { New = 1, Reply = 2, ReplyAll = 3, Forward = 4 }
    Answer { Unanswered = 1, Accepted = 2, Tentative = 3, Declined = 4 }
    StatusKind { Waiting = 1, Done = 2, Declined = 3, Resolving = 4 }
    Snooze { LaterToday = 1, Tomorrow = 2, ThisWeekend = 3, NextWeek = 4 }
    Identity { Google = 1, Microsoft = 2 }
    Ai { On = 1, NoKey = 2 }
    ResultsKind { Groups = 1, Briefing = 2, Conversation = 3 }
    AccountKind { Synced = 1, SignInExpired = 2, Syncing = 3 }
    SendKind { Undoable = 1, Scheduled = 2, Sending = 3, Failed = 4 }
    ScheduleKind { Triggers = 1, BuiltIn = 2, Times = 3, Interval = 4, OnMessage = 5 }
    CauseKind {
        Mail = 1, Opened = 2, Schedule = 3, Message = 4, Handoff = 5, Connected = 6, RunNow = 7,
        Sorted = 8, Flagged = 9, Starred = 10, Finished = 11, Resolved = 12, Approaching = 13,
    }
    OutcomeKind { Done = 1, Waiting = 2, Running = 3, Failed = 4, Stopped = 5, Paused = 6 }
    RefKind { Thread = 1, Event = 2, Approval = 3, Quarantined = 4 }
    TurnKind { Running = 1, Waiting = 2, Answered = 3, Failed = 4, Stopped = 5 }
    ApprovalKind { Send = 1, Unsubscribe = 2 }
    StepPermission { Read = 1, Auto = 2, Asked = 3, Off = 4 }
    Section { Activity = 1, Newsletters = 2, Updates = 3 }
    ConditionKind {
        SenderContains = 1, SenderDomain = 2, SubjectContains = 3, Newsletter = 4,
        NewSenderWithLink = 5,
    }
    ActionKind {
        MoveTo = 1, Archive = 2, Trash = 3, MarkImportant = 4, HandTo = 5, KeepForBriefing = 6,
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::*;

    fn read<T: FromSql>(code: i64) -> rusqlite::Result<T> {
        Connection::open_in_memory()?.query_row("SELECT ?1", [code], |row| row.get(0))
    }

    #[test]
    fn codes_round_trip_and_unknown_codes_fail() {
        let c = Connection::open_in_memory().unwrap();
        let back: Place = c
            .query_row("SELECT ?1", [Place::Trash], |row| row.get(0))
            .unwrap();
        assert_eq!(back, Place::Trash);
        assert_eq!(read::<Place>(3).unwrap(), Place::Trash);
        assert!(matches!(
            read::<Place>(9),
            Err(rusqlite::Error::IntegralValueOutOfRange(_, 9))
        ));
        assert!(read::<CauseKind>(0).is_err());
        assert_eq!(read::<CauseKind>(13).unwrap(), CauseKind::Approaching);
    }

    #[test]
    fn ui_enums_reject_unspecified_and_unknown_numbers() {
        assert_eq!(read::<MailProvider>(1).unwrap(), MailProvider::Gmail);
        assert!(read::<MailProvider>(0).is_err(), "unspecified");
        assert!(read::<MailProvider>(99).is_err(), "unknown");
        assert!(read::<PermissionLevel>(i64::from(i32::MAX) + 1).is_err());
    }
}
