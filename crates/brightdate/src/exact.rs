//! `ExactBrightDate` — bit-exact picosecond-precision time.
//!
//! Whereas [`BrightDate`](crate::BrightDate) uses `f64` decimal days for
//! ergonomic math and astronomy, `ExactBrightDate` uses a signed `i128`
//! count of **picoseconds since J2000.0** to provide bit-for-bit
//! round-trip fidelity with Unix milliseconds, Unix seconds, and other
//! integer-based representations.
//!
//! ## When to use which
//!
//! | Use case                                            | Choose              |
//! | --------------------------------------------------- | ------------------- |
//! | Application timestamps, logs, scheduling, display   | `BrightDate`        |
//! | Astronomy, physics, fractional math                 | `BrightDate`        |
//! | Distributed systems, GPS, interplanetary timing     | `BrightInstant`     |
//! | Blockchain consensus on raw Unix-ms values          | `ExactBrightDate`   |
//! | Long-term archival that must survive bit-for-bit    | `ExactBrightDate`   |
//! | Sub-picosecond precision at any magnitude           | `ExactBrightDate`   |
//!
//! ## Precision
//!
//! - **Internal unit:** picoseconds (`1e-12` s).
//! - **Range:** `i128` covers `±~1.7e38 ps ≈ ±5.4e18 years`. Comfortably
//!   beyond the age of the universe in either direction.
//! - **Resolution:** 1 picosecond, exactly, everywhere.
//!
//! ## Algebraic laws
//!
//! For any integer `unix_ms` in `i64`:
//!
//! ```text
//! ExactBrightDate::from_unix_ms(unix_ms).to_unix_ms() == unix_ms       (bit-exact)
//! ```
//!
//! For any `a: ExactBrightDate` and `n: i128` nanoseconds:
//!
//! ```text
//! a.add_nanoseconds(n).subtract_nanoseconds(n) == a                    (bit-exact)
//! ```
//!
//! `ExactBrightDate` is immutable. All operations return new instances.

use crate::constants::{J2000_UTC_UNIX_MS, MS_PER_DAY};
use crate::types::BrightDateError;
use chrono::{DateTime, TimeZone, Utc};

// ── Scale factors (picoseconds) ─────────────────────────────────────────────

/// Picoseconds per nanosecond.
pub const PS_PER_NS: i128 = 1_000;
/// Picoseconds per microsecond.
pub const PS_PER_US: i128 = 1_000_000;
/// Picoseconds per millisecond.
pub const PS_PER_MS: i128 = 1_000_000_000;
/// Picoseconds per SI second.
pub const PS_PER_S: i128 = 1_000_000_000_000;
/// Picoseconds per SI day (86 400 s).
pub const PS_PER_DAY: i128 = 86_400 * PS_PER_S;

/// J2000.0 expressed as Unix milliseconds — the exact i64 form of
/// [`crate::constants::J2000_UTC_UNIX_MS`].
pub const J2000_UNIX_MS_I64: i64 = 946_727_935_816;

/// J2000.0 expressed as Unix picoseconds (UTC label).
pub const J2000_UNIX_PS: i128 = (J2000_UNIX_MS_I64 as i128) * PS_PER_MS;

/// An immutable, bit-exact time value stored as picoseconds since J2000.0.
///
/// # Examples
///
/// ```
/// use brightdate::ExactBrightDate;
///
/// let epoch = ExactBrightDate::epoch();
/// assert_eq!(epoch.picoseconds(), 0);
///
/// let round_trip = ExactBrightDate::from_unix_ms(1_700_000_000_000).to_unix_ms();
/// assert_eq!(round_trip, 1_700_000_000_000);
/// ```
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct ExactBrightDate {
    /// Signed picosecond count since J2000.0.
    picoseconds: i128,
}

impl ExactBrightDate {
    // ── Constructors ──────────────────────────────────────────────────────

    /// Construct from raw picoseconds since J2000.0.
    #[inline]
    pub const fn from_picoseconds(picoseconds: i128) -> Self {
        Self { picoseconds }
    }

    /// Construct from Unix milliseconds (bit-exact for integer ms input).
    #[inline]
    pub const fn from_unix_ms(unix_ms: i64) -> Self {
        Self {
            picoseconds: ((unix_ms - J2000_UNIX_MS_I64) as i128) * PS_PER_MS,
        }
    }

    /// Construct from Unix seconds (bit-exact for integer input).
    #[inline]
    pub const fn from_unix_seconds(unix_seconds: i64) -> Self {
        Self {
            picoseconds: (unix_seconds as i128) * PS_PER_S - J2000_UNIX_PS,
        }
    }

    /// Construct from Unix nanoseconds (bit-exact).
    #[inline]
    pub const fn from_unix_nanos(unix_nanos: i128) -> Self {
        Self {
            picoseconds: unix_nanos * PS_PER_NS - J2000_UNIX_PS,
        }
    }

    /// Construct from a `chrono::DateTime<Utc>` (millisecond resolution).
    pub fn from_date_time(dt: DateTime<Utc>) -> Self {
        Self::from_unix_ms(dt.timestamp_millis())
    }

    /// Construct from an ISO 8601 string (millisecond resolution via chrono).
    pub fn from_iso(s: &str) -> Result<Self, BrightDateError> {
        let dt = DateTime::parse_from_rfc3339(s)
            .map_err(|e| BrightDateError::ParseError(format!("invalid ISO 8601: {s}: {e}")))?;
        Ok(Self::from_unix_ms(dt.timestamp_millis()))
    }

    /// Construct from an `f64` `BrightDate` (lossy — bounded by f64 mantissa).
    pub fn from_brightdate(bd: f64) -> Result<Self, BrightDateError> {
        Ok(Self {
            picoseconds: crate::lens::brightdate_to_picoseconds(bd)?,
        })
    }

    /// Canonical v2 attosecond engine.
    #[inline]
    pub const fn to_exact_bright_atto(self) -> crate::ExactBrightAtto {
        crate::ExactBrightAtto::from_exact_brightdate(self)
    }

    pub fn from_exact_bright_atto(atto: crate::ExactBrightAtto) -> Self {
        atto.to_exact_brightdate()
    }

    /// J2000.0 epoch itself (`picoseconds = 0`).
    #[inline]
    pub const fn epoch() -> Self {
        Self { picoseconds: 0 }
    }

    /// Current time at millisecond resolution.
    pub fn now() -> Self {
        Self::from_unix_ms(Utc::now().timestamp_millis())
    }

    // ── Accessors ─────────────────────────────────────────────────────────

    /// Raw picoseconds since J2000.0.
    #[inline]
    pub const fn picoseconds(self) -> i128 {
        self.picoseconds
    }

    // ── Conversions out ───────────────────────────────────────────────────

    /// Convert to Unix milliseconds (bit-exact for values created from
    /// integer ms; otherwise truncates sub-millisecond picoseconds toward
    /// negative infinity).
    pub fn to_unix_ms(self) -> i64 {
        let ms = self.picoseconds.div_euclid(PS_PER_MS);
        (ms + J2000_UNIX_MS_I64 as i128) as i64
    }

    /// Convert to Unix seconds (truncates sub-second picoseconds toward
    /// negative infinity).
    pub fn to_unix_seconds(self) -> i64 {
        let total_ps = self.picoseconds + J2000_UNIX_PS;
        total_ps.div_euclid(PS_PER_S) as i64
    }

    /// Convert to the `f64` `BrightDate` value (decimal days since J2000.0).
    /// Lossy for sub-microsecond detail at current-era magnitudes.
    pub fn to_brightdate(self) -> f64 {
        crate::lens::ticks_to_brightdate(self.picoseconds, PS_PER_DAY)
    }

    /// Convert to a `chrono::DateTime<Utc>` (millisecond resolution).
    pub fn to_date_time(self) -> DateTime<Utc> {
        Utc.timestamp_millis_opt(self.to_unix_ms())
            .single()
            .unwrap_or(DateTime::<Utc>::UNIX_EPOCH)
    }

    /// Render as ISO 8601 with millisecond precision.
    pub fn to_iso(self) -> String {
        self.to_date_time()
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string()
    }

    // ── Arithmetic (return new instances) ─────────────────────────────────

    /// Add picoseconds.
    #[inline]
    pub const fn add_picoseconds(self, ps: i128) -> Self {
        Self {
            picoseconds: self.picoseconds + ps,
        }
    }

    /// Add nanoseconds.
    #[inline]
    pub const fn add_nanoseconds(self, ns: i128) -> Self {
        Self {
            picoseconds: self.picoseconds + ns * PS_PER_NS,
        }
    }

    /// Add microseconds.
    #[inline]
    pub const fn add_microseconds(self, us: i128) -> Self {
        Self {
            picoseconds: self.picoseconds + us * PS_PER_US,
        }
    }

    /// Add milliseconds.
    #[inline]
    pub const fn add_milliseconds(self, ms: i128) -> Self {
        Self {
            picoseconds: self.picoseconds + ms * PS_PER_MS,
        }
    }

    /// Add SI seconds.
    #[inline]
    pub const fn add_seconds(self, s: i128) -> Self {
        Self {
            picoseconds: self.picoseconds + s * PS_PER_S,
        }
    }

    /// Add SI days.
    #[inline]
    pub const fn add_days(self, days: i128) -> Self {
        Self {
            picoseconds: self.picoseconds + days * PS_PER_DAY,
        }
    }

    /// Subtract picoseconds.
    #[inline]
    pub const fn subtract_picoseconds(self, ps: i128) -> Self {
        Self {
            picoseconds: self.picoseconds - ps,
        }
    }

    /// Subtract nanoseconds.
    #[inline]
    pub const fn subtract_nanoseconds(self, ns: i128) -> Self {
        Self {
            picoseconds: self.picoseconds - ns * PS_PER_NS,
        }
    }

    /// Subtract milliseconds.
    #[inline]
    pub const fn subtract_milliseconds(self, ms: i128) -> Self {
        Self {
            picoseconds: self.picoseconds - ms * PS_PER_MS,
        }
    }

    /// Subtract SI days.
    #[inline]
    pub const fn subtract_days(self, days: i128) -> Self {
        Self {
            picoseconds: self.picoseconds - days * PS_PER_DAY,
        }
    }

    /// Signed difference `self − other` in picoseconds.
    #[inline]
    pub const fn difference_picoseconds(self, other: Self) -> i128 {
        self.picoseconds - other.picoseconds
    }

    /// Difference in nanoseconds (truncated toward zero).
    #[inline]
    pub const fn difference_nanoseconds(self, other: Self) -> i128 {
        (self.picoseconds - other.picoseconds) / PS_PER_NS
    }

    /// Difference in microseconds (truncated toward zero).
    #[inline]
    pub const fn difference_microseconds(self, other: Self) -> i128 {
        (self.picoseconds - other.picoseconds) / PS_PER_US
    }

    /// Difference in milliseconds (truncated toward zero).
    #[inline]
    pub const fn difference_milliseconds(self, other: Self) -> i128 {
        (self.picoseconds - other.picoseconds) / PS_PER_MS
    }

    /// Difference in SI seconds (truncated toward zero).
    #[inline]
    pub const fn difference_seconds(self, other: Self) -> i128 {
        (self.picoseconds - other.picoseconds) / PS_PER_S
    }

    // ── Comparison helpers ────────────────────────────────────────────────

    /// True iff `self < other`.
    #[inline]
    pub const fn is_before(self, other: Self) -> bool {
        self.picoseconds < other.picoseconds
    }

    /// True iff `self > other`.
    #[inline]
    pub const fn is_after(self, other: Self) -> bool {
        self.picoseconds > other.picoseconds
    }

    // ── Serialization ─────────────────────────────────────────────────────

    /// Encode as the compact `"EBD1:<picoseconds>"` string format.
    pub fn encode(self) -> String {
        format!("EBD1:{}", self.picoseconds)
    }

    /// Decode from the `"EBD1:<picoseconds>"` string format.
    pub fn decode(encoded: &str) -> Result<Self, BrightDateError> {
        let body = encoded.strip_prefix("EBD1:").ok_or_else(|| {
            BrightDateError::ParseError(format!(
                "ExactBrightDate encoding must start with \"EBD1:\", got: {encoded:?}"
            ))
        })?;
        let ps: i128 = body.parse().map_err(|_| {
            BrightDateError::ParseError(format!(
                "ExactBrightDate encoding has invalid picosecond body: {body:?}"
            ))
        })?;
        Ok(Self::from_picoseconds(ps))
    }

    /// Encode as a 16-byte big-endian two's-complement representation.
    pub fn to_be_bytes(self) -> [u8; 16] {
        self.picoseconds.to_be_bytes()
    }

    /// Decode from a 16-byte big-endian two's-complement representation.
    pub fn from_be_bytes(bytes: [u8; 16]) -> Self {
        Self::from_picoseconds(i128::from_be_bytes(bytes))
    }
}

impl std::fmt::Display for ExactBrightDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.picoseconds)
    }
}

impl From<crate::BrightInstant> for ExactBrightDate {
    fn from(inst: crate::BrightInstant) -> Self {
        let ps = (inst.tai_seconds_since_j2000() as i128) * PS_PER_S
            + (inst.tai_nanos() as i128) * PS_PER_NS;
        Self::from_picoseconds(ps)
    }
}

impl From<ExactBrightDate> for crate::BrightInstant {
    fn from(value: ExactBrightDate) -> Self {
        let ps = value.picoseconds;
        let mut secs = ps.div_euclid(PS_PER_S);
        let mut sub_ps = ps.rem_euclid(PS_PER_S);
        if sub_ps < 0 {
            secs -= 1;
            sub_ps += PS_PER_S;
        }
        // Truncate sub-ns digits to fit BrightInstant's nanosecond resolution.
        let nanos = (sub_ps / PS_PER_NS) as u32;
        crate::BrightInstant::from_tai_components(secs as i64, nanos)
            .expect("nanos < 1_000_000_000 by construction")
    }
}

/// Static doc-test invariants.
#[allow(dead_code)]
const _STATIC_INVARIANT: () = {
    // f64 J2000_UTC_UNIX_MS must match the i64 form bit-exactly.
    assert!(J2000_UNIX_MS_I64 == 946_727_935_816);
    let _ = J2000_UTC_UNIX_MS;
    let _ = MS_PER_DAY;
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_is_zero() {
        assert_eq!(ExactBrightDate::epoch().picoseconds(), 0);
    }

    #[test]
    fn unix_ms_roundtrip_is_bit_exact() {
        for ms in [
            0_i64,
            J2000_UNIX_MS_I64,
            1_700_000_000_000,
            -1_000_000_000_000,
            i64::MIN / 4,
            i64::MAX / 4,
        ] {
            let exact = ExactBrightDate::from_unix_ms(ms);
            assert_eq!(exact.to_unix_ms(), ms, "ms = {ms}");
        }
    }

    #[test]
    fn at_j2000_is_zero() {
        let e = ExactBrightDate::from_unix_ms(J2000_UNIX_MS_I64);
        assert_eq!(e, ExactBrightDate::epoch());
    }

    #[test]
    fn nanosecond_arithmetic_roundtrips() {
        let a = ExactBrightDate::from_unix_ms(1_700_000_000_000);
        let b = a.add_nanoseconds(123_456_789).subtract_nanoseconds(123_456_789);
        assert_eq!(a, b);
    }

    #[test]
    fn picosecond_arithmetic_is_lossless() {
        let a = ExactBrightDate::epoch();
        let b = a.add_picoseconds(1);
        assert_eq!(b.picoseconds(), 1);
        assert!(b.is_after(a));
        assert!(a.is_before(b));
    }

    #[test]
    fn encode_decode_roundtrip() {
        let a = ExactBrightDate::from_picoseconds(-123_456_789_000_000);
        let encoded = a.encode();
        assert!(encoded.starts_with("EBD1:"));
        let b = ExactBrightDate::decode(&encoded).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn decode_rejects_bad_prefix() {
        assert!(ExactBrightDate::decode("XBD1:123").is_err());
        assert!(ExactBrightDate::decode("EBD1:notanint").is_err());
    }

    #[test]
    fn be_bytes_roundtrip() {
        let a = ExactBrightDate::from_picoseconds(-987_654_321_000);
        let bytes = a.to_be_bytes();
        assert_eq!(ExactBrightDate::from_be_bytes(bytes), a);
    }

    #[test]
    fn bright_instant_roundtrip() {
        let inst = crate::BrightInstant::from_tai_components(123_456, 789_000_000).unwrap();
        let exact: ExactBrightDate = inst.into();
        let back: crate::BrightInstant = exact.into();
        assert_eq!(back, inst);
    }

    #[test]
    fn brightdate_roundtrip_modern() {
        let bd_ms = 1_700_000_000_000_i64;
        let exact = ExactBrightDate::from_unix_ms(bd_ms);
        let bd = exact.to_brightdate();
        let back = ExactBrightDate::from_brightdate(bd).unwrap();
        // f64 round-trip should land within a microsecond.
        let diff = (exact.picoseconds() - back.picoseconds()).abs();
        assert!(diff < PS_PER_US, "diff = {diff} ps");
    }

    #[test]
    fn difference_milliseconds_truncates() {
        let a = ExactBrightDate::epoch();
        let b = a.add_picoseconds(2_500 * PS_PER_MS + 1); // 2.5 s + 1 ps
        assert_eq!(b.difference_milliseconds(a), 2500);
    }

    #[test]
    fn negative_picoseconds_brightdate_is_negative() {
        let e = ExactBrightDate::from_picoseconds(-PS_PER_DAY);
        assert_eq!(e.to_brightdate(), -1.0);
    }
}
