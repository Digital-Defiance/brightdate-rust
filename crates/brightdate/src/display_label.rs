//! Display Label — `BD` / `PBD` prefix convention for BrightDate scalars.
//!
//! The canonical BrightDate representation is a signed 64-bit float.
//! Negative scalars are mathematically fine for storage, comparison, and
//! arithmetic, but they read poorly in user-facing displays.
//!
//! The display convention is a **sign-flipping prefix**:
//!
//! | Internal scalar | Label form         |
//! | --------------- | ------------------ |
//! | `bd >= 0`       | `BD <bd>`          |
//! | `bd < 0`        | `PBD <abs(bd)>`    |
//!
//! `BD 0` is the canonical label for J2000.0. **There is no `PBD 0`.**
//! Formatters MUST NOT produce `PBD 0`; parsers MUST reject `PBD 0` as
//! invalid input.
//!
//! The internal scalar is unchanged; round-tripping through the label
//! layer is exact (sign-flip is bit-exact in IEEE 754).

use crate::types::BrightDateError;

/// Default decimal precision used by [`format_bd`] when no explicit
/// precision is supplied.
pub const DEFAULT_BD_PRECISION: usize = 3;

/// Discriminated union for label-only consumers.
///
/// - `BrightLabel::BD(value)`  with `value >= 0` represents post-J2000.0.
/// - `BrightLabel::PBD(value)` with `value > 0`  represents pre-J2000.0
///   (the underlying scalar is `-value`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BrightLabel {
    BD(f64),
    PBD(f64),
}

/// Render a signed `f64` BrightDate as a display label.
///
/// - `bd >= 0` → `"BD <bd>"`.
/// - `bd <  0` → `"PBD <abs(bd)>"`.
/// - `bd == 0` (including `-0.0`) → `"BD 0"`. Never `"PBD 0"`.
pub fn format_bd(bd: f64, precision: usize) -> Result<String, BrightDateError> {
    if !bd.is_finite() {
        return Err(BrightDateError::InvalidNumber(format!(
            "format_bd: value must be finite, got {bd}"
        )));
    }
    // Treat -0.0 as 0.0 so it never falls into the PBD branch.
    let v = if bd == 0.0 { 0.0_f64 } else { bd };
    let prefix = if v < 0.0 { "PBD" } else { "BD" };
    let magnitude = v.abs();
    Ok(format!("{prefix} {:.*}", precision, magnitude))
}

/// Render a [`BrightLabel`] tuple to its canonical string form.
pub fn format_bd_label(
    label: BrightLabel,
    precision: usize,
) -> Result<String, BrightDateError> {
    match label {
        BrightLabel::BD(value) if value < 0.0 => Err(BrightDateError::InvalidInput(
            format!("format_bd_label: BD value must be non-negative, got {value}"),
        )),
        BrightLabel::PBD(value) if value <= 0.0 => Err(BrightDateError::InvalidInput(
            format!(
                "format_bd_label: PBD value must be strictly positive, got {value}"
            ),
        )),
        BrightLabel::BD(value) => Ok(format!("BD {:.*}", precision, value)),
        BrightLabel::PBD(value) => Ok(format!("PBD {:.*}", precision, value)),
    }
}

/// Parse a display label back to a signed `f64` BrightDate.
///
/// - `"BD X"`  → `+X` (with `X >= 0`).
/// - `"PBD X"` → `-X` (with `X > 0`; `"PBD 0"` is rejected).
pub fn parse_bd(label: &str) -> Result<f64, BrightDateError> {
    let trimmed = label.trim();
    let (prefix, rest) = if let Some(rest) = trimmed.strip_prefix("BD") {
        ("BD", rest)
    } else if let Some(rest) = trimmed.strip_prefix("PBD") {
        ("PBD", rest)
    } else {
        return Err(BrightDateError::ParseError(format!(
            "parse_bd: not a recognised label, expected \"BD <n>\" or \"PBD <n>\", got {label:?}"
        )));
    };
    let body = rest.trim_start();
    let value: f64 = body.parse().map_err(|_| {
        BrightDateError::ParseError(format!(
            "parse_bd: numeric body did not parse, got {body:?}"
        ))
    })?;
    if !value.is_finite() {
        return Err(BrightDateError::ParseError(format!(
            "parse_bd: numeric body must be finite, got {body:?}"
        )));
    }
    match prefix {
        "BD" => {
            if value < 0.0 {
                Err(BrightDateError::InvalidInput(format!(
                    "parse_bd: BD value must be non-negative, got {value}"
                )))
            } else {
                Ok(value)
            }
        }
        "PBD" => {
            if value <= 0.0 {
                Err(BrightDateError::InvalidInput(format!(
                    "parse_bd: PBD value must be strictly positive, got {value}"
                )))
            } else {
                Ok(-value)
            }
        }
        _ => unreachable!(),
    }
}

/// Parse a display label into a [`BrightLabel`] tuple.
pub fn parse_bd_label(label: &str) -> Result<BrightLabel, BrightDateError> {
    let scalar = parse_bd(label)?;
    if scalar >= 0.0 {
        Ok(BrightLabel::BD(scalar))
    } else {
        Ok(BrightLabel::PBD(-scalar))
    }
}

/// Total order on label tuples.
///
/// 1. Any `BD` is later than any `PBD`.
/// 2. Within `BD`, larger value is later.
/// 3. Within `PBD`, smaller value is later (closer to J2000.0).
pub fn compare_bd_labels(a: BrightLabel, b: BrightLabel) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    match (a, b) {
        (BrightLabel::BD(_), BrightLabel::PBD(_)) => Ordering::Greater,
        (BrightLabel::PBD(_), BrightLabel::BD(_)) => Ordering::Less,
        (BrightLabel::BD(x), BrightLabel::BD(y)) => {
            x.partial_cmp(&y).unwrap_or(Ordering::Equal)
        }
        (BrightLabel::PBD(x), BrightLabel::PBD(y)) => {
            // Smaller PBD value is later.
            y.partial_cmp(&x).unwrap_or(Ordering::Equal)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_bd_renders_zero_as_canonical_bd() {
        assert_eq!(format_bd(0.0, DEFAULT_BD_PRECISION).unwrap(), "BD 0.000");
    }

    #[test]
    fn format_bd_never_produces_pbd_zero() {
        // Negative-zero must collapse to BD 0, not PBD 0.
        assert_eq!(format_bd(-0.0, DEFAULT_BD_PRECISION).unwrap(), "BD 0.000");
    }

    #[test]
    fn format_bd_renders_positive_with_bd_prefix() {
        assert_eq!(format_bd(9622.504, 3).unwrap(), "BD 9622.504");
        assert_eq!(format_bd(1.0, 3).unwrap(), "BD 1.000");
    }

    #[test]
    fn format_bd_renders_negative_with_pbd_prefix() {
        assert_eq!(format_bd(-11125.154, 3).unwrap(), "PBD 11125.154");
        assert_eq!(format_bd(-1.0, 0).unwrap(), "PBD 1");
    }

    #[test]
    fn format_bd_rejects_non_finite() {
        assert!(format_bd(f64::NAN, 3).is_err());
        assert!(format_bd(f64::INFINITY, 3).is_err());
    }

    #[test]
    fn parse_bd_parses_bd_labels() {
        assert_eq!(parse_bd("BD 0").unwrap(), 0.0);
        assert_eq!(parse_bd("BD 9622.504").unwrap(), 9622.504);
    }

    #[test]
    fn parse_bd_parses_pbd_labels() {
        assert_eq!(parse_bd("PBD 1").unwrap(), -1.0);
        assert_eq!(parse_bd("PBD 11125.154").unwrap(), -11125.154);
    }

    #[test]
    fn parse_bd_tolerates_whitespace() {
        assert_eq!(parse_bd("  BD 1.0  ").unwrap(), 1.0);
        assert_eq!(parse_bd("PBD  2.5").unwrap(), -2.5);
    }

    #[test]
    fn parse_bd_rejects_pbd_zero() {
        assert!(parse_bd("PBD 0").is_err());
        assert!(parse_bd("PBD 0.0").is_err());
    }

    #[test]
    fn parse_bd_rejects_negative_bodies() {
        assert!(parse_bd("BD -1").is_err());
        assert!(parse_bd("PBD -1").is_err());
    }

    #[test]
    fn parse_bd_rejects_unrecognised_input() {
        assert!(parse_bd("9622.504").is_err());
        assert!(parse_bd("XBD 1").is_err());
        assert!(parse_bd("").is_err());
    }

    #[test]
    fn round_trip_format_then_parse() {
        let cases = [0.0_f64, 1.0, 9622.504, -1.0, -11125.154, 1e6, -1e6];
        for v in cases {
            let label = format_bd(v, 9).unwrap();
            let back = parse_bd(&label).unwrap();
            assert!((back - v).abs() < 1e-9, "{v} -> {label} -> {back}");
        }
    }

    #[test]
    fn label_round_trip() {
        let bd = BrightLabel::BD(9622.504);
        let s = format_bd_label(bd, 3).unwrap();
        assert_eq!(s, "BD 9622.504");
        assert_eq!(parse_bd_label(&s).unwrap(), bd);

        let pbd = BrightLabel::PBD(11125.154);
        let s = format_bd_label(pbd, 3).unwrap();
        assert_eq!(s, "PBD 11125.154");
        assert_eq!(parse_bd_label(&s).unwrap(), pbd);
    }

    #[test]
    fn label_format_rejects_invalid_combinations() {
        assert!(format_bd_label(BrightLabel::BD(-1.0), 3).is_err());
        assert!(format_bd_label(BrightLabel::PBD(0.0), 3).is_err());
        assert!(format_bd_label(BrightLabel::PBD(-1.0), 3).is_err());
    }

    #[test]
    fn parse_bd_label_returns_bd_for_zero() {
        assert_eq!(parse_bd_label("BD 0").unwrap(), BrightLabel::BD(0.0));
    }

    #[test]
    fn compare_bd_orders_correctly() {
        use std::cmp::Ordering;
        let bd0 = BrightLabel::BD(0.0);
        let bd9k = BrightLabel::BD(9622.504);
        let pbd1 = BrightLabel::PBD(1.0);
        let pbd11k = BrightLabel::PBD(11125.154);

        assert_eq!(compare_bd_labels(bd0, pbd1), Ordering::Greater);
        assert_eq!(compare_bd_labels(pbd11k, bd0), Ordering::Less);
        assert_eq!(compare_bd_labels(bd9k, bd0), Ordering::Greater);
        assert_eq!(compare_bd_labels(pbd1, pbd11k), Ordering::Greater);
        assert_eq!(compare_bd_labels(bd9k, BrightLabel::BD(9622.504)), Ordering::Equal);
    }

    #[test]
    fn compare_bd_agrees_with_native_numeric_comparison() {
        // For each pair, label compare matches sign(scalarA - scalarB).
        let labels = [
            (0.0_f64, BrightLabel::BD(0.0)),
            (9622.504, BrightLabel::BD(9622.504)),
            (-1.0, BrightLabel::PBD(1.0)),
            (-11125.154, BrightLabel::PBD(11125.154)),
        ];
        for (sa, la) in labels.iter() {
            for (sb, lb) in labels.iter() {
                let label_ord = compare_bd_labels(*la, *lb);
                let scalar_ord = sa.partial_cmp(sb).unwrap();
                assert_eq!(
                    label_ord, scalar_ord,
                    "mismatch comparing {sa} vs {sb}"
                );
            }
        }
    }
}
