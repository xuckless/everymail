//! What wakes an agent on its own, as data, and every label said of it: the agents list, the
//! agent's page and settings, the builder and a scheduled run's cause all read their text here.

use crate::time;

/// The shortest interval the Backend allows: a proactive agent wakes at most every 15 minutes. A new
/// agent's custom interval starts here.
pub const MIN_INTERVAL_SECONDS: u32 = 900;
pub const INTERVAL_HINT: &str = "At least every 15 minutes";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Schedule {
    /// Only its triggers wake it.
    Triggers,
    /// The Backend's order on new mail reaches it, as the Backend says: "On every new mail, after
    /// your rules". Gatekeeper, Security Guard and Scribe.
    BuiltIn(String),
    /// At these minutes after midnight, in order: every day, or on weekdays only.
    Times { weekdays: bool, minutes: Vec<u32> },
    /// Every this many seconds.
    Interval(u32),
    /// Only when the user writes to it.
    OnMessage,
}

/// "Every 20 minutes", "Every hour", "Every 2 hours"
pub fn interval_label(seconds: u32) -> String {
    match seconds / 60 {
        60 => "Every hour".into(),
        m if m % 60 == 0 => format!("Every {} hours", m / 60),
        m => format!("Every {m} minutes"),
    }
}

/// "Daily at", "Weekdays at"
pub fn times_prefix(weekdays: bool) -> &'static str {
    if weekdays { "Weekdays at" } else { "Daily at" }
}

/// "a", "a and b", "a, b and c"
fn listed(items: &[String]) -> String {
    match items.split_last() {
        Some((last, rest)) if !rest.is_empty() => format!("{} and {last}", rest.join(", ")),
        Some((last, _)) => last.clone(),
        None => String::new(),
    }
}

/// The first letter in capitals.
pub fn capitalized(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map_or(String::new(), |c| c.to_uppercase().chain(chars).collect())
}

fn lowered(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map_or(String::new(), |c| c.to_lowercase().chain(chars).collect())
}

impl Schedule {
    /// The schedule alone: "Daily at 7:30 and 18:00", "Every hour"; `None` for triggers only.
    pub fn label(&self) -> Option<String> {
        match self {
            Schedule::Triggers => None,
            Schedule::BuiltIn(text) => Some(text.clone()),
            Schedule::Times { weekdays, minutes } => {
                let times: Vec<String> = minutes.iter().map(|m| time::clock(*m)).collect();
                Some(format!("{} {}", times_prefix(*weekdays), listed(&times)))
            }
            Schedule::Interval(seconds) => Some(interval_label(*seconds)),
            Schedule::OnMessage => Some("Only when you write to it".into()),
        }
    }

    /// The cause of a scheduled run at `slot`: "Weekdays at 9:00", "Every hour".
    pub fn slot_label(&self, slot: i64) -> String {
        match self {
            Schedule::Times { weekdays, .. } => {
                format!(
                    "{} {}",
                    times_prefix(*weekdays),
                    time::clock(time::minute(slot))
                )
            }
            Schedule::Triggers
            | Schedule::BuiltIn(_)
            | Schedule::Interval(_)
            | Schedule::OnMessage => self.label().unwrap_or_default(),
        }
    }

    /// When it wakes next: the first slot after it last woke. A slot already past is due now,
    /// once, however many were missed.
    pub fn next_wake(&self, woken_at: i64) -> Option<i64> {
        match self {
            Schedule::Interval(seconds) => Some(woken_at + i64::from(*seconds)),
            Schedule::Times { weekdays, minutes } => {
                let first = time::date(woken_at);
                (0..8)
                    .map(|d| first.add_days(d))
                    .filter(|day| !weekdays || day.weekday() < 5)
                    .flat_map(|day| minutes.iter().map(move |m| time::at(day, *m)))
                    .find(|at| *at > woken_at)
            }
            Schedule::Triggers | Schedule::BuiltIn(_) | Schedule::OnMessage => None,
        }
    }
}

/// Everything that wakes it in one line: "Mail sorted into a category or folder, every hour",
/// "Weekdays at 9:00". `triggers` are the chosen triggers' labels, in order.
pub fn wakes(triggers: &[&str], schedule: &Schedule) -> String {
    let mut parts: Vec<String> = triggers.iter().map(|t| (*t).to_string()).collect();
    if let Some(label) = schedule.label() {
        parts.push(if parts.is_empty() {
            label
        } else {
            lowered(&label)
        });
    }
    if parts.is_empty() {
        "No schedule".into()
    } else {
        parts.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::date::Date;

    fn at(day: &str, minute: u32) -> i64 {
        time::at(Date::parse(day).expect(day), minute)
    }

    #[test]
    fn labels_come_from_the_schedule() {
        let daily = Schedule::Times {
            weekdays: false,
            minutes: vec![450, 1080],
        };
        assert_eq!(daily.label().as_deref(), Some("Daily at 7:30 and 18:00"));
        assert_eq!(daily.slot_label(at("2026-11-11", 1080)), "Daily at 18:00");
        assert_eq!(interval_label(900), "Every 15 minutes");
        assert_eq!(interval_label(3600), "Every hour");
        assert_eq!(interval_label(7200), "Every 2 hours");
        assert_eq!(
            wakes(
                &["Mail sorted into a category or folder"],
                &Schedule::Interval(3600)
            ),
            "Mail sorted into a category or folder, every hour"
        );
        assert_eq!(wakes(&[], &Schedule::Triggers), "No schedule");
        assert_eq!(wakes(&[], &Schedule::Interval(1200)), "Every 20 minutes");
    }

    #[test]
    fn the_next_wake_is_the_first_slot_after_the_last() {
        let weekdays = Schedule::Times {
            weekdays: true,
            minutes: vec![540],
        };
        // Wednesday 9:50: Thursday 9:00. Friday 9:00: Monday 9:00.
        assert_eq!(
            weekdays.next_wake(at("2026-11-11", 590)),
            Some(at("2026-11-12", 540))
        );
        assert_eq!(
            weekdays.next_wake(at("2026-11-13", 540)),
            Some(at("2026-11-16", 540))
        );
        // Days missed come due once.
        assert_eq!(
            weekdays.next_wake(at("2026-11-02", 600)),
            Some(at("2026-11-03", 540))
        );
        assert_eq!(Schedule::Interval(900).next_wake(100), Some(1000));
        assert_eq!(Schedule::Triggers.next_wake(100), None);
        assert_eq!(Schedule::BuiltIn("x".into()).next_wake(100), None);
    }
}
