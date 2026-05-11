//! Comparison and set utilities for collections of BrightDate values.

use crate::types::BrightDateValue;
use std::collections::HashMap;

/// Statistics computed over a collection of BrightDate values.
#[derive(Debug, Clone, PartialEq)]
pub struct Stats {
    pub count: usize,
    pub min: BrightDateValue,
    pub max: BrightDateValue,
    pub range: BrightDateValue,
    pub mean: BrightDateValue,
    pub median: BrightDateValue,
    pub std_dev: BrightDateValue,
}

/// Return the value in `values` closest to `target`, or `None` if empty.
pub fn closest(target: BrightDateValue, values: &[BrightDateValue]) -> Option<BrightDateValue> {
    values
        .iter()
        .copied()
        .min_by(|a, b| {
            let da = (a - target).abs();
            let db = (b - target).abs();
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
}

/// Return all values within `max_distance` of `target` (inclusive ≤).
pub fn within(
    target: BrightDateValue,
    values: &[BrightDateValue],
    max_distance: f64,
) -> Vec<BrightDateValue> {
    values
        .iter()
        .copied()
        .filter(|&v| (v - target).abs() <= max_distance)
        .collect()
}

/// Partition `values` into `(past, future)` relative to `reference`.
///
/// Values strictly less than `reference` go into `past`;
/// values ≥ `reference` go into `future` (so the reference point itself
/// is in `future`, matching the TypeScript behaviour).
pub fn partition(
    values: &[BrightDateValue],
    reference: BrightDateValue,
) -> (Vec<BrightDateValue>, Vec<BrightDateValue>) {
    let past: Vec<_> = values.iter().copied().filter(|&v| v < reference).collect();
    let future: Vec<_> = values.iter().copied().filter(|&v| v >= reference).collect();
    (past, future)
}

/// Group values by their integer (floor) day.
pub fn group_by_day(values: &[BrightDateValue]) -> HashMap<i64, Vec<BrightDateValue>> {
    let mut map: HashMap<i64, Vec<BrightDateValue>> = HashMap::new();
    for &v in values {
        map.entry(v.floor() as i64).or_default().push(v);
    }
    map
}

/// Compute statistics over `values`. Panics if `values` is empty.
pub fn statistics(values: &[BrightDateValue]) -> Stats {
    assert!(!values.is_empty(), "Cannot compute statistics of empty array");

    let count = values.len();
    let sum: f64 = values.iter().sum();
    let mean = sum / count as f64;

    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = max - min;

    // Compute median on a sorted copy (do not mutate input)
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = if count % 2 == 1 {
        sorted[count / 2]
    } else {
        (sorted[count / 2 - 1] + sorted[count / 2]) / 2.0
    };

    let variance = if count > 1 {
        values.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / (count as f64 - 1.0)
    } else {
        0.0
    };
    let std_dev = variance.sqrt();

    Stats { count, min, max, range, mean, median, std_dev }
}

/// Compute gaps (differences between consecutive sorted values).
///
/// Sorts the input first; does not mutate the original.
pub fn gaps(values: &[BrightDateValue]) -> Vec<f64> {
    if values.len() < 2 {
        return vec![];
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    sorted.windows(2).map(|w| w[1] - w[0]).collect()
}

/// Description of the largest gap between consecutive values.
#[derive(Debug, Clone, PartialEq)]
pub struct GapInfo {
    pub gap: f64,
    pub before: BrightDateValue,
    pub after: BrightDateValue,
}

/// Find the largest gap between consecutive sorted values, or `None` if < 2 values.
pub fn largest_gap(values: &[BrightDateValue]) -> Option<GapInfo> {
    if values.len() < 2 {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    sorted
        .windows(2)
        .map(|w| GapInfo { gap: w[1] - w[0], before: w[0], after: w[1] })
        .max_by(|a, b| a.gap.partial_cmp(&b.gap).unwrap_or(std::cmp::Ordering::Equal))
}

/// Return a deduplicated copy, sorted first.
///
/// Two adjacent (after sorting) values are considered duplicates when their
/// absolute difference is **not** strictly greater than `tolerance`.
/// The default tolerance is `0.0` (exact equality).
pub fn deduplicate(values: &[BrightDateValue], tolerance: f64) -> Vec<BrightDateValue> {
    if values.is_empty() {
        return vec![];
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut result = vec![sorted[0]];
    for &v in &sorted[1..] {
        let last = *result.last().unwrap();
        if (v - last).abs() > tolerance {
            result.push(v);
        }
    }
    result
}

/// True if every consecutive pair satisfies `a[i] < a[i+1]` (strictly).
pub fn is_monotonically_increasing(values: &[BrightDateValue]) -> bool {
    values.windows(2).all(|w| w[0] < w[1])
}

/// True if every consecutive pair satisfies `a[i] <= a[i+1]` (non-decreasing).
pub fn is_non_decreasing(values: &[BrightDateValue]) -> bool {
    values.windows(2).all(|w| w[0] <= w[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closest_basic() {
        assert_eq!(closest(5.0, &[1.0, 4.0, 7.0, 10.0]), Some(4.0));
        assert_eq!(closest(5.0, &[]), None);
    }

    #[test]
    fn statistics_basic() {
        let s = statistics(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(s.min, 1.0);
        assert_eq!(s.max, 5.0);
        assert_eq!(s.median, 3.0);
    }

    #[test]
    #[should_panic(expected = "Cannot compute statistics of empty array")]
    fn statistics_empty_panics() {
        statistics(&[]);
    }
}
