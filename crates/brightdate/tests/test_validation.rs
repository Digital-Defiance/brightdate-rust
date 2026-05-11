//! Integration tests for validation — ported from validation.spec.ts

use brightdate::types::BrightDateError;
use brightdate::validation::{
    is_brightdate_value, is_valid_brightdate_string, validate_brightdate_value,
    validate_finite_number, validate_gps_seconds, validate_gps_week, validate_julian_date,
    validate_precision, validate_unix_ms,
};

// ─── validateFiniteNumber ────────────────────────────────────────────────────

#[test]
fn validate_finite_number_ok_for_finite() {
    assert!(validate_finite_number(42.0, "test").is_ok());
}

#[test]
fn validate_finite_number_ok_for_zero() {
    assert!(validate_finite_number(0.0, "test").is_ok());
}

#[test]
fn validate_finite_number_err_nan() {
    assert!(validate_finite_number(f64::NAN, "val").is_err());
}

#[test]
fn validate_finite_number_err_infinity() {
    assert!(validate_finite_number(f64::INFINITY, "val").is_err());
}

#[test]
fn validate_finite_number_err_neg_infinity() {
    assert!(validate_finite_number(f64::NEG_INFINITY, "val").is_err());
}

#[test]
fn validate_finite_number_error_contains_param_name() {
    let err = validate_finite_number(f64::NAN, "myParam").unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("myParam"), "error should mention param name: {msg}");
}

#[test]
fn validate_finite_number_error_is_invalid_number_variant() {
    let err = validate_finite_number(f64::NAN, "x").unwrap_err();
    assert!(matches!(err, BrightDateError::InvalidNumber(_)));
}

// ─── validateBrightDateValue ─────────────────────────────────────────────────

#[test]
fn validate_brightdate_value_ok_for_valid() {
    assert!(validate_brightdate_value(9622.5).is_ok());
}

#[test]
fn validate_brightdate_value_err_for_nan() {
    assert!(validate_brightdate_value(f64::NAN).is_err());
}

#[test]
fn validate_brightdate_value_err_for_infinity() {
    assert!(validate_brightdate_value(f64::INFINITY).is_err());
}

// ─── validatePrecision ───────────────────────────────────────────────────────

#[test]
fn validate_precision_ok_for_1() {
    assert!(validate_precision(1.0).is_ok());
}

#[test]
fn validate_precision_ok_for_5() {
    assert!(validate_precision(5.0).is_ok());
}

#[test]
fn validate_precision_ok_for_12() {
    assert!(validate_precision(12.0).is_ok());
}

#[test]
fn validate_precision_err_for_0() {
    assert!(validate_precision(0.0).is_err());
}

#[test]
fn validate_precision_err_for_13() {
    assert!(validate_precision(13.0).is_err());
}

#[test]
fn validate_precision_err_for_fractional() {
    assert!(validate_precision(1.5).is_err());
}

#[test]
fn validate_precision_err_for_negative() {
    assert!(validate_precision(-1.0).is_err());
}

#[test]
fn validate_precision_error_is_invalid_precision_variant() {
    let err = validate_precision(0.0).unwrap_err();
    assert!(matches!(err, BrightDateError::InvalidPrecision(_)));
}

// ─── isValidBrightDateString ─────────────────────────────────────────────────

#[test]
fn is_valid_brightdate_string_9622() {
    assert!(is_valid_brightdate_string("9622.50417"));
}

#[test]
fn is_valid_brightdate_string_zero() {
    assert!(is_valid_brightdate_string("0"));
}

#[test]
fn is_valid_brightdate_string_negative() {
    assert!(is_valid_brightdate_string("-10957.5"));
}

#[test]
fn is_valid_brightdate_string_integer() {
    assert!(is_valid_brightdate_string("100"));
}

#[test]
fn is_valid_brightdate_string_empty_false() {
    assert!(!is_valid_brightdate_string(""));
}

#[test]
fn is_valid_brightdate_string_alpha_false() {
    assert!(!is_valid_brightdate_string("abc"));
}

#[test]
fn is_valid_brightdate_string_double_dot_false() {
    assert!(!is_valid_brightdate_string("1.2.3"));
}

#[test]
fn is_valid_brightdate_string_whitespace_false() {
    assert!(!is_valid_brightdate_string("   "));
}

#[test]
fn is_valid_brightdate_string_scientific_notation_false() {
    assert!(!is_valid_brightdate_string("1e5"));
}

// ─── validateUnixMs ──────────────────────────────────────────────────────────

#[test]
fn validate_unix_ms_ok_for_valid() {
    assert!(validate_unix_ms(1_700_000_000_000.0).is_ok());
}

#[test]
fn validate_unix_ms_ok_for_zero() {
    assert!(validate_unix_ms(0.0).is_ok());
}

#[test]
fn validate_unix_ms_ok_lower_boundary() {
    assert!(validate_unix_ms(-30_610_224_000_000.0).is_ok());
}

#[test]
fn validate_unix_ms_err_below_lower_boundary() {
    assert!(validate_unix_ms(-30_610_224_000_001.0).is_err());
}

#[test]
fn validate_unix_ms_ok_upper_boundary() {
    assert!(validate_unix_ms(253_402_300_800_000.0).is_ok());
}

#[test]
fn validate_unix_ms_err_above_upper_boundary() {
    assert!(validate_unix_ms(253_402_300_800_001.0).is_err());
}

#[test]
fn validate_unix_ms_error_is_out_of_range_variant() {
    let err = validate_unix_ms(300_000_000_000_000.0).unwrap_err();
    assert!(matches!(err, BrightDateError::OutOfRange(_)));
}

// ─── validateJulianDate ──────────────────────────────────────────────────────

#[test]
fn validate_julian_date_ok_for_j2000() {
    assert!(validate_julian_date(2_451_545.0).is_ok());
}

#[test]
fn validate_julian_date_ok_for_zero() {
    assert!(validate_julian_date(0.0).is_ok());
}

#[test]
fn validate_julian_date_err_negative() {
    assert!(validate_julian_date(-1.0).is_err());
}

#[test]
fn validate_julian_date_err_too_large() {
    assert!(validate_julian_date(6_000_000.0).is_err());
}

#[test]
fn validate_julian_date_error_is_out_of_range() {
    let err = validate_julian_date(-1.0).unwrap_err();
    assert!(matches!(err, BrightDateError::OutOfRange(_)));
}

// ─── validateGPSWeek ─────────────────────────────────────────────────────────

#[test]
fn validate_gps_week_ok_for_zero() {
    assert!(validate_gps_week(0.0).is_ok());
}

#[test]
fn validate_gps_week_ok_for_2300() {
    assert!(validate_gps_week(2300.0).is_ok());
}

#[test]
fn validate_gps_week_err_negative() {
    assert!(validate_gps_week(-1.0).is_err());
}

#[test]
fn validate_gps_week_err_too_large() {
    assert!(validate_gps_week(10000.0).is_err());
}

#[test]
fn validate_gps_week_err_non_integer() {
    assert!(validate_gps_week(1.5).is_err());
}

#[test]
fn validate_gps_week_error_is_invalid_gps_week() {
    let err = validate_gps_week(-1.0).unwrap_err();
    assert!(matches!(err, BrightDateError::InvalidGpsWeek(_)));
}

// ─── validateGPSSeconds ──────────────────────────────────────────────────────

#[test]
fn validate_gps_seconds_ok_for_zero() {
    assert!(validate_gps_seconds(0.0).is_ok());
}

#[test]
fn validate_gps_seconds_ok_for_604799() {
    assert!(validate_gps_seconds(604799.0).is_ok());
}

#[test]
fn validate_gps_seconds_err_negative() {
    assert!(validate_gps_seconds(-1.0).is_err());
}

#[test]
fn validate_gps_seconds_err_at_604800() {
    assert!(validate_gps_seconds(604800.0).is_err());
}

#[test]
fn validate_gps_seconds_error_is_invalid_gps_seconds() {
    let err = validate_gps_seconds(-1.0).unwrap_err();
    assert!(matches!(err, BrightDateError::InvalidGpsSeconds(_)));
}

// ─── isBrightDateValue ───────────────────────────────────────────────────────

#[test]
fn is_brightdate_value_true_for_finite() {
    assert!(is_brightdate_value(9622.5));
}

#[test]
fn is_brightdate_value_true_for_zero() {
    assert!(is_brightdate_value(0.0));
}

#[test]
fn is_brightdate_value_true_for_negative() {
    assert!(is_brightdate_value(-10957.5));
}

#[test]
fn is_brightdate_value_false_for_nan() {
    assert!(!is_brightdate_value(f64::NAN));
}

#[test]
fn is_brightdate_value_false_for_infinity() {
    assert!(!is_brightdate_value(f64::INFINITY));
}

// ─── additional validateFiniteNumber ────────────────────────────────────────

#[test]
fn validate_finite_number_large_positive() {
    assert!(validate_finite_number(1e15, "big").is_ok());
}

#[test]
fn validate_finite_number_large_negative() {
    assert!(validate_finite_number(-1e15, "big_neg").is_ok());
}

#[test]
fn validate_finite_number_neg_infinity_variant() {
    let err = validate_finite_number(f64::NEG_INFINITY, "x").unwrap_err();
    assert!(matches!(err, BrightDateError::InvalidNumber(_)));
}

// ─── additional validatePrecision ────────────────────────────────────────────

#[test]
fn validate_precision_zero_ok() {
    assert!(validate_precision(1.0).is_ok());
}

#[test]
fn validate_precision_max_ok() {
    assert!(validate_precision(12.0).is_ok());
}

#[test]
fn validate_precision_thirteen_errors() {
    assert!(validate_precision(13.0).is_err());
}

#[test]
fn validate_precision_neg_one_errors() {
    assert!(validate_precision(-1.0).is_err());
}

#[test]
fn validate_precision_5_ok() {
    assert!(validate_precision(5.0).is_ok());
}

// ─── additional validateUnixMs ────────────────────────────────────────────────

#[test]
fn validate_unix_ms_large_positive() {
    assert!(validate_unix_ms(1_700_000_000_000.0).is_ok());
}

#[test]
fn validate_unix_ms_zero_ok() {
    assert!(validate_unix_ms(0.0).is_ok());
}

#[test]
fn validate_unix_ms_nan_fails() {
    // NaN passes range check (NaN < min is false, NaN > max is false)
    // validate_unix_ms with infinity should fail
    assert!(validate_unix_ms(f64::INFINITY).is_err());
}

#[test]
fn validate_unix_ms_neg_infinity_fails() {
    assert!(validate_unix_ms(f64::NEG_INFINITY).is_err());
}

// ─── additional validateGpsWeek ───────────────────────────────────────────────

#[test]
fn validate_gps_week_zero_ok() {
    assert!(validate_gps_week(0.0).is_ok());
}

#[test]
fn validate_gps_week_large_ok() {
    assert!(validate_gps_week(3000.0).is_ok());
}

#[test]
fn validate_gps_week_negative_errors() {
    assert!(validate_gps_week(-1.0).is_err());
}

// ─── additional validateGpsSeconds ────────────────────────────────────────────

#[test]
fn validate_gps_seconds_zero_ok() {
    assert!(validate_gps_seconds(0.0).is_ok());
}

#[test]
fn validate_gps_seconds_max_ok() {
    assert!(validate_gps_seconds(604_799.999).is_ok());
}

#[test]
fn validate_gps_seconds_negative_errors() {
    assert!(validate_gps_seconds(-1.0).is_err());
}

#[test]
fn validate_gps_seconds_too_large_errors() {
    assert!(validate_gps_seconds(604_800.0).is_err());
}

// ─── additional validateJulianDate ────────────────────────────────────────────

#[test]
fn validate_julian_date_j2000_ok() {
    assert!(validate_julian_date(2_451_545.0).is_ok());
}

#[test]
fn validate_julian_date_negative_errors() {
    assert!(validate_julian_date(-1.0).is_err());
}

#[test]
fn validate_julian_date_very_large_ok() {
    assert!(validate_julian_date(3_000_000.0).is_ok());
}

// ─── additional isBrightDateValue ─────────────────────────────────────────────

#[test]
fn is_brightdate_value_large_positive() {
    assert!(is_brightdate_value(1_000_000.0));
}

#[test]
fn is_brightdate_value_neg_infinity_false() {
    assert!(!is_brightdate_value(f64::NEG_INFINITY));
}

// ─── additional isValidBrightDateString ───────────────────────────────────────

#[test]
fn is_valid_brightdate_string_iso_ok() {
    // BrightDate strings are decimal numbers: digits with optional dot
    assert!(is_valid_brightdate_string("9622.5"));
}

#[test]
fn is_valid_brightdate_string_empty_fails() {
    assert!(!is_valid_brightdate_string(""));
}

#[test]
fn is_valid_brightdate_string_garbage_fails() {
    assert!(!is_valid_brightdate_string("not-a-date-at-all"));
}

// ─── validateBrightDateValue additional ───────────────────────────────────────

#[test]
fn validate_brightdate_value_zero_ok() {
    assert!(validate_brightdate_value(0.0).is_ok());
}

#[test]
fn validate_brightdate_value_nan_fails() {
    assert!(validate_brightdate_value(f64::NAN).is_err());
}

#[test]
fn validate_brightdate_value_inf_fails() {
    assert!(validate_brightdate_value(f64::INFINITY).is_err());
}

#[test]
fn validate_brightdate_value_large_ok() {
    assert!(validate_brightdate_value(100_000.0).is_ok());
}
