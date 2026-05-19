//! Tests for the BrightDate Spacetime Standard module.

use brightdate::constants::SECONDS_PER_DAY;
use brightdate::spacetime::*;

const EPS: f64 = 1e-9;

// ── Fundamental constants ────────────────────────────────────────────────────

#[test]
fn speed_of_light_exact_si_2019() {
    assert_eq!(SPEED_OF_LIGHT_M_PER_S, 299_792_458.0);
}

#[test]
fn bright_meter_equals_speed_of_light_seconds() {
    assert_eq!(BRIGHT_METER_M, SPEED_OF_LIGHT_M_PER_S);
}

#[test]
fn light_day_equals_c_times_86400_exactly() {
    assert_eq!(LIGHT_DAY_M, 25_902_068_371_200.0);
    assert_eq!(LIGHT_DAY_M, SPEED_OF_LIGHT_M_PER_S * SECONDS_PER_DAY);
}

// ── Unit hierarchies ─────────────────────────────────────────────────────────

#[test]
fn bright_meter_units_consistent_metres_equals_seconds_times_c() {
    for u in BRIGHT_METER_UNITS {
        assert!(
            (u.metres - u.seconds * SPEED_OF_LIGHT_M_PER_S).abs() < 1e-3,
            "unit {} (metres = {}, seconds × c = {}) inconsistent",
            u.symbol,
            u.metres,
            u.seconds * SPEED_OF_LIGHT_M_PER_S,
        );
    }
}

#[test]
fn light_day_units_metres_consistent() {
    for u in LIGHT_DAY_UNITS {
        let derived = u.seconds * SPEED_OF_LIGHT_M_PER_S;
        assert!(
            (u.metres - derived).abs() < derived.abs() * 1e-12 + 1e-3,
            "unit {} (metres = {}, derived = {}) inconsistent",
            u.symbol,
            u.metres,
            derived,
        );
    }
}

#[test]
fn bright_meter_units_include_canonical_symbols() {
    let symbols: Vec<&str> = BRIGHT_METER_UNITS.iter().map(|u| u.symbol).collect();
    assert!(symbols.contains(&"μbm"));
    assert!(symbols.contains(&"mbm"));
    assert!(symbols.contains(&"bm"));
    assert!(symbols.contains(&"Mbm"));
    assert!(symbols.contains(&"Gbm"));
}

#[test]
fn light_day_units_include_canonical_symbols() {
    let symbols: Vec<&str> = LIGHT_DAY_UNITS.iter().map(|u| u.symbol).collect();
    assert!(symbols.contains(&"Lμd"));
    assert!(symbols.contains(&"Lmd"));
    assert!(symbols.contains(&"Ld"));
    assert!(symbols.contains(&"Lkd"));
}

// ── Conversions: seconds ↔ metres ────────────────────────────────────────────

#[test]
fn seconds_metres_round_trip_one_second() {
    assert_eq!(seconds_to_metres(1.0), SPEED_OF_LIGHT_M_PER_S);
    assert_eq!(metres_to_seconds(SPEED_OF_LIGHT_M_PER_S), 1.0);
}

#[test]
fn seconds_metres_handle_zero_and_negatives() {
    assert_eq!(seconds_to_metres(0.0), 0.0);
    assert_eq!(metres_to_seconds(0.0), 0.0);
    assert_eq!(seconds_to_metres(-1.0), -SPEED_OF_LIGHT_M_PER_S);
    assert_eq!(metres_to_seconds(-SPEED_OF_LIGHT_M_PER_S), -1.0);
}

// ── BrightMeters ↔ everything ───────────────────────────────────────────────

#[test]
fn bright_meters_seconds_are_identity() {
    assert_eq!(seconds_to_bright_meters(42.0), 42.0);
    assert_eq!(bright_meters_to_seconds(42.0), 42.0);
}

#[test]
fn metres_bright_meters_round_trip() {
    let m = 1_234_567_890.0;
    assert!((bright_meters_to_metres(metres_to_bright_meters(m)) - m).abs() < EPS);
}

#[test]
fn days_to_metres_one_day_equals_light_day() {
    assert_eq!(days_to_metres(1.0), LIGHT_DAY_M);
    assert_eq!(metres_to_days(LIGHT_DAY_M), 1.0);
}

#[test]
fn days_bright_seconds_round_trip() {
    let d = 0.5;
    assert!((bright_seconds_to_days(days_to_bright_seconds(d)) - d).abs() < EPS);
}

#[test]
fn one_au_in_bright_seconds_about_499() {
    let au_m = 149_597_870_700.0;
    let bs = metres_to_bright_meters(au_m);
    assert!(bs > 498.9 && bs < 499.1, "1 AU = {} bs (expected ≈ 499)", bs);
}
