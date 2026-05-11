//! Integration tests for leap_seconds — ported from leapSeconds.spec.ts

use brightdate::leap_seconds::{
    get_tai_utc_offset, get_tai_utc_offset_at_j2000, is_during_leap_second, leap_seconds_between,
    tai_to_utc, utc_to_tai,
};

// ─── getTaiUtcOffset ─────────────────────────────────────────────────────────

#[test]
fn tai_utc_offset_before_any_leap_second_returns_10() {
    // Unix second 0 (1970-01-01) is before the first table entry (1972-01-01).
    // The implementation returns 10 as a pre-1972 approximation.
    let offset = get_tai_utc_offset(0);
    assert_eq!(offset, 10);
}

#[test]
fn tai_utc_offset_at_1972() {
    // 1972-01-01T00:00:00 UTC = 63_072_000 Unix seconds
    let offset = get_tai_utc_offset(63_072_000);
    assert_eq!(offset, 10);
}

#[test]
fn tai_utc_offset_in_2017() {
    // After 2017-01-01T00:00:00 UTC = 1_483_228_800 → offset 37
    let offset = get_tai_utc_offset(1_483_228_800);
    assert_eq!(offset, 37);
}

#[test]
fn tai_utc_offset_current_is_37() {
    // After most recent leap second (2017), offset remains 37
    let offset = get_tai_utc_offset(1_700_000_000);
    assert_eq!(offset, 37);
}

// ─── getTaiUtcOffsetAtJ2000 ──────────────────────────────────────────────────

#[test]
fn tai_utc_offset_at_j2000_returns_32() {
    assert_eq!(get_tai_utc_offset_at_j2000(), 32);
}

// ─── utcToTai ────────────────────────────────────────────────────────────────

#[test]
fn utc_to_tai_at_j2000() {
    // J2000.0 = 946_728_000 UTC seconds → TAI = UTC + 32
    let tai = utc_to_tai(946_728_000);
    assert_eq!(tai, 946_728_000 + 32);
}

#[test]
fn utc_to_tai_at_2017() {
    let utc = 1_483_228_800_i64;
    let tai = utc_to_tai(utc);
    assert_eq!(tai, utc + 37);
}

// ─── taiToUtc ────────────────────────────────────────────────────────────────

#[test]
fn tai_to_utc_roundtrip_j2000() {
    let utc = 946_728_000_i64;
    let tai = utc_to_tai(utc);
    assert_eq!(tai_to_utc(tai), utc);
}

#[test]
fn tai_to_utc_roundtrip_2017() {
    let utc = 1_483_228_800_i64;
    let tai = utc_to_tai(utc);
    assert_eq!(tai_to_utc(tai), utc);
}

#[test]
fn tai_to_utc_at_j2000() {
    let tai = 946_728_000_i64 + 32;
    assert_eq!(tai_to_utc(tai), 946_728_000);
}

// ─── isDuringLeapSecond ──────────────────────────────────────────────────────

#[test]
fn is_during_leap_second_false_for_normal_time() {
    assert!(!is_during_leap_second(946_728_000));
}

#[test]
fn is_during_leap_second_false_for_2017_entry() {
    // The entry itself (1_483_228_800) is the new UTC second AFTER the leap second
    assert!(!is_during_leap_second(1_483_228_800));
}

// ─── leapSecondsBetween ──────────────────────────────────────────────────────

#[test]
fn leap_seconds_between_1972_and_2017() {
    // 1972-01-01 = 63_072_000, 2017-01-01 = 1_483_228_800
    // Total additional leap seconds: offset went from 10 to 37 = 27 added seconds
    let n = leap_seconds_between(63_072_000, 1_483_228_800);
    assert_eq!(n, 27);
}

#[test]
fn leap_seconds_between_same_epoch() {
    assert_eq!(leap_seconds_between(946_728_000, 946_728_000), 0);
}

#[test]
fn leap_seconds_between_symmetric() {
    let a = leap_seconds_between(63_072_000, 1_483_228_800);
    let b = leap_seconds_between(1_483_228_800, 63_072_000);
    assert_eq!(a, b.abs());
}

#[test]
fn leap_seconds_between_no_change_period() {
    // After the last leap second (2017) no more were added
    let n = leap_seconds_between(1_483_228_800, 1_700_000_000);
    assert_eq!(n, 0);
}

#[test]
fn leap_seconds_between_one_second_span_1972() {
    // At 1972-01-01 (63_072_000), offset=10. Before the table, offset is also 10.
    // So there are 0 NEW leap seconds between 0 and 63_072_000 by the table's own accounting.
    let n = leap_seconds_between(0, 63_072_000);
    assert_eq!(n, 0);
}

// ─── historical leap second boundary values ───────────────────────────────────

#[test]
fn tai_utc_offset_at_1972_jul_01() {
    // 1972-07-01 = Unix s 78_796_800 → offset 11
    let offset = get_tai_utc_offset(78_796_800);
    assert_eq!(offset, 11);
}

#[test]
fn tai_utc_offset_at_1973_jan_01() {
    let offset = get_tai_utc_offset(94_694_400);
    assert_eq!(offset, 12);
}

#[test]
fn tai_utc_offset_at_1974_jan_01() {
    let offset = get_tai_utc_offset(126_230_400);
    assert_eq!(offset, 13);
}

#[test]
fn tai_utc_offset_at_1980_jan_01() {
    let offset = get_tai_utc_offset(315_532_800);
    assert_eq!(offset, 19);
}

#[test]
fn tai_utc_offset_at_1990_jan_01() {
    let offset = get_tai_utc_offset(631_152_000);
    assert_eq!(offset, 25);
}

#[test]
fn tai_utc_offset_at_1996_jan_01() {
    let offset = get_tai_utc_offset(820_454_400);
    assert_eq!(offset, 30);
}

#[test]
fn tai_utc_offset_at_1999_jan_01() {
    let offset = get_tai_utc_offset(915_148_800);
    assert_eq!(offset, 32);
}

#[test]
fn tai_utc_offset_at_2006_jan_01() {
    let offset = get_tai_utc_offset(1_136_073_600);
    assert_eq!(offset, 33);
}

#[test]
fn tai_utc_offset_at_2009_jan_01() {
    let offset = get_tai_utc_offset(1_230_768_000);
    assert_eq!(offset, 34);
}

#[test]
fn tai_utc_offset_at_2012_jul_01() {
    let offset = get_tai_utc_offset(1_341_100_800);
    assert_eq!(offset, 35);
}

#[test]
fn tai_utc_offset_at_2015_jul_01() {
    let offset = get_tai_utc_offset(1_435_708_800);
    assert_eq!(offset, 36);
}

// ─── offset one second BEFORE each boundary ───────────────────────────────────

#[test]
fn tai_utc_offset_just_before_1972_jul() {
    // 1 second before 1972-07-01
    let offset = get_tai_utc_offset(78_796_800 - 1);
    assert_eq!(offset, 10);
}

#[test]
fn tai_utc_offset_just_before_2006() {
    let offset = get_tai_utc_offset(1_136_073_600 - 1);
    assert_eq!(offset, 32);
}

#[test]
fn tai_utc_offset_just_before_2017() {
    let offset = get_tai_utc_offset(1_483_228_800 - 1);
    assert_eq!(offset, 36);
}

// ─── utcToTai / taiToUtc additional ──────────────────────────────────────────

#[test]
fn utc_to_tai_1990() {
    let utc = 631_152_000_i64;
    let tai = utc_to_tai(utc);
    assert_eq!(tai, utc + 25);
}

#[test]
fn utc_to_tai_1996() {
    let utc = 820_454_400_i64;
    assert_eq!(utc_to_tai(utc), utc + 30);
}

#[test]
fn utc_to_tai_2009() {
    let utc = 1_230_768_000_i64;
    assert_eq!(utc_to_tai(utc), utc + 34);
}

#[test]
fn tai_to_utc_roundtrip_1990() {
    // 1990-01-01 01:00:00 UTC (1 hour past boundary, no ambiguity)
    let utc = 631_155_600_i64;
    assert_eq!(tai_to_utc(utc_to_tai(utc)), utc);
}

#[test]
fn tai_to_utc_roundtrip_2009() {
    // 2009-01-01 01:00:00 UTC (1 hour past boundary)
    let utc = 1_230_771_600_i64;
    assert_eq!(tai_to_utc(utc_to_tai(utc)), utc);
}

#[test]
fn tai_to_utc_roundtrip_2015() {
    // 2015-07-01 01:00:00 UTC (1 hour past boundary)
    let utc = 1_435_712_400_i64;
    assert_eq!(tai_to_utc(utc_to_tai(utc)), utc);
}

// ─── leapSecondsBetween additional ───────────────────────────────────────────

#[test]
fn leap_seconds_between_1972_and_1990() {
    // offset at 1972-01-01 = 10, at 1990-01-01 = 25 → diff = 15
    let n = leap_seconds_between(63_072_000, 631_152_000);
    assert_eq!(n, 15);
}

#[test]
fn leap_seconds_between_1990_and_2000() {
    // offset at 1990-01-01 = 25, at 1999-01-01 = 32 → diff = 7
    let n = leap_seconds_between(631_152_000, 946_728_000);
    assert_eq!(n, 7);
}

#[test]
fn leap_seconds_between_2017_and_future() {
    // No new leap seconds after 2017
    let n = leap_seconds_between(1_483_228_800, 2_000_000_000);
    assert_eq!(n, 0);
}

#[test]
fn leap_seconds_between_1980_and_1990() {
    // 1980-01-01 = 315_532_800 → offset 19; 1990-01-01 = 631_152_000 → offset 25 → diff 6
    let n = leap_seconds_between(315_532_800, 631_152_000);
    assert_eq!(n, 6);
}

// ─── isDuringLeapSecond boundary ─────────────────────────────────────────────

#[test]
fn is_during_leap_second_just_before_1972_jul() {
    // At 78_796_800 - 1 = 78_796_799, next second has higher offset
    assert!(is_during_leap_second(78_796_800 - 1));
}

#[test]
fn is_during_leap_second_at_2017_boundary() {
    // At 1_483_228_800 itself, the next second already has the new offset (37),
    // but the current second already IS 37 → false
    assert!(!is_during_leap_second(1_483_228_800));
}

#[test]
fn is_during_leap_second_just_before_1990() {
    assert!(is_during_leap_second(631_152_000 - 1));
}

#[test]
fn is_during_leap_second_just_before_2009() {
    assert!(is_during_leap_second(1_230_768_000 - 1));
}
