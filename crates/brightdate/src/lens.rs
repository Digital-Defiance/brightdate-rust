//! BrightDate lens — integer engine ↔ decimal-day presentation.
//!
//! Canonical storage uses signed integer ticks (attoseconds or picoseconds
//! since J2000.0). The v1 `f64` decimal-day value is a **lossy lens**
//! derived via Euclidean divmod.

/// Attoseconds per SI second (exact).
pub const ATTOSECONDS_PER_SECOND: i128 = 1_000_000_000_000_000_000;

/// Picoseconds per attosecond (exact).
pub const ATTOSECONDS_PER_PICOSECOND: i128 = 1_000_000;

/// Attoseconds per SI day.
pub const ATTOSECONDS_PER_DAY: i128 = 86_400 * ATTOSECONDS_PER_SECOND;

/// Picoseconds per SI day.
pub const PICOSECONDS_PER_DAY: i128 = ATTOSECONDS_PER_DAY / ATTOSECONDS_PER_PICOSECOND;

/// Split ticks into whole days and non-negative remainder (Euclidean).
#[inline]
pub fn split_ticks_into_day_parts(ticks: i128, ticks_per_day: i128) -> (i128, i128) {
    let days = ticks.div_euclid(ticks_per_day);
    let rem = ticks.rem_euclid(ticks_per_day);
    (days, rem)
}

/// Integer divmod → `f64` decimal days (lossy only in the final `f64` combine).
#[inline]
pub fn ticks_to_brightdate(ticks: i128, ticks_per_day: i128) -> f64 {
    let (days, rem) = split_ticks_into_day_parts(ticks, ticks_per_day);
    days as f64 + (rem as f64 / ticks_per_day as f64)
}

/// Lossy: `f64` decimal days → attoseconds (floor days + round fractional limb).
pub fn brightdate_to_attoseconds(bd: f64) -> Result<i128, crate::types::BrightDateError> {
    if !bd.is_finite() {
        return Err(crate::types::BrightDateError::InvalidInput(format!(
            "expected finite BrightDate, got {bd}"
        )));
    }
    let days_floor = bd.floor();
    let frac = bd - days_floor;
    let days_i = days_floor as i128;
    let frac_as = (frac * ATTOSECONDS_PER_DAY as f64).round() as i128;
    Ok(days_i * ATTOSECONDS_PER_DAY + frac_as)
}

/// Lossy: `f64` decimal days → picoseconds.
pub fn brightdate_to_picoseconds(bd: f64) -> Result<i128, crate::types::BrightDateError> {
    if !bd.is_finite() {
        return Err(crate::types::BrightDateError::InvalidInput(format!(
            "expected finite BrightDate, got {bd}"
        )));
    }
    let days_floor = bd.floor();
    let frac = bd - days_floor;
    let days_i = days_floor as i128;
    let frac_ps = (frac * PICOSECONDS_PER_DAY as f64).round() as i128;
    Ok(days_i * PICOSECONDS_PER_DAY + frac_ps)
}
