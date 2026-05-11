//! Core constants for the BrightDate system.
//!
//! # Epoch
//!
//! The BrightDate epoch is **J2000.0** — defined by the IAU as
//! `2000-01-01T12:00:00.000 TT` (Terrestrial Time). This same instant is:
//!
//! | Timescale | Wall-clock label                  |
//! |-----------|-----------------------------------|
//! | TT        | `2000-01-01T12:00:00.000`         |
//! | TAI       | `2000-01-01T11:59:27.816`         |
//! | UTC       | `2000-01-01T11:58:55.816`         |
//! | Julian    | `JD 2451545.0`                    |
//! | Modified  | `MJD 51544.5`                     |
//!
//! `BrightDate 0.0 ≡ J2000.0`, exactly. Bright days are SI days (86400 SI
//! seconds), so the BrightDate timeline ticks uniformly (TAI-coherent) and
//! has no leap-second discontinuities. Leap seconds only intervene when
//! converting to/from UTC labels (`Unix ms`, ISO strings).

use crate::types::Precision;

/// J2000.0 expressed in Unix milliseconds (a UTC-labelled timestamp).
///
/// `946_727_935_816` ms = `2000-01-01T11:58:55.816 UTC`. This is the
/// canonical astronomical instant called J2000.0.
///
/// Historical note: prior versions of this library used `946_728_000_000`
/// (which is actually J2000.0 expressed in *TT* misread as a UTC label —
/// a 64.184 s error). All astronomical claims were silently offset by
/// that amount. Fixed in v1.0.
pub const J2000_UTC_UNIX_MS: f64 = 946_727_935_816.0;

/// J2000.0 in "TAI Unix seconds" — Unix-style elapsed seconds counted on
/// the TAI timescale. Equals `946_727_967.816`. Used internally as the
/// origin against which BrightDate values are measured.
pub const J2000_TAI_UNIX_S: f64 = 946_727_967.816;

/// J2000.0 in "TT Unix seconds" (TT label expressed as Unix-style elapsed
/// seconds on the TT timescale). Equals `946_728_000.000`. This is the
/// number the old library wrongly used as `J2000_UNIX_MS_UTC`.
pub const J2000_TT_UNIX_S: f64 = 946_728_000.0;

/// J2000.0 as a Julian Date — exact by definition of the J2000.0 epoch.
pub const J2000_JD: f64 = 2_451_545.0;

/// J2000.0 as a Modified Julian Date — exact by definition.
pub const J2000_MJD: f64 = 51_544.5;

/// Backwards-compatible alias. **Deprecated.** Use [`J2000_UTC_UNIX_MS`].
///
/// Retained at its *correct* value so user code that subtracted this from
/// Unix ms no longer encodes the 64.184 s error.
#[deprecated(since = "1.0.0", note = "renamed to J2000_UTC_UNIX_MS for clarity")]
pub const J2000_UNIX_MS_UTC: f64 = J2000_UTC_UNIX_MS;

/// Milliseconds per (SI) day.
pub const MS_PER_DAY: f64 = 86_400_000.0;

/// Seconds per (SI) day.
pub const SECONDS_PER_DAY: f64 = 86_400.0;

/// TAI − UTC offset at J2000.0 (32 leap seconds had been inserted by then).
pub const TAI_UTC_OFFSET_AT_J2000: i32 = 32;

/// TT − TAI offset (constant, defined by convention). TT = TAI + 32.184 s.
pub const TT_TAI_OFFSET_SECONDS: f64 = 32.184;

/// Default display precision (5 decimal places ≈ 0.864 s resolution).
pub const DEFAULT_PRECISION: Precision = 5;

/// Maximum supported precision (12 decimal places ≈ 86.4 ps).
pub const MAX_PRECISION: Precision = 12;

/// Current TAI − UTC offset (37 s since 2017-01-01; no new leap seconds as of 2026).
pub const CURRENT_TAI_UTC_OFFSET: i32 = 37;

/// Source of the leap-second table.
pub const LEAP_SECOND_TABLE_SOURCE: &str = "IERS Bulletin C / IANA leap-seconds.list";

/// The Unix-seconds timestamp after which the table should be re-checked.
/// 2028-06-28T00:00:00Z
pub const LEAP_SECOND_TABLE_VALID_UNTIL_UNIX_S: i64 = 1_845_129_600;

/// When the table was last reviewed (ISO date string).
pub const LEAP_SECOND_TABLE_REVIEWED_AT: &str = "2026-05-10";

/// Leap-second table: `(utc_unix_seconds, cumulative_tai_utc_offset)`.
///
/// Each entry marks the moment a new offset became effective.
/// After 2017-01-01 the offset has been 37 s; no new leap seconds have been
/// announced through the table's validity window.
pub const LEAP_SECOND_TABLE: &[(i64, i32)] = &[
    (63_072_000, 10),    // 1972-01-01
    (78_796_800, 11),    // 1972-07-01
    (94_694_400, 12),    // 1973-01-01
    (126_230_400, 13),   // 1974-01-01
    (157_766_400, 14),   // 1975-01-01
    (189_302_400, 15),   // 1976-01-01
    (220_924_800, 16),   // 1977-01-01
    (252_460_800, 17),   // 1978-01-01
    (283_996_800, 18),   // 1979-01-01
    (315_532_800, 19),   // 1980-01-01
    (362_793_600, 20),   // 1981-07-01
    (394_329_600, 21),   // 1982-07-01
    (425_865_600, 22),   // 1983-07-01
    (489_024_000, 23),   // 1985-07-01
    (567_993_600, 24),   // 1988-01-01
    (631_152_000, 25),   // 1990-01-01
    (662_688_000, 26),   // 1991-01-01
    (709_948_800, 27),   // 1992-07-01
    (741_484_800, 28),   // 1993-07-01
    (773_020_800, 29),   // 1994-07-01
    (820_454_400, 30),   // 1996-01-01
    (867_715_200, 31),   // 1997-07-01
    (915_148_800, 32),   // 1999-01-01
    (1_136_073_600, 33), // 2006-01-01
    (1_230_768_000, 34), // 2009-01-01
    (1_341_100_800, 35), // 2012-07-01
    (1_435_708_800, 36), // 2015-07-01
    (1_483_228_800, 37), // 2017-01-01
];

/// Metric unit descriptors: `(name, days_per_unit)`.
pub const METRIC_UNITS: &[(&str, f64)] = &[
    ("days", 1.0),
    ("day", 1.0),
    ("millidays", 1e-3),
    ("milliday", 1e-3),
    ("microdays", 1e-6),
    ("microday", 1e-6),
    ("nanodays", 1e-9),
    ("nanoday", 1e-9),
];
