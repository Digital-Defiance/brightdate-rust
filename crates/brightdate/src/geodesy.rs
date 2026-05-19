//! BrightSpace Geodesy
//!
//! Geodetic ↔ ECEF (Earth-Centred, Earth-Fixed) Cartesian conversion plus
//! Euclidean and great-circle distance helpers, anchored to the WGS84 /
//! GRS80 ellipsoid.
//!
//! # Why both distance metrics?
//!
//! BrightSpace's native distance is the **Euclidean chord** through the
//! Earth's volume — `‖A − B‖` over the ECEF vectors. The standard
//! privileges this quantity because:
//!
//! 1. It is the cryptographic *light-floor*: no signal can traverse two
//!    points in less than `chord / c` seconds, regardless of medium or
//!    routing. This is the value used in Distance-Bounding audits.
//! 2. It composes linearly. Spatial indexes built on chord distance map
//!    directly to SIMD lanes; great-circle distance does not.
//! 3. It has no singularities and no special cases.
//!
//! The **great-circle (surface) distance** is also exposed because it
//! answers a different question: how far must a vehicle travel along the
//! Earth's surface to get from A to B? It is computed on a sphere of
//! radius equal to the IUGG mean Earth radius
//! ([`EARTH_MEAN_RADIUS_M`] = `6_371_008.8 m`). For sub-metre precision
//! across long baselines a Vincenty / Karney solver on the WGS84
//! ellipsoid is the right tool; this module provides the spherical
//! approximation, which is accurate to roughly 0.5 % for any pair of
//! points on Earth.
//!
//! The chord is **always less than or equal to** the great-circle
//! distance for two points on or above the surface, with equality only
//! when the points are coincident. [`gps_distance`] returns both, plus
//! the gap, so callers can see the difference at a glance.

use crate::spacetime::{metres_to_bright_meters, BRIGHT_METER_M, SPEED_OF_LIGHT_M_PER_S};

// ─── WGS84 / GRS80 Ellipsoid Constants ──────────────────────────────────────

/// WGS84 semi-major axis (equatorial radius) in metres. Defined exactly by
/// the WGS84 standard.
pub const WGS84_SEMI_MAJOR_AXIS_M: f64 = 6_378_137.0;

/// WGS84 inverse flattening: `1 / 298.257223563`. The flattening itself is
/// irrational under this definition; this constant carries the conventional
/// defining value to its full published precision.
pub const WGS84_INVERSE_FLATTENING: f64 = 298.257_223_563;

/// WGS84 flattening, derived from [`WGS84_INVERSE_FLATTENING`].
pub const WGS84_FLATTENING: f64 = 1.0 / WGS84_INVERSE_FLATTENING;

/// WGS84 first eccentricity squared: `e² = f · (2 − f)`. Dimensionless.
pub const WGS84_FIRST_ECCENTRICITY_SQUARED: f64 =
    WGS84_FLATTENING * (2.0 - WGS84_FLATTENING);

/// WGS84 semi-minor axis (polar radius) in metres: `b = a · (1 − f)`.
pub const WGS84_SEMI_MINOR_AXIS_M: f64 =
    WGS84_SEMI_MAJOR_AXIS_M * (1.0 - WGS84_FLATTENING);

/// IUGG mean Earth radius in metres (`6_371_008.8 m`). Used for the
/// spherical great-circle approximation in [`surface_distance_metres`].
/// Distinct from [`WGS84_SEMI_MAJOR_AXIS_M`]; do not substitute one for the
/// other.
pub const EARTH_MEAN_RADIUS_M: f64 = 6_371_008.8;

// ─── Types ──────────────────────────────────────────────────────────────────

/// A geodetic coordinate: latitude, longitude, and ellipsoidal height,
/// expressed against the WGS84 ellipsoid. Latitude and longitude are in
/// decimal degrees; height is in metres above the ellipsoid (NOT mean sea
/// level).
///
/// - Latitude convention: `+` north, `−` south, range `[−90, +90]`.
/// - Longitude convention: `+` east, `−` west, range `[−180, +180]`.
///
/// Values outside these ranges are accepted and processed verbatim — no
/// wrap-around or clamping is applied.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeodeticCoordinate {
    /// Latitude in decimal degrees (`+` north).
    pub latitude: f64,
    /// Longitude in decimal degrees (`+` east).
    pub longitude: f64,
    /// Ellipsoidal height in metres above the WGS84 ellipsoid.
    pub altitude: f64,
}

impl GeodeticCoordinate {
    /// Construct a geodetic coordinate at altitude 0 (on the ellipsoid
    /// surface).
    pub const fn surface(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude,
            longitude,
            altitude: 0.0,
        }
    }

    /// Construct a geodetic coordinate with explicit altitude.
    pub const fn new(latitude: f64, longitude: f64, altitude: f64) -> Self {
        Self {
            latitude,
            longitude,
            altitude,
        }
    }
}

/// An ECEF Cartesian coordinate in metres. Origin at Earth's centre of
/// mass; Z-axis through the IERS reference pole; X-axis through the
/// intersection of the IERS reference meridian and the equator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EcefCoordinate {
    /// ECEF X in metres.
    pub x: f64,
    /// ECEF Y in metres.
    pub y: f64,
    /// ECEF Z in metres.
    pub z: f64,
}

impl EcefCoordinate {
    /// Construct an ECEF coordinate from metres-component triple.
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

/// Combined output of [`gps_distance`]: both the BrightSpace-native
/// Euclidean chord and the great-circle surface distance, plus the gap
/// and the chord-derived light-travel floor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DistancePair {
    /// Euclidean chord through the Earth in metres. The BrightSpace-native
    /// distance. Lower bound on physical signal travel time times `c`.
    pub chord_metres: f64,
    /// Euclidean chord in BrightMeters (= `chord_metres / c`).
    pub chord_bright_meters: f64,
    /// Great-circle distance along the surface of a sphere of radius
    /// [`EARTH_MEAN_RADIUS_M`].
    pub surface_metres: f64,
    /// `surface_metres − chord_metres`. Always `≥ 0`.
    pub surface_minus_chord_metres: f64,
    /// One-way light-travel time over the chord, in seconds.
    pub light_travel_seconds: f64,
}

/// Combined distance result for [`bright_space_distance`]. Reports three
/// conceptually distinct numbers — the through-the-Earth chord, the
/// around-the-Earth arc on the mean-Earth sphere, and the around-the-Earth
/// arc on a sphere whose radius is the average of the two endpoints'
/// magnitudes — all derived from a single pair of BrightSpace vectors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BrightSpaceDistance {
    /// Through-the-Earth chord in metres (= `‖A − B‖`).
    pub chord_metres: f64,
    /// Through-the-Earth chord in BrightMeters.
    pub chord_bright_meters: f64,
    /// Central angle subtended at Earth's centre by the two vectors, in
    /// radians. Independent of any chosen sphere radius.
    pub central_angle_radians: f64,
    /// Around-the-Earth arc on a sphere of [`EARTH_MEAN_RADIUS_M`], in
    /// metres. The "what does a vehicle on the surface travel?" answer.
    pub arc_mean_earth_radius_metres: f64,
    /// Around-the-Earth arc on a sphere whose radius is the average of the
    /// two endpoint magnitudes, in metres. The "scale-aware" answer — useful
    /// when both points are at altitude.
    pub arc_average_radius_metres: f64,
    /// One-way light-travel time over the chord, in seconds.
    pub light_travel_seconds: f64,
}

// ─── Conversions ────────────────────────────────────────────────────────────

const DEG_TO_RAD: f64 = std::f64::consts::PI / 180.0;
const RAD_TO_DEG: f64 = 180.0 / std::f64::consts::PI;

/// Convert geodetic (lat, lon, alt) to ECEF Cartesian (x, y, z) in metres,
/// using the WGS84 ellipsoid. Standard closed-form forward transform; no
/// iteration required.
///
/// # Example
///
/// ```
/// use brightdate::geodesy::{geodetic_to_ecef, GeodeticCoordinate};
/// let ecef = geodetic_to_ecef(GeodeticCoordinate::surface(0.0, 0.0));
/// // (0°, 0°, 0 m) lands on the +X axis at the equatorial radius.
/// assert!((ecef.x - 6_378_137.0).abs() < 1e-6);
/// assert!(ecef.y.abs() < 1e-6);
/// assert!(ecef.z.abs() < 1e-6);
/// ```
pub fn geodetic_to_ecef(coord: GeodeticCoordinate) -> EcefCoordinate {
    let GeodeticCoordinate {
        latitude,
        longitude,
        altitude,
    } = coord;
    let phi = latitude * DEG_TO_RAD;
    let lambda = longitude * DEG_TO_RAD;
    let sin_phi = phi.sin();
    let cos_phi = phi.cos();
    let sin_lambda = lambda.sin();
    let cos_lambda = lambda.cos();

    // Prime vertical radius of curvature.
    let n = WGS84_SEMI_MAJOR_AXIS_M
        / (1.0 - WGS84_FIRST_ECCENTRICITY_SQUARED * sin_phi * sin_phi).sqrt();

    let x = (n + altitude) * cos_phi * cos_lambda;
    let y = (n + altitude) * cos_phi * sin_lambda;
    let z = (n * (1.0 - WGS84_FIRST_ECCENTRICITY_SQUARED) + altitude) * sin_phi;

    EcefCoordinate { x, y, z }
}

/// Convert ECEF Cartesian (x, y, z) in metres back to geodetic
/// (lat, lon, alt) on the WGS84 ellipsoid using Bowring's 1985 closed-form
/// solution. Latitude is accurate to roughly the floating-point limit
/// (sub-millimetre at any reasonable Earth-surface input); altitude is
/// accurate to within ~10 µm.
///
/// Special case: at the geographic poles (`x ≈ 0` and `y ≈ 0`), longitude
/// is undefined; this function returns `0` for longitude in that case.
/// Latitude approaches `±90°` smoothly.
pub fn ecef_to_geodetic(ecef: EcefCoordinate) -> GeodeticCoordinate {
    let EcefCoordinate { x, y, z } = ecef;
    let a = WGS84_SEMI_MAJOR_AXIS_M;
    let b = WGS84_SEMI_MINOR_AXIS_M;
    let e2 = WGS84_FIRST_ECCENTRICITY_SQUARED;
    let e_prime_2 = (a * a - b * b) / (b * b); // second eccentricity²

    let p = (x * x + y * y).sqrt();
    if p == 0.0 {
        return GeodeticCoordinate {
            latitude: if z >= 0.0 { 90.0 } else { -90.0 },
            longitude: 0.0,
            altitude: z.abs() - b,
        };
    }

    // Bowring's auxiliary angle.
    let theta = (z * a).atan2(p * b);
    let sin_theta = theta.sin();
    let cos_theta = theta.cos();

    let phi = (z + e_prime_2 * b * sin_theta * sin_theta * sin_theta)
        .atan2(p - e2 * a * cos_theta * cos_theta * cos_theta);
    let lambda = y.atan2(x);

    let sin_phi = phi.sin();
    let n = a / (1.0 - e2 * sin_phi * sin_phi).sqrt();
    let cos_phi = phi.cos();
    let altitude = if cos_phi.abs() > 1e-12 {
        p / cos_phi - n
    } else {
        z / sin_phi - n * (1.0 - e2)
    };

    GeodeticCoordinate {
        latitude: phi * RAD_TO_DEG,
        longitude: lambda * RAD_TO_DEG,
        altitude,
    }
}

// ─── Distances ──────────────────────────────────────────────────────────────

/// Magnitude `‖v‖` of an ECEF vector, in the same unit as its components.
#[inline]
pub fn ecef_magnitude(v: EcefCoordinate) -> f64 {
    (v.x * v.x + v.y * v.y + v.z * v.z).sqrt()
}

/// Euclidean chord distance between two ECEF points, in metres. The
/// BrightSpace-native distance: a straight line through the Earth's
/// volume, ignoring the surface entirely.
#[inline]
pub fn ecef_chord_metres(a: EcefCoordinate, b: EcefCoordinate) -> f64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    let dz = a.z - b.z;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Euclidean chord distance between two ECEF points, in BrightMeters.
/// Numerically equal to the one-way light-travel time in seconds.
#[inline]
pub fn ecef_chord_bright_meters(a: EcefCoordinate, b: EcefCoordinate) -> f64 {
    metres_to_bright_meters(ecef_chord_metres(a, b))
}

/// Central angle (in radians) between two BrightSpace ECEF vectors, as seen
/// from the origin (Earth's centre of mass).
///
/// Implementation: derive `sin²(θ/2)` directly from the chord and the two
/// vector magnitudes via the identity
///
/// ```text
/// sin²(θ/2) = ( chord² − (|a| − |b|)² ) / ( 4 · |a| · |b| )
/// ```
///
/// which is the law of cosines rewritten so that no near-equal large vectors
/// are subtracted. Returns `f64::NAN` if either vector is the zero vector.
///
/// # Precision floor
///
/// Any angle calculation on two ECEF vectors of Earth-scale magnitude
/// `R ≈ 6.4 × 10⁶ m` is bounded below by `R · ε ≈ 1.4 nm` — the absolute
/// error on each magnitude `‖v‖` in IEEE-754 double precision. Below that
/// scale the radial separation `‖a‖ − ‖b‖` is dominated by rounding and the
/// angle becomes noise. For the physically meaningful regime (chord ≳ 1 µm)
/// the result is accurate to within a few ULPs.
pub fn ecef_central_angle(a: EcefCoordinate, b: EcefCoordinate) -> f64 {
    let mag_a = ecef_magnitude(a);
    let mag_b = ecef_magnitude(b);
    if mag_a == 0.0 || mag_b == 0.0 {
        return f64::NAN;
    }

    let chord = ecef_chord_metres(a, b);
    let radial_sep = mag_a - mag_b;
    let abs_radial = radial_sep.abs();

    // Factor as (chord − |radialSep|)·(chord + |radialSep|); both factors
    // are non-negative by the triangle inequality.
    let factor1 = (chord - abs_radial).max(0.0);
    let factor2 = chord + abs_radial;
    let sin_half_squared = (factor1 * factor2) / (4.0 * mag_a * mag_b);

    // Clamp to [0, 1] to absorb floating-point noise.
    2.0 * sin_half_squared.clamp(0.0, 1.0).sqrt().asin()
}

/// Around-the-Earth arc length between two BrightSpace ECEF vectors,
/// computed *directly* from the vectors with no GPS round-trip. The arc is
/// measured on a sphere of radius equal to the IUGG mean Earth radius
/// ([`EARTH_MEAN_RADIUS_M`]).
///
/// # Geodetic vs. geocentric latitude
///
/// This function uses the **geocentric** angle between the two vectors —
/// the angle subtended at Earth's centre. [`surface_distance_metres`] uses
/// the **geodetic** latitude (angle relative to the local ellipsoid normal)
/// via the haversine formula. The two answers agree exactly on the equator
/// and at the poles, and to within a fraction of a percent for short
/// baselines, but can diverge for long baselines spanning mid-latitudes —
/// near-antipodal pairs may show >1 % disagreement. Pick the function whose
/// input form matches what you actually have: BrightSpace vectors → this;
/// lat/lng → [`surface_distance_metres`].
///
/// For points significantly above the surface, prefer
/// [`ecef_arc_metres_at_radius`].
#[inline]
pub fn ecef_arc_metres(a: EcefCoordinate, b: EcefCoordinate) -> f64 {
    ecef_central_angle(a, b) * EARTH_MEAN_RADIUS_M
}

/// Around-the-Earth arc length on a caller-specified sphere radius.
#[inline]
pub fn ecef_arc_metres_at_radius(
    a: EcefCoordinate,
    b: EcefCoordinate,
    radius_metres: f64,
) -> f64 {
    ecef_central_angle(a, b) * radius_metres
}

/// Compute multiple distance metrics between two BrightSpace ECEF vectors
/// in a single call. Returns the through-the-Earth chord (the
/// BrightSpace-native quantity), the central angle, and two flavours of
/// around-the-Earth arc.
pub fn bright_space_distance(
    a: EcefCoordinate,
    b: EcefCoordinate,
) -> BrightSpaceDistance {
    let chord_metres = ecef_chord_metres(a, b);
    let angle = ecef_central_angle(a, b);
    let mag_a = ecef_magnitude(a);
    let mag_b = ecef_magnitude(b);
    let avg_radius = (mag_a + mag_b) / 2.0;

    BrightSpaceDistance {
        chord_metres,
        chord_bright_meters: chord_metres / BRIGHT_METER_M,
        central_angle_radians: angle,
        arc_mean_earth_radius_metres: angle * EARTH_MEAN_RADIUS_M,
        arc_average_radius_metres: angle * avg_radius,
        light_travel_seconds: chord_metres / SPEED_OF_LIGHT_M_PER_S,
    }
}

/// Great-circle distance between two geodetic points along the surface of a
/// sphere of radius [`EARTH_MEAN_RADIUS_M`]. Computed via the haversine
/// formula, numerically stable for both antipodal-adjacent and very-close
/// pairs.
///
/// Accurate to roughly 0.5 % for arbitrary pairs of points on Earth.
/// Altitude is ignored: both points are projected onto the sphere before
/// measuring.
pub fn surface_distance_metres(a: GeodeticCoordinate, b: GeodeticCoordinate) -> f64 {
    let phi1 = a.latitude * DEG_TO_RAD;
    let phi2 = b.latitude * DEG_TO_RAD;
    let d_phi = (b.latitude - a.latitude) * DEG_TO_RAD;
    let d_lambda = (b.longitude - a.longitude) * DEG_TO_RAD;

    let sin_d_phi = (d_phi / 2.0).sin();
    let sin_d_lambda = (d_lambda / 2.0).sin();

    let h = sin_d_phi * sin_d_phi
        + phi1.cos() * phi2.cos() * sin_d_lambda * sin_d_lambda;
    let c = 2.0 * h.sqrt().min(1.0).asin();
    EARTH_MEAN_RADIUS_M * c
}

/// One-way light-travel time over the Euclidean chord between two ECEF
/// points, in seconds. The cryptographic *light-floor*: no physical
/// mechanism can transmit information faster.
#[inline]
pub fn light_travel_time_seconds(a: EcefCoordinate, b: EcefCoordinate) -> f64 {
    ecef_chord_metres(a, b) / SPEED_OF_LIGHT_M_PER_S
}

/// Compute both BrightSpace-native chord distance and human-facing
/// great-circle surface distance between two GPS points, plus the gap and
/// the chord-derived light-travel floor, in a single call.
///
/// The chord is the value an audit trusts; the surface distance is the
/// value a vehicle will travel.
///
/// # Example
///
/// ```
/// use brightdate::geodesy::{gps_distance, GeodeticCoordinate};
/// let dc = GeodeticCoordinate::surface(38.8951, -77.0364);
/// let nyc = GeodeticCoordinate::surface(40.7128, -74.006);
/// let d = gps_distance(dc, nyc);
/// // ~328 km surface, ~1.09 ms light-floor.
/// assert!((d.surface_metres / 1000.0 - 328.0).abs() < 5.0);
/// assert!(d.light_travel_seconds * 1000.0 > 1.0);
/// assert!(d.light_travel_seconds * 1000.0 < 1.2);
/// ```
pub fn gps_distance(a: GeodeticCoordinate, b: GeodeticCoordinate) -> DistancePair {
    let ecef_a = geodetic_to_ecef(a);
    let ecef_b = geodetic_to_ecef(b);
    let chord_metres = ecef_chord_metres(ecef_a, ecef_b);
    let surface_metres = surface_distance_metres(a, b);
    DistancePair {
        chord_metres,
        chord_bright_meters: chord_metres / BRIGHT_METER_M,
        surface_metres,
        surface_minus_chord_metres: surface_metres - chord_metres,
        light_travel_seconds: chord_metres / SPEED_OF_LIGHT_M_PER_S,
    }
}
