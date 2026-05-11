//! BrightDate Interplanetary Utilities
//!
//! Functions for working with time across the solar system.
//! Demonstrates BrightDate's natural fit for space applications where
//! timezone-free, continuous time is essential.

use crate::types::BrightDateValue;

// ─── Constants ────────────────────────────────────────────────────────────────

/// Speed-of-light delay: seconds per Astronomical Unit.
pub const LIGHT_SECONDS_PER_AU: f64 = 499.004_784;

/// Earth days per AU of light travel.
pub const LIGHT_DAYS_PER_AU: f64 = LIGHT_SECONDS_PER_AU / 86_400.0;

/// Duration of one Martian solar day in Earth decimal days.
pub const MARS_SOL_IN_EARTH_DAYS: f64 = 1.027_491_25;

/// Julian Date reference used in the Mars Sol Date formula.
const MSD_JD_REF: f64 = 2_405_522.002_877_9;

/// J2000.0 Julian Date (= BrightDate 0.0).
const J2000_JD: f64 = 2_451_545.0;

// ─── Solar system body data ───────────────────────────────────────────────────

/// Basic orbital data for a solar system body.
#[derive(Debug, Clone, Copy)]
pub struct SolarSystemBody {
    /// Human-readable name.
    pub name: &'static str,
    /// Semi-major axis in AU.
    pub semi_major_axis_au: f64,
    /// Orbital period in Earth days.
    pub orbital_period_days: f64,
}

/// Known solar system bodies with their approximate orbital parameters.
pub const SOLAR_SYSTEM_BODIES: &[SolarSystemBody] = &[
    SolarSystemBody { name: "Mercury", semi_major_axis_au: 0.387,    orbital_period_days: 87.97     },
    SolarSystemBody { name: "Venus",   semi_major_axis_au: 0.723,    orbital_period_days: 224.7     },
    SolarSystemBody { name: "Earth",   semi_major_axis_au: 1.0,      orbital_period_days: 365.25    },
    SolarSystemBody { name: "Mars",    semi_major_axis_au: 1.524,    orbital_period_days: 687.0     },
    SolarSystemBody { name: "Jupiter", semi_major_axis_au: 5.203,    orbital_period_days: 4_332.59  },
    SolarSystemBody { name: "Saturn",  semi_major_axis_au: 9.537,    orbital_period_days: 10_759.22 },
    SolarSystemBody { name: "Uranus",  semi_major_axis_au: 19.191,   orbital_period_days: 30_688.5  },
    SolarSystemBody { name: "Neptune", semi_major_axis_au: 30.069,   orbital_period_days: 60_182.0  },
    SolarSystemBody { name: "Moon",    semi_major_axis_au: 0.002_57, orbital_period_days: 27.32     },
];

/// Look up a body by case-insensitive name.
pub fn find_body(name: &str) -> Option<&'static SolarSystemBody> {
    let lower = name.to_lowercase();
    SOLAR_SYSTEM_BODIES
        .iter()
        .find(|b| b.name.to_lowercase() == lower)
}

// ─── Light-travel functions ───────────────────────────────────────────────────

/// One-way light-travel time in decimal days for a given distance.
pub fn light_travel_time(distance_au: f64) -> f64 {
    distance_au * LIGHT_DAYS_PER_AU
}

/// One-way light delay to a named solar system body (using semi-major axis).
pub fn light_delay_to(body: &SolarSystemBody) -> f64 {
    light_travel_time(body.semi_major_axis_au)
}

/// Round-trip communication delay (two-way) to a body.
pub fn round_trip_delay(body: &SolarSystemBody) -> f64 {
    light_delay_to(body) * 2.0
}

/// BrightDate when a signal sent at `send_time` arrives at `body`.
pub fn signal_arrival_time(body: &SolarSystemBody, send_time: BrightDateValue) -> BrightDateValue {
    send_time + light_delay_to(body)
}

/// BrightDate when a signal received at `receive_time` was sent from `body`.
pub fn signal_send_time(body: &SolarSystemBody, receive_time: BrightDateValue) -> BrightDateValue {
    receive_time - light_delay_to(body)
}

// ─── Mars time ────────────────────────────────────────────────────────────────

/// Convert an Earth-day duration to Mars sols.
pub fn earth_days_to_mars_sols(earth_days: f64) -> f64 {
    earth_days / MARS_SOL_IN_EARTH_DAYS
}

/// Convert a Mars sol duration to Earth decimal days.
pub fn mars_sols_to_earth_days(sols: f64) -> f64 {
    sols * MARS_SOL_IN_EARTH_DAYS
}

/// Calculate the Mars Sol Date (MSD) for a BrightDate.
///
/// MSD is a continuous count of Martian solar days since a reference epoch
/// near December 29, 1873.
pub fn to_mars_sol_date(bright_date: BrightDateValue) -> f64 {
    let jd = bright_date + J2000_JD;
    (jd - MSD_JD_REF) / MARS_SOL_IN_EARTH_DAYS
}

/// Convert a Mars Sol Date back to a BrightDate.
pub fn from_mars_sol_date(msd: f64) -> BrightDateValue {
    let jd = msd * MARS_SOL_IN_EARTH_DAYS + MSD_JD_REF;
    jd - J2000_JD
}

/// Coordinated Mars Time (MTC) as a fractional sol in `[0, 1)`.
///
/// MTC is the Mars equivalent of UTC — mean solar time at the Mars prime
/// meridian.
pub fn coordinated_mars_time(bright_date: BrightDateValue) -> f64 {
    let msd = to_mars_sol_date(bright_date);
    ((msd % 1.0) + 1.0) % 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_travel_earth_au() {
        // Light from Earth (1 AU) takes ~499 s ≈ 0.00578 days
        let t = light_travel_time(1.0);
        assert!((t - LIGHT_SECONDS_PER_AU / 86_400.0).abs() < 1e-10);
    }

    #[test]
    fn mars_sol_roundtrip() {
        let bd = 9000.0_f64;
        let msd = to_mars_sol_date(bd);
        let back = from_mars_sol_date(msd);
        assert!((back - bd).abs() < 1e-8);
    }

    #[test]
    fn mtc_in_range() {
        let mtc = coordinated_mars_time(9622.5);
        assert!((0.0..1.0).contains(&mtc));
    }

    #[test]
    fn find_body_case_insensitive() {
        let body = find_body("mars").unwrap();
        assert_eq!(body.name, "Mars");
    }

    #[test]
    fn earth_mars_sols_roundtrip() {
        let d = 100.0_f64;
        assert!((mars_sols_to_earth_days(earth_days_to_mars_sols(d)) - d).abs() < 1e-10);
    }
}
