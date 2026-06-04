//! `ExactBrightAtto` — canonical v2 engine (attoseconds since J2000.0).
//!
//! One attosecond tick is one **light-attosecond** under the Bright Spacetime
//! `c = 1` convention. Use [`ExactBrightDate`](crate::ExactBrightDate) when
//! you prefer picosecond storage; convert between them exactly.

use crate::constants::J2000_UTC_UNIX_MS;
use crate::exact::{ExactBrightDate, J2000_UNIX_MS_I64};
use crate::lens::{
    brightdate_to_attoseconds, ticks_to_brightdate, ATTOSECONDS_PER_DAY,
    ATTOSECONDS_PER_PICOSECOND, ATTOSECONDS_PER_SECOND,
};
use crate::types::BrightDateError;
use chrono::{DateTime, Utc};

const AS_PER_MS: i128 = ATTOSECONDS_PER_SECOND / 1_000_000_000;

/// J2000.0 as Unix attoseconds (UTC label).
pub const J2000_UNIX_AS: i128 = (J2000_UNIX_MS_I64 as i128) * AS_PER_MS;

/// Immutable bit-exact time stored as attoseconds since J2000.0.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct ExactBrightAtto {
    attoseconds: i128,
}

impl ExactBrightAtto {
    #[inline]
    pub const fn from_attoseconds(attoseconds: i128) -> Self {
        Self { attoseconds }
    }

    #[inline]
    pub const fn from_unix_ms(unix_ms: i64) -> Self {
        Self {
            attoseconds: ((unix_ms - J2000_UNIX_MS_I64) as i128) * AS_PER_MS,
        }
    }

    #[inline]
    pub const fn from_unix_seconds(unix_seconds: i64) -> Self {
        Self {
            attoseconds: (unix_seconds as i128) * ATTOSECONDS_PER_SECOND - J2000_UNIX_AS,
        }
    }

    pub fn from_date_time(dt: DateTime<Utc>) -> Self {
        Self::from_unix_ms(dt.timestamp_millis())
    }

    pub fn from_iso(s: &str) -> Result<Self, BrightDateError> {
        let dt = DateTime::parse_from_rfc3339(s)
            .map_err(|e| BrightDateError::ParseError(format!("invalid ISO 8601: {s}: {e}")))?;
        Ok(Self::from_unix_ms(dt.timestamp_millis()))
    }

    pub fn from_brightdate(bd: f64) -> Result<Self, BrightDateError> {
        Ok(Self {
            attoseconds: brightdate_to_attoseconds(bd)?,
        })
    }

    pub const fn from_exact_brightdate(exact: ExactBrightDate) -> Self {
        Self {
            attoseconds: exact.picoseconds() * ATTOSECONDS_PER_PICOSECOND,
        }
    }

    #[inline]
    pub const fn epoch() -> Self {
        Self { attoseconds: 0 }
    }

    pub fn now() -> Self {
        Self::from_unix_ms(Utc::now().timestamp_millis())
    }

    #[inline]
    pub const fn attoseconds(self) -> i128 {
        self.attoseconds
    }

    pub fn to_unix_ms(self) -> i64 {
        let ms = self.attoseconds.div_euclid(AS_PER_MS);
        (ms + J2000_UNIX_MS_I64 as i128) as i64
    }

    pub fn to_brightdate(self) -> f64 {
        ticks_to_brightdate(self.attoseconds, ATTOSECONDS_PER_DAY)
    }

    pub fn to_exact_brightdate(self) -> ExactBrightDate {
        ExactBrightDate::from_picoseconds(self.attoseconds / ATTOSECONDS_PER_PICOSECOND)
    }

    #[inline]
    pub const fn add_attoseconds(self, as_ticks: i128) -> Self {
        Self {
            attoseconds: self.attoseconds + as_ticks,
        }
    }

    #[inline]
    pub const fn add_days(self, days: i128) -> Self {
        Self {
            attoseconds: self.attoseconds + days * ATTOSECONDS_PER_DAY,
        }
    }

    pub fn encode(self) -> String {
        format!("EBA1:{}", self.attoseconds)
    }

    pub fn decode(encoded: &str) -> Result<Self, BrightDateError> {
        let body = encoded
            .strip_prefix("EBA1:")
            .ok_or_else(|| BrightDateError::ParseError(format!("expected EBA1: prefix: {encoded}")))?;
        let v: i128 = body
            .parse()
            .map_err(|_| BrightDateError::ParseError(format!("invalid attoseconds: {body}")))?;
        Ok(Self::from_attoseconds(v))
    }

    pub fn to_be_bytes(self) -> [u8; 16] {
        self.attoseconds.to_be_bytes()
    }

    pub fn from_be_bytes(bytes: [u8; 16]) -> Self {
        Self::from_attoseconds(i128::from_be_bytes(bytes))
    }
}

impl From<ExactBrightDate> for ExactBrightAtto {
    fn from(exact: ExactBrightDate) -> Self {
        ExactBrightAtto::from_exact_brightdate(exact)
    }
}

impl From<ExactBrightAtto> for ExactBrightDate {
    fn from(atto: ExactBrightAtto) -> Self {
        atto.to_exact_brightdate()
    }
}

const _J2000_MS_MATCH: () = {
    assert!(J2000_UTC_UNIX_MS == 946_727_935_816.0);
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_is_zero() {
        assert_eq!(ExactBrightAtto::epoch().attoseconds(), 0);
    }

    #[test]
    fn unix_ms_roundtrip() {
        let ms = 1_700_000_000_000_i64;
        assert_eq!(ExactBrightAtto::from_unix_ms(ms).to_unix_ms(), ms);
    }

    #[test]
    fn picosecond_bridge_is_exact() {
        let ps = ExactBrightDate::from_picoseconds(42);
        let atto = ExactBrightAtto::from_exact_brightdate(ps);
        assert_eq!(atto.attoseconds(), 42 * ATTOSECONDS_PER_PICOSECOND);
    }

    #[test]
    fn encode_decode_roundtrip() {
        let a = ExactBrightAtto::from_attoseconds(-999);
        let b = ExactBrightAtto::decode(&a.encode()).unwrap();
        assert_eq!(a, b);
    }
}
