//! Conversions between BrightDate values and other time representations.
//!
//! # Timescale model
//!
//! `BrightDate` is **decimal SI days since J2000.0** (`2000-01-01T12:00:00 TT`).
//! Its underlying clock ticks uniformly with TAI/TT — there are *no* leap-
//! second discontinuities on the BrightDate timeline. Leap seconds appear
//! only when converting to/from UTC labels (`Unix ms`, ISO strings).
//!
//! ## Reference identities (all exact)
//!
//! - `BrightDate 0.0 ≡ J2000.0`
//! - `BrightDate + 2_451_545.0 = Julian Date` (TT)
//! - `BrightDate + 51_544.5 = Modified Julian Date` (TT)
//!
//! ## Reference identities (leap-second dependent)
//!
//! - `2000-01-01T11:58:55.816 UTC` (Unix ms `946_727_935_816`) → `BD 0.0`
//! - `2000-01-01T12:00:00.000 UTC` (Unix ms `946_728_000_000`) → `BD 0.000_742_870_370_370…`
//!   (i.e. 64.184 seconds = 32 leap + 32.184 TT−TAI past J2000.0)

use crate::constants::{J2000_JD, J2000_MJD, J2000_TAI_UNIX_S, SECONDS_PER_DAY};
use crate::leap_seconds::{get_tai_utc_offset, tai_to_utc_full};
use crate::types::{BrightDateError, BrightDateValue};
use chrono::{DateTime, TimeZone, Utc};

// ── From other timescales → BrightDate ─────────────────────────────────────

/// Convert a Unix timestamp (ms) — a UTC label — to a BrightDate value.
///
/// The input is interpreted as Unix milliseconds since `1970-01-01T00:00:00 UTC`
/// using the POSIX convention (leap seconds collapse onto their boundary).
/// We apply the leap-second table to obtain the corresponding TAI instant,
/// then express that as decimal days since J2000.0.
pub fn from_unix_ms(ms: f64) -> Result<BrightDateValue, BrightDateError> {
    if !ms.is_finite() {
        return Err(BrightDateError::InvalidInput(format!(
            "expected finite Unix ms, got {ms}"
        )));
    }
    let utc_s = ms / 1_000.0;
    let offset = get_tai_utc_offset(utc_s.floor() as i64) as f64;
    let tai_s_since_unix = utc_s + offset;
    Ok((tai_s_since_unix - J2000_TAI_UNIX_S) / SECONDS_PER_DAY)
}

/// Convert a Unix timestamp (seconds) to a BrightDate value.
pub fn from_unix_seconds(s: f64) -> Result<BrightDateValue, BrightDateError> {
    if !s.is_finite() {
        return Err(BrightDateError::InvalidInput(format!(
            "expected finite Unix seconds, got {s}"
        )));
    }
    from_unix_ms(s * 1_000.0)
}

/// Convert a `chrono::DateTime<Utc>` to a BrightDate value.
pub fn from_date_time(dt: DateTime<Utc>) -> BrightDateValue {
    from_unix_ms(dt.timestamp_millis() as f64).unwrap_or(f64::NAN)
}

/// Convert a Julian Date (TT) to a BrightDate value. **Exact** — `JD 2451545.0` → `BD 0.0`.
pub fn from_julian_date(jd: f64) -> BrightDateValue {
    jd - J2000_JD
}

/// Convert a Modified Julian Date (TT) to a BrightDate value. **Exact** — `MJD 51544.5` → `BD 0.0`.
pub fn from_modified_julian_date(mjd: f64) -> BrightDateValue {
    mjd - J2000_MJD
}

/// Parse an ISO 8601 string to a BrightDate value.
pub fn from_iso(s: &str) -> Result<BrightDateValue, BrightDateError> {
    let dt = s.parse::<DateTime<Utc>>().map_err(|e| {
        BrightDateError::ParseError(format!("invalid ISO 8601 \"{s}\": {e}"))
    })?;
    Ok(from_date_time(dt))
}

/// Convert GPS time (week + seconds) to a BrightDate value.
///
/// GPS epoch: `1980-01-06T00:00:00 UTC` (Unix s = 315_964_800). GPS time
/// runs on TAI, offset by a constant 19 s: `GPS = TAI − 19 s`.
pub fn from_gps_time(gps_week: u32, gps_seconds: f64) -> BrightDateValue {
    const GPS_EPOCH_UNIX_TAI: i64 = 315_964_800 + 19; // = 315_964_819 TAI Unix s
    const SECONDS_PER_WEEK: f64 = 604_800.0;
    let gps_elapsed_s = gps_week as f64 * SECONDS_PER_WEEK + gps_seconds;
    let tai_s_since_unix = GPS_EPOCH_UNIX_TAI as f64 + gps_elapsed_s;
    (tai_s_since_unix - J2000_TAI_UNIX_S) / SECONDS_PER_DAY
}

// ── BrightDate → other timescales ──────────────────────────────────────────

/// Convert a BrightDate to TAI seconds since the Unix epoch (TAI-labelled).
#[inline]
fn bd_to_tai_unix_s(bd: BrightDateValue) -> f64 {
    bd * SECONDS_PER_DAY + J2000_TAI_UNIX_S
}

/// Convert a BrightDate value to a Unix timestamp (ms) — a UTC label.
///
/// For instants that fall on a leap second, this returns the Unix ms of the
/// *repeated* second (NTP convention). Use [`to_iso_with_leap`] or
/// `tai_to_utc_full` for precise leap-second handling.
pub fn to_unix_ms(bd: BrightDateValue) -> f64 {
    let tai_s = bd_to_tai_unix_s(bd);
    let conv = tai_to_utc_full(tai_s.floor() as i64);
    // Preserve sub-second precision: the fractional TAI second equals the
    // fractional UTC second (a leap second has no fractional displacement).
    let frac = tai_s - tai_s.floor();
    (conv.utc_unix_seconds as f64 + frac) * 1_000.0
}

/// Convert a BrightDate value to a Unix timestamp (seconds).
pub fn to_unix_seconds(bd: BrightDateValue) -> f64 {
    to_unix_ms(bd) / 1_000.0
}

/// Convert a BrightDate value to a `chrono::DateTime<Utc>`.
pub fn to_date_time(bd: BrightDateValue) -> DateTime<Utc> {
    let ms = to_unix_ms(bd) as i64;
    Utc.timestamp_millis_opt(ms).single().unwrap_or(DateTime::<Utc>::UNIX_EPOCH)
}

/// Convert a BrightDate value to a Julian Date (TT). **Exact.**
pub fn to_julian_date(bd: BrightDateValue) -> f64 {
    bd + J2000_JD
}

/// Convert a BrightDate value to a Modified Julian Date (TT). **Exact.**
pub fn to_modified_julian_date(bd: BrightDateValue) -> f64 {
    bd + J2000_MJD
}

/// Convert a BrightDate to an ISO 8601 string. Leap-second instants emit `:60`.
pub fn to_iso(bd: BrightDateValue) -> String {
    let tai_s = bd_to_tai_unix_s(bd);
    let tai_s_int = tai_s.floor() as i64;
    let frac = tai_s - tai_s.floor();
    let conv = tai_to_utc_full(tai_s_int);

    if conv.is_leap_second {
        // Render as YYYY-MM-DDTHH:MM:60.fffZ.
        // `utc_unix_seconds` is `boundary - 1` (a 23:59:59 second).
        let dt = Utc
            .timestamp_opt(conv.utc_unix_seconds, 0)
            .single()
            .unwrap_or(DateTime::<Utc>::UNIX_EPOCH);
        let millis = (frac * 1000.0).round() as i64;
        return format!(
            "{}T{}:{}:60.{:03}Z",
            dt.format("%Y-%m-%d"),
            dt.format("%H"),
            dt.format("%M"),
            millis.clamp(0, 999),
        );
    }

    let ms = (conv.utc_unix_seconds as f64 + frac) * 1_000.0;
    let dt = Utc
        .timestamp_millis_opt(ms as i64)
        .single()
        .unwrap_or(DateTime::<Utc>::UNIX_EPOCH);
    dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

/// Convert a BrightDate value to GPS time `(gps_week, gps_seconds)`.
pub fn to_gps_time(bd: BrightDateValue) -> (u32, f64) {
    const GPS_EPOCH_UNIX_TAI: f64 = 315_964_819.0; // 315_964_800 UTC + 19 (GPS-TAI offset constant)
    const SECONDS_PER_WEEK: f64 = 604_800.0;
    let tai_s = bd_to_tai_unix_s(bd);
    let gps_elapsed = (tai_s - GPS_EPOCH_UNIX_TAI).max(0.0);
    let week = (gps_elapsed / SECONDS_PER_WEEK).floor() as u32;
    let sec = gps_elapsed - (week as f64 * SECONDS_PER_WEEK);
    (week, sec)
}

// ── TAI ↔ UTC (legacy helpers) ─────────────────────────────────────────────

/// **Identity** in v1.0. BrightDate is TAI-coherent by construction; there
/// is no separate UTC-anchored BrightDate timeline. Retained for source-
/// compatibility with v0.x.
#[deprecated(since = "1.0.0", note = "BrightDate is TAI-coherent; this is now identity")]
pub fn utc_to_tai_bright_date(bd: BrightDateValue) -> BrightDateValue {
    bd
}

/// **Identity** in v1.0. See [`utc_to_tai_bright_date`].
#[deprecated(since = "1.0.0", note = "BrightDate is TAI-coherent; this is now identity")]
pub fn tai_to_utc_bright_date(bd: BrightDateValue) -> BrightDateValue {
    bd
}

/// Return the TAI − UTC offset (s) for the UTC label at the BrightDate
/// instant `bd`.
pub fn tai_utc_offset_seconds_at(bd: BrightDateValue) -> i32 {
    let tai_s = bd_to_tai_unix_s(bd);
    let utc_s = tai_to_utc_full(tai_s.floor() as i64).utc_unix_seconds;
    get_tai_utc_offset(utc_s)
}

/// Normalize a BrightDate value (no-op; here for API symmetry with TS library).
pub fn normalize(bd: BrightDateValue) -> BrightDateValue {
    bd
}

// Re-export for transitional callers that imported these from `conversions`.
pub use crate::leap_seconds::{tai_to_utc, TaiToUtc};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::J2000_UTC_UNIX_MS;

    #[test]
    fn j2000_unix_ms_maps_to_bd_zero() {
        let bd = from_unix_ms(J2000_UTC_UNIX_MS).unwrap();
        assert!(bd.abs() < 1e-9, "BD at J2000.0 should be 0, got {bd}");
    }

    #[test]
    fn noon_utc_maps_to_64_184_seconds_past_j2000() {
        let bd = from_unix_ms(946_728_000_000.0).unwrap();
        let expected = 64.184 / SECONDS_PER_DAY;
        assert!(
            (bd - expected).abs() < 1e-12,
            "noon UTC should be 64.184 s past J2000.0; got BD={bd}, expected {expected}"
        );
    }

    #[test]
    fn unix_ms_roundtrip() {
        let ms = 1_700_000_000_000.0; // 2023-11-14T22:13:20 UTC
        let bd = from_unix_ms(ms).unwrap();
        let back = to_unix_ms(bd);
        assert!((back - ms).abs() < 1.0, "roundtrip drift: {} ms", back - ms);
    }

    #[test]
    fn iso_roundtrip() {
        let iso = "2025-06-15T10:30:00.000Z";
        let bd = from_iso(iso).unwrap();
        let back = to_iso(bd);
        assert_eq!(back, iso);
    }

    #[test]
    fn julian_date_exact_at_j2000() {
        assert_eq!(from_julian_date(2_451_545.0), 0.0);
        assert_eq!(to_julian_date(0.0), 2_451_545.0);
    }

    #[test]
    fn modified_julian_date_exact_at_j2000() {
        assert_eq!(from_modified_julian_date(51_544.5), 0.0);
        assert_eq!(to_modified_julian_date(0.0), 51_544.5);
    }
}
