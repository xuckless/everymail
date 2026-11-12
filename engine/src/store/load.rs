//! Reads the replica into a `World`. Codes read through `codes`; a row whose state lacks the data
//! it needs is an error, never a default.

use std::collections::HashMap;

use rusqlite::{Connection, Row};

use super::{
    Result, Store, StoreError,
    codes::{
        AccountKind, ActionKind, ApprovalKind, CauseKind, ConditionKind, OutcomeKind, RefKind,
        ScheduleKind, Section, SendKind, StatusKind, TurnKind,
    },
};
use crate::{builder, date::Date, preferences::Preferences, schedule::Schedule, world::*};

fn rows<T>(
    connection: &Connection,
    sql: &str,
    map: impl FnMut(&Row) -> rusqlite::Result<T>,
) -> Result<Vec<T>> {
    Ok(connection
        .prepare(sql)?
        .query_map([], map)?
        .collect::<rusqlite::Result<_>>()?)
}

/// Rows keyed by their first column, in query order.
fn grouped<T>(
    connection: &Connection,
    sql: &str,
    mut map: impl FnMut(&Row) -> rusqlite::Result<T>,
) -> Result<HashMap<String, Vec<T>>> {
    let mut groups: HashMap<String, Vec<T>> = HashMap::new();
    for (key, value) in rows(connection, sql, |row| {
        Ok((row.get::<_, String>(0)?, map(row)?))
    })? {
        groups.entry(key).or_default().push(value);
    }
    Ok(groups)
}

/// A reference from its kind and target, at `index` and the column after.
fn reference(row: &Row, index: usize) -> rusqlite::Result<Ref> {
    let target: String = row.get(index + 1)?;
    Ok(match row.get::<_, RefKind>(index)? {
        RefKind::Thread => Ref::Thread(target),
        RefKind::Event => Ref::Event(target),
        RefKind::Approval => Ref::Approval(target),
        RefKind::Quarantined => Ref::Quarantined(target),
    })
}

/// Permissions with their action's label and hint, in the actions' order.
fn permissions(
    c: &Connection,
    table: &str,
    owner: &str,
) -> Result<HashMap<String, Vec<Permission>>> {
    grouped(
        c,
        &format!(
            "SELECT {owner}, a.id, a.label, a.hint, p.level, p.ask_at_most FROM {table} p JOIN actions a ON a.id = p.action_id ORDER BY {owner}, a.position"
        ),
        |row| {
            Ok(Permission {
                id: row.get(1)?,
                label: row.get(2)?,
                hint: row.get(3)?,
                level: row.get(4)?,
                levels: match row.get::<_, Option<String>>(5)? {
                    Some(reason) => Levels::AskAtMost(reason),
                    None => Levels::All,
                },
            })
        },
    )
}

fn stat(row: &Row) -> rusqlite::Result<Stat> {
    Ok(Stat {
        value: row.get(1)?,
        label: row.get(2)?,
    })
}

fn broken(what: &str, id: &str) -> StoreError {
    StoreError(format!("{what} {id} lacks what its state needs"))
}

/// A run's cause from its kind, its text and its references.
fn cause(run: &str, kind: CauseKind, text: Option<String>, refs: Vec<Ref>) -> Result<Cause> {
    let text = || text.clone().ok_or_else(|| broken("run", run));
    let one = || refs.first().cloned().ok_or_else(|| broken("run", run));
    Ok(match kind {
        CauseKind::Mail => Cause::Mail(one()?),
        CauseKind::Opened => Cause::Opened(one()?),
        CauseKind::Schedule => Cause::Schedule(text()?),
        CauseKind::Message => Cause::Message(text()?),
        CauseKind::Handoff => Cause::Handoff(text()?, one()?),
        CauseKind::Connected => Cause::Connected(text()?),
        CauseKind::RunNow => Cause::RunNow,
        CauseKind::Sorted => Cause::Sorted(text()?, one()?),
        CauseKind::Flagged => Cause::Flagged(one()?),
        CauseKind::Starred => Cause::Starred(one()?),
        CauseKind::Finished => Cause::Finished(text()?, refs.clone()),
        CauseKind::Resolved => Cause::Resolved(one()?),
        CauseKind::Approaching => Cause::Approaching(text()?, one()?),
    })
}

/// A run as stored, before its cause is put together.
struct RunRow {
    run: Run,
    cause: CauseKind,
    cause_text: Option<String>,
}

/// The world before the first sync: the session, if any, and no data.
fn unsynced(now: i64, user: Option<String>) -> World {
    World {
        now,
        user,
        accounts: Vec::new(),
        folders: Vec::new(),
        threads: Vec::new(),
        agents: Vec::new(),
        approvals: Vec::new(),
        models: Vec::new(),
        calendars: Vec::new(),
        events: Vec::new(),
        quarantine: Vec::new(),
        editions: Vec::new(),
        news_topics: Vec::new(),
        drafts: Vec::new(),
        settings: Settings {
            defaults: Vec::new(),
            notify_when_waiting: false,
            agents_paused: false,
            rules: Vec::new(),
            blocked: Vec::new(),
            keys: Vec::new(),
            tasks: Vec::new(),
            budget_tokens: None,
            used_tokens: 0,
            compose_tone: String::new(),
            ai: Ai::NoKey,
            quarantine_days: 0,
            dry_run_messages: 0,
        },
        builder: Builder {
            name: String::new(),
            instructions: String::new(),
            schedule: builder::NO_SCHEDULE,
            interval_seconds: crate::schedule::MIN_INTERVAL_SECONDS,
            ask_before_new_recipient: true,
            wakes: Wakes::default(),
            accounts: Vec::new(),
            permissions: Vec::new(),
        },
        search: None,
        snoozes: Vec::new(),
    }
}

impl Store {
    /// Everything the views read. `search` is the navigation's query, if any.
    pub fn load(&self, now: i64, search: Option<&str>) -> Result<World> {
        let c = self.connection();
        let user = rows(c, "SELECT user_name FROM session", |row| row.get(0))?
            .into_iter()
            .next();
        let synced: bool = c.query_row("SELECT EXISTS (SELECT 1 FROM settings)", [], |row| {
            row.get(0)
        })?;
        if !synced {
            return Ok(unsynced(now, user));
        }
        let ref_rows = |table: &str, owner: &str| {
            grouped(
                c,
                &format!("SELECT {owner}, kind, target FROM {table} ORDER BY {owner}, position"),
                |row| reference(row, 1),
            )
        };
        let mut item_refs = ref_rows("item_refs", "item_id")?;
        let mut cause_refs = ref_rows("cause_refs", "run_id")?;
        let mut turn_refs = ref_rows("turn_refs", "turn_id")?;
        let mut step_refs = grouped(
            c,
            "SELECT run_id || '/' || step, kind, target FROM step_refs ORDER BY run_id, step, position",
            |row| reference(row, 1),
        )?;
        let accounts = rows(
            c,
            "SELECT id, name, address, provider, provider_name, state, sync_percent, expired_reason, synced_at, signature FROM accounts ORDER BY position",
            |row| {
                Ok(Account {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    address: row.get(2)?,
                    provider: row.get(3)?,
                    provider_name: row.get(4)?,
                    state: match row.get::<_, AccountKind>(5)? {
                        AccountKind::Synced => AccountState::Synced,
                        AccountKind::Syncing => AccountState::Syncing(row.get(6)?),
                        AccountKind::SignInExpired => AccountState::SignInExpired(row.get(7)?),
                    },
                    synced_at: row.get(8)?,
                    signature: row.get(9)?,
                })
            },
        )?;
        let folders = rows(
            c,
            "SELECT id, name, kind FROM folders ORDER BY position",
            |row| {
                Ok(Folder {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    kind: row.get(2)?,
                })
            },
        )?;
        let mut attachments = grouped(
            c,
            "SELECT message_id, id, name, size_bytes, kind FROM attachments ORDER BY message_id, position",
            |row| {
                Ok(Attachment {
                    id: row.get(1)?,
                    name: row.get(2)?,
                    size_bytes: row.get(3)?,
                    kind: row.get(4)?,
                })
            },
        )?;
        let mut messages = grouped(
            c,
            "SELECT thread_id, id, from_you, sender, preview, sent_at, body, invitation_event, starred FROM messages ORDER BY thread_id, position",
            |row| {
                Ok(Message {
                    attachments: Vec::new(),
                    id: row.get(1)?,
                    from_you: row.get(2)?,
                    sender: row.get(3)?,
                    preview: row.get(4)?,
                    sent_at: row.get(5)?,
                    body: row.get(6)?,
                    invitation: row.get(7)?,
                    starred: row.get(8)?,
                })
            },
        )?;
        for message in messages.values_mut().flatten() {
            message.attachments = attachments.remove(&message.id).unwrap_or_default();
        }
        let mut thread_folders = grouped(
            c,
            "SELECT thread_id, folder_id FROM thread_folders ORDER BY thread_id, folder_id",
            |row| row.get::<_, String>(1),
        )?;
        let mut people = grouped(
            c,
            "SELECT thread_id, name FROM thread_people ORDER BY thread_id, position",
            |row| row.get::<_, String>(1),
        )?;
        let mut replies = grouped(
            c,
            "SELECT thread_id, text FROM quick_replies ORDER BY thread_id, position",
            |row| row.get::<_, String>(1),
        )?;
        let threads = rows(
            c,
            "SELECT id, account_id, place, category, sender, subject, latest_at, note_text, note_kind, summary, unread, snoozed_until FROM threads ORDER BY latest_at DESC",
            |row| {
                Ok(Thread {
                    id: row.get(0)?,
                    account: row.get(1)?,
                    place: row.get(2)?,
                    category: row.get(3)?,
                    from: row.get(4)?,
                    subject: row.get(5)?,
                    latest_at: row.get(6)?,
                    note: match row.get(8)? {
                        Some(kind) => Some((row.get(7)?, kind)),
                        None => None,
                    },
                    summary: row.get(9)?,
                    unread: row.get(10)?,
                    snoozed_until: row.get(11)?,
                    folders: Vec::new(),
                    others: Vec::new(),
                    messages: Vec::new(),
                    quick_replies: Vec::new(),
                })
            },
        )?
        .into_iter()
        .map(|mut t| {
            t.folders = thread_folders.remove(&t.id).unwrap_or_default();
            t.others = people.remove(&t.id).unwrap_or_default();
            t.messages = messages.remove(&t.id).unwrap_or_default();
            t.quick_replies = replies.remove(&t.id).unwrap_or_default();
            t
        })
        .collect();
        let mut permissions = permissions(c, "agent_permissions", "p.agent_id")?;
        let mut times = grouped(
            c,
            "SELECT agent_id, minute FROM agent_times ORDER BY agent_id, minute",
            |row| row.get::<_, u32>(1),
        )?;
        let mut agent_accounts = grouped(
            c,
            "SELECT a.agent_id, a.account_id FROM agent_accounts a JOIN accounts b ON b.id = a.account_id ORDER BY a.agent_id, b.position",
            |row| row.get::<_, String>(1),
        )?;
        let mut stats = grouped(
            c,
            "SELECT agent_id, value, label FROM agent_stats ORDER BY agent_id, position",
            stat,
        )?;
        let mut result_stats = grouped(
            c,
            "SELECT agent_id, value, label FROM result_stats ORDER BY agent_id, position",
            stat,
        )?;
        let mut items = grouped(
            c,
            "SELECT agent_id, id, group_title, at, title, detail FROM result_items ORDER BY agent_id, group_position, position",
            |row| {
                Ok(ResultItem {
                    id: row.get(1)?,
                    group: row.get(2)?,
                    at: row.get(3)?,
                    title: row.get(4)?,
                    detail: row.get(5)?,
                    refs: Vec::new(),
                })
            },
        )?;
        for item in items.values_mut().flatten() {
            item.refs = item_refs.remove(&item.id).unwrap_or_default();
        }
        let mut steps = grouped(
            c,
            "SELECT run_id, position, verb, what, permission FROM run_steps ORDER BY run_id, position",
            |row| {
                Ok((
                    row.get::<_, i64>(1)?,
                    Step {
                        verb: row.get(2)?,
                        what: row.get(3)?,
                        permission: row.get(4)?,
                        refs: Vec::new(),
                    },
                ))
            },
        )?;
        let run_rows = grouped(
            c,
            "SELECT agent_id, id, started_at, cause, cause_text, seconds, tokens, outcome, outcome_text, summary FROM runs ORDER BY agent_id, started_at DESC",
            |row| {
                Ok(RunRow {
                    run: Run {
                        id: row.get(1)?,
                        started_at: row.get(2)?,
                        cause: Cause::RunNow,
                        seconds: row.get(5)?,
                        tokens: row.get(6)?,
                        outcome: match row.get::<_, OutcomeKind>(7)? {
                            OutcomeKind::Done => Outcome::Done,
                            OutcomeKind::Waiting => Outcome::Waiting,
                            OutcomeKind::Running => Outcome::Running,
                            OutcomeKind::Paused => Outcome::Paused,
                            OutcomeKind::Failed => Outcome::Failed(row.get(8)?),
                            OutcomeKind::Stopped => Outcome::Stopped(row.get(8)?),
                        },
                        summary: row.get(9)?,
                        steps: Vec::new(),
                    },
                    cause: row.get(3)?,
                    cause_text: row.get(4)?,
                })
            },
        )?;
        let mut runs: HashMap<String, Vec<Run>> = HashMap::new();
        for (agent, agent_runs) in run_rows {
            let mut built = Vec::new();
            for RunRow {
                mut run,
                cause: kind,
                cause_text,
            } in agent_runs
            {
                let refs = cause_refs.remove(&run.id).unwrap_or_default();
                run.cause = cause(&run.id, kind, cause_text, refs)?;
                run.steps = steps
                    .remove(&run.id)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(position, mut step)| {
                        step.refs = step_refs
                            .remove(&format!("{}/{position}", run.id))
                            .unwrap_or_default();
                        step
                    })
                    .collect();
                built.push(run);
            }
            runs.insert(agent, built);
        }
        let mut turns = grouped(
            c,
            "SELECT agent_id, id, sent_at, message, state, text, approval_id FROM turns ORDER BY agent_id, sent_at",
            |row| {
                let id: String = row.get(1)?;
                Ok(Turn {
                    sent_at: row.get(2)?,
                    message: row.get(3)?,
                    state: match row.get::<_, TurnKind>(4)? {
                        TurnKind::Running => TurnState::Running(row.get(5)?),
                        TurnKind::Waiting => TurnState::Waiting(row.get(6)?),
                        TurnKind::Answered => TurnState::Answered(
                            row.get(5)?,
                            turn_refs.remove(&id).unwrap_or_default(),
                        ),
                        TurnKind::Failed => TurnState::Failed(row.get(5)?),
                        TurnKind::Stopped => TurnState::Stopped(row.get(5)?),
                    },
                    id,
                })
            },
        )?;
        let mut triggers = grouped(
            c,
            "SELECT agent_id, trigger_id FROM agent_triggers ORDER BY agent_id, trigger_id",
            |row| row.get::<_, String>(1),
        )?;
        let mut filters = grouped(
            c,
            "SELECT agent_id, trigger_id, filter_id, choice_id FROM agent_filters ORDER BY agent_id, trigger_id, filter_id",
            |row| Ok((row.get(1)?, row.get(2)?, row.get(3)?)),
        )?;
        let agents = rows(
            c,
            "SELECT id, name, built_in, active, instructions, model_name, model_provider, schedule, schedule_note, weekdays_only, interval_seconds, woken_at, key_note, results_kind, has_chat FROM agents ORDER BY position",
            |row| {
                let id: String = row.get(0)?;
                Ok(Agent {
                    name: row.get(1)?,
                    built_in: row.get(2)?,
                    active: row.get(3)?,
                    instructions: row.get(4)?,
                    model: Model {
                        name: row.get(5)?,
                        provider: row.get(6)?,
                    },
                    schedule: match row.get::<_, ScheduleKind>(7)? {
                        ScheduleKind::Triggers => Schedule::Triggers,
                        ScheduleKind::BuiltIn => Schedule::BuiltIn(row.get(8)?),
                        ScheduleKind::Times => Schedule::Times {
                            weekdays: row.get(9)?,
                            minutes: times.remove(&id).unwrap_or_default(),
                        },
                        ScheduleKind::Interval => Schedule::Interval(row.get(10)?),
                        ScheduleKind::OnMessage => Schedule::OnMessage,
                    },
                    woken_at: row.get(11)?,
                    key_note: row.get(12)?,
                    results: row.get(13)?,
                    has_chat: row.get(14)?,
                    wakes: Wakes {
                        triggers: triggers.remove(&id).unwrap_or_default(),
                        filters: filters.remove(&id).unwrap_or_default(),
                    },
                    permissions: permissions.remove(&id).unwrap_or_default(),
                    accounts: agent_accounts.remove(&id).unwrap_or_default(),
                    stats: stats.remove(&id).unwrap_or_default(),
                    result_stats: result_stats.remove(&id).unwrap_or_default(),
                    result_items: items.remove(&id).unwrap_or_default(),
                    runs: runs.remove(&id).unwrap_or_default(),
                    turns: turns.remove(&id).unwrap_or_default(),
                    id,
                })
            },
        )?;
        let approvals = rows(
            c,
            "SELECT p.id, p.agent_id, a.name, p.created_at, p.title, p.detail, p.reason, p.action, p.thread_id, p.status, p.draft, p.declined_reason FROM approvals p JOIN agents a ON a.id = p.agent_id ORDER BY p.created_at DESC",
            |row| {
                Ok(Approval {
                    id: row.get(0)?,
                    agent: row.get(1)?,
                    agent_name: row.get(2)?,
                    created_at: row.get(3)?,
                    title: row.get(4)?,
                    detail: row.get(5)?,
                    reason: row.get(6)?,
                    action: match row.get::<_, ApprovalKind>(7)? {
                        ApprovalKind::Send => ApprovalAction::Send(row.get(8)?),
                        ApprovalKind::Unsubscribe => ApprovalAction::Unsubscribe,
                    },
                    status: match row.get::<_, StatusKind>(9)? {
                        StatusKind::Waiting => ApprovalStatus::Waiting,
                        StatusKind::Resolving => ApprovalStatus::Resolving,
                        StatusKind::Done => ApprovalStatus::Done,
                        StatusKind::Declined => ApprovalStatus::Declined(row.get(11)?),
                    },
                    draft: row.get(10)?,
                })
            },
        )?;
        let models = rows(
            c,
            "SELECT name, provider FROM models ORDER BY position",
            |row| {
                Ok(Model {
                    name: row.get(0)?,
                    provider: row.get(1)?,
                })
            },
        )?;
        let calendars = rows(
            c,
            "SELECT id, name, provider, provider_name, color, shown FROM calendars ORDER BY position",
            |row| {
                Ok(CalendarSource {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    provider: row.get(2)?,
                    provider_name: row.get(3)?,
                    color: row.get(4)?,
                    shown: row.get(5)?,
                })
            },
        )?;
        let mut guests = grouped(
            c,
            "SELECT event_id, name FROM event_guests ORDER BY event_id, position",
            |row| row.get::<_, String>(1),
        )?;
        let mut invitations: HashMap<String, Invitation> = rows(
            c,
            "SELECT event_id, invited_by, people, answer FROM invitations",
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    Invitation {
                        invited_by: row.get(1)?,
                        people: row.get(2)?,
                        answer: row.get(3)?,
                    },
                ))
            },
        )?
        .into_iter()
        .collect();
        let events = rows(
            c,
            "SELECT id, calendar_id, day, start_minute, end_minute, title, place FROM events ORDER BY day, start_minute",
            |row| {
                Ok((
                    row.get::<_, String>(2)?,
                    CalendarEvent {
                        id: row.get(0)?,
                        calendar: row.get(1)?,
                        date: Date {
                            year: 1970,
                            month: 1,
                            day: 1,
                        },
                        start: row.get(3)?,
                        end: row.get(4)?,
                        title: row.get(5)?,
                        place: row.get(6)?,
                        guests: Vec::new(),
                        invitation: None,
                    },
                ))
            },
        )?
        .into_iter()
        .map(|(day, mut e)| {
            e.date = Date::parse(&day).ok_or_else(|| broken("event", &e.id))?;
            e.guests = guests.remove(&e.id).unwrap_or_default();
            e.invitation = invitations.remove(&e.id);
            Ok(e)
        })
        .collect::<Result<_>>()?;
        let quarantine = rows(
            c,
            "SELECT id, sender, account_id, received_at, subject, reason, caught_by, why, outcome, body, drops_at FROM quarantine ORDER BY received_at DESC",
            |row| {
                Ok(Quarantined {
                    id: row.get(0)?,
                    sender: row.get(1)?,
                    account: row.get(2)?,
                    received_at: row.get(3)?,
                    subject: row.get(4)?,
                    reason: row.get(5)?,
                    caught_by: row.get(6)?,
                    why: row.get(7)?,
                    outcome: row.get(8)?,
                    body: row.get(9)?,
                    drops_at: row.get(10)?,
                })
            },
        )?;
        let mut lines = grouped(
            c,
            "SELECT edition_id, section, source, text FROM edition_lines ORDER BY edition_id, section, position",
            |row| {
                Ok((
                    row.get::<_, Section>(1)?,
                    Line {
                        source: row.get(2)?,
                        text: row.get(3)?,
                    },
                ))
            },
        )?;
        let mut waiting = grouped(
            c,
            "SELECT edition_id, approval_id, title, detail FROM edition_waiting ORDER BY edition_id, position",
            |row| Ok((row.get(1)?, row.get(2)?, row.get(3)?)),
        )?;
        let mut briefing_events = grouped(
            c,
            "SELECT edition_id, time, title, detail FROM edition_events ORDER BY edition_id, position",
            |row| Ok((row.get(1)?, row.get(2)?, row.get(3)?)),
        )?;
        let mut news = grouped(
            c,
            "SELECT edition_id, topic, headline, source FROM edition_news ORDER BY edition_id, position",
            |row| {
                Ok((
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            },
        )?;
        let editions = rows(
            c,
            "SELECT id, written_at, byline, caught_mail, calendar_title, unread FROM editions ORDER BY written_at DESC",
            |row| {
                Ok(Edition {
                    id: row.get(0)?,
                    written_at: row.get(1)?,
                    byline: row.get(2)?,
                    caught_mail: row.get(3)?,
                    calendar_title: row.get(4)?,
                    unread: row.get(5)?,
                    activity: Vec::new(),
                    waiting: Vec::new(),
                    newsletters: Vec::new(),
                    updates: Vec::new(),
                    calendar: Vec::new(),
                    news: Vec::new(),
                })
            },
        )?
        .into_iter()
        .map(|mut e| {
            for (section, line) in lines.remove(&e.id).unwrap_or_default() {
                match section {
                    Section::Activity => e.activity.push(line),
                    Section::Newsletters => e.newsletters.push(line),
                    Section::Updates => e.updates.push(line),
                }
            }
            e.waiting = waiting.remove(&e.id).unwrap_or_default();
            e.calendar = briefing_events.remove(&e.id).unwrap_or_default();
            for (topic, headline, source) in news.remove(&e.id).unwrap_or_default() {
                match e.news.last_mut() {
                    Some((last, items)) if *last == topic => items.push((headline, source)),
                    _ => e.news.push((topic, vec![(headline, source)])),
                }
            }
            e
        })
        .collect();
        let news_topics = rows(c, "SELECT name FROM news_topics ORDER BY position", |row| {
            row.get(0)
        })?;
        let mut draft_attachments = grouped(
            c,
            "SELECT draft_id, id, name, size_bytes FROM draft_attachments ORDER BY draft_id, position",
            |row| Ok((row.get(1)?, row.get(2)?, row.get(3)?)),
        )?;
        let drafts = rows(
            c,
            "SELECT d.id, d.kind, d.thread_id, d.account_id, d.recipients, d.subject, d.body, d.updated_at, o.state, o.due_at, o.failure, d.message_id, d.approval_id FROM drafts d LEFT JOIN outbox o ON o.draft_id = d.id ORDER BY d.updated_at DESC",
            |row| {
                let send = match row.get::<_, Option<SendKind>>(8)? {
                    None => None,
                    Some(SendKind::Undoable) => Some(SendState::Undoable(row.get(9)?)),
                    Some(SendKind::Scheduled) => Some(SendState::Scheduled(row.get(9)?)),
                    Some(SendKind::Sending) => Some(SendState::Sending),
                    Some(SendKind::Failed) => Some(SendState::Failed(row.get(10)?)),
                };
                Ok(Draft {
                    id: row.get(0)?,
                    kind: row.get(1)?,
                    thread: row.get(2)?,
                    message: row.get(11)?,
                    approval: row.get(12)?,
                    account: row.get(3)?,
                    recipients: row.get(4)?,
                    subject: row.get(5)?,
                    body: row.get(6)?,
                    updated_at: row.get(7)?,
                    attachments: Vec::new(),
                    send,
                })
            },
        )?
        .into_iter()
        .map(|mut d| {
            d.attachments = draft_attachments.remove(&d.id).unwrap_or_default();
            d
        })
        .collect();
        let settings = self.settings(now)?;
        let builder = self.builder()?;
        let search = match search {
            Some(query) => Some((query.to_string(), self.search(query)?)),
            None => None,
        };
        let snoozes = rows(
            c,
            "SELECT preset, until FROM snooze_times ORDER BY preset",
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        Ok(World {
            now,
            user,
            builder,
            accounts,
            folders,
            threads,
            agents,
            approvals,
            models,
            calendars,
            events,
            quarantine,
            editions,
            news_topics,
            drafts,
            settings,
            search,
            snoozes,
        })
    }

    fn rules(&self) -> Result<Vec<Rule>> {
        let c = self.connection();
        let mut words = grouped(
            c,
            "SELECT rule_id, word FROM rule_words ORDER BY rule_id, position",
            |row| row.get::<_, String>(1),
        )?;
        let mut actions = grouped(
            c,
            "SELECT rule_id, action, folder_id, agent_id FROM rule_actions ORDER BY rule_id, position",
            |row| {
                Ok(match row.get::<_, ActionKind>(1)? {
                    ActionKind::MoveTo => RuleAction::MoveTo(row.get(2)?),
                    ActionKind::Archive => RuleAction::Archive,
                    ActionKind::Trash => RuleAction::Trash,
                    ActionKind::MarkImportant => RuleAction::MarkImportant,
                    ActionKind::HandTo => RuleAction::HandTo(row.get(3)?),
                    ActionKind::KeepForBriefing => RuleAction::KeepForBriefing,
                })
            },
        )?;
        rows(
            c,
            "SELECT id, condition, parameter, enabled FROM rules ORDER BY position",
            |row| {
                let id: String = row.get(0)?;
                Ok(Rule {
                    condition: match row.get::<_, ConditionKind>(1)? {
                        ConditionKind::SenderContains => Condition::SenderContains(row.get(2)?),
                        ConditionKind::SenderDomain => Condition::SenderDomain(row.get(2)?),
                        ConditionKind::SubjectContains => {
                            Condition::SubjectContains(words.remove(&id).unwrap_or_default())
                        }
                        ConditionKind::Newsletter => Condition::Newsletter,
                        ConditionKind::NewSenderWithLink => Condition::NewSenderWithLink,
                    },
                    actions: actions.remove(&id).unwrap_or_default(),
                    enabled: row.get(3)?,
                    id,
                })
            },
        )
    }

    fn settings(&self, now: i64) -> Result<Settings> {
        let c = self.connection();
        let defaults = permissions(c, "default_permissions", "''")?
            .remove("")
            .unwrap_or_default();
        let blocked = rows(
            c,
            "SELECT address, blocked_by, blocked_at FROM blocked ORDER BY position",
            |row| {
                Ok(Blocked {
                    address: row.get(0)?,
                    by: row.get(1)?,
                    at: row.get(2)?,
                })
            },
        )?;
        let keys = rows(
            c,
            "SELECT provider, name, is_set FROM model_keys ORDER BY provider",
            |row| {
                Ok(Key {
                    provider: row.get(0)?,
                    name: row.get(1)?,
                    set: row.get(2)?,
                })
            },
        )?;
        let tasks = rows(
            c,
            "SELECT id, label, hint, model_name FROM task_models ORDER BY position",
            |row| {
                Ok(TaskModel {
                    id: row.get(0)?,
                    label: row.get(1)?,
                    hint: row.get(2)?,
                    model: row.get(3)?,
                })
            },
        )?;
        let today = crate::time::start_of_day(now);
        let used_tokens: i64 = c.query_row(
            "SELECT coalesce(sum(tokens), 0) FROM runs WHERE started_at >= ?1",
            [today],
            |row| row.get(0),
        )?;
        let (notify_when_waiting, agents_paused, budget_tokens, compose_tone, ai) = c.query_row(
            "SELECT notify_when_waiting, agents_paused, budget_tokens, compose_tone, ai FROM settings",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        )?;
        let (quarantine_days, dry_run_messages) = c.query_row(
            "SELECT quarantine_days, dry_run_messages FROM settings",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        Ok(Settings {
            defaults,
            notify_when_waiting,
            agents_paused,
            rules: self.rules()?,
            blocked,
            keys,
            tasks,
            budget_tokens,
            used_tokens,
            compose_tone,
            ai,
            quarantine_days,
            dry_run_messages,
        })
    }

    fn builder(&self) -> Result<Builder> {
        let c = self.connection();
        let (name, instructions, schedule, interval_seconds, ask_before_new_recipient): (
            String,
            String,
            String,
            u32,
            bool,
        ) = c.query_row(
            "SELECT name, instructions, schedule, interval_seconds, ask_before_new_recipient FROM builder",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        )?;
        Ok(Builder {
            name,
            instructions,
            schedule: builder::schedule_option(&schedule).ok_or_else(|| {
                StoreError(format!("the builder's schedule {schedule} is unknown"))
            })?,
            interval_seconds,
            ask_before_new_recipient,
            wakes: Wakes {
                triggers: rows(
                    c,
                    "SELECT trigger_id FROM builder_triggers WHERE chosen ORDER BY trigger_id",
                    |row| row.get(0),
                )?,
                filters: rows(
                    c,
                    "SELECT trigger_id, filter_id, choice_id FROM builder_filters ORDER BY trigger_id, filter_id",
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )?,
            },
            accounts: rows(
                c,
                "SELECT b.account_id FROM builder_accounts b JOIN accounts a ON a.id = b.account_id ORDER BY a.position",
                |row| row.get(0),
            )?,
            permissions: permissions(c, "builder_permissions", "''")?
                .remove("")
                .unwrap_or_default(),
        })
    }

    /// The threads whose messages match `query`, best match first.
    pub fn search(&self, query: &str) -> Result<Vec<String>> {
        let terms: Vec<String> = query
            .split_whitespace()
            .map(|term| format!("\"{}\"*", term.replace('"', "\"\"")))
            .collect();
        if terms.is_empty() {
            return Ok(Vec::new());
        }
        let mut statement = self.connection().prepare(
            "SELECT thread_id FROM mail_search WHERE mail_search MATCH ?1 GROUP BY thread_id ORDER BY min(rank)",
        )?;
        Ok(statement
            .query_map([terms.join(" ")], |row| row.get(0))?
            .collect::<rusqlite::Result<_>>()?)
    }

    pub fn preferences(&self) -> Result<Option<Preferences>> {
        Ok(rows(
            self.connection(),
            "SELECT theme, density, sidebar FROM preferences",
            |row| {
                Ok(Preferences {
                    theme: row.get(0)?,
                    density: row.get(1)?,
                    sidebar: row.get(2)?,
                })
            },
        )?
        .into_iter()
        .next())
    }

    pub fn navigation_path(&self) -> Result<Option<String>> {
        Ok(
            rows(self.connection(), "SELECT path FROM navigation", |row| {
                row.get(0)
            })?
            .into_iter()
            .next(),
        )
    }
}
