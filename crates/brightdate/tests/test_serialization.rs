use brightdate::serialization::*;
use brightdate::types::BrightDateError;

const EPS: f64 = 1e-9;

// ── serialize / deserialize ──────────────────────────────────────────────────

#[test]
fn serialize_stores_value() {
    let s = serialize(9622.5, Timescale::Utc);
    assert!((s.v - 9622.5).abs() < EPS);
}

#[test]
fn serialize_stores_timescale_utc() {
    let s = serialize(0.0, Timescale::Utc);
    assert_eq!(s.ts, Timescale::Utc);
}

#[test]
fn serialize_stores_timescale_tai() {
    let s = serialize(0.0, Timescale::Tai);
    assert_eq!(s.ts, Timescale::Tai);
}

#[test]
fn serialize_format_version_is_1() {
    let s = serialize(1234.0, Timescale::Utc);
    assert_eq!(s.fmt, 1);
}

#[test]
fn deserialize_valid_ok() {
    let s = serialize(9622.5, Timescale::Utc);
    let (v, ts) = deserialize(&s).unwrap();
    assert!((v - 9622.5).abs() < EPS);
    assert_eq!(ts, Timescale::Utc);
}

#[test]
fn deserialize_valid_tai() {
    let s = serialize(100.0, Timescale::Tai);
    let (v, ts) = deserialize(&s).unwrap();
    assert!((v - 100.0).abs() < EPS);
    assert_eq!(ts, Timescale::Tai);
}

#[test]
fn deserialize_wrong_version_errors() {
    let mut s = serialize(9622.5, Timescale::Utc);
    s.fmt = 2;
    assert!(deserialize(&s).is_err());
}

#[test]
fn deserialize_nan_errors() {
    let mut s = serialize(9622.5, Timescale::Utc);
    s.v = f64::NAN;
    assert!(deserialize(&s).is_err());
}

#[test]
fn deserialize_inf_errors() {
    let mut s = serialize(9622.5, Timescale::Utc);
    s.v = f64::INFINITY;
    assert!(deserialize(&s).is_err());
}

#[test]
fn deserialize_neg_inf_errors() {
    let mut s = serialize(9622.5, Timescale::Utc);
    s.v = f64::NEG_INFINITY;
    assert!(deserialize(&s).is_err());
}

#[test]
fn deserialize_negative_value_ok() {
    let s = serialize(-500.0, Timescale::Utc);
    let (v, _) = deserialize(&s).unwrap();
    assert!((v - (-500.0)).abs() < EPS);
}

#[test]
fn deserialize_zero_ok() {
    let s = serialize(0.0, Timescale::Utc);
    let (v, _) = deserialize(&s).unwrap();
    assert!(v.abs() < EPS);
}

// ── to_json / from_json ───────────────────────────────────────────────────────

#[test]
fn to_json_contains_v_field() {
    let json = to_json(9622.5, Timescale::Utc);
    assert!(json.contains("\"v\""));
}

#[test]
fn to_json_contains_ts_field() {
    let json = to_json(9622.5, Timescale::Utc);
    assert!(json.contains("\"ts\""));
}

#[test]
fn to_json_contains_fmt_field() {
    let json = to_json(9622.5, Timescale::Utc);
    assert!(json.contains("\"fmt\""));
}

#[test]
fn to_json_utc_timescale() {
    let json = to_json(9622.5, Timescale::Utc);
    assert!(json.contains("\"utc\""));
}

#[test]
fn to_json_tai_timescale() {
    let json = to_json(9622.5, Timescale::Tai);
    assert!(json.contains("\"tai\""));
}

#[test]
fn from_json_roundtrip_utc() {
    let json = to_json(9622.5, Timescale::Utc);
    let (v, ts) = from_json(&json).unwrap();
    assert!((v - 9622.5).abs() < EPS);
    assert_eq!(ts, Timescale::Utc);
}

#[test]
fn from_json_roundtrip_tai() {
    let json = to_json(9622.5, Timescale::Tai);
    let (v, ts) = from_json(&json).unwrap();
    assert!((v - 9622.5).abs() < EPS);
    assert_eq!(ts, Timescale::Tai);
}

#[test]
fn from_json_invalid_json_errors() {
    assert!(from_json("not json").is_err());
}

#[test]
fn from_json_empty_string_errors() {
    assert!(from_json("").is_err());
}

#[test]
fn from_json_empty_object_errors() {
    assert!(from_json("{}").is_err());
}

#[test]
fn from_json_negative_value() {
    let json = to_json(-100.0, Timescale::Utc);
    let (v, _) = from_json(&json).unwrap();
    assert!((v - (-100.0)).abs() < EPS);
}

#[test]
fn from_json_zero() {
    let json = to_json(0.0, Timescale::Utc);
    let (v, _) = from_json(&json).unwrap();
    assert!(v.abs() < EPS);
}

#[test]
fn from_json_large_value() {
    let json = to_json(1_000_000.0, Timescale::Utc);
    let (v, _) = from_json(&json).unwrap();
    assert!((v - 1_000_000.0).abs() < 0.001);
}

// ── encode / decode ───────────────────────────────────────────────────────────

#[test]
fn encode_starts_with_bd1() {
    let s = encode(9622.5, Timescale::Utc, 5);
    assert!(s.starts_with("BD1:"), "got: {s}");
}

#[test]
fn encode_utc_no_tai_suffix() {
    let s = encode(9622.5, Timescale::Utc, 5);
    assert!(!s.ends_with(":tai"), "got: {s}");
}

#[test]
fn encode_tai_has_tai_suffix() {
    let s = encode(9622.5, Timescale::Tai, 5);
    assert!(s.ends_with(":tai"), "got: {s}");
}

#[test]
fn encode_precision_controls_decimals() {
    let s = encode(9622.5, Timescale::Utc, 3);
    assert!(s.contains("9622.500"), "got: {s}");
}

#[test]
fn decode_valid_utc() {
    let (v, ts) = decode("BD1:9622.5").unwrap();
    assert!((v - 9622.5).abs() < 0.001);
    assert_eq!(ts, Timescale::Utc);
}

#[test]
fn decode_valid_tai() {
    let (v, ts) = decode("BD1:9622.5:tai").unwrap();
    assert!((v - 9622.5).abs() < 0.001);
    assert_eq!(ts, Timescale::Tai);
}

#[test]
fn decode_invalid_prefix_errors() {
    assert!(decode("INVALID:9622").is_err());
}

#[test]
fn decode_empty_errors() {
    assert!(decode("").is_err());
}

#[test]
fn decode_bd1_only_errors() {
    assert!(decode("BD1:").is_err());
}

#[test]
fn decode_nan_value_errors() {
    assert!(decode("BD1:NaN").is_err());
}

#[test]
fn decode_inf_value_errors() {
    assert!(decode("BD1:inf").is_err());
}

#[test]
fn encode_decode_roundtrip_utc() {
    let original = 9622.50417;
    let encoded = encode(original, Timescale::Utc, 8);
    let (v, ts) = decode(&encoded).unwrap();
    assert!((v - original).abs() < 1e-6);
    assert_eq!(ts, Timescale::Utc);
}

#[test]
fn encode_decode_roundtrip_tai() {
    let original = 9622.50417;
    let encoded = encode(original, Timescale::Tai, 8);
    let (v, ts) = decode(&encoded).unwrap();
    assert!((v - original).abs() < 1e-6);
    assert_eq!(ts, Timescale::Tai);
}

#[test]
fn encode_decode_negative_value() {
    let encoded = encode(-500.25, Timescale::Utc, 5);
    let (v, ts) = decode(&encoded).unwrap();
    assert!((v - (-500.25)).abs() < 0.001);
    assert_eq!(ts, Timescale::Utc);
}

#[test]
fn encode_decode_zero() {
    let encoded = encode(0.0, Timescale::Utc, 5);
    let (v, _) = decode(&encoded).unwrap();
    assert!(v.abs() < 0.001);
}

// ── to_bytes / from_bytes ─────────────────────────────────────────────────────

#[test]
fn to_bytes_returns_8_bytes() {
    let bytes = to_bytes(9622.5);
    assert_eq!(bytes.len(), 8);
}

#[test]
fn from_bytes_roundtrip_epoch() {
    let value = 0.0_f64;
    let bytes = to_bytes(value);
    let back = from_bytes(bytes).unwrap();
    assert!(back.abs() < EPS);
}

#[test]
fn from_bytes_roundtrip_positive() {
    let value = 9622.504_17_f64;
    let bytes = to_bytes(value);
    let back = from_bytes(bytes).unwrap();
    assert!((back - value).abs() < f64::EPSILON);
}

#[test]
fn from_bytes_roundtrip_negative() {
    let value = -500.25_f64;
    let bytes = to_bytes(value);
    let back = from_bytes(bytes).unwrap();
    assert!((back - value).abs() < f64::EPSILON);
}

#[test]
fn from_bytes_nan_errors() {
    let nan_bytes = f64::NAN.to_bits().to_be_bytes();
    assert!(from_bytes(nan_bytes).is_err());
}

#[test]
fn from_bytes_inf_errors() {
    let inf_bytes = f64::INFINITY.to_bits().to_be_bytes();
    assert!(from_bytes(inf_bytes).is_err());
}

#[test]
fn bytes_are_big_endian() {
    // 0.0 in IEEE 754 big-endian is all zeros
    let bytes = to_bytes(0.0);
    assert_eq!(bytes, [0u8; 8]);
}

#[test]
fn to_bytes_preserves_sign() {
    let pos = to_bytes(1.0);
    let neg = to_bytes(-1.0);
    // Sign bit is in most significant byte
    assert_ne!(pos[0], neg[0]);
}

// ── Timescale ─────────────────────────────────────────────────────────────────

#[test]
fn timescale_utc_ne_tai() {
    assert_ne!(Timescale::Utc, Timescale::Tai);
}

#[test]
fn timescale_clone() {
    let ts = Timescale::Tai;
    let cloned = ts;
    assert_eq!(ts, cloned);
}

#[test]
fn timescale_debug() {
    let s = format!("{:?}", Timescale::Utc);
    assert!(s.contains("Utc"));
}
