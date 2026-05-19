//! Tests for the BrightDate Relativity module.

use brightdate::relativity::*;

const EPS: f64 = 1e-12;

fn ev(t: f64, x: f64, y: f64, z: f64) -> SpacetimeEvent {
    SpacetimeEvent::new(t, x, y, z)
}

// ── interval_squared ─────────────────────────────────────────────────────────

#[test]
fn interval_squared_zero_for_coincident_events() {
    let a = ev(1.0, 2.0, 3.0, 4.0);
    assert_eq!(interval_squared(a, a), 0.0);
}

#[test]
fn interval_squared_negative_for_pure_time_separation() {
    // (Δt = 5, Δx = Δy = Δz = 0): ds² = -25
    let a = ev(0.0, 0.0, 0.0, 0.0);
    let b = ev(5.0, 0.0, 0.0, 0.0);
    assert!((interval_squared(a, b) - -25.0).abs() < EPS);
}

#[test]
fn interval_squared_positive_for_pure_space_separation() {
    let a = ev(0.0, 0.0, 0.0, 0.0);
    let b = ev(0.0, 3.0, 4.0, 0.0);
    assert!((interval_squared(a, b) - 25.0).abs() < EPS);
}

#[test]
fn interval_squared_zero_on_light_cone() {
    let a = ev(0.0, 0.0, 0.0, 0.0);
    let b = ev(5.0, 3.0, 4.0, 0.0); // Δt = 5, |Δx| = 5
    assert!(interval_squared(a, b).abs() < EPS);
}

#[test]
fn interval_squared_symmetric() {
    let a = ev(1.0, 2.0, -3.0, 4.0);
    let b = ev(-2.0, 5.0, 6.0, -7.0);
    assert!((interval_squared(a, b) - interval_squared(b, a)).abs() < EPS);
}

// ── interval_kind ────────────────────────────────────────────────────────────

#[test]
fn classifies_timelike_lightlike_spacelike() {
    let origin = ev(0.0, 0.0, 0.0, 0.0);
    assert_eq!(
        interval_kind(origin, ev(5.0, 0.0, 0.0, 0.0), 0.0),
        IntervalKind::Timelike
    );
    assert_eq!(
        interval_kind(origin, ev(5.0, 5.0, 0.0, 0.0), 0.0),
        IntervalKind::Lightlike
    );
    assert_eq!(
        interval_kind(origin, ev(0.0, 5.0, 0.0, 0.0), 0.0),
        IntervalKind::Spacelike
    );
}

#[test]
fn tolerance_widens_lightlike_classification() {
    let a = ev(0.0, 0.0, 0.0, 0.0);
    let b = ev(5.0, 4.999, 0.0, 0.0); // ds² ≈ -0.01
    assert_eq!(interval_kind(a, b, 0.0), IntervalKind::Timelike);
    assert_eq!(interval_kind(a, b, 0.02), IntervalKind::Lightlike);
}

// ── causally_connected ───────────────────────────────────────────────────────

#[test]
fn causally_connected_handles_each_kind() {
    let origin = ev(0.0, 0.0, 0.0, 0.0);
    assert!(causally_connected(origin, ev(5.0, 1.0, 0.0, 0.0), 0.0));
    assert!(causally_connected(origin, ev(5.0, 5.0, 0.0, 0.0), 0.0));
    assert!(!causally_connected(origin, ev(0.0, 5.0, 0.0, 0.0), 0.0));
}

// ── proper_time_between / proper_distance_between ───────────────────────────

#[test]
fn proper_time_zero_at_origin() {
    let a = ev(0.0, 0.0, 0.0, 0.0);
    assert_eq!(proper_time_between(a, a), 0.0);
}

#[test]
fn proper_time_pure_time_equals_delta_t() {
    let a = ev(0.0, 0.0, 0.0, 0.0);
    let b = ev(7.0, 0.0, 0.0, 0.0);
    assert!((proper_time_between(a, b) - 7.0).abs() < EPS);
}

#[test]
fn proper_time_nan_for_spacelike() {
    let a = ev(0.0, 0.0, 0.0, 0.0);
    let b = ev(0.0, 5.0, 0.0, 0.0);
    assert!(proper_time_between(a, b).is_nan());
}

#[test]
fn proper_distance_pure_space_equals_norm() {
    let a = ev(0.0, 0.0, 0.0, 0.0);
    let b = ev(0.0, 3.0, 4.0, 0.0);
    assert!((proper_distance_between(a, b) - 5.0).abs() < EPS);
}

#[test]
fn proper_distance_nan_for_timelike() {
    let a = ev(0.0, 0.0, 0.0, 0.0);
    let b = ev(5.0, 0.0, 0.0, 0.0);
    assert!(proper_distance_between(a, b).is_nan());
}

// ── Twin paradox: proper_time_along ─────────────────────────────────────────

#[test]
fn twin_paradox_recovers_2t_over_gamma() {
    // Travelling twin out at β = 0.6 for T = 10, instantaneous
    // turnaround, return. Proper time = 2T / γ = 16 yr.
    let beta = 0.6;
    let t_leg = 10.0;
    let tau = proper_time_along(&[
        ev(0.0, 0.0, 0.0, 0.0),
        ev(t_leg, beta * t_leg, 0.0, 0.0),
        ev(2.0 * t_leg, 0.0, 0.0, 0.0),
    ])
    .unwrap();
    assert!((tau - 16.0).abs() < 1e-9, "tau = {} (expected 16)", tau);
}

#[test]
fn proper_time_along_rejects_short_input() {
    assert!(proper_time_along(&[]).is_err());
    assert!(proper_time_along(&[ev(0.0, 0.0, 0.0, 0.0)]).is_err());
}

#[test]
fn proper_time_along_rejects_spacelike_segment() {
    let result = proper_time_along(&[
        ev(0.0, 0.0, 0.0, 0.0),
        ev(0.0, 5.0, 0.0, 0.0), // spacelike
    ]);
    assert!(result.is_err());
}

// ── speed / gamma / rapidity / add_velocities ───────────────────────────────

#[test]
fn speed_three_four_zero_is_five() {
    assert!((speed([3.0, 4.0, 0.0]) - 5.0).abs() < EPS);
}

#[test]
fn gamma_one_at_rest() {
    assert!((gamma_from_speed(0.0) - 1.0).abs() < EPS);
    assert!((gamma([0.0, 0.0, 0.0]) - 1.0).abs() < EPS);
}

#[test]
fn gamma_one_point_two_five_at_beta_zero_six() {
    assert!((gamma_from_speed(0.6) - 1.25).abs() < EPS);
}

#[test]
fn gamma_infinite_at_beta_one() {
    assert!(gamma_from_speed(1.0).is_infinite());
}

#[test]
fn gamma_nan_above_beta_one() {
    assert!(gamma_from_speed(1.5).is_nan());
}

#[test]
fn rapidity_atanh_inverse() {
    let beta = 0.42;
    assert!((rapidity(beta) - beta.atanh()).abs() < EPS);
}

#[test]
fn add_velocities_caps_at_unity() {
    // 0.9 ⊕ 0.9 should remain < 1.
    let composed = add_velocities(0.9, 0.9);
    assert!(composed < 1.0);
    assert!(composed > 0.99);
}

#[test]
fn add_velocities_zero_is_identity() {
    assert!((add_velocities(0.7, 0.0) - 0.7).abs() < EPS);
    assert!((add_velocities(0.0, 0.3) - 0.3).abs() < EPS);
}

#[test]
fn add_velocities_symmetric() {
    let u = 0.42;
    let v = -0.17;
    assert!((add_velocities(u, v) - add_velocities(v, u)).abs() < EPS);
}

// ── boost ────────────────────────────────────────────────────────────────────

#[test]
fn boost_zero_velocity_is_identity() {
    let event = ev(7.0, 1.0, 2.0, 3.0);
    let boosted = boost(event, [0.0, 0.0, 0.0]).unwrap();
    assert!((boosted.t - event.t).abs() < EPS);
    assert!((boosted.x - event.x).abs() < EPS);
    assert!((boosted.y - event.y).abs() < EPS);
    assert!((boosted.z - event.z).abs() < EPS);
}

#[test]
fn boost_preserves_origin() {
    let boosted = boost(ev(0.0, 0.0, 0.0, 0.0), [0.6, 0.0, 0.0]).unwrap();
    assert!(boosted.t.abs() < EPS);
    assert!(boosted.x.abs() < EPS);
    assert!(boosted.y.abs() < EPS);
    assert!(boosted.z.abs() < EPS);
}

#[test]
fn boost_preserves_interval() {
    // Boost by β = (0.4, 0.3, 0.0), check that interval_squared is invariant
    // for a pair of arbitrary events.
    let a = ev(2.0, 1.0, 0.5, -0.7);
    let b = ev(5.0, -2.0, 1.5, 0.3);
    let beta = [0.4, 0.3, 0.0];

    let ds2_before = interval_squared(a, b);
    let a_b = boost(a, beta).unwrap();
    let b_b = boost(b, beta).unwrap();
    let ds2_after = interval_squared(a_b, b_b);

    assert!(
        (ds2_before - ds2_after).abs() < 1e-10,
        "interval not preserved: before = {}, after = {}",
        ds2_before,
        ds2_after
    );
}

#[test]
fn boost_collinear_recovers_one_d_form() {
    // Pure x boost at β = 0.6 of (t=1, x=0): standard formula gives
    // (γ·1, -γβ·1, 0, 0) = (1.25, -0.75, 0, 0).
    let boosted = boost(ev(1.0, 0.0, 0.0, 0.0), [0.6, 0.0, 0.0]).unwrap();
    assert!((boosted.t - 1.25).abs() < EPS);
    assert!((boosted.x - -0.75).abs() < EPS);
    assert!(boosted.y.abs() < EPS);
    assert!(boosted.z.abs() < EPS);
}

#[test]
fn boost_rejects_superluminal_beta() {
    assert!(boost(ev(1.0, 0.0, 0.0, 0.0), [1.0, 0.0, 0.0]).is_err());
    assert!(boost(ev(1.0, 0.0, 0.0, 0.0), [2.0, 0.0, 0.0]).is_err());
}

// ── doppler_factor ──────────────────────────────────────────────────────────

#[test]
fn doppler_unity_at_rest() {
    assert!((doppler_factor(0.0) - 1.0).abs() < EPS);
}

#[test]
fn doppler_redshifts_for_recession() {
    let factor = doppler_factor(0.5);
    assert!(factor < 1.0); // observed frequency lower → red
    assert!(factor > 0.0);
}

#[test]
fn doppler_blueshifts_for_approach() {
    let factor = doppler_factor(-0.5);
    assert!(factor > 1.0); // observed frequency higher → blue
}

#[test]
fn doppler_nan_at_or_above_light_speed() {
    assert!(doppler_factor(1.0).is_nan());
    assert!(doppler_factor(-1.5).is_nan());
}

#[test]
fn doppler_symmetric_about_zero() {
    // f(β) · f(−β) = 1
    let beta = 0.42;
    let product = doppler_factor(beta) * doppler_factor(-beta);
    assert!((product - 1.0).abs() < EPS);
}
