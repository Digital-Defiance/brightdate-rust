use brightdate::scheduling::*;

const EPS: f64 = 1e-12;

// ── Interval constants ────────────────────────────────────────────────────────

#[test]
fn interval_second_value() {
    assert!((INTERVAL_SECOND - 1.0 / 86_400.0).abs() < EPS);
}

#[test]
fn interval_minute_value() {
    assert!((INTERVAL_MINUTE - 1.0 / 1_440.0).abs() < EPS);
}

#[test]
fn interval_hour_value() {
    assert!((INTERVAL_HOUR - 1.0 / 24.0).abs() < EPS);
}

#[test]
fn interval_half_day_value() {
    assert!((INTERVAL_HALF_DAY - 0.5).abs() < EPS);
}

#[test]
fn interval_day_value() {
    assert!((INTERVAL_DAY - 1.0).abs() < EPS);
}

#[test]
fn interval_week_value() {
    assert!((INTERVAL_WEEK - 7.0).abs() < EPS);
}

#[test]
fn interval_fortnight_value() {
    assert!((INTERVAL_FORTNIGHT - 14.0).abs() < EPS);
}

#[test]
fn interval_ordering() {
    assert!(INTERVAL_SECOND < INTERVAL_MINUTE);
    assert!(INTERVAL_MINUTE < INTERVAL_HOUR);
    assert!(INTERVAL_HOUR < INTERVAL_HALF_DAY);
    assert!(INTERVAL_HALF_DAY < INTERVAL_DAY);
    assert!(INTERVAL_DAY < INTERVAL_WEEK);
    assert!(INTERVAL_WEEK < INTERVAL_FORTNIGHT);
}

#[test]
fn interval_five_minutes_value() {
    assert!((INTERVAL_FIVE_MINUTES - 5.0 / 1_440.0).abs() < EPS);
}

#[test]
fn interval_quarter_hour_value() {
    assert!((INTERVAL_QUARTER_HOUR - 15.0 / 1_440.0).abs() < EPS);
}

#[test]
fn interval_half_hour_value() {
    assert!((INTERVAL_HALF_HOUR - 30.0 / 1_440.0).abs() < EPS);
}

#[test]
fn interval_year_approx_value() {
    assert!((INTERVAL_YEAR_APPROX - 365.25).abs() < EPS);
}

// ── RecurrencePattern ─────────────────────────────────────────────────────────

#[test]
fn recurrence_pattern_new() {
    let p = RecurrencePattern::new(9622.0, 1.0);
    assert!((p.start - 9622.0).abs() < EPS);
    assert!((p.interval_days - 1.0).abs() < EPS);
    assert!(p.end.is_none());
    assert!(p.max_occurrences.is_none());
}

#[test]
fn recurrence_pattern_with_end() {
    let p = RecurrencePattern::new(0.0, 1.0).with_end(10.0);
    assert_eq!(p.end, Some(10.0));
}

#[test]
fn recurrence_pattern_with_max() {
    let p = RecurrencePattern::new(0.0, 1.0).with_max(5);
    assert_eq!(p.max_occurrences, Some(5));
}

#[test]
fn recurrence_pattern_chained_builders() {
    let p = RecurrencePattern::new(0.0, 7.0).with_end(100.0).with_max(10);
    assert_eq!(p.end, Some(100.0));
    assert_eq!(p.max_occurrences, Some(10));
}

// ── recurrences ───────────────────────────────────────────────────────────────

#[test]
fn recurrences_with_max_count() {
    let p = RecurrencePattern::new(0.0, 1.0).with_max(5);
    let r = recurrences(&p);
    assert_eq!(r.len(), 5);
}

#[test]
fn recurrences_starts_at_start() {
    let p = RecurrencePattern::new(9622.0, 1.0).with_max(3);
    let r = recurrences(&p);
    assert!((r[0] - 9622.0).abs() < EPS);
}

#[test]
fn recurrences_interval_daily() {
    let p = RecurrencePattern::new(0.0, 1.0).with_max(3);
    let r = recurrences(&p);
    assert!((r[1] - r[0] - 1.0).abs() < EPS);
    assert!((r[2] - r[1] - 1.0).abs() < EPS);
}

#[test]
fn recurrences_interval_weekly() {
    let p = RecurrencePattern::new(0.0, 7.0).with_max(4);
    let r = recurrences(&p);
    for w in r.windows(2) {
        assert!((w[1] - w[0] - 7.0).abs() < EPS);
    }
}

#[test]
fn recurrences_with_end_stops_at_end() {
    let p = RecurrencePattern::new(0.0, 1.0).with_end(5.0);
    let r = recurrences(&p);
    assert!(r.iter().all(|&v| v <= 5.0));
}

#[test]
fn recurrences_with_end_includes_start() {
    let p = RecurrencePattern::new(0.0, 1.0).with_end(3.0);
    let r = recurrences(&p);
    assert!(!r.is_empty());
    assert!((r[0] - 0.0).abs() < EPS);
}

#[test]
fn recurrences_with_end_correct_count() {
    let p = RecurrencePattern::new(0.0, 1.0).with_end(4.0);
    let r = recurrences(&p);
    // 0, 1, 2, 3, 4 = 5 items
    assert_eq!(r.len(), 5);
}

#[test]
fn recurrences_zero_interval_returns_empty() {
    let p = RecurrencePattern::new(0.0, 0.0).with_max(10);
    assert!(recurrences(&p).is_empty());
}

#[test]
fn recurrences_negative_interval_returns_empty() {
    let p = RecurrencePattern::new(0.0, -1.0).with_max(10);
    assert!(recurrences(&p).is_empty());
}

#[test]
fn recurrences_max_zero_returns_empty() {
    let p = RecurrencePattern::new(0.0, 1.0).with_max(0);
    assert!(recurrences(&p).is_empty());
}

#[test]
fn recurrences_hour_interval() {
    let p = RecurrencePattern::new(0.0, INTERVAL_HOUR).with_max(24);
    let r = recurrences(&p);
    assert_eq!(r.len(), 24);
    assert!((r.last().unwrap() - 23.0 * INTERVAL_HOUR).abs() < EPS);
}

#[test]
fn recurrences_no_bound_returns_one_item() {
    // Without end or max, recurrences returns just the start
    let p = RecurrencePattern::new(0.0, 1.0);
    let r = recurrences(&p);
    assert_eq!(r.len(), 1);
    assert!((r[0] - 0.0).abs() < EPS);
}

// ── next_occurrences ──────────────────────────────────────────────────────────

#[test]
fn next_occurrences_count_3() {
    let p = RecurrencePattern::new(0.0, 1.0);
    let r = next_occurrences(&p, 3);
    assert_eq!(r.len(), 3);
}

#[test]
fn next_occurrences_starts_at_start() {
    let p = RecurrencePattern::new(9622.0, INTERVAL_DAY);
    let r = next_occurrences(&p, 5);
    assert!((r[0] - 9622.0).abs() < EPS);
}

#[test]
fn next_occurrences_zero_count_empty() {
    let p = RecurrencePattern::new(0.0, 1.0);
    assert!(next_occurrences(&p, 0).is_empty());
}

#[test]
fn next_occurrences_interval_preserved() {
    let p = RecurrencePattern::new(0.0, 7.0);
    let r = next_occurrences(&p, 4);
    for w in r.windows(2) {
        assert!((w[1] - w[0] - 7.0).abs() < EPS);
    }
}

#[test]
fn next_occurrences_zero_interval_empty() {
    let p = RecurrencePattern::new(0.0, 0.0);
    assert!(next_occurrences(&p, 5).is_empty());
}

// ── next_occurrence_after ─────────────────────────────────────────────────────

#[test]
fn next_occurrence_after_returns_next_step() {
    let p = RecurrencePattern::new(0.0, 1.0);
    let next = next_occurrence_after(&p, 0.5).unwrap();
    assert!((next - 1.0).abs() < EPS);
}

#[test]
fn next_occurrence_after_before_start_returns_start() {
    let p = RecurrencePattern::new(10.0, 1.0);
    let next = next_occurrence_after(&p, 5.0).unwrap();
    assert!((next - 10.0).abs() < EPS);
}

#[test]
fn next_occurrence_after_on_occurrence() {
    // Asking for after 0.0 when 0.0 is an occurrence gives 1.0
    let p = RecurrencePattern::new(0.0, 1.0);
    let next = next_occurrence_after(&p, 0.0).unwrap();
    assert!((next - 1.0).abs() < EPS);
}

#[test]
fn next_occurrence_after_past_end_returns_none() {
    let p = RecurrencePattern::new(0.0, 1.0).with_end(5.0);
    assert!(next_occurrence_after(&p, 5.5).is_none());
}

#[test]
fn next_occurrence_after_zero_interval_returns_none() {
    let p = RecurrencePattern::new(0.0, 0.0);
    assert!(next_occurrence_after(&p, 0.0).is_none());
}

// ── previous_occurrence_before ────────────────────────────────────────────────

#[test]
fn previous_occurrence_before_returns_prior() {
    let p = RecurrencePattern::new(0.0, 1.0);
    let prev = previous_occurrence_before(&p, 2.5).unwrap();
    assert!((prev - 2.0).abs() < EPS);
}

#[test]
fn previous_occurrence_before_start_returns_none() {
    let p = RecurrencePattern::new(10.0, 1.0);
    assert!(previous_occurrence_before(&p, 9.0).is_none());
}

#[test]
fn previous_occurrence_before_on_occurrence_returns_prior() {
    let p = RecurrencePattern::new(0.0, 1.0);
    let prev = previous_occurrence_before(&p, 3.0).unwrap();
    assert!((prev - 2.0).abs() < EPS);
}

#[test]
fn previous_occurrence_before_zero_interval_returns_none() {
    let p = RecurrencePattern::new(0.0, 0.0);
    assert!(previous_occurrence_before(&p, 5.0).is_none());
}

// ── ScheduledEvent / BrightDateTimeline ───────────────────────────────────────

fn make_event(id: &str, at: f64) -> ScheduledEvent {
    ScheduledEvent {
        id: id.to_string(),
        name: format!("Event {id}"),
        time: at,
        duration: None,
    }
}

#[test]
fn timeline_new_is_empty() {
    let tl = BrightDateTimeline::new();
    assert!(tl.is_empty());
    assert_eq!(tl.len(), 0);
}

#[test]
fn timeline_add_event() {
    let mut tl = BrightDateTimeline::new();
    tl.add(make_event("e1", 100.0));
    assert_eq!(tl.len(), 1);
    assert!(!tl.is_empty());
}

#[test]
fn timeline_add_multiple() {
    let mut tl = BrightDateTimeline::new();
    tl.add(make_event("e1", 100.0));
    tl.add(make_event("e2", 200.0));
    tl.add(make_event("e3", 300.0));
    assert_eq!(tl.len(), 3);
}

#[test]
fn timeline_remove_existing() {
    let mut tl = BrightDateTimeline::new();
    tl.add(make_event("e1", 100.0));
    let removed = tl.remove("e1");
    assert!(removed);
    assert_eq!(tl.len(), 0);
}

#[test]
fn timeline_remove_nonexistent() {
    let mut tl = BrightDateTimeline::new();
    let removed = tl.remove("nonexistent");
    assert!(!removed);
}

#[test]
fn timeline_get_in_range() {
    let mut tl = BrightDateTimeline::new();
    tl.add(make_event("e1", 100.0));
    tl.add(make_event("e2", 200.0));
    tl.add(make_event("e3", 300.0));
    let events = tl.get_in_range(150.0, 250.0);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].id, "e2");
}

#[test]
fn timeline_get_in_range_inclusive() {
    let mut tl = BrightDateTimeline::new();
    tl.add(make_event("e1", 100.0));
    tl.add(make_event("e2", 200.0));
    let events = tl.get_in_range(100.0, 200.0);
    assert_eq!(events.len(), 2);
}

#[test]
fn timeline_get_next() {
    let mut tl = BrightDateTimeline::new();
    tl.add(make_event("e1", 100.0));
    tl.add(make_event("e2", 200.0));
    let next = tl.get_next(150.0).unwrap();
    assert_eq!(next.id, "e2");
}

#[test]
fn timeline_get_next_none_when_past_all() {
    let mut tl = BrightDateTimeline::new();
    tl.add(make_event("e1", 100.0));
    assert!(tl.get_next(200.0).is_none());
}

#[test]
fn timeline_get_previous() {
    let mut tl = BrightDateTimeline::new();
    tl.add(make_event("e1", 100.0));
    tl.add(make_event("e2", 200.0));
    let prev = tl.get_previous(150.0).unwrap();
    assert_eq!(prev.id, "e1");
}

#[test]
fn timeline_get_previous_none_when_before_all() {
    let mut tl = BrightDateTimeline::new();
    tl.add(make_event("e1", 100.0));
    assert!(tl.get_previous(50.0).is_none());
}

#[test]
fn timeline_iter_all() {
    let mut tl = BrightDateTimeline::new();
    tl.add(make_event("e1", 100.0));
    tl.add(make_event("e2", 200.0));
    let count = tl.iter().count();
    assert_eq!(count, 2);
}

#[test]
fn timeline_get_in_range_empty() {
    let tl = BrightDateTimeline::new();
    let events = tl.get_in_range(0.0, 1000.0);
    assert!(events.is_empty());
}

#[test]
fn timeline_remove_reduces_count() {
    let mut tl = BrightDateTimeline::new();
    tl.add(make_event("e1", 100.0));
    tl.add(make_event("e2", 200.0));
    tl.remove("e1");
    assert_eq!(tl.len(), 1);
}
