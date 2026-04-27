//! NotaRecord round-trip tests.
//!
//! Covers: empty unit-struct records (`(Ok)`), single-field
//! records (`(Node "User")`), multi-field records with mixed
//! primitive + enum + transparent-newtype field types
//! (`(Edge 100 200 DependsOn)`), nested records, and the
//! ExpectedRecordHead error path.

use nota_codec::{
    Decoder, Encoder, Error, NotaDecode, NotaEncode, NotaEnum, NotaRecord, NotaTransparent,
};

#[derive(NotaTransparent, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot(u64);

#[derive(NotaEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationKind {
    Flow,
    DependsOn,
    Contains,
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Ok {}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub name: String,
}

#[derive(NotaRecord, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edge {
    pub from: Slot,
    pub to: Slot,
    pub kind: RelationKind,
}

fn round_trip<T: NotaEncode + NotaDecode + PartialEq + std::fmt::Debug>(value: T, expected_text: &str) {
    let mut encoder = Encoder::nexus();
    value.encode(&mut encoder).unwrap();
    let text = encoder.into_string();
    assert_eq!(text, expected_text, "encode produced unexpected text");

    let mut decoder = Decoder::nexus(&text);
    let recovered = T::decode(&mut decoder).unwrap();
    assert_eq!(value, recovered, "decode did not round-trip the value");
}

#[test]
fn ok_unit_record_round_trips() {
    round_trip(Ok {}, "(Ok)");
}

#[test]
fn single_field_record_round_trips() {
    round_trip(Node { name: "User".to_string() }, "(Node \"User\")");
}

#[test]
fn multi_field_record_round_trips_with_mixed_field_types() {
    round_trip(
        Edge { from: Slot(100), to: Slot(200), kind: RelationKind::DependsOn },
        "(Edge 100 200 DependsOn)",
    );
}

#[test]
fn record_round_trips_in_nota_dialect_too() {
    let original = Edge { from: Slot(1), to: Slot(2), kind: RelationKind::Flow };
    let mut encoder = Encoder::nota();
    original.encode(&mut encoder).unwrap();
    let text = encoder.into_string();
    assert_eq!(text, "(Edge 1 2 Flow)");
    let mut decoder = Decoder::nota(&text);
    let recovered = Edge::decode(&mut decoder).unwrap();
    assert_eq!(original, recovered);
}

#[test]
fn wrong_record_head_returns_typed_error() {
    let mut decoder = Decoder::nexus("(Edge 100 200 Flow)");
    let err = Node::decode(&mut decoder).unwrap_err();
    match err {
        Error::ExpectedRecordHead { expected, got } => {
            assert_eq!(expected, "Node");
            assert_eq!(got, "Edge");
        }
        other => panic!("expected ExpectedRecordHead, got {other:?}"),
    }
}

#[test]
fn missing_opening_paren_returns_typed_error() {
    let mut decoder = Decoder::nexus("Node \"User\"");
    let err = Node::decode(&mut decoder).unwrap_err();
    match err {
        Error::UnexpectedToken { expected, .. } => {
            assert_eq!(expected, "`(` opening a record");
        }
        other => panic!("expected UnexpectedToken, got {other:?}"),
    }
}
