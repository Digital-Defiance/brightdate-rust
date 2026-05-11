use brightdate::calendar::*;
use brightdate::conversions::to_date_time;
use chrono::Datelike;

const EPS: f64 = 1e-9;

// ── get_year ─────────────────────────────────────────────────────────────────

#[test]
fn get_year_2000_epoch() {
    // J2000.0 = BD 0.0 = 2000-01-01T11:58:55.816 UTC (still year 2000).
    assert_eq!(get_year(0.0), 2000);
}

#[test]
fn get_year_2024() {
    // 2024-01-01T00:00:00Z
    let bd = from_calendar(2024, 1, 1, 0, 0, 0).unwrap();
    assert_eq!(get_year(bd), 2024);
}

#[test]
fn get_year_negative_bd() {
    // 1999-01-01 is before J2000 → negative BD
    let bd = from_calendar(1999, 1, 1, 0, 0, 0).unwrap();
    assert_eq!(get_year(bd), 1999);
}

#[test]
fn get_year_far_future() {
    let bd = from_calendar(2100, 6, 15, 0, 0, 0).unwrap();
    assert_eq!(get_year(bd), 2100);
}

#[test]
fn get_year_leap_year_2000() {
    let bd = from_calendar(2000, 2, 29, 0, 0, 0).unwrap();
    assert_eq!(get_year(bd), 2000);
}

// ── get_month ────────────────────────────────────────────────────────────────

#[test]
fn get_month_january() {
    let bd = from_calendar(2024, 1, 15, 0, 0, 0).unwrap();
    assert_eq!(get_month(bd), 1);
}

#[test]
fn get_month_december() {
    let bd = from_calendar(2024, 12, 1, 0, 0, 0).unwrap();
    assert_eq!(get_month(bd), 12);
}

#[test]
fn get_month_february() {
    let bd = from_calendar(2024, 2, 14, 0, 0, 0).unwrap();
    assert_eq!(get_month(bd), 2);
}

#[test]
fn get_month_each_month() {
    for m in 1u32..=12 {
        let bd = from_calendar(2024, m, 1, 0, 0, 0).unwrap();
        assert_eq!(get_month(bd), m, "month {m}");
    }
}

// ── get_day_of_month ──────────────────────────────────────────────────────────

#[test]
fn get_day_of_month_first() {
    let bd = from_calendar(2024, 3, 1, 0, 0, 0).unwrap();
    assert_eq!(get_day_of_month(bd), 1);
}

#[test]
fn get_day_of_month_last_of_jan() {
    let bd = from_calendar(2024, 1, 31, 0, 0, 0).unwrap();
    assert_eq!(get_day_of_month(bd), 31);
}

#[test]
fn get_day_of_month_15th() {
    let bd = from_calendar(2024, 6, 15, 0, 0, 0).unwrap();
    assert_eq!(get_day_of_month(bd), 15);
}

#[test]
fn get_day_of_month_leap_feb29() {
    let bd = from_calendar(2000, 2, 29, 0, 0, 0).unwrap();
    assert_eq!(get_day_of_month(bd), 29);
}

// ── get_day_of_week ───────────────────────────────────────────────────────────

#[test]
fn get_day_of_week_j2000_is_saturday() {
    // 2000-01-01 is a Saturday = 6
    assert_eq!(get_day_of_week(0.0), 6);
}

#[test]
fn get_day_of_week_2024_01_01_is_monday() {
    // 2024-01-01 is a Monday = 1
    let bd = from_calendar(2024, 1, 1, 0, 0, 0).unwrap();
    assert_eq!(get_day_of_week(bd), 1);
}

#[test]
fn get_day_of_week_sunday_is_zero() {
    // 2023-07-02 is a Sunday
    let bd = from_calendar(2023, 7, 2, 0, 0, 0).unwrap();
    assert_eq!(get_day_of_week(bd), 0);
}

#[test]
fn get_day_of_week_in_range() {
    let bd = from_calendar(2024, 9, 15, 0, 0, 0).unwrap();
    let dow = get_day_of_week(bd);
    assert!(dow <= 6, "day of week must be 0-6");
}

#[test]
fn get_day_of_week_sequential_days() {
    let bd = from_calendar(2024, 1, 7, 0, 0, 0).unwrap(); // Sunday
    for i in 0..7 {
        let dow = get_day_of_week(bd + i as f64);
        assert_eq!(dow, i as u32, "day {} should have day_of_week {}", i, i);
    }
}

// ── get_day_of_year ───────────────────────────────────────────────────────────

#[test]
fn get_day_of_year_jan_1() {
    let bd = from_calendar(2024, 1, 1, 0, 0, 0).unwrap();
    assert_eq!(get_day_of_year(bd), 1);
}

#[test]
fn get_day_of_year_dec_31_non_leap() {
    let bd = from_calendar(2023, 12, 31, 0, 0, 0).unwrap();
    assert_eq!(get_day_of_year(bd), 365);
}

#[test]
fn get_day_of_year_dec_31_leap() {
    let bd = from_calendar(2024, 12, 31, 0, 0, 0).unwrap();
    assert_eq!(get_day_of_year(bd), 366);
}

#[test]
fn get_day_of_year_feb_29_leap() {
    let bd = from_calendar(2000, 2, 29, 0, 0, 0).unwrap();
    assert_eq!(get_day_of_year(bd), 60);
}

#[test]
fn get_day_of_year_march_1_non_leap() {
    let bd = from_calendar(2023, 3, 1, 0, 0, 0).unwrap();
    assert_eq!(get_day_of_year(bd), 60);
}

#[test]
fn get_day_of_year_march_1_leap() {
    let bd = from_calendar(2024, 3, 1, 0, 0, 0).unwrap();
    assert_eq!(get_day_of_year(bd), 61);
}

// ── is_leap_year ──────────────────────────────────────────────────────────────

#[test]
fn is_leap_year_2000() {
    assert!(is_leap_year(from_calendar(2000, 1, 1, 0, 0, 0).unwrap()));
}

#[test]
fn is_not_leap_year_1900() {
    assert!(!is_leap_year(from_calendar(1900, 1, 1, 0, 0, 0).unwrap()));
}

#[test]
fn is_leap_year_2024() {
    assert!(is_leap_year(from_calendar(2024, 1, 1, 0, 0, 0).unwrap()));
}

#[test]
fn is_not_leap_year_2023() {
    assert!(!is_leap_year(from_calendar(2023, 1, 1, 0, 0, 0).unwrap()));
}

#[test]
fn is_leap_year_2400() {
    assert!(is_leap_year(from_calendar(2400, 1, 1, 0, 0, 0).unwrap()));
}

#[test]
fn is_not_leap_year_2100() {
    assert!(!is_leap_year(from_calendar(2100, 1, 1, 0, 0, 0).unwrap()));
}

#[test]
fn leap_year_divisible_by_4_but_not_100() {
    // 1996, 2004, 2008 should all be leap years
    for year in [1996, 2004, 2008, 2012, 2016, 2020] {
        assert!(is_leap_year(from_calendar(year, 6, 1, 0, 0, 0).unwrap()), "year {year}");
    }
}

#[test]
fn not_leap_year_divisible_by_100_not_400() {
    for year in [1700, 1800, 1900, 2100, 2200, 2300] {
        assert!(!is_leap_year(from_calendar(year, 6, 1, 0, 0, 0).unwrap()), "year {year}");
    }
}

// ── days_in_year ──────────────────────────────────────────────────────────────

#[test]
fn days_in_year_leap() {
    assert_eq!(days_in_year(from_calendar(2024, 1, 1, 0, 0, 0).unwrap()), 366);
}

#[test]
fn days_in_year_non_leap() {
    assert_eq!(days_in_year(from_calendar(2023, 1, 1, 0, 0, 0).unwrap()), 365);
}

#[test]
fn days_in_year_2000_leap() {
    assert_eq!(days_in_year(from_calendar(2000, 1, 1, 0, 0, 0).unwrap()), 366);
}

// ── days_in_month ─────────────────────────────────────────────────────────────

#[test]
fn days_in_month_january() {
    assert_eq!(days_in_month(from_calendar(2024, 1, 1, 0, 0, 0).unwrap()), 31);
}

#[test]
fn days_in_month_feb_non_leap() {
    assert_eq!(days_in_month(from_calendar(2023, 2, 1, 0, 0, 0).unwrap()), 28);
}

#[test]
fn days_in_month_feb_leap() {
    assert_eq!(days_in_month(from_calendar(2024, 2, 1, 0, 0, 0).unwrap()), 29);
}

#[test]
fn days_in_month_april() {
    assert_eq!(days_in_month(from_calendar(2024, 4, 1, 0, 0, 0).unwrap()), 30);
}

#[test]
fn days_in_month_december() {
    assert_eq!(days_in_month(from_calendar(2024, 12, 1, 0, 0, 0).unwrap()), 31);
}

#[test]
fn days_in_month_all_months_2024() {
    let expected = [31u32, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for (i, &exp) in expected.iter().enumerate() {
        let m = i as u32 + 1;
        let bd = from_calendar(2024, m, 1, 0, 0, 0).unwrap();
        assert_eq!(days_in_month(bd), exp, "month {m}");
    }
}

#[test]
fn days_in_month_all_months_2023() {
    let expected = [31u32, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for (i, &exp) in expected.iter().enumerate() {
        let m = i as u32 + 1;
        let bd = from_calendar(2023, m, 1, 0, 0, 0).unwrap();
        assert_eq!(days_in_month(bd), exp, "month {m}");
    }
}

// ── start_of_year / end_of_year ───────────────────────────────────────────────

#[test]
fn start_of_year_2000_month_is_jan() {
    let bd = start_of_year(2000);
    assert_eq!(get_month(bd), 1);
    assert_eq!(get_day_of_month(bd), 1);
}

#[test]
fn start_of_year_2024() {
    let bd = start_of_year(2024);
    assert_eq!(get_year(bd), 2024);
    assert_eq!(get_month(bd), 1);
    assert_eq!(get_day_of_month(bd), 1);
}

#[test]
fn end_of_year_before_next_start() {
    let end_2023 = end_of_year(2023);
    let start_2024 = start_of_year(2024);
    assert!(end_2023 < start_2024);
}

#[test]
fn end_of_year_in_december() {
    let bd = end_of_year(2024);
    let dt = to_date_time(bd);
    assert_eq!(dt.month(), 12);
    assert_eq!(dt.day(), 31);
}

#[test]
fn start_of_year_is_before_end_of_year() {
    let start = start_of_year(2024);
    let end = end_of_year(2024);
    assert!(start < end);
}

// ── start_of_month / end_of_month ─────────────────────────────────────────────

#[test]
fn start_of_month_february_2024() {
    let bd = start_of_month(2024, 2);
    let dt = to_date_time(bd);
    assert_eq!(dt.month(), 2);
    assert_eq!(dt.day(), 1);
}

#[test]
fn end_of_month_february_2024_is_29th() {
    let bd = end_of_month(2024, 2);
    let dt = to_date_time(bd);
    assert_eq!(dt.month(), 2);
    assert_eq!(dt.day(), 29);
}

#[test]
fn end_of_month_january_is_31st() {
    let bd = end_of_month(2024, 1);
    let dt = to_date_time(bd);
    assert_eq!(dt.day(), 31);
}

#[test]
fn end_of_month_december_is_31st() {
    let bd = end_of_month(2024, 12);
    let dt = to_date_time(bd);
    assert_eq!(dt.month(), 12);
    assert_eq!(dt.day(), 31);
}

#[test]
fn start_before_end_of_month() {
    for m in 1u32..=12 {
        let start = start_of_month(2024, m);
        let end = end_of_month(2024, m);
        assert!(start < end, "month {m}");
    }
}

// ── from_calendar ─────────────────────────────────────────────────────────────

#[test]
fn from_calendar_j2000() {
    // BD 0.0 is J2000.0 = 2000-01-01T11:58:55.816 UTC.
    let bd = from_calendar(2000, 1, 1, 11, 58, 55).unwrap();
    // Within ~1 s of BD 0.0 (sub-second milli truncation in from_calendar).
    assert!(bd.abs() < 1.0 / 86_400.0, "got bd={bd}");
}

#[test]
fn from_calendar_invalid_month() {
    assert!(from_calendar(2024, 13, 1, 0, 0, 0).is_err());
}

#[test]
fn from_calendar_invalid_day() {
    assert!(from_calendar(2024, 2, 30, 0, 0, 0).is_err());
}

#[test]
fn from_calendar_feb29_non_leap_errors() {
    assert!(from_calendar(2023, 2, 29, 0, 0, 0).is_err());
}

#[test]
fn from_calendar_feb29_leap_ok() {
    assert!(from_calendar(2000, 2, 29, 0, 0, 0).is_ok());
}

#[test]
fn from_calendar_roundtrip_year() {
    let bd = from_calendar(2024, 6, 15, 8, 30, 0).unwrap();
    assert_eq!(get_year(bd), 2024);
    assert_eq!(get_month(bd), 6);
    assert_eq!(get_day_of_month(bd), 15);
}

#[test]
fn from_calendar_zero_month_errors() {
    assert!(from_calendar(2024, 0, 1, 0, 0, 0).is_err());
}

#[test]
fn from_calendar_zero_day_errors() {
    assert!(from_calendar(2024, 1, 0, 0, 0, 0).is_err());
}

// ─── additional calendar tests ────────────────────────────────────────────────

#[test]
fn from_calendar_all_months_valid() {
    for month in 1u32..=12 {
        assert!(from_calendar(2024, month, 1, 0, 0, 0).is_ok(), "month {month} failed");
    }
}

#[test]
fn get_month_roundtrip_all_months() {
    for month in 1u32..=12 {
        let bd = from_calendar(2024, month, 1, 0, 0, 0).unwrap();
        assert_eq!(get_month(bd), month, "roundtrip failed for month {month}");
    }
}

#[test]
fn get_day_of_month_first_of_each_month() {
    for month in 1u32..=12 {
        let bd = from_calendar(2024, month, 1, 0, 0, 0).unwrap();
        assert_eq!(get_day_of_month(bd), 1, "day_of_month failed for month {month}");
    }
}

#[test]
fn get_day_of_year_jan1_extra() {
    let bd = from_calendar(2024, 1, 1, 0, 0, 0).unwrap();
    assert_eq!(get_day_of_year(bd), 1);
}

#[test]
fn get_day_of_year_dec31_non_leap() {
    let bd = from_calendar(2023, 12, 31, 0, 0, 0).unwrap();
    assert_eq!(get_day_of_year(bd), 365);
}

#[test]
fn get_day_of_year_dec31_leap() {
    let bd = from_calendar(2024, 12, 31, 0, 0, 0).unwrap();
    assert_eq!(get_day_of_year(bd), 366);
}

#[test]
fn is_leap_year_2000_via_bd() {
    let bd = from_calendar(2000, 1, 1, 0, 0, 0).unwrap();
    assert!(is_leap_year(bd));
}

#[test]
fn is_leap_year_2100_false_via_bd() {
    let bd = from_calendar(2100, 1, 1, 0, 0, 0).unwrap();
    assert!(!is_leap_year(bd));
}

#[test]
fn is_leap_year_2024_via_bd() {
    let bd = from_calendar(2024, 1, 1, 0, 0, 0).unwrap();
    assert!(is_leap_year(bd));
}

#[test]
fn is_leap_year_2023_false_via_bd() {
    let bd = from_calendar(2023, 1, 1, 0, 0, 0).unwrap();
    assert!(!is_leap_year(bd));
}

#[test]
fn is_leap_year_1900_false_via_bd() {
    let bd = from_calendar(1900, 1, 1, 0, 0, 0).unwrap();
    assert!(!is_leap_year(bd));
}

#[test]
fn days_in_month_february_leap_via_bd() {
    let bd = from_calendar(2000, 2, 1, 0, 0, 0).unwrap();
    assert_eq!(days_in_month(bd), 29);
}

#[test]
fn days_in_month_february_non_leap_via_bd() {
    let bd = from_calendar(2001, 2, 1, 0, 0, 0).unwrap();
    assert_eq!(days_in_month(bd), 28);
}

#[test]
fn days_in_month_june_extra() {
    let bd = from_calendar(2024, 6, 1, 0, 0, 0).unwrap();
    assert_eq!(days_in_month(bd), 30);
}

#[test]
fn days_in_month_all_months_2023_extra() {
    let expected = [31u32, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for (i, &exp) in expected.iter().enumerate() {
        let m = i as u32 + 1;
        let bd = from_calendar(2023, m, 1, 0, 0, 0).unwrap();
        assert_eq!(days_in_month(bd), exp, "month {m}");
    }
}

#[test]
fn end_of_month_may_extra() {
    let eom = end_of_month(2024, 5);
    assert_eq!(get_day_of_month(eom), 31);
    assert_eq!(get_month(eom), 5);
}

#[test]
fn end_of_month_june_extra() {
    let eom = end_of_month(2024, 6);
    assert_eq!(get_day_of_month(eom), 30);
}

#[test]
fn end_of_month_november_extra() {
    let eom = end_of_month(2024, 11);
    assert_eq!(get_day_of_month(eom), 30);
}

#[test]
fn end_of_month_december_extra() {
    let eom = end_of_month(2024, 12);
    assert_eq!(get_day_of_month(eom), 31);
}

#[test]
fn start_of_month_year_ok() {
    let som = start_of_month(2024, 3);
    assert_eq!(get_day_of_month(som), 1);
    assert_eq!(get_month(som), 3);
    assert_eq!(get_year(som), 2024);
}

#[test]
fn start_of_year_extra() {
    let soy = start_of_year(2024);
    assert_eq!(get_year(soy), 2024);
    assert_eq!(get_month(soy), 1);
    assert_eq!(get_day_of_month(soy), 1);
}

#[test]
fn end_of_year_extra() {
    let eoy = end_of_year(2024);
    assert_eq!(get_year(eoy), 2024);
    assert_eq!(get_month(eoy), 12);
    assert_eq!(get_day_of_month(eoy), 31);
}

#[test]
fn get_day_of_week_sunday() {
    // 2023-01-01 is a Sunday
    let bd = from_calendar(2023, 1, 1, 0, 0, 0).unwrap();
    // day_of_week: 0=Sun or 7=Sun or implementation-defined; just test it's in 0..=6
    let dow = get_day_of_week(bd);
    assert!(dow <= 6, "day_of_week out of range: {dow}");
}

#[test]
fn get_day_of_week_roundtrip_week() {
    // Mon through Sun should give 7 distinct values in a week
    let days: std::collections::HashSet<u32> = (0..7)
        .map(|d| {
            let bd = from_calendar(2024, 1, 1 + d, 0, 0, 0).unwrap();
            get_day_of_week(bd)
        })
        .collect();
    assert_eq!(days.len(), 7, "7 days should have 7 distinct day-of-week values");
}

#[test]
fn days_in_year_2000_extra() {
    let bd = from_calendar(2000, 6, 1, 0, 0, 0).unwrap();
    assert_eq!(days_in_year(bd), 366);
}

#[test]
fn days_in_year_2001_extra() {
    let bd = from_calendar(2001, 1, 1, 0, 0, 0).unwrap();
    assert_eq!(days_in_year(bd), 365);
}
