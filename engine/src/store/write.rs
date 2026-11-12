//! Writes to the replica. Each public function is one statement or one transaction.

use rusqlite::{Connection, OptionalExtension, Params, Transaction, params};

use super::{
    Result, Store,
    codes::{
        AccountKind, ActionKind, ApprovalKind, CauseKind, ConditionKind, OutcomeKind, RefKind,
        ScheduleKind, Section, SendKind, StatusKind, TurnKind,
    },
};
use crate::{
    backend::{NewMessage, NewQuarantined, NewThread, Update},
    builder,
    preferences::Preferences,
    proto::ui::v1::{InboxCategory, ModelProvider, PermissionLevel},
    schedule::Schedule,
    world::{
        Answer, Approval, ApprovalAction, ApprovalStatus, Cause, Condition, DraftKind, Edition,
        FolderKind, Identity, Levels, Outcome, Owner, Place, Ref, ResultItem, RuleAction,
        SendState, TurnState, attachment_kind, recipients,
    },
};

/// Rebuilds the search index from every message.
pub(super) fn index_all(c: &Connection) -> Result<()> {
    c.execute("DELETE FROM mail_search", [])?;
    let ids: Vec<String> = c
        .prepare("SELECT id FROM messages")?
        .query_map([], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    for id in ids {
        index_message(c, &id)?;
    }
    Ok(())
}

/// Indexes one message: its thread's subject, its sender, its recipients line and its text.
fn index_message(c: &Connection, message: &str) -> Result<()> {
    let (thread, subject, other_party, sender, from_you, body): (
        String,
        String,
        String,
        String,
        bool,
        String,
    ) = c.query_row(
        "SELECT t.id, t.subject, t.sender, m.sender, m.from_you, m.body FROM messages m JOIN threads t ON t.id = m.thread_id WHERE m.id = ?1",
        [message],
        |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        },
    )?;
    let others: Vec<String> = c
        .prepare("SELECT name FROM thread_people WHERE thread_id = ?1 ORDER BY position")?
        .query_map([&thread], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    c.execute("DELETE FROM mail_search WHERE message_id = ?1", [message])?;
    c.execute(
        "INSERT INTO mail_search (message_id, thread_id, subject, sender, recipients, body) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            message,
            thread,
            subject,
            sender,
            recipients(&other_party, &others, from_you),
            body
        ],
    )?;
    Ok(())
}

/// Removes the search rows of messages about to be deleted with their thread or account.
fn unindex(c: &Connection, threads_where: &str, value: &str) -> Result<()> {
    c.execute(
        &format!(
            "DELETE FROM mail_search WHERE thread_id IN (SELECT id FROM threads WHERE {threads_where})"
        ),
        [value],
    )?;
    Ok(())
}

fn reference(r: &Ref) -> (RefKind, &str) {
    match r {
        Ref::Thread(t) => (RefKind::Thread, t),
        Ref::Event(t) => (RefKind::Event, t),
        Ref::Approval(t) => (RefKind::Approval, t),
        Ref::Quarantined(t) => (RefKind::Quarantined, t),
    }
}

/// What references belong to, each in its own table so they go with it.
enum Holder<'a> {
    Item(&'a str),
    Cause(&'a str),
    Step(&'a str, i64),
    Turn(&'a str),
}

fn add_refs(c: &Connection, owner: Holder, refs: &[Ref]) -> Result<()> {
    for (i, r) in refs.iter().enumerate() {
        let (kind, target) = reference(r);
        let position = i as i64;
        match owner {
            Holder::Item(id) => c.execute(
                "INSERT INTO item_refs VALUES (?1, ?2, ?3, ?4)",
                params![id, position, kind, target],
            ),
            Holder::Cause(id) => c.execute(
                "INSERT INTO cause_refs VALUES (?1, ?2, ?3, ?4)",
                params![id, position, kind, target],
            ),
            Holder::Step(run, step) => c.execute(
                "INSERT INTO step_refs VALUES (?1, ?2, ?3, ?4, ?5)",
                params![run, step, position, kind, target],
            ),
            Holder::Turn(id) => c.execute(
                "INSERT INTO turn_refs VALUES (?1, ?2, ?3, ?4)",
                params![id, position, kind, target],
            ),
        }?;
    }
    Ok(())
}

/// A turn's state as its columns: kind, text, approval.
fn turn_row(state: &TurnState) -> (TurnKind, Option<&str>, Option<&str>) {
    match state {
        TurnState::Running(label) => (TurnKind::Running, Some(label), None),
        TurnState::Waiting(approval) => (TurnKind::Waiting, None, Some(approval)),
        TurnState::Answered(text, _) => (TurnKind::Answered, Some(text), None),
        TurnState::Failed(reason) => (TurnKind::Failed, Some(reason), None),
        TurnState::Stopped(text) => (TurnKind::Stopped, Some(text), None),
    }
}

fn outcome_row(outcome: &Outcome) -> (OutcomeKind, Option<&str>) {
    match outcome {
        Outcome::Done => (OutcomeKind::Done, None),
        Outcome::Waiting => (OutcomeKind::Waiting, None),
        Outcome::Running => (OutcomeKind::Running, None),
        Outcome::Paused => (OutcomeKind::Paused, None),
        Outcome::Failed(text) => (OutcomeKind::Failed, Some(text)),
        Outcome::Stopped(text) => (OutcomeKind::Stopped, Some(text)),
    }
}

/// A cause as its columns: kind, text and references.
fn cause_row(cause: &Cause) -> (CauseKind, Option<&str>, Vec<Ref>) {
    let one = |r: &Ref| vec![r.clone()];
    match cause {
        Cause::Mail(r) => (CauseKind::Mail, None, one(r)),
        Cause::Opened(r) => (CauseKind::Opened, None, one(r)),
        Cause::Schedule(s) => (CauseKind::Schedule, Some(s), vec![]),
        Cause::Message(t) => (CauseKind::Message, Some(t), vec![]),
        Cause::Handoff(by, r) => (CauseKind::Handoff, Some(by), one(r)),
        Cause::Connected(a) => (CauseKind::Connected, Some(a), vec![]),
        Cause::RunNow => (CauseKind::RunNow, None, vec![]),
        Cause::Sorted(into, r) => (CauseKind::Sorted, Some(into), one(r)),
        Cause::Flagged(r) => (CauseKind::Flagged, None, one(r)),
        Cause::Starred(r) => (CauseKind::Starred, None, one(r)),
        Cause::Finished(run, refs) => (CauseKind::Finished, Some(run), refs.clone()),
        Cause::Resolved(r) => (CauseKind::Resolved, None, one(r)),
        Cause::Approaching(lead, r) => (CauseKind::Approaching, Some(lead), one(r)),
    }
}

/// A schedule as its columns: kind, note, weekdays only, interval; the times are rows of their own.
fn schedule_row(schedule: &Schedule) -> (ScheduleKind, Option<&str>, Option<bool>, Option<u32>) {
    match schedule {
        Schedule::Triggers => (ScheduleKind::Triggers, None, None, None),
        Schedule::BuiltIn(note) => (ScheduleKind::BuiltIn, Some(note), None, None),
        Schedule::Times { weekdays, .. } => (ScheduleKind::Times, None, Some(*weekdays), None),
        Schedule::Interval(seconds) => (ScheduleKind::Interval, None, None, Some(*seconds)),
        Schedule::OnMessage => (ScheduleKind::OnMessage, None, None, None),
    }
}

/// A send's state as its columns: kind, due time, failure.
fn send_row(state: &SendState) -> (SendKind, Option<i64>, Option<&str>) {
    match state {
        SendState::Undoable(until) => (SendKind::Undoable, Some(*until), None),
        SendState::Scheduled(at) => (SendKind::Scheduled, Some(*at), None),
        SendState::Sending => (SendKind::Sending, None, None),
        SendState::Failed(reason) => (SendKind::Failed, None, Some(reason)),
    }
}

/// An approval's status as its columns: kind and the reason it was declined.
fn status_row(status: &ApprovalStatus) -> (StatusKind, Option<&str>) {
    match status {
        ApprovalStatus::Waiting => (StatusKind::Waiting, None),
        ApprovalStatus::Resolving => (StatusKind::Resolving, None),
        ApprovalStatus::Done => (StatusKind::Done, None),
        ApprovalStatus::Declined(reason) => (StatusKind::Declined, Some(reason)),
    }
}

/// The reason Auto is not offered, as stored: NULL when every level is.
fn levels_row(levels: &Levels) -> Option<&str> {
    match levels {
        Levels::All => None,
        Levels::AskAtMost(reason) => Some(reason),
    }
}

/// Drops every row but the session and the device's preferences.
fn clear_data(c: &Connection) -> Result<()> {
    let tables: Vec<String> = c
        .prepare(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE 'mail_search_%' AND name NOT IN ('session', 'preferences')",
        )?
        .query_map([], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    c.execute_batch("PRAGMA defer_foreign_keys = ON")?;
    for table in tables {
        c.execute(&format!("DELETE FROM \"{table}\""), [])?;
    }
    Ok(())
}

/// Starts the builder over: nothing written, nothing chosen, every account, and each offered
/// action at its default level and levels, Off without one.
fn reset_builder(c: &Connection) -> Result<()> {
    c.execute_batch(
        "DELETE FROM builder; DELETE FROM builder_triggers; DELETE FROM builder_filters; DELETE FROM builder_accounts; DELETE FROM builder_permissions; INSERT INTO builder_accounts SELECT id FROM accounts;",
    )?;
    c.execute(
        "INSERT INTO builder VALUES (1, '', '', ?1, ?2, 1)",
        params![
            builder::NO_SCHEDULE.id,
            crate::schedule::MIN_INTERVAL_SECONDS
        ],
    )?;
    c.execute(
        "INSERT INTO builder_permissions SELECT a.id, coalesce(d.level, ?1), d.ask_at_most FROM actions a LEFT JOIN default_permissions d ON d.action_id = a.id WHERE a.offered",
        [PermissionLevel::Off],
    )?;
    Ok(())
}

fn upsert_session(c: &Connection, user_name: &str, provider: Identity) -> Result<()> {
    c.execute(
        "INSERT INTO session VALUES (1, ?1, ?2) ON CONFLICT (id) DO UPDATE SET user_name = ?1, provider = ?2",
        params![user_name, provider],
    )?;
    Ok(())
}

/// The first line of a body, short: a message's preview.
fn preview(body: &str) -> String {
    let line = body.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
    match line.char_indices().nth(90) {
        Some((at, _)) => format!("{}…", &line[..at]),
        None => line.into(),
    }
}

/// A draft to create.
pub struct NewDraft<'a> {
    pub kind: DraftKind,
    pub thread: Option<&'a str>,
    pub message: Option<&'a str>,
    pub approval: Option<&'a str>,
    pub account: &'a str,
    pub recipients: &'a str,
    pub subject: &'a str,
    pub body: &'a str,
}

/// An event's values as the drawer saves them.
pub struct EventValues<'a> {
    pub title: &'a str,
    pub day: &'a str,
    pub start: u32,
    pub end: u32,
    pub calendar: &'a str,
    pub guests: &'a [String],
    /// `None` keeps the saved place.
    pub place: Option<&'a str>,
}

/// A new agent, from the builder.
pub struct NewAgent<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub instructions: &'a str,
    pub model: &'a str,
    pub model_provider: ModelProvider,
    pub schedule: &'a Schedule,
}

impl Store {
    fn transaction(&self) -> Result<Transaction<'_>> {
        Ok(self.connection().unchecked_transaction()?)
    }

    fn run(&self, sql: &str, params: impl Params) -> Result<usize> {
        Ok(self.connection().execute(sql, params)?)
    }

    /// A free id with this prefix: "d-3".
    pub fn next_id(&self, table: &str, prefix: &str) -> Result<String> {
        let mut n: i64 = self.connection().query_row(
            &format!("SELECT count(*) FROM \"{table}\""),
            [],
            |row| row.get(0),
        )?;
        loop {
            n += 1;
            let id = format!("{prefix}-{n}");
            let taken = self
                .connection()
                .query_row(
                    &format!("SELECT 1 FROM \"{table}\" WHERE id = ?1"),
                    [&id],
                    |_| Ok(()),
                )
                .optional()?
                .is_some();
            if !taken {
                return Ok(id);
            }
        }
    }

    // Session, preferences, navigation.

    pub fn save_preferences(&self, p: Preferences) -> Result<()> {
        self.run(
            "INSERT INTO preferences VALUES (1, ?1, ?2, ?3) ON CONFLICT (id) DO UPDATE SET theme = ?1, density = ?2, sidebar = ?3",
            params![p.theme, p.density, p.sidebar],
        )?;
        Ok(())
    }

    pub fn save_navigation(&self, path: &str) -> Result<()> {
        self.run(
            "INSERT INTO navigation VALUES (1, ?1) ON CONFLICT (id) DO UPDATE SET path = ?1",
            [path],
        )?;
        Ok(())
    }

    /// Signs out: the session goes with every row but the device's preferences.
    pub fn sign_out(&self) -> Result<()> {
        let tx = self.transaction()?;
        clear_data(&tx)?;
        tx.execute("DELETE FROM session", [])?;
        tx.commit()?;
        Ok(())
    }

    /// Drops the local data to sync it again, keeping the session and the device's preferences.
    pub fn reset(&self) -> Result<()> {
        let tx = self.transaction()?;
        clear_data(&tx)?;
        tx.commit()?;
        Ok(())
    }

    // Accounts.

    /// Removes an account with its mail, quarantine and drafts.
    pub fn remove_account(&self, account: &str) -> Result<()> {
        let tx = self.transaction()?;
        unindex(&tx, "account_id = ?1", account)?;
        tx.execute("DELETE FROM accounts WHERE id = ?1", [account])?;
        tx.commit()?;
        Ok(())
    }

    pub fn set_signature(&self, account: &str, signature: Option<&str>) -> Result<()> {
        self.run(
            "UPDATE accounts SET signature = ?2 WHERE id = ?1",
            params![account, signature],
        )?;
        Ok(())
    }

    // Mail.

    pub fn set_unread(&self, thread: &str, unread: bool) -> Result<()> {
        self.run(
            "UPDATE threads SET unread = ?2 WHERE id = ?1",
            params![thread, unread],
        )?;
        Ok(())
    }

    /// Moves a thread to the inbox, the archive or the trash, ending any snooze.
    pub fn set_place(&self, thread: &str, place: Place) -> Result<()> {
        self.run(
            "UPDATE threads SET place = ?2, snoozed_until = NULL WHERE id = ?1",
            params![thread, place],
        )?;
        Ok(())
    }

    /// Puts a thread back where it was: its place, snooze and folders.
    pub fn restore_thread(
        &self,
        thread: &str,
        place: Place,
        snoozed_until: Option<i64>,
        folders: &[String],
    ) -> Result<()> {
        let tx = self.transaction()?;
        tx.execute(
            "UPDATE threads SET place = ?2, snoozed_until = ?3 WHERE id = ?1",
            params![thread, place, snoozed_until],
        )?;
        tx.execute("DELETE FROM thread_folders WHERE thread_id = ?1", [thread])?;
        for folder in folders {
            tx.execute(
                "INSERT INTO thread_folders VALUES (?1, ?2)",
                params![thread, folder],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn set_starred(&self, message: &str, starred: bool) -> Result<()> {
        self.run(
            "UPDATE messages SET starred = ?2 WHERE id = ?1",
            params![message, starred],
        )?;
        Ok(())
    }

    /// Files a thread in one folder the user made, out of the inbox and every other folder.
    pub fn move_to_folder(&self, thread: &str, folder: &str) -> Result<()> {
        let tx = self.transaction()?;
        Self::file(&tx, thread, folder)?;
        tx.commit()?;
        Ok(())
    }

    fn file(tx: &Transaction, thread: &str, folder: &str) -> Result<()> {
        tx.execute("DELETE FROM thread_folders WHERE thread_id = ?1", [thread])?;
        tx.execute(
            "INSERT INTO thread_folders VALUES (?1, ?2)",
            params![thread, folder],
        )?;
        tx.execute(
            "UPDATE threads SET place = ?2, snoozed_until = NULL WHERE id = ?1 AND place IN (?3, ?4)",
            params![thread, Place::Archive, Place::Inbox, Place::Trash],
        )?;
        Ok(())
    }

    /// A folder the user made, after the others and before Quarantine and Trash.
    pub fn create_folder(&self, id: &str, name: &str) -> Result<()> {
        let tx = self.transaction()?;
        let position: i64 = tx.query_row(
            "SELECT coalesce(max(position), 0) FROM folders WHERE kind IN (?1, ?2, ?3, ?4)",
            params![
                FolderKind::Starred,
                FolderKind::Sent,
                FolderKind::Archive,
                FolderKind::User
            ],
            |row| row.get(0),
        )?;
        tx.execute(
            "UPDATE folders SET position = position + 1 WHERE position > ?1",
            [position],
        )?;
        tx.execute(
            "INSERT INTO folders VALUES (?1, ?2, ?3, ?4)",
            params![id, position + 1, name, FolderKind::User],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Adds a message to an existing thread or a new one, with the search index, in one
    /// transaction. The thread moves to the top.
    fn add_mail(
        tx: &Transaction,
        thread: Option<&NewThread>,
        thread_id: &str,
        message: &NewMessage,
        unread: bool,
    ) -> Result<()> {
        if let Some(t) = thread {
            tx.execute(
                "INSERT INTO threads VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, NULL, ?10, NULL)",
                params![
                    t.id,
                    t.account,
                    t.place,
                    t.category,
                    t.from,
                    t.subject,
                    message.sent_at,
                    t.note.as_ref().map(|(text, _)| text),
                    t.note.as_ref().map(|(_, kind)| kind),
                    unread
                ],
            )?;
            for (i, name) in t.others.iter().enumerate() {
                tx.execute(
                    "INSERT INTO thread_people VALUES (?1, ?2, ?3)",
                    params![t.id, i as i64, name],
                )?;
            }
            if let Some(folder) = &t.folder {
                tx.execute(
                    "INSERT INTO thread_folders VALUES (?1, ?2)",
                    params![t.id, folder],
                )?;
            }
        } else {
            tx.execute(
                "UPDATE threads SET unread = unread OR ?2, place = CASE WHEN ?2 AND place IN (?3, ?4) THEN ?5 ELSE place END WHERE id = ?1",
                params![thread_id, unread, Place::Archive, Place::Sent, Place::Inbox],
            )?;
        }
        let position: i64 = tx.query_row(
            "SELECT count(*) FROM messages WHERE thread_id = ?1",
            [thread_id],
            |row| row.get(0),
        )?;
        tx.execute(
            "INSERT INTO messages VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL, 0)",
            params![
                message.id,
                thread_id,
                position,
                message.from_you,
                message.sender,
                preview(&message.body),
                message.sent_at,
                message.body
            ],
        )?;
        for (i, (name, size)) in message.attachments.iter().enumerate() {
            tx.execute(
                "INSERT INTO attachments VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    format!("{}-{i}", message.id),
                    message.id,
                    i as i64,
                    name,
                    size,
                    attachment_kind(name)
                ],
            )?;
        }
        tx.execute(
            "UPDATE threads SET latest_at = max(latest_at, ?2) WHERE id = ?1",
            params![thread_id, message.sent_at],
        )?;
        index_message(tx, &message.id)?;
        Ok(())
    }

    // Drafts and sending.

    pub fn create_draft(&self, draft: &NewDraft, now: i64) -> Result<String> {
        let id = self.next_id("drafts", "d")?;
        self.run(
            "INSERT INTO drafts VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                id,
                draft.kind,
                draft.thread,
                draft.message,
                draft.approval,
                draft.account,
                draft.recipients,
                draft.subject,
                draft.body,
                now
            ],
        )?;
        Ok(id)
    }

    pub fn update_draft(
        &self,
        draft: &str,
        to: &str,
        subject: &str,
        body: &str,
        now: i64,
    ) -> Result<()> {
        self.run(
            "UPDATE drafts SET recipients = ?2, subject = ?3, body = ?4, updated_at = ?5 WHERE id = ?1",
            params![draft, to, subject, body, now],
        )?;
        Ok(())
    }

    pub fn set_draft_account(&self, draft: &str, account: &str) -> Result<()> {
        self.run(
            "UPDATE drafts SET account_id = ?2 WHERE id = ?1",
            params![draft, account],
        )?;
        Ok(())
    }

    /// Attaches a file last, under a new id.
    pub fn attach(&self, draft: &str, name: &str, size: i64) -> Result<()> {
        let id = self.next_id("draft_attachments", "attachment")?;
        self.run(
            "INSERT INTO draft_attachments VALUES (?1, ?2, (SELECT coalesce(max(position) + 1, 0) FROM draft_attachments WHERE draft_id = ?2), ?3, ?4)",
            params![id, draft, name, size],
        )?;
        Ok(())
    }

    pub fn remove_attachment(&self, draft: &str, attachment: &str) -> Result<()> {
        self.run(
            "DELETE FROM draft_attachments WHERE draft_id = ?1 AND id = ?2",
            params![draft, attachment],
        )?;
        Ok(())
    }

    pub fn discard_draft(&self, draft: &str) -> Result<()> {
        self.run("DELETE FROM drafts WHERE id = ?1", [draft])?;
        Ok(())
    }

    // Calendar.

    pub fn save_event(&self, id: &str, values: &EventValues, create: bool) -> Result<()> {
        let tx = self.transaction()?;
        if create {
            tx.execute(
                "INSERT INTO events VALUES (?1, ?2, ?3, ?4, ?5, ?6, coalesce(?7, ''))",
                params![
                    id,
                    values.calendar,
                    values.day,
                    values.start,
                    values.end,
                    values.title,
                    values.place
                ],
            )?;
        } else {
            tx.execute(
                "UPDATE events SET calendar_id = ?2, day = ?3, start_minute = ?4, end_minute = ?5, title = ?6, place = coalesce(?7, place) WHERE id = ?1",
                params![
                    id,
                    values.calendar,
                    values.day,
                    values.start,
                    values.end,
                    values.title,
                    values.place
                ],
            )?;
        }
        tx.execute("DELETE FROM event_guests WHERE event_id = ?1", [id])?;
        for (i, guest) in values.guests.iter().enumerate() {
            tx.execute(
                "INSERT INTO event_guests VALUES (?1, ?2, ?3)",
                params![id, i as i64, guest],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn delete_event(&self, id: &str) -> Result<()> {
        self.run("DELETE FROM events WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn set_calendar_shown(&self, calendar: &str, shown: bool) -> Result<()> {
        self.run(
            "UPDATE calendars SET shown = ?2 WHERE id = ?1",
            params![calendar, shown],
        )?;
        Ok(())
    }

    pub fn answer_invitation(&self, event: &str, answer: Answer) -> Result<()> {
        self.run(
            "UPDATE invitations SET answer = ?2 WHERE event_id = ?1",
            params![event, answer],
        )?;
        Ok(())
    }

    // Agents.

    pub fn set_agent_active(&self, agent: &str, active: bool) -> Result<()> {
        self.run(
            "UPDATE agents SET active = ?2 WHERE id = ?1",
            params![agent, active],
        )?;
        Ok(())
    }

    pub fn set_agent_instructions(&self, agent: &str, text: &str) -> Result<()> {
        self.run(
            "UPDATE agents SET instructions = ?2 WHERE id = ?1",
            params![agent, text],
        )?;
        Ok(())
    }

    pub fn add_agent_time(&self, agent: &str, minute: u32) -> Result<()> {
        self.run(
            "INSERT INTO agent_times VALUES (?1, ?2)",
            params![agent, minute],
        )?;
        Ok(())
    }

    pub fn remove_agent_time(&self, agent: &str, minute: u32) -> Result<()> {
        self.run(
            "DELETE FROM agent_times WHERE agent_id = ?1 AND minute = ?2",
            params![agent, minute],
        )?;
        Ok(())
    }

    pub fn set_agent_model(&self, agent: &str, model: &str, provider: ModelProvider) -> Result<()> {
        self.run(
            "UPDATE agents SET model_name = ?2, model_provider = ?3 WHERE id = ?1",
            params![agent, model, provider],
        )?;
        Ok(())
    }

    pub fn remove_news_topic(&self, topic: &str) -> Result<()> {
        self.run("DELETE FROM news_topics WHERE name = ?1", [topic])?;
        Ok(())
    }

    /// Removes an agent with everything that is its own.
    pub fn remove_agent(&self, agent: &str) -> Result<()> {
        self.run("DELETE FROM agents WHERE id = ?1", [agent])?;
        Ok(())
    }

    pub fn add_turn(&self, agent: &str, message: &str, label: &str, now: i64) -> Result<String> {
        let id = self.next_id("turns", "turn")?;
        self.run(
            "INSERT INTO turns VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL)",
            params![id, agent, now, message, TurnKind::Running, label],
        )?;
        Ok(id)
    }

    pub fn clear_turns(&self, agent: &str) -> Result<()> {
        self.run("DELETE FROM turns WHERE agent_id = ?1", [agent])?;
        Ok(())
    }

    /// The user answered a waiting approval: it resolves until the Backend reports back.
    pub fn resolving(&self, approval: &str) -> Result<()> {
        self.run(
            "UPDATE approvals SET status = ?2 WHERE id = ?1 AND status = ?3",
            params![approval, StatusKind::Resolving, StatusKind::Waiting],
        )?;
        Ok(())
    }

    /// Editions shown are read.
    pub fn read_editions(&self) -> Result<()> {
        self.run("UPDATE editions SET unread = 0 WHERE unread", [])?;
        Ok(())
    }

    // Quarantine.

    /// Not spam: the mail becomes a thread in the inbox and its sender leaves the block list.
    pub fn release_quarantined(&self, id: &str, thread: &str) -> Result<()> {
        let tx = self.transaction()?;
        let (sender, account, received_at, subject, body): (String, String, i64, String, String) =
            tx.query_row(
                "SELECT sender, account_id, received_at, subject, body FROM quarantine WHERE id = ?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )?;
        let new_thread = NewThread {
            id: thread.into(),
            account,
            place: Place::Inbox,
            category: InboxCategory::ForYou,
            from: sender.clone(),
            subject,
            others: Vec::new(),
            note: None,
            folder: None,
        };
        let message = NewMessage {
            id: format!("{thread}-1"),
            from_you: false,
            sender: sender.clone(),
            sent_at: received_at,
            body,
            attachments: Vec::new(),
        };
        Self::add_mail(&tx, Some(&new_thread), thread, &message, true)?;
        tx.execute("DELETE FROM blocked WHERE address = ?1", [&sender])?;
        tx.execute("DELETE FROM quarantine WHERE id = ?1", [id])?;
        tx.commit()?;
        Ok(())
    }

    pub fn delete_quarantined(&self, id: &str) -> Result<()> {
        self.run("DELETE FROM quarantine WHERE id = ?1", [id])?;
        Ok(())
    }

    // Settings.

    /// A rule, enabled, after the others.
    pub fn add_rule(&self, id: &str, condition: &Condition, actions: &[RuleAction]) -> Result<()> {
        let tx = self.transaction()?;
        let (kind, parameter, words): (ConditionKind, Option<&str>, &[String]) = match condition {
            Condition::SenderContains(text) => (ConditionKind::SenderContains, Some(text), &[]),
            Condition::SenderDomain(domain) => (ConditionKind::SenderDomain, Some(domain), &[]),
            Condition::SubjectContains(words) => (ConditionKind::SubjectContains, None, words),
            Condition::Newsletter => (ConditionKind::Newsletter, None, &[]),
            Condition::NewSenderWithLink => (ConditionKind::NewSenderWithLink, None, &[]),
        };
        tx.execute(
            "INSERT INTO rules VALUES (?1, (SELECT coalesce(max(position), -1) + 1 FROM rules), ?2, ?3, 1)",
            params![id, kind, parameter],
        )?;
        for (i, word) in words.iter().enumerate() {
            tx.execute(
                "INSERT INTO rule_words VALUES (?1, ?2, ?3)",
                params![id, i as i64, word],
            )?;
        }
        for (i, action) in actions.iter().enumerate() {
            let (kind, folder, agent) = match action {
                RuleAction::MoveTo(folder) => (ActionKind::MoveTo, Some(folder), None),
                RuleAction::Archive => (ActionKind::Archive, None, None),
                RuleAction::Trash => (ActionKind::Trash, None, None),
                RuleAction::MarkImportant => (ActionKind::MarkImportant, None, None),
                RuleAction::HandTo(agent) => (ActionKind::HandTo, None, Some(agent)),
                RuleAction::KeepForBriefing => (ActionKind::KeepForBriefing, None, None),
            };
            tx.execute(
                "INSERT INTO rule_actions VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, i as i64, kind, folder, agent],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn set_rule_enabled(&self, id: &str, enabled: bool) -> Result<()> {
        self.run(
            "UPDATE rules SET enabled = ?2 WHERE id = ?1",
            params![id, enabled],
        )?;
        Ok(())
    }

    pub fn remove_rule(&self, id: &str) -> Result<()> {
        self.run("DELETE FROM rules WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn unblock(&self, address: &str) -> Result<()> {
        self.run("DELETE FROM blocked WHERE address = ?1", [address])?;
        Ok(())
    }

    pub fn set_task_model(&self, task: &str, model: &str) -> Result<()> {
        self.run(
            "UPDATE task_models SET model_name = ?2 WHERE id = ?1",
            params![task, model],
        )?;
        Ok(())
    }

    pub fn set_notify(&self, notify: bool) -> Result<()> {
        self.run("UPDATE settings SET notify_when_waiting = ?1", [notify])?;
        Ok(())
    }

    pub fn set_compose_tone(&self, tone: &str) -> Result<()> {
        self.run("UPDATE settings SET compose_tone = ?1", [tone])?;
        Ok(())
    }

    // The agent builder.

    pub fn update_builder(&self, name: &str, instructions: &str) -> Result<()> {
        self.run(
            "UPDATE builder SET name = ?1, instructions = ?2",
            params![name, instructions],
        )?;
        Ok(())
    }

    pub fn set_builder_trigger(&self, trigger: &str, chosen: bool) -> Result<()> {
        self.run(
            "INSERT INTO builder_triggers VALUES (?1, ?2) ON CONFLICT (trigger_id) DO UPDATE SET chosen = ?2",
            params![trigger, chosen],
        )?;
        Ok(())
    }

    pub fn set_trigger_filter(&self, trigger: &str, filter: &str, choice: &str) -> Result<()> {
        self.run(
            "INSERT INTO builder_filters VALUES (?1, ?2, ?3) ON CONFLICT (trigger_id, filter_id) DO UPDATE SET choice_id = ?3",
            params![trigger, filter, choice],
        )?;
        Ok(())
    }

    pub fn set_builder_schedule(&self, schedule: &str) -> Result<()> {
        self.run("UPDATE builder SET schedule = ?1", [schedule])?;
        Ok(())
    }

    pub fn set_ask_before_new_recipient(&self, ask: bool) -> Result<()> {
        self.run("UPDATE builder SET ask_before_new_recipient = ?1", [ask])?;
        Ok(())
    }

    /// Starts the builder over.
    pub fn reset_builder(&self) -> Result<()> {
        let tx = self.transaction()?;
        reset_builder(&tx)?;
        tx.commit()?;
        Ok(())
    }

    /// Creates an agent from the builder's triggers, accounts and permissions, as the last of the
    /// agents, and starts the builder over. Its schedule counts from `now`.
    pub fn create_agent(&self, agent: &NewAgent, now: i64) -> Result<()> {
        let tx = self.transaction()?;
        let (kind, note, weekdays, interval) = schedule_row(agent.schedule);
        tx.execute(
            "INSERT INTO agents VALUES (?1, (SELECT coalesce(max(position), -1) + 1 FROM agents), ?2, NULL, 1, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'Off until you add a model key', ?11, 1)",
            params![
                agent.id,
                agent.name,
                agent.instructions,
                agent.model,
                agent.model_provider,
                kind,
                note,
                weekdays,
                interval,
                now,
                crate::world::ResultsKind::Groups
            ],
        )?;
        tx.execute(
            "INSERT INTO agent_triggers SELECT ?1, trigger_id FROM builder_triggers WHERE chosen",
            [agent.id],
        )?;
        tx.execute(
            "INSERT INTO agent_filters SELECT ?1, f.trigger_id, f.filter_id, f.choice_id FROM builder_filters f JOIN builder_triggers t ON t.trigger_id = f.trigger_id AND t.chosen",
            [agent.id],
        )?;
        if let Schedule::Times { minutes, .. } = agent.schedule {
            for minute in minutes {
                tx.execute(
                    "INSERT INTO agent_times VALUES (?1, ?2)",
                    params![agent.id, minute],
                )?;
            }
        }
        tx.execute(
            "INSERT INTO agent_permissions SELECT ?1, action_id, level, ask_at_most FROM builder_permissions",
            [agent.id],
        )?;
        tx.execute(
            "INSERT INTO agent_accounts SELECT ?1, account_id FROM builder_accounts",
            [agent.id],
        )?;
        tx.execute(
            "INSERT INTO agent_stats VALUES (?1, 0, '0', 'runs'), (?1, 1, '0', 'waiting')",
            [agent.id],
        )?;
        reset_builder(&tx)?;
        tx.commit()?;
        Ok(())
    }

    // What the Backend sends.

    /// Applies an update from the Backend in one transaction.
    pub fn apply(&self, update: &Update, now: i64) -> Result<()> {
        let tx = self.transaction()?;
        match update {
            Update::SignedIn {
                user_name,
                provider,
            } => upsert_session(&tx, user_name, *provider)?,
            Update::Snapshot(rows) => {
                clear_data(&tx)?;
                tx.execute_batch(rows)?;
                index_all(&tx)?;
            }
            Update::AccountAdded(account) => {
                tx.execute(
                    "INSERT INTO accounts VALUES (?1, (SELECT coalesce(max(position), -1) + 1 FROM accounts), ?2, ?3, ?4, ?5, ?6, 0, NULL, ?7, NULL)",
                    params![
                        account.id,
                        account.name,
                        account.address,
                        account.provider,
                        account.provider_name,
                        AccountKind::Syncing,
                        now
                    ],
                )?;
                tx.execute("INSERT INTO builder_accounts VALUES (?1)", [&account.id])?;
            }
            Update::SyncProgress { account, percent } => {
                tx.execute(
                    "UPDATE accounts SET sync_percent = ?2 WHERE id = ?1 AND state = ?3",
                    params![account, percent, AccountKind::Syncing],
                )?;
            }
            Update::AccountSynced(account) => {
                tx.execute(
                    "UPDATE accounts SET state = ?3, sync_percent = NULL, expired_reason = NULL, synced_at = ?2 WHERE id = ?1",
                    params![account, now, AccountKind::Synced],
                )?;
            }
            Update::Fetched(account) => {
                tx.execute(
                    "UPDATE accounts SET synced_at = ?2 WHERE id = ?1",
                    params![account, now],
                )?;
            }
            Update::Mail {
                thread,
                thread_id,
                message,
            } => Self::add_mail(&tx, thread.as_ref(), thread_id, message, !message.from_you)?,
            Update::Sent { draft, message, at } => Self::sent(&tx, draft, message, *at)?,
            Update::Outbox { draft, state } => {
                let (kind, due, failure) = send_row(state);
                tx.execute(
                    "INSERT INTO outbox VALUES (?1, ?2, ?3, ?4) ON CONFLICT (draft_id) DO UPDATE SET state = ?2, due_at = ?3, failure = ?4",
                    params![draft, kind, due, failure],
                )?;
            }
            Update::SendCancelled(draft) => {
                tx.execute("DELETE FROM outbox WHERE draft_id = ?1", [draft])?;
            }
            Update::Snoozed { thread, until } => {
                tx.execute(
                    "UPDATE threads SET snoozed_until = ?2, place = ?3 WHERE id = ?1",
                    params![thread, until, Place::Inbox],
                )?;
            }
            Update::SnoozeEnded(thread) => {
                tx.execute(
                    "UPDATE threads SET latest_at = max(latest_at, snoozed_until), snoozed_until = NULL, unread = 1 WHERE id = ?1 AND snoozed_until IS NOT NULL",
                    [thread],
                )?;
            }
            Update::SnoozeTimes(times) => {
                tx.execute("DELETE FROM snooze_times", [])?;
                for (preset, until) in times {
                    tx.execute(
                        "INSERT INTO snooze_times VALUES (?1, ?2)",
                        params![preset, until],
                    )?;
                }
            }
            Update::Summary { thread, text } => {
                tx.execute(
                    "UPDATE threads SET summary = ?2 WHERE id = ?1",
                    params![thread, text],
                )?;
            }
            Update::RunStarted { agent, run, cause } => {
                let (kind, text, refs) = cause_row(cause);
                tx.execute(
                    "INSERT INTO runs VALUES (?1, ?2, ?3, ?4, ?5, NULL, 0, ?6, NULL, 'Running')",
                    params![run, agent, now, kind, text, OutcomeKind::Running],
                )?;
                add_refs(&tx, Holder::Cause(run), &refs)?;
            }
            Update::Woke { agent, at } => {
                tx.execute(
                    "UPDATE agents SET woken_at = ?2 WHERE id = ?1",
                    params![agent, at],
                )?;
            }
            Update::RunPaused(run) => {
                tx.execute(
                    "UPDATE runs SET outcome = ?2 WHERE id = ?1 AND outcome = ?3",
                    params![run, OutcomeKind::Paused, OutcomeKind::Running],
                )?;
            }
            Update::RunResumed(run) => {
                tx.execute(
                    "UPDATE runs SET outcome = ?2 WHERE id = ?1 AND outcome = ?3",
                    params![run, OutcomeKind::Running, OutcomeKind::Paused],
                )?;
            }
            Update::RunStep { run, step } => {
                // A stopped run takes no more steps.
                let position: Option<i64> = tx
                    .query_row(
                        "SELECT (SELECT count(*) FROM run_steps WHERE run_id = ?1) FROM runs WHERE id = ?1 AND outcome = ?2",
                        params![run, OutcomeKind::Running],
                        |row| row.get(0),
                    )
                    .optional()?;
                if let Some(position) = position {
                    tx.execute(
                        "INSERT INTO run_steps VALUES (?1, ?2, ?3, ?4, ?5)",
                        params![run, position, step.verb, step.what, step.permission],
                    )?;
                    add_refs(&tx, Holder::Step(run, position), &step.refs)?;
                }
            }
            Update::RunFinished {
                run,
                outcome,
                summary,
                seconds,
                tokens,
            } => {
                let (outcome, text) = outcome_row(outcome);
                // A run that ended keeps its outcome.
                tx.execute(
                    "UPDATE runs SET outcome = ?2, outcome_text = ?3, summary = ?4, seconds = ?5, tokens = ?6 WHERE id = ?1 AND outcome IN (?7, ?8, ?9)",
                    params![
                        run,
                        outcome,
                        text,
                        summary,
                        seconds,
                        tokens,
                        OutcomeKind::Waiting,
                        OutcomeKind::Running,
                        OutcomeKind::Paused
                    ],
                )?;
            }
            Update::Result { agent, item } => Self::add_result(&tx, agent, item)?,
            Update::Approval(approval) => Self::add_approval(&tx, approval)?,
            Update::Resolved { approval, status } => {
                let (kind, reason) = status_row(status);
                tx.execute(
                    "UPDATE approvals SET status = ?2, declined_reason = ?3 WHERE id = ?1",
                    params![approval, kind, reason],
                )?;
            }
            Update::Filed { thread, folder } => Self::file(&tx, thread, folder)?,
            Update::Quarantined(q) => Self::quarantine(&tx, q, now)?,
            Update::QuarantineDropped(id) => {
                tx.execute("DELETE FROM quarantine WHERE id = ?1", [id])?;
            }
            Update::Turn { turn, state } => {
                let (kind, text, approval) = turn_row(state);
                tx.execute(
                    "UPDATE turns SET state = ?2, text = ?3, approval_id = ?4 WHERE id = ?1 AND state IN (?5, ?6)",
                    params![
                        turn,
                        kind,
                        text,
                        approval,
                        TurnKind::Running,
                        TurnKind::Waiting
                    ],
                )?;
                if let TurnState::Answered(_, refs) = state {
                    add_refs(&tx, Holder::Turn(turn), refs)?;
                }
            }
            Update::Edition(edition) => Self::add_edition(&tx, edition)?,
            Update::Permission {
                owner,
                action,
                level,
                levels,
            } => {
                let limit = levels_row(levels);
                match owner {
                    Owner::Agent(agent) => tx.execute(
                        "UPDATE agent_permissions SET level = ?3, ask_at_most = ?4 WHERE agent_id = ?1 AND action_id = ?2",
                        params![agent, action, level, limit],
                    ),
                    Owner::Defaults => tx.execute(
                        "UPDATE default_permissions SET level = ?2, ask_at_most = ?3 WHERE action_id = ?1",
                        params![action, level, limit],
                    ),
                    Owner::Builder => tx.execute(
                        "UPDATE builder_permissions SET level = ?2, ask_at_most = ?3 WHERE action_id = ?1",
                        params![action, level, limit],
                    ),
                }?;
            }
            Update::Accounts { owner, accounts } => match owner {
                Owner::Agent(agent) => {
                    tx.execute("DELETE FROM agent_accounts WHERE agent_id = ?1", [agent])?;
                    for account in accounts {
                        tx.execute(
                            "INSERT INTO agent_accounts VALUES (?1, ?2)",
                            params![agent, account],
                        )?;
                    }
                }
                Owner::Builder => {
                    tx.execute("DELETE FROM builder_accounts", [])?;
                    for account in accounts {
                        tx.execute("INSERT INTO builder_accounts VALUES (?1)", [account])?;
                    }
                }
                Owner::Defaults => {}
            },
            Update::TopicAdded(topic) => {
                tx.execute(
                    "INSERT INTO news_topics VALUES ((SELECT coalesce(max(position), -1) + 1 FROM news_topics), ?1)",
                    [topic],
                )?;
            }
            Update::Interval(seconds) => {
                tx.execute(
                    "UPDATE builder SET interval_seconds = ?1, schedule = ?2",
                    params![seconds, builder::CUSTOM.id],
                )?;
            }
            Update::Key { provider, set, ai } => {
                tx.execute(
                    "UPDATE model_keys SET is_set = ?2 WHERE provider = ?1",
                    params![provider, set],
                )?;
                tx.execute("UPDATE settings SET ai = ?1", [ai])?;
            }
            Update::Budget(tokens) => {
                tx.execute("UPDATE settings SET budget_tokens = ?1", [tokens])?;
            }
            Update::AgentsPaused(paused) => {
                tx.execute("UPDATE settings SET agents_paused = ?1", [paused])?;
            }
            Update::UserDeleted => {
                clear_data(&tx)?;
                tx.execute("DELETE FROM session", [])?;
            }
            Update::Refused { .. }
            | Update::Failed { .. }
            | Update::ConnectRejected(_)
            | Update::Reply { .. }
            | Update::DryRun(_)
            | Update::ExportProgress(_)
            | Update::ExportReady { .. } => {}
        }
        tx.commit()?;
        Ok(())
    }

    /// A draft became a sent message: in its thread, or a thread of its own in Sent. The approval
    /// it answers is done.
    fn sent(tx: &Transaction, draft: &str, message: &str, at: i64) -> Result<()> {
        type Row = (
            DraftKind,
            Option<String>,
            String,
            String,
            String,
            String,
            Option<String>,
        );
        let row: Option<Row> = tx
            .query_row(
                "SELECT kind, thread_id, account_id, recipients, subject, body, approval_id FROM drafts WHERE id = ?1",
                [draft],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                    ))
                },
            )
            .optional()?;
        let Some((kind, thread, account, to, subject, body, approval)) = row else {
            return Ok(());
        };
        let user: String = tx
            .query_row("SELECT user_name FROM session", [], |row| row.get(0))
            .optional()?
            .unwrap_or_default();
        let attachments: Vec<(String, i64)> = tx
            .prepare("SELECT name, size_bytes FROM draft_attachments WHERE draft_id = ?1 ORDER BY position")?
            .query_map([draft], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<rusqlite::Result<_>>()?;
        let new_message = NewMessage {
            id: message.into(),
            from_you: true,
            sender: user,
            sent_at: at,
            body,
            attachments,
        };
        match thread.filter(|_| matches!(kind, DraftKind::Reply | DraftKind::ReplyAll)) {
            Some(thread) => Self::add_mail(tx, None, &thread, &new_message, false)?,
            None => {
                let mut people = to
                    .split(',')
                    .map(|p| p.trim().to_string())
                    .filter(|p| !p.is_empty());
                let new_thread = NewThread {
                    id: format!("t-{message}"),
                    account,
                    place: Place::Sent,
                    category: InboxCategory::ForYou,
                    from: people.next().unwrap_or_default(),
                    subject,
                    others: people.collect(),
                    note: None,
                    folder: None,
                };
                Self::add_mail(tx, Some(&new_thread), &new_thread.id, &new_message, false)?;
            }
        }
        if let Some(approval) = approval {
            tx.execute(
                "UPDATE approvals SET status = ?2 WHERE id = ?1",
                params![approval, StatusKind::Done],
            )?;
        }
        tx.execute("DELETE FROM drafts WHERE id = ?1", [draft])?;
        Ok(())
    }

    /// Mail kept out of the inbox: a new quarantined copy, the thread it was gone, and its sender
    /// blocked.
    fn quarantine(tx: &Transaction, q: &NewQuarantined, now: i64) -> Result<()> {
        tx.execute(
            "INSERT INTO quarantine VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                q.id,
                q.sender,
                q.account,
                q.received_at,
                q.subject,
                q.reason,
                q.caught_by,
                q.why,
                q.outcome,
                q.body,
                q.drops_at
            ],
        )?;
        if let Some(thread) = &q.thread {
            unindex(tx, "id = ?1", thread)?;
            tx.execute("DELETE FROM threads WHERE id = ?1", [thread])?;
        }
        if let Some((address, by)) = &q.block {
            tx.execute(
                "INSERT OR IGNORE INTO blocked VALUES (?1, (SELECT coalesce(max(position), -1) + 1 FROM blocked), ?2, ?3)",
                params![address, by, now],
            )?;
        }
        Ok(())
    }

    fn add_result(tx: &Transaction, agent: &str, item: &ResultItem) -> Result<()> {
        // A group new to the agent goes first; an item goes first in its group.
        let group: i64 = tx
            .query_row(
                "SELECT group_position FROM result_items WHERE agent_id = ?1 AND group_title = ?2 LIMIT 1",
                params![agent, item.group],
                |row| row.get(0),
            )
            .optional()?
            .map_or_else(
                || {
                    tx.execute(
                        "UPDATE result_items SET group_position = group_position + 1 WHERE agent_id = ?1",
                        [agent],
                    )
                    .map(|_| 0)
                },
                Ok,
            )?;
        tx.execute(
            "UPDATE result_items SET position = position + 1 WHERE agent_id = ?1 AND group_title = ?2",
            params![agent, item.group],
        )?;
        tx.execute(
            "INSERT INTO result_items VALUES (?1, ?2, ?3, ?4, 0, ?5, ?6, ?7)",
            params![
                item.id,
                agent,
                item.group,
                group,
                item.at,
                item.title,
                item.detail
            ],
        )?;
        add_refs(tx, Holder::Item(&item.id), &item.refs)?;
        Ok(())
    }

    fn add_approval(tx: &Transaction, a: &Approval) -> Result<()> {
        let (action, thread) = match &a.action {
            ApprovalAction::Send(thread) => (ApprovalKind::Send, Some(thread.as_str())),
            ApprovalAction::Unsubscribe => (ApprovalKind::Unsubscribe, None),
        };
        tx.execute(
            "INSERT INTO approvals VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, NULL)",
            params![
                a.id,
                a.agent,
                a.created_at,
                a.title,
                a.detail,
                a.reason,
                action,
                thread,
                StatusKind::Waiting,
                a.draft
            ],
        )?;
        Ok(())
    }

    fn add_edition(tx: &Transaction, e: &Edition) -> Result<()> {
        tx.execute(
            "INSERT INTO editions VALUES (?1, ?2, ?3, ?4, ?5, 1)",
            params![
                e.id,
                e.written_at,
                e.byline,
                e.caught_mail,
                e.calendar_title
            ],
        )?;
        for (section, lines) in [
            (Section::Activity, &e.activity),
            (Section::Newsletters, &e.newsletters),
            (Section::Updates, &e.updates),
        ] {
            for (i, line) in lines.iter().enumerate() {
                tx.execute(
                    "INSERT INTO edition_lines VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![e.id, section, i as i64, line.source, line.text],
                )?;
            }
        }
        for (i, (approval, title, detail)) in e.waiting.iter().enumerate() {
            tx.execute(
                "INSERT INTO edition_waiting VALUES (?1, ?2, ?3, ?4, ?5)",
                params![e.id, i as i64, approval, title, detail],
            )?;
        }
        for (i, (time, title, detail)) in e.calendar.iter().enumerate() {
            tx.execute(
                "INSERT INTO edition_events VALUES (?1, ?2, ?3, ?4, ?5)",
                params![e.id, i as i64, time, title, detail],
            )?;
        }
        let mut i = 0;
        for (topic, items) in &e.news {
            for (headline, source) in items {
                tx.execute(
                    "INSERT INTO edition_news VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![e.id, topic, i, headline, source],
                )?;
                i += 1;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn previews_are_the_first_line_cut_short() {
        assert_eq!(preview("\nHello Mira,\n\nMore"), "Hello Mira,");
        let long = "word ".repeat(30);
        assert!(preview(&long).ends_with('…'));
        assert_eq!(preview(&long).chars().count(), 91);
    }
}
