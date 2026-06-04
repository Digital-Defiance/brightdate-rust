//! Tests for the BrightDate integer lens (divmod ↔ decimal days).

use brightdate::lens::{
    brightdate_to_attoseconds, brightdate_to_picoseconds, split_ticks_into_day_parts,
    ticks_to_brightdate, ATTOSECONDS_PER_DAY, PICOSECONDS_PER_DAY,
};

const EPS: f64 = 1e-10;

#[test]
fn split_ticks_negative_euclidean() {
    let (days, rem) = split_ticks_into_day_parts(-1, ATTOSECONDS_PER_DAY);
    assert_eq!(days, -1);
    assert_eq!(rem, ATTOSECONDS_PER_DAY - 1);
}

#[test]
fn split_ticks_one_day_plus_five() {
    let t = ATTOSECONDS_PER_DAY + 5;
    let (days, rem) = split_ticks_into_day_parts(t, ATTOSECONDS_PER_DAY);
    assert_eq!(days, 1);
    assert_eq!(rem, 5);
}

#[test]
fn ticks_to_brightdate_matches_one_day() {
    let bd = ticks_to_brightdate(ATTOSECONDS_PER_DAY, ATTOSECONDS_PER_DAY);
    assert!((bd - 1.0).abs() < EPS);
}

#[test]
fn brightdate_to_attoseconds_roundtrip_typical() {
    let bd = 9622.50417;
    let as_ticks = brightdate_to_attoseconds(bd).unwrap();
    let back = ticks_to_brightdate(as_ticks, ATTOSECONDS_PER_DAY);
    assert!((back - bd).abs() < EPS);
}

#[test]
fn brightdate_to_attoseconds_rejects_nan() {
    assert!(brightdate_to_attoseconds(f64::NAN).is_err());
}

#[test]
fn brightdate_to_picoseconds_one_day() {
    let ps = brightdate_to_picoseconds(1.0).unwrap();
    assert_eq!(ps, PICOSECONDS_PER_DAY);
}
