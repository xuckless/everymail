//! Calendar dates without a clock: what the calendar's periods need, in the proleptic Gregorian
//! calendar. Weeks start on Monday.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    pub year: i32,
    /// 1 to 12.
    pub month: u32,
    /// 1 to the month's length.
    pub day: u32,
}

pub const WEEKDAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

fn leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        2 if leap(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

impl Date {
    /// `None` for a day that does not exist.
    pub fn new(year: i32, month: u32, day: u32) -> Option<Self> {
        ((1..=12).contains(&month) && (1..=days_in_month(year, month)).contains(&day))
            .then_some(Self { year, month, day })
    }

    /// "2026-11-11"
    pub fn parse(text: &str) -> Option<Self> {
        let mut parts = text.split('-');
        let (year, month, day) = (parts.next()?, parts.next()?, parts.next()?);
        if parts.next().is_some() || year.len() != 4 || month.len() != 2 || day.len() != 2 {
            return None;
        }
        Self::new(year.parse().ok()?, month.parse().ok()?, day.parse().ok()?)
    }

    /// "2026-12": the first day of that month.
    pub fn parse_month(text: &str) -> Option<Self> {
        let (year, month) = text.split_once('-')?;
        if year.len() != 4 || month.len() != 2 {
            return None;
        }
        Self::new(year.parse().ok()?, month.parse().ok()?, 1)
    }

    /// "2026-11-11"
    pub fn id(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    /// "2026-11"
    pub fn month_id(self) -> String {
        format!("{:04}-{:02}", self.year, self.month)
    }

    /// "November"
    pub fn month_name(self) -> &'static str {
        MONTHS[self.month as usize - 1]
    }

    /// "Nov"
    pub fn month_short(self) -> &'static str {
        &self.month_name()[..3]
    }

    /// Days since 1970-01-01, negative before.
    pub fn days_since_epoch(self) -> i64 {
        // Howard Hinnant's days_from_civil.
        let year = i64::from(self.year) - i64::from(self.month <= 2);
        let era = year.div_euclid(400);
        let year_of_era = year - era * 400;
        let month = i64::from(self.month);
        let day_of_year =
            (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + i64::from(self.day) - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        era * 146_097 + day_of_era - 719_468
    }

    pub fn from_days_since_epoch(days: i64) -> Self {
        // Howard Hinnant's civil_from_days.
        let days = days + 719_468;
        let era = days.div_euclid(146_097);
        let day_of_era = days - era * 146_097;
        let year_of_era =
            (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let mp = (5 * day_of_year + 2) / 153;
        let day = day_of_year - (153 * mp + 2) / 5 + 1;
        let month = if mp < 10 { mp + 3 } else { mp - 9 };
        let year = year_of_era + era * 400 + i64::from(month <= 2);
        Self {
            year: i32::try_from(year).unwrap_or(i32::MAX),
            month: u32::try_from(month).unwrap_or(1),
            day: u32::try_from(day).unwrap_or(1),
        }
    }

    pub fn add_days(self, days: i64) -> Self {
        Self::from_days_since_epoch(self.days_since_epoch() + days)
    }

    /// The same day `months` later, or the month's last day when it is shorter.
    pub fn add_months(self, months: i32) -> Self {
        let index = self.year * 12 + self.month as i32 - 1 + months;
        let (year, month) = (index.div_euclid(12), index.rem_euclid(12) as u32 + 1);
        Self {
            year,
            month,
            day: self.day.min(days_in_month(year, month)),
        }
    }

    /// 0 for Monday to 6 for Sunday.
    pub fn weekday(self) -> usize {
        // 1970-01-01 was a Thursday.
        (self.days_since_epoch() + 3).rem_euclid(7) as usize
    }

    /// The Monday of its week.
    pub fn week_start(self) -> Self {
        self.add_days(-(self.weekday() as i64))
    }

    pub fn first_of_month(self) -> Self {
        Self { day: 1, ..self }
    }

    /// The weeks, Monday to Sunday, that hold this date's month.
    pub fn month_weeks(self) -> Vec<[Self; 7]> {
        let first = self.first_of_month();
        let last = Self {
            day: days_in_month(self.year, self.month),
            ..self
        };
        let mut weeks = Vec::new();
        let mut monday = first.week_start();
        while monday <= last {
            weeks.push(std::array::from_fn(|i| monday.add_days(i as i64)));
            monday = monday.add_days(7);
        }
        weeks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(text: &str) -> Date {
        Date::parse(text).expect(text)
    }

    #[test]
    fn parses_and_writes_dates() {
        for text in ["2026-11-11", "2024-02-29", "1999-12-31", "2000-01-01"] {
            assert_eq!(date(text).id(), text);
        }
        for text in [
            "2026-02-29",
            "2026-13-01",
            "2026-00-10",
            "2026-11-31",
            "2026-1-11",
            "26-11-11",
            "2026-11-11-1",
            "",
        ] {
            assert_eq!(Date::parse(text), None, "{text}");
        }
        assert_eq!(Date::parse_month("2026-12"), Some(date("2026-12-01")));
        assert_eq!(Date::parse_month("2026-13"), None);
    }

    #[test]
    fn weekdays() {
        assert_eq!(WEEKDAYS[date("2026-11-11").weekday()], "Wed");
        assert_eq!(WEEKDAYS[date("1970-01-01").weekday()], "Thu");
        assert_eq!(WEEKDAYS[date("2024-02-29").weekday()], "Thu");
        assert_eq!(WEEKDAYS[date("1900-01-01").weekday()], "Mon");
    }

    #[test]
    fn day_arithmetic_crosses_months_years_and_leap_days() {
        assert_eq!(date("2026-11-30").add_days(1), date("2026-12-01"));
        assert_eq!(date("2026-12-31").add_days(1), date("2027-01-01"));
        assert_eq!(date("2024-02-28").add_days(1), date("2024-02-29"));
        assert_eq!(date("2023-02-28").add_days(1), date("2023-03-01"));
        assert_eq!(date("2026-01-01").add_days(-1), date("2025-12-31"));
        assert_eq!(date("2026-11-11").add_days(365), date("2027-11-11"));
        assert_eq!(date("2026-11-11").week_start(), date("2026-11-09"));
        assert_eq!(date("2026-11-09").week_start(), date("2026-11-09"));
        assert_eq!(date("2027-01-03").week_start(), date("2026-12-28"));
    }

    #[test]
    fn month_arithmetic_keeps_the_day_or_clamps_it() {
        assert_eq!(date("2026-11-11").add_months(1), date("2026-12-11"));
        assert_eq!(date("2026-12-15").add_months(1), date("2027-01-15"));
        assert_eq!(date("2026-01-31").add_months(1), date("2026-02-28"));
        assert_eq!(date("2024-01-31").add_months(1), date("2024-02-29"));
        assert_eq!(date("2026-03-31").add_months(-1), date("2026-02-28"));
        assert_eq!(date("2026-01-10").add_months(-1), date("2025-12-10"));
        assert_eq!(date("2024-02-29").add_months(12), date("2025-02-28"));
    }

    #[test]
    fn month_weeks_cover_the_month_from_monday_to_sunday() {
        let weeks = date("2026-11-11").month_weeks();
        assert_eq!(
            weeks.len(),
            6,
            "November 2026 starts on a Sunday and ends on a Monday"
        );
        assert_eq!(weeks[0][0], date("2026-10-26"));
        assert_eq!(weeks[5][6], date("2026-12-06"));
        let weeks = date("2027-02-10").month_weeks();
        assert_eq!(
            (weeks.len(), weeks[0][0], weeks[3][6]),
            (4, date("2027-02-01"), date("2027-02-28")),
            "February 2027 is exactly four weeks"
        );
    }
}
