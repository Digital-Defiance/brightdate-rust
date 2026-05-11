use brightdate::interplanetary::*;

const EPS: f64 = 1e-10;

// ── Constants ─────────────────────────────────────────────────────────────────

#[test]
fn light_seconds_per_au_approx_499() {
    assert!((LIGHT_SECONDS_PER_AU - 499.0).abs() < 1.0);
}

#[test]
fn light_days_per_au_derived_from_seconds() {
    assert!((LIGHT_DAYS_PER_AU - LIGHT_SECONDS_PER_AU / 86_400.0).abs() < EPS);
}

#[test]
fn mars_sol_greater_than_one_earth_day() {
    assert!(MARS_SOL_IN_EARTH_DAYS > 1.0);
}

#[test]
fn mars_sol_less_than_1_1_earth_days() {
    assert!(MARS_SOL_IN_EARTH_DAYS < 1.1);
}

#[test]
fn light_days_per_au_positive() {
    assert!(LIGHT_DAYS_PER_AU > 0.0);
}

// ── SOLAR_SYSTEM_BODIES ───────────────────────────────────────────────────────

#[test]
fn solar_system_bodies_count() {
    assert_eq!(SOLAR_SYSTEM_BODIES.len(), 9);
}

#[test]
fn all_bodies_positive_semi_major_axis() {
    for body in SOLAR_SYSTEM_BODIES {
        assert!(body.semi_major_axis_au > 0.0, "body {} semi-major axis <= 0", body.name);
    }
}

#[test]
fn all_bodies_positive_orbital_period() {
    for body in SOLAR_SYSTEM_BODIES {
        assert!(body.orbital_period_days > 0.0, "body {} orbital period <= 0", body.name);
    }
}

#[test]
fn earth_semi_major_axis_is_1au() {
    let earth = find_body("Earth").unwrap();
    assert!((earth.semi_major_axis_au - 1.0).abs() < 0.001);
}

#[test]
fn mars_orbital_period_approx_687_days() {
    let mars = find_body("Mars").unwrap();
    assert!((mars.orbital_period_days - 687.0).abs() < 1.0);
}

#[test]
fn outer_planets_larger_orbit_than_inner() {
    let mars = find_body("Mars").unwrap();
    let jupiter = find_body("Jupiter").unwrap();
    assert!(jupiter.semi_major_axis_au > mars.semi_major_axis_au);
}

// ── find_body ─────────────────────────────────────────────────────────────────

#[test]
fn find_body_exact_case() {
    let body = find_body("Mercury").unwrap();
    assert_eq!(body.name, "Mercury");
}

#[test]
fn find_body_lowercase() {
    let body = find_body("mars").unwrap();
    assert_eq!(body.name, "Mars");
}

#[test]
fn find_body_uppercase() {
    let body = find_body("MARS").unwrap();
    assert_eq!(body.name, "Mars");
}

#[test]
fn find_body_mixed_case() {
    let body = find_body("jUpItEr").unwrap();
    assert_eq!(body.name, "Jupiter");
}

#[test]
fn find_body_pluto_returns_none() {
    assert!(find_body("pluto").is_none());
}

#[test]
fn find_body_empty_returns_none() {
    assert!(find_body("").is_none());
}

#[test]
fn find_body_unknown_returns_none() {
    assert!(find_body("Vulcan").is_none());
}

#[test]
fn find_body_all_nine() {
    for name in ["Mercury", "Venus", "Earth", "Mars", "Jupiter", "Saturn", "Uranus", "Neptune", "Moon"] {
        assert!(find_body(name).is_some(), "could not find {name}");
    }
}

// ── light_travel_time ─────────────────────────────────────────────────────────

#[test]
fn light_travel_time_zero_au() {
    assert_eq!(light_travel_time(0.0), 0.0);
}

#[test]
fn light_travel_time_one_au() {
    assert!((light_travel_time(1.0) - LIGHT_DAYS_PER_AU).abs() < EPS);
}

#[test]
fn light_travel_time_two_au() {
    assert!((light_travel_time(2.0) - 2.0 * LIGHT_DAYS_PER_AU).abs() < EPS);
}

#[test]
fn light_travel_time_proportional() {
    let t1 = light_travel_time(1.0);
    let t5 = light_travel_time(5.0);
    assert!((t5 - 5.0 * t1).abs() < EPS);
}

// ── light_delay_to / round_trip_delay ─────────────────────────────────────────

#[test]
fn light_delay_to_earth_is_one_au_travel() {
    let earth = find_body("Earth").unwrap();
    let delay = light_delay_to(earth);
    assert!((delay - light_travel_time(1.0)).abs() < EPS);
}

#[test]
fn light_delay_to_mars_greater_than_earth() {
    let earth = find_body("Earth").unwrap();
    let mars = find_body("Mars").unwrap();
    assert!(light_delay_to(mars) > light_delay_to(earth));
}

#[test]
fn round_trip_delay_is_twice_one_way() {
    let mars = find_body("Mars").unwrap();
    let one_way = light_delay_to(mars);
    let two_way = round_trip_delay(mars);
    assert!((two_way - 2.0 * one_way).abs() < EPS);
}

#[test]
fn round_trip_delay_positive() {
    let jupiter = find_body("Jupiter").unwrap();
    assert!(round_trip_delay(jupiter) > 0.0);
}

// ── signal_arrival_time / signal_send_time ────────────────────────────────────

#[test]
fn signal_arrival_time_is_later() {
    let mars = find_body("Mars").unwrap();
    let sent = 9622.0;
    let arrived = signal_arrival_time(mars, sent);
    assert!(arrived > sent);
}

#[test]
fn signal_send_time_is_earlier() {
    let mars = find_body("Mars").unwrap();
    let received = 9622.0;
    let sent = signal_send_time(mars, received);
    assert!(sent < received);
}

#[test]
fn signal_arrival_send_inverse() {
    let mars = find_body("Mars").unwrap();
    let original_send = 9622.0;
    let arrival = signal_arrival_time(mars, original_send);
    let recovered_send = signal_send_time(mars, arrival);
    assert!((recovered_send - original_send).abs() < EPS);
}

#[test]
fn signal_send_arrival_inverse() {
    let venus = find_body("Venus").unwrap();
    let original_receive = 5000.0;
    let send = signal_send_time(venus, original_receive);
    let recovered_receive = signal_arrival_time(venus, send);
    assert!((recovered_receive - original_receive).abs() < EPS);
}

#[test]
fn signal_delay_equals_light_delay_to() {
    let mars = find_body("Mars").unwrap();
    let send = 9622.0;
    let delay = signal_arrival_time(mars, send) - send;
    assert!((delay - light_delay_to(mars)).abs() < EPS);
}

// ── earth_days_to_mars_sols / mars_sols_to_earth_days ─────────────────────────

#[test]
fn one_mars_sol_in_earth_days() {
    let earth_days = mars_sols_to_earth_days(1.0);
    assert!((earth_days - MARS_SOL_IN_EARTH_DAYS).abs() < EPS);
}

#[test]
fn one_earth_day_in_mars_sols() {
    let sols = earth_days_to_mars_sols(MARS_SOL_IN_EARTH_DAYS);
    assert!((sols - 1.0).abs() < EPS);
}

#[test]
fn earth_days_to_mars_sols_roundtrip() {
    let days = 100.0_f64;
    let back = mars_sols_to_earth_days(earth_days_to_mars_sols(days));
    assert!((back - days).abs() < EPS);
}

#[test]
fn mars_sols_to_earth_days_roundtrip() {
    let sols = 500.0_f64;
    let back = earth_days_to_mars_sols(mars_sols_to_earth_days(sols));
    assert!((back - sols).abs() < EPS);
}

#[test]
fn mars_sol_greater_than_one_earth_day_conversion() {
    assert!(mars_sols_to_earth_days(1.0) > 1.0);
}

#[test]
fn earth_day_is_less_than_one_mars_sol() {
    assert!(earth_days_to_mars_sols(1.0) < 1.0);
}

// ── to_mars_sol_date / from_mars_sol_date ─────────────────────────────────────

#[test]
fn mars_sol_date_roundtrip_epoch() {
    let bd = 0.0_f64;
    let msd = to_mars_sol_date(bd);
    let back = from_mars_sol_date(msd);
    assert!((back - bd).abs() < 1e-8);
}

#[test]
fn mars_sol_date_roundtrip_positive() {
    let bd = 9622.5;
    let msd = to_mars_sol_date(bd);
    let back = from_mars_sol_date(msd);
    assert!((back - bd).abs() < 1e-8);
}

#[test]
fn mars_sol_date_roundtrip_negative() {
    let bd = -1000.0;
    let msd = to_mars_sol_date(bd);
    let back = from_mars_sol_date(msd);
    assert!((back - bd).abs() < 1e-8);
}

#[test]
fn mars_sol_date_larger_later() {
    let bd1 = 9622.0;
    let bd2 = 9623.0;
    assert!(to_mars_sol_date(bd2) > to_mars_sol_date(bd1));
}

#[test]
fn from_mars_sol_date_inverse_of_to() {
    let msd = 50_000.0;
    let bd = from_mars_sol_date(msd);
    let back = to_mars_sol_date(bd);
    assert!((back - msd).abs() < 1e-8);
}

// ── coordinated_mars_time ─────────────────────────────────────────────────────

#[test]
fn coordinated_mars_time_in_range() {
    let mtc = coordinated_mars_time(9622.5);
    assert!((0.0..1.0).contains(&mtc), "MTC = {mtc}");
}

#[test]
fn coordinated_mars_time_always_in_range() {
    for bd in [-5000.0, 0.0, 100.0, 9622.5, 50000.0] {
        let mtc = coordinated_mars_time(bd);
        assert!((0.0..1.0).contains(&mtc), "MTC out of range for BD={bd}: {mtc}");
    }
}

#[test]
fn coordinated_mars_time_never_one() {
    let mtc = coordinated_mars_time(9622.5);
    assert!(mtc < 1.0);
}

#[test]
fn coordinated_mars_time_epoch() {
    let mtc = coordinated_mars_time(0.0);
    assert!((0.0..1.0).contains(&mtc));
}
