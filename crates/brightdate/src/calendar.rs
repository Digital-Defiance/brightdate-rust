//! BrightDate Calendar Utilities
//!
//! Convenience functions that bridge BrightDate values to traditional Gregorian
//! calendar concepts such as year boundaries, month boundaries, day-of-week,
//! and leap-year detection.

use chrono::{Datelike, TimeZone, Utc};

use crate::conversions::{from_unix_ms, to_date_time};
use crate::types::{BrightDateError, BrightDateValue};

// ─── Helper ───────────────────────────────────────────────────────────────────

fn date_to_bd(year: i32, month: u32, day: u32, hour: u32, min: u32, sec: u32) -> BrightDateValue {
    let dt = Utc
        .with_ymd_and_hms(year, month, day, hour, min, sec)
        .single()
        .expect("invalid calendar date");
    from_unix_ms(dt.timestamp_millis() as f64).unwrap_or(f64::NAN)
}

// ─── Year boundaries ──────────────────────────────────────────────────────────

/// BrightDate at the start of `year` (Jan 1, 00:00:00 UTC).
pub fn start_of_year(year: i32) -> BrightDateValue {
    date_to_bd(year, 1, 1, 0, 0, 0)
}

/// BrightDate at the last instant of `year` (Dec 31, 23:59:59 UTC).
pub fn end_of_year(year: i32) -> BrightDateValue {
    date_to_bd(year, 12, 31, 23, 59, 59)
}

// ─── Month boundaries ─────────────────────────────────────────────────────────

/// BrightDate at the start of month `month` (1–12) of `year`.
pub fn start_of_month(year: i32, month: u32) -> BrightDateValue {
    date_to_bd(year, month, 1, 0, 0, 0)
}

/// BrightDate at the last second of month `month` (1–12) of `year`.
pub fn end_of_month(year: i32, month: u32) -> BrightDateValue {
    // First day of the *next* month, then subtract one second
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    let start_next = date_to_bd(next_year, next_month, 1, 0, 0, 0);
    // subtract 1 second expressed in decimal days
    start_next - 1.0 / 86_400.0
}

// ─── Calendar field accessors ─────────────────────────────────────────────────

/// Calendar year of a BrightDate.
pub fn get_year(bd: BrightDateValue) -> i32 {
    to_date_time(bd).year()
}

/// Calendar month (1–12) of a BrightDate.
pub fn get_month(bd: BrightDateValue) -> u32 {
    to_date_time(bd).month()
}

/// Day of month (1–31) of a BrightDate.
pub fn get_day_of_month(bd: BrightDateValue) -> u32 {
    to_date_time(bd).day()
}

/// Day of week (0 = Sunday, 6 = Saturday) of a BrightDate.
pub fn get_day_of_week(bd: BrightDateValue) -> u32 {
    // chrono: Mon=0..Sun=6 for `weekday().num_days_from_monday()`
    // We need Sun=0..Sat=6
    let wd = to_date_time(bd).weekday();
    wd.num_days_from_sunday()
}

/// Day of year (1–366) of a BrightDate.
pub fn get_day_of_year(bd: BrightDateValue) -> u32 {
    to_date_time(bd).ordinal()
}

// ─── Leap year & month length ─────────────────────────────────────────────────

/// Return `true` if the year containing `bd` is a leap year.
pub fn is_leap_year(bd: BrightDateValue) -> bool {
    let year = get_year(bd);
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Number of days in the year containing `bd` (365 or 366).
pub fn days_in_year(bd: BrightDateValue) -> u32 {
    if is_leap_year(bd) { 366 } else { 365 }
}

/// Number of days in the month containing `bd` (28–31).
pub fn days_in_month(bd: BrightDateValue) -> u32 {
    let dt = to_date_time(bd);
    let year = dt.year();
    let month = dt.month();
    // Last day of the month = first day of next month, day 0
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1u32)
    } else {
        (year, month + 1)
    };
    Utc.with_ymd_and_hms(next_year, next_month, 1, 0, 0, 0)
        .single()
        .map(|d| {
            let prev = d - chrono::Duration::days(1);
            prev.day()
        })
        .unwrap_or(30)
}

// ─── fromCalendar shorthand ───────────────────────────────────────────────────

/// Build a BrightDate from individual calendar fields (UTC).
///
/// # Errors
/// Returns `BrightDateError::InvalidInput` if the date is not valid.
pub fn from_calendar(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> Result<BrightDateValue, BrightDateError> {
    let dt = Utc
        .with_ymd_and_hms(year, month, day, hour, minute, second)
        .single()
        .ok_or_else(|| {
            BrightDateError::InvalidInput(format!(
                "invalid calendar date: {year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}"
            ))
        })?;
    from_unix_ms(dt.timestamp_millis() as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_of_year_2000() {
        // 2000-01-01T00:00:00Z in Unix ms
        let bd = start_of_year(2000);
        let dt = to_date_time(bd);
        assert_eq!(dt.year(), 2000);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 1);
    }

    #[test]
    fn end_of_year_before_next() {
        let end_2024 = end_of_year(2024);
        let start_2025 = start_of_year(2025);
        assert!(end_2024 < start_2025);
    }

    #[test]
    fn month_boundaries_2000_02() {
        let start = start_of_month(2000, 2);
        let end = end_of_month(2000, 2);
        let dt_start = to_date_time(start);
        let dt_end = to_date_time(end);
        assert_eq!(dt_start.month(), 2);
        assert_eq!(dt_end.month(), 2);
        assert!(start < end);
    }

    #[test]
    fn leap_year_detection() {
        let bd_2000 = start_of_year(2000);
        let bd_1900 = start_of_year(1900);
        assert!(is_leap_year(bd_2000));
        assert!(!is_leap_year(bd_1900));
    }

    #[test]
    fn days_in_month_feb_leap() {
        let bd = start_of_month(2000, 2);
        assert_eq!(days_in_month(bd), 29);
    }

    #[test]
    fn from_calendar_roundtrip() {
        let bd = from_calendar(2024, 6, 15, 12, 0, 0).unwrap();
        let dt = to_date_time(bd);
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 6);
        assert_eq!(dt.day(), 15);
    }

    #[test]
    fn day_of_week_j2000() {
        // J2000.0 = 2000-01-01T12:00:00Z — that's a Saturday (6)
        let bd = 0.0_f64;
        assert_eq!(get_day_of_week(bd), 6);
    }
}
