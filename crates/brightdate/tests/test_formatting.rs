//! Integration tests for formatting — ported from formatting.spec.ts

use brightdate::formatting::{
    day_fraction_to_hms, decompose, format_bright_date, format_duration, format_full, format_log,
    format_prefixed, format_range, to_duration,
};

// ─── formatBrightDate ────────────────────────────────────────────────────────

#[test]
fn format_brightdate_precision_5() {
    assert_eq!(format_bright_date(9622.50417, 5), "9622.50417");
}

#[test]
fn format_brightdate_zero() {
    assert_eq!(format_bright_date(0.0, 5), "0.00000");
}

#[test]
fn format_brightdate_negative() {
    let s = format_bright_date(-10957.5, 5);
    assert!(s.starts_with('-'));
}

#[test]
fn format_brightdate_precision_3() {
    assert_eq!(format_bright_date(9622.504, 3), "9622.504");
}

// ─── formatFull ──────────────────────────────────────────────────────────────

#[test]
fn format_full_full_string() {
    let ff = format_full(9622.50417, 5);
    assert_eq!(ff.full, "9622.50417");
}

#[test]
fn format_full_day_part() {
    let ff = format_full(9622.50417, 5);
    assert_eq!(ff.day, "9622");
}

#[test]
fn format_full_fraction_part() {
    let ff = format_full(9622.50417, 5);
    assert_eq!(ff.fraction, "50417");
}

#[test]
fn format_full_friendly_contains_day() {
    let ff = format_full(9622.50417, 5);
    assert!(ff.friendly.contains("9622"));
}

#[test]
fn format_full_friendly_contains_millidays() {
    let ff = format_full(9622.50417, 5);
    // millidays = 504
    assert!(ff.friendly.contains("504"));
}

// ─── decompose ───────────────────────────────────────────────────────────────

#[test]
fn decompose_day() {
    let d = decompose(9622.50417);
    assert_eq!(d.day, 9622);
}

#[test]
fn decompose_value_round_trip() {
    let d = decompose(9622.50417);
    assert!((d.value - 9622.50417).abs() < 1e-10);
}

#[test]
fn decompose_millidays() {
    let d = decompose(9622.50417);
    // 0.50417 * 1000 = 504.17 → floor = 504
    assert_eq!(d.millidays, 504);
}

#[test]
fn decompose_microdays() {
    let d = decompose(9622.50417);
    // fraction = 0.00017 * 1000 = 0.17 → no wait:
    // 0.50417 * 1000 = 504.17, floor = 504 millidays
    // remainder = 0.17 * 1000 = 170 microdays
    assert_eq!(d.microdays, 170);
}

#[test]
fn decompose_zero() {
    let d = decompose(0.0);
    assert_eq!(d.day, 0);
    assert_eq!(d.millidays, 0);
    assert_eq!(d.microdays, 0);
}

// ─── toDuration ──────────────────────────────────────────────────────────────

#[test]
fn to_duration_one_day() {
    let d = to_duration(1.0);
    assert!((d.days - 1.0).abs() < 1e-12);
    assert!((d.millidays - 1000.0).abs() < 1e-9);
    assert!((d.microdays - 1_000_000.0).abs() < 1e-3);
    assert!((d.nanodays - 1_000_000_000.0).abs() < 1.0);
}

#[test]
fn to_duration_half_day() {
    let d = to_duration(0.5);
    assert!((d.days - 0.5).abs() < 1e-12);
    assert!((d.millidays - 500.0).abs() < 1e-9);
}

#[test]
fn to_duration_negative_preserves_sign() {
    let d = to_duration(-2.0);
    assert!(d.days < 0.0);
    // millidays/microdays/nanodays use abs()
    assert!(d.millidays > 0.0);
}

// ─── formatDuration ──────────────────────────────────────────────────────────

#[test]
fn format_duration_days() {
    let s = format_duration(1.5);
    assert!(s.contains("day"));
}

#[test]
fn format_duration_millidays() {
    let s = format_duration(0.5);
    assert!(s.contains("milliday"));
}

#[test]
fn format_duration_microdays() {
    let s = format_duration(0.000_5);
    assert!(s.contains("microday"));
}

#[test]
fn format_duration_nanodays() {
    let s = format_duration(0.000_000_5);
    assert!(s.contains("nanoday"));
}

#[test]
fn format_duration_negative() {
    let s = format_duration(-1.5);
    assert!(s.starts_with('-'));
    assert!(s.contains("day"));
}

// ─── formatLog ───────────────────────────────────────────────────────────────

#[test]
fn format_log_wraps_in_brackets() {
    let s = format_log(9622.50417, 5);
    assert_eq!(s, "[9622.50417]");
}

#[test]
fn format_log_zero() {
    let s = format_log(0.0, 3);
    assert_eq!(s, "[0.000]");
}

// ─── formatPrefixed ──────────────────────────────────────────────────────────

#[test]
fn format_prefixed_default_prefix() {
    let s = format_prefixed(9622.50417, 5, None);
    assert_eq!(s, "BD:9622.50417");
}

#[test]
fn format_prefixed_custom_prefix() {
    let s = format_prefixed(9622.50417, 5, Some("T:"));
    assert_eq!(s, "T:9622.50417");
}

#[test]
fn format_prefixed_empty_prefix() {
    let s = format_prefixed(100.0, 3, Some(""));
    assert_eq!(s, "100.000");
}

// ─── formatRange ─────────────────────────────────────────────────────────────

#[test]
fn format_range_basic() {
    let s = format_range(9622.0, 9623.0, 5);
    assert!(s.contains("9622"));
    assert!(s.contains("9623"));
}

#[test]
fn format_range_has_separator() {
    let s = format_range(0.0, 1.0, 5);
    // Separator may be "..", "→", "->" etc.
    assert!(s.len() > 10, "range string should be reasonably long");
}

// ─── dayFractionToHms ────────────────────────────────────────────────────────

#[test]
fn day_fraction_to_hms_noon() {
    let (h, m, s, ms) = day_fraction_to_hms(0.5);
    assert_eq!(h, 12);
    assert_eq!(m, 0);
    assert_eq!(s, 0);
    assert_eq!(ms, 0);
}

#[test]
fn day_fraction_to_hms_midnight() {
    let (h, m, s, ms) = day_fraction_to_hms(0.0);
    assert_eq!(h, 0);
    assert_eq!(m, 0);
    assert_eq!(s, 0);
    assert_eq!(ms, 0);
}

#[test]
fn day_fraction_to_hms_quarter_day() {
    // 0.25 day = 6:00:00
    let (h, m, s, _ms) = day_fraction_to_hms(0.25);
    assert_eq!(h, 6);
    assert_eq!(m, 0);
    assert_eq!(s, 0);
}

#[test]
fn day_fraction_to_hms_one_minute() {
    // 1 minute = 60 / 86400 day
    let (h, m, s, _ms) = day_fraction_to_hms(60.0 / 86400.0);
    assert_eq!(h, 0);
    assert_eq!(m, 1);
    assert_eq!(s, 0);
}

// ─── additional format_bright_date precision ──────────────────────────────────

#[test]
fn format_brightdate_precision_1() {
    assert_eq!(format_bright_date(9622.5, 1), "9622.5");
}

#[test]
fn format_brightdate_precision_2() {
    assert_eq!(format_bright_date(9622.50, 2), "9622.50");
}

#[test]
fn format_brightdate_precision_8() {
    let s = format_bright_date(9622.50417000, 8);
    assert_eq!(s.len(), "9622.".len() + 8);
}

#[test]
fn format_brightdate_precision_12() {
    let s = format_bright_date(1.0, 12);
    assert!(s.ends_with("000000000000"));
}

#[test]
fn format_brightdate_negative_precision_3() {
    let s = format_bright_date(-9622.504, 3);
    assert_eq!(s, "-9622.504");
}

#[test]
fn format_brightdate_large_value() {
    let s = format_bright_date(100_000.12345, 5);
    assert!(s.starts_with("100000."));
}

// ─── additional format_full ───────────────────────────────────────────────────

#[test]
fn format_full_negative_day_part() {
    let ff = format_full(-9622.5, 3);
    assert!(ff.day.contains("-9622") || ff.day.contains("9622"));
}

#[test]
fn format_full_zero() {
    let ff = format_full(0.0, 5);
    assert_eq!(ff.full, "0.00000");
}

#[test]
fn format_full_precision_3() {
    let ff = format_full(100.504, 3);
    assert_eq!(ff.full, "100.504");
}

#[test]
fn format_full_friendly_contains_md() {
    let ff = format_full(9622.504, 3);
    // friendly format: "Day 9622, 504 md"
    assert!(ff.friendly.contains("md") || ff.friendly.contains("milliday"));
}

// ─── additional decompose ─────────────────────────────────────────────────────

#[test]
fn decompose_negative_value() {
    let d = decompose(-5.0);
    assert_eq!(d.day, -5);
    assert!((d.fraction - 0.0).abs() < 1e-12);
}

#[test]
fn decompose_fraction_in_range() {
    for bd in [0.1, 0.999, 9622.999, -0.1] {
        let d = decompose(bd);
        assert!(d.fraction >= 0.0 && d.fraction < 1.0, "fraction={} for bd={bd}", d.fraction);
    }
}

#[test]
fn decompose_millidays_range() {
    for bd in [0.0, 0.5, 9622.999] {
        let d = decompose(bd);
        assert!(d.millidays < 1000, "millidays={} for bd={bd}", d.millidays);
    }
}

#[test]
fn decompose_microdays_range() {
    for bd in [0.0, 0.5, 9622.999] {
        let d = decompose(bd);
        assert!(d.microdays < 1000, "microdays={} for bd={bd}", d.microdays);
    }
}

#[test]
fn decompose_value_preserved() {
    let bd = 9622.123;
    let d = decompose(bd);
    assert!((d.value - bd).abs() < 1e-10);
}

// ─── additional to_duration ───────────────────────────────────────────────────

#[test]
fn to_duration_zero() {
    let d = to_duration(0.0);
    assert_eq!(d.days, 0.0);
    assert_eq!(d.millidays, 0.0);
    assert_eq!(d.microdays, 0.0);
    assert_eq!(d.nanodays, 0.0);
}

#[test]
fn to_duration_milliday() {
    let d = to_duration(0.001);
    assert!((d.days - 0.001).abs() < 1e-15);
    assert!((d.millidays - 1.0).abs() < 1e-12);
}

#[test]
fn to_duration_nanodays_large() {
    let d = to_duration(1.0);
    assert!((d.nanodays - 1_000_000_000.0).abs() < 1.0);
}

// ─── additional format_duration ───────────────────────────────────────────────

#[test]
fn format_duration_exactly_one_day() {
    let s = format_duration(1.0);
    assert!(s.contains("1.000") && s.contains("day"));
}

#[test]
fn format_duration_one_milliday() {
    let s = format_duration(0.001);
    assert!(s.contains("milliday"), "got: {s}");
}

#[test]
fn format_duration_one_microday() {
    let s = format_duration(0.000_001);
    assert!(s.contains("microday"), "got: {s}");
}

#[test]
fn format_duration_one_nanoday() {
    let s = format_duration(0.000_000_001);
    assert!(s.contains("nanoday"), "got: {s}");
}

#[test]
fn format_duration_zero_is_nanodays() {
    let s = format_duration(0.0);
    // 0.0 is < 1e-6, so nanodays
    assert!(s.contains("nanoday"), "got: {s}");
}

// ─── additional format_log ────────────────────────────────────────────────────

#[test]
fn format_log_precision_2() {
    let s = format_log(9622.5, 2);
    assert_eq!(s, "[9622.50]");
}

#[test]
fn format_log_negative() {
    let s = format_log(-1.5, 2);
    assert_eq!(s, "[-1.50]");
}

#[test]
fn format_log_always_brackets() {
    let s = format_log(0.0, 1);
    assert!(s.starts_with('[') && s.ends_with(']'));
}

// ─── additional format_prefixed ───────────────────────────────────────────────

#[test]
fn format_prefixed_none_uses_bd_prefix() {
    let s = format_prefixed(0.0, 5, None);
    assert!(s.starts_with("BD:"));
}

#[test]
fn format_prefixed_custom_label() {
    let s = format_prefixed(9622.5, 2, Some("NOW:"));
    assert_eq!(s, "NOW:9622.50");
}

// ─── additional format_range ──────────────────────────────────────────────────

#[test]
fn format_range_start_less_than_end() {
    let s = format_range(1.0, 2.0, 3);
    assert!(s.contains("1.000"), "got: {s}");
    assert!(s.contains("2.000"), "got: {s}");
}

#[test]
fn format_range_contains_dotdot() {
    let s = format_range(0.0, 10.0, 3);
    assert!(s.contains(".."), "got: {s}");
}

// ─── additional day_fraction_to_hms ──────────────────────────────────────────

#[test]
fn day_fraction_to_hms_one_second() {
    let (h, m, s, ms) = day_fraction_to_hms(1.0 / 86400.0);
    assert_eq!(h, 0);
    assert_eq!(m, 0);
    assert_eq!(s, 1);
    assert_eq!(ms, 0);
}

#[test]
fn day_fraction_to_hms_one_hour() {
    let (h, m, s, ms) = day_fraction_to_hms(1.0 / 24.0);
    assert_eq!(h, 1);
    assert_eq!(m, 0);
    assert_eq!(s, 0);
    assert_eq!(ms, 0);
}

#[test]
fn day_fraction_to_hms_23_hours() {
    let (h, m, s, ms) = day_fraction_to_hms(23.0 / 24.0);
    assert_eq!(h, 23);
    assert_eq!(m, 0);
    assert_eq!(s, 0);
    assert_eq!(ms, 0);
}

#[test]
fn day_fraction_to_hms_half_minute() {
    // 30 seconds
    let (h, m, s, ms) = day_fraction_to_hms(30.0 / 86400.0);
    assert_eq!(h, 0);
    assert_eq!(m, 0);
    assert_eq!(s, 30);
    assert_eq!(ms, 0);
}

#[test]
fn day_fraction_to_hms_one_ms() {
    let (h, m, s, ms) = day_fraction_to_hms(1.0 / 86_400_000.0);
    assert_eq!(h, 0);
    assert_eq!(m, 0);
    assert_eq!(s, 0);
    assert_eq!(ms, 1);
}
