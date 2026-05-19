//! Tests for the BrightSpace geodesy module.

use brightdate::geodesy::*;
use brightdate::spacetime::{BRIGHT_METER_M, SPEED_OF_LIGHT_M_PER_S};

const EPS: f64 = 1e-9;

// ── Reference data ──────────────────────────────────────────────────────────

// NASA GSFC / GODE station, ITRF2020 epoch 2015.0, SOLN 5.
// The ECEF vector is the definitive published quantity; geodetic equivalents
// are derived from it via `ecef_to_geodetic` and checked for round-trip
// stability rather than against an external published lat/lng pair.
fn gode_ecef() -> EcefCoordinate {
    EcefCoordinate::new(1_130_773.595_6, -4_831_253.571_8, 3_994_200.445_3)
}

fn dc() -> GeodeticCoordinate {
    GeodeticCoordinate::surface(38.8951, -77.0364)
}
fn nyc() -> GeodeticCoordinate {
    GeodeticCoordinate::surface(40.7128, -74.006)
}
fn london() -> GeodeticCoordinate {
    GeodeticCoordinate::surface(51.5074, -0.1278)
}
fn sydney() -> GeodeticCoordinate {
    GeodeticCoordinate::surface(-33.8688, 151.2093)
}

const DC_NYC_KM_APPROX: f64 = 328.0;
const LONDON_SYDNEY_KM_APPROX: f64 = 16_993.0;

// ── WGS84 constants ──────────────────────────────────────────────────────────

#[test]
fn wgs84_semi_major_axis_exact() {
    assert_eq!(WGS84_SEMI_MAJOR_AXIS_M, 6_378_137.0);
}

#[test]
fn wgs84_inverse_flattening_full_precision() {
    assert_eq!(WGS84_INVERSE_FLATTENING, 298.257_223_563);
    assert!((WGS84_FLATTENING - 1.0 / 298.257_223_563).abs() < 1e-15);
}

#[test]
fn wgs84_semi_minor_axis_derived_from_a_times_one_minus_f() {
    assert!((WGS84_SEMI_MINOR_AXIS_M - 6_356_752.314_245).abs() < 1e-3);
}

#[test]
fn wgs84_first_eccentricity_squared_derived() {
    assert!((WGS84_FIRST_ECCENTRICITY_SQUARED - 0.006_694_379_990_141_316).abs() < 1e-15);
}

#[test]
fn iugg_mean_earth_radius_distinct_from_wgs84_axis() {
    assert_eq!(EARTH_MEAN_RADIUS_M, 6_371_008.8);
    assert_ne!(EARTH_MEAN_RADIUS_M, WGS84_SEMI_MAJOR_AXIS_M);
}

// ── geodetic_to_ecef ─────────────────────────────────────────────────────────

#[test]
fn equator_zero_lng_lands_on_plus_x_axis() {
    let p = geodetic_to_ecef(GeodeticCoordinate::surface(0.0, 0.0));
    assert!((p.x - WGS84_SEMI_MAJOR_AXIS_M).abs() < 1e-6);
    assert!(p.y.abs() < 1e-6);
    assert!(p.z.abs() < 1e-6);
}

#[test]
fn equator_ninety_east_lands_on_plus_y_axis() {
    let p = geodetic_to_ecef(GeodeticCoordinate::surface(0.0, 90.0));
    assert!(p.x.abs() < 1e-6);
    assert!((p.y - WGS84_SEMI_MAJOR_AXIS_M).abs() < 1e-6);
    assert!(p.z.abs() < 1e-6);
}

#[test]
fn equator_one_eighty_lands_on_minus_x_axis() {
    let p = geodetic_to_ecef(GeodeticCoordinate::surface(0.0, 180.0));
    assert!((p.x + WGS84_SEMI_MAJOR_AXIS_M).abs() < 1e-6);
    assert!(p.y.abs() < 1e-6);
    assert!(p.z.abs() < 1e-6);
}

#[test]
fn north_pole_lands_on_plus_z_at_polar_radius() {
    let p = geodetic_to_ecef(GeodeticCoordinate::surface(90.0, 0.0));
    assert!(p.x.abs() < 1e-4);
    assert!(p.y.abs() < 1e-4);
    assert!((p.z - WGS84_SEMI_MINOR_AXIS_M).abs() < 1e-4);
}

#[test]
fn south_pole_lands_on_minus_z_at_polar_radius() {
    let p = geodetic_to_ecef(GeodeticCoordinate::surface(-90.0, 0.0));
    assert!((p.z + WGS84_SEMI_MINOR_AXIS_M).abs() < 1e-4);
}

#[test]
fn altitude_zero_default_via_surface_constructor() {
    let with_default = geodetic_to_ecef(GeodeticCoordinate::surface(0.0, 0.0));
    let explicit = geodetic_to_ecef(GeodeticCoordinate::new(0.0, 0.0, 0.0));
    assert_eq!(with_default, explicit);
}

#[test]
fn altitude_lifts_along_local_normal() {
    let surface = geodetic_to_ecef(GeodeticCoordinate::surface(0.0, 0.0));
    let aloft = geodetic_to_ecef(GeodeticCoordinate::new(0.0, 0.0, 1000.0));
    assert!((aloft.x - surface.x - 1000.0).abs() < 1e-6);
    assert!(aloft.y.abs() < 1e-6);
    assert!(aloft.z.abs() < 1e-6);
}

// ── ecef_to_geodetic ─────────────────────────────────────────────────────────

#[test]
fn recovers_equator_zero_lng_from_plus_x_axis() {
    let g = ecef_to_geodetic(EcefCoordinate::new(WGS84_SEMI_MAJOR_AXIS_M, 0.0, 0.0));
    assert!(g.latitude.abs() < 1e-9);
    assert!(g.longitude.abs() < 1e-9);
    assert!(g.altitude.abs() < 1e-6);
}

#[test]
fn recovers_north_pole_as_lat_ninety() {
    let g = ecef_to_geodetic(EcefCoordinate::new(0.0, 0.0, WGS84_SEMI_MINOR_AXIS_M));
    assert!((g.latitude - 90.0).abs() < 1e-6);
    assert_eq!(g.longitude, 0.0); // pole guard
    assert!(g.altitude.abs() < 1e-4);
}

#[test]
fn recovers_south_pole_as_lat_minus_ninety() {
    let g = ecef_to_geodetic(EcefCoordinate::new(0.0, 0.0, -WGS84_SEMI_MINOR_AXIS_M));
    assert!((g.latitude + 90.0).abs() < 1e-6);
    assert_eq!(g.longitude, 0.0);
}

#[test]
fn round_trip_at_gode_within_sub_micrometre() {
    let g = ecef_to_geodetic(gode_ecef());
    let p = geodetic_to_ecef(g);
    let target = gode_ecef();
    assert!((p.x - target.x).abs() < 1e-6);
    assert!((p.y - target.y).abs() < 1e-6);
    assert!((p.z - target.z).abs() < 1e-6);
    // Recovered geodetic is in Maryland.
    assert!(g.latitude > 38.0 && g.latitude < 40.0);
    assert!(g.longitude > -78.0 && g.longitude < -76.0);
}

#[test]
fn round_trip_arbitrary_geodetic_inputs_within_tolerance() {
    // Sample a coarse grid of inputs and confirm round-trip stability.
    for &lat in &[-89.9, -45.0, -10.0, 0.0, 10.0, 45.0, 89.9] {
        for &lon in &[-180.0, -90.0, -30.0, 0.0, 30.0, 90.0, 179.9] {
            for &alt in &[-100.0, 0.0, 1000.0, 100_000.0] {
                let coord = GeodeticCoordinate::new(lat, lon, alt);
                let ecef = geodetic_to_ecef(coord);
                let back = ecef_to_geodetic(ecef);
                assert!(
                    (back.latitude - lat).abs() < 1e-5,
                    "lat round-trip {} → {}",
                    lat,
                    back.latitude
                );
                let dlon = ((back.longitude - lon + 540.0) % 360.0) - 180.0;
                assert!(dlon.abs() < 1e-5, "lon round-trip drift = {}", dlon);
                assert!(
                    (back.altitude - alt).abs() < 1e-3,
                    "alt round-trip {} → {}",
                    alt,
                    back.altitude
                );
            }
        }
    }
}

// ── ecef_chord_metres ────────────────────────────────────────────────────────

#[test]
fn chord_zero_for_identical_points() {
    let p = EcefCoordinate::new(1.0, 2.0, 3.0);
    assert_eq!(ecef_chord_metres(p, p), 0.0);
}

#[test]
fn chord_symmetric() {
    let a = EcefCoordinate::new(100.0, 200.0, 300.0);
    let b = EcefCoordinate::new(-50.0, 75.0, 125.0);
    assert_eq!(ecef_chord_metres(a, b), ecef_chord_metres(b, a));
}

#[test]
fn chord_axis_aligned_three_four_five() {
    let a = EcefCoordinate::new(0.0, 0.0, 0.0);
    let b = EcefCoordinate::new(3.0, 4.0, 0.0);
    assert!((ecef_chord_metres(a, b) - 5.0).abs() < EPS);
}

#[test]
fn chord_antipodal_equatorial_equals_two_a() {
    let a = geodetic_to_ecef(GeodeticCoordinate::surface(0.0, 0.0));
    let b = geodetic_to_ecef(GeodeticCoordinate::surface(0.0, 180.0));
    assert!((ecef_chord_metres(a, b) - 2.0 * WGS84_SEMI_MAJOR_AXIS_M).abs() < 1e-4);
}

#[test]
fn chord_satisfies_triangle_inequality() {
    let a = EcefCoordinate::new(1.0, 2.0, 3.0);
    let b = EcefCoordinate::new(4.0, 0.0, -1.0);
    let c = EcefCoordinate::new(-2.0, 5.0, 7.0);
    let ab = ecef_chord_metres(a, b);
    let bc = ecef_chord_metres(b, c);
    let ac = ecef_chord_metres(a, c);
    assert!(ac <= ab + bc + 1e-12);
}

// ── ecef_chord_bright_meters ────────────────────────────────────────────────

#[test]
fn bright_meter_chord_one_at_one_light_second() {
    let a = EcefCoordinate::new(0.0, 0.0, 0.0);
    let b = EcefCoordinate::new(SPEED_OF_LIGHT_M_PER_S, 0.0, 0.0);
    assert!((ecef_chord_bright_meters(a, b) - 1.0).abs() < 1e-12);
}

#[test]
fn bright_meter_chord_consistent_with_metres() {
    let a = geodetic_to_ecef(GeodeticCoordinate::surface(35.0, -100.0));
    let b = geodetic_to_ecef(GeodeticCoordinate::surface(50.0, 20.0));
    let bm = ecef_chord_bright_meters(a, b);
    let m_over_c = ecef_chord_metres(a, b) / BRIGHT_METER_M;
    assert!((bm - m_over_c).abs() < 1e-15);
}

// ── surface_distance_metres ─────────────────────────────────────────────────

#[test]
fn surface_zero_for_identical_points() {
    assert_eq!(surface_distance_metres(dc(), dc()), 0.0);
}

#[test]
fn surface_symmetric() {
    let ab = surface_distance_metres(dc(), nyc());
    let ba = surface_distance_metres(nyc(), dc());
    assert!((ab - ba).abs() < 1e-9);
}

#[test]
fn surface_dc_nyc_about_328_km() {
    let km = surface_distance_metres(dc(), nyc()) / 1000.0;
    assert!(
        (km - DC_NYC_KM_APPROX).abs() < 5.0,
        "DC-NYC = {} km (expected ≈ {})",
        km,
        DC_NYC_KM_APPROX
    );
}

#[test]
fn surface_london_sydney_within_one_percent() {
    let km = surface_distance_metres(london(), sydney()) / 1000.0;
    assert!(
        (km - LONDON_SYDNEY_KM_APPROX).abs() < 100.0,
        "London-Sydney = {} km (expected ≈ {})",
        km,
        LONDON_SYDNEY_KM_APPROX
    );
}

#[test]
fn surface_pi_r_for_equatorial_antipodes() {
    let a = GeodeticCoordinate::surface(0.0, 0.0);
    let b = GeodeticCoordinate::surface(0.0, 180.0);
    let expected = std::f64::consts::PI * EARTH_MEAN_RADIUS_M;
    assert!((surface_distance_metres(a, b) - expected).abs() < 1e-3);
}

#[test]
fn surface_ignores_altitude() {
    let ground = surface_distance_metres(dc(), nyc());
    let aloft = surface_distance_metres(
        GeodeticCoordinate::new(dc().latitude, dc().longitude, 10_000.0),
        GeodeticCoordinate::new(nyc().latitude, nyc().longitude, 10_000.0),
    );
    assert_eq!(aloft, ground);
}

// ── light_travel_time_seconds ───────────────────────────────────────────────

#[test]
fn light_time_equals_chord_over_c() {
    let a = geodetic_to_ecef(GeodeticCoordinate::surface(0.0, 0.0));
    let b = geodetic_to_ecef(GeodeticCoordinate::surface(0.0, 90.0));
    let expected = ecef_chord_metres(a, b) / SPEED_OF_LIGHT_M_PER_S;
    assert!((light_travel_time_seconds(a, b) - expected).abs() < 1e-15);
}

#[test]
fn light_time_zero_for_identical_points() {
    let p = geodetic_to_ecef(GeodeticCoordinate::surface(12.34, 56.78));
    assert_eq!(light_travel_time_seconds(p, p), 0.0);
}

#[test]
fn light_time_equals_chord_bright_meters_numerically() {
    let a = geodetic_to_ecef(GeodeticCoordinate::surface(10.0, 20.0));
    let b = geodetic_to_ecef(GeodeticCoordinate::surface(-30.0, 100.0));
    assert!(
        (light_travel_time_seconds(a, b) - ecef_chord_bright_meters(a, b)).abs() < 1e-15
    );
}

// ── gps_distance ────────────────────────────────────────────────────────────

#[test]
fn gps_distance_coincident_points_zero_everywhere() {
    let d = gps_distance(dc(), dc());
    assert_eq!(d.chord_metres, 0.0);
    assert_eq!(d.surface_metres, 0.0);
    assert_eq!(d.surface_minus_chord_metres, 0.0);
    assert_eq!(d.light_travel_seconds, 0.0);
}

#[test]
fn gps_distance_surface_geq_chord() {
    let d = gps_distance(london(), sydney());
    assert!(d.surface_metres >= d.chord_metres);
    assert!(d.surface_minus_chord_metres >= 0.0);
}

#[test]
fn gps_distance_internal_consistency() {
    let d = gps_distance(dc(), nyc());
    assert!((d.chord_bright_meters - d.chord_metres / BRIGHT_METER_M).abs() < 1e-15);
    assert!(
        (d.light_travel_seconds - d.chord_metres / SPEED_OF_LIGHT_M_PER_S).abs() < 1e-15
    );
    assert!((d.light_travel_seconds - d.chord_bright_meters).abs() < 1e-15);
}

#[test]
fn gps_distance_dc_nyc_one_point_one_ms_light_floor() {
    let d = gps_distance(dc(), nyc());
    let one_way_ms = d.light_travel_seconds * 1000.0;
    assert!(one_way_ms > 1.0 && one_way_ms < 1.2);
}

#[test]
fn gps_distance_antipodal_collapses_to_pi_r_and_two_a() {
    let a = GeodeticCoordinate::surface(0.0, 0.0);
    let b = GeodeticCoordinate::surface(0.0, 180.0);
    let d = gps_distance(a, b);
    assert!((d.chord_metres - 2.0 * WGS84_SEMI_MAJOR_AXIS_M).abs() < 1e-4);
    assert!(
        (d.surface_metres - std::f64::consts::PI * EARTH_MEAN_RADIUS_M).abs() < 1e-3
    );
    let ratio = d.surface_metres / d.chord_metres;
    assert!(
        ratio > 1.55 && ratio < 1.58,
        "antipodal arc/chord ratio = {} (expected ≈ π/2 ≈ 1.5708)",
        ratio
    );
}

// ── BrightSpace vector primitives ───────────────────────────────────────────

#[test]
fn ecef_magnitude_zero_for_zero_vector() {
    assert_eq!(ecef_magnitude(EcefCoordinate::new(0.0, 0.0, 0.0)), 0.0);
}

#[test]
fn ecef_magnitude_three_four_zero_is_five() {
    assert!((ecef_magnitude(EcefCoordinate::new(3.0, 4.0, 0.0)) - 5.0).abs() < 1e-12);
}

#[test]
fn ecef_magnitude_equator_surface_is_equatorial_radius() {
    let p = geodetic_to_ecef(GeodeticCoordinate::surface(0.0, 0.0));
    assert!((ecef_magnitude(p) - WGS84_SEMI_MAJOR_AXIS_M).abs() < 1e-6);
}

#[test]
fn central_angle_zero_for_identical_vectors() {
    let v = EcefCoordinate::new(1.0, 2.0, 3.0);
    assert!(ecef_central_angle(v, v).abs() < 1e-12);
}

#[test]
fn central_angle_pi_for_antipodal_equatorial() {
    let a = geodetic_to_ecef(GeodeticCoordinate::surface(0.0, 0.0));
    let b = geodetic_to_ecef(GeodeticCoordinate::surface(0.0, 180.0));
    assert!((ecef_central_angle(a, b) - std::f64::consts::PI).abs() < 1e-6);
}

#[test]
fn central_angle_pi_over_two_for_orthogonal_equatorial() {
    let a = geodetic_to_ecef(GeodeticCoordinate::surface(0.0, 0.0));
    let b = geodetic_to_ecef(GeodeticCoordinate::surface(0.0, 90.0));
    assert!((ecef_central_angle(a, b) - std::f64::consts::FRAC_PI_2).abs() < 1e-6);
}

#[test]
fn central_angle_pi_over_two_for_equator_to_pole() {
    let a = geodetic_to_ecef(GeodeticCoordinate::surface(0.0, 0.0));
    let b = EcefCoordinate::new(0.0, 0.0, WGS84_SEMI_MINOR_AXIS_M);
    assert!((ecef_central_angle(a, b) - std::f64::consts::FRAC_PI_2).abs() < 1e-6);
}

#[test]
fn central_angle_symmetric() {
    let a = geodetic_to_ecef(GeodeticCoordinate::surface(12.0, 34.0));
    let b = geodetic_to_ecef(GeodeticCoordinate::surface(-56.0, 78.0));
    assert!((ecef_central_angle(a, b) - ecef_central_angle(b, a)).abs() < 1e-12);
}

#[test]
fn central_angle_invariant_under_radial_scaling() {
    let a = geodetic_to_ecef(GeodeticCoordinate::surface(30.0, 40.0));
    let b = geodetic_to_ecef(GeodeticCoordinate::surface(50.0, 60.0));
    let a_scaled = EcefCoordinate::new(a.x * 1000.0, a.y * 1000.0, a.z * 1000.0);
    assert!((ecef_central_angle(a, b) - ecef_central_angle(a_scaled, b)).abs() < 1e-10);
}

#[test]
fn central_angle_nan_at_zero_vector() {
    let zero = EcefCoordinate::new(0.0, 0.0, 0.0);
    let nonzero = EcefCoordinate::new(1.0, 0.0, 0.0);
    assert!(ecef_central_angle(zero, nonzero).is_nan());
    assert!(ecef_central_angle(nonzero, zero).is_nan());
}

#[test]
fn central_angle_stable_for_short_baselines() {
    // Two points 100 m apart on the equator — should produce a tiny but
    // non-zero, well-conditioned angle.
    let a = geodetic_to_ecef(GeodeticCoordinate::surface(0.0, 0.0));
    let lon_offset_deg = (100.0 / WGS84_SEMI_MAJOR_AXIS_M) * (180.0 / std::f64::consts::PI);
    let b = geodetic_to_ecef(GeodeticCoordinate::surface(0.0, lon_offset_deg));
    let angle = ecef_central_angle(a, b);
    assert!(angle > 1e-8 && angle < 1e-4);
}

// ── ecef_arc_metres / at_radius ─────────────────────────────────────────────

#[test]
fn arc_matches_surface_within_one_percent() {
    // surface_distance_metres uses geodetic latitude (haversine on lat/lng);
    // ecef_arc_metres uses geocentric latitude (angle between vectors). They
    // diverge sub-percent due to ellipsoid flattening.
    let from_gps = surface_distance_metres(dc(), nyc());
    let from_ecef = ecef_arc_metres(geodetic_to_ecef(dc()), geodetic_to_ecef(nyc()));
    let ratio = from_ecef / from_gps;
    assert!(ratio > 0.99 && ratio < 1.01, "ratio = {}", ratio);
}

#[test]
fn arc_pi_r_mean_for_antipodal_equatorial() {
    let a = geodetic_to_ecef(GeodeticCoordinate::surface(0.0, 0.0));
    let b = geodetic_to_ecef(GeodeticCoordinate::surface(0.0, 180.0));
    let expected = std::f64::consts::PI * EARTH_MEAN_RADIUS_M;
    assert!((ecef_arc_metres(a, b) - expected).abs() < 1e-3);
}

#[test]
fn arc_zero_for_identical_vectors() {
    let v = geodetic_to_ecef(GeodeticCoordinate::surface(12.0, 34.0));
    assert!(ecef_arc_metres(v, v).abs() < 1e-9);
}

#[test]
fn arc_at_radius_scales_linearly() {
    let a = geodetic_to_ecef(GeodeticCoordinate::surface(0.0, 0.0));
    let b = geodetic_to_ecef(GeodeticCoordinate::surface(0.0, 90.0));
    let r1 = ecef_arc_metres_at_radius(a, b, 1_000_000.0);
    let r2 = ecef_arc_metres_at_radius(a, b, 2_000_000.0);
    assert!((r2 - 2.0 * r1).abs() < 1e-6);
}

#[test]
fn arc_at_mean_earth_radius_equals_default_arc() {
    let a = geodetic_to_ecef(GeodeticCoordinate::surface(10.0, 20.0));
    let b = geodetic_to_ecef(GeodeticCoordinate::surface(-30.0, 100.0));
    let direct = ecef_arc_metres(a, b);
    let via_radius = ecef_arc_metres_at_radius(a, b, EARTH_MEAN_RADIUS_M);
    assert!((direct - via_radius).abs() < 1e-9);
}

// ── bright_space_distance ───────────────────────────────────────────────────

fn opmt_ecef() -> EcefCoordinate {
    EcefCoordinate::new(4_202_777.411_5, 171_368.413_5, 4_778_660.190_3)
}

#[test]
fn bright_space_distance_zero_for_coincident_points() {
    let d = bright_space_distance(gode_ecef(), gode_ecef());
    assert_eq!(d.chord_metres, 0.0);
    assert_eq!(d.chord_bright_meters, 0.0);
    assert!(d.central_angle_radians.abs() < 1e-12);
    assert!(d.arc_mean_earth_radius_metres.abs() < 1e-6);
    assert!(d.arc_average_radius_metres.abs() < 1e-6);
    assert_eq!(d.light_travel_seconds, 0.0);
}

#[test]
fn bright_space_distance_internal_consistency() {
    let d = bright_space_distance(gode_ecef(), opmt_ecef());
    assert!((d.chord_bright_meters - d.chord_metres / BRIGHT_METER_M).abs() < 1e-15);
    assert!(
        (d.light_travel_seconds - d.chord_metres / SPEED_OF_LIGHT_M_PER_S).abs() < 1e-15
    );
    assert!((d.light_travel_seconds - d.chord_bright_meters).abs() < 1e-15);
}

#[test]
fn bright_space_distance_arc_geq_chord() {
    let d = bright_space_distance(gode_ecef(), opmt_ecef());
    assert!(d.arc_mean_earth_radius_metres >= d.chord_metres);
    assert!(d.arc_average_radius_metres >= d.chord_metres);
}

#[test]
fn bright_space_distance_matches_haversine_within_one_percent() {
    let d = bright_space_distance(geodetic_to_ecef(dc()), geodetic_to_ecef(nyc()));
    let from_gps = surface_distance_metres(dc(), nyc());
    let ratio = d.arc_mean_earth_radius_metres / from_gps;
    assert!(ratio > 0.99 && ratio < 1.01, "ratio = {}", ratio);
}

#[test]
fn bright_space_distance_average_radius_arc_larger_above_surface() {
    // Both points at ~400 km altitude (rough ISS scale).
    let lift_a = geodetic_to_ecef(GeodeticCoordinate::new(40.0, -75.0, 400_000.0));
    let lift_b = geodetic_to_ecef(GeodeticCoordinate::new(50.0, 10.0, 400_000.0));
    let d = bright_space_distance(lift_a, lift_b);
    assert!(d.arc_average_radius_metres > d.arc_mean_earth_radius_metres);
}

#[test]
fn bright_space_distance_antipodal_collapses_to_pi_r() {
    let a = geodetic_to_ecef(GeodeticCoordinate::surface(0.0, 0.0));
    let b = geodetic_to_ecef(GeodeticCoordinate::surface(0.0, 180.0));
    let d = bright_space_distance(a, b);
    assert!((d.central_angle_radians - std::f64::consts::PI).abs() < 1e-6);
    assert!((d.chord_metres - 2.0 * WGS84_SEMI_MAJOR_AXIS_M).abs() < 1e-4);
    assert!(
        (d.arc_mean_earth_radius_metres - std::f64::consts::PI * EARTH_MEAN_RADIUS_M)
            .abs()
            < 1e-3
    );
}

#[test]
fn bright_space_distance_symmetric() {
    let ab = bright_space_distance(gode_ecef(), opmt_ecef());
    let ba = bright_space_distance(opmt_ecef(), gode_ecef());
    assert!((ab.chord_metres - ba.chord_metres).abs() < 1e-9);
    assert!((ab.central_angle_radians - ba.central_angle_radians).abs() < 1e-12);
    assert!(
        (ab.arc_mean_earth_radius_metres - ba.arc_mean_earth_radius_metres).abs() < 1e-9
    );
    assert!((ab.arc_average_radius_metres - ba.arc_average_radius_metres).abs() < 1e-9);
}
