//! NotaTransparent round-trip tests.
//!
//! Verifies the derive emits `NotaEncode`, `NotaDecode`, and the
//! two `From` conversions; that the wrapped value round-trips
//! through the encoder/decoder without the wrapper appearing in
//! the wire form; and that the same text decodes to the same
//! value across both dialects.

use nota_codec::{Decoder, Encoder, NotaDecode, NotaEncode, NotaTransparent};

#[derive(NotaTransparent, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot(u64);

#[derive(NotaTransparent, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Revision(u64);

#[test]
fn slot_encodes_as_bare_integer() {
    let slot = Slot::from(42u64);
    let mut encoder = Encoder::nexus();
    slot.encode(&mut encoder).unwrap();
    assert_eq!(encoder.into_string(), "42");
}

#[test]
fn slot_decodes_from_bare_integer() {
    let mut decoder = Decoder::nexus("42");
    let slot = Slot::decode(&mut decoder).unwrap();
    assert_eq!(slot, Slot::from(42u64));
}

#[test]
fn slot_round_trips_in_nexus_dialect() {
    let original = Slot::from(100u64);
    let mut encoder = Encoder::nexus();
    original.encode(&mut encoder).unwrap();
    let text = encoder.into_string();
    let mut decoder = Decoder::nexus(&text);
    let recovered = Slot::decode(&mut decoder).unwrap();
    assert_eq!(original, recovered);
}

#[test]
fn slot_round_trips_in_nota_dialect() {
    let original = Slot::from(100u64);
    let mut encoder = Encoder::nota();
    original.encode(&mut encoder).unwrap();
    let text = encoder.into_string();
    let mut decoder = Decoder::nota(&text);
    let recovered = Slot::decode(&mut decoder).unwrap();
    assert_eq!(original, recovered);
}

#[test]
fn from_conversions_are_emitted() {
    // NotaTransparent emits both directions; this test exists
    // as a compile-time check that future refactors don't drop
    // either impl.
    let slot: Slot = 7u64.into();
    let inner: u64 = slot.into();
    assert_eq!(inner, 7u64);
}

#[test]
fn distinct_newtypes_do_not_share_text_representation_through_the_type_system() {
    // Slot and Revision both encode as bare u64 in the wire,
    // but the types are distinct — this test exists to remind
    // future readers that the *position* in the schema (which
    // type the decoder is asked to read) is what disambiguates
    // them, not anything in the wire bytes.
    let slot = Slot::from(5u64);
    let revision = Revision::from(5u64);

    let mut e1 = Encoder::nexus();
    slot.encode(&mut e1).unwrap();
    let mut e2 = Encoder::nexus();
    revision.encode(&mut e2).unwrap();

    assert_eq!(e1.into_string(), e2.into_string()); // same text
    // …but you can't assign one to the other in Rust:
    let _: u64 = slot.into();
    let _: u64 = revision.into();
}
