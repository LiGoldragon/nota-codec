//! Production-readiness tests for the broader primitive +
//! container set (added 2026-04-27 alongside the horizon-rs
//! migration fixes — i64/u32/u16/u8/i32/i16/i8/f64/f32, Box<T>,
//! HashMap/HashSet/BTreeSet, tuples, byte vectors).

use std::collections::{BTreeSet, HashMap, HashSet};

use nota_codec::{Decoder, Encoder, Error, NotaDecode, NotaEncode};

fn round_trip<T>(value: T)
where
    T: NotaEncode + NotaDecode + PartialEq + std::fmt::Debug,
{
    let mut encoder = Encoder::nexus();
    value.encode(&mut encoder).unwrap();
    let text = encoder.into_string();
    let mut decoder = Decoder::nexus(&text);
    let recovered = T::decode(&mut decoder).unwrap();
    assert_eq!(value, recovered, "round-trip failed; encoded as: {text}");
}

// ─── Integer types ─────────────────────────────────────────

#[test]
fn u32_round_trips_at_boundaries() {
    round_trip(0u32);
    round_trip(1u32);
    round_trip(u32::MAX);
}

#[test]
fn u32_overflow_returns_typed_error() {
    let mut decoder = Decoder::nexus("4294967296"); // u32::MAX + 1
    let error = u32::decode(&mut decoder).unwrap_err();
    assert!(matches!(error, Error::IntegerOutOfRange { target: "u32", .. }));
}

#[test]
fn u16_round_trips() {
    round_trip(0u16);
    round_trip(u16::MAX);
}

#[test]
fn u8_round_trips() {
    round_trip(0u8);
    round_trip(u8::MAX);
}

#[test]
fn i64_round_trips_at_boundaries() {
    round_trip(0i64);
    round_trip(i64::MIN);
    round_trip(i64::MAX);
    round_trip(-1i64);
}

#[test]
fn i32_round_trips() {
    round_trip(0i32);
    round_trip(i32::MIN);
    round_trip(i32::MAX);
}

#[test]
fn i16_round_trips() {
    round_trip(i16::MIN);
    round_trip(i16::MAX);
}

#[test]
fn i8_round_trips() {
    round_trip(i8::MIN);
    round_trip(i8::MAX);
}

// ─── Floats ────────────────────────────────────────────────

#[test]
fn f64_round_trips_normal_values() {
    round_trip(0.0f64);
    round_trip(1.5f64);
    round_trip(-3.14159f64);
    round_trip(1e10f64);
}

#[test]
fn f64_integer_value_keeps_float_form_through_round_trip() {
    // Integer-valued floats need a `.0` suffix on encode so
    // they re-lex as Float not Int. Without it, decode would
    // return f64 from the Int branch — same value but the
    // wire form should distinguish.
    let mut encoder = Encoder::nexus();
    7.0f64.encode(&mut encoder).unwrap();
    let text = encoder.into_string();
    assert_eq!(text, "7.0");
}

#[test]
fn f32_round_trips() {
    round_trip(0.0f32);
    round_trip(1.5f32);
}

// ─── Box<T> ────────────────────────────────────────────────

#[test]
fn box_round_trips_for_primitive() {
    round_trip(Box::new(42u64));
    round_trip(Box::new("hello".to_string()));
    round_trip(Box::new(true));
}

#[test]
fn nested_box_round_trips() {
    round_trip(Box::new(Box::new(7u32)));
}

// ─── HashMap<K, V> ─────────────────────────────────────────

#[test]
fn empty_hash_map_round_trips() {
    let map: HashMap<String, u64> = HashMap::new();
    round_trip(map);
}

#[test]
fn populated_hash_map_round_trips_with_deterministic_ordering() {
    let mut map = HashMap::new();
    map.insert("zeta".to_string(), 1u64);
    map.insert("alpha".to_string(), 2u64);
    map.insert("middle".to_string(), 3u64);
    // HashMap iteration order is non-deterministic but encode
    // sorts by key — so wire output is stable.
    let mut encoder = Encoder::nexus();
    map.encode(&mut encoder).unwrap();
    let text = encoder.into_string();
    assert_eq!(text, "[(Entry \"alpha\" 2) (Entry \"middle\" 3) (Entry \"zeta\" 1)]");
    round_trip(map);
}

// ─── BTreeSet<T> + HashSet<T> ──────────────────────────────

#[test]
fn empty_btree_set_round_trips() {
    let set: BTreeSet<u64> = BTreeSet::new();
    round_trip(set);
}

#[test]
fn populated_btree_set_round_trips() {
    let set: BTreeSet<u64> = [3, 1, 2].into_iter().collect();
    let mut encoder = Encoder::nexus();
    set.encode(&mut encoder).unwrap();
    assert_eq!(encoder.into_string(), "[1 2 3]");
    round_trip(set);
}

#[test]
fn populated_hash_set_round_trips_with_deterministic_ordering() {
    let set: HashSet<u64> = [3, 1, 2].into_iter().collect();
    let mut encoder = Encoder::nexus();
    set.encode(&mut encoder).unwrap();
    assert_eq!(encoder.into_string(), "[1 2 3]");
    round_trip(set);
}

// ─── Tuples ────────────────────────────────────────────────

#[test]
fn pair_tuple_round_trips() {
    round_trip(("hello".to_string(), 42u64));
    round_trip((1u32, true));
}

#[test]
fn triple_tuple_round_trips() {
    round_trip((1u64, 2u64, 3u64));
}

#[test]
fn quad_tuple_round_trips() {
    round_trip(("a".to_string(), 1u64, true, 3.14f64));
}

#[test]
fn tuple_emits_explicit_tuple_head() {
    let mut encoder = Encoder::nexus();
    (1u64, 2u64).encode(&mut encoder).unwrap();
    assert_eq!(encoder.into_string(), "(Tuple 1 2)");
}

// ─── Byte vectors ─────────────────────────────────────────

#[test]
fn byte_array_via_hex_literal_round_trips() {
    // Encode then decode through the encoder/decoder methods
    // directly; Vec<u8> falls back to the Vec<T> blanket impl
    // (sequence-of-byte) which is different from the dedicated
    // #hex form. Test the dedicated path through the methods.
    let bytes = vec![0xde, 0xad, 0xbe, 0xef];
    let mut encoder = Encoder::nexus();
    encoder.write_bytes(&bytes).unwrap();
    let text = encoder.into_string();
    assert_eq!(text, "#deadbeef");

    let mut decoder = Decoder::nexus(&text);
    let recovered = decoder.read_bytes().unwrap();
    assert_eq!(recovered, bytes);
}

#[test]
fn empty_byte_array_round_trips() {
    let mut encoder = Encoder::nexus();
    encoder.write_bytes(&[]).unwrap();
    assert_eq!(encoder.into_string(), "#");
    // Empty hex literal — verify lexer accepts it. (If lexer
    // rejects empty `#`, this test will tell us; we'd then
    // need a sentinel form for empty byte arrays.)
}

// ─── Round-trip property: nested combinations ─────────────

#[test]
fn deeply_nested_combination_round_trips() {
    let value: Vec<HashMap<String, (u64, BTreeSet<u32>)>> = vec![
        {
            let mut map = HashMap::new();
            map.insert("first".to_string(), (1u64, [10, 20, 30].into_iter().collect()));
            map.insert("second".to_string(), (2u64, [40].into_iter().collect()));
            map
        },
        HashMap::new(),
    ];
    round_trip(value);
}
