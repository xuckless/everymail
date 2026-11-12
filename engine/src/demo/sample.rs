//! The sample world the stand-in Backend syncs: written as of Wednesday, November 11 2026, and
//! moved by whole days so its today is the device's today.

use crate::{date::Date, time};

/// The day the sample was written as of.
const SAMPLE_DAY: Date = Date {
    year: 2026,
    month: 11,
    day: 11,
};
const SEED: &str = include_str!("seed.sql");
/// Instants in the seed lie in this range; nothing else there is a number this large.
const INSTANTS: std::ops::Range<i64> = 1_700_000_000..1_900_000_000;

/// The snapshot as of `now`.
pub fn snapshot(now: i64) -> String {
    shifted(
        SEED,
        time::date(now).days_since_epoch() - SAMPLE_DAY.days_since_epoch(),
    )
}

/// The seed with its instants and its quoted days moved by `days`.
pub fn shifted(seed: &str, days: i64) -> String {
    let mut out = String::with_capacity(seed.len());
    let mut rest = seed;
    while let Some(start) = rest.find(|c: char| c.is_ascii_digit()) {
        out.push_str(&rest[..start]);
        rest = &rest[start..];
        let end = rest
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(rest.len());
        let digits = &rest[..end];
        // "2026-11-11" in quotes: a day.
        let day = (end == 4 && out.ends_with('\''))
            .then(|| rest.get(..11).filter(|d| d.ends_with('\'')))
            .flatten()
            .and_then(|d| Date::parse(&d[..10]));
        match (day, digits.parse::<i64>()) {
            (Some(day), _) => {
                out.push_str(&day.add_days(days).id());
                rest = &rest[10..];
            }
            (None, Ok(n)) if INSTANTS.contains(&n) => {
                out.push_str(&(n + days * time::DAY).to_string());
                rest = &rest[end..];
            }
            (None, _) => {
                out.push_str(digits);
                rest = &rest[end..];
            }
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sample_moves_by_whole_days() {
        let seed = "VALUES ('e', '2026-11-11', 540, 1794390540, 'Nov 11', 'x12');";
        assert_eq!(
            shifted(seed, 2),
            "VALUES ('e', '2026-11-13', 540, 1794563340, 'Nov 11', 'x12');"
        );
        assert_eq!(shifted(seed, 0), seed);
        let next_day = time::at(Date::parse("2026-11-12").unwrap(), 23 * 60);
        assert!(
            snapshot(next_day).contains("'2026-11-13'"),
            "Thursday's exam"
        );
        assert!(!snapshot(next_day).contains("1794390540"));
    }
}
