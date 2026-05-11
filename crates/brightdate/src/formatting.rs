//! Formatting BrightDate values as human-readable strings.

use crate::types::{BrightDateComponents, BrightDateValue, BrightDuration, FormattedBrightDate, Precision};

/// Format a BrightDate value to `precision` decimal places.
pub fn format_bright_date(bd: BrightDateValue, precision: Precision) -> String {
    format!("{:.prec$}", bd, prec = precision as usize)
}

/// Decompose a BrightDate value into its component parts.
///
/// Quantizes the fractional day to the nearest integer nanoday once, then
/// performs all sub-day decomposition via integer division. This guarantees
/// that `millidays`, `microdays`, and `nanodays` are consistent slices of a
/// single canonical 9-digit fractional representation (no floor-drift across
/// units).
pub fn decompose(bd: BrightDateValue) -> BrightDateComponents {
    let mut day = bd.floor() as i64;
    let fraction = bd - day as f64;
    // Round to nearest integer nanoday (1 day = 1e9 nd).
    let mut total_nano = (fraction * 1_000_000_000.0).round() as i64;
    // Carry up if rounding pushed us to the next whole day.
    if total_nano >= 1_000_000_000 {
        day += 1;
        total_nano -= 1_000_000_000;
    }
    let millidays = (total_nano / 1_000_000) as u32;
    let microdays = ((total_nano / 1_000) % 1_000) as u32;
    let nanodays = (total_nano % 1_000) as u32;
    BrightDateComponents { day, fraction, value: bd, millidays, microdays, nanodays }
}

/// Full decomposed display object.
pub fn format_full(bd: BrightDateValue, precision: Precision) -> FormattedBrightDate {
    let full = format_bright_date(bd, precision);
    let dot = full.find('.');
    let day = dot.map_or(full.clone(), |i| full[..i].to_string());
    let fraction = dot.map_or_else(
        || "0".repeat(precision as usize),
        |i| full[i + 1..].to_string(),
    );
    let c = decompose(bd);
    let friendly = format!("Day {}, {} md", c.day, c.millidays);
    FormattedBrightDate { full, day, fraction, friendly }
}

/// Format a BrightDate value as a compact log string: `"[9622.50417]"`.
pub fn format_log(bd: BrightDateValue, precision: Precision) -> String {
    format!("[{}]", format_bright_date(bd, precision))
}

/// Format a BrightDate value with a prefix label (default `"BD:"`).
pub fn format_prefixed(bd: BrightDateValue, precision: Precision, prefix: Option<&str>) -> String {
    format!("{}{}", prefix.unwrap_or("BD:"), format_bright_date(bd, precision))
}

/// Convert a duration in decimal days to a `BrightDuration`.
pub fn to_duration(days: f64) -> BrightDuration {
    let abs = days.abs();
    BrightDuration {
        days,
        millidays: abs * 1_000.0,
        microdays: abs * 1_000_000.0,
        nanodays: abs * 1_000_000_000.0,
    }
}

/// Format a duration in the most human-appropriate metric unit.
pub fn format_duration(days: f64) -> String {
    let abs = days.abs();
    let sign = if days < 0.0 { "-" } else { "" };
    if abs >= 1.0 {
        format!("{}{:.3} days", sign, abs)
    } else if abs >= 1e-3 {
        format!("{}{:.3} millidays", sign, abs * 1_000.0)
    } else if abs >= 1e-6 {
        format!("{}{:.3} microdays", sign, abs * 1_000_000.0)
    } else {
        format!("{}{:.3} nanodays", sign, abs * 1_000_000_000.0)
    }
}

/// Format a BrightDate range string.
pub fn format_range(start: BrightDateValue, end: BrightDateValue, precision: Precision) -> String {
    format!(
        "{}..{}",
        format_bright_date(start, precision),
        format_bright_date(end, precision)
    )
}

/// Convert a day fraction to HH:MM:SS.mmm components.
///
/// Returns `(hours, minutes, seconds, milliseconds)`.
pub fn day_fraction_to_hms(fraction: f64) -> (u32, u32, u32, u32) {
    let total_ms = (fraction * 86_400_000.0).round() as u64;
    let ms = (total_ms % 1_000) as u32;
    let total_s = total_ms / 1_000;
    let secs = (total_s % 60) as u32;
    let total_m = total_s / 60;
    let mins = (total_m % 60) as u32;
    let hours = (total_m / 60) as u32;
    (hours, mins, secs, ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_precision() {
        assert_eq!(format_bright_date(9622.50417, 5), "9622.50417");
        assert_eq!(format_bright_date(0.0, 5), "0.00000");
    }

    #[test]
    fn decompose_values() {
        let c = decompose(9622.504_17);
        assert_eq!(c.day, 9622);
        assert_eq!(c.millidays, 504);
    }

    #[test]
    fn hms_noon() {
        let (h, m, s, ms) = day_fraction_to_hms(0.5);
        assert_eq!(h, 12);
        assert_eq!(m, 0);
        assert_eq!(s, 0);
        assert_eq!(ms, 0);
    }

    #[test]
    fn duration_format_days() {
        assert_eq!(format_duration(1.5), "1.500 days");
    }

    #[test]
    fn duration_format_millidays() {
        // 0.5 milliday = 500 microdays (below the 1-milliday threshold)
        assert_eq!(format_duration(0.0005), "500.000 microdays");
        // 0.5 days = 500 millidays (above the 1-milliday threshold)
        assert_eq!(format_duration(0.5), "500.000 millidays");
    }
}
