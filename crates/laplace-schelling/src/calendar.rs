//! Game calendar. One tick is one day; the proleptic Gregorian calendar is used.

use core::fmt;

use serde::{Deserialize, Serialize};

/// A calendar day. Ordering follows the calendar.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Date {
    /// Year, proleptic Gregorian (no year zero skipping: astronomical numbering).
    pub year: i32,
    /// Month, 1 to 12.
    pub month: u8,
    /// Day of the month, 1 to 31.
    pub day: u8,
}

/// Day of the week.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Weekday {
    /// Monday.
    Monday,
    /// Tuesday.
    Tuesday,
    /// Wednesday.
    Wednesday,
    /// Thursday.
    Thursday,
    /// Friday.
    Friday,
    /// Saturday.
    Saturday,
    /// Sunday.
    Sunday,
}

impl Weekday {
    /// All weekdays, Monday first.
    pub const ALL: [Weekday; 7] = [
        Weekday::Monday,
        Weekday::Tuesday,
        Weekday::Wednesday,
        Weekday::Thursday,
        Weekday::Friday,
        Weekday::Saturday,
        Weekday::Sunday,
    ];
}

impl Date {
    /// Builds a date without validating it; see [`Date::is_valid`].
    pub const fn new(year: i32, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    /// Whether `year` is a Gregorian leap year.
    pub const fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
    }

    /// Number of days in `month` of `year`.
    pub const fn days_in_month(year: i32, month: u8) -> u8 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if Self::is_leap_year(year) => 29,
            _ => 28,
        }
    }

    /// Whether the month and day exist.
    pub const fn is_valid(self) -> bool {
        self.month >= 1
            && self.month <= 12
            && self.day >= 1
            && self.day <= Self::days_in_month(self.year, self.month)
    }

    /// The following day.
    pub const fn next_day(self) -> Self {
        if self.day < Self::days_in_month(self.year, self.month) {
            Self::new(self.year, self.month, self.day + 1)
        } else if self.month < 12 {
            Self::new(self.year, self.month + 1, 1)
        } else {
            Self::new(self.year + 1, 1, 1)
        }
    }

    /// Days since 1970-01-01 (negative before it).
    ///
    /// Algorithm `days_from_civil` by Howard Hinnant,
    /// <https://howardhinnant.github.io/date_algorithms.html>.
    pub const fn to_days(self) -> i64 {
        let month = self.month as i64;
        let year = self.year as i64 - if month <= 2 { 1 } else { 0 };
        let era = if year >= 0 { year } else { year - 399 } / 400;
        let year_of_era = year - era * 400;
        let month_index = if month > 2 { month - 3 } else { month + 9 };
        let day_of_year = (153 * month_index + 2) / 5 + self.day as i64 - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        era * 146_097 + day_of_era - 719_468
    }

    /// Inverse of [`Date::to_days`] (Hinnant's `civil_from_days`).
    pub const fn from_days(days: i64) -> Self {
        let z = days + 719_468;
        let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
        let day_of_era = z - era * 146_097;
        let year_of_era =
            (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let month_index = (5 * day_of_year + 2) / 153;
        let day = day_of_year - (153 * month_index + 2) / 5 + 1;
        let month = if month_index < 10 {
            month_index + 3
        } else {
            month_index - 9
        };
        let year = year_of_era + era * 400 + if month <= 2 { 1 } else { 0 };
        Self::new(year as i32, month as u8, day as u8)
    }

    /// Day of the week.
    pub const fn weekday(self) -> Weekday {
        // 1970-01-01 was a Thursday.
        Weekday::ALL[(self.to_days() + 3).rem_euclid(7) as usize]
    }
}

/// ISO 8601: `1925-01-01`.
impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[cfg(test)]
mod tests {
    use super::{Date, Weekday};

    #[test]
    fn rolls_over_month_year_and_leap_day() {
        assert_eq!(Date::new(1925, 1, 31).next_day(), Date::new(1925, 2, 1));
        assert_eq!(Date::new(1925, 12, 31).next_day(), Date::new(1926, 1, 1));
        assert_eq!(Date::new(1928, 2, 28).next_day(), Date::new(1928, 2, 29));
        assert_eq!(Date::new(1900, 2, 28).next_day(), Date::new(1900, 3, 1));
        assert_eq!(Date::new(2000, 2, 28).next_day(), Date::new(2000, 2, 29));
    }

    #[test]
    fn day_numbers_round_trip_and_follow_next_day() {
        assert_eq!(Date::new(1970, 1, 1).to_days(), 0);
        assert_eq!(Date::new(2000, 3, 1).to_days(), 11_017);
        let mut date = Date::new(1899, 12, 1);
        let mut days = date.to_days();
        while date < Date::new(2101, 1, 1) {
            assert!(date.is_valid());
            assert_eq!(Date::from_days(days), date);
            date = date.next_day();
            days += 1;
            assert_eq!(date.to_days(), days);
        }
    }

    #[test]
    fn weekdays_match_the_calendar() {
        assert_eq!(Date::new(1925, 1, 1).weekday(), Weekday::Thursday);
        assert_eq!(Date::new(1970, 1, 1).weekday(), Weekday::Thursday);
        assert_eq!(Date::new(2000, 1, 1).weekday(), Weekday::Saturday);
        assert_eq!(Date::new(2025, 1, 1).weekday(), Weekday::Wednesday);
    }

    #[test]
    fn displays_as_iso() {
        assert_eq!(Date::new(1925, 1, 1).to_string(), "1925-01-01");
        assert!(!Date::new(1925, 2, 29).is_valid());
    }
}
