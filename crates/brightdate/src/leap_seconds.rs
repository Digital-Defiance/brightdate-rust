//! Leap-second table lookup utilities.

use crate::constants::{
    LEAP_SECOND_TABLE, LEAP_SECOND_TABLE_VALID_UNTIL_UNIX_S,
};
use std::sync::atomic::{AtomicBool, Ordering};

static STALE_WARNED: AtomicBool = AtomicBool::new(false);

fn warn_if_stale(utc_unix_seconds: i64) {
    if utc_unix_seconds <= LEAP_SECOND_TABLE_VALID_UNTIL_UNIX_S {
        return;
    }
    if STALE_WARNED.swap(true, Ordering::Relaxed) {
        return;
    }
    eprintln!(
        "[brightdate] Leap-second table may be stale (queried Unix s={}). \
         TAI calculations could be off by ±1 s if a new leap second was \
         inserted after the table was reviewed. Update LEAP_SECOND_TABLE \
         from IERS Bulletin C if you need authoritative TAI past that date.",
        utc_unix_seconds
    );
}

/// Return the TAI − UTC offset (seconds) for the given UTC Unix timestamp.
///
/// Uses binary search on the leap-second table — O(log n).
pub fn get_tai_utc_offset(utc_unix_seconds: i64) -> i32 {
    warn_if_stale(utc_unix_seconds);

    if LEAP_SECOND_TABLE.is_empty() || utc_unix_seconds < LEAP_SECOND_TABLE[0].0 {
        // Before the era of integer leap seconds (pre-1972), return the
        // initial offset of 10 s as a reasonable approximation.
        return 10;
    }

    let mut lo = 0usize;
    let mut hi = LEAP_SECOND_TABLE.len() - 1;

    while lo < hi {
        let mid = lo + (hi - lo).div_ceil(2);
        if LEAP_SECOND_TABLE[mid].0 <= utc_unix_seconds {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }

    LEAP_SECOND_TABLE[lo].1
}

/// Convert a UTC Unix timestamp (seconds) to TAI Unix timestamp (seconds).
///
/// This is unambiguous because Unix time has no representation for a leap
/// second instant (`23:59:60`) — every Unix-second input is a regular UTC
/// second and maps to exactly one TAI second.
pub fn utc_to_tai(utc_unix_seconds: i64) -> i64 {
    utc_unix_seconds + get_tai_utc_offset(utc_unix_seconds) as i64
}

/// Result of converting TAI → UTC.
///
/// During a leap-second insertion, a TAI second exists that has no regular
/// Unix-second representation; it's the fictional `23:59:60` displayed on
/// some UTC clocks. When `is_leap_second` is `true`, `utc_unix_seconds`
/// holds the Unix second that was *repeated* (i.e. `boundary - 1`), and
/// formatters should emit `:60` instead of `:59`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaiToUtc {
    /// UTC Unix seconds (timestamp). For leap-second instants this is the
    /// repeated second (`boundary − 1`), matching the NTP convention.
    pub utc_unix_seconds: i64,
    /// `true` iff this TAI instant is a leap second (UTC label `23:59:60`).
    pub is_leap_second: bool,
}

/// Convert a TAI Unix timestamp (seconds) to a UTC labelling.
///
/// Returns an explicit struct rather than a bare `i64` because the mapping
/// is not bijective: leap-second TAI instants need a side-channel flag to
/// be distinguishable from the following regular UTC second.
///
/// **Convention:** During a leap-second insertion at boundary `B` (the UTC
/// Unix-second when the offset increments), the TAI second immediately
/// preceding `B + new_offset` is the leap second. It is reported as
/// `(B - 1, is_leap_second: true)`, i.e. the previous second repeated,
/// matching how NTP and Linux `clock_gettime` (with `CLOCK_TAI`) behave.
pub fn tai_to_utc_full(tai_unix_seconds: i64) -> TaiToUtc {
    // Probe with the maximum offset (post-2017 = 37) to get an approximate
    // UTC, then look up the actual offset at that approximate UTC.
    let probe_utc = tai_unix_seconds - 37;
    let offset_at_probe = get_tai_utc_offset(probe_utc);
    let candidate_utc = tai_unix_seconds - offset_at_probe as i64;

    // Verify: does the offset at `candidate_utc` give back the same TAI?
    let offset_at_candidate = get_tai_utc_offset(candidate_utc);
    if offset_at_candidate == offset_at_probe {
        // Stable: no leap-second boundary crossed.
        return TaiToUtc {
            utc_unix_seconds: candidate_utc,
            is_leap_second: false,
        };
    }

    // The offsets disagree — `tai_unix_seconds` straddles a leap-second
    // insertion. The candidate UTC under the *new* (post-boundary) offset:
    let new_offset = offset_at_candidate as i64;
    let utc_under_new = tai_unix_seconds - new_offset;

    // If `utc_under_new` lands exactly at a boundary `B`, then this TAI
    // instant is the first regular second of the new era (00:00:00).
    // The leap second itself is one TAI second earlier and corresponds
    // to `B - 1` with `is_leap_second = true`.
    //
    // We detect "is this the boundary?" by checking whether removing the
    // *old* offset would land before the boundary (i.e. `tai - old_offset`
    // < `boundary`, equivalently `tai - old_offset == boundary - 1`).
    let utc_under_old = tai_unix_seconds - offset_at_probe as i64;
    if utc_under_old == utc_under_new - 1 {
        // TAI = boundary - 1 under old offset → this is the leap second.
        TaiToUtc {
            utc_unix_seconds: utc_under_new - 1,
            is_leap_second: true,
        }
    } else {
        // TAI = boundary under new offset → first regular second.
        TaiToUtc {
            utc_unix_seconds: utc_under_new,
            is_leap_second: false,
        }
    }
}

/// Convert a TAI Unix timestamp to a UTC Unix timestamp (legacy API).
///
/// Discards the leap-second flag. Prefer [`tai_to_utc_full`] for
/// astronomically precise work that needs to render `23:59:60`.
pub fn tai_to_utc(tai_unix_seconds: i64) -> i64 {
    tai_to_utc_full(tai_unix_seconds).utc_unix_seconds
}

/// Return the TAI − UTC offset that was in effect at the J2000.0 epoch (32 s).
pub fn get_tai_utc_offset_at_j2000() -> i32 {
    // J2000.0 = 2000-01-01T12:00:00Z = Unix s 946_728_000.
    // The 1999-01-01 entry sets offset=32; the next entry (2006-01-01) sets 33.
    get_tai_utc_offset(946_728_000)
}

/// True if the UTC second at `utc_unix_seconds` is a leap second.
///
/// A leap second occupies the window `[boundary - 1, boundary)` where
/// `boundary` is the UTC Unix timestamp of the new offset's start.
pub fn is_during_leap_second(utc_unix_seconds: i64) -> bool {
    // The offset changes at the *next* second
    get_tai_utc_offset(utc_unix_seconds + 1) > get_tai_utc_offset(utc_unix_seconds)
}

/// Number of leap seconds inserted between two UTC Unix timestamps (absolute).
pub fn leap_seconds_between(from: i64, to: i64) -> i32 {
    let a = get_tai_utc_offset(from);
    let b = get_tai_utc_offset(to);
    (a - b).abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offset_at_j2000() {
        // 2000-01-01T12:00:00Z → Unix s = 946_728_000
        assert_eq!(get_tai_utc_offset(946_728_000), 32);
    }

    #[test]
    fn offset_after_2017() {
        // 2020-06-15T00:00:00Z → Unix s = 1_592_179_200
        assert_eq!(get_tai_utc_offset(1_592_179_200), 37);
    }

    #[test]
    fn pre_table_returns_10() {
        assert_eq!(get_tai_utc_offset(0), 10);
    }
}
