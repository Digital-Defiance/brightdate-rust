//! Integration tests for constants — ported from constants.spec.ts

#[allow(deprecated)]
use brightdate::constants::{
    CURRENT_TAI_UTC_OFFSET, DEFAULT_PRECISION, J2000_UNIX_MS_UTC, LEAP_SECOND_TABLE,
    MAX_PRECISION, METRIC_UNITS, MS_PER_DAY, SECONDS_PER_DAY, TAI_UTC_OFFSET_AT_J2000,
    TT_TAI_OFFSET_SECONDS,
};

// ─── J2000_UNIX_MS_UTC ────────────────────────────────────

#[test]
#[allow(deprecated)]
fn j2000_unix_ms_utc_value() {
    // J2000.0 = 2000-01-01T12:00:00.000 TT = 2000-01-01T11:58:55.816 UTC
    // (Old library used 946_728_000_000.0 which was J2000.0 in TT misread as
    //  UTC, off by 64.184 s. Corrected in v1.0.)
    assert_eq!(J2000_UNIX_MS_UTC, 946_727_935_816.0);
}

#[test]
#[allow(deprecated)]
fn j2000_unix_ms_utc_is_finite() {
    assert!(J2000_UNIX_MS_UTC.is_finite());
}

// ─── MS_PER_DAY ──────────────────────────────────────────────────────────────

#[test]
fn ms_per_day_value() {
    assert_eq!(MS_PER_DAY, 86_400_000.0);
}

#[test]
fn ms_per_day_is_86400_times_1000() {
    assert!((MS_PER_DAY - 86_400.0 * 1000.0).abs() < 1e-9);
}

// ─── SECONDS_PER_DAY ─────────────────────────────────────────────────────────

#[test]
fn seconds_per_day_value() {
    assert_eq!(SECONDS_PER_DAY, 86_400.0);
}

#[test]
fn seconds_per_day_times_1000_equals_ms_per_day() {
    assert!((SECONDS_PER_DAY * 1000.0 - MS_PER_DAY).abs() < 1e-9);
}

// ─── TAI_UTC_OFFSET_AT_J2000 ─────────────────────────────────────────────────

#[test]
fn tai_utc_offset_at_j2000_value() {
    assert_eq!(TAI_UTC_OFFSET_AT_J2000, 32);
}

// ─── TT_TAI_OFFSET_SECONDS ───────────────────────────────────────────────────

#[test]
fn tt_tai_offset_seconds_value() {
    assert!((TT_TAI_OFFSET_SECONDS - 32.184).abs() < 1e-10);
}

// ─── DEFAULT_PRECISION ───────────────────────────────────────────────────────

#[test]
fn default_precision_value() {
    assert_eq!(DEFAULT_PRECISION, 5);
}

// ─── MAX_PRECISION ───────────────────────────────────────────────────────────

#[test]
fn max_precision_value() {
    assert_eq!(MAX_PRECISION, 12);
}

#[test]
fn max_precision_greater_than_default() {
    assert!(MAX_PRECISION > DEFAULT_PRECISION);
}

// ─── CURRENT_TAI_UTC_OFFSET ──────────────────────────────────────────────────

#[test]
fn current_tai_utc_offset_value() {
    assert_eq!(CURRENT_TAI_UTC_OFFSET, 37);
}

#[test]
fn current_tai_utc_offset_matches_last_table_entry() {
    let last = LEAP_SECOND_TABLE.last().expect("table is non-empty");
    assert_eq!(CURRENT_TAI_UTC_OFFSET, last.1);
}

// ─── LEAP_SECOND_TABLE ───────────────────────────────────────────────────────

#[test]
fn leap_second_table_has_28_entries() {
    assert_eq!(LEAP_SECOND_TABLE.len(), 28);
}

#[test]
fn leap_second_table_first_entry() {
    assert_eq!(LEAP_SECOND_TABLE[0], (63_072_000, 10));
}

#[test]
fn leap_second_table_last_entry() {
    assert_eq!(LEAP_SECOND_TABLE[27], (1_483_228_800, 37));
}

#[test]
fn leap_second_table_timestamps_monotonically_increasing() {
    for i in 1..LEAP_SECOND_TABLE.len() {
        assert!(
            LEAP_SECOND_TABLE[i].0 > LEAP_SECOND_TABLE[i - 1].0,
            "timestamp at index {i} not > previous"
        );
    }
}

#[test]
fn leap_second_table_offsets_monotonically_increasing() {
    for i in 1..LEAP_SECOND_TABLE.len() {
        assert!(
            LEAP_SECOND_TABLE[i].1 > LEAP_SECOND_TABLE[i - 1].1,
            "offset at index {i} not > previous"
        );
    }
}

#[test]
fn leap_second_table_each_offset_increments_by_one() {
    for i in 1..LEAP_SECOND_TABLE.len() {
        assert_eq!(
            LEAP_SECOND_TABLE[i].1 - LEAP_SECOND_TABLE[i - 1].1,
            1,
            "offset increment != 1 at index {i}"
        );
    }
}

// ─── METRIC_UNITS ────────────────────────────────────────────────────────────

fn metric_unit(name: &str) -> f64 {
    METRIC_UNITS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, v)| *v)
        .unwrap_or_else(|| panic!("metric unit '{name}' not found"))
}

#[test]
fn metric_units_milliday_is_0_001() {
    assert!((metric_unit("milliday") - 0.001).abs() < 1e-15);
}

#[test]
fn metric_units_microday_is_0_000001() {
    assert!((metric_unit("microday") - 0.000_001).abs() < 1e-15);
}

#[test]
fn metric_units_nanoday_is_0_000000001() {
    assert!((metric_unit("nanoday") - 0.000_000_001).abs() < 1e-15);
}

#[test]
fn metric_units_milliday_equals_1000_microdays() {
    let milli = metric_unit("milliday");
    let micro = metric_unit("microday");
    assert!((milli - micro * 1000.0).abs() < 1e-15);
}

#[test]
fn metric_units_microday_equals_1000_nanodays() {
    let micro = metric_unit("microday");
    let nano = metric_unit("nanoday");
    assert!((micro - nano * 1000.0).abs() < 1e-15);
}
