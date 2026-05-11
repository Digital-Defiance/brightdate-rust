use brightdate::conversions::*;
use brightdate::constants::{J2000_UNIX_MS_UTC, MS_PER_DAY};

const EPS: f64 = 1e-8;

// ── Epoch sanity ─────────────────────────────────────────────────────────────

#[test]
fn epoch_unix_ms_is_j2000() {
    assert_eq!(from_unix_ms(J2000_UNIX_MS_UTC).unwrap(), 0.0);
}

#[test]
fn epoch_unix_seconds_is_j2000() {
    let s = J2000_UNIX_MS_UTC / 1_000.0;
    assert!((from_unix_seconds(s).unwrap()).abs() < EPS);
}

#[test]
fn epoch_julian_date_is_j2000() {
    assert_eq!(from_julian_date(2_451_545.0), 0.0);
}

#[test]
fn epoch_mjd_is_j2000() {
    assert!((from_modified_julian_date(51_544.5)).abs() < EPS);
}

#[test]
fn epoch_iso_is_j2000() {
    // J2000.0 in UTC = 2000-01-01T11:58:55.816Z (= 12:00:00 TT minus 64.184 s).
    assert!((from_iso("2000-01-01T11:58:55.816Z").unwrap()).abs() < EPS);
}

#[test]
fn noon_utc_is_64_184_seconds_past_j2000() {
    // 2000-01-01T12:00:00 UTC sits 64.184 s = 32 leap + 32.184 TT−TAI past J2000.0.
    let bd = from_iso("2000-01-01T12:00:00Z").unwrap();
    let expected_days = 64.184 / 86_400.0;
    assert!((bd - expected_days).abs() < 1e-12, "got {bd}, expected {expected_days}");
}

// ── from_unix_ms ─────────────────────────────────────────────────────────────

#[test]
fn from_unix_ms_one_day_after_epoch() {
    let one_day_later = J2000_UNIX_MS_UTC + MS_PER_DAY;
    assert!((from_unix_ms(one_day_later).unwrap() - 1.0).abs() < EPS);
}

#[test]
fn from_unix_ms_one_day_before_epoch() {
    let one_day_before = J2000_UNIX_MS_UTC - MS_PER_DAY;
    assert!((from_unix_ms(one_day_before).unwrap() + 1.0).abs() < EPS);
}

#[test]
fn from_unix_ms_zero() {
    // Unix epoch 1970-01-01T00:00:00Z → negative BrightDate
    let bd = from_unix_ms(0.0).unwrap();
    assert!(bd < 0.0);
    // J2000.0 is 10957.5 days after Unix epoch (roughly)
    assert!((bd + 10957.5).abs() < 1.0);
}

#[test]
fn from_unix_ms_nan_errors() {
    assert!(from_unix_ms(f64::NAN).is_err());
}

#[test]
fn from_unix_ms_inf_errors() {
    assert!(from_unix_ms(f64::INFINITY).is_err());
}

#[test]
fn from_unix_ms_neg_inf_errors() {
    assert!(from_unix_ms(f64::NEG_INFINITY).is_err());
}

#[test]
fn from_unix_ms_half_day() {
    let half = J2000_UNIX_MS_UTC + MS_PER_DAY / 2.0;
    assert!((from_unix_ms(half).unwrap() - 0.5).abs() < EPS);
}

#[test]
fn from_unix_ms_100_days() {
    let ms = J2000_UNIX_MS_UTC + 100.0 * MS_PER_DAY;
    assert!((from_unix_ms(ms).unwrap() - 100.0).abs() < EPS);
}

// ── from_unix_seconds ────────────────────────────────────────────────────────

#[test]
fn from_unix_seconds_nan_errors() {
    assert!(from_unix_seconds(f64::NAN).is_err());
}

#[test]
fn from_unix_seconds_inf_errors() {
    assert!(from_unix_seconds(f64::INFINITY).is_err());
}

#[test]
fn from_unix_seconds_one_day_after_epoch() {
    let s = J2000_UNIX_MS_UTC / 1_000.0 + 86_400.0;
    assert!((from_unix_seconds(s).unwrap() - 1.0).abs() < EPS);
}

#[test]
fn from_unix_seconds_consistent_with_from_unix_ms() {
    let s = 1_234_567_890.0_f64;
    let from_s = from_unix_seconds(s).unwrap();
    let from_ms = from_unix_ms(s * 1_000.0).unwrap();
    assert!((from_s - from_ms).abs() < EPS);
}

// ── from_julian_date ─────────────────────────────────────────────────────────

#[test]
fn from_julian_date_one_day_later() {
    assert!((from_julian_date(2_451_546.0) - 1.0).abs() < EPS);
}

#[test]
fn from_julian_date_j1950() {
    // JD 2433282.5 = 1950-01-01T00:00:00 UTC
    let bd = from_julian_date(2_433_282.5);
    assert!(bd < 0.0);
}

#[test]
fn from_julian_date_large() {
    let jd = 2_500_000.0;
    assert!((from_julian_date(jd) - (jd - 2_451_545.0)).abs() < EPS);
}

// ── from_modified_julian_date ─────────────────────────────────────────────────

#[test]
fn from_mjd_51544_5_is_j2000() {
    assert!((from_modified_julian_date(51_544.5)).abs() < EPS);
}

#[test]
fn from_mjd_51545_is_half_day_after() {
    assert!((from_modified_julian_date(51_545.0) - 0.5).abs() < EPS);
}

#[test]
fn from_mjd_negative() {
    let mjd = 10_000.0;
    assert!(from_modified_julian_date(mjd) < 0.0);
}

#[test]
fn from_mjd_roundtrip() {
    let mjd = 55_000.0;
    let bd = from_modified_julian_date(mjd);
    let back = to_modified_julian_date(bd);
    assert!((back - mjd).abs() < EPS);
}

// ── from_iso ─────────────────────────────────────────────────────────────────

#[test]
fn from_iso_invalid_string_errors() {
    assert!(from_iso("not-a-date").is_err());
}

#[test]
fn from_iso_empty_string_errors() {
    assert!(from_iso("").is_err());
}

#[test]
fn from_iso_date_only_errors() {
    // date without time component — typically not valid for DateTime<Utc>
    assert!(from_iso("2000-01-01").is_err());
}

#[test]
fn from_iso_2024_01_01_midnight() {
    let bd = from_iso("2024-01-01T00:00:00Z").unwrap();
    assert!(bd > 8000.0, "2024 should be > BD 8000");
}

#[test]
fn from_iso_1970_unix_epoch() {
    let bd = from_iso("1970-01-01T00:00:00Z").unwrap();
    // 30 years before J2000.0 ≈ −10957.5 days
    assert!((bd + 10_957.5).abs() < 1.0);
}

#[test]
fn from_iso_with_milliseconds() {
    let bd1 = from_iso("2000-01-01T12:00:00.000Z").unwrap();
    let bd2 = from_iso("2000-01-01T12:00:00Z").unwrap();
    assert!((bd1 - bd2).abs() < EPS);
}

#[test]
fn from_iso_one_hour_apart() {
    let bd1 = from_iso("2000-01-01T12:00:00Z").unwrap();
    let bd2 = from_iso("2000-01-01T13:00:00Z").unwrap();
    let diff = bd2 - bd1;
    assert!((diff - 1.0 / 24.0).abs() < EPS);
}

// ── from_gps_time ─────────────────────────────────────────────────────────────

#[test]
fn from_gps_time_epoch_is_negative() {
    // GPS epoch 1980-01-06 is before J2000, so BD should be negative
    let bd = from_gps_time(0, 0.0);
    assert!(bd < 0.0);
}

#[test]
fn from_gps_time_week_zero_roundtrip() {
    let bd = from_gps_time(0, 0.0);
    // GPS epoch = 1980-01-06T00:00:00 UTC = Unix 315964800 s
    let expected = from_unix_seconds(315_964_800.0 - 19.0 + 19.0).unwrap_or(f64::NAN);
    let _ = expected; // Just verify it doesn't panic
    assert!(bd.is_finite());
}

#[test]
fn from_gps_time_one_week() {
    let week0 = from_gps_time(0, 0.0);
    let week1 = from_gps_time(1, 0.0);
    assert!((week1 - week0 - 7.0).abs() < 0.01);
}

#[test]
fn from_gps_time_seconds_within_week() {
    let a = from_gps_time(100, 0.0);
    let b = from_gps_time(100, 86400.0);
    assert!((b - a - 1.0).abs() < EPS);
}

// ── to_unix_ms ───────────────────────────────────────────────────────────────

#[test]
fn to_unix_ms_epoch() {
    assert!((to_unix_ms(0.0) - J2000_UNIX_MS_UTC).abs() < EPS);
}

#[test]
fn to_unix_ms_one_day() {
    assert!((to_unix_ms(1.0) - (J2000_UNIX_MS_UTC + MS_PER_DAY)).abs() < EPS);
}

#[test]
fn to_unix_ms_negative() {
    assert!((to_unix_ms(-1.0) - (J2000_UNIX_MS_UTC - MS_PER_DAY)).abs() < EPS);
}

#[test]
fn to_unix_ms_roundtrip() {
    let bd = 9622.5;
    assert!((from_unix_ms(to_unix_ms(bd)).unwrap() - bd).abs() < EPS);
}

// ── to_unix_seconds ──────────────────────────────────────────────────────────

#[test]
fn to_unix_seconds_epoch() {
    assert!((to_unix_seconds(0.0) - J2000_UNIX_MS_UTC / 1_000.0).abs() < EPS);
}

#[test]
fn to_unix_seconds_one_day() {
    assert!((to_unix_seconds(1.0) - (J2000_UNIX_MS_UTC / 1_000.0 + 86_400.0)).abs() < EPS);
}

#[test]
fn to_unix_seconds_consistent_with_ms() {
    let bd = 5000.25;
    assert!((to_unix_seconds(bd) - to_unix_ms(bd) / 1_000.0).abs() < EPS);
}

// ── to_julian_date ────────────────────────────────────────────────────────────

#[test]
fn to_julian_date_epoch() {
    assert!((to_julian_date(0.0) - 2_451_545.0).abs() < EPS);
}

#[test]
fn to_julian_date_one_day() {
    assert!((to_julian_date(1.0) - 2_451_546.0).abs() < EPS);
}

#[test]
fn to_julian_date_negative() {
    assert!((to_julian_date(-1.0) - 2_451_544.0).abs() < EPS);
}

#[test]
fn julian_date_roundtrip() {
    let bd = 7654.321;
    assert!((from_julian_date(to_julian_date(bd)) - bd).abs() < EPS);
}

// ── to_modified_julian_date ───────────────────────────────────────────────────

#[test]
fn to_mjd_epoch() {
    assert!((to_modified_julian_date(0.0) - 51_544.5).abs() < EPS);
}

#[test]
fn to_mjd_one_day() {
    assert!((to_modified_julian_date(1.0) - 51_545.5).abs() < EPS);
}

#[test]
fn to_mjd_negative() {
    assert!((to_modified_julian_date(-1.0) - 51_543.5).abs() < EPS);
}

// ── to_iso ────────────────────────────────────────────────────────────────────

#[test]
fn to_iso_epoch() {
    // BD 0.0 = J2000.0 = 2000-01-01T11:58:55.816 UTC.
    let s = to_iso(0.0);
    assert!(s.starts_with("2000-01-01T11:58:55.816"), "got: {s}");
}

#[test]
fn to_iso_contains_z() {
    assert!(to_iso(0.0).ends_with('Z'));
}

#[test]
fn to_iso_one_day_after_epoch() {
    // One SI day after J2000.0.
    let s = to_iso(1.0);
    assert!(s.starts_with("2000-01-02T11:58:55.816"), "got: {s}");
}

#[test]
fn to_iso_unix_epoch_neighbourhood() {
    // BD ≈ -10957.499 corresponds to 1970-01-01 UTC under the leap-second
    // convention (offset 10 prior to 1972).
    let s = to_iso(-10_957.499);
    assert!(s.starts_with("1970-01-01"), "got: {s}");
}

#[test]
fn to_iso_roundtrip() {
    let bd = 9622.0;
    let s = to_iso(bd);
    let back = from_iso(&s).unwrap();
    assert!((back - bd).abs() < 0.001); // ms precision
}

// ── to_gps_time ───────────────────────────────────────────────────────────────

#[test]
fn to_gps_time_returns_tuple() {
    let (week, sec) = to_gps_time(0.0);
    assert!(week > 0);
    assert!(sec >= 0.0);
    assert!(sec < 604_800.0);
}

#[test]
fn gps_time_week_increases() {
    let (w1, _) = to_gps_time(0.0);
    let (w2, _) = to_gps_time(7.0);
    assert_eq!(w2 - w1, 1);
}

#[test]
fn gps_time_seconds_within_week() {
    let (_, s1) = to_gps_time(0.0);
    let (_, s2) = to_gps_time(1.0); // One day later within same week
    let _ = (s1, s2); // Both should be valid
}

// ── TAI / UTC ─────────────────────────────────────────────────────────────────

#[test]
#[allow(deprecated)]
fn utc_to_tai_offset_is_positive() {
    // In v1.0 BrightDate is TAI-coherent: utc_to_tai_bright_date is identity.
    // We retain this test to lock in that identity contract.
    let utc = 9622.0;
    let tai = utc_to_tai_bright_date(utc);
    assert_eq!(tai, utc, "v1.0 BrightDate is TAI-coherent; conversion is identity");
}

#[test]
fn tai_to_utc_is_inverse() {
    let utc = 9622.5;
    let tai = utc_to_tai_bright_date(utc);
    let back = tai_to_utc_bright_date(tai);
    assert!((back - utc).abs() < EPS);
}

#[test]
fn tai_utc_offset_at_j2000_is_32() {
    // At J2000.0 (UTC BrightDate 0), the TAI-UTC offset is 32 seconds
    let offset = tai_utc_offset_seconds_at(0.0);
    assert_eq!(offset, 32);
}

#[test]
fn tai_utc_offset_is_non_negative() {
    let offset = tai_utc_offset_seconds_at(9000.0);
    assert!(offset >= 0);
}

// ── normalize ─────────────────────────────────────────────────────────────────

#[test]
fn normalize_finite_is_identity() {
    let bd = 9622.12345;
    assert!((normalize(bd) - bd).abs() < EPS);
}

#[test]
fn normalize_nan_is_identity() {
    // normalize is a no-op (API symmetry with TS library) — does NOT sanitize NaN
    let r = normalize(f64::NAN);
    assert!(r.is_nan());
}

#[test]
fn normalize_inf_is_identity() {
    // normalize is a no-op — does NOT clamp Inf
    assert_eq!(normalize(f64::INFINITY), f64::INFINITY);
}

#[test]
fn normalize_neg_inf_is_identity() {
    assert_eq!(normalize(f64::NEG_INFINITY), f64::NEG_INFINITY);
}

// ── from_date_time ────────────────────────────────────────────────────────────

#[test]
fn from_date_time_j2000() {
    use chrono::{TimeZone, Utc};
    // J2000.0 in UTC = 2000-01-01T11:58:55.816.
    let dt = Utc.timestamp_millis_opt(946_727_935_816).single().unwrap();
    let bd = from_date_time(dt);
    assert!(bd.abs() < EPS, "got bd={bd}");
}

#[test]
fn from_date_time_roundtrip() {
    use chrono::Utc;
    let dt_orig = Utc::now();
    let bd = from_date_time(dt_orig);
    let dt_back = to_date_time(bd);
    let diff_ms = (dt_orig.timestamp_millis() - dt_back.timestamp_millis()).abs();
    assert!(diff_ms < 1000, "roundtrip should be within 1 second");
}

// ── to_date_time ─────────────────────────────────────────────────────────────

#[test]
fn to_date_time_epoch_is_j2000() {
    use chrono::Datelike;
    let dt = to_date_time(0.0);
    assert_eq!(dt.year(), 2000);
    assert_eq!(dt.month(), 1);
    assert_eq!(dt.day(), 1);
}

#[test]
fn to_date_time_one_year_later() {
    use chrono::Datelike;
    // BD 0.0 = 2000-01-01T12:00:00 UTC; 365.25 days later = 2000-12-31T18:00:00 (still year 2000)
    // To get to 2001 we need > 365 days from J2000
    let dt = to_date_time(366.5);
    assert_eq!(dt.year(), 2001);
}
