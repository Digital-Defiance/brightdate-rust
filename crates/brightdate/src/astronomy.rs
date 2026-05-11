//! Astronomical utilities: sidereal time, solar position, lunar phase.

use crate::types::BrightDateValue;
use std::f64::consts::PI;

fn deg_to_rad(deg: f64) -> f64 { deg * PI / 180.0 }
fn rad_to_deg(rad: f64) -> f64 { rad * 180.0 / PI }
fn normalize_deg(deg: f64) -> f64 { ((deg % 360.0) + 360.0) % 360.0 }

/// Greenwich Mean Sidereal Time (GMST) for a BrightDate value.
///
/// Uses the IAU 1982 formula; accurate to ~0.1 s over centuries.
/// Returns degrees `[0, 360)`.
pub fn greenwich_mean_sidereal_time(bd: BrightDateValue) -> f64 {
    let t = bd / 36_525.0; // Julian centuries since J2000.0
    let gmst = 280.460_618_37
        + 360.985_647_366_29 * bd
        + 0.000_387_933 * t * t
        - (t * t * t) / 38_710_000.0;
    normalize_deg(gmst)
}

/// Local Mean Sidereal Time (LMST) for a BrightDate and observer longitude.
///
/// `longitude_deg`: East-positive degrees. Returns degrees `[0, 360)`.
pub fn local_mean_sidereal_time(bd: BrightDateValue, longitude_deg: f64) -> f64 {
    normalize_deg(greenwich_mean_sidereal_time(bd) + longitude_deg)
}

/// Julian centuries since J2000.0.
pub fn julian_century(bd: BrightDateValue) -> f64 {
    bd / 36_525.0
}

/// Approximate ecliptic longitude of the Sun (~1° accuracy).
///
/// Returns degrees `[0, 360)`.
pub fn solar_longitude(bd: BrightDateValue) -> f64 {
    let l = normalize_deg(280.46 + 0.985_647_4 * bd);
    let g = normalize_deg(357.528 + 0.985_600_3 * bd);
    let g_rad = deg_to_rad(g);
    let lambda = l + 1.915 * g_rad.sin() + 0.02 * (2.0 * g_rad).sin();
    normalize_deg(lambda)
}

/// Approximate solar declination in degrees.
pub fn solar_declination(bd: BrightDateValue) -> f64 {
    let lambda = solar_longitude(bd);
    let eps = 23.439 - 0.000_000_36 * bd; // approximate obliquity
    let sin_dec = deg_to_rad(eps).sin() * deg_to_rad(lambda).sin();
    rad_to_deg(sin_dec.asin())
}

/// Approximate solar right ascension in degrees `[0, 360)`.
pub fn solar_right_ascension(bd: BrightDateValue) -> f64 {
    let lambda = solar_longitude(bd);
    let eps = 23.439 - 0.000_000_36 * bd;
    let ra = rad_to_deg(
        deg_to_rad(lambda).sin().atan2(
            deg_to_rad(eps).cos() * deg_to_rad(lambda).cos(),
        ),
    );
    normalize_deg(ra)
}

/// Approximate lunar phase angle in degrees (0 = new moon, 180 = full moon).
pub fn lunar_phase_angle(bd: BrightDateValue) -> f64 {
    // Simplified — Meeus Chapter 49 low-precision
    let d = bd;
    let m_sun = normalize_deg(357.529 + 0.985_600_3 * d);
    let m_moon = normalize_deg(134.963 + 13.064_993 * d);
    let f = normalize_deg(93.272 + 13.229_350 * d);
    let lambda_moon = normalize_deg(
        218.316
            + 13.176_396 * d
            + 6.289 * deg_to_rad(m_moon).sin()
            - 1.274 * deg_to_rad(2.0 * f - m_moon).sin()
            + 0.658 * deg_to_rad(2.0 * f).sin()
            - 0.214 * deg_to_rad(2.0 * m_moon).sin()
            - 0.110 * deg_to_rad(m_sun).sin(),
    );
    let sol = solar_longitude(bd);
    normalize_deg(lambda_moon - sol)
}

/// Approximate lunar illuminated fraction (0 = new, 1 = full).
pub fn lunar_illuminated_fraction(bd: BrightDateValue) -> f64 {
    let phase = lunar_phase_angle(bd);
    (1.0 - deg_to_rad(phase).cos()) / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gmst_in_range() {
        let gmst = greenwich_mean_sidereal_time(9622.0);
        assert!(gmst >= 0.0 && gmst < 360.0);
    }

    #[test]
    fn lunar_fraction_bounds() {
        for d in [0.0_f64, 1000.0, 9622.0, -1000.0] {
            let f = lunar_illuminated_fraction(d);
            assert!(f >= 0.0 && f <= 1.0, "fraction={f} at d={d}");
        }
    }
}
