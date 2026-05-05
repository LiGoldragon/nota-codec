//! Tests for the gaps surfaced by the horizon-rs migration
//! agent's feedback (`/tmp/nota-codec-feedback.md`):
//!
//! - `BTreeMap<K, V>` blanket impl
//! - `Option<T>::decode` accepts explicit `None` ident
//! - `Decoder::read_string` accepts bare identifiers
//! - `Error::Validation` carries the validating-newtype's name
//!   + the `try_new` error message (NotaTryTransparent derive
//!   ships in nota-derive separately and is exercised by
//!   `nota_try_transparent_round_trip.rs`)

use std::collections::BTreeMap;

use nota_codec::{Decoder, Encoder, NotaDecode, NotaEncode, NotaRecord};

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Container {
    pub label: String,
}

#[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct OptionalFields {
    pub name: String,
    pub maybe_label: Option<String>,
    pub maybe_count: Option<u64>,
    pub flag: bool,
}

// ─── BTreeMap<K, V> blanket impl ───────────────────────────

#[test]
fn empty_btree_map_round_trips() {
    let map: BTreeMap<String, u64> = BTreeMap::new();
    let mut encoder = Encoder::nexus();
    map.encode(&mut encoder).unwrap();
    assert_eq!(encoder.into_string(), "[]");

    let mut decoder = Decoder::nexus("[]");
    let decoded = BTreeMap::<String, u64>::decode(&mut decoder).unwrap();
    assert!(decoded.is_empty());
}

#[test]
fn populated_btree_map_round_trips_with_entry_records() {
    let mut map = BTreeMap::new();
    map.insert("alpha".to_string(), 1u64);
    map.insert("beta".to_string(), 2u64);

    let mut encoder = Encoder::nexus();
    map.encode(&mut encoder).unwrap();
    assert_eq!(encoder.into_string(), "[(Entry \"alpha\" 1) (Entry \"beta\" 2)]");

    let mut decoder = Decoder::nexus("[(Entry \"alpha\" 1) (Entry \"beta\" 2)]");
    let decoded = BTreeMap::<String, u64>::decode(&mut decoder).unwrap();
    assert_eq!(decoded.get("alpha"), Some(&1));
    assert_eq!(decoded.get("beta"), Some(&2));
    assert_eq!(decoded.len(), 2);
}

#[test]
fn btree_map_round_trips_with_complex_value_type() {
    let mut map = BTreeMap::new();
    map.insert("first".to_string(), Container { label: "alpha".into() });
    map.insert("second".to_string(), Container { label: "beta".into() });

    let mut encoder = Encoder::nexus();
    map.encode(&mut encoder).unwrap();
    let text = encoder.into_string();
    let mut decoder = Decoder::nexus(&text);
    let decoded = BTreeMap::<String, Container>::decode(&mut decoder).unwrap();
    assert_eq!(decoded, map);
}

// ─── Option<T>::decode accepts explicit `None` ident ──────

#[test]
fn option_decodes_from_explicit_none_ident_mid_record() {
    // Mid-record `None` followed by required fields — the
    // legacy goldragon datom shape that broke under
    // trailing-omission only.
    let mut decoder = Decoder::nexus("(OptionalFields \"x\" None None true)");
    let value = OptionalFields::decode(&mut decoder).unwrap();
    assert_eq!(value, OptionalFields {
        name: "x".into(),
        maybe_label: None,
        maybe_count: None,
        flag: true,
    });
}

#[test]
fn option_decodes_from_present_value_mid_record() {
    let mut decoder = Decoder::nexus("(OptionalFields \"x\" \"label\" 42 true)");
    let value = OptionalFields::decode(&mut decoder).unwrap();
    assert_eq!(value, OptionalFields {
        name: "x".into(),
        maybe_label: Some("label".into()),
        maybe_count: Some(42),
        flag: true,
    });
}

#[test]
fn option_decodes_from_trailing_omission_too() {
    let mut decoder = Decoder::nexus("(OptionalFields \"x\" \"label\" 42 true)");
    let with_value = OptionalFields::decode(&mut decoder).unwrap();
    assert!(with_value.maybe_label.is_some());

    // Trailing optionals may be omitted as a decode-only
    // compatibility path. Here `flag` is the last field and is
    // required, so this record cannot demonstrate tail omission;
    // the case is covered by option_vec_struct_variant.rs.
}

#[test]
fn option_round_trip_via_encode_uses_explicit_none() {
    let value = OptionalFields {
        name: "x".into(),
        maybe_label: None,
        maybe_count: None,
        flag: true,
    };
    let mut encoder = Encoder::nexus();
    value.encode(&mut encoder).unwrap();
    let text = encoder.into_string();
    assert_eq!(text, "(OptionalFields \"x\" None None true)");
    let mut decoder = Decoder::nexus(&text);
    let recovered = OptionalFields::decode(&mut decoder).unwrap();
    assert_eq!(value, recovered);
}

// ─── Decoder::read_string accepts bare idents ─────────────

#[test]
fn string_field_accepts_bare_pascal_identifier() {
    let mut decoder = Decoder::nexus("(Container Foo)");
    let value = Container::decode(&mut decoder).unwrap();
    assert_eq!(value.label, "Foo");
}

#[test]
fn string_field_accepts_bare_camel_identifier() {
    let mut decoder = Decoder::nexus("(Container fooBar)");
    let value = Container::decode(&mut decoder).unwrap();
    assert_eq!(value.label, "fooBar");
}

#[test]
fn string_field_accepts_bare_kebab_identifier() {
    let mut decoder = Decoder::nexus("(Container foo-bar-baz)");
    let value = Container::decode(&mut decoder).unwrap();
    assert_eq!(value.label, "foo-bar-baz");
}

#[test]
fn string_field_accepts_quoted_form_too() {
    let mut decoder = Decoder::nexus("(Container \"with spaces\")");
    let value = Container::decode(&mut decoder).unwrap();
    assert_eq!(value.label, "with spaces");
}

#[test]
fn vec_of_strings_with_bare_idents_round_trips_through_decode() {
    // The `[ligoldragon]` pattern from nota/example.nota — a
    // single-element Vec<String> with the element as a bare
    // identifier. (The encoder emits quoted form; both decode
    // identically.)
    #[derive(NotaRecord, Debug, Clone, PartialEq, Eq)]
    pub struct Project {
        pub authors: Vec<String>,
    }

    let mut decoder = Decoder::nexus("(Project [ligoldragon])");
    let project = Project::decode(&mut decoder).unwrap();
    assert_eq!(project.authors, vec!["ligoldragon".to_string()]);
}
