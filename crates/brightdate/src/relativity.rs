//! BrightDate Relativity Module
//!
//! Special-relativistic spacetime operations expressed in **Bright units**
//! (c = 1). Coordinates are:
//!
//! - `t` — time component in Bright-Seconds (numerically equal to SI seconds)
//! - `x`, `y`, `z` — spatial components in BrightMeters (numerically equal
//!   to seconds of light-travel time, or equivalently 299,792,458 m each)
//!
//! Because 1 BrightMeter = 1 Bright-Second, every coordinate carries the
//! same numerical units and the Minkowski metric loses its factor of *c*:
//!
//! ```text
//! ds² = -dt² + dx² + dy² + dz²
//! ```
//!
//! with the relativist convention `(−,+,+,+)`. Velocities are expressed as
//! dimensionless fractions of the speed of light (β = v/c), so |β| < 1 for
//! any massive particle.
//!
//! To convert SI inputs into Bright coordinates, use the helpers in
//! [`crate::spacetime`].

// ─── Types ──────────────────────────────────────────────────────────────────

/// An event in 4-dimensional Minkowski spacetime, expressed in Bright units.
/// All four components share the same numerical scale (seconds ≡ BrightMeters
/// because c = 1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpacetimeEvent {
    /// Time coordinate in Bright-Seconds.
    pub t: f64,
    /// x-coordinate in BrightMeters.
    pub x: f64,
    /// y-coordinate in BrightMeters.
    pub y: f64,
    /// z-coordinate in BrightMeters.
    pub z: f64,
}

impl SpacetimeEvent {
    /// Construct a SpacetimeEvent from raw `(t, x, y, z)` components.
    pub const fn new(t: f64, x: f64, y: f64, z: f64) -> Self {
        Self { t, x, y, z }
    }
}

/// A 3-velocity expressed as a dimensionless fraction of the speed of light
/// (β = v/c). The magnitude must be strictly less than 1 for any frame
/// reachable by a Lorentz boost of a massive observer.
pub type Velocity = [f64; 3];

/// Causal classification of a spacetime interval under the (−,+,+,+)
/// signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntervalKind {
    /// `ds² < 0`: events are causally connected; a massive worldline can
    /// pass through both.
    Timelike,
    /// `ds² = 0`: events are connected only by a light signal.
    Lightlike,
    /// `ds² > 0`: events are causally disconnected (outside each other's
    /// light cones).
    Spacelike,
}

// ─── Interval Operations ────────────────────────────────────────────────────

/// Spacetime interval squared between two events, using the (−,+,+,+)
/// signature: `ds² = -(Δt)² + (Δx)² + (Δy)² + (Δz)²`.
///
/// Negative for timelike, zero for lightlike, positive for spacelike.
#[inline]
pub fn interval_squared(a: SpacetimeEvent, b: SpacetimeEvent) -> f64 {
    let dt = b.t - a.t;
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let dz = b.z - a.z;
    -dt * dt + dx * dx + dy * dy + dz * dz
}

/// Classify the spacetime interval between two events as timelike,
/// lightlike, or spacelike.
///
/// `tolerance` is the absolute value below which `|ds²|` is treated as zero
/// (i.e. lightlike). Pass `0.0` for a strict classification.
pub fn interval_kind(
    a: SpacetimeEvent,
    b: SpacetimeEvent,
    tolerance: f64,
) -> IntervalKind {
    let ds2 = interval_squared(a, b);
    if ds2.abs() <= tolerance {
        IntervalKind::Lightlike
    } else if ds2 < 0.0 {
        IntervalKind::Timelike
    } else {
        IntervalKind::Spacelike
    }
}

/// True iff the two events are causally connected, i.e. one lies on or
/// inside the other's light cone (timelike or lightlike separation).
pub fn causally_connected(
    a: SpacetimeEvent,
    b: SpacetimeEvent,
    tolerance: f64,
) -> bool {
    interval_kind(a, b, tolerance) != IntervalKind::Spacelike
}

/// Proper time elapsed between two timelike-separated events, measured by
/// an inertial observer whose worldline passes through both.
///
/// Returns `f64::NAN` if the interval is spacelike.
pub fn proper_time_between(a: SpacetimeEvent, b: SpacetimeEvent) -> f64 {
    let ds2 = interval_squared(a, b);
    if ds2 > 0.0 {
        f64::NAN
    } else {
        (-ds2).sqrt()
    }
}

/// Proper (rest-frame) distance between two spacelike-separated events.
/// Returns `f64::NAN` if the interval is timelike.
pub fn proper_distance_between(a: SpacetimeEvent, b: SpacetimeEvent) -> f64 {
    let ds2 = interval_squared(a, b);
    if ds2 < 0.0 {
        f64::NAN
    } else {
        ds2.sqrt()
    }
}

/// Proper time along a piecewise-linear (inertial-segment) worldline,
/// summing the proper times of each timelike segment.
///
/// Returns `Err` if the worldline has fewer than two events or any
/// consecutive pair is spacelike-separated.
pub fn proper_time_along(worldline: &[SpacetimeEvent]) -> Result<f64, &'static str> {
    if worldline.len() < 2 {
        return Err("proper_time_along requires at least two events");
    }
    let mut total = 0.0;
    for window in worldline.windows(2) {
        let tau = proper_time_between(window[0], window[1]);
        if tau.is_nan() {
            return Err("Worldline segment is spacelike; not a physical worldline");
        }
        total += tau;
    }
    Ok(total)
}

// ─── Velocity Helpers ───────────────────────────────────────────────────────

/// Magnitude of a 3-velocity β (dimensionless, fraction of c).
#[inline]
pub fn speed(beta: Velocity) -> f64 {
    (beta[0] * beta[0] + beta[1] * beta[1] + beta[2] * beta[2]).sqrt()
}

/// Lorentz factor `γ = 1 / √(1 − β²)`.
///
/// Returns `f64::INFINITY` exactly at `|β| = 1`, `f64::NAN` for `|β| > 1`.
pub fn gamma_from_speed(beta: f64) -> f64 {
    let b = beta.abs();
    if b > 1.0 {
        f64::NAN
    } else if b == 1.0 {
        f64::INFINITY
    } else {
        1.0 / (1.0 - b * b).sqrt()
    }
}

/// Lorentz factor for a 3-velocity. Equivalent to
/// [`gamma_from_speed`]`(speed(beta))`.
#[inline]
pub fn gamma(beta: Velocity) -> f64 {
    gamma_from_speed(speed(beta))
}

/// Rapidity associated with a (1D) velocity β: `φ = atanh(β)`.
///
/// Rapidities are additive under collinear Lorentz boosts, unlike
/// velocities.
#[inline]
pub fn rapidity(beta: f64) -> f64 {
    beta.atanh()
}

/// Relativistic addition of two collinear velocities:
/// `u ⊕ v = (u + v) / (1 + uv)` (with c = 1). Closed on `(−1, 1)`.
#[inline]
pub fn add_velocities(u: f64, v: f64) -> f64 {
    (u + v) / (1.0 + u * v)
}

// ─── Lorentz Boost ──────────────────────────────────────────────────────────

/// Active Lorentz boost of a [`SpacetimeEvent`] by 3-velocity β.
///
/// Implements the general (non-collinear) boost:
///
/// ```text
/// t' = γ (t − β·x)
/// x' = x + [(γ − 1) (β·x) / |β|² − γ t] β
/// ```
///
/// Returns `Err` if `|β| ≥ 1`.
pub fn boost(event: SpacetimeEvent, beta: Velocity) -> Result<SpacetimeEvent, &'static str> {
    let [bx, by, bz] = beta;
    let b2 = bx * bx + by * by + bz * bz;
    if b2 >= 1.0 {
        return Err("Lorentz boost requires |β| < 1");
    }
    if b2 == 0.0 {
        return Ok(event);
    }

    let g = 1.0 / (1.0 - b2).sqrt();
    let b_dot_x = bx * event.x + by * event.y + bz * event.z;
    let k = (g - 1.0) / b2;
    let coef = k * b_dot_x - g * event.t;

    Ok(SpacetimeEvent {
        t: g * (event.t - b_dot_x),
        x: event.x + coef * bx,
        y: event.y + coef * by,
        z: event.z + coef * bz,
    })
}

/// Relativistic Doppler factor for a source moving with radial velocity β
/// along the line of sight (β > 0 = recession):
/// `f_obs / f_emit = √((1 − β) / (1 + β))`.
///
/// Returns `f64::NAN` for `|β| ≥ 1`.
pub fn doppler_factor(beta: f64) -> f64 {
    if beta.abs() >= 1.0 {
        f64::NAN
    } else {
        ((1.0 - beta) / (1.0 + beta)).sqrt()
    }
}
