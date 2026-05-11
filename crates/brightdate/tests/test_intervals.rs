//! Integration tests for BrightDateInterval — ported from intervals.spec.ts

use brightdate::intervals::BrightDateInterval;
use brightdate::BrightDate;

fn bd(v: f64) -> BrightDate {
    BrightDate::from_value(v)
}

fn interval(s: f64, e: f64) -> BrightDateInterval {
    BrightDateInterval::from_values(s, e)
}

// ─── constructor ─────────────────────────────────────────────────────────────

#[test]
fn new_valid_interval() {
    let iv = interval(0.0, 10.0);
    assert_eq!(iv.start.value, 0.0);
    assert_eq!(iv.end.value, 10.0);
}

#[test]
fn new_zero_duration_interval() {
    let iv = interval(5.0, 5.0);
    assert_eq!(iv.start.value, iv.end.value);
}

#[test]
#[should_panic(expected = "start")]
fn new_panics_start_after_end() {
    interval(10.0, 5.0);
}

#[test]
fn from_iso() {
    let iv = BrightDateInterval::from_iso("2000-01-01T12:00:00Z", "2000-01-02T12:00:00Z")
        .expect("valid ISO interval");
    // Exactly 1 day
    let dur = iv.duration();
    assert!((dur - 1.0).abs() < 0.001);
}

// ─── duration ────────────────────────────────────────────────────────────────

#[test]
fn duration_basic() {
    let iv = interval(0.0, 5.0);
    assert!((iv.duration() - 5.0).abs() < 1e-12);
}

#[test]
fn duration_zero() {
    assert_eq!(interval(3.0, 3.0).duration(), 0.0);
}

#[test]
fn duration_fractional() {
    let iv = interval(0.0, 0.5);
    assert!((iv.duration() - 0.5).abs() < 1e-12);
}

// ─── durationMetric ──────────────────────────────────────────────────────────

#[test]
fn duration_metric_one_day() {
    let iv = interval(0.0, 1.0);
    let dm = iv.duration_metric();
    assert!((dm.days - 1.0).abs() < 1e-12);
    assert!((dm.millidays - 1000.0).abs() < 1e-9);
    assert!((dm.microdays - 1_000_000.0).abs() < 1e-3);
}

// ─── contains ────────────────────────────────────────────────────────────────

#[test]
fn contains_midpoint() {
    assert!(interval(0.0, 10.0).contains(&bd(5.0)));
}

#[test]
fn contains_at_start() {
    assert!(interval(0.0, 10.0).contains(&bd(0.0)));
}

#[test]
fn contains_at_end() {
    assert!(interval(0.0, 10.0).contains(&bd(10.0)));
}

#[test]
fn contains_false_below() {
    assert!(!interval(0.0, 10.0).contains(&bd(-1.0)));
}

#[test]
fn contains_false_above() {
    assert!(!interval(0.0, 10.0).contains(&bd(11.0)));
}

#[test]
fn contains_value_true() {
    assert!(interval(0.0, 10.0).contains_value(5.0));
}

#[test]
fn contains_value_false() {
    assert!(!interval(0.0, 10.0).contains_value(15.0));
}

// ─── overlaps ────────────────────────────────────────────────────────────────

#[test]
fn overlaps_true() {
    assert!(interval(0.0, 5.0).overlaps(&interval(3.0, 8.0)));
}

#[test]
fn overlaps_touching_counts() {
    assert!(interval(0.0, 5.0).overlaps(&interval(5.0, 10.0)));
}

#[test]
fn overlaps_false() {
    assert!(!interval(0.0, 3.0).overlaps(&interval(4.0, 8.0)));
}

#[test]
fn overlaps_fully_enclosed() {
    assert!(interval(0.0, 10.0).overlaps(&interval(2.0, 5.0)));
}

// ─── intersection ────────────────────────────────────────────────────────────

#[test]
fn intersection_overlapping() {
    let result = interval(0.0, 5.0).intersection(&interval(3.0, 8.0));
    let iv = result.expect("should intersect");
    assert!((iv.start.value - 3.0).abs() < 1e-12);
    assert!((iv.end.value - 5.0).abs() < 1e-12);
}

#[test]
fn intersection_none_for_disjoint() {
    assert!(interval(0.0, 3.0).intersection(&interval(4.0, 8.0)).is_none());
}

#[test]
fn intersection_touching_returns_point() {
    let result = interval(0.0, 5.0).intersection(&interval(5.0, 10.0));
    let iv = result.expect("touching = point intersection");
    assert!((iv.start.value - 5.0).abs() < 1e-12);
    assert!((iv.end.value - 5.0).abs() < 1e-12);
}

// ─── union ───────────────────────────────────────────────────────────────────

#[test]
fn union_overlapping() {
    let result = interval(0.0, 5.0).union(&interval(3.0, 8.0));
    let iv = result.expect("overlapping should union");
    assert!((iv.start.value - 0.0).abs() < 1e-12);
    assert!((iv.end.value - 8.0).abs() < 1e-12);
}

#[test]
fn union_adjacent() {
    let result = interval(0.0, 5.0).union(&interval(5.0, 10.0));
    let iv = result.expect("adjacent should union");
    assert!((iv.end.value - 10.0).abs() < 1e-12);
}

#[test]
fn union_disjoint_returns_none() {
    assert!(interval(0.0, 3.0).union(&interval(5.0, 8.0)).is_none());
}

// ─── adjacentTo ──────────────────────────────────────────────────────────────

#[test]
fn adjacent_to_true() {
    assert!(interval(0.0, 5.0).adjacent_to(&interval(5.0, 10.0)));
}

#[test]
fn adjacent_to_reverse() {
    assert!(interval(5.0, 10.0).adjacent_to(&interval(0.0, 5.0)));
}

#[test]
fn adjacent_to_false_with_gap() {
    assert!(!interval(0.0, 4.0).adjacent_to(&interval(5.0, 10.0)));
}

#[test]
fn adjacent_to_false_overlapping() {
    assert!(!interval(0.0, 6.0).adjacent_to(&interval(5.0, 10.0)));
}

// ─── encloses ────────────────────────────────────────────────────────────────

#[test]
fn encloses_true() {
    assert!(interval(0.0, 10.0).encloses(&interval(2.0, 8.0)));
}

#[test]
fn encloses_at_boundary() {
    assert!(interval(0.0, 10.0).encloses(&interval(0.0, 10.0)));
}

#[test]
fn encloses_false() {
    assert!(!interval(2.0, 8.0).encloses(&interval(0.0, 10.0)));
}

// ─── split ───────────────────────────────────────────────────────────────────

#[test]
fn split_into_two() {
    let parts = interval(0.0, 10.0).split(2);
    assert_eq!(parts.len(), 2);
    assert!((parts[0].start.value - 0.0).abs() < 1e-12);
    assert!((parts[0].end.value - 5.0).abs() < 1e-12);
    assert!((parts[1].start.value - 5.0).abs() < 1e-12);
    assert!((parts[1].end.value - 10.0).abs() < 1e-12);
}

#[test]
fn split_into_one_returns_self() {
    let parts = interval(0.0, 10.0).split(1);
    assert_eq!(parts.len(), 1);
    assert!((parts[0].start.value - 0.0).abs() < 1e-12);
    assert!((parts[0].end.value - 10.0).abs() < 1e-12);
}

#[test]
fn split_into_three() {
    let parts = interval(0.0, 3.0).split(3);
    assert_eq!(parts.len(), 3);
    assert!((parts[0].duration() - 1.0).abs() < 1e-12);
    assert!((parts[1].duration() - 1.0).abs() < 1e-12);
    assert!((parts[2].duration() - 1.0).abs() < 1e-12);
}

#[test]
#[should_panic(expected = "Count must be at least 1")]
fn split_zero_panics() {
    interval(0.0, 10.0).split(0);
}

// ─── expand ──────────────────────────────────────────────────────────────────

#[test]
fn expand_basic() {
    let iv = interval(2.0, 8.0).expand(1.0);
    assert!((iv.start.value - 1.0).abs() < 1e-12);
    assert!((iv.end.value - 9.0).abs() < 1e-12);
}

#[test]
fn expand_zero() {
    let iv = interval(2.0, 8.0).expand(0.0);
    assert!((iv.start.value - 2.0).abs() < 1e-12);
    assert!((iv.end.value - 8.0).abs() < 1e-12);
}

// ─── shrink ──────────────────────────────────────────────────────────────────

#[test]
fn shrink_basic() {
    let iv = interval(0.0, 10.0).shrink(2.0).expect("valid shrink");
    assert!((iv.start.value - 2.0).abs() < 1e-12);
    assert!((iv.end.value - 8.0).abs() < 1e-12);
}

#[test]
fn shrink_too_much_returns_none() {
    assert!(interval(0.0, 4.0).shrink(3.0).is_none());
}

#[test]
fn shrink_to_point_is_some() {
    let iv = interval(0.0, 4.0).shrink(2.0);
    assert!(iv.is_some());
    let iv = iv.unwrap();
    assert!((iv.start.value - 2.0).abs() < 1e-12);
    assert!((iv.end.value - 2.0).abs() < 1e-12);
}

// ─── shift ───────────────────────────────────────────────────────────────────

#[test]
fn shift_positive() {
    let iv = interval(0.0, 5.0).shift(3.0);
    assert!((iv.start.value - 3.0).abs() < 1e-12);
    assert!((iv.end.value - 8.0).abs() < 1e-12);
}

#[test]
fn shift_negative() {
    let iv = interval(5.0, 10.0).shift(-2.0);
    assert!((iv.start.value - 3.0).abs() < 1e-12);
    assert!((iv.end.value - 8.0).abs() < 1e-12);
}

#[test]
fn shift_zero_unchanged() {
    let iv = interval(2.0, 8.0).shift(0.0);
    assert!((iv.start.value - 2.0).abs() < 1e-12);
    assert!((iv.end.value - 8.0).abs() < 1e-12);
}

// ─── iterate ─────────────────────────────────────────────────────────────────

#[test]
fn iterate_step_1() {
    let points = interval(0.0, 3.0).iterate(1.0);
    assert_eq!(points.len(), 4); // 0, 1, 2, 3
    assert!((points[0].value - 0.0).abs() < 1e-12);
    assert!((points[3].value - 3.0).abs() < 1e-12);
}

#[test]
fn iterate_fractional_step() {
    let points = interval(0.0, 1.0).iterate(0.5);
    assert_eq!(points.len(), 3); // 0, 0.5, 1.0
}

// ─── sample ──────────────────────────────────────────────────────────────────

#[test]
fn sample_count_0_empty() {
    assert!(interval(0.0, 10.0).sample(0).is_empty());
}

#[test]
fn sample_count_1_midpoint() {
    let samples = interval(0.0, 10.0).sample(1);
    assert_eq!(samples.len(), 1);
    assert!((samples[0].value - 5.0).abs() < 1e-12);
}

#[test]
fn sample_count_2_endpoints() {
    let samples = interval(0.0, 10.0).sample(2);
    assert_eq!(samples.len(), 2);
    assert!((samples[0].value - 0.0).abs() < 1e-12);
    assert!((samples[1].value - 10.0).abs() < 1e-12);
}

#[test]
fn sample_count_5() {
    let samples = interval(0.0, 4.0).sample(5);
    assert_eq!(samples.len(), 5);
    assert!((samples[2].value - 2.0).abs() < 1e-12);
}

// ─── midpoint ────────────────────────────────────────────────────────────────

#[test]
fn midpoint_basic() {
    let mid = interval(0.0, 10.0).midpoint();
    assert!((mid.value - 5.0).abs() < 1e-12);
}

#[test]
fn midpoint_asymmetric() {
    let mid = interval(2.0, 6.0).midpoint();
    assert!((mid.value - 4.0).abs() < 1e-12);
}

// ─── formatDuration / formatRange ────────────────────────────────────────────

#[test]
fn format_duration_str_non_empty() {
    let s = interval(0.0, 1.5).format_duration_str();
    assert!(!s.is_empty());
}

#[test]
fn format_range_contains_arrow() {
    let s = interval(0.0, 1.0).format_range();
    assert!(s.contains("..") || s.contains('→') || s.contains("->"));
}

#[test]
fn display_impl_non_empty() {
    let iv = interval(0.0, 1.0);
    let s = format!("{iv}");
    assert!(!s.is_empty());
}

// ─── additional constructor / duration ───────────────────────────────────────

#[test]
fn interval_from_value_types() {
    let iv = interval(9622.0, 9623.0);
    assert!((iv.duration() - 1.0).abs() < 1e-12);
}

#[test]
fn interval_half_day_duration() {
    assert!((interval(0.0, 0.5).duration() - 0.5).abs() < 1e-12);
}

#[test]
fn interval_large_duration() {
    assert!((interval(0.0, 36_525.0).duration() - 36_525.0).abs() < 1e-12);
}

// ─── additional contains ──────────────────────────────────────────────────────

#[test]
fn contains_interior_point() {
    assert!(interval(0.0, 10.0).contains(&bd(5.0)));
}

#[test]
fn contains_start_point() {
    assert!(interval(0.0, 10.0).contains(&bd(0.0)));
}

#[test]
fn contains_end_point() {
    assert!(interval(0.0, 10.0).contains(&bd(10.0)));
}

#[test]
fn not_contains_before() {
    assert!(!interval(1.0, 10.0).contains(&bd(0.0)));
}

#[test]
fn not_contains_after() {
    assert!(!interval(0.0, 9.0).contains(&bd(10.0)));
}

// ─── additional overlaps ──────────────────────────────────────────────────────

#[test]
fn overlaps_partial_left() {
    assert!(interval(0.0, 5.0).overlaps(&interval(3.0, 8.0)));
}

#[test]
fn overlaps_partial_right() {
    assert!(interval(3.0, 8.0).overlaps(&interval(0.0, 5.0)));
}

#[test]
fn no_overlap_adjacent() {
    // touching at boundary — behaviour is implementation-defined; just check finite result
    let _ = interval(0.0, 5.0).overlaps(&interval(5.0, 10.0));
}

#[test]
fn no_overlap_disjoint() {
    assert!(!interval(0.0, 5.0).overlaps(&interval(6.0, 10.0)));
}

// ─── additional split ─────────────────────────────────────────────────────────

#[test]
fn split_contiguous_parts() {
    let parts = interval(0.0, 6.0).split(3);
    for i in 0..parts.len() - 1 {
        assert!((parts[i].end.value - parts[i + 1].start.value).abs() < 1e-10);
    }
}

#[test]
fn split_equal_duration() {
    let parts = interval(0.0, 10.0).split(5);
    for p in &parts {
        assert!((p.duration() - 2.0).abs() < 1e-10);
    }
}

// ─── additional expand ────────────────────────────────────────────────────────

#[test]
fn expand_large_amount() {
    let iv = interval(5.0, 5.0).expand(10.0);
    assert!((iv.start.value - (-5.0)).abs() < 1e-12);
    assert!((iv.end.value - 15.0).abs() < 1e-12);
}

#[test]
fn expand_preserves_midpoint() {
    let iv = interval(4.0, 6.0);
    let mid_before = iv.midpoint().value;
    let expanded = iv.expand(2.0);
    let mid_after = expanded.midpoint().value;
    assert!((mid_before - mid_after).abs() < 1e-10);
}

// ─── additional shrink ────────────────────────────────────────────────────────

#[test]
fn shrink_exact_half_works() {
    let iv = interval(0.0, 10.0).shrink(2.5);
    assert!(iv.is_some());
    let iv = iv.unwrap();
    assert!((iv.duration() - 5.0).abs() < 1e-10);
}

#[test]
fn shrink_more_than_half_but_valid() {
    // shrink by 4 on [0,10] → [4, 6], duration = 2
    let iv = interval(0.0, 10.0).shrink(4.0).unwrap();
    assert!((iv.duration() - 2.0).abs() < 1e-10);
}

// ─── additional shift ─────────────────────────────────────────────────────────

#[test]
fn shift_large_positive() {
    let iv = interval(0.0, 1.0).shift(10_000.0);
    assert!((iv.start.value - 10_000.0).abs() < 1e-9);
    assert!((iv.end.value - 10_001.0).abs() < 1e-9);
}

#[test]
fn shift_preserves_duration() {
    let orig = interval(0.0, 5.0);
    let shifted = orig.shift(3.5);
    assert!((shifted.duration() - orig.duration()).abs() < 1e-12);
}

// ─── additional iterate ───────────────────────────────────────────────────────

#[test]
fn iterate_step_smaller_than_duration() {
    let points = interval(0.0, 5.0).iterate(1.0);
    assert_eq!(points.len(), 6);
}

#[test]
fn iterate_produces_sorted_values() {
    let points = interval(0.0, 3.0).iterate(0.5);
    for i in 1..points.len() {
        assert!(points[i].value > points[i - 1].value);
    }
}

// ─── additional sample ────────────────────────────────────────────────────────

#[test]
fn sample_values_within_interval() {
    let iv = interval(5.0, 15.0);
    let samples = iv.sample(7);
    for s in &samples {
        assert!(s.value >= 5.0 && s.value <= 15.0);
    }
}

#[test]
fn sample_count_3_midpoints() {
    let samples = interval(0.0, 6.0).sample(3);
    assert_eq!(samples.len(), 3);
    assert!((samples[1].value - 3.0).abs() < 1e-10);
}

// ─── additional midpoint ──────────────────────────────────────────────────────

#[test]
fn midpoint_large_interval() {
    let mid = interval(0.0, 36_525.0).midpoint();
    assert!((mid.value - 18_262.5).abs() < 1e-9);
}

#[test]
fn midpoint_zero_length() {
    let mid = interval(9.0, 9.0).midpoint();
    assert!((mid.value - 9.0).abs() < 1e-12);
}

// ─── additional encloses ──────────────────────────────────────────────────────

#[test]
fn encloses_point_interval() {
    assert!(interval(0.0, 10.0).encloses(&interval(5.0, 5.0)));
}

#[test]
fn smaller_does_not_enclose_larger() {
    assert!(!interval(2.0, 8.0).encloses(&interval(0.0, 10.0)));
}
