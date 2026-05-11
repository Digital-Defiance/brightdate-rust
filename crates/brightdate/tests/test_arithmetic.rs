//! Integration tests for arithmetic — ported from arithmetic.spec.ts

use brightdate::arithmetic::{
    absolute_difference, add, add_microdays, add_millidays, ceil_to_day, clamp, compare,
    difference, equals, floor_to_day, is_in_range, lerp, linspace, max_of, midpoint, min_of,
    round_to_microday, round_to_milliday, sort, subtract, whole_days_between,
};
use std::cmp::Ordering;

// ─── add ─────────────────────────────────────────────────────────────────────

#[test]
fn add_basic() {
    assert!((add(100.0, 5.0) - 105.0).abs() < 1e-12);
}

#[test]
fn add_negative() {
    assert!((add(100.0, -10.0) - 90.0).abs() < 1e-12);
}

#[test]
fn add_zero() {
    assert!((add(9622.5, 0.0) - 9622.5).abs() < 1e-12);
}

// ─── subtract ────────────────────────────────────────────────────────────────

#[test]
fn subtract_basic() {
    assert!((subtract(100.0, 5.0) - 95.0).abs() < 1e-12);
}

#[test]
fn subtract_to_negative() {
    assert!((subtract(3.0, 5.0) - -2.0).abs() < 1e-12);
}

// ─── addMillidays ────────────────────────────────────────────────────────────

#[test]
fn add_millidays_basic() {
    let result = add_millidays(9622.0, 500.0);
    assert!((result - 9622.5).abs() < 1e-10);
}

#[test]
fn add_millidays_fractional() {
    let result = add_millidays(0.0, 1.0);
    assert!((result - 0.001).abs() < 1e-15);
}

// ─── addMicrodays ────────────────────────────────────────────────────────────

#[test]
fn add_microdays_basic() {
    let result = add_microdays(0.0, 1.0);
    assert!((result - 0.000_001).abs() < 1e-15);
}

// ─── difference ──────────────────────────────────────────────────────────────

#[test]
fn difference_positive() {
    assert!((difference(10.0, 3.0) - 7.0).abs() < 1e-12);
}

#[test]
fn difference_negative() {
    assert!((difference(3.0, 10.0) - -7.0).abs() < 1e-12);
}

#[test]
fn difference_zero() {
    assert_eq!(difference(5.0, 5.0), 0.0);
}

// ─── absoluteDifference ──────────────────────────────────────────────────────

#[test]
fn absolute_difference_positive() {
    assert!((absolute_difference(3.0, 10.0) - 7.0).abs() < 1e-12);
}

#[test]
fn absolute_difference_already_positive() {
    assert!((absolute_difference(10.0, 3.0) - 7.0).abs() < 1e-12);
}

#[test]
fn absolute_difference_symmetric() {
    assert_eq!(absolute_difference(3.0, 7.0), absolute_difference(7.0, 3.0));
}

// ─── compare ─────────────────────────────────────────────────────────────────

#[test]
fn compare_less() {
    assert_eq!(compare(1.0, 2.0), Ordering::Less);
}

#[test]
fn compare_greater() {
    assert_eq!(compare(2.0, 1.0), Ordering::Greater);
}

#[test]
fn compare_equal() {
    assert_eq!(compare(5.0, 5.0), Ordering::Equal);
}

#[test]
fn compare_negative_values() {
    assert_eq!(compare(-10.0, -1.0), Ordering::Less);
}

// ─── equals ──────────────────────────────────────────────────────────────────

#[test]
fn equals_identical() {
    assert!(equals(5.0, 5.0, None));
}

#[test]
fn equals_within_default_tolerance() {
    // default tolerance 1e-6; 0.0000005 < 1e-6
    assert!(equals(5.0, 5.0 + 0.000_000_5, None));
}

#[test]
fn equals_outside_default_tolerance() {
    assert!(!equals(5.0, 5.0 + 0.000_002, None));
}

#[test]
fn equals_custom_tolerance_true() {
    assert!(equals(5.0, 5.0 + 0.01, Some(0.1)));
}

#[test]
fn equals_custom_tolerance_false() {
    assert!(!equals(5.0, 5.0 + 0.2, Some(0.1)));
}

// ─── isInRange ───────────────────────────────────────────────────────────────

#[test]
fn is_in_range_within() {
    assert!(is_in_range(5.0, 1.0, 10.0));
}

#[test]
fn is_in_range_at_start() {
    assert!(is_in_range(1.0, 1.0, 10.0));
}

#[test]
fn is_in_range_at_end() {
    assert!(is_in_range(10.0, 1.0, 10.0));
}

#[test]
fn is_in_range_below() {
    assert!(!is_in_range(0.0, 1.0, 10.0));
}

#[test]
fn is_in_range_above() {
    assert!(!is_in_range(11.0, 1.0, 10.0));
}

// ─── min_of ──────────────────────────────────────────────────────────────────

#[test]
fn min_of_basic() {
    assert_eq!(min_of(&[3.0, 1.0, 4.0, 1.0, 5.0]), 1.0);
}

#[test]
fn min_of_single() {
    assert_eq!(min_of(&[42.0]), 42.0);
}

#[test]
#[should_panic(expected = "Cannot find minimum of empty array")]
fn min_of_empty_panics() {
    min_of(&[]);
}

#[test]
fn min_of_negative() {
    assert_eq!(min_of(&[-5.0, -1.0, -10.0]), -10.0);
}

// ─── max_of ──────────────────────────────────────────────────────────────────

#[test]
fn max_of_basic() {
    assert_eq!(max_of(&[3.0, 1.0, 4.0, 1.0, 5.0]), 5.0);
}

#[test]
fn max_of_single() {
    assert_eq!(max_of(&[42.0]), 42.0);
}

#[test]
#[should_panic(expected = "Cannot find maximum of empty array")]
fn max_of_empty_panics() {
    max_of(&[]);
}

// ─── clamp ───────────────────────────────────────────────────────────────────

#[test]
fn clamp_within() {
    assert_eq!(clamp(5.0, 1.0, 10.0), 5.0);
}

#[test]
fn clamp_below() {
    assert_eq!(clamp(-5.0, 1.0, 10.0), 1.0);
}

#[test]
fn clamp_above() {
    assert_eq!(clamp(15.0, 1.0, 10.0), 10.0);
}

#[test]
fn clamp_at_lower() {
    assert_eq!(clamp(1.0, 1.0, 10.0), 1.0);
}

#[test]
fn clamp_at_upper() {
    assert_eq!(clamp(10.0, 1.0, 10.0), 10.0);
}

// ─── lerp ────────────────────────────────────────────────────────────────────

#[test]
fn lerp_t0() {
    assert_eq!(lerp(10.0, 20.0, 0.0), 10.0);
}

#[test]
fn lerp_t1() {
    assert_eq!(lerp(10.0, 20.0, 1.0), 20.0);
}

#[test]
fn lerp_t_half() {
    assert_eq!(lerp(10.0, 20.0, 0.5), 15.0);
}

#[test]
fn lerp_t_quarter() {
    assert!((lerp(0.0, 100.0, 0.25) - 25.0).abs() < 1e-12);
}

#[test]
fn lerp_negative() {
    assert!((lerp(-10.0, 10.0, 0.5) - 0.0).abs() < 1e-12);
}

// ─── midpoint ────────────────────────────────────────────────────────────────

#[test]
fn midpoint_basic() {
    assert_eq!(midpoint(10.0, 20.0), 15.0);
}

#[test]
fn midpoint_equal() {
    assert_eq!(midpoint(5.0, 5.0), 5.0);
}

#[test]
fn midpoint_negative() {
    assert_eq!(midpoint(-10.0, 10.0), 0.0);
}

#[test]
fn midpoint_commutative() {
    assert_eq!(midpoint(3.0, 7.0), midpoint(7.0, 3.0));
}

// ─── linspace ────────────────────────────────────────────────────────────────

#[test]
fn linspace_5_values() {
    let result = linspace(0.0, 1.0, 5);
    assert_eq!(result.len(), 5);
    assert_eq!(result[0], 0.0);
    assert_eq!(result[4], 1.0);
    assert!((result[2] - 0.5).abs() < 1e-12);
}

#[test]
fn linspace_count_less_than_2_returns_start() {
    assert_eq!(linspace(5.0, 10.0, 1), vec![5.0]);
}

#[test]
fn linspace_count_2() {
    assert_eq!(linspace(0.0, 10.0, 2), vec![0.0, 10.0]);
}

#[test]
fn linspace_correct_step() {
    let result = linspace(0.0, 3.0, 4);
    assert!((result[1] - 1.0).abs() < 1e-12);
    assert!((result[2] - 2.0).abs() < 1e-12);
}

// ─── sort ────────────────────────────────────────────────────────────────────

#[test]
fn sort_ascending() {
    assert_eq!(sort(&[3.0, 1.0, 4.0, 1.0, 5.0]), vec![1.0, 1.0, 3.0, 4.0, 5.0]);
}

#[test]
fn sort_non_mutating() {
    let original = vec![3.0, 1.0, 2.0];
    let _ = sort(&original);
    assert_eq!(original, vec![3.0, 1.0, 2.0]);
}

#[test]
fn sort_empty() {
    let v: Vec<f64> = vec![];
    assert_eq!(sort(&v), Vec::<f64>::new());
}

#[test]
fn sort_single() {
    assert_eq!(sort(&[42.0]), vec![42.0]);
}

// ─── floorToDay ──────────────────────────────────────────────────────────────

#[test]
fn floor_to_day_basic() {
    assert_eq!(floor_to_day(9622.75), 9622.0);
}

#[test]
fn floor_to_day_integer_unchanged() {
    assert_eq!(floor_to_day(100.0), 100.0);
}

#[test]
fn floor_to_day_negative() {
    assert_eq!(floor_to_day(-0.5), -1.0);
}

// ─── ceilToDay ───────────────────────────────────────────────────────────────

#[test]
fn ceil_to_day_basic() {
    assert_eq!(ceil_to_day(9622.25), 9623.0);
}

#[test]
fn ceil_to_day_integer_unchanged() {
    assert_eq!(ceil_to_day(100.0), 100.0);
}

#[test]
fn ceil_to_day_negative() {
    assert!((ceil_to_day(-0.5) - 0.0).abs() < 1e-12);
}

// ─── roundToMilliday ─────────────────────────────────────────────────────────

#[test]
fn round_to_milliday_basic() {
    assert!((round_to_milliday(9622.5004) - 9622.5).abs() < 0.001);
}

#[test]
fn round_to_milliday_up() {
    assert!((round_to_milliday(0.0005) - 0.001).abs() < 0.001);
}

#[test]
fn round_to_milliday_integer_unchanged() {
    assert_eq!(round_to_milliday(100.0), 100.0);
}

// ─── roundToMicroday ─────────────────────────────────────────────────────────

#[test]
fn round_to_microday_basic() {
    let result = round_to_microday(9622.5004005);
    assert!((result - 9622.500_401).abs() < 0.000_001);
}

// ─── wholeDaysBetween ────────────────────────────────────────────────────────

#[test]
fn whole_days_between_basic() {
    assert_eq!(whole_days_between(10.0, 13.7), 3);
}

#[test]
fn whole_days_between_same() {
    assert_eq!(whole_days_between(5.0, 5.0), 0);
}

#[test]
fn whole_days_between_negative() {
    // from=13, to=10 → diff = -3.0, trunc = -3
    assert_eq!(whole_days_between(13.0, 10.0), -3);
}

// ─── additional add / subtract ────────────────────────────────────────────────

#[test]
fn add_chained() {
    let result = add(add(1.0, 2.0), 3.0);
    assert!((result - 6.0).abs() < 1e-12);
}

#[test]
fn add_large_values() {
    let result = add(1_000_000.0, 0.5);
    assert!((result - 1_000_000.5).abs() < 1e-9);
}

#[test]
fn subtract_chained() {
    let result = subtract(subtract(100.0, 10.0), 10.0);
    assert!((result - 80.0).abs() < 1e-12);
}

#[test]
fn subtract_large() {
    let result = subtract(1_000_000.0, 500_000.0);
    assert!((result - 500_000.0).abs() < 1e-9);
}

#[test]
fn add_then_subtract_roundtrip() {
    let start = 9622.5;
    let result = subtract(add(start, 365.0), 365.0);
    assert!((result - start).abs() < 1e-10);
}

// ─── additional addMillidays / addMicrodays ────────────────────────────────────

#[test]
fn add_millidays_zero() {
    assert!((add_millidays(9622.0, 0.0) - 9622.0).abs() < 1e-12);
}

#[test]
fn add_millidays_negative() {
    let result = add_millidays(9622.5, -500.0);
    assert!((result - 9622.0).abs() < 1e-10);
}

#[test]
fn add_microdays_zero() {
    assert!((add_microdays(9622.0, 0.0) - 9622.0).abs() < 1e-12);
}

#[test]
fn add_microdays_1000_equals_1_milliday() {
    let via_micro = add_microdays(0.0, 1000.0);
    let via_milli = add_millidays(0.0, 1.0);
    assert!((via_micro - via_milli).abs() < 1e-12);
}

// ─── additional absoluteDifference ────────────────────────────────────────────

#[test]
fn absolute_difference_ten_minus_three() {
    assert!((absolute_difference(10.0, 3.0) - 7.0).abs() < 1e-12);
}

#[test]
fn absolute_difference_negative_order() {
    assert!((absolute_difference(3.0, 10.0) - 7.0).abs() < 1e-12);
}

#[test]
fn absolute_difference_zero() {
    assert_eq!(absolute_difference(5.0, 5.0), 0.0);
}

#[test]
fn absolute_difference_commutative() {
    let a = absolute_difference(1.5, 9.5);
    let b = absolute_difference(9.5, 1.5);
    assert!((a - b).abs() < 1e-12);
}

// ─── additional compare ────────────────────────────────────────────────────────

#[test]
fn compare_less_additional() {
    assert_eq!(compare(1.0, 2.0), Ordering::Less);
}

#[test]
fn compare_equal_additional() {
    assert_eq!(compare(3.0, 3.0), Ordering::Equal);
}

#[test]
fn compare_greater_additional() {
    assert_eq!(compare(5.0, 2.0), Ordering::Greater);
}

// ─── additional equals ────────────────────────────────────────────────────────

#[test]
fn equals_exact_match() {
    assert!(equals(5.0, 5.0, Some(1e-12)));
}

#[test]
fn equals_large_diff_fails() {
    assert!(!equals(5.0, 6.0, Some(1e-12)));
}

#[test]
fn equals_tolerance_boundary_check() {
    assert!(equals(1.0, 1.0 + 1e-10, Some(1e-9)));
    assert!(!equals(1.0, 1.0 + 1e-9, Some(1e-10)));
}

// ─── additional floor/ceil/round ──────────────────────────────────────────────

#[test]
fn floor_to_day_fraction_just_below_1() {
    assert_eq!(floor_to_day(9622.9999), 9622.0);
}

#[test]
fn ceil_to_day_fraction_just_above_0() {
    assert_eq!(ceil_to_day(9622.0001), 9623.0);
}

#[test]
fn round_to_milliday_exactly_half_rounds() {
    // 0.0005 should round to 0.001 or 0.000 depending on banker's rounding
    let result = round_to_milliday(0.5005);
    assert!((result - 0.5).abs() < 0.002);
}

// ─── additional clamp ─────────────────────────────────────────────────────────

#[test]
fn clamp_large_range() {
    assert_eq!(clamp(500.0, 0.0, 1000.0), 500.0);
}

#[test]
fn clamp_equal_min_max() {
    assert_eq!(clamp(50.0, 10.0, 10.0), 10.0);
}

// ─── additional linspace ──────────────────────────────────────────────────────

#[test]
fn linspace_uniform_spacing() {
    let result = linspace(0.0, 4.0, 5);
    for (i, &v) in result.iter().enumerate() {
        assert!((v - i as f64).abs() < 1e-12, "v={v} at i={i}");
    }
}

#[test]
fn linspace_10_values() {
    let result = linspace(0.0, 9.0, 10);
    assert_eq!(result.len(), 10);
    assert!((result[9] - 9.0).abs() < 1e-12);
}

// ─── additional wholeDaysBetween ──────────────────────────────────────────────

#[test]
fn whole_days_between_fractional_span() {
    // 9622.0 to 9622.9 = 0.9 → 0 whole days
    assert_eq!(whole_days_between(9622.0, 9622.9), 0);
}

#[test]
fn whole_days_between_exactly_one() {
    assert_eq!(whole_days_between(100.0, 101.0), 1);
}

#[test]
fn whole_days_between_large_span() {
    assert_eq!(whole_days_between(0.0, 365.0), 365);
}
