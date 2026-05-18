//! PBD — Pre-BrightDate Eras (Tera-second paging).
//!
//! The "deep-time" naming layer for [`BrightDate`](crate::BrightDate). PBD is
//! *strictly* the historical / pre-epoch view: it labels instants that lie
//! **before** J2000.0. Anything from J2000.0 forward is plain **BD** (a
//! continuous positive scalar) — there is no `PBD0`.
//!
//! ```text
//!   t < 0  →  PBDN (paged, N ≥ 1)         "The Archives"
//!   t ≥ 0  →  BD   (scalar, never paged)   "The Era of Light"
//! ```
//!
//! ## Paging model
//!
//! Let `T = 1_000_000_000_000` (one Tera-second).
//!
//! | Domain | Raw-second range          | Label form                            |
//! | ------ | ------------------------- | ------------------------------------- |
//! | BD     | `[0, +∞)`                 | scalar, e.g. `9_635.845 BD`           |
//! | PBD1   | `(−T, 0)`                 | ~31,710 yr; contains recorded history |
//! | PBD2   | `(−2T, −T]`               | Late Paleolithic                      |
//! | PBD*N* | `(−N·T, −(N−1)·T]`        | One Tera-second per page              |
//!
//! **Linear-vector rule.** Page values *always* increase toward J2000.0.
//! Larger `page` within an era = later in real time. Numbers do **not**
//! count backwards in pre-epoch eras (this is the opposite of BC labeling).
//!
//! **Mapping formula** (defined only for `raw_seconds < 0`):
//!
//! ```text
//! era  = floor(|raw_seconds| / T) + 1     // ≥ 1
//! page = (raw_seconds mod T) + T          // always in (0, T]
//! ```
//!
//! For the unified label that handles both halves of the timeline, see
//! [`to_bright_label`] / [`format_bright_label`].

use crate::constants::SECONDS_PER_DAY;
use crate::exact::{ExactBrightDate, PS_PER_NS, PS_PER_S};
use crate::instant::BrightInstant;
use crate::types::BrightDateError;
use crate::BrightDate;

// ── Constants ───────────────────────────────────────────────────────────────

/// One Tera-Bright. The PBD page size, expressed in **Bright-seconds**
/// (SI seconds). Exactly `10¹² s ≈ 31,709.79` Julian years.
pub const PBD_ERA_SECONDS: i64 = 1_000_000_000_000;

/// `PBD_ERA_SECONDS` as `f64`, for the Float64 PBD path.
pub const PBD_ERA_SECONDS_F: f64 = PBD_ERA_SECONDS as f64;

/// `PBD_ERA_SECONDS` expressed in picoseconds. Used by the exact
/// ([`ExactBrightDate`]) PBD path.
///
/// `PBD_ERA_SECONDS × 10¹² ps/s = 10²⁴ ps`.
pub const PBD_ERA_PICOSECONDS: i128 = 1_000_000_000_000_000_000_000_000;

/// Default decimal precision for [`format_pbd`].
pub const DEFAULT_PBD_PRECISION: u8 = 3;

/// Default decimal precision for the `BD` branch of [`format_bright_label`].
pub const DEFAULT_BD_PRECISION: u8 = 3;

// ── Types ────────────────────────────────────────────────────────────────────

/// Float64 PBD tuple. **Only valid for pre-J2000.0 instants.**
///
/// - `era`: positive integer (`≥ 1`). There is no `PBD0` — non-negative
///   scalars are plain BD, not a PBD page.
/// - `page`: Bright-seconds within the era, in `(0, PBD_ERA_SECONDS]`.
///   *Larger page = later in real time* (the linear-vector rule).
#[derive(
    Debug, Clone, Copy, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub struct Pbd {
    pub era: u32,
    pub page: f64,
}

/// Bit-exact PBD tuple paired with [`ExactBrightDate`]. **Pre-J2000.0 only**,
/// same rules as [`Pbd`].
///
/// - `era`: positive integer (≥ 1). Even cosmological depth (~4.35×10⁵ eras
///   at the Big Bang) easily fits.
/// - `page_picoseconds`: picoseconds within the era, in
///   `(0, PBD_ERA_PICOSECONDS]`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct ExactPbd {
    pub era: u32,
    pub page_picoseconds: u128,
}

/// Unified label for any instant on the BrightDate timeline.
#[derive(
    Debug, Clone, Copy, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "kind")]
pub enum BrightLabel {
    /// J2000.0 and forward; carries the raw scalar in seconds (`≥ 0`).
    #[serde(rename = "BD")]
    Bd { seconds: f64 },
    /// Strictly before J2000.0; carries the canonical paged `(era, page)`
    /// tuple with `era ≥ 1`.
    #[serde(rename = "PBD")]
    Pbd { era: u32, page: f64 },
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn assert_finite(value: f64, name: &str) -> Result<(), BrightDateError> {
    if !value.is_finite() {
        return Err(BrightDateError::InvalidInput(format!(
            "{name} must be a finite number, got: {value}"
        )));
    }
    Ok(())
}

fn assert_era(era: u32) -> Result<(), BrightDateError> {
    if era < 1 {
        return Err(BrightDateError::InvalidInput(format!(
            "PBD era must be ≥ 1 — there is no PBD0. Got: {era}"
        )));
    }
    Ok(())
}

fn assert_negative(raw_seconds: f64) -> Result<(), BrightDateError> {
    if raw_seconds >= 0.0 {
        return Err(BrightDateError::OutOfRange(format!(
            "PBD is defined only for t < 0 (pre-J2000.0). \
             For non-negative scalars, use BD directly. Got: {raw_seconds}"
        )));
    }
    Ok(())
}

// ── Float64 conversions ──────────────────────────────────────────────────────

/// Convert a raw signed Bright-second count to its canonical PBD tuple.
///
/// **Defined only for `raw_seconds < 0`.** Non-negative inputs return
/// [`BrightDateError::OutOfRange`] — they are plain BD scalars and have no
/// paged form. For a timeline-agnostic label that handles both halves, use
/// [`to_bright_label`].
pub fn to_pbd(raw_seconds: f64) -> Result<Pbd, BrightDateError> {
    assert_finite(raw_seconds, "raw_seconds")?;
    assert_negative(raw_seconds)?;
    let era = ((-raw_seconds) / PBD_ERA_SECONDS_F).floor() as u32 + 1;
    let page = (raw_seconds % PBD_ERA_SECONDS_F) + PBD_ERA_SECONDS_F;
    Ok(Pbd { era, page })
}

/// Invert [`to_pbd`]: rebuild the signed Bright-second scalar from `(era, page)`.
/// Result is always `< 0` for canonical tuples.
///
/// Lenient on the `page` value — accepts non-canonical pairs and treats them
/// as the linear offset `raw_seconds = page − era · PBD_ERA_SECONDS`.
pub fn from_pbd(pbd: Pbd) -> Result<f64, BrightDateError> {
    assert_era(pbd.era)?;
    assert_finite(pbd.page, "PBD page")?;
    Ok(pbd.page - (pbd.era as f64) * PBD_ERA_SECONDS_F)
}

/// Convenience: convert a *pre-J2000.0* [`BrightDate`] to a [`Pbd`] tuple.
pub fn bright_date_to_pbd(bd: BrightDate) -> Result<Pbd, BrightDateError> {
    to_pbd(bd.value * SECONDS_PER_DAY)
}

/// Convenience: build a [`BrightDate`] from a [`Pbd`] tuple. Result is always
/// pre-J2000.0 for canonical tuples.
pub fn bright_date_from_pbd(pbd: Pbd) -> Result<BrightDate, BrightDateError> {
    Ok(BrightDate::from_value(from_pbd(pbd)? / SECONDS_PER_DAY))
}

/// The era index for a pre-J2000.0 raw Bright-second count, without
/// computing the page value.
pub fn pbd_era(raw_seconds: f64) -> Result<u32, BrightDateError> {
    assert_finite(raw_seconds, "raw_seconds")?;
    assert_negative(raw_seconds)?;
    Ok(((-raw_seconds) / PBD_ERA_SECONDS_F).floor() as u32 + 1)
}

/// The page value for a pre-J2000.0 raw Bright-second count, without
/// recomputing the era.
pub fn pbd_page(raw_seconds: f64) -> Result<f64, BrightDateError> {
    assert_finite(raw_seconds, "raw_seconds")?;
    assert_negative(raw_seconds)?;
    Ok((raw_seconds % PBD_ERA_SECONDS_F) + PBD_ERA_SECONDS_F)
}

// ── Comparison ───────────────────────────────────────────────────────────────

/// Sort-order comparator over PBD tuples by the instant they label.
///
/// Smaller `era` = later in time. Within an era, larger `page` = later.
pub fn compare_pbd(a: Pbd, b: Pbd) -> std::cmp::Ordering {
    use std::cmp::Ordering::*;
    if a.era != b.era {
        return if a.era < b.era { Greater } else { Less };
    }
    if a.page == b.page {
        Equal
    } else if a.page > b.page {
        Greater
    } else {
        Less
    }
}

/// True iff `a` is strictly later than `b` on the timeline.
pub fn is_pbd_later(a: Pbd, b: Pbd) -> bool {
    compare_pbd(a, b) == std::cmp::Ordering::Greater
}

// ── Formatting / parsing ─────────────────────────────────────────────────────

/// Format a PBD tuple as a human-readable label, e.g.
/// `"PBD1: 842000000000.000"`.
pub fn format_pbd(pbd: Pbd, precision: u8) -> Result<String, BrightDateError> {
    assert_era(pbd.era)?;
    assert_finite(pbd.page, "PBD page")?;
    if precision > 20 {
        return Err(BrightDateError::InvalidPrecision(format!(
            "PBD precision must be in [0, 20], got: {precision}"
        )));
    }
    Ok(format!(
        "PBD{}: {:.*}",
        pbd.era,
        precision as usize,
        pbd.page
    ))
}

/// Parse a PBD label produced by [`format_pbd`].
pub fn parse_pbd(label: &str) -> Result<Pbd, BrightDateError> {
    let trimmed = label.trim();
    let after_pbd = trimmed
        .strip_prefix("PBD")
        .ok_or_else(|| BrightDateError::ParseError(format!("invalid PBD label: {label:?}")))?;
    let (era_part, page_part) = after_pbd
        .split_once(':')
        .ok_or_else(|| BrightDateError::ParseError(format!("invalid PBD label: {label:?}")))?;
    let era: u32 = era_part
        .trim()
        .trim_start_matches('+')
        .parse()
        .map_err(|_| BrightDateError::ParseError(format!("invalid PBD era in {label:?}")))?;
    let page: f64 = page_part
        .trim()
        .parse()
        .map_err(|_| BrightDateError::ParseError(format!("invalid PBD page in {label:?}")))?;
    if !page.is_finite() {
        return Err(BrightDateError::ParseError(format!(
            "non-finite PBD page in {label:?}"
        )));
    }
    Ok(Pbd { era, page })
}

// ── ExactBrightDate conversions ──────────────────────────────────────────────

/// Convert an [`ExactBrightDate`] to its canonical exact PBD tuple.
///
/// **Defined only for pre-J2000.0 instants** (`picoseconds < 0`). The
/// picosecond page preserves every picosecond of the underlying instant.
pub fn to_exact_pbd(value: ExactBrightDate) -> Result<ExactPbd, BrightDateError> {
    let ps = value.picoseconds();
    if ps >= 0 {
        return Err(BrightDateError::OutOfRange(format!(
            "PBD is defined only for t < 0 (pre-J2000.0). \
             For non-negative instants, use BD directly. Got picoseconds: {ps}"
        )));
    }
    // era = floor(|ps| / T_ps) + 1. With ps < 0, integer division of (-ps)
    // by T_ps truncates toward zero, which equals floor for positive ratios.
    let era = ((-ps) / PBD_ERA_PICOSECONDS + 1) as u32;
    // page = (ps mod T_ps) + T_ps. Rust i128 % preserves sign of dividend,
    // matching the TypeScript/Float64 spec. ps % T is in (-T, 0], so adding
    // T puts the page in (0, T].
    let page_signed = (ps % PBD_ERA_PICOSECONDS) + PBD_ERA_PICOSECONDS;
    Ok(ExactPbd {
        era,
        page_picoseconds: page_signed as u128,
    })
}

/// Invert [`to_exact_pbd`]: rebuild an [`ExactBrightDate`]. Result is always
/// pre-J2000.0 for canonical tuples.
pub fn from_exact_pbd(pbd: ExactPbd) -> Result<ExactBrightDate, BrightDateError> {
    assert_era(pbd.era)?;
    let ps = (pbd.page_picoseconds as i128) - (pbd.era as i128) * PBD_ERA_PICOSECONDS;
    Ok(ExactBrightDate::from_picoseconds(ps))
}

/// Sort-order comparator over [`ExactPbd`] tuples. Same semantics as
/// [`compare_pbd`].
pub fn compare_exact_pbd(a: ExactPbd, b: ExactPbd) -> std::cmp::Ordering {
    use std::cmp::Ordering::*;
    if a.era != b.era {
        return if a.era < b.era { Greater } else { Less };
    }
    a.page_picoseconds.cmp(&b.page_picoseconds)
}

// ── BrightInstant bridge ─────────────────────────────────────────────────────

/// Convert a *pre-J2000.0* [`BrightInstant`] to its canonical [`ExactPbd`]
/// tuple.
///
/// Since `BrightInstant` is TAI-anchored (no leap seconds), its
/// `(tai_seconds, tai_nanos)` pair is a pure SI-second offset from J2000.0.
/// The PBD label drops below `BrightInstant`'s nanosecond resolution into
/// picoseconds (sub-ns digits are zero), so the bridge is lossless in both
/// directions for any negative-TAI instant.
pub fn bright_instant_to_pbd(instant: BrightInstant) -> Result<ExactPbd, BrightDateError> {
    let ps = (instant.tai_seconds_since_j2000() as i128) * PS_PER_S
        + (instant.tai_nanos() as i128) * PS_PER_NS;
    to_exact_pbd(ExactBrightDate::from_picoseconds(ps))
}

/// Invert [`bright_instant_to_pbd`]: rebuild a [`BrightInstant`] from an
/// [`ExactPbd`] tuple.
///
/// Any sub-nanosecond residue on the PBD page is **truncated toward negative
/// infinity** (Euclidean) so the result satisfies
/// `tai_nanos ∈ [0, 1_000_000_000)`. Round-trips from a `BrightInstant`
/// (which carries no sub-ns digits) are bit-exact.
pub fn bright_instant_from_pbd(pbd: ExactPbd) -> Result<BrightInstant, BrightDateError> {
    let ps = from_exact_pbd(pbd)?.picoseconds();
    let mut secs = ps.div_euclid(PS_PER_S);
    let mut sub_ps = ps.rem_euclid(PS_PER_S);
    if sub_ps < 0 {
        secs -= 1;
        sub_ps += PS_PER_S;
    }
    let tai_nanos = (sub_ps / PS_PER_NS) as u32;
    BrightInstant::from_tai_components(secs as i64, tai_nanos)
}

// ── Unified BrightLabel ──────────────────────────────────────────────────────

/// Convert a raw signed Bright-second count to the unified [`BrightLabel`].
///
/// - `raw_seconds ≥ 0`  →  `BrightLabel::Bd { seconds }`
/// - `raw_seconds < 0`  →  `BrightLabel::Pbd { era, page }` with `era ≥ 1`
pub fn to_bright_label(raw_seconds: f64) -> Result<BrightLabel, BrightDateError> {
    assert_finite(raw_seconds, "raw_seconds")?;
    if raw_seconds >= 0.0 {
        return Ok(BrightLabel::Bd {
            seconds: raw_seconds,
        });
    }
    let era = ((-raw_seconds) / PBD_ERA_SECONDS_F).floor() as u32 + 1;
    let page = (raw_seconds % PBD_ERA_SECONDS_F) + PBD_ERA_SECONDS_F;
    Ok(BrightLabel::Pbd { era, page })
}

/// Invert [`to_bright_label`]: rebuild the signed Bright-second scalar.
pub fn from_bright_label(label: BrightLabel) -> Result<f64, BrightDateError> {
    match label {
        BrightLabel::Bd { seconds } => {
            assert_finite(seconds, "BD seconds")?;
            if seconds < 0.0 {
                return Err(BrightDateError::OutOfRange(format!(
                    "BD scalar must be ≥ 0; negative values are PBD. Got: {seconds}"
                )));
            }
            Ok(seconds)
        }
        BrightLabel::Pbd { era, page } => from_pbd(Pbd { era, page }),
    }
}

/// Format a [`BrightLabel`] as a human-readable string.
///
/// - BD branch  →  `"9635.845 BD"`
/// - PBD branch →  `"PBD1: 842000000000.000"`
pub fn format_bright_label(
    label: BrightLabel,
    bd_precision: u8,
    pbd_precision: u8,
) -> Result<String, BrightDateError> {
    match label {
        BrightLabel::Bd { seconds } => {
            if bd_precision > 20 {
                return Err(BrightDateError::InvalidPrecision(format!(
                    "BD precision must be in [0, 20], got: {bd_precision}"
                )));
            }
            assert_finite(seconds, "BD seconds")?;
            Ok(format!("{:.*} BD", bd_precision as usize, seconds))
        }
        BrightLabel::Pbd { era, page } => format_pbd(Pbd { era, page }, pbd_precision),
    }
}

/// Parse a label produced by [`format_bright_label`]. Accepts either form.
pub fn parse_bright_label(label: &str) -> Result<BrightLabel, BrightDateError> {
    let trimmed = label.trim();
    if let Some(num_part) = trimmed.strip_suffix("BD") {
        let seconds: f64 = num_part.trim().parse().map_err(|_| {
            BrightDateError::ParseError(format!("invalid BD scalar in {label:?}"))
        })?;
        if !seconds.is_finite() || seconds < 0.0 {
            return Err(BrightDateError::ParseError(format!(
                "invalid BD scalar in {label:?}"
            )));
        }
        return Ok(BrightLabel::Bd { seconds });
    }
    let pbd = parse_pbd(label)?;
    Ok(BrightLabel::Pbd {
        era: pbd.era,
        page: pbd.page,
    })
}

/// Convenience: label a [`BrightDate`] directly.
pub fn brightdate_to_label(bd: BrightDate) -> Result<BrightLabel, BrightDateError> {
    to_bright_label(bd.value * SECONDS_PER_DAY)
}

#[cfg(test)]
mod tests {
    use super::*;

    const T: f64 = PBD_ERA_SECONDS_F;

    #[test]
    fn just_before_j2000_is_pbd1_top() {
        let pbd = to_pbd(-1.0).unwrap();
        assert_eq!(pbd.era, 1);
        assert!((pbd.page - (T - 1.0)).abs() < 1e-3);
    }

    #[test]
    fn exact_minus_t_is_pbd2_top() {
        let pbd = to_pbd(-T).unwrap();
        assert_eq!(pbd.era, 2);
        assert!((pbd.page - T).abs() < 1e-3);
    }

    #[test]
    fn pbd_roundtrip() {
        for raw in [-1.0, -T + 1.0, -T, -T - 1.0, -1.578e11, -3.156e12] {
            let pbd = to_pbd(raw).unwrap();
            let back = from_pbd(pbd).unwrap();
            assert!(
                (back - raw).abs() < 1e-3,
                "raw={raw}  pbd={pbd:?}  back={back}"
            );
        }
    }

    #[test]
    fn to_pbd_rejects_non_negative() {
        assert!(to_pbd(0.0).is_err());
        assert!(to_pbd(1.0).is_err());
        assert!(to_pbd(f64::NAN).is_err());
    }

    #[test]
    fn era_and_page_helpers_agree() {
        let raw = -1.578e11;
        let pbd = to_pbd(raw).unwrap();
        assert_eq!(pbd_era(raw).unwrap(), pbd.era);
        assert!((pbd_page(raw).unwrap() - pbd.page).abs() < 1e-6);
    }

    #[test]
    fn format_and_parse_pbd_roundtrip() {
        let pbd = Pbd {
            era: 1,
            page: 842_000_000_000.0,
        };
        let s = format_pbd(pbd, 3).unwrap();
        assert_eq!(s, "PBD1: 842000000000.000");
        let parsed = parse_pbd(&s).unwrap();
        assert_eq!(parsed.era, pbd.era);
        assert!((parsed.page - pbd.page).abs() < 1e-3);
    }

    #[test]
    fn compare_pbd_orders_by_real_time() {
        // older (deeper-era OR smaller page within era)
        let earlier = Pbd { era: 2, page: 1.0 };
        let later = Pbd { era: 1, page: 1.0 };
        assert!(is_pbd_later(later, earlier));
        let same_era_earlier = Pbd { era: 1, page: 1.0 };
        let same_era_later = Pbd {
            era: 1,
            page: 100.0,
        };
        assert!(is_pbd_later(same_era_later, same_era_earlier));
    }

    #[test]
    fn exact_pbd_roundtrip() {
        for ps in [-1_i128, -PBD_ERA_PICOSECONDS, -PBD_ERA_PICOSECONDS - 1, -123_456_789_012_345i128] {
            let exact = ExactBrightDate::from_picoseconds(ps);
            let pbd = to_exact_pbd(exact).unwrap();
            assert!(pbd.era >= 1);
            assert!(pbd.page_picoseconds > 0 && pbd.page_picoseconds <= PBD_ERA_PICOSECONDS as u128);
            let back = from_exact_pbd(pbd).unwrap();
            assert_eq!(back.picoseconds(), ps);
        }
    }

    #[test]
    fn exact_pbd_rejects_non_negative() {
        assert!(to_exact_pbd(ExactBrightDate::epoch()).is_err());
        assert!(to_exact_pbd(ExactBrightDate::from_picoseconds(1)).is_err());
    }

    #[test]
    fn bright_instant_pbd_roundtrip() {
        let inst = BrightInstant::from_tai_components(-1_000_000_000, 123_456_789).unwrap();
        let pbd = bright_instant_to_pbd(inst).unwrap();
        let back = bright_instant_from_pbd(pbd).unwrap();
        assert_eq!(back, inst);
    }

    #[test]
    fn bright_label_dispatch() {
        assert!(matches!(
            to_bright_label(0.0).unwrap(),
            BrightLabel::Bd { .. }
        ));
        assert!(matches!(
            to_bright_label(1.0).unwrap(),
            BrightLabel::Bd { .. }
        ));
        assert!(matches!(
            to_bright_label(-1.0).unwrap(),
            BrightLabel::Pbd { era: 1, .. }
        ));
        assert!(matches!(
            to_bright_label(-PBD_ERA_SECONDS_F).unwrap(),
            BrightLabel::Pbd { era: 2, .. }
        ));
    }

    #[test]
    fn bright_label_roundtrip() {
        for raw in [0.0, 1.0, 12345.0, -1.0, -T + 1.0, -2.0 * T] {
            let label = to_bright_label(raw).unwrap();
            let back = from_bright_label(label).unwrap();
            assert!((back - raw).abs() < 1e-3);
        }
    }

    #[test]
    fn format_parse_bright_label_roundtrip() {
        let bd = BrightLabel::Bd { seconds: 9635.845 };
        let s = format_bright_label(bd, 3, 3).unwrap();
        assert_eq!(s, "9635.845 BD");
        let parsed = parse_bright_label(&s).unwrap();
        assert!(matches!(parsed, BrightLabel::Bd { .. }));

        let pbd = BrightLabel::Pbd {
            era: 1,
            page: 842_000_000_000.0,
        };
        let s = format_bright_label(pbd, 3, 3).unwrap();
        assert!(s.starts_with("PBD1"));
        let parsed = parse_bright_label(&s).unwrap();
        assert!(matches!(parsed, BrightLabel::Pbd { era: 1, .. }));
    }

    #[test]
    fn brightdate_to_label_dispatches() {
        let positive = BrightDate::from_value(1.0);
        assert!(matches!(
            brightdate_to_label(positive).unwrap(),
            BrightLabel::Bd { .. }
        ));
        let pre = BrightDate::from_value(-1.0);
        assert!(matches!(
            brightdate_to_label(pre).unwrap(),
            BrightLabel::Pbd { era: 1, .. }
        ));
    }
}
