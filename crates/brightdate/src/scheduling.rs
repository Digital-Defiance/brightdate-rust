//! BrightDate Scheduling Utilities
//!
//! Tools for recurring events and time-based coordination using
//! BrightDate's decimal day system.

use crate::types::BrightDateValue;

// ─── Common intervals ─────────────────────────────────────────────────────────

/// Every second (in decimal days).
pub const INTERVAL_SECOND: f64 = 1.0 / 86_400.0;
/// Every minute.
pub const INTERVAL_MINUTE: f64 = 1.0 / 1_440.0;
/// Every 5 minutes.
pub const INTERVAL_FIVE_MINUTES: f64 = 5.0 / 1_440.0;
/// Every 15 minutes.
pub const INTERVAL_QUARTER_HOUR: f64 = 15.0 / 1_440.0;
/// Every 30 minutes.
pub const INTERVAL_HALF_HOUR: f64 = 30.0 / 1_440.0;
/// Every hour.
pub const INTERVAL_HOUR: f64 = 1.0 / 24.0;
/// Every 2 hours.
pub const INTERVAL_TWO_HOURS: f64 = 2.0 / 24.0;
/// Every 4 hours.
pub const INTERVAL_FOUR_HOURS: f64 = 4.0 / 24.0;
/// Every 6 hours.
pub const INTERVAL_SIX_HOURS: f64 = 6.0 / 24.0;
/// Every 8 hours.
pub const INTERVAL_EIGHT_HOURS: f64 = 8.0 / 24.0;
/// Every 12 hours.
pub const INTERVAL_HALF_DAY: f64 = 0.5;
/// Every day.
pub const INTERVAL_DAY: f64 = 1.0;
/// Every 7 days.
pub const INTERVAL_WEEK: f64 = 7.0;
/// Every 14 days.
pub const INTERVAL_FORTNIGHT: f64 = 14.0;
/// Approximate month (30 days).
pub const INTERVAL_MONTH_APPROX: f64 = 30.0;
/// Approximate quarter (91.25 days).
pub const INTERVAL_QUARTER_APPROX: f64 = 91.25;
/// Approximate year (365.25 days).
pub const INTERVAL_YEAR_APPROX: f64 = 365.25;

// ─── RecurrencePattern ────────────────────────────────────────────────────────

/// Pattern that describes a recurring event.
#[derive(Debug, Clone)]
pub struct RecurrencePattern {
    /// Interval between occurrences in decimal days (must be > 0).
    pub interval_days: f64,
    /// BrightDate value of the first occurrence.
    pub start: BrightDateValue,
    /// Optional exclusive end — no occurrences after this value.
    pub end: Option<BrightDateValue>,
    /// Optional cap on total occurrences.
    pub max_occurrences: Option<usize>,
}

impl RecurrencePattern {
    /// Create a simple repeating pattern with no end.
    pub fn new(start: BrightDateValue, interval_days: f64) -> Self {
        Self {
            interval_days,
            start,
            end: None,
            max_occurrences: None,
        }
    }

    /// Builder: set an end value.
    pub fn with_end(mut self, end: BrightDateValue) -> Self {
        self.end = Some(end);
        self
    }

    /// Builder: set a max-occurrence count.
    pub fn with_max(mut self, n: usize) -> Self {
        self.max_occurrences = Some(n);
        self
    }
}

// ─── Core functions ───────────────────────────────────────────────────────────

/// Generate all occurrences for a [`RecurrencePattern`].
///
/// Returns a `Vec` rather than a lazy iterator so callers don't need to carry
/// the lifetime of the pattern.
pub fn recurrences(pattern: &RecurrencePattern) -> Vec<BrightDateValue> {
    if pattern.interval_days <= 0.0 {
        return Vec::new();
    }
    let mut result = Vec::new();
    let mut current = pattern.start;
    let mut count = 0usize;

    loop {
        if let Some(end) = pattern.end {
            if current > end {
                break;
            }
        }
        if let Some(max) = pattern.max_occurrences {
            if count >= max {
                break;
            }
        }
        result.push(current);
        current += pattern.interval_days;
        count += 1;

        // Safety valve against unbounded patterns that have neither end nor max.
        if pattern.end.is_none() && pattern.max_occurrences.is_none() {
            break; // caller should use next_occurrences() for unbounded patterns
        }
    }
    result
}

/// Return the next `count` occurrences starting at (and including) `pattern.start`.
pub fn next_occurrences(pattern: &RecurrencePattern, count: usize) -> Vec<BrightDateValue> {
    if pattern.interval_days <= 0.0 {
        return Vec::new();
    }
    let bounded = RecurrencePattern {
        max_occurrences: Some(count),
        end: None,
        ..pattern.clone()
    };
    recurrences(&bounded)
}

/// Find the first occurrence **strictly after** `after`.
pub fn next_occurrence_after(
    pattern: &RecurrencePattern,
    after: BrightDateValue,
) -> Option<BrightDateValue> {
    if pattern.interval_days <= 0.0 {
        return None;
    }
    let elapsed = after - pattern.start;
    let next = if elapsed < 0.0 {
        pattern.start
    } else {
        let intervals = (elapsed / pattern.interval_days).floor() as i64 + 1;
        pattern.start + intervals as f64 * pattern.interval_days
    };

    if let Some(end) = pattern.end {
        if next > end {
            return None;
        }
    }
    if let Some(max) = pattern.max_occurrences {
        let intervals_from_start =
            ((next - pattern.start) / pattern.interval_days).round() as usize;
        if intervals_from_start >= max {
            return None;
        }
    }
    Some(next)
}

/// Find the last occurrence **strictly before** `before`.
pub fn previous_occurrence_before(
    pattern: &RecurrencePattern,
    before: BrightDateValue,
) -> Option<BrightDateValue> {
    if pattern.interval_days <= 0.0 {
        return None;
    }
    let elapsed = before - pattern.start;
    if elapsed <= 0.0 {
        return None;
    }
    let intervals = (elapsed / pattern.interval_days).floor() as i64;
    let prev = pattern.start + intervals as f64 * pattern.interval_days;

    // If prev == before, step back one more
    if (prev - before).abs() < 1e-12 {
        if intervals <= 0 {
            return None;
        }
        let adjusted = pattern.start + (intervals - 1) as f64 * pattern.interval_days;
        return if adjusted >= pattern.start {
            Some(adjusted)
        } else {
            None
        };
    }

    Some(prev)
}

// ─── ScheduledEvent ───────────────────────────────────────────────────────────

/// A named, timestamped event.
#[derive(Debug, Clone)]
pub struct ScheduledEvent {
    /// Unique identifier.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// BrightDate value when the event occurs.
    pub time: BrightDateValue,
    /// Optional duration in decimal days.
    pub duration: Option<f64>,
}

// ─── BrightDateTimeline ───────────────────────────────────────────────────────

/// A sorted list of [`ScheduledEvent`]s.
#[derive(Debug, Default)]
pub struct BrightDateTimeline {
    events: Vec<ScheduledEvent>,
}

impl BrightDateTimeline {
    /// Create an empty timeline.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an event, keeping the list sorted by time.
    pub fn add(&mut self, event: ScheduledEvent) {
        self.events.push(event);
        self.events.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));
    }

    /// Remove an event by id.  Returns `true` if it was found and removed.
    pub fn remove(&mut self, id: &str) -> bool {
        if let Some(pos) = self.events.iter().position(|e| e.id == id) {
            self.events.remove(pos);
            true
        } else {
            false
        }
    }

    /// Return all events whose `time` falls in `[start, end]`.
    pub fn get_in_range(&self, start: BrightDateValue, end: BrightDateValue) -> Vec<&ScheduledEvent> {
        self.events
            .iter()
            .filter(|e| e.time >= start && e.time <= end)
            .collect()
    }

    /// Return the first event **strictly after** `after`.
    pub fn get_next(&self, after: BrightDateValue) -> Option<&ScheduledEvent> {
        self.events.iter().find(|e| e.time > after)
    }

    /// Return the last event **strictly before** `before`.
    pub fn get_previous(&self, before: BrightDateValue) -> Option<&ScheduledEvent> {
        self.events.iter().rev().find(|e| e.time < before)
    }

    /// Number of events in the timeline.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// `true` if the timeline has no events.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Iterate over all events in time order.
    pub fn iter(&self) -> std::slice::Iter<'_, ScheduledEvent> {
        self.events.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_occurrences_basic() {
        let p = RecurrencePattern::new(0.0, 1.0);
        let v = next_occurrences(&p, 3);
        assert_eq!(v, vec![0.0, 1.0, 2.0]);
    }

    #[test]
    fn next_occurrence_after_basic() {
        let p = RecurrencePattern::new(0.0, 1.0);
        assert_eq!(next_occurrence_after(&p, 0.5), Some(1.0));
    }

    #[test]
    fn previous_occurrence_before_basic() {
        let p = RecurrencePattern::new(0.0, 1.0);
        assert_eq!(previous_occurrence_before(&p, 2.5), Some(2.0));
    }

    #[test]
    fn timeline_sorted_insert() {
        let mut tl = BrightDateTimeline::new();
        tl.add(ScheduledEvent { id: "b".into(), name: "B".into(), time: 2.0, duration: None });
        tl.add(ScheduledEvent { id: "a".into(), name: "A".into(), time: 1.0, duration: None });
        let times: Vec<f64> = tl.iter().map(|e| e.time).collect();
        assert_eq!(times, vec![1.0, 2.0]);
    }
}
