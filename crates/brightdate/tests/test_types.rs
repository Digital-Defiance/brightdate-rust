use brightdate::types::*;

// ── BrightDateError variants ──────────────────────────────────────────────────

#[test]
fn error_invalid_input_display() {
    let e = BrightDateError::InvalidInput("test".to_string());
    let s = e.to_string();
    assert!(s.contains("test"), "got: {s}");
}

#[test]
fn error_parse_error_display() {
    let e = BrightDateError::ParseError("bad".to_string());
    let s = e.to_string();
    assert!(s.contains("bad"), "got: {s}");
}

#[test]
fn error_invalid_number_display() {
    let e = BrightDateError::InvalidNumber("NaN".to_string());
    let s = e.to_string();
    assert!(s.contains("NaN"), "got: {s}");
}

#[test]
fn error_invalid_precision_display() {
    let e = BrightDateError::InvalidPrecision("99".to_string());
    let s = e.to_string();
    assert!(s.contains("99"), "got: {s}");
}

#[test]
fn error_out_of_range_display() {
    let e = BrightDateError::OutOfRange("too large".to_string());
    let s = e.to_string();
    assert!(s.contains("too large"), "got: {s}");
}

#[test]
fn error_invalid_gps_week_display() {
    let e = BrightDateError::InvalidGpsWeek("week -1".to_string());
    let s = e.to_string();
    assert!(s.contains("week -1"), "got: {s}");
}

#[test]
fn error_invalid_gps_seconds_display() {
    let e = BrightDateError::InvalidGpsSeconds("604801".to_string());
    let s = e.to_string();
    assert!(s.contains("604801"), "got: {s}");
}

#[test]
fn error_debug_formatting() {
    let e = BrightDateError::InvalidInput("x".to_string());
    let s = format!("{e:?}");
    assert!(s.contains("InvalidInput"), "got: {s}");
}

#[test]
fn error_equality() {
    let a = BrightDateError::ParseError("x".to_string());
    let b = BrightDateError::ParseError("x".to_string());
    assert_eq!(a, b);
}

#[test]
fn error_inequality_different_variant() {
    let a = BrightDateError::ParseError("x".to_string());
    let b = BrightDateError::InvalidInput("x".to_string());
    assert_ne!(a, b);
}

#[test]
fn error_inequality_different_message() {
    let a = BrightDateError::ParseError("a".to_string());
    let b = BrightDateError::ParseError("b".to_string());
    assert_ne!(a, b);
}

#[test]
fn error_clone() {
    let e = BrightDateError::OutOfRange("far".to_string());
    let c = e.clone();
    assert_eq!(e, c);
}

// ── BrightDateComponents ──────────────────────────────────────────────────────

#[test]
fn bright_date_components_fields_accessible() {
    let comp = BrightDateComponents {
        day: 100,
        fraction: 0.5,
        value: 100.5,
        millidays: 500,
        microdays: 0,
        nanodays: 0,
    };
    assert_eq!(comp.day, 100);
    assert!((comp.fraction - 0.5).abs() < 1e-15);
    assert!((comp.value - 100.5).abs() < 1e-15);
    assert_eq!(comp.millidays, 500);
    assert_eq!(comp.microdays, 0);
}

#[test]
fn bright_date_components_debug() {
    let comp = BrightDateComponents {
        day: 0, fraction: 0.0, value: 0.0, millidays: 0, microdays: 0, nanodays: 0,
    };
    let s = format!("{comp:?}");
    assert!(s.contains("BrightDateComponents"), "got: {s}");
}

#[test]
fn bright_date_components_clone() {
    let comp = BrightDateComponents {
        day: 42, fraction: 0.25, value: 42.25, millidays: 250, microdays: 0, nanodays: 0,
    };
    let c = comp.clone();
    assert_eq!(comp, c);
}

#[test]
fn bright_date_components_equality() {
    let c1 = BrightDateComponents { day: 10, fraction: 0.1, value: 10.1, millidays: 100, microdays: 0, nanodays: 0 };
    let c2 = BrightDateComponents { day: 10, fraction: 0.1, value: 10.1, millidays: 100, microdays: 0, nanodays: 0 };
    assert_eq!(c1, c2);
}

#[test]
fn bright_date_components_negative_day() {
    let comp = BrightDateComponents {
        day: -365,
        fraction: 0.5,
        value: -364.5,
        millidays: 500,
        microdays: 0,
        nanodays: 0,
    };
    assert_eq!(comp.day, -365);
}

// ── BrightDuration ────────────────────────────────────────────────────────────

#[test]
fn bright_duration_fields_accessible() {
    let dur = BrightDuration {
        days: 1.0,
        millidays: 1000.0,
        microdays: 1_000_000.0,
        nanodays: 1_000_000_000.0,
    };
    assert!((dur.days - 1.0).abs() < 1e-15);
    assert!((dur.millidays - 1000.0).abs() < 1e-12);
    assert!((dur.microdays - 1_000_000.0).abs() < 1e-9);
    assert!((dur.nanodays - 1_000_000_000.0).abs() < 1.0);
}

#[test]
fn bright_duration_debug() {
    let dur = BrightDuration { days: 0.0, millidays: 0.0, microdays: 0.0, nanodays: 0.0 };
    let s = format!("{dur:?}");
    assert!(s.contains("BrightDuration"), "got: {s}");
}

#[test]
fn bright_duration_clone() {
    let dur = BrightDuration { days: 7.0, millidays: 7000.0, microdays: 7_000_000.0, nanodays: 7e9 };
    let c = dur.clone();
    assert_eq!(dur, c);
}

#[test]
fn bright_duration_equality() {
    let d1 = BrightDuration { days: 2.5, millidays: 2500.0, microdays: 2_500_000.0, nanodays: 2.5e9 };
    let d2 = BrightDuration { days: 2.5, millidays: 2500.0, microdays: 2_500_000.0, nanodays: 2.5e9 };
    assert_eq!(d1, d2);
}

// ── BrightDateOptions ─────────────────────────────────────────────────────────

#[test]
fn bright_date_options_default_all_none() {
    let opts = BrightDateOptions::default();
    assert!(opts.precision.is_none());
    assert!(opts.use_tai.is_none());
}

#[test]
fn bright_date_options_set_precision() {
    let opts = BrightDateOptions { precision: Some(8), use_tai: None };
    assert_eq!(opts.precision, Some(8));
}

#[test]
fn bright_date_options_set_tai() {
    let opts = BrightDateOptions { precision: None, use_tai: Some(true) };
    assert_eq!(opts.use_tai, Some(true));
}

#[test]
fn bright_date_options_clone() {
    let opts = BrightDateOptions { precision: Some(5), use_tai: Some(false) };
    let c = opts.clone();
    assert_eq!(opts.precision, c.precision);
    assert_eq!(opts.use_tai, c.use_tai);
}

#[test]
fn bright_date_options_debug() {
    let opts = BrightDateOptions::default();
    let s = format!("{opts:?}");
    assert!(s.contains("BrightDateOptions"), "got: {s}");
}

// ── Type aliases ──────────────────────────────────────────────────────────────

#[test]
fn bright_date_value_is_f64() {
    let v: BrightDateValue = 9622.5;
    assert!((v - 9622.5).abs() < 1e-15);
}

#[test]
fn precision_is_u8() {
    let p: Precision = 5;
    assert_eq!(p, 5u8);
}
