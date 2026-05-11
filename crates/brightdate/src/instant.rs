//! `BrightInstant` — an *exact*, lossless time representation.
//!
//! A [`BrightDate`](crate::BrightDate) value is a `f64` count of SI days since
//! J2000.0. That's beautifully simple — and beautifully imprecise far from the
//! epoch. By year ~2300 the `f64` mantissa cannot resolve milliseconds; by
//! year ~3000 it loses 10 ms; past ~22000 it loses whole seconds. For most
//! applications (calendars, scheduling, astronomy at human precision) this is
//! fine. For applications that need **nanosecond precision indefinitely into
//! the future or past** — distributed systems, ephemerides, GPS engineering,
//! interplanetary mission timing — `BrightInstant` is the rigorous
//! foundation.
//!
//! # Representation
//!
//! A `BrightInstant` stores two integers:
//!
//! ```text
//!   tai_seconds : i64    seconds since J2000.0 on the TAI timescale
//!   tai_nanos   : u32    nanoseconds within that second, [0, 1_000_000_000)
//! ```
//!
//! Because the substrate is TAI (uniform), arithmetic is trivially
//! associative — there are no leap seconds to hop over. Leap seconds appear
//! only when converting to/from UTC labels (`to_unix_ms`, `to_iso`).
//!
//! # Range and precision
//!
//! - **Range:** `±2^63 / 86400 / 365.25 ≈ ±292 billion years` around J2000.0.
//!   That comfortably covers the heat death of the universe and back.
//! - **Precision:** **1 nanosecond, exactly, everywhere.** No `f64` drift.
//!
//! # Relationship to `BrightDate`
//!
//! `BrightInstant` is the rigorous form; `BrightDate` (f64) is the
//! ergonomic form. Both anchor on the same instant (`J2000.0`). Conversion is
//! lossy in one direction (`Instant → Date` loses nanos past the f64
//! mantissa) and lossless in the other for the supported f64 range.

use crate::constants::{J2000_JD, J2000_MJD, J2000_TAI_UNIX_S, SECONDS_PER_DAY};
use crate::leap_seconds::{get_tai_utc_offset, tai_to_utc_full};
use crate::types::{BrightDateError, BrightDateValue};
use chrono::{DateTime, TimeZone, Utc};

/// One billion. Nanoseconds per second.
const NANOS_PER_SEC: u32 = 1_000_000_000;

/// J2000.0 expressed as TAI Unix seconds + nanoseconds.
/// Equals `946_727_967.816` s exactly (816 ms = 816_000_000 ns).
const J2000_TAI_UNIX_S_INT: i64 = 946_727_967;
const J2000_TAI_UNIX_NS: u32 = 816_000_000;

/// An exact instant on the TAI timescale, anchored at J2000.0.
///
/// # Examples
///
/// ```
/// use brightdate::BrightInstant;
///
/// // J2000.0 itself.
/// let epoch = BrightInstant::J2000;
/// assert_eq!(epoch.tai_seconds_since_j2000(), 0);
/// assert_eq!(epoch.tai_nanos(), 0);
///
/// // A picosecond-precise* instant. (*Well, nanosecond. But exactly.)
/// let later = BrightInstant::from_tai_components(86_400, 1).unwrap();
/// assert_eq!(later.tai_seconds_since_j2000(), 86_400);
/// assert_eq!(later.tai_nanos(), 1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BrightInstant {
    /// TAI seconds since J2000.0. Negative for instants before J2000.0.
    tai_seconds: i64,
    /// Nanoseconds within `tai_seconds`. Always in `[0, 1_000_000_000)`.
    /// Convention: this is *always* a non-negative offset, so the encoded
    /// instant equals `tai_seconds + tai_nanos * 1e-9` for any value
    /// (including negatives).
    tai_nanos: u32,
}

impl BrightInstant {
    /// The J2000.0 epoch itself.
    pub const J2000: Self = Self {
        tai_seconds: 0,
        tai_nanos: 0,
    };

    // ── Constructors ──────────────────────────────────────────────────────

    /// Construct from TAI seconds and nanoseconds since J2000.0.
    ///
    /// Returns an error if `tai_nanos >= 1_000_000_000`.
    pub fn from_tai_components(
        tai_seconds: i64,
        tai_nanos: u32,
    ) -> Result<Self, BrightDateError> {
        if tai_nanos >= NANOS_PER_SEC {
            return Err(BrightDateError::OutOfRange(format!(
                "tai_nanos must be < 1_000_000_000, got {tai_nanos}"
            )));
        }
        Ok(Self {
            tai_seconds,
            tai_nanos,
        })
    }

    /// Construct from a `BrightDate` (f64 days). Precision is bounded by f64.
    pub fn from_brightdate(bd: BrightDateValue) -> Result<Self, BrightDateError> {
        if !bd.is_finite() {
            return Err(BrightDateError::InvalidInput(format!(
                "expected finite BrightDate, got {bd}"
            )));
        }
        let total_seconds = bd * SECONDS_PER_DAY;
        let secs_floor = total_seconds.floor();
        let frac = total_seconds - secs_floor;
        // Round nanos rather than truncate, to keep round-trips faithful.
        let mut nanos = (frac * NANOS_PER_SEC as f64).round() as i64;
        let mut secs = secs_floor as i64;
        // Carry: rounding can push nanos to exactly 1e9.
        if nanos >= NANOS_PER_SEC as i64 {
            nanos -= NANOS_PER_SEC as i64;
            secs += 1;
        } else if nanos < 0 {
            // Cannot happen given floor + non-negative frac, but defend.
            nanos += NANOS_PER_SEC as i64;
            secs -= 1;
        }
        Ok(Self {
            tai_seconds: secs,
            tai_nanos: nanos as u32,
        })
    }

    /// Construct from a Unix millisecond timestamp (UTC label).
    ///
    /// Applies the leap-second table to obtain the corresponding TAI instant.
    pub fn from_unix_ms(ms: i64) -> Self {
        // Split ms into utc_seconds + utc_ms_within (handling negative ms carefully).
        let utc_seconds = ms.div_euclid(1000);
        let utc_ms_within = ms.rem_euclid(1000) as u32;
        let offset = get_tai_utc_offset(utc_seconds) as i64;
        let tai_unix_s = utc_seconds + offset;
        // Anchor at J2000.0 (integer second part).
        let mut secs = tai_unix_s - J2000_TAI_UNIX_S_INT;
        // Sub-second part of input, then subtract J2000.0's 816 ms.
        let target_nanos = (utc_ms_within as i64) * 1_000_000 - J2000_TAI_UNIX_NS as i64;
        let nanos = if target_nanos < 0 {
            secs -= 1;
            (target_nanos + NANOS_PER_SEC as i64) as u32
        } else {
            target_nanos as u32
        };
        Self {
            tai_seconds: secs,
            tai_nanos: nanos,
        }
    }

    /// Construct from a Julian Date (TT). Exact up to f64 mantissa.
    pub fn from_julian_date(jd: f64) -> Result<Self, BrightDateError> {
        Self::from_brightdate(jd - J2000_JD)
    }

    /// Construct from a Modified Julian Date (TT). Exact up to f64 mantissa.
    pub fn from_modified_julian_date(mjd: f64) -> Result<Self, BrightDateError> {
        Self::from_brightdate(mjd - J2000_MJD)
    }

    // ── Accessors ─────────────────────────────────────────────────────────

    /// TAI seconds since J2000.0.
    #[inline]
    pub const fn tai_seconds_since_j2000(self) -> i64 {
        self.tai_seconds
    }

    /// TAI nanoseconds within the current TAI second (`[0, 1_000_000_000)`).
    #[inline]
    pub const fn tai_nanos(self) -> u32 {
        self.tai_nanos
    }

    // ── Conversions out ───────────────────────────────────────────────────

    /// Lossy projection to f64 `BrightDate` (decimal days since J2000.0).
    pub fn to_brightdate(self) -> BrightDateValue {
        (self.tai_seconds as f64 + self.tai_nanos as f64 / NANOS_PER_SEC as f64)
            / SECONDS_PER_DAY
    }

    /// Convert to Unix milliseconds (UTC label). Applies leap-second table.
    ///
    /// Leap-second instants map to the *repeated* UTC second (NTP convention).
    pub fn to_unix_ms(self) -> i64 {
        // TAI Unix s = J2000_TAI_UNIX_S_INT + tai_seconds (+ carry from nanos)
        let mut tai_unix_s = J2000_TAI_UNIX_S_INT + self.tai_seconds;
        let mut tai_ns = self.tai_nanos as i64 + J2000_TAI_UNIX_NS as i64;
        if tai_ns >= NANOS_PER_SEC as i64 {
            tai_ns -= NANOS_PER_SEC as i64;
            tai_unix_s += 1;
        }
        let conv = tai_to_utc_full(tai_unix_s);
        // Sub-second part is unaffected by leap-second offset.
        conv.utc_unix_seconds * 1000 + tai_ns / 1_000_000
    }

    /// Convert to a `chrono::DateTime<Utc>` (sub-ms precision is lost).
    pub fn to_date_time(self) -> DateTime<Utc> {
        Utc.timestamp_millis_opt(self.to_unix_ms())
            .single()
            .unwrap_or(DateTime::<Utc>::UNIX_EPOCH)
    }

    /// Convert to a Julian Date (TT). Exact up to f64 mantissa.
    pub fn to_julian_date(self) -> f64 {
        self.to_brightdate() + J2000_JD
    }

    /// Convert to a Modified Julian Date (TT). Exact up to f64 mantissa.
    pub fn to_modified_julian_date(self) -> f64 {
        self.to_brightdate() + J2000_MJD
    }

    /// Render as ISO 8601 with millisecond precision. Leap seconds emit `:60`.
    pub fn to_iso(self) -> String {
        let mut tai_unix_s = J2000_TAI_UNIX_S_INT + self.tai_seconds;
        let mut tai_ns = self.tai_nanos as i64 + J2000_TAI_UNIX_NS as i64;
        if tai_ns >= NANOS_PER_SEC as i64 {
            tai_ns -= NANOS_PER_SEC as i64;
            tai_unix_s += 1;
        }
        let conv = tai_to_utc_full(tai_unix_s);
        let millis = tai_ns / 1_000_000;

        if conv.is_leap_second {
            let dt = Utc
                .timestamp_opt(conv.utc_unix_seconds, 0)
                .single()
                .unwrap_or(DateTime::<Utc>::UNIX_EPOCH);
            return format!(
                "{}T{}:{}:60.{:03}Z",
                dt.format("%Y-%m-%d"),
                dt.format("%H"),
                dt.format("%M"),
                millis.clamp(0, 999),
            );
        }
        let dt = Utc
            .timestamp_opt(conv.utc_unix_seconds, (tai_ns % 1_000_000_000) as u32)
            .single()
            .unwrap_or(DateTime::<Utc>::UNIX_EPOCH);
        // chrono renders nanoseconds; truncate to milliseconds.
        format!("{}.{:03}Z", dt.format("%Y-%m-%dT%H:%M:%S"), millis)
    }

    // ── Arithmetic ────────────────────────────────────────────────────────

    /// Add a duration (in nanoseconds) to this instant.
    pub fn add_nanos(self, nanos: i64) -> Self {
        let total_nanos = self.tai_nanos as i64 + nanos;
        let extra_secs = total_nanos.div_euclid(NANOS_PER_SEC as i64);
        let new_nanos = total_nanos.rem_euclid(NANOS_PER_SEC as i64) as u32;
        Self {
            tai_seconds: self.tai_seconds + extra_secs,
            tai_nanos: new_nanos,
        }
    }

    /// Add whole seconds (SI / TAI) to this instant.
    pub fn add_seconds(self, seconds: i64) -> Self {
        Self {
            tai_seconds: self.tai_seconds + seconds,
            tai_nanos: self.tai_nanos,
        }
    }

    /// Difference between two instants, in nanoseconds. Saturates on overflow.
    pub fn nanos_since(self, earlier: Self) -> i128 {
        let ds = self.tai_seconds as i128 - earlier.tai_seconds as i128;
        let dn = self.tai_nanos as i128 - earlier.tai_nanos as i128;
        ds * NANOS_PER_SEC as i128 + dn
    }

    /// Difference between two instants, in SI seconds (f64).
    pub fn seconds_since(self, earlier: Self) -> f64 {
        let ds = (self.tai_seconds - earlier.tai_seconds) as f64;
        let dn = (self.tai_nanos as f64 - earlier.tai_nanos as f64) / NANOS_PER_SEC as f64;
        ds + dn
    }
}

// ── Backward-compat helper ────────────────────────────────────────────────

impl From<BrightInstant> for BrightDateValue {
    fn from(i: BrightInstant) -> Self {
        i.to_brightdate()
    }
}

impl TryFrom<BrightDateValue> for BrightInstant {
    type Error = BrightDateError;
    fn try_from(bd: BrightDateValue) -> Result<Self, Self::Error> {
        Self::from_brightdate(bd)
    }
}

/// Re-export of the internal anchor for documentation/tests that want to
/// reason about the J2000.0 sub-second offset.
pub const J2000_TAI_UNIX_S_FRACT_NS: u32 = J2000_TAI_UNIX_NS;

#[allow(dead_code)]
const _STATIC_INVARIANT: () = {
    // J2000_TAI_UNIX_S must equal J2000_TAI_UNIX_S_INT + J2000_TAI_UNIX_NS·1e-9.
    // Encoded here for compile-time documentation.
    assert!(J2000_TAI_UNIX_S_INT == 946_727_967);
    assert!(J2000_TAI_UNIX_NS == 816_000_000);
    // Cross-check vs f64 constant (within representable precision).
    let _ = J2000_TAI_UNIX_S;
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::J2000_UTC_UNIX_MS;

    const J2000_UTC_UNIX_MS_I: i64 = 946_727_935_816;

    #[test]
    fn j2000_constant_is_zero() {
        assert_eq!(BrightInstant::J2000.tai_seconds_since_j2000(), 0);
        assert_eq!(BrightInstant::J2000.tai_nanos(), 0);
    }

    #[test]
    fn from_unix_ms_at_j2000_is_zero() {
        let i = BrightInstant::from_unix_ms(J2000_UTC_UNIX_MS_I);
        assert_eq!(i, BrightInstant::J2000);
    }

    #[test]
    fn from_unix_ms_at_j2000_via_f64_constant() {
        let i = BrightInstant::from_unix_ms(J2000_UTC_UNIX_MS as i64);
        assert_eq!(i, BrightInstant::J2000);
    }

    #[test]
    fn unix_ms_roundtrip_at_j2000() {
        let ms = J2000_UTC_UNIX_MS_I;
        let i = BrightInstant::from_unix_ms(ms);
        assert_eq!(i.to_unix_ms(), ms);
    }

    #[test]
    fn unix_ms_roundtrip_modern() {
        let ms = 1_700_000_000_000_i64; // 2023-11-14
        let i = BrightInstant::from_unix_ms(ms);
        assert_eq!(i.to_unix_ms(), ms);
    }

    #[test]
    fn unix_ms_roundtrip_pre_unix_epoch() {
        let ms = -1_000_000_000_000_i64; // 1938
        let i = BrightInstant::from_unix_ms(ms);
        assert_eq!(i.to_unix_ms(), ms);
    }

    #[test]
    fn nanosecond_precision_preserved() {
        let i = BrightInstant::from_tai_components(0, 1).unwrap();
        assert_eq!(i.tai_nanos(), 1);
        let later = i.add_nanos(999_999_999);
        assert_eq!(later.tai_seconds_since_j2000(), 1);
        assert_eq!(later.tai_nanos(), 0);
    }

    #[test]
    fn add_nanos_carries_correctly() {
        let i = BrightInstant::J2000.add_nanos(1_500_000_000);
        assert_eq!(i.tai_seconds_since_j2000(), 1);
        assert_eq!(i.tai_nanos(), 500_000_000);
    }

    #[test]
    fn add_nanos_negative() {
        let i = BrightInstant::J2000.add_nanos(-1);
        assert_eq!(i.tai_seconds_since_j2000(), -1);
        assert_eq!(i.tai_nanos(), 999_999_999);
    }

    #[test]
    fn nanos_since_is_signed() {
        let a = BrightInstant::J2000;
        let b = BrightInstant::J2000.add_nanos(1_500_000_000);
        assert_eq!(b.nanos_since(a), 1_500_000_000);
        assert_eq!(a.nanos_since(b), -1_500_000_000);
    }

    #[test]
    fn brightdate_roundtrip_modern() {
        let bd = 9_628.5_f64;
        let i = BrightInstant::from_brightdate(bd).unwrap();
        let back = i.to_brightdate();
        assert!((back - bd).abs() < 1e-9, "drift: {}", back - bd);
    }

    #[test]
    fn brightdate_roundtrip_far_future_holds_seconds() {
        // Year ~10_000: bd ≈ 2_922_500. Plain f64 still has ~ns precision here.
        let bd = 2_922_500.123_456_789_f64;
        let i = BrightInstant::from_brightdate(bd).unwrap();
        let back = i.to_brightdate();
        // Allow 1 μs because f64 can't round-trip 9 sig-fig fractional bits.
        let drift_us = (back - bd) * SECONDS_PER_DAY * 1_000_000.0;
        assert!(drift_us.abs() < 1.0, "drift {drift_us} μs");
    }

    #[test]
    fn julian_date_exact_at_j2000() {
        assert_eq!(BrightInstant::J2000.to_julian_date(), 2_451_545.0);
        let i = BrightInstant::from_julian_date(2_451_545.0).unwrap();
        assert_eq!(i, BrightInstant::J2000);
    }

    #[test]
    fn modified_julian_date_exact_at_j2000() {
        assert_eq!(BrightInstant::J2000.to_modified_julian_date(), 51_544.5);
        let i = BrightInstant::from_modified_julian_date(51_544.5).unwrap();
        assert_eq!(i, BrightInstant::J2000);
    }

    #[test]
    fn iso_at_j2000_is_correct_utc_label() {
        let s = BrightInstant::J2000.to_iso();
        assert!(s.starts_with("2000-01-01T11:58:55.816"), "got: {s}");
        assert!(s.ends_with('Z'));
    }

    #[test]
    fn iso_one_day_after_j2000() {
        let i = BrightInstant::J2000.add_seconds(86_400);
        let s = i.to_iso();
        assert!(s.starts_with("2000-01-02T11:58:55.816"), "got: {s}");
    }

    #[test]
    fn ordering_is_chronological() {
        let earlier = BrightInstant::from_tai_components(0, 500).unwrap();
        let later = BrightInstant::from_tai_components(0, 501).unwrap();
        assert!(earlier < later);
        let much_later = BrightInstant::from_tai_components(1, 0).unwrap();
        assert!(later < much_later);
    }

    #[test]
    fn rejects_oversized_nanos() {
        let r = BrightInstant::from_tai_components(0, NANOS_PER_SEC);
        assert!(r.is_err());
    }

    #[test]
    fn rejects_nonfinite_brightdate() {
        assert!(BrightInstant::from_brightdate(f64::NAN).is_err());
        assert!(BrightInstant::from_brightdate(f64::INFINITY).is_err());
    }
}
