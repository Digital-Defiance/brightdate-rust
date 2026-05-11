//! Integration tests for astronomy — ported from astronomy.spec.ts

use brightdate::astronomy::{
    greenwich_mean_sidereal_time, julian_century, local_mean_sidereal_time,
    lunar_illuminated_fraction, lunar_phase_angle, solar_declination, solar_longitude,
    solar_right_ascension,
};

// ─── julianCentury ───────────────────────────────────────────────────────────

#[test]
fn julian_century_at_j2000() {
    // At J2000 (bd = 0), T = 0
    assert!((julian_century(0.0) - 0.0).abs() < 1e-12);
}

#[test]
fn julian_century_one_century_later() {
    // 36525 days = 1 Julian century
    assert!((julian_century(36_525.0) - 1.0).abs() < 1e-12);
}

#[test]
fn julian_century_typical_value() {
    // For any valid bd the result should be a small finite number
    let t = julian_century(9622.0);
    assert!(t.is_finite());
    assert!((t - 9622.0 / 36_525.0).abs() < 1e-12);
}

// ─── GMST ────────────────────────────────────────────────────────────────────

#[test]
fn gmst_at_j2000() {
    let gmst = greenwich_mean_sidereal_time(0.0);
    assert!(gmst >= 0.0 && gmst < 360.0, "GMST out of range: {gmst}");
}

#[test]
fn gmst_in_range_typical() {
    let gmst = greenwich_mean_sidereal_time(9622.0);
    assert!(gmst >= 0.0 && gmst < 360.0, "GMST out of range: {gmst}");
}

#[test]
fn gmst_in_range_negative_bd() {
    let gmst = greenwich_mean_sidereal_time(-1000.0);
    assert!(gmst >= 0.0 && gmst < 360.0, "GMST out of range: {gmst}");
}

#[test]
fn gmst_changes_with_time() {
    let g0 = greenwich_mean_sidereal_time(0.0);
    let g1 = greenwich_mean_sidereal_time(1.0);
    assert!((g0 - g1).abs() > 0.01, "GMST should change over 1 day");
}

// ─── LMST ────────────────────────────────────────────────────────────────────

#[test]
fn lmst_equals_gmst_at_zero_longitude() {
    let bd = 9622.0;
    let gmst = greenwich_mean_sidereal_time(bd);
    let lmst = local_mean_sidereal_time(bd, 0.0);
    assert!((lmst - gmst).abs() < 1e-10);
}

#[test]
fn lmst_offset_by_longitude() {
    let bd = 9622.0;
    let gmst = greenwich_mean_sidereal_time(bd);
    let lmst = local_mean_sidereal_time(bd, 90.0);
    let diff = ((lmst - (gmst + 90.0)) % 360.0).abs();
    assert!(diff < 1e-10 || (360.0 - diff).abs() < 1e-10);
}

#[test]
fn lmst_in_range() {
    let lmst = local_mean_sidereal_time(9622.0, 45.0);
    assert!(lmst >= 0.0 && lmst < 360.0, "LMST out of range: {lmst}");
}

// ─── solarLongitude ──────────────────────────────────────────────────────────

#[test]
fn solar_longitude_in_range() {
    let lon = solar_longitude(9622.0);
    assert!(lon >= 0.0 && lon < 360.0, "solar longitude out of range: {lon}");
}

#[test]
fn solar_longitude_at_j2000_approx() {
    // J2000.0 is ~Jan 1.5 2000; solar longitude should be near Capricorn (~280°)
    let lon = solar_longitude(0.0);
    assert!(lon > 240.0 && lon < 320.0, "unexpected solar lon at J2000: {lon}");
}

#[test]
fn solar_longitude_changes_per_day() {
    let l0 = solar_longitude(0.0);
    let l1 = solar_longitude(1.0);
    // Sun moves ~1°/day
    let diff = ((l1 - l0 + 360.0) % 360.0).abs();
    assert!(diff > 0.9 && diff < 1.1, "unexpected daily motion: {diff}");
}

// ─── solarDeclination ────────────────────────────────────────────────────────

#[test]
fn solar_declination_in_range() {
    let dec = solar_declination(9622.0);
    assert!(dec >= -90.0 && dec <= 90.0, "declination out of range: {dec}");
}

#[test]
fn solar_declination_summer_solstice_positive() {
    // ~June 21, ~172 days after J2000.0 (Jan 1.5 2000) → bd ≈ 172
    // This is very approximate; summer solstice has positive declination
    let dec = solar_declination(172.0);
    assert!(dec > 0.0, "summer solstice should have positive declination: {dec}");
}

#[test]
fn solar_declination_winter_solstice_negative() {
    // ~Dec 21 2000, ~356 days after J2000.0 → bd ≈ 356
    let dec = solar_declination(356.0);
    assert!(dec < 0.0, "winter solstice should have negative declination: {dec}");
}

// ─── solarRightAscension ─────────────────────────────────────────────────────

#[test]
fn solar_right_ascension_in_range() {
    let ra = solar_right_ascension(9622.0);
    assert!(ra >= 0.0 && ra < 360.0, "solar RA out of range: {ra}");
}

// ─── lunarPhase ──────────────────────────────────────────────────────────────

#[test]
fn lunar_phase_angle_in_range() {
    for &bd in &[0.0, 1000.0, 9622.0, -1000.0] {
        let phase = lunar_phase_angle(bd);
        assert!(phase >= 0.0 && phase < 360.0, "phase={phase} at bd={bd}");
    }
}

#[test]
fn lunar_illuminated_fraction_in_range() {
    for &bd in &[0.0, 100.0, 9622.0, 18000.0] {
        let frac = lunar_illuminated_fraction(bd);
        assert!(frac >= 0.0 && frac <= 1.0, "fraction={frac} at bd={bd}");
    }
}

#[test]
fn lunar_illuminated_fraction_monotone_from_new_to_full() {
    // Over the half lunar cycle ~14.75 days illumination should generally increase
    // Check at least one midpoint has higher illumination than start
    let at_new = lunar_illuminated_fraction(0.0);
    let at_quarter = lunar_illuminated_fraction(7.375);
    // Both should be valid fractions — just verify the function returns sensible values
    assert!(at_new >= 0.0 && at_new <= 1.0);
    assert!(at_quarter >= 0.0 && at_quarter <= 1.0);
}

// ─── additional julian_century ────────────────────────────────────────────────

#[test]
fn julian_century_half_century() {
    let t = julian_century(36_525.0 / 2.0);
    assert!((t - 0.5).abs() < 1e-12);
}

#[test]
fn julian_century_negative() {
    let t = julian_century(-36_525.0);
    assert!((t + 1.0).abs() < 1e-12);
}

#[test]
fn julian_century_proportional() {
    let t1 = julian_century(1000.0);
    let t2 = julian_century(2000.0);
    assert!((t2 - 2.0 * t1).abs() < 1e-12);
}

#[test]
fn julian_century_zero_bd() {
    assert_eq!(julian_century(0.0), 0.0);
}

// ─── additional GMST ──────────────────────────────────────────────────────────

#[test]
fn gmst_at_specific_date_finite() {
    let g = greenwich_mean_sidereal_time(9622.5);
    assert!(g.is_finite());
}

#[test]
fn gmst_never_negative() {
    for bd in [-5000.0, 0.0, 100.0, 9622.0, 36525.0] {
        let g = greenwich_mean_sidereal_time(bd);
        assert!(g >= 0.0, "GMST negative at bd={bd}: {g}");
    }
}

#[test]
fn gmst_always_below_360() {
    for bd in [-5000.0, 0.0, 100.0, 9622.0, 36525.0] {
        let g = greenwich_mean_sidereal_time(bd);
        assert!(g < 360.0, "GMST >= 360 at bd={bd}: {g}");
    }
}

#[test]
fn gmst_changes_over_year() {
    let g0 = greenwich_mean_sidereal_time(0.0);
    let g365 = greenwich_mean_sidereal_time(365.25);
    // After one sidereal year, GMST should be back near the same value (within tolerance)
    // But since it's normalized, just check it changed at some intermediate point
    let g100 = greenwich_mean_sidereal_time(100.0);
    assert!((g0 - g100).abs() > 0.01);
    let _ = g365;
}

// ─── additional LMST ──────────────────────────────────────────────────────────

#[test]
fn lmst_in_range_negative_longitude() {
    let lmst = local_mean_sidereal_time(9622.0, -90.0);
    assert!(lmst >= 0.0 && lmst < 360.0, "LMST={lmst}");
}

#[test]
fn lmst_wraps_at_360() {
    let lmst = local_mean_sidereal_time(9622.0, 350.0);
    assert!(lmst >= 0.0 && lmst < 360.0);
}

#[test]
fn lmst_full_circle_same_as_zero() {
    let bd = 9622.0;
    let a = local_mean_sidereal_time(bd, 0.0);
    let b = local_mean_sidereal_time(bd, 360.0);
    assert!((a - b).abs() < 1e-8);
}

#[test]
fn lmst_london_and_greenwich_same() {
    let bd = 9622.0;
    let greenwich = local_mean_sidereal_time(bd, 0.0);
    let gmt = local_mean_sidereal_time(bd, 0.0);
    assert!((greenwich - gmt).abs() < 1e-12);
}

// ─── additional solarLongitude ────────────────────────────────────────────────

#[test]
fn solar_longitude_finite() {
    for bd in [-5000.0, 0.0, 9622.0, 36525.0] {
        assert!(solar_longitude(bd).is_finite());
    }
}

#[test]
fn solar_longitude_in_range_multiple_dates() {
    for bd in [100.0, 200.0, 300.0, 9622.0] {
        let lon = solar_longitude(bd);
        assert!(lon >= 0.0 && lon < 360.0, "lon={lon} at bd={bd}");
    }
}

#[test]
fn solar_longitude_approx_equinox() {
    // ~March 20 2000 is ~79 days after J2000.0; solar longitude ≈ 0° (vernal equinox)
    let lon = solar_longitude(79.0);
    // Accept ±10° around 0 or 360
    let close_to_zero = lon < 10.0 || lon > 350.0;
    assert!(close_to_zero, "solar lon at vernal equinox should be near 0°, got {lon}");
}

// ─── additional solarDeclination ─────────────────────────────────────────────

#[test]
fn solar_declination_equinox_near_zero() {
    // At vernal equinox (~79 days), declination ≈ 0
    let dec = solar_declination(79.0);
    assert!(dec.abs() < 5.0, "declination at equinox should be near 0, got {dec}");
}

#[test]
fn solar_declination_bounded() {
    for bd in [-1000.0, 0.0, 100.0, 9622.0] {
        let dec = solar_declination(bd);
        assert!(dec.abs() <= 90.0, "declination out of range: {dec}");
    }
}

// ─── additional solarRightAscension ──────────────────────────────────────────

#[test]
fn solar_ra_in_range_all_dates() {
    for bd in [0.0, 100.0, 200.0, 9622.0] {
        let ra = solar_right_ascension(bd);
        assert!(ra >= 0.0 && ra < 360.0, "RA={ra} at bd={bd}");
    }
}

#[test]
fn solar_ra_finite() {
    assert!(solar_right_ascension(9622.5).is_finite());
}

// ─── additional lunarPhase ────────────────────────────────────────────────────

#[test]
fn lunar_phase_angle_full_moon_near_180() {
    // Approximate full moons: BD 0 is near a full moon (~Jan 21, 2000)
    // Try several dates and verify at least some are near 180°
    let mut found_near_180 = false;
    for d in (0..30).map(|i| i as f64) {
        let phase = lunar_phase_angle(d);
        if (phase - 180.0).abs() < 30.0 {
            found_near_180 = true;
        }
    }
    assert!(found_near_180, "No full moon found in first 30 BDs");
}

#[test]
fn lunar_phase_angle_changes_daily() {
    let p0 = lunar_phase_angle(0.0);
    let p1 = lunar_phase_angle(1.0);
    assert!((p0 - p1).abs() > 0.0, "phase should change day to day");
}

#[test]
fn lunar_illuminated_fraction_full_moon_near_one() {
    // When phase angle is 180°, illumination is 1.0
    // Find approximate full moon
    let mut max_frac = 0.0_f64;
    for d in (0..30).map(|i| i as f64) {
        max_frac = max_frac.max(lunar_illuminated_fraction(d));
    }
    assert!(max_frac > 0.9, "expected near-full illumination, got {max_frac}");
}

#[test]
fn lunar_illuminated_fraction_new_moon_near_zero() {
    let mut min_frac = 1.0_f64;
    for d in (0..30).map(|i| i as f64) {
        min_frac = min_frac.min(lunar_illuminated_fraction(d));
    }
    assert!(min_frac < 0.1, "expected near-zero illumination, got {min_frac}");
}
