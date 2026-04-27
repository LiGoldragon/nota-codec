//! NotaEnum round-trip tests.
//!
//! Verifies the derive emits `NotaEncode` + `NotaDecode` that
//! round-trip every unit variant through its PascalCase
//! identifier in both dialects, and that an unknown identifier
//! produces `Error::UnknownVariant` carrying the enum name +
//! offending text.

use nota_codec::{Decoder, Encoder, Error, NotaDecode, NotaEncode, NotaEnum};

#[derive(NotaEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cardinality {
    One,
    Many,
    Optional,
}

#[derive(NotaEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationKind {
    Flow,
    DependsOn,
    Contains,
    References,
    Produces,
    Consumes,
    Calls,
    Implements,
    IsA,
}

fn round_trip<T: NotaEncode + NotaDecode + PartialEq + std::fmt::Debug>(value: T) {
    let mut encoder = Encoder::nexus();
    value.encode(&mut encoder).unwrap();
    let text = encoder.into_string();
    let mut decoder = Decoder::nexus(&text);
    let recovered = T::decode(&mut decoder).unwrap();
    assert_eq!(value, recovered);
}

#[test]
fn cardinality_one_encodes_as_its_identifier() {
    let mut encoder = Encoder::nexus();
    Cardinality::One.encode(&mut encoder).unwrap();
    assert_eq!(encoder.into_string(), "One");
}

#[test]
fn cardinality_decodes_from_its_identifier() {
    let mut decoder = Decoder::nexus("Many");
    assert_eq!(Cardinality::decode(&mut decoder).unwrap(), Cardinality::Many);
}

#[test]
fn every_cardinality_variant_round_trips() {
    round_trip(Cardinality::One);
    round_trip(Cardinality::Many);
    round_trip(Cardinality::Optional);
}

#[test]
fn every_relation_kind_round_trips() {
    for kind in [
        RelationKind::Flow,
        RelationKind::DependsOn,
        RelationKind::Contains,
        RelationKind::References,
        RelationKind::Produces,
        RelationKind::Consumes,
        RelationKind::Calls,
        RelationKind::Implements,
        RelationKind::IsA,
    ] {
        round_trip(kind);
    }
}

#[test]
fn unknown_identifier_returns_typed_error() {
    let mut decoder = Decoder::nexus("Maybe");
    let err = Cardinality::decode(&mut decoder).unwrap_err();
    match err {
        Error::UnknownVariant { enum_name, got } => {
            assert_eq!(enum_name, "Cardinality");
            assert_eq!(got, "Maybe");
        }
        other => panic!("expected UnknownVariant, got {other:?}"),
    }
}

#[test]
fn round_trips_in_nota_dialect_too() {
    let original = RelationKind::DependsOn;
    let mut encoder = Encoder::nota();
    original.encode(&mut encoder).unwrap();
    let text = encoder.into_string();
    let mut decoder = Decoder::nota(&text);
    let recovered = RelationKind::decode(&mut decoder).unwrap();
    assert_eq!(original, recovered);
}
