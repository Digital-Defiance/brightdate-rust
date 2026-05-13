use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

/// Convert a C argc/argv pair into a `Vec<String>`.
///
/// # Safety
/// `argv` must be a valid array of `argc` non-null, NUL-terminated C strings.
unsafe fn argv_to_vec(argc: c_int, argv: *const *const c_char) -> Vec<String> {
    (0..argc as usize)
        .map(|i| {
            // SAFETY: caller guarantees each pointer is valid and NUL-terminated
            CStr::from_ptr(*argv.add(i))
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

/// `bdate` builtin entry point called from the bsh C module.
///
/// # Safety
/// `argv` must be a valid array of `argc` NUL-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn bsh_bdate(argc: c_int, argv: *const *const c_char) -> c_int {
    bdate::run(&argv_to_vec(argc, argv))
}

/// `btime` builtin entry point called from the bsh C module.
///
/// # Safety
/// `argv` must be a valid array of `argc` NUL-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn bsh_btime(argc: c_int, argv: *const *const c_char) -> c_int {
    btime::run(&argv_to_vec(argc, argv))
}

/// `buptime` builtin entry point called from the bsh C module.
///
/// # Safety
/// `argv` must be a valid array of `argc` NUL-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn bsh_buptime(argc: c_int, argv: *const *const c_char) -> c_int {
    buptime::run(&argv_to_vec(argc, argv))
}

/// `bcal` builtin entry point called from the bsh C module.
///
/// # Safety
/// `argv` must be a valid array of `argc` NUL-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn bsh_bcal(argc: c_int, argv: *const *const c_char) -> c_int {
    bcal::run(&argv_to_vec(argc, argv))
}

/// `bwatch` builtin entry point called from the bsh C module.
///
/// # Safety
/// `argv` must be a valid array of `argc` NUL-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn bsh_bwatch(argc: c_int, argv: *const *const c_char) -> c_int {
    bwatch::run(&argv_to_vec(argc, argv))
}

/// Returns the current BrightDate decimal value for use in $BRIGHTDATE.
#[no_mangle]
pub extern "C" fn bsh_brightdate_now() -> f64 {
    brightdate::BrightDate::now().value
}

/// Convert a Unix timestamp (seconds) to a BrightDate value.
/// Returns NaN if the input is non-finite. Used to display history timestamps
/// and login/logout times in BrightDate format throughout bsh.
#[no_mangle]
pub extern "C" fn bsh_unix_to_brightdate(unix_secs: f64) -> f64 {
    brightdate::BrightDate::from_unix_seconds(unix_secs)
        .map(|bd| bd.value)
        .unwrap_or(f64::NAN)
}

/// Convert a BrightDate decimal value to a Unix timestamp (seconds).
/// Returns NaN if the input is non-finite. Used by the sched builtin to
/// accept absolute BrightDate values as scheduled times.
#[no_mangle]
pub extern "C" fn bsh_brightdate_to_unix(bd: f64) -> f64 {
    if !bd.is_finite() {
        return f64::NAN;
    }
    brightdate::BrightDate::from_value(bd).to_unix_seconds()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// J2000.0 epoch in UTC: 2000-01-01T11:58:55.816 UTC = Unix 946727935.816
    const J2000_UNIX: f64 = 946727935.816;
    /// Tolerance for round-trip comparisons (1 ms).
    const TOL: f64 = 0.001;

    fn approx_eq(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() < tol
    }

    // ── bsh_unix_to_brightdate ────────────────────────────────────────────

    #[test]
    fn unix_to_bd_epoch_gives_zero() {
        // J2000.0 Unix timestamp should yield BD ≈ 0.0
        let bd = bsh_unix_to_brightdate(J2000_UNIX);
        assert!(approx_eq(bd, 0.0, TOL / 86400.0),
            "expected BD ~0.0, got {bd}");
    }

    #[test]
    fn unix_to_bd_nan_input_returns_nan() {
        assert!(bsh_unix_to_brightdate(f64::NAN).is_nan());
    }

    #[test]
    fn unix_to_bd_infinity_returns_nan() {
        assert!(bsh_unix_to_brightdate(f64::INFINITY).is_nan());
        assert!(bsh_unix_to_brightdate(f64::NEG_INFINITY).is_nan());
    }

    // ── bsh_brightdate_to_unix ────────────────────────────────────────────

    #[test]
    fn bd_to_unix_epoch_gives_j2000_unix() {
        // BD 0.0 (J2000.0 epoch) should return Unix ≈ 946727935.816
        let unix = bsh_brightdate_to_unix(0.0);
        assert!(approx_eq(unix, J2000_UNIX, TOL),
            "expected unix ~{J2000_UNIX}, got {unix}");
    }

    #[test]
    fn bd_to_unix_nan_input_returns_nan() {
        assert!(bsh_brightdate_to_unix(f64::NAN).is_nan());
    }

    #[test]
    fn bd_to_unix_infinity_returns_nan() {
        assert!(bsh_brightdate_to_unix(f64::INFINITY).is_nan());
        assert!(bsh_brightdate_to_unix(f64::NEG_INFINITY).is_nan());
    }

    // ── round-trip ────────────────────────────────────────────────────────

    #[test]
    fn round_trip_unix_to_bd_and_back() {
        // A known Unix timestamp (2026-05-12T00:00:00 UTC).
        let unix_in: f64 = 1778601600.0;
        let bd = bsh_unix_to_brightdate(unix_in);
        assert!(!bd.is_nan(), "intermediate BD should not be NaN");
        let unix_out = bsh_brightdate_to_unix(bd);
        assert!(approx_eq(unix_in, unix_out, TOL),
            "round-trip drift > 1 ms: {unix_in} → {bd} → {unix_out}");
    }

    #[test]
    fn round_trip_bd_to_unix_and_back() {
        // A known BrightDate value (approximately 2026-05-12).
        let bd_in: f64 = 9627.884852;
        let unix = bsh_brightdate_to_unix(bd_in);
        assert!(!unix.is_nan(), "intermediate unix should not be NaN");
        let bd_out = bsh_unix_to_brightdate(unix);
        assert!(approx_eq(bd_in, bd_out, TOL / 86400.0),
            "round-trip drift > 1 ms: {bd_in} → {unix} → {bd_out}");
    }

    #[test]
    fn positive_day_offset_increases_unix_time() {
        let unix0 = bsh_brightdate_to_unix(0.0);
        let unix1 = bsh_brightdate_to_unix(1.0); // one day later
        assert!(approx_eq(unix1 - unix0, 86400.0, TOL),
            "one BD day should be 86400 unix seconds, got {}", unix1 - unix0);
    }
}
