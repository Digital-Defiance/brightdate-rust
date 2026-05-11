//! Validation utilities for BrightDate inputs.

use crate::types::{BrightDateError, BrightDateValue};

// Unix-ms bounds: year ~1000 to year ~9999
const UNIX_MS_MIN: f64 = -30_610_224_000_000.0;
const UNIX_MS_MAX: f64 = 253_402_300_800_000.0;

// Julian Date bounds: JD 0 to ~year 9999
const JD_MAX: f64 = 5_373_484.0;

const GPS_WEEK_MAX: u32 = 9_999;
const GPS_SECONDS_MAX: f64 = 604_800.0; // exclusive: 0 ..< 604800

/// Validate that a value is a finite number; error code `INVALID_NUMBER`.
pub fn validate_finite_number(value: f64, param_name: &str) -> Result<(), BrightDateError> {
    if !value.is_finite() {
        return Err(BrightDateError::InvalidNumber(format!(
            "'{param_name}' must be a finite number, got {value}"
        )));
    }
    Ok(())
}

/// Validate that a value is a valid BrightDate (finite).
pub fn validate_brightdate_value(value: BrightDateValue) -> Result<(), BrightDateError> {
    validate_finite_number(value, "value")
}

/// Validate precision is an integer in [1, 12].
pub fn validate_precision(precision: f64) -> Result<(), BrightDateError> {
    if precision.fract() != 0.0 || !(1.0..=12.0).contains(&precision) {
        return Err(BrightDateError::InvalidPrecision(format!(
            "Precision must be an integer in [1, 12], got {precision}"
        )));
    }
    Ok(())
}

/// Validate a Unix millisecond timestamp is within supported range.
pub fn validate_unix_ms(ms: f64) -> Result<(), BrightDateError> {
    if !(UNIX_MS_MIN..=UNIX_MS_MAX).contains(&ms) {
        return Err(BrightDateError::OutOfRange(format!(
            "Unix ms timestamp {ms} is out of range [{UNIX_MS_MIN}, {UNIX_MS_MAX}]"
        )));
    }
    Ok(())
}

/// Validate a Julian Date is within supported range [0, 5_373_484].
pub fn validate_julian_date(jd: f64) -> Result<(), BrightDateError> {
    if !(0.0..=JD_MAX).contains(&jd) {
        return Err(BrightDateError::OutOfRange(format!(
            "Julian Date {jd} is out of range [0, {JD_MAX}]"
        )));
    }
    Ok(())
}

/// Validate a GPS week number is in [0, 9999] and is an integer.
pub fn validate_gps_week(week: f64) -> Result<(), BrightDateError> {
    if week.fract() != 0.0 || week < 0.0 || week > GPS_WEEK_MAX as f64 {
        return Err(BrightDateError::InvalidGpsWeek(format!(
            "GPS week must be a non-negative integer <= {GPS_WEEK_MAX}, got {week}"
        )));
    }
    Ok(())
}

/// Validate GPS seconds-into-week is in [0, 604800).
pub fn validate_gps_seconds(seconds: f64) -> Result<(), BrightDateError> {
    if !(0.0..GPS_SECONDS_MAX).contains(&seconds) {
        return Err(BrightDateError::InvalidGpsSeconds(format!(
            "GPS seconds must be in [0, {GPS_SECONDS_MAX}), got {seconds}"
        )));
    }
    Ok(())
}

/// Returns `true` if `value` is a finite `f64`.
pub fn is_brightdate_value(value: f64) -> bool {
    value.is_finite()
}

/// Returns `true` if the string is a valid BrightDate literal
/// (optional `-`, digits, optional single `.` then digits; no scientific notation).
pub fn is_valid_brightdate_string(s: &str) -> bool {
    if s.is_empty() || s.trim().is_empty() {
        return false;
    }
    // Allow optional leading '-', then digits, optional '.' then more digits
    let s = s.trim();
    let s = s.strip_prefix('-').unwrap_or(s);
    if s.is_empty() {
        return false;
    }
    let mut dot_seen = false;
    for ch in s.chars() {
        if ch == '.' {
            if dot_seen {
                return false;
            }
            dot_seen = true;
        } else if !ch.is_ascii_digit() {
            return false;
        }
    }
    // Must have at least one digit
    s.chars().any(|c| c.is_ascii_digit())
}
