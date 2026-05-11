//! BrightDate Timezone Utilities
//!
//! While BrightDate is inherently timezone-free (UTC-based),
//! these utilities bridge to local civil time for display purposes.
//!
//! Philosophy: store and compute in BrightDate, display in local time only at
//! the edges.

use crate::types::BrightDateValue;

// ─── Timezone offset table ────────────────────────────────────────────────────

/// Named timezone offsets as `(name, fractional_days)`.
/// Positive = East of UTC, Negative = West of UTC.
pub const TIMEZONE_OFFSETS: &[(&str, f64)] = &[
    // Americas
    ("UTC-12", -12.0 / 24.0),
    ("UTC-11", -11.0 / 24.0),
    ("UTC-10", -10.0 / 24.0), // Hawaii
    ("UTC-9", -9.0 / 24.0),   // Alaska
    ("UTC-8", -8.0 / 24.0),   // Pacific
    ("UTC-7", -7.0 / 24.0),   // Mountain
    ("UTC-6", -6.0 / 24.0),   // Central
    ("UTC-5", -5.0 / 24.0),   // Eastern
    ("UTC-4", -4.0 / 24.0),   // Atlantic
    ("UTC-3", -3.0 / 24.0),   // Argentina, Brazil
    ("UTC-2", -2.0 / 24.0),
    ("UTC-1", -1.0 / 24.0),
    // Europe/Africa
    ("UTC+0", 0.0),           // UK, Portugal, Ghana
    ("UTC+1", 1.0 / 24.0),   // Central Europe
    ("UTC+2", 2.0 / 24.0),   // Eastern Europe
    ("UTC+3", 3.0 / 24.0),   // Moscow, East Africa
    // Asia
    ("UTC+4", 4.0 / 24.0),    // Gulf
    ("UTC+5", 5.0 / 24.0),    // Pakistan
    ("UTC+5.5", 5.5 / 24.0),  // India
    ("UTC+5.75", 5.75 / 24.0),// Nepal
    ("UTC+6", 6.0 / 24.0),    // Bangladesh
    ("UTC+6.5", 6.5 / 24.0),  // Myanmar
    ("UTC+7", 7.0 / 24.0),    // Indochina
    ("UTC+8", 8.0 / 24.0),    // China, Singapore
    ("UTC+9", 9.0 / 24.0),    // Japan, Korea
    ("UTC+9.5", 9.5 / 24.0),  // Central Australia
    ("UTC+10", 10.0 / 24.0),  // Eastern Australia
    ("UTC+11", 11.0 / 24.0),
    ("UTC+12", 12.0 / 24.0),  // New Zealand
    ("UTC+13", 13.0 / 24.0),  // Samoa
    ("UTC+14", 14.0 / 24.0),  // Line Islands
];

// ─── Core functions ───────────────────────────────────────────────────────────

/// Shift a UTC BrightDate by a timezone offset (for display only).
///
/// **Note:** The returned value is *not* a true UTC BrightDate; it is offset
/// from the universal timeline. Use only for display purposes.
pub fn to_local_value(bright_date: BrightDateValue, offset_days: f64) -> BrightDateValue {
    bright_date + offset_days
}

/// Convert a "local BrightDate" back to UTC by removing the timezone offset.
pub fn from_local_value(local_value: BrightDateValue, offset_days: f64) -> BrightDateValue {
    local_value - offset_days
}

/// Look up a named timezone offset in fractional days.
///
/// Returns `None` if the timezone name is not in the table.
pub fn get_timezone_offset(timezone: &str) -> Option<f64> {
    TIMEZONE_OFFSETS
        .iter()
        .find(|(name, _)| *name == timezone)
        .map(|(_, offset)| *offset)
}

/// Convert a timezone offset in hours to fractional days.
pub fn hours_to_fractional_days(hours: f64) -> f64 {
    hours / 24.0
}

/// Convert a timezone offset in fractional days to hours.
pub fn fractional_days_to_hours(fractional_days: f64) -> f64 {
    fractional_days * 24.0
}

/// Format a BrightDate with a timezone annotation.
///
/// Example output: `"9622.50417 (UTC-5: 9622.29584)"`.
pub fn format_with_timezone(bright_date: BrightDateValue, timezone: &str, precision: usize) -> String {
    match get_timezone_offset(timezone) {
        None => format!(
            "{:.prec$} (unknown timezone: {timezone})",
            bright_date,
            prec = precision
        ),
        Some(offset) => {
            let local = to_local_value(bright_date, offset);
            format!(
                "{:.prec$} ({timezone}: {:.prec$})",
                bright_date,
                local,
                prec = precision
            )
        }
    }
}

/// Return the local time-of-day as a fraction in `[0, 1)`.
pub fn local_time_of_day(bright_date: BrightDateValue, offset_days: f64) -> f64 {
    let local = bright_date + offset_days;
    let frac = local - local.floor();
    ((frac % 1.0) + 1.0) % 1.0
}

/// Return `true` if local time is between 06:00 and 18:00 (simple approximation).
pub fn is_daytime(bright_date: BrightDateValue, offset_days: f64) -> bool {
    let tod = local_time_of_day(bright_date, offset_days);
    (6.0 / 24.0..18.0 / 24.0).contains(&tod)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utc_offset_zero() {
        assert_eq!(get_timezone_offset("UTC+0"), Some(0.0));
    }

    #[test]
    fn local_value_roundtrip() {
        let bd = 9622.5;
        let offset = -5.0 / 24.0;
        let local = to_local_value(bd, offset);
        let back = from_local_value(local, offset);
        assert!((back - bd).abs() < 1e-12);
    }

    #[test]
    fn hours_conversion_roundtrip() {
        let h = 5.5_f64;
        assert!((fractional_days_to_hours(hours_to_fractional_days(h)) - h).abs() < 1e-12);
    }

    #[test]
    fn daytime_detection() {
        // noon UTC + no offset
        let noon = 0.0 + 12.0 / 24.0; // noon on day 0 as fraction
        // A BrightDate where the fractional part is 0.5 is noon UTC
        assert!(is_daytime(100.5, 0.0));
        // 2 AM UTC
        assert!(!is_daytime(100.0 + 2.0 / 24.0, 0.0));
        let _ = noon; // suppress lint
    }
}
