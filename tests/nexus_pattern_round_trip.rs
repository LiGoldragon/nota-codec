//! NexusPattern round-trip tests.
//!
//! Covers the three field forms (`Wildcard` / `Bind` / `Match`)
//! across single-field and multi-field query types, plus the
//! WrongBindName error path that fires when `@<name>` doesn't
//! match the schema field name.

use nota_codec::{
    Decoder, Encoder, Error, NexusPattern, NotaDecode, NotaEncode, NotaEnum, NotaRecord,
    NotaTransparent, PatternField,
};

#[derive(NotaTransparent, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot(u64);

#[derive(NotaEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationKind {
    Flow,
    DependsOn,
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub name: String,
}

#[derive(NexusPattern, Debug, Clone, PartialEq, Eq)]
#[nota(queries = "Node")]
pub struct NodeQuery {
    pub name: PatternField<String>,
}

#[derive(NexusPattern, Debug, Clone, PartialEq, Eq)]
#[nota(queries = "Edge")]
pub struct EdgeQuery {
    pub from: PatternField<Slot>,
    pub to: PatternField<Slot>,
    pub kind: PatternField<RelationKind>,
}

fn encode_text<T: NotaEncode>(value: &T) -> String {
    let mut encoder = Encoder::nexus();
    value.encode(&mut encoder).unwrap();
    encoder.into_string()
}

#[test]
fn wildcard_field_emits_underscore() {
    let q = NodeQuery { name: PatternField::Wildcard };
    assert_eq!(encode_text(&q), "(| Node _ |)");
}

#[test]
fn bind_field_emits_at_schema_name() {
    let q = NodeQuery { name: PatternField::Bind };
    assert_eq!(encode_text(&q), "(| Node @name |)");
}

#[test]
fn match_field_emits_inner_value() {
    let q = NodeQuery { name: PatternField::Match("User".to_string()) };
    assert_eq!(encode_text(&q), "(| Node \"User\" |)");
}

#[test]
fn three_field_query_round_trips_all_wildcard() {
    let original = EdgeQuery {
        from: PatternField::Wildcard,
        to: PatternField::Wildcard,
        kind: PatternField::Wildcard,
    };
    let text = encode_text(&original);
    let mut decoder = Decoder::nexus(&text);
    let recovered = EdgeQuery::decode(&mut decoder).unwrap();
    assert_eq!(original, recovered);
}

#[test]
fn three_field_query_round_trips_mixed_forms() {
    let original = EdgeQuery {
        from: PatternField::Match(Slot(100)),
        to: PatternField::Bind,
        kind: PatternField::Wildcard,
    };
    let text = encode_text(&original);
    let mut decoder = Decoder::nexus(&text);
    let recovered = EdgeQuery::decode(&mut decoder).unwrap();
    assert_eq!(original, recovered);
}

#[test]
fn wrong_bind_name_returns_typed_error() {
    // `@source` is not the schema field name `from` — should
    // produce WrongBindName carrying both names.
    let mut decoder = Decoder::nexus("(| Edge @source @to _ |)");
    let err = EdgeQuery::decode(&mut decoder).unwrap_err();
    match err {
        Error::WrongBindName { expected, got } => {
            assert_eq!(expected, "from");
            assert_eq!(got, "source");
        }
        other => panic!("expected WrongBindName, got {other:?}"),
    }
}
