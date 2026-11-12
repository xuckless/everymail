//! The Backend's answers to changes the user asks for, as pure functions: when snoozes end and
//! sends go, which permission levels an agent may have, and the limits on topics, intervals, keys
//! and budgets.

use crate::{
    backend::{SendAt, Update},
    live::Target,
    proto::ui::v1::{Form, MailProvider, ModelProvider, PermissionLevel},
    schedule::{INTERVAL_HINT, MIN_INTERVAL_SECONDS},
    time::{self, DAY, HOUR, MINUTE},
    world::{Ai, Key, Levels, Owner, Permission, SendState, Snooze, World},
};

/// The undo window of "send now".
pub const UNDO_SEND_SECONDS: i64 = 10;
/// News Bearer follows at most this many topics.
pub const TOPIC_CAP: usize = 10;
pub const SEND_ACTION: &str = "send";
const GMAIL_SEND: &str = "Can’t be Auto on Gmail yet, Google is still reviewing EveryMail";

/// The presets, in menu order.
pub const SNOOZES: [Snooze; 4] = [
    Snooze::LaterToday,
    Snooze::Tomorrow,
    Snooze::ThisWeekend,
    Snooze::NextWeek,
];

/// When Quarantine drops the copy of mail that came in at `at`: the days the replica's settings
/// hold, which the Backend sets.
pub fn quarantine_ends(at: i64, world: &World) -> i64 {
    at + world.settings.quarantine_days * DAY
}

/// When a snooze preset ends: later today at 18:00 (three hours from now once past 15:00),
/// tomorrow at 8:00, Saturday at 9:00, Monday at 8:00.
pub fn snooze_until(preset: Snooze, now: i64) -> i64 {
    let today = time::date(now);
    let weekday = today.weekday() as i64;
    match preset {
        Snooze::LaterToday if time::minute(now) >= 15 * 60 => now + 3 * HOUR,
        Snooze::LaterToday => time::at(today, 18 * 60),
        Snooze::Tomorrow => time::at(today.add_days(1), 8 * 60),
        Snooze::ThisWeekend => {
            let days = match (5 - weekday).rem_euclid(7) {
                0 => 7,
                d => d,
            };
            time::at(today.add_days(days), 9 * 60)
        }
        Snooze::NextWeek => time::at(today.add_days(7 - weekday), 8 * 60),
    }
}

/// When every preset ends, as of now.
pub fn snooze_times(now: i64) -> Vec<(Snooze, i64)> {
    SNOOZES
        .iter()
        .map(|p| (*p, snooze_until(*p, now)))
        .collect()
}

/// When the presets' times change next: at 15:00, then every minute until midnight.
pub fn snooze_times_change(now: i64) -> i64 {
    if time::minute(now) >= 15 * 60 {
        now - now.rem_euclid(MINUTE) + MINUTE
    } else {
        time::at(time::date(now), 15 * 60)
    }
}

/// What the Backend holds a send as, or why it will not.
pub fn send(at: &SendAt, now: i64) -> Result<SendState, &'static str> {
    match at {
        SendAt::Now => Ok(SendState::Undoable(now + UNDO_SEND_SECONDS)),
        SendAt::Later(day, minute) => {
            let at = time::at(*day, *minute);
            if at > now {
                Ok(SendState::Scheduled(at))
            } else {
                Err("Pick a time later than now")
            }
        }
    }
}

/// The levels an action may take on accounts of these providers.
pub fn levels(action: &str, providers: &[MailProvider]) -> Levels {
    if action == SEND_ACTION && providers.contains(&MailProvider::Gmail) {
        Levels::AskAtMost(GMAIL_SEND.into())
    } else {
        Levels::All
    }
}

/// Where a refusal about an owner's permissions shows.
fn target(owner: &Owner) -> Target {
    match owner {
        Owner::Agent(id) => Target::Agent(id.clone()),
        Owner::Defaults | Owner::Builder => Target::Form(Form::AgentBuilder),
    }
}

fn refused(target: Target, reason: &str) -> Update {
    Update::Refused {
        target,
        reason: reason.into(),
    }
}

/// A permission change: the stored result, or why it is refused.
pub fn permission(owner: Owner, action: &str, level: PermissionLevel, world: &World) -> Update {
    let Some((accounts, permissions)) = world.grants(&owner) else {
        return refused(target(&owner), "That agent is gone");
    };
    if !permissions.iter().any(|p| p.id == action) {
        return refused(target(&owner), "It does not take that action");
    }
    let levels = levels(action, &world.providers(accounts));
    match &levels {
        Levels::AskAtMost(reason) if level == PermissionLevel::Auto => {
            refused(target(&owner), reason)
        }
        Levels::AskAtMost(_) | Levels::All => Update::Permission {
            owner,
            action: action.into(),
            level,
            levels,
        },
    }
}

/// The accounts an agent or the builder sees, then each permission whose levels change: Auto is
/// lowered to Ask where it is no longer offered.
pub fn accounts(owner: Owner, accounts: Vec<String>, world: &World) -> Vec<Update> {
    let Some((_, permissions)) = world.grants(&owner) else {
        return vec![refused(target(&owner), "That agent is gone")];
    };
    let accounts: Vec<String> = world
        .accounts
        .iter()
        .filter(|a| accounts.contains(&a.id))
        .map(|a| a.id.clone())
        .collect();
    let providers = world.providers(&accounts);
    let changed: Vec<Update> = permissions
        .iter()
        .filter_map(|p: &Permission| {
            let levels = levels(&p.id, &providers);
            let level = match levels {
                Levels::AskAtMost(_) if p.level == PermissionLevel::Auto => PermissionLevel::Ask,
                Levels::AskAtMost(_) | Levels::All => p.level,
            };
            (levels != p.levels || level != p.level).then(|| Update::Permission {
                owner: owner.clone(),
                action: p.id.clone(),
                level,
                levels,
            })
        })
        .collect();
    std::iter::once(Update::Accounts { owner, accounts })
        .chain(changed)
        .collect()
}

/// A topic to follow, or why not.
pub fn topic(topic: &str, topics: &[String]) -> Update {
    let form = || Target::Form(Form::NewsTopic);
    if topics.iter().any(|t| t.eq_ignore_ascii_case(topic)) {
        refused(form(), "You follow it already")
    } else if topics.len() >= TOPIC_CAP {
        refused(
            form(),
            &format!("Up to {TOPIC_CAP} topics. Remove one first"),
        )
    } else {
        Update::TopicAdded(topic.into())
    }
}

pub fn interval(seconds: u32) -> Update {
    if seconds < MIN_INTERVAL_SECONDS {
        refused(Target::Form(Form::BuilderInterval), INTERVAL_HINT)
    } else {
        Update::Interval(seconds)
    }
}

/// Whether model work runs once these keys are set.
fn ai(keys: impl Iterator<Item = bool>) -> Ai {
    if keys.into_iter().any(|set| set) {
        Ai::On
    } else {
        Ai::NoKey
    }
}

/// A key set or removed; a set key must look like one.
pub fn key(provider: ModelProvider, key: Option<&str>, keys: &[Key]) -> Update {
    if key.is_some_and(|k| k.chars().count() < 20) {
        return refused(Target::Key(provider), "That is too short to be an API key");
    }
    let set = key.is_some();
    Update::Key {
        provider,
        set,
        ai: ai(keys
            .iter()
            .map(|k| if k.provider == provider { set } else { k.set })),
    }
}

pub fn budget(tokens: Option<u64>) -> Update {
    match tokens {
        Some(t) if t < 1_000 => refused(Target::Form(Form::Budget), "At least 1,000 tokens a day"),
        Some(t) => Update::Budget(Some(i64::try_from(t).unwrap_or(i64::MAX))),
        None => Update::Budget(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{date::Date, demo::tests::world};

    /// 2026-11-11 is a Wednesday.
    fn at(day: &str, minute: u32) -> i64 {
        time::at(Date::parse(day).expect(day), minute)
    }

    #[test]
    fn snooze_presets() {
        let morning = at("2026-11-11", 9 * 60 + 50);
        assert_eq!(
            snooze_times(morning),
            vec![
                (Snooze::LaterToday, at("2026-11-11", 18 * 60)),
                (Snooze::Tomorrow, at("2026-11-12", 8 * 60)),
                (Snooze::ThisWeekend, at("2026-11-14", 9 * 60)),
                (Snooze::NextWeek, at("2026-11-16", 8 * 60)),
            ]
        );
        let afternoon = at("2026-11-11", 15 * 60 + 30);
        assert_eq!(
            snooze_until(Snooze::LaterToday, afternoon),
            afternoon + 3 * HOUR,
            "past 15:00, later today is three hours on"
        );
        assert_eq!(
            snooze_until(Snooze::ThisWeekend, at("2026-11-14", 10 * 60)),
            at("2026-11-21", 9 * 60),
            "on the weekend, the next one"
        );
        assert_eq!(
            snooze_until(Snooze::NextWeek, at("2026-11-15", 10 * 60)),
            at("2026-11-16", 8 * 60)
        );
        assert_eq!(snooze_times_change(morning), at("2026-11-11", 15 * 60));
        assert_eq!(
            snooze_times_change(afternoon + 7),
            afternoon + MINUTE,
            "after 15:00 the times move every minute"
        );
    }

    #[test]
    fn sends_wait_out_the_undo_window_or_their_time() {
        let now = at("2026-11-11", 9 * 60 + 50);
        let day = Date::parse("2026-11-12").unwrap();
        assert_eq!(
            send(&SendAt::Now, now),
            Ok(SendState::Undoable(now + UNDO_SEND_SECONDS))
        );
        assert_eq!(
            send(&SendAt::Later(day, 8 * 60), now),
            Ok(SendState::Scheduled(at("2026-11-12", 8 * 60)))
        );
        assert_eq!(
            send(
                &SendAt::Later(Date::parse("2026-11-11").unwrap(), 9 * 60),
                now
            ),
            Err("Pick a time later than now")
        );
    }

    #[test]
    fn send_on_auto_is_refused_and_lowered_where_an_agent_sees_gmail() {
        let (store, w) = world();
        // Job application tracker sees the Gmail account.
        let tracker = || Owner::Agent("job-tracker".into());
        assert_eq!(
            permission(tracker(), "send", PermissionLevel::Auto, &w),
            Update::Refused {
                target: Target::Agent("job-tracker".into()),
                reason: GMAIL_SEND.into(),
            }
        );
        assert_eq!(
            permission(tracker(), "send", PermissionLevel::Off, &w),
            Update::Permission {
                owner: tracker(),
                action: "send".into(),
                level: PermissionLevel::Off,
                levels: Levels::AskAtMost(GMAIL_SEND.into()),
            }
        );
        // Without the Gmail account Auto is offered again; with it, Auto drops to Ask.
        let laurentian = accounts(tracker(), vec!["laurentian".into()], &w);
        assert_eq!(
            laurentian,
            vec![
                Update::Accounts {
                    owner: tracker(),
                    accounts: vec!["laurentian".into()],
                },
                Update::Permission {
                    owner: tracker(),
                    action: "send".into(),
                    level: PermissionLevel::Ask,
                    levels: Levels::All,
                },
            ]
        );
        for update in &laurentian {
            store.apply(update, w.now).unwrap();
        }
        let without = store.load(w.now, None).unwrap();
        store
            .apply(
                &permission(tracker(), "send", PermissionLevel::Auto, &without),
                w.now,
            )
            .unwrap();
        let after = store.load(w.now, None).unwrap();
        assert_eq!(
            after.agent("job-tracker").unwrap().permissions[3].level,
            PermissionLevel::Auto
        );
        assert_eq!(
            accounts(
                tracker(),
                vec!["personal".into(), "laurentian".into()],
                &after
            ),
            vec![
                Update::Accounts {
                    owner: tracker(),
                    accounts: vec!["personal".into(), "laurentian".into()],
                },
                Update::Permission {
                    owner: tracker(),
                    action: "send".into(),
                    level: PermissionLevel::Ask,
                    levels: Levels::AskAtMost(GMAIL_SEND.into()),
                },
            ]
        );
        assert_eq!(
            permission(Owner::Defaults, "send", PermissionLevel::Auto, &w),
            Update::Permission {
                owner: Owner::Defaults,
                action: "send".into(),
                level: PermissionLevel::Auto,
                levels: Levels::All,
            },
            "the defaults see no account"
        );
    }

    #[test]
    fn limits() {
        let topics: Vec<String> = (0..10).map(|i| format!("t{i}")).collect();
        let form = Target::Form(Form::NewsTopic);
        assert_eq!(
            topic("T1", &topics),
            refused(form.clone(), "You follow it already")
        );
        assert_eq!(
            topic("new", &topics),
            refused(form, "Up to 10 topics. Remove one first")
        );
        assert_eq!(topic("new", &topics[..9]), Update::TopicAdded("new".into()));
        assert_eq!(
            interval(899),
            refused(Target::Form(Form::BuilderInterval), INTERVAL_HINT)
        );
        assert_eq!(interval(900), Update::Interval(900));
        assert_eq!(
            budget(Some(999)),
            refused(Target::Form(Form::Budget), "At least 1,000 tokens a day")
        );
        let keys = [Key {
            provider: ModelProvider::Openai,
            name: "OpenAI".into(),
            set: true,
        }];
        assert_eq!(
            key(ModelProvider::Openai, Some("short"), &keys),
            refused(
                Target::Key(ModelProvider::Openai),
                "That is too short to be an API key"
            )
        );
        assert_eq!(
            key(ModelProvider::Openai, None, &keys),
            Update::Key {
                provider: ModelProvider::Openai,
                set: false,
                ai: Ai::NoKey,
            },
            "the last key removed turns model work off"
        );
    }
}
