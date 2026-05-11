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
