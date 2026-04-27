//! Coverage for the three additions that signal types need:
//! `Option<T>` trailing-omission, `Vec<T>` sequences, and
//! `NexusVerb` struct-variants.

use nota_codec::{
    Decoder, Encoder, NexusVerb, NotaDecode, NotaEncode, NotaRecord, NotaTransparent,
};

#[derive(NotaTransparent, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot(u64);

#[derive(NotaTransparent, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Revision(u64);

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub name: String,
}

// ─── Option<T> ─────────────────────────────────────────────

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    pub slot: Slot,
    pub expected_rev: Option<Revision>,
}

#[test]
fn option_present_round_trips_with_trailing_value() {
    let edit = Edit { slot: Slot(100), expected_rev: Some(Revision(5)) };
    let mut encoder = Encoder::nexus();
    edit.encode(&mut encoder).unwrap();
    assert_eq!(encoder.into_string(), "(Edit 100 5)");

    let mut decoder = Decoder::nexus("(Edit 100 5)");
    let recovered = Edit::decode(&mut decoder).unwrap();
    assert_eq!(recovered, edit);
}

#[test]
fn option_absent_round_trips_with_no_trailing_value() {
    let edit = Edit { slot: Slot(100), expected_rev: None };
    let mut encoder = Encoder::nexus();
    edit.encode(&mut encoder).unwrap();
    assert_eq!(encoder.into_string(), "(Edit 100)");

    let mut decoder = Decoder::nexus("(Edit 100)");
    let recovered = Edit::decode(&mut decoder).unwrap();
    assert_eq!(recovered, edit);
}

// ─── Vec<T> ─────────────────────────────────────────────────

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Graph {
    pub title: String,
    pub nodes: Vec<Slot>,
}

#[test]
fn empty_vec_emits_empty_brackets() {
    let graph = Graph { title: "G".to_string(), nodes: vec![] };
    let mut encoder = Encoder::nexus();
    graph.encode(&mut encoder).unwrap();
    assert_eq!(encoder.into_string(), "(Graph \"G\" [])");

    let mut decoder = Decoder::nexus("(Graph \"G\" [])");
    assert_eq!(Graph::decode(&mut decoder).unwrap(), graph);
}

#[test]
fn vec_of_three_round_trips() {
    let graph = Graph { title: "G".to_string(), nodes: vec![Slot(1), Slot(2), Slot(3)] };
    let mut encoder = Encoder::nexus();
    graph.encode(&mut encoder).unwrap();
    assert_eq!(encoder.into_string(), "(Graph \"G\" [1 2 3])");

    let mut decoder = Decoder::nexus("(Graph \"G\" [1 2 3])");
    assert_eq!(Graph::decode(&mut decoder).unwrap(), graph);
}

// ─── NexusVerb struct-variant ───────────────────────────────

#[derive(NexusVerb, Debug, Clone, PartialEq, Eq)]
pub enum MutateOperation {
    Node {
        slot: Slot,
        new: Node,
        expected_rev: Option<Revision>,
    },
    Graph {
        slot: Slot,
        new: Graph,
        expected_rev: Option<Revision>,
    },
}

#[test]
fn struct_variant_with_present_optional_round_trips() {
    let mutation = MutateOperation::Node {
        slot: Slot(100),
        new: Node { name: "Alice".to_string() },
        expected_rev: Some(Revision(7)),
    };
    let mut encoder = Encoder::nexus();
    mutation.encode(&mut encoder).unwrap();
    assert_eq!(encoder.into_string(), "(Node 100 (Node \"Alice\") 7)");

    let mut decoder = Decoder::nexus("(Node 100 (Node \"Alice\") 7)");
    assert_eq!(MutateOperation::decode(&mut decoder).unwrap(), mutation);
}

#[test]
fn struct_variant_with_absent_optional_round_trips() {
    let mutation = MutateOperation::Node {
        slot: Slot(100),
        new: Node { name: "Alice".to_string() },
        expected_rev: None,
    };
    let mut encoder = Encoder::nexus();
    mutation.encode(&mut encoder).unwrap();
    assert_eq!(encoder.into_string(), "(Node 100 (Node \"Alice\"))");

    let mut decoder = Decoder::nexus("(Node 100 (Node \"Alice\"))");
    assert_eq!(MutateOperation::decode(&mut decoder).unwrap(), mutation);
}

#[test]
fn struct_variant_dispatches_to_graph_variant() {
    let mutation = MutateOperation::Graph {
        slot: Slot(200),
        new: Graph { title: "G".to_string(), nodes: vec![Slot(1)] },
        expected_rev: None,
    };
    let mut encoder = Encoder::nexus();
    mutation.encode(&mut encoder).unwrap();
    assert_eq!(encoder.into_string(), "(Graph 200 (Graph \"G\" [1]))");

    let mut decoder = Decoder::nexus("(Graph 200 (Graph \"G\" [1]))");
    assert_eq!(MutateOperation::decode(&mut decoder).unwrap(), mutation);
}
