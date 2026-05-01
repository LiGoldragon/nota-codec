//! `PatternField<T>` — a slot in a `*Query` record kind.
//!
//! Each field of a `*Query` type is a `PatternField<T>` where
//! `T` is the corresponding field type on the data record. A
//! pattern field is one of:
//!
//! - `Wildcard` — match any value (`_` in nexus text)
//! - `Bind` — match any value and capture it under the schema
//!   field's name. The bind name is *implicit from the field's
//!   position* in the `*Query` record; no string is carried in
//!   the IR. The wire form is `@<schema-field-name>`.
//! - `Match(value)` — match the literal value of type `T`
//!
//! See
//! nexus/spec/grammar.md
//! for the auto-name-from-schema rule that requires `@name` to
//! match the schema field name at the position it appears.

use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

use crate::decoder::Decoder;
use crate::encoder::Encoder;
use crate::error::Result;
use crate::traits::{NotaDecode, NotaEncode};

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PatternField<T> {
    Wildcard,
    Bind,
    Match(T),
}

// `PatternField<T>` is encoded / decoded by the protocol
// methods on `Encoder` / `Decoder`, not by directly
// implementing `NotaEncode` / `NotaDecode`. The reason: the
// bind name is contextual — it comes from the schema field
// position, which only the surrounding `NexusPattern` derive
// knows. A field-position-unaware impl would have no way to
// validate `@name` against the expected schema field name.
//
// To still let `PatternField<T>` participate in regular
// `NotaEncode` / `NotaDecode` plumbing (e.g. when stored
// inside non-pattern records), we provide these conservative
// blanket impls that round-trip Wildcard and Match but reject
// Bind — Bind only makes sense at a known schema-field
// position.

impl<T: NotaEncode> NotaEncode for PatternField<T> {
    fn encode(&self, encoder: &mut Encoder) -> Result<()> {
        match self {
            PatternField::Wildcard => encoder.write_wildcard(),
            PatternField::Bind => Err(crate::Error::PatternBindOutOfContext),
            PatternField::Match(value) => value.encode(encoder),
        }
    }
}

impl<T: NotaDecode> NotaDecode for PatternField<T> {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self> {
        // Without a known schema field name, decoding `@name`
        // is undecidable — defer to the contextual
        // `decode_pattern_field` path. Here we only handle
        // `_` (wildcard) and a literal value.
        if decoder.peek_is_wildcard()? {
            decoder.consume_wildcard()?;
            Ok(PatternField::Wildcard)
        } else if decoder.peek_is_bind_marker()? {
            Err(crate::Error::PatternBindOutOfContext)
        } else {
            Ok(PatternField::Match(T::decode(decoder)?))
        }
    }
}
