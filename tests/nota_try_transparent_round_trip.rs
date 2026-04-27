//! NotaTryTransparent round-trip tests — validating newtype
//! pattern (the horizon-rs migration agent's primary need).

use nota_codec::{Decoder, Encoder, Error, NotaDecode, NotaEncode, NotaTryTransparent};

/// A wrapper around `String` that only accepts non-empty
/// values that look like a hex digest of exactly 8 chars
/// (chosen as a small validator so the test is fast).
#[derive(NotaTryTransparent, Debug, Clone, PartialEq, Eq)]
pub struct ShortHex(String);

#[derive(Debug, thiserror::Error)]
pub enum ShortHexError {
    #[error("must be exactly 8 chars, got {0}")]
    WrongLength(usize),
    #[error("must be hex digits only, got non-hex char `{0}`")]
    NonHexChar(char),
}

impl ShortHex {
    pub fn try_new(value: String) -> Result<Self, ShortHexError> {
        if value.len() != 8 {
            return Err(ShortHexError::WrongLength(value.len()));
        }
        for character in value.chars() {
            if !character.is_ascii_hexdigit() {
                return Err(ShortHexError::NonHexChar(character));
            }
        }
        Ok(Self(value))
    }
}

#[test]
fn valid_input_decodes_through_try_new() {
    let mut decoder = Decoder::nexus("\"deadbeef\"");
    let value = ShortHex::decode(&mut decoder).unwrap();
    assert_eq!(value, ShortHex::try_new("deadbeef".into()).unwrap());
}

#[test]
fn invalid_length_returns_typed_validation_error() {
    let mut decoder = Decoder::nexus("\"deadbe\""); // 6 chars, not 8
    let error = ShortHex::decode(&mut decoder).unwrap_err();
    match error {
        Error::Validation { type_name, message } => {
            assert_eq!(type_name, "ShortHex");
            assert!(message.contains("8 chars"), "message was: {message}");
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn invalid_chars_return_typed_validation_error() {
    let mut decoder = Decoder::nexus("\"xxxxxxxx\""); // 8 chars but non-hex
    let error = ShortHex::decode(&mut decoder).unwrap_err();
    match error {
        Error::Validation { type_name, message } => {
            assert_eq!(type_name, "ShortHex");
            assert!(message.contains("non-hex"), "message was: {message}");
        }
        other => panic!("expected Validation error, got {other:?}"),
    }
}

#[test]
fn round_trip_holds_for_valid_value() {
    let value = ShortHex::try_new("0123abcd".into()).unwrap();
    let mut encoder = Encoder::nexus();
    value.encode(&mut encoder).unwrap();
    let text = encoder.into_string();
    assert_eq!(text, "\"0123abcd\"");

    let mut decoder = Decoder::nexus(&text);
    assert_eq!(ShortHex::decode(&mut decoder).unwrap(), value);
}

#[test]
fn from_self_for_inner_is_emitted() {
    let value = ShortHex::try_new("0123abcd".into()).unwrap();
    let inner: String = value.into();
    assert_eq!(inner, "0123abcd");
}

#[test]
fn bare_ident_input_also_validates() {
    // `read_string` accepts bare idents now, so a bare hex
    // string parses too.
    let mut decoder = Decoder::nexus("deadbeef");
    let value = ShortHex::decode(&mut decoder).unwrap();
    assert_eq!(value, ShortHex::try_new("deadbeef".into()).unwrap());
}
