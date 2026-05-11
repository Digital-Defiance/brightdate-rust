//! BrightDate Serialization
//!
//! Compact serialization/deserialization of BrightDate values for storage,
//! transmission, and interoperability.

use serde::{Deserialize, Serialize};

use crate::types::{BrightDateError, BrightDateValue};

// ─── JSON-friendly struct ─────────────────────────────────────────────────────

/// Serialized BrightDate format for JSON storage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SerializedBrightDate {
    /// The BrightDate value.
    pub v: BrightDateValue,
    /// Timescale: `"utc"` or `"tai"`.
    pub ts: Timescale,
    /// Serialization format version (always `1`).
    pub fmt: u8,
}

/// Timescale discriminant used in serialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Timescale {
    Utc,
    Tai,
}

// ─── JSON serialization ───────────────────────────────────────────────────────

/// Serialize a BrightDate value to a compact [`SerializedBrightDate`].
pub fn serialize(value: BrightDateValue, timescale: Timescale) -> SerializedBrightDate {
    SerializedBrightDate {
        v: value,
        ts: timescale,
        fmt: 1,
    }
}

/// Deserialize a [`SerializedBrightDate`].
///
/// Returns an error if the format version is not supported or the value is not
/// finite.
pub fn deserialize(
    data: &SerializedBrightDate,
) -> Result<(BrightDateValue, Timescale), BrightDateError> {
    if data.fmt != 1 {
        return Err(BrightDateError::ParseError(format!(
            "unsupported BrightDate format version: {}",
            data.fmt
        )));
    }
    if !data.v.is_finite() {
        return Err(BrightDateError::InvalidNumber(format!(
            "BrightDate value is not finite: {}", data.v
        )));
    }
    Ok((data.v, data.ts))
}

/// Serialize to a JSON string.
pub fn to_json(value: BrightDateValue, timescale: Timescale) -> String {
    let s = serialize(value, timescale);
    serde_json::to_string(&s).unwrap_or_else(|_| "{}".to_string())
}

/// Deserialize from a JSON string.
pub fn from_json(json: &str) -> Result<(BrightDateValue, Timescale), BrightDateError> {
    let data: SerializedBrightDate = serde_json::from_str(json)
        .map_err(|e| BrightDateError::ParseError(e.to_string()))?;
    deserialize(&data)
}

// ─── Compact string encoding ──────────────────────────────────────────────────

/// Encode a BrightDate as a compact string for URLs or headers.
///
/// Format: `"BD1:<value>"` for UTC, `"BD1:<value>:tai"` for TAI.
///
/// `precision` controls the number of decimal places (default 8).
pub fn encode(value: BrightDateValue, timescale: Timescale, precision: usize) -> String {
    let s = format!("{:.prec$}", value, prec = precision);
    match timescale {
        Timescale::Tai => format!("BD1:{s}:tai"),
        Timescale::Utc => format!("BD1:{s}"),
    }
}

/// Decode a BrightDate from an encoded string produced by [`encode`].
///
/// Expects the format `"BD1:<value>[:<timescale>]"`.
pub fn decode(encoded: &str) -> Result<(BrightDateValue, Timescale), BrightDateError> {
    let body = encoded
        .strip_prefix("BD1:")
        .ok_or_else(|| BrightDateError::ParseError(
            format!("invalid BrightDate encoding: must start with \"BD1:\", got \"{encoded}\""),
        ))?;

    let parts: Vec<&str> = body.splitn(2, ':').collect();
    let value: f64 = parts[0].parse().map_err(|_| {
        BrightDateError::ParseError(format!(
            "invalid BrightDate encoding: invalid value \"{}\"",
            parts[0]
        ))
    })?;
    if !value.is_finite() {
        return Err(BrightDateError::InvalidNumber(format!(
            "BrightDate value is not finite: {}", value
        )));
    }

    let timescale = if parts.get(1) == Some(&"tai") {
        Timescale::Tai
    } else {
        Timescale::Utc
    };

    Ok((value, timescale))
}

// ─── Binary (big-endian f64) ──────────────────────────────────────────────────

/// Encode a BrightDate value as 8 bytes (big-endian IEEE 754 float64).
pub fn to_bytes(value: BrightDateValue) -> [u8; 8] {
    value.to_bits().to_be_bytes()
}

/// Decode 8 bytes (big-endian IEEE 754 float64) back to a BrightDate value.
pub fn from_bytes(bytes: [u8; 8]) -> Result<BrightDateValue, BrightDateError> {
    let bits = u64::from_be_bytes(bytes);
    let value = f64::from_bits(bits);
    if !value.is_finite() {
        return Err(BrightDateError::InvalidNumber(format!(
            "BrightDate binary value is not finite: {:?}", bytes
        )));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_roundtrip() {
        let json = to_json(9622.5, Timescale::Utc);
        let (v, ts) = from_json(&json).unwrap();
        assert!((v - 9622.5).abs() < 1e-10);
        assert_eq!(ts, Timescale::Utc);
    }

    #[test]
    fn json_tai_roundtrip() {
        let json = to_json(9622.5, Timescale::Tai);
        let (v, ts) = from_json(&json).unwrap();
        assert_eq!(ts, Timescale::Tai);
        assert!((v - 9622.5).abs() < 1e-10);
    }

    #[test]
    fn encode_decode_utc() {
        let s = encode(9622.5, Timescale::Utc, 5);
        assert!(s.starts_with("BD1:"));
        let (v, ts) = decode(&s).unwrap();
        assert!((v - 9622.5).abs() < 1e-4);
        assert_eq!(ts, Timescale::Utc);
    }

    #[test]
    fn encode_decode_tai() {
        let s = encode(9622.5, Timescale::Tai, 5);
        assert!(s.ends_with(":tai"));
        let (v, ts) = decode(&s).unwrap();
        assert_eq!(ts, Timescale::Tai);
        assert!((v - 9622.5).abs() < 1e-4);
    }

    #[test]
    fn bytes_roundtrip() {
        let value = 9622.504_17_f64;
        let bytes = to_bytes(value);
        let back = from_bytes(bytes).unwrap();
        assert!((back - value).abs() < f64::EPSILON);
    }

    #[test]
    fn decode_invalid_prefix() {
        assert!(decode("INVALID:9622").is_err());
    }
}
