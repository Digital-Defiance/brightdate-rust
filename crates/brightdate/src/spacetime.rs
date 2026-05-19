//! BrightDate Spacetime Standard
//!
//! The Bright Spacetime Standard collapses the distinction between space and
//! time by setting the speed of light *c* equal to **1** (natural units). In
//! this system, a duration *is* a distance: the metres light travels in that
//! duration.
//!
//! Three parallel scalar hierarchies are defined:
//!
//! 1. **Bright-Seconds (bs)** — the canonical light-travel unit.
//!    `1 bs = c × 1 s = 299_792_458 m` exactly.
//! 2. **BrightMeters (bm)** — an alias of the Bright-Second emphasising the
//!    spatial interpretation. `1 bm = 1 bs`, with SI-style decimal sub-prefixes
//!    (μbm, mbm, bm, Mbm, Gbm).
//! 3. **Light-Days (Ld)** — the day-based hierarchy aligning with BrightDate's
//!    native unit. `1 Ld = c × 86_400 s = 25_902_068_371_200 m` exactly.
//!
//! Because the SI second and *c* are both defined to exact integer values,
//! every conversion in this module is **exact in principle** — limited only
//! by IEEE-754 double precision at very large magnitudes.

use crate::constants::SECONDS_PER_DAY;
use crate::types::BrightDateValue;

// ─── Fundamental Constants ──────────────────────────────────────────────────

/// Speed of light in vacuum, in metres per second. **Exact** by the 2019 SI
/// redefinition: the metre is defined such that *c* = `299_792_458 m/s`.
pub const SPEED_OF_LIGHT_M_PER_S: f64 = 299_792_458.0;

/// One BrightMeter / Bright-Second in metres: the distance light travels in
/// one SI second. Equal to [`SPEED_OF_LIGHT_M_PER_S`].
pub const BRIGHT_METER_M: f64 = SPEED_OF_LIGHT_M_PER_S;

/// One Light-Day in metres: `c × 86_400 s = 25_902_068_371_200 m` exactly.
pub const LIGHT_DAY_M: f64 = SPEED_OF_LIGHT_M_PER_S * SECONDS_PER_DAY;

// ─── Unit catalog ───────────────────────────────────────────────────────────

/// A unit in the Bright-Second / BrightMeter scalar hierarchy.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BrightUnit {
    /// Canonical unit name.
    pub name: &'static str,
    /// Standard symbol (e.g. `bs`, `bm`, `Ld`).
    pub symbol: &'static str,
    /// Definition expressed as seconds of light-travel time.
    pub seconds: f64,
    /// Equivalent physical distance in metres (= seconds × c).
    pub metres: f64,
    /// Short human-readable context phrase.
    pub context: &'static str,
}

/// BrightMeter hierarchy: SI-prefixed sub- and super-multiples of the
/// BrightMeter (= 1 Bright-Second = c × 1 s).
pub const BRIGHT_METER_UNITS: &[BrightUnit] = &[
    BrightUnit {
        name: "Micro-BrightMeter",
        symbol: "μbm",
        seconds: 1e-6,
        metres: SPEED_OF_LIGHT_M_PER_S * 1e-6,
        context: "≈ 300 m (city block, fibre tap distance)",
    },
    BrightUnit {
        name: "Milli-BrightMeter",
        symbol: "mbm",
        seconds: 1e-3,
        metres: SPEED_OF_LIGHT_M_PER_S * 1e-3,
        context: "≈ 300 km (low Earth orbit)",
    },
    BrightUnit {
        name: "BrightMeter",
        symbol: "bm",
        seconds: 1.0,
        metres: SPEED_OF_LIGHT_M_PER_S,
        context: "≈ 299_792 km (Earth–Moon ≈ 1.28 bm)",
    },
    BrightUnit {
        name: "Mega-BrightMeter",
        symbol: "Mbm",
        seconds: 1e6,
        metres: SPEED_OF_LIGHT_M_PER_S * 1e6,
        context: "≈ 2 × Earth–Sun (1 AU ≈ 0.5 Mbm)",
    },
    BrightUnit {
        name: "Giga-BrightMeter",
        symbol: "Gbm",
        seconds: 1e9,
        metres: SPEED_OF_LIGHT_M_PER_S * 1e9,
        context: "≈ 32 light-years (interstellar)",
    },
];

/// Light-Day hierarchy: day-aligned distance units that interoperate with
/// BrightDate's native day unit.
pub const LIGHT_DAY_UNITS: &[BrightUnit] = &[
    BrightUnit {
        name: "Light-Microday",
        symbol: "Lμd",
        seconds: SECONDS_PER_DAY * 1e-6,
        metres: LIGHT_DAY_M * 1e-6,
        context: "≈ 25.9 km",
    },
    BrightUnit {
        name: "Light-Milliday",
        symbol: "Lmd",
        seconds: SECONDS_PER_DAY * 1e-3,
        metres: LIGHT_DAY_M * 1e-3,
        context: "≈ 25_902 km (geostationary belt)",
    },
    BrightUnit {
        name: "Light-Day",
        symbol: "Ld",
        seconds: SECONDS_PER_DAY,
        metres: LIGHT_DAY_M,
        context: "≈ 25.9 × 10¹² m (cislunar to inner-system)",
    },
    BrightUnit {
        name: "Light-Kiloday",
        symbol: "Lkd",
        seconds: SECONDS_PER_DAY * 1e3,
        metres: LIGHT_DAY_M * 1e3,
        context: "≈ 2.74 light-years",
    },
];

// ─── Conversions: seconds ↔ metres ──────────────────────────────────────────

/// Convert SI seconds of light-travel time to metres.
#[inline]
pub fn seconds_to_metres(seconds: f64) -> f64 {
    seconds * SPEED_OF_LIGHT_M_PER_S
}

/// Convert metres to SI seconds of light-travel time.
#[inline]
pub fn metres_to_seconds(metres: f64) -> f64 {
    metres / SPEED_OF_LIGHT_M_PER_S
}

// ─── Conversions: BrightMeters ↔ everything ─────────────────────────────────

/// Convert seconds to BrightMeters. Numerically the identity (1 s of
/// light-travel = 1 BrightMeter), but typed at the API surface.
#[inline]
pub fn seconds_to_bright_meters(seconds: f64) -> f64 {
    seconds
}

/// Convert BrightMeters to seconds. Numerically the identity.
#[inline]
pub fn bright_meters_to_seconds(bright_meters: f64) -> f64 {
    bright_meters
}

/// Convert a distance in metres to BrightMeters.
#[inline]
pub fn metres_to_bright_meters(metres: f64) -> f64 {
    metres / BRIGHT_METER_M
}

/// Convert BrightMeters to metres.
#[inline]
pub fn bright_meters_to_metres(bright_meters: f64) -> f64 {
    bright_meters * BRIGHT_METER_M
}

/// Convert a duration in decimal days to Bright-Seconds. `1 day = 86_400 bs`.
#[inline]
pub fn days_to_bright_seconds(days: BrightDateValue) -> f64 {
    days * SECONDS_PER_DAY
}

/// Convert Bright-Seconds to a duration in decimal days. `86_400 bs = 1 day`.
#[inline]
pub fn bright_seconds_to_days(bright_seconds: f64) -> BrightDateValue {
    bright_seconds / SECONDS_PER_DAY
}

/// Convert a duration in decimal days to metres of light-travel distance.
#[inline]
pub fn days_to_metres(days: BrightDateValue) -> f64 {
    days * LIGHT_DAY_M
}

/// Convert metres to a duration in decimal days.
#[inline]
pub fn metres_to_days(metres: f64) -> BrightDateValue {
    metres / LIGHT_DAY_M
}
