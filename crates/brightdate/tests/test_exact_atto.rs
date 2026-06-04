//! Integration tests for `ExactBrightAtto` and `BrightDate` bridges.

use brightdate::{
    exact::J2000_UNIX_MS_I64,
    lens::{ticks_to_brightdate, ATTOSECONDS_PER_DAY, ATTOSECONDS_PER_PICOSECOND},
    BrightDate, ExactBrightAtto, ExactBrightDate,
};

const EPS: f64 = 1e-10;

#[test]
fn epoch_attoseconds_is_zero() {
    assert_eq!(ExactBrightAtto::epoch().attoseconds(), 0);
}

#[test]
fn j2000_unix_ms_maps_to_epoch() {
    assert_eq!(
        ExactBrightAtto::from_unix_ms(J2000_UNIX_MS_I64).attoseconds(),
        0
    );
}

#[test]
fn unix_ms_roundtrip() {
    let ms = 1_700_000_000_123_i64;
    assert_eq!(ExactBrightAtto::from_unix_ms(ms).to_unix_ms(), ms);
}

#[test]
fn to_brightdate_uses_divmod_lens() {
    let t = 9622_i128 * ATTOSECONDS_PER_DAY + 43_520_000_000_000_000_000;
    let atto = ExactBrightAtto::from_attoseconds(t);
    assert!((atto.to_brightdate() - ticks_to_brightdate(t, ATTOSECONDS_PER_DAY)).abs() < EPS);
}

#[test]
fn encode_decode_roundtrip() {
    let a = ExactBrightAtto::from_attoseconds(-999);
    let b = ExactBrightAtto::decode(&a.encode()).unwrap();
    assert_eq!(a, b);
}

#[test]
fn decode_rejects_bad_prefix() {
    assert!(ExactBrightAtto::decode("EBD1:123").is_err());
}

#[test]
fn be_bytes_roundtrip() {
    let a = ExactBrightAtto::from_attoseconds(-987_654_321);
    let bytes = a.to_be_bytes();
    assert_eq!(ExactBrightAtto::from_be_bytes(bytes), a);
}

#[test]
fn picosecond_bridge_exact() {
    let ps = ExactBrightDate::from_picoseconds(42);
    let atto = ExactBrightAtto::from_exact_brightdate(ps);
    assert_eq!(atto.attoseconds(), 42 * ATTOSECONDS_PER_PICOSECOND);
    assert_eq!(atto.to_exact_brightdate().picoseconds(), 42);
}

#[test]
fn add_days_advances_attoseconds() {
    let e = ExactBrightAtto::epoch().add_days(1);
    assert_eq!(e.attoseconds(), ATTOSECONDS_PER_DAY);
}

#[test]
fn brightdate_from_exact_bright_atto_matches_lens() {
    let atto = ExactBrightAtto::from_unix_ms(1_700_000_000_000);
    let bd = BrightDate::from_exact_bright_atto(atto);
    assert!((bd.value - atto.to_brightdate()).abs() < EPS);
}

#[test]
fn brightdate_attosecond_bridge_roundtrip_whole_days() {
    let atto = ExactBrightAtto::from_attoseconds(9622 * ATTOSECONDS_PER_DAY);
    let bd = BrightDate::from_exact_bright_atto(atto);
    let back = bd.to_exact_bright_atto().unwrap();
    assert_eq!(back, atto);
}

#[test]
fn brightdate_attosecond_bridge_preserves_decimal_days() {
    let atto = ExactBrightAtto::from_unix_ms(1_700_000_000_000);
    let bd = BrightDate::from_exact_bright_atto(atto);
    let back = bd.to_exact_bright_atto().unwrap();
    assert!((back.to_brightdate() - bd.value).abs() < EPS);
}

#[test]
fn from_brightdate_value_lossy_but_stable() {
    let bd = 9622.50417;
    let atto = ExactBrightAtto::from_brightdate(bd).unwrap();
    assert!((atto.to_brightdate() - bd).abs() < EPS);
}
