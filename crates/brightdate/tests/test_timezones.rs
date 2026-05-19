// `local_time_of_day` and `is_daytime` are deprecated in 0.3.0 because they
// return BD-day-grid fractions, not civil-clock fractions. The tests below
// pin their behavior so consumers don't break silently before the eventual
// removal — they intentionally do *not* assert anything about civil clocks.
#![allow(deprecated)]

use brightdate::timezones::*;

const EPS: f64 = 1e-12;

// ── hours_to_fractional_days / fractional_days_to_hours ───────────────────────

#[test]
fn hours_to_fractional_days_24() {
    assert!((hours_to_fractional_days(24.0) - 1.0).abs() < EPS);
}

#[test]
fn hours_to_fractional_days_12() {
    assert!((hours_to_fractional_days(12.0) - 0.5).abs() < EPS);
}

#[test]
fn hours_to_fractional_days_1() {
    assert!((hours_to_fractional_days(1.0) - 1.0 / 24.0).abs() < EPS);
}

#[test]
fn hours_to_fractional_days_zero() {
    assert_eq!(hours_to_fractional_days(0.0), 0.0);
}

#[test]
fn hours_to_fractional_days_negative() {
    assert!((hours_to_fractional_days(-5.0) - (-5.0 / 24.0)).abs() < EPS);
}

#[test]
fn fractional_days_to_hours_one_day() {
    assert!((fractional_days_to_hours(1.0) - 24.0).abs() < EPS);
}

#[test]
fn fractional_days_to_hours_half_day() {
    assert!((fractional_days_to_hours(0.5) - 12.0).abs() < EPS);
}

#[test]
fn fractional_days_to_hours_zero() {
    assert_eq!(fractional_days_to_hours(0.0), 0.0);
}

#[test]
fn fractional_days_to_hours_negative() {
    assert!((fractional_days_to_hours(-1.0 / 24.0) + 1.0).abs() < EPS);
}

#[test]
fn hours_fractional_roundtrip() {
    let hours = 5.5_f64;
    let back = fractional_days_to_hours(hours_to_fractional_days(hours));
    assert!((back - hours).abs() < EPS);
}

// ── to_local_value / from_local_value ─────────────────────────────────────────

#[test]
fn to_local_value_positive_offset() {
    let utc = 9622.5;
    let offset = hours_to_fractional_days(5.0);
    let local = to_local_value(utc, offset);
    assert!((local - (utc + offset)).abs() < EPS);
}

#[test]
fn to_local_value_negative_offset() {
    let utc = 9622.5;
    let offset = hours_to_fractional_days(-8.0);
    let local = to_local_value(utc, offset);
    assert!(local < utc);
}

#[test]
fn from_local_value_reverses_offset() {
    let utc = 9622.5;
    let offset = hours_to_fractional_days(3.0);
    let local = to_local_value(utc, offset);
    let back = from_local_value(local, offset);
    assert!((back - utc).abs() < EPS);
}

#[test]
fn to_local_zero_offset_no_change() {
    let utc = 9622.5;
    assert!((to_local_value(utc, 0.0) - utc).abs() < EPS);
}

#[test]
fn from_local_zero_offset_no_change() {
    let local = 9622.5;
    assert!((from_local_value(local, 0.0) - local).abs() < EPS);
}

#[test]
fn offset_roundtrip_utc8() {
    let utc = 9622.0;
    let offset = hours_to_fractional_days(8.0);
    let local = to_local_value(utc, offset);
    let back = from_local_value(local, offset);
    assert!((back - utc).abs() < EPS);
}

#[test]
fn offset_roundtrip_utc_minus_5() {
    let utc = 5000.75;
    let offset = hours_to_fractional_days(-5.0);
    let local = to_local_value(utc, offset);
    let back = from_local_value(local, offset);
    assert!((back - utc).abs() < EPS);
}

// ── get_timezone_offset ───────────────────────────────────────────────────────

#[test]
fn get_timezone_offset_utc0() {
    let off = get_timezone_offset("UTC+0").unwrap();
    assert_eq!(off, 0.0);
}

#[test]
fn get_timezone_offset_utc_plus_1() {
    let off = get_timezone_offset("UTC+1").unwrap();
    assert!((off - 1.0 / 24.0).abs() < EPS);
}

#[test]
fn get_timezone_offset_utc_minus_5() {
    let off = get_timezone_offset("UTC-5").unwrap();
    assert!((off - (-5.0 / 24.0)).abs() < EPS);
}

#[test]
fn get_timezone_offset_utc_plus_8() {
    let off = get_timezone_offset("UTC+8").unwrap();
    assert!((off - 8.0 / 24.0).abs() < EPS);
}

#[test]
fn get_timezone_offset_utc_plus_5_5_india() {
    let off = get_timezone_offset("UTC+5.5").unwrap();
    assert!((off - 5.5 / 24.0).abs() < EPS);
}

#[test]
fn get_timezone_offset_utc_minus_12() {
    let off = get_timezone_offset("UTC-12").unwrap();
    assert!((off - (-12.0 / 24.0)).abs() < EPS);
}

#[test]
fn get_timezone_offset_utc_plus_14() {
    let off = get_timezone_offset("UTC+14").unwrap();
    assert!((off - 14.0 / 24.0).abs() < EPS);
}

#[test]
fn get_timezone_offset_unknown_returns_none() {
    assert!(get_timezone_offset("America/New_York").is_none());
}

#[test]
fn get_timezone_offset_case_sensitive_fails() {
    assert!(get_timezone_offset("utc+0").is_none());
}

#[test]
fn get_timezone_offset_empty_returns_none() {
    assert!(get_timezone_offset("").is_none());
}

#[test]
fn get_timezone_offset_utc_plus_9() {
    let off = get_timezone_offset("UTC+9").unwrap();
    assert!((off - 9.0 / 24.0).abs() < EPS);
}

#[test]
fn get_timezone_offset_utc_plus_9_5_australia() {
    let off = get_timezone_offset("UTC+9.5").unwrap();
    assert!((off - 9.5 / 24.0).abs() < EPS);
}

#[test]
fn get_timezone_offset_utc_plus_5_75_nepal() {
    let off = get_timezone_offset("UTC+5.75").unwrap();
    assert!((off - 5.75 / 24.0).abs() < EPS);
}

// ── format_with_timezone ──────────────────────────────────────────────────────

#[test]
fn format_with_timezone_known_contains_timezone_name() {
    let s = format_with_timezone(9622.5, "UTC+0", 5);
    assert!(s.contains("UTC+0"), "got: {s}");
}

#[test]
fn format_with_timezone_known_contains_both_values() {
    let s = format_with_timezone(9622.5, "UTC+0", 5);
    // UTC+0 has offset 0, so local == utc
    assert!(s.contains("9622.5"), "got: {s}");
}

#[test]
fn format_with_timezone_unknown_says_unknown() {
    let s = format_with_timezone(9622.5, "Fake/Zone", 5);
    assert!(s.contains("unknown timezone"), "got: {s}");
}

#[test]
fn format_with_timezone_unknown_contains_name() {
    let s = format_with_timezone(9622.5, "Fake/Zone", 5);
    assert!(s.contains("Fake/Zone"), "got: {s}");
}

#[test]
fn format_with_timezone_utc_plus_8_larger_local() {
    // UTC+8 adds a positive offset, so local value > utc value
    let utc = 9622.0;
    let s = format_with_timezone(utc, "UTC+8", 5);
    let offset = 8.0 / 24.0;
    let local = utc + offset;
    let local_str = format!("{:.5}", local);
    assert!(s.contains(&local_str[..8]), "got: {s}");
}

#[test]
fn format_with_timezone_utc_minus_5_smaller_local() {
    let utc = 9622.5;
    let offset = -5.0 / 24.0;
    let local = utc + offset;
    let s = format_with_timezone(utc, "UTC-5", 5);
    let local_str = format!("{:.5}", local);
    assert!(s.contains(&local_str[..8]), "got: {s}");
}

// ── local_time_of_day (deprecated; tests pin BD-day-grid behavior) ──────────

#[test]
fn local_time_of_day_zero_offset_epoch() {
    // BD 0.0 has BD-day fraction 0.0 (start of a BD day on the BD-day grid).
    // This is *not* UTC noon — see the deprecation note on the function.
    let frac = local_time_of_day(0.0, 0.0);
    assert!((frac - 0.0).abs() < 0.001);
}

#[test]
fn local_time_of_day_in_range() {
    let frac = local_time_of_day(9622.5, 0.0);
    assert!((0.0..=1.0).contains(&frac), "got: {frac}");
}

#[test]
fn local_time_of_day_offset_shifts() {
    let bd = 9622.0;
    let frac_utc = local_time_of_day(bd, 0.0);
    let frac_local = local_time_of_day(bd, hours_to_fractional_days(8.0));
    // The offset shifts the BD-day-grid fraction by 8/24, modulo 1.
    let expected_diff = 8.0 / 24.0;
    assert!((frac_local - frac_utc - expected_diff).abs() < 0.001);
}

// ── is_daytime (deprecated; tests pin BD-day-grid behavior) ─────────────────

#[test]
fn is_daytime_bd_day_fraction_half_is_true() {
    // is_daytime returns true when BD-day fraction is in [0.25, 0.75).
    // BD 9622.5 has fraction 0.5 on the BD-day grid → "daytime" on that grid.
    // This does NOT mean civil noon UTC.
    assert!(is_daytime(9622.5, 0.0));
}

#[test]
fn is_daytime_bd_day_fraction_zero_is_false() {
    // BD 9622.0 has BD-day fraction 0.0 — outside [0.25, 0.75).
    let bd = 9622.0;
    assert!(!is_daytime(bd, 0.0));
}

#[test]
fn is_daytime_returns_bool() {
    let _ = is_daytime(9622.5, hours_to_fractional_days(5.0));
}
