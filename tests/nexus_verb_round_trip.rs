//! NexusVerb round-trip tests.
//!
//! Verifies the head-identifier dispatch: `(Node "User")` →
//! `AssertOperation::Node(Node { name })`, etc. Tests every
//! variant in a small toy verb enum, plus the
//! UnknownKindForVerb error path.

use nota_codec::{
    Decoder, Encoder, Error, NexusVerb, NotaDecode, NotaEncode, NotaRecord, NotaTransparent,
};

#[derive(NotaTransparent, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot(u64);

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub name: String,
}

#[derive(NotaRecord, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edge {
    pub from: Slot,
    pub to: Slot,
}

#[derive(NexusVerb, Debug, Clone, PartialEq, Eq)]
pub enum AssertOperation {
    Node(Node),
    Edge(Edge),
}

fn round_trip<T: NotaEncode + NotaDecode + PartialEq + std::fmt::Debug>(value: T, expected: &str) {
    let mut encoder = Encoder::nexus();
    value.encode(&mut encoder).unwrap();
    let text = encoder.into_string();
    assert_eq!(text, expected);

    let mut decoder = Decoder::nexus(&text);
    let recovered = T::decode(&mut decoder).unwrap();
    assert_eq!(value, recovered);
}

#[test]
fn assert_operation_dispatches_to_node_variant() {
    round_trip(
        AssertOperation::Node(Node { name: "User".to_string() }),
        "(Node \"User\")",
    );
}

#[test]
fn assert_operation_dispatches_to_edge_variant() {
    round_trip(
        AssertOperation::Edge(Edge { from: Slot(1), to: Slot(2) }),
        "(Edge 1 2)",
    );
}

#[test]
fn unknown_kind_returns_typed_error() {
    let mut decoder = Decoder::nexus("(Mystery 42)");
    let err = AssertOperation::decode(&mut decoder).unwrap_err();
    match err {
        Error::UnknownKindForVerb { verb, got } => {
            assert_eq!(verb, "AssertOperation");
            assert_eq!(got, "Mystery");
        }
        other => panic!("expected UnknownKindForVerb, got {other:?}"),
    }
}

#[test]
fn peek_does_not_consume_so_full_decode_works() {
    // This is the load-bearing test for the Decoder pushback
    // mechanism: peek_record_head reads `(` + identifier, then
    // pushes them back so the variant's full Node::decode reads
    // them again.
    let mut decoder = Decoder::nexus("(Node \"Alice\")");
    let result = AssertOperation::decode(&mut decoder).unwrap();
    assert_eq!(result, AssertOperation::Node(Node { name: "Alice".to_string() }));
}
