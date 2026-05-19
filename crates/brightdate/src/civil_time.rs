//! BrightDate Local-Clock Bridge.
//!
//! BrightDate has **one** scalar and **one** fraction: the BD-day fraction
//! (`bd - floor(bd)`). It is universal — every observer at the same instant
//! sees the same number. There is no UTC fraction, local fraction, or any
//! per-observer flavor of BrightDate.
//!
//! What this module *does* provide is a single helper to answer one
//! legitimate question:
//!
//! > At the instant a particular wall clock reads HH:MM:SS today, what
//! > BD scalar will it produce?
//!
//! The output is still the universal BD scalar; only the *input* (a local
//! wall-clock value plus a UTC offset) is observer-specific. Two callers in
//! different zones asking for "their 11:00" get different BD scalars,
//! because they are different physical instants — not because BrightDate
//! has a localized form.
//!
//! Use this for UI tables that map a user's local hour to its BD scalar.
//! Don't use it (or anything else) to invent a second "fraction".

use crate::conversions::{from_unix_ms, to_unix_ms};
use crate::types::{BrightDateError, BrightDateValue};

const MS_PER_DAY: f64 = 86_400_000.0;

/// The BrightDate value at the instant a **local wall clock** reads
/// `hours:minutes:seconds` on the same local civil date as `reference`,
/// given a fixed UTC offset.
///
/// **No "local fraction" is implied.** BrightDate is timezone-free; the
/// value returned is the universal BD scalar at the matching UTC instant.
///
/// # DST
///
/// Assumes a fixed offset for the day. For DST-aware behavior across a
/// transition, compute the correct UTC instant via your platform's calendar
/// API (`chrono-tz`, etc.) and call [`crate::conversions::from_unix_ms`]
/// directly.
///
/// # Arguments
///
/// - `reference`: any BD value on the target local civil date
/// - `hours`: 0–23
/// - `minutes`: 0–59
/// - `seconds`: 0–59.999...
/// - `offset_days`: UTC offset in fractional days (positive east of UTC,
///   negative west). For Pacific Daylight Time use `-7.0 / 24.0`.
///
/// # Example
///
/// ```
/// use brightdate::civil_time::bd_from_local_clock;
/// // The BD value when a PDT (UTC-7) wall clock reads 11:00 on the local
/// // civil day containing the J2000.0 anchor.
/// let bd = bd_from_local_clock(0.0, 11, 0, 0.0, -7.0 / 24.0).unwrap();
/// assert!(bd.is_finite());
/// ```
pub fn bd_from_local_clock(
    reference: BrightDateValue,
    hours: u32,
    minutes: u32,
    seconds: f64,
    offset_days: f64,
) -> Result<BrightDateValue, BrightDateError> {
    let ref_ms = to_unix_ms(reference);
    let offset_ms = offset_days * MS_PER_DAY;
    // Shift into local time so floor(local_ms / day) lands on the local day.
    let local_ms = ref_ms + offset_ms;
    let local_day_start_ms = (local_ms / MS_PER_DAY).floor() * MS_PER_DAY;
    // Convert the local-day-start back to a UTC instant.
    let utc_day_start_ms = local_day_start_ms - offset_ms;
    let target_ms = utc_day_start_ms
        + (hours as f64 * 3_600.0 + minutes as f64 * 60.0 + seconds) * 1_000.0;
    from_unix_ms(target_ms)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversions::from_unix_ms;

    fn date_utc_ms(y: i32, m: u32, d: u32, h: u32, mi: u32, s: u32) -> f64 {
        use chrono::{TimeZone, Utc};
        Utc.with_ymd_and_hms(y, m, d, h, mi, s)
            .single()
            .unwrap()
            .timestamp_millis() as f64
    }

    #[test]
    fn offset_zero_lands_on_requested_utc_clock() {
        let bd = bd_from_local_clock(0.0, 14, 30, 0.0, 0.0).unwrap();
        // The result should be at UTC 14:30 on the same UTC civil date as
        // the J2000 anchor — i.e. 14:30 UTC on 2000-01-01.
        let ms = to_unix_ms(bd);
        let expected = date_utc_ms(2000, 1, 1, 14, 30, 0);
        assert!((ms - expected).abs() < 1.0, "ms {ms} vs {expected}");
    }

    #[test]
    fn pdt_11_lands_on_utc_18_same_date() {
        let reference = from_unix_ms(date_utc_ms(2024, 6, 15, 18, 0, 0)).unwrap();
        let bd = bd_from_local_clock(reference, 11, 0, 0.0, -7.0 / 24.0).unwrap();
        let ms = to_unix_ms(bd);
        let expected = date_utc_ms(2024, 6, 15, 18, 0, 0);
        assert!((ms - expected).abs() < 1.0);
    }

    #[test]
    fn jst_22_lands_on_utc_13_same_local_day() {
        let reference = from_unix_ms(date_utc_ms(2024, 6, 15, 12, 0, 0)).unwrap();
        let bd = bd_from_local_clock(reference, 22, 0, 0.0, 9.0 / 24.0).unwrap();
        let ms = to_unix_ms(bd);
        let expected = date_utc_ms(2024, 6, 15, 13, 0, 0);
        assert!((ms - expected).abs() < 1.0);
    }

    #[test]
    fn ist_preserves_half_hour_offset() {
        let reference = from_unix_ms(date_utc_ms(2024, 6, 15, 12, 0, 0)).unwrap();
        // 09:00 IST = 03:30 UTC same day.
        let bd = bd_from_local_clock(reference, 9, 0, 0.0, 5.5 / 24.0).unwrap();
        let ms = to_unix_ms(bd);
        let expected = date_utc_ms(2024, 6, 15, 3, 30, 0);
        assert!((ms - expected).abs() < 1.0);
    }
}
