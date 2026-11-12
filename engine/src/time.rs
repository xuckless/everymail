//! Instants as the user reads them. An instant is Unix seconds, read in UTC until the user's zone
//! is a setting.

use crate::date::{Date, WEEKDAYS};

pub const MINUTE: i64 = 60;
pub const HOUR: i64 = 3600;
pub const DAY: i64 = 86_400;

pub fn date(at: i64) -> Date {
    Date::from_days_since_epoch(at.div_euclid(DAY))
}

/// Minutes after midnight.
pub fn minute(at: i64) -> u32 {
    (at.rem_euclid(DAY) / MINUTE) as u32
}

/// The instant the day of `at` starts.
pub fn start_of_day(at: i64) -> i64 {
    at - at.rem_euclid(DAY)
}

pub fn at(day: Date, minute: u32) -> i64 {
    day.days_since_epoch() * DAY + i64::from(minute) * MINUTE
}

/// "9:00", "19:30"
pub fn clock(minute: u32) -> String {
    format!("{}:{:02}", minute / 60, minute % 60)
}

/// "Nov 3"
pub fn day_month(day: Date) -> String {
    format!("{} {}", day.month_short(), day.day)
}

/// Days from `at` to `now`, by calendar day.
fn days_ago(at: i64, now: i64) -> i64 {
    date(now).days_since_epoch() - date(at).days_since_epoch()
}

/// For lists: "9:41" today, "Yesterday", "Sat" within the week, "Nov 3" before.
pub fn short(at: i64, now: i64) -> String {
    let day = date(at);
    match days_ago(at, now) {
        0 => clock(minute(at)),
        1 => "Yesterday".into(),
        2..=6 => WEEKDAYS[day.weekday()].into(),
        _ => day_month(day),
    }
}

/// With the time, past or coming: "8:50" today, "Mon 16:12" or "Thu 8:00" within a week either
/// way, "Nov 3 11:20" further.
pub fn long(at: i64, now: i64) -> String {
    let day = date(at);
    let time = clock(minute(at));
    match days_ago(at, now) {
        0 => time,
        -6..=6 => format!("{} {time}", WEEKDAYS[day.weekday()]),
        _ => format!("{} {time}", day_month(day)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn instant(day: &str, minute: u32) -> i64 {
        at(Date::parse(day).expect(day), minute)
    }

    #[test]
    fn labels_relative_to_now() {
        let now = instant("2026-11-11", 9 * 60 + 50);
        let cases = [
            (instant("2026-11-11", 9 * 60 + 41), "9:41", "9:41"),
            (instant("2026-11-10", 18 * 60), "Yesterday", "Tue 18:00"),
            (instant("2026-11-07", 11 * 60 + 2), "Sat", "Sat 11:02"),
            (instant("2026-11-03", 11 * 60 + 20), "Nov 3", "Nov 3 11:20"),
            (
                instant("2025-12-31", 23 * 60 + 59),
                "Dec 31",
                "Dec 31 23:59",
            ),
        ];
        for (at, short_label, long_label) in cases {
            assert_eq!(
                (short(at, now), long(at, now)),
                (short_label.into(), long_label.into())
            );
        }
        assert_eq!(long(instant("2026-11-12", 8 * 60), now), "Thu 8:00");
        assert_eq!(long(instant("2026-11-21", 9 * 60), now), "Nov 21 9:00");
        assert_eq!(
            start_of_day(instant("2026-11-11", 9 * 60 + 50)),
            instant("2026-11-11", 0)
        );
    }
}
