//! Game calendar. One tick is one day; the proleptic Gregorian calendar is used.

use serde::{Deserialize, Serialize};

/// A calendar day. Ordering follows the calendar.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Date {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

impl Date {
    pub const fn new(year: i32, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    pub const fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
    }

    pub const fn days_in_month(year: i32, month: u8) -> u8 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if Self::is_leap_year(year) => 29,
            _ => 28,
        }
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
}

#[cfg(test)]
mod tests {
    use super::Date;

    #[test]
    fn rolls_over_month_year_and_leap_day() {
        assert_eq!(Date::new(1925, 1, 31).next_day(), Date::new(1925, 2, 1));
        assert_eq!(Date::new(1925, 12, 31).next_day(), Date::new(1926, 1, 1));
        assert_eq!(Date::new(1928, 2, 28).next_day(), Date::new(1928, 2, 29));
        assert_eq!(Date::new(1900, 2, 28).next_day(), Date::new(1900, 3, 1));
        assert_eq!(Date::new(2000, 2, 28).next_day(), Date::new(2000, 2, 29));
    }
}
