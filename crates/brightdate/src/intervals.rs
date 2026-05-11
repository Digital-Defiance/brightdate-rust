//! BrightDate intervals — a closed `[start, end]` time span.

use crate::formatting::{format_duration, format_range, to_duration};
use crate::types::{BrightDateValue, BrightDuration};
use crate::BrightDate;

/// A closed time interval `[start, end]` in BrightDate values.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BrightDateInterval {
    pub start: BrightDate,
    pub end: BrightDate,
}

impl BrightDateInterval {
    /// Create from two `BrightDate` instances. Panics if `start > end`.
    pub fn new(start: BrightDate, end: BrightDate) -> Self {
        assert!(
            start.value <= end.value,
            "BrightDateInterval: start ({}) must be <= end ({})",
            start.value,
            end.value
        );
        Self { start, end }
    }

    /// Create from two raw `f64` values.
    pub fn from_values(start: BrightDateValue, end: BrightDateValue) -> Self {
        Self::new(BrightDate::from_value(start), BrightDate::from_value(end))
    }

    /// Create from two JavaScript-style `Date` values (via `chrono::DateTime<Utc>`).
    pub fn from_dates(
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Self, crate::types::BrightDateError> {
        use crate::conversions::from_unix_ms;
        let s = from_unix_ms(start.timestamp_millis() as f64)?;
        let e = from_unix_ms(end.timestamp_millis() as f64)?;
        Ok(Self::from_values(s, e))
    }

    /// Create from two ISO-8601 strings.
    pub fn from_iso(start: &str, end: &str) -> Result<Self, crate::types::BrightDateError> {
        use crate::conversions::from_iso;
        let s = from_iso(start)?;
        let e = from_iso(end)?;
        Ok(Self::from_values(s, e))
    }
    /// Create from a start `BrightDate` and duration in decimal days.
    pub fn from_duration(start: BrightDate, duration_days: f64) -> Self {
        let end = start.add_days(duration_days);
        Self::new(start, end)
    }

    /// Duration in decimal days.
    #[allow(clippy::misnamed_getters)]
    pub fn duration(&self) -> f64 {
        self.end.value - self.start.value
    }

    /// Duration as a `BrightDuration`.
    pub fn duration_metric(&self) -> BrightDuration {
        to_duration(self.duration())
    }

    /// True if `point` is inside (or on the boundary of) this interval.
    pub fn contains(&self, point: &BrightDate) -> bool {
        point.value >= self.start.value && point.value <= self.end.value
    }

    /// True if `value` (raw f64) is inside (or on the boundary of) this interval.
    pub fn contains_value(&self, value: f64) -> bool {
        value >= self.start.value && value <= self.end.value
    }

    /// True if this interval overlaps with `other` (touching endpoints count).
    pub fn overlaps(&self, other: &Self) -> bool {
        self.start.value <= other.end.value && self.end.value >= other.start.value
    }

    /// Return the intersection with `other`, or `None` if they don't overlap.
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        let s = self.start.value.max(other.start.value);
        let e = self.end.value.min(other.end.value);
        if s <= e {
            Some(Self::from_values(s, e))
        } else {
            None
        }
    }

    /// Return the union if the intervals overlap or are adjacent; `None` otherwise.
    pub fn union(&self, other: &Self) -> Option<Self> {
        if self.overlaps(other) || self.adjacent_to(other) {
            let s = self.start.value.min(other.start.value);
            let e = self.end.value.max(other.end.value);
            Some(Self::from_values(s, e))
        } else {
            None
        }
    }

    /// True if the intervals share exactly one endpoint and do not overlap.
    pub fn adjacent_to(&self, other: &Self) -> bool {
        self.end.value == other.start.value || other.end.value == self.start.value
    }

    /// True if `self` fully contains `other` (both endpoints included).
    pub fn encloses(&self, other: &Self) -> bool {
        self.start.value <= other.start.value && self.end.value >= other.end.value
    }

    /// Split into `count` equal sub-intervals. Panics if `count < 1`.
    pub fn split(&self, count: usize) -> Vec<Self> {
        assert!(count >= 1, "Count must be at least 1");
        let step = self.duration() / count as f64;
        (0..count)
            .map(|i| {
                let s = self.start.value + step * i as f64;
                let e = s + step;
                Self::from_values(s, e)
            })
            .collect()
    }

    /// Return a new interval expanded by `amount` on each side.
    pub fn expand(&self, amount: f64) -> Self {
        Self::from_values(self.start.value - amount, self.end.value + amount)
    }

    /// Return a new interval shrunk by `amount` on each side, or `None` if it would invert.
    pub fn shrink(&self, amount: f64) -> Option<Self> {
        let s = self.start.value + amount;
        let e = self.end.value - amount;
        if s <= e { Some(Self::from_values(s, e)) } else { None }
    }

    /// Shift both endpoints by `amount`.
    pub fn shift(&self, amount: f64) -> Self {
        Self::from_values(self.start.value + amount, self.end.value + amount)
    }

    /// Iterate over the interval at the given `step` size, yielding each point.
    pub fn iterate(&self, step: f64) -> Vec<BrightDate> {
        assert!(step > 0.0, "Step must be positive");
        let mut result = Vec::new();
        let mut current = self.start.value;
        while current <= self.end.value + f64::EPSILON {
            result.push(BrightDate::from_value(current));
            current += step;
        }
        result
    }

    /// Return `count` evenly-spaced sample points across the interval.
    ///
    /// `count = 0` → empty; `count = 1` → midpoint.
    pub fn sample(&self, count: usize) -> Vec<BrightDate> {
        if count == 0 {
            return vec![];
        }
        if count == 1 {
            return vec![self.midpoint()];
        }
        let step = self.duration() / (count - 1) as f64;
        (0..count)
            .map(|i| BrightDate::from_value(self.start.value + step * i as f64))
            .collect()
    }

    /// Midpoint of this interval.
    pub fn midpoint(&self) -> BrightDate {
        self.start.lerp(&self.end, 0.5)
    }

    /// Human-readable duration string.
    pub fn format_duration_str(&self) -> String {
        format_duration(self.duration())
    }

    /// Range string, e.g. `"9622.00000 → 9623.00000"`.
    pub fn format_range(&self) -> String {
        format_range(self.start.value, self.end.value, self.start.precision)
    }
}

impl std::fmt::Display for BrightDateInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format_range())
    }
}
