//! Arithmetic operations on BrightDate values.

use crate::types::BrightDateValue;

/// Add `days` to a BrightDate value.
pub fn add(bd: BrightDateValue, days: f64) -> BrightDateValue {
    bd + days
}

/// Subtract `days` from a BrightDate value.
pub fn subtract(bd: BrightDateValue, days: f64) -> BrightDateValue {
    bd - days
}

/// Add millidays to a BrightDate value.
pub fn add_millidays(bd: BrightDateValue, millidays: f64) -> BrightDateValue {
    bd + millidays * 1e-3
}

/// Add microdays to a BrightDate value.
pub fn add_microdays(bd: BrightDateValue, microdays: f64) -> BrightDateValue {
    bd + microdays * 1e-6
}

/// Signed difference `a − b` in decimal days.
pub fn difference(a: BrightDateValue, b: BrightDateValue) -> f64 {
    a - b
}

/// Absolute difference between two BrightDate values.
pub fn absolute_difference(a: BrightDateValue, b: BrightDateValue) -> f64 {
    (a - b).abs()
}

/// Compare two BrightDate values, returning `std::cmp::Ordering`.
pub fn compare(a: BrightDateValue, b: BrightDateValue) -> std::cmp::Ordering {
    a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
}

/// Test equality within `tolerance` decimal days.
///
/// Default tolerance when `None` is passed: 1 microday (`1e-6`).
pub fn equals(a: BrightDateValue, b: BrightDateValue, tolerance: Option<f64>) -> bool {
    let tol = tolerance.unwrap_or(1e-6);
    (a - b).abs() <= tol
}

/// True if `value` is in the closed interval `[start, end]`.
pub fn is_in_range(value: BrightDateValue, start: BrightDateValue, end: BrightDateValue) -> bool {
    value >= start && value <= end
}

/// Linear interpolation: `a + t * (b − a)`.
pub fn lerp(a: BrightDateValue, b: BrightDateValue, t: f64) -> BrightDateValue {
    a + t * (b - a)
}

/// Midpoint of two BrightDate values.
pub fn midpoint(a: BrightDateValue, b: BrightDateValue) -> BrightDateValue {
    (a + b) / 2.0
}

/// Floor to the nearest whole-day boundary.
pub fn floor_to_day(bd: BrightDateValue) -> BrightDateValue {
    bd.floor()
}

/// Ceiling to the nearest whole-day boundary.
pub fn ceil_to_day(bd: BrightDateValue) -> BrightDateValue {
    bd.ceil()
}

/// Round to nearest milliday.
pub fn round_to_milliday(bd: BrightDateValue) -> BrightDateValue {
    (bd * 1_000.0).round() / 1_000.0
}

/// Round to nearest microday.
pub fn round_to_microday(bd: BrightDateValue) -> BrightDateValue {
    (bd * 1_000_000.0).round() / 1_000_000.0
}

/// Clamp `value` to the closed interval `[min_val, max_val]`.
pub fn clamp(value: BrightDateValue, min_val: BrightDateValue, max_val: BrightDateValue) -> BrightDateValue {
    value.max(min_val).min(max_val)
}

/// Whole-day count between two BrightDate values (truncates toward zero).
pub fn whole_days_between(a: BrightDateValue, b: BrightDateValue) -> i64 {
    (b - a).trunc() as i64
}

/// Generate `n` evenly-spaced values from `start` to `end` inclusive.
///
/// If `n < 2`, returns `vec![start]`.
pub fn linspace(start: BrightDateValue, end: BrightDateValue, n: usize) -> Vec<BrightDateValue> {
    if n < 2 {
        return vec![start];
    }
    let step = (end - start) / (n - 1) as f64;
    (0..n).map(|i| start + step * i as f64).collect()
}

/// Return the minimum value in a slice. Panics if the slice is empty.
pub fn min_of(values: &[BrightDateValue]) -> BrightDateValue {
    assert!(!values.is_empty(), "Cannot find minimum of empty array");
    values.iter().cloned().fold(f64::INFINITY, f64::min)
}

/// Return the maximum value in a slice. Panics if the slice is empty.
pub fn max_of(values: &[BrightDateValue]) -> BrightDateValue {
    assert!(!values.is_empty(), "Cannot find maximum of empty array");
    values.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
}

/// Return a sorted copy of `values` (does not mutate the input).
pub fn sort(values: &[BrightDateValue]) -> Vec<BrightDateValue> {
    let mut v = values.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_subtract_roundtrip() {
        let bd = 9_622.5_f64;
        assert_eq!(subtract(add(bd, 1.0), 1.0), bd);
    }

    #[test]
    fn milliday_microday() {
        let bd = 0.0_f64;
        assert!((add_millidays(bd, 1.0) - 0.001).abs() < 1e-12);
        assert!((add_microdays(bd, 1.0) - 0.000_001).abs() < 1e-15);
    }

    #[test]
    fn lerp_endpoints() {
        let a = 100.0_f64;
        let b = 200.0_f64;
        assert_eq!(lerp(a, b, 0.0), a);
        assert_eq!(lerp(a, b, 1.0), b);
        assert_eq!(lerp(a, b, 0.5), 150.0);
    }

    #[test]
    fn midpoint_symmetry() {
        assert_eq!(midpoint(0.0, 10.0), 5.0);
    }
}
