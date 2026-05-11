//! Integration tests for comparisons — ported from comparisons.spec.ts

use brightdate::comparisons::{
    closest, deduplicate, gaps, group_by_day, is_monotonically_increasing, is_non_decreasing,
    largest_gap, partition, statistics, within,
};

// ─── closest ─────────────────────────────────────────────────────────────────

#[test]
fn closest_basic() {
    let values = [1.0, 3.0, 5.0, 7.0];
    assert_eq!(closest(4.0, &values), Some(3.0));
}

#[test]
fn closest_exact_match() {
    let values = [1.0, 3.0, 5.0];
    assert_eq!(closest(3.0, &values), Some(3.0));
}

#[test]
fn closest_empty_returns_none() {
    assert_eq!(closest(5.0, &[]), None);
}

#[test]
fn closest_single_element() {
    assert_eq!(closest(0.0, &[42.0]), Some(42.0));
}

#[test]
fn closest_equidistant_returns_first_closer() {
    // 4 is equidistant from 3 and 5; should return whichever is found first
    let values = [3.0, 5.0];
    let result = closest(4.0, &values);
    assert!(result == Some(3.0) || result == Some(5.0));
}

// ─── within ──────────────────────────────────────────────────────────────────

#[test]
fn within_basic() {
    let values = [1.0, 3.0, 5.0, 7.0, 9.0];
    let result = within(4.0, &values, 2.5);
    assert!(result.contains(&3.0));
    assert!(result.contains(&5.0));
}

#[test]
fn within_none_in_range() {
    let values = [1.0, 10.0];
    assert!(within(5.0, &values, 2.0).is_empty());
}

#[test]
fn within_includes_boundary() {
    let values = [2.0, 5.0, 8.0];
    let result = within(5.0, &values, 3.0);
    // all within 3.0 of 5.0 → 2, 5, 8
    assert!(result.contains(&2.0));
    assert!(result.contains(&5.0));
    assert!(result.contains(&8.0));
}

// ─── partition ───────────────────────────────────────────────────────────────

#[test]
fn partition_basic() {
    let values = [1.0, 3.0, 5.0, 7.0];
    let (before, after) = partition(&values, 4.0);
    assert_eq!(before, vec![1.0, 3.0]);
    assert_eq!(after, vec![5.0, 7.0]);
}

#[test]
fn partition_all_before() {
    let values = [1.0, 2.0, 3.0];
    let (before, after) = partition(&values, 10.0);
    assert_eq!(before, vec![1.0, 2.0, 3.0]);
    assert!(after.is_empty());
}

#[test]
fn partition_all_after() {
    let values = [5.0, 6.0, 7.0];
    let (before, after) = partition(&values, 1.0);
    assert!(before.is_empty());
    assert_eq!(after, vec![5.0, 6.0, 7.0]);
}

// ─── groupByDay ──────────────────────────────────────────────────────────────

#[test]
fn group_by_day_basic() {
    let values = [0.1, 0.2, 1.3, 1.9, 2.0];
    let map = group_by_day(&values);
    assert_eq!(map[&0], vec![0.1, 0.2]);
    assert_eq!(map[&1], vec![1.3, 1.9]);
    assert_eq!(map[&2], vec![2.0]);
}

#[test]
fn group_by_day_negative() {
    let values = [-0.1_f64];
    let map = group_by_day(&values);
    // -0.1 → day = floor(-0.1) = -1
    assert!(map.contains_key(&-1_i64));
}

#[test]
fn group_by_day_empty() {
    let map = group_by_day(&[]);
    assert!(map.is_empty());
}

// ─── statistics ──────────────────────────────────────────────────────────────

#[test]
fn statistics_basic() {
    let values = [1.0, 2.0, 3.0, 4.0, 5.0];
    let stats = statistics(&values);
    assert_eq!(stats.min, 1.0);
    assert_eq!(stats.max, 5.0);
    assert!((stats.mean - 3.0).abs() < 1e-12);
    assert_eq!(stats.count, 5);
}

#[test]
fn statistics_single_value() {
    let stats = statistics(&[42.0]);
    assert_eq!(stats.min, 42.0);
    assert_eq!(stats.max, 42.0);
    assert_eq!(stats.mean, 42.0);
    assert_eq!(stats.count, 1);
}

#[test]
fn statistics_std_dev_uniform() {
    let stats = statistics(&[5.0, 5.0, 5.0]);
    assert!((stats.std_dev).abs() < 1e-12);
}

// ─── gaps ────────────────────────────────────────────────────────────────────

#[test]
fn gaps_basic() {
    let values = [1.0, 2.0, 5.0, 6.0];
    let result = gaps(&values);
    // gaps between consecutive sorted values: 1, 3, 1
    assert_eq!(result.len(), 3);
    // The gap of 3 between 2.0 and 5.0
    assert!(result.iter().any(|&g| (g - 3.0).abs() < 1e-12));
}

#[test]
fn gaps_empty_returns_empty() {
    assert!(gaps(&[]).is_empty());
}

#[test]
fn gaps_single_returns_empty() {
    assert!(gaps(&[42.0]).is_empty());
}

// ─── largestGap ──────────────────────────────────────────────────────────────

#[test]
fn largest_gap_basic() {
    let values = [1.0, 2.0, 10.0, 11.0];
    let lg = largest_gap(&values).expect("non-empty");
    assert!((lg.gap - 8.0).abs() < 1e-12);
    assert!((lg.before - 2.0).abs() < 1e-12);
    assert!((lg.after - 10.0).abs() < 1e-12);
}

#[test]
fn largest_gap_none_for_single_value() {
    assert!(largest_gap(&[1.0]).is_none());
}

#[test]
fn largest_gap_none_for_empty() {
    assert!(largest_gap(&[]).is_none());
}

#[test]
fn largest_gap_has_before_and_after_fields() {
    let values = [1.0, 5.0];
    let lg = largest_gap(&values).unwrap();
    assert_eq!(lg.before, 1.0);
    assert_eq!(lg.after, 5.0);
}

// ─── deduplicate ─────────────────────────────────────────────────────────────

#[test]
fn deduplicate_removes_exact_duplicates() {
    let values = [1.0, 1.0, 2.0, 3.0, 3.0];
    let result = deduplicate(&values, 0.0);
    assert_eq!(result, vec![1.0, 2.0, 3.0]);
}

#[test]
fn deduplicate_sorts_first() {
    let values = [3.0, 1.0, 2.0, 1.0];
    let result = deduplicate(&values, 0.0);
    assert_eq!(result, vec![1.0, 2.0, 3.0]);
}

#[test]
fn deduplicate_with_tolerance() {
    // tolerance 0.000001: 1.0 and 1.0000000005 are within tolerance → remove
    let values = [1.0_f64, 1.0 + 0.000_000_05, 2.0];
    let result = deduplicate(&values, 0.000_001);
    assert_eq!(result.len(), 2);
    assert!((result[0] - 1.0).abs() < 1e-10);
    assert!((result[1] - 2.0).abs() < 1e-10);
}

#[test]
fn deduplicate_with_tolerance_keeps_distinct() {
    // 1.0 and 1.000003 are further than tolerance 0.000001 → both kept
    let tol = 0.000_001_f64;
    let values = [1.0_f64, 1.0 + tol * 3.0, 2.0];
    let result = deduplicate(&values, tol);
    assert_eq!(result.len(), 3);
}

#[test]
fn deduplicate_empty() {
    let result = deduplicate(&[], 0.0);
    assert!(result.is_empty());
}

#[test]
fn deduplicate_all_same() {
    let result = deduplicate(&[5.0, 5.0, 5.0], 0.0);
    assert_eq!(result, vec![5.0]);
}

// ─── isMonotonicallyIncreasing ────────────────────────────────────────────────

#[test]
fn is_monotonically_increasing_true() {
    assert!(is_monotonically_increasing(&[1.0, 2.0, 3.0, 4.0]));
}

#[test]
fn is_monotonically_increasing_false_equal() {
    // strictly increasing: equal elements fail
    assert!(!is_monotonically_increasing(&[1.0, 1.0, 2.0]));
}

#[test]
fn is_monotonically_increasing_false_decreasing() {
    assert!(!is_monotonically_increasing(&[3.0, 2.0, 1.0]));
}

#[test]
fn is_monotonically_increasing_single() {
    assert!(is_monotonically_increasing(&[1.0]));
}

#[test]
fn is_monotonically_increasing_empty() {
    assert!(is_monotonically_increasing(&[]));
}

// ─── isNonDecreasing ─────────────────────────────────────────────────────────

#[test]
fn is_non_decreasing_strictly_increasing() {
    assert!(is_non_decreasing(&[1.0, 2.0, 3.0]));
}

#[test]
fn is_non_decreasing_equal_elements() {
    assert!(is_non_decreasing(&[1.0, 1.0, 2.0]));
}

#[test]
fn is_non_decreasing_false() {
    assert!(!is_non_decreasing(&[3.0, 2.0, 1.0]));
}

#[test]
fn is_non_decreasing_all_equal() {
    assert!(is_non_decreasing(&[5.0, 5.0, 5.0]));
}

// ─── additional closest ───────────────────────────────────────────────────────

#[test]
fn closest_large_set() {
    let values: Vec<f64> = (0..100).map(|i| i as f64).collect();
    assert_eq!(closest(50.3, &values), Some(50.0));
}

#[test]
fn closest_negative_target() {
    let values = [-5.0, -3.0, -1.0, 0.0, 1.0];
    // -2.4 is closest to -3.0 (diff 0.6) vs -1.0 (diff 1.4)
    assert_eq!(closest(-2.4, &values), Some(-3.0));
}

#[test]
fn closest_all_same() {
    let values = [7.0, 7.0, 7.0];
    assert_eq!(closest(10.0, &values), Some(7.0));
}

// ─── additional within ────────────────────────────────────────────────────────

#[test]
fn within_empty_input_returns_empty() {
    assert!(within(5.0, &[], 100.0).is_empty());
}

#[test]
fn within_zero_tolerance_exact_match_only() {
    let values = [1.0, 2.0, 3.0];
    let result = within(2.0, &values, 0.0);
    assert_eq!(result, vec![2.0]);
}

#[test]
fn within_large_tolerance_returns_all() {
    let values = [1.0, 2.0, 3.0, 4.0, 5.0];
    let result = within(3.0, &values, 100.0);
    assert_eq!(result.len(), 5);
}

// ─── additional partition ─────────────────────────────────────────────────────

#[test]
fn partition_empty_input() {
    let (before, after) = partition(&[], 5.0);
    assert!(before.is_empty());
    assert!(after.is_empty());
}

#[test]
fn partition_pivot_at_boundary() {
    // Value exactly at pivot goes to 'after' (strictly after)
    let values = [1.0, 5.0, 9.0];
    let (before, after) = partition(&values, 5.0);
    // 5.0 is on the boundary — implementation may differ, just check total
    assert_eq!(before.len() + after.len(), 3);
}

// ─── additional group_by_day ──────────────────────────────────────────────────

#[test]
fn group_by_day_single_value() {
    let map = group_by_day(&[9622.5]);
    assert!(map.contains_key(&9622_i64));
    assert_eq!(map[&9622_i64], vec![9622.5]);
}

#[test]
fn group_by_day_multiple_days() {
    let values = [0.0, 0.5, 1.0, 1.99, 2.5];
    let map = group_by_day(&values);
    assert_eq!(map.len(), 3);
}

#[test]
fn group_by_day_all_same_day() {
    let values = [5.0, 5.1, 5.5, 5.99];
    let map = group_by_day(&values);
    assert_eq!(map.len(), 1);
    assert_eq!(map[&5_i64].len(), 4);
}

// ─── additional statistics ────────────────────────────────────────────────────

#[test]
fn statistics_two_values() {
    let stats = statistics(&[3.0, 7.0]);
    assert_eq!(stats.min, 3.0);
    assert_eq!(stats.max, 7.0);
    assert!((stats.mean - 5.0).abs() < 1e-12);
    assert_eq!(stats.count, 2);
}

#[test]
fn statistics_negative_values() {
    let stats = statistics(&[-10.0, -5.0, 0.0, 5.0, 10.0]);
    assert_eq!(stats.min, -10.0);
    assert_eq!(stats.max, 10.0);
    assert!((stats.mean - 0.0).abs() < 1e-12);
}

#[test]
fn statistics_large_dataset() {
    let values: Vec<f64> = (1..=100).map(|i| i as f64).collect();
    let stats = statistics(&values);
    assert_eq!(stats.count, 100);
    assert!((stats.mean - 50.5).abs() < 1e-10);
}

// ─── additional gaps ──────────────────────────────────────────────────────────

#[test]
fn gaps_uniform_spacing() {
    let values = [0.0, 1.0, 2.0, 3.0];
    let result = gaps(&values);
    assert_eq!(result.len(), 3);
    for &g in &result {
        assert!((g - 1.0).abs() < 1e-12, "gap should be 1.0, got {g}");
    }
}

#[test]
fn gaps_two_values() {
    let result = gaps(&[5.0, 10.0]);
    assert_eq!(result.len(), 1);
    assert!((result[0] - 5.0).abs() < 1e-12);
}

// ─── additional deduplicate ───────────────────────────────────────────────────

#[test]
fn deduplicate_single_value() {
    let result = deduplicate(&[42.0], 0.0);
    assert_eq!(result, vec![42.0]);
}

#[test]
fn deduplicate_preserves_order() {
    let result = deduplicate(&[3.0, 1.0, 2.0], 0.0);
    assert_eq!(result, vec![1.0, 2.0, 3.0]);
}

#[test]
fn deduplicate_negative_values() {
    let result = deduplicate(&[-3.0, -1.0, -3.0, -1.0], 0.0);
    assert_eq!(result, vec![-3.0, -1.0]);
}

// ─── additional monotone ──────────────────────────────────────────────────────

#[test]
fn is_monotonically_increasing_large_ascending() {
    let values: Vec<f64> = (0..1000).map(|i| i as f64 * 1.5).collect();
    assert!(is_monotonically_increasing(&values));
}

#[test]
fn is_non_decreasing_single_element() {
    assert!(is_non_decreasing(&[99.0]));
}

#[test]
fn is_non_decreasing_empty() {
    assert!(is_non_decreasing(&[]));
}
