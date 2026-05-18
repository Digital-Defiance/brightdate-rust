//! BrightDate — Universal Decimal Time System
//!
//! A scientifically grounded, timezone-free time representation anchored at
//! J2000.0. One `f64` value. Trivially sortable, diffable, and storable.
//!
//! # Format
//!
//! ```text
//! DDDDD.ddddd
//! ↑           ↑
//! │           Fractional day (decimal time-of-day)
//! Integer days since J2000.0 epoch
//! ```
//!
//! - **Epoch:** J2000.0 = `2000-01-01T12:00:00.000 TT`
//!   = `2000-01-01T11:58:55.816 UTC` (Unix ms `946_727_935_816`)
//!   = `JD 2_451_545.0` = `MJD 51_544.5`.
//! - **Unit:** SI days (86400 SI seconds).
//! - **Timescale:** TAI-coherent. The BrightDate clock ticks uniformly and
//!   has no leap-second discontinuities; leap seconds intervene only when
//!   converting to/from UTC labels (Unix ms, ISO strings).
//!
//! # Quick Start
//!
//! ```rust
//! use brightdate::BrightDate;
//!
//! let bd = BrightDate::from_unix_ms(1_700_000_000_000.0).unwrap();
//! println!("{}", bd);          // e.g. "8704.09722"
//! println!("{}", bd.to_log_string()); // "[8704.09722]"
//!
//! let tomorrow = bd.add_days(1.0);
//! let elapsed = tomorrow.difference(&bd); // 1.0
//! ```

pub mod arithmetic;
pub mod astronomy;
pub mod calendar;
pub mod comparisons;
pub mod constants;
pub mod conversions;
pub mod exact;
pub mod formatting;
pub mod instant;
pub mod interplanetary;
pub mod intervals;
pub mod leap_seconds;
pub mod pbd;
pub mod scheduling;
pub mod serialization;
pub mod timezones;
pub mod types;
pub mod validation;

pub use exact::ExactBrightDate;
pub use instant::BrightInstant;
pub use pbd::{
    bright_date_from_pbd, bright_date_to_pbd, bright_instant_from_pbd, bright_instant_to_pbd,
    brightdate_to_label, compare_exact_pbd, compare_pbd, format_bright_label, format_pbd,
    from_bright_label, from_exact_pbd, from_pbd, is_pbd_later, parse_bright_label, parse_pbd,
    pbd_era, pbd_page, to_bright_label, to_exact_pbd, to_pbd, BrightLabel, ExactPbd, Pbd,
    DEFAULT_BD_PRECISION, DEFAULT_PBD_PRECISION, PBD_ERA_PICOSECONDS, PBD_ERA_SECONDS,
    PBD_ERA_SECONDS_F,
};
pub use types::{BrightDateComponents, BrightDateOptions, BrightDuration, Precision};

use crate::constants::DEFAULT_PRECISION;
use crate::conversions::{
    from_gps_time, from_iso, from_julian_date, from_modified_julian_date, from_unix_ms,
    from_unix_seconds, to_date_time, to_gps_time, to_iso, to_julian_date,
    to_modified_julian_date, to_unix_ms, to_unix_seconds, tai_utc_offset_seconds_at,
};
use crate::arithmetic::{
    add, add_microdays, add_millidays, ceil_to_day, compare, difference,
    absolute_difference, equals, floor_to_day, is_in_range, lerp, midpoint,
    round_to_microday, round_to_milliday, subtract,
};
use crate::formatting::{
    decompose, format_bright_date, format_duration, format_full, format_log,
    format_prefixed, format_range, to_duration,
};
use chrono::{DateTime, Utc};

/// Immutable BrightDate value — decimal days since J2000.0 epoch.
///
/// This is the primary type for nearly all time operations. Wrap a raw `f64`
/// value to get the full BrightDate API including formatting, arithmetic, and
/// conversion methods.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
pub struct BrightDate {
    /// Raw decimal-day value since J2000.0
    pub value: f64,
    /// Display precision (decimal places)
    pub precision: Precision,
    /// Whether this value is on the TAI timescale
    pub is_tai: bool,
}

impl BrightDate {
    // ── Factory ───────────────────────────────────────────────────────────

    /// Create from a raw `f64` value (decimal days since J2000.0).
    pub fn from_value(value: f64) -> Self {
        Self {
            value,
            precision: DEFAULT_PRECISION,
            is_tai: false,
        }
    }

    /// Create from a raw value with full options.
    pub fn from_value_with_options(value: f64, options: BrightDateOptions) -> Self {
        Self {
            value,
            precision: options.precision.unwrap_or(DEFAULT_PRECISION),
            is_tai: options.use_tai.unwrap_or(false),
        }
    }

    /// Current time as a BrightDate (UTC).
    pub fn now() -> Self {
        let now_ms = Utc::now().timestamp_millis();
        Self::from_value(from_unix_ms(now_ms as f64).unwrap_or(0.0))
    }

    /// Create from a `chrono::DateTime<Utc>`.
    pub fn from_date_time(dt: DateTime<Utc>) -> Self {
        Self::from_value(from_unix_ms(dt.timestamp_millis() as f64).unwrap_or(0.0))
    }

    /// Create from a Unix timestamp in milliseconds.
    pub fn from_unix_ms(ms: f64) -> Result<Self, crate::types::BrightDateError> {
        Ok(Self::from_value(from_unix_ms(ms)?))
    }

    /// Create from a Unix timestamp in seconds.
    pub fn from_unix_seconds(s: f64) -> Result<Self, crate::types::BrightDateError> {
        Ok(Self::from_value(from_unix_seconds(s)?))
    }

    /// Create from a Julian Date.
    pub fn from_julian_date(jd: f64) -> Self {
        Self::from_value(from_julian_date(jd))
    }

    /// Create from a Modified Julian Date.
    pub fn from_modified_julian_date(mjd: f64) -> Self {
        Self::from_value(from_modified_julian_date(mjd))
    }

    /// Create from an ISO 8601 string.
    pub fn from_iso(s: &str) -> Result<Self, crate::types::BrightDateError> {
        Ok(Self::from_value(from_iso(s)?))
    }

    /// Create from GPS week number and seconds within that week.
    pub fn from_gps_time(gps_week: u32, gps_seconds: f64) -> Self {
        Self::from_value(from_gps_time(gps_week, gps_seconds))
    }

    /// The J2000.0 epoch itself (value = 0.0).
    pub fn epoch() -> Self {
        Self::from_value(0.0)
    }

    // ── Conversions ───────────────────────────────────────────────────────

    /// Convert to a `chrono::DateTime<Utc>`.
    pub fn to_date_time(&self) -> DateTime<Utc> {
        to_date_time(self.value)
    }

    /// Convert to Unix milliseconds.
    pub fn to_unix_ms(&self) -> f64 {
        to_unix_ms(self.value)
    }

    /// Convert to Unix seconds.
    pub fn to_unix_seconds(&self) -> f64 {
        to_unix_seconds(self.value)
    }

    /// Convert to Julian Date.
    pub fn to_julian_date(&self) -> f64 {
        to_julian_date(self.value)
    }

    /// Convert to Modified Julian Date.
    pub fn to_modified_julian_date(&self) -> f64 {
        to_modified_julian_date(self.value)
    }

    /// Convert to ISO 8601 string.
    pub fn to_iso(&self) -> String {
        to_iso(self.value)
    }

    /// Convert to GPS time `(gps_week, gps_seconds)`.
    pub fn to_gps_time(&self) -> (u32, f64) {
        to_gps_time(self.value)
    }

    /// Mark this BrightDate as TAI-flagged. In v1.0 the underlying value is
    /// always TAI-coherent, so this only toggles the display flag.
    pub fn to_tai(&self) -> Self {
        Self { value: self.value, precision: self.precision, is_tai: true }
    }

    /// Mark this BrightDate as UTC-flagged. In v1.0 the underlying value is
    /// always TAI-coherent, so this only toggles the display flag.
    pub fn to_utc(&self) -> Self {
        Self { value: self.value, precision: self.precision, is_tai: false }
    }

    /// Current TAI − UTC offset in whole seconds at this instant.
    pub fn tai_utc_offset_seconds(&self) -> i32 {
        tai_utc_offset_seconds_at(self.value)
    }

    // ── Arithmetic ────────────────────────────────────────────────────────

    /// Add `days` (decimal days) to this BrightDate.
    pub fn add_days(&self, days: f64) -> Self {
        Self { value: add(self.value, days), ..*self }
    }

    /// Subtract `days` (decimal days) from this BrightDate.
    pub fn sub_days(&self, days: f64) -> Self {
        Self { value: subtract(self.value, days), ..*self }
    }

    /// Add millidays.
    pub fn add_millidays(&self, md: f64) -> Self {
        Self { value: add_millidays(self.value, md), ..*self }
    }

    /// Add microdays.
    pub fn add_microdays(&self, ud: f64) -> Self {
        Self { value: add_microdays(self.value, ud), ..*self }
    }

    /// Signed difference `self − other` in decimal days.
    pub fn difference(&self, other: &Self) -> f64 {
        difference(self.value, other.value)
    }

    /// Absolute difference from `other` in decimal days.
    pub fn absolute_difference(&self, other: &Self) -> f64 {
        absolute_difference(self.value, other.value)
    }

    /// Compare ordering to `other`.
    pub fn compare(&self, other: &Self) -> std::cmp::Ordering {
        compare(self.value, other.value)
    }

    /// Test equality within `tolerance` decimal days (default: 1 microday).
    pub fn approx_eq(&self, other: &Self, tolerance: Option<f64>) -> bool {
        equals(self.value, other.value, tolerance)
    }

    /// True if `self < other`.
    pub fn is_before(&self, other: &Self) -> bool {
        self.value < other.value
    }

    /// True if `self > other`.
    pub fn is_after(&self, other: &Self) -> bool {
        self.value > other.value
    }

    /// True if `self` falls in `[start, end]`.
    pub fn is_in_range(&self, start: &Self, end: &Self) -> bool {
        is_in_range(self.value, start.value, end.value)
    }

    /// Linear interpolation between `self` and `other` at parameter `t ∈ [0,1]`.
    pub fn lerp(&self, other: &Self, t: f64) -> Self {
        Self { value: lerp(self.value, other.value, t), ..*self }
    }

    /// Midpoint between `self` and `other`.
    pub fn midpoint(&self, other: &Self) -> Self {
        Self { value: midpoint(self.value, other.value), ..*self }
    }

    /// Floor to the nearest whole day boundary.
    pub fn floor_to_day(&self) -> Self {
        Self { value: floor_to_day(self.value), ..*self }
    }

    /// Ceiling to the nearest whole day boundary.
    pub fn ceil_to_day(&self) -> Self {
        Self { value: ceil_to_day(self.value), ..*self }
    }

    /// Round to nearest milliday.
    pub fn round_to_milliday(&self) -> Self {
        Self { value: round_to_milliday(self.value), ..*self }
    }

    /// Round to nearest microday.
    pub fn round_to_microday(&self) -> Self {
        Self { value: round_to_microday(self.value), ..*self }
    }

    // ── Formatting ────────────────────────────────────────────────────────

    /// Format as decimal-day string with this instance's precision, e.g. `"9622.50417"`.
    pub fn format(&self) -> String {
        format_bright_date(self.value, self.precision)
    }

    /// Full decomposed struct.
    pub fn decompose(&self) -> BrightDateComponents {
        decompose(self.value)
    }

    /// Full formatted breakdown.
    pub fn format_full(&self) -> crate::types::FormattedBrightDate {
        format_full(self.value, self.precision)
    }

    /// Compact log string, e.g. `"[9622.50417]"`.
    pub fn to_log_string(&self) -> String {
        format_log(self.value, self.precision)
    }

    /// Prefixed string, e.g. `"BD:9622.50417"`.
    pub fn to_prefixed_string(&self, prefix: Option<&str>) -> String {
        format_prefixed(self.value, self.precision, prefix)
    }

    /// Format duration from `self` to `other`.
    pub fn format_duration_to(&self, other: &Self) -> String {
        let days = other.value - self.value;
        format_duration(days)
    }

    /// Duration breakdown.
    pub fn duration_to(&self, other: &Self) -> BrightDuration {
        to_duration(other.value - self.value)
    }

    /// Range string for `self..=other`.
    pub fn format_range_to(&self, other: &Self) -> String {
        format_range(self.value, other.value, self.precision)
    }
}

impl std::fmt::Display for BrightDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}

impl std::ops::Add<f64> for BrightDate {
    type Output = Self;
    fn add(self, rhs: f64) -> Self {
        self.add_days(rhs)
    }
}

impl std::ops::Sub<f64> for BrightDate {
    type Output = Self;
    fn sub(self, rhs: f64) -> Self {
        self.sub_days(rhs)
    }
}

impl std::ops::Sub<BrightDate> for BrightDate {
    type Output = f64;
    fn sub(self, rhs: BrightDate) -> f64 {
        self.difference(&rhs)
    }
}
