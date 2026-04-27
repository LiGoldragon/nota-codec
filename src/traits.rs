//! `NotaEncode` + `NotaDecode` traits + blanket impls for
//! primitives and standard containers.
//!
//! New blanket impls land in this file; the per-derive codegen
//! in `nota-derive` calls into the `Decoder`/`Encoder` protocol
//! methods, which in turn use these blanket impls for primitive
//! field types.

use crate::decoder::Decoder;
use crate::encoder::Encoder;
use crate::error::Result;

/// A value that knows how to write itself as nota or nexus
/// text via an [`Encoder`].
pub trait NotaEncode {
    fn encode(&self, encoder: &mut Encoder) -> Result<()>;
}

/// A value that knows how to read itself from nota or nexus
/// text via a [`Decoder`].
pub trait NotaDecode: Sized {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self>;
}

// ─── Primitives ─────────────────────────────────────────────

impl NotaEncode for u64 {
    fn encode(&self, encoder: &mut Encoder) -> Result<()> {
        encoder.write_u64(*self)
    }
}

impl NotaDecode for u64 {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self> {
        decoder.read_u64()
    }
}

impl NotaEncode for String {
    fn encode(&self, encoder: &mut Encoder) -> Result<()> {
        encoder.write_string(self)
    }
}

impl NotaDecode for String {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self> {
        decoder.read_string()
    }
}

impl NotaEncode for bool {
    fn encode(&self, encoder: &mut Encoder) -> Result<()> {
        encoder.write_bool(*self)
    }
}

impl NotaDecode for bool {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self> {
        decoder.read_bool()
    }
}

// ─── Option<T> — explicit `None` ident, always emitted ────
//
// **Encode** always emits an explicit `None` identifier for
// `None`; the inner value for `Some`. Symmetric with decode —
// no asymmetry between writer and reader.
//
// **Decode** accepts BOTH explicit `None` ident (canonical;
// used by goldragon-style datom files) AND trailing-omission
// (record ends `)` / `|)` before the field is reached) — the
// latter is a backward-compat path so older data files that
// elided trailing optionals still parse.
//
// **Why not trailing-omission for encode?** Because mid-record
// `Option<T>` fields can't distinguish "absent" from "next
// field's value" at decode time. Always-explicit `None` is
// position-independent, so `Option<T>` may appear anywhere in
// a record without ambiguity.
//
// **Ambiguity to know about:** `Option<String>` can't
// distinguish the literal string `"None"` from the absent
// value when the literal is bare. Quote the literal (`"None"`)
// to disambiguate.

impl<T: NotaEncode> NotaEncode for Option<T> {
    fn encode(&self, encoder: &mut Encoder) -> Result<()> {
        match self {
            Some(value) => value.encode(encoder),
            None => encoder.write_pascal_identifier("None"),
        }
    }
}

impl<T: NotaDecode> NotaDecode for Option<T> {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self> {
        if decoder.peek_is_explicit_none()? {
            decoder.consume_explicit_none()?;
            Ok(None)
        } else if decoder.peek_is_record_end()? {
            Ok(None)
        } else {
            Ok(Some(T::decode(decoder)?))
        }
    }
}

// ─── BTreeMap<K, V> — `[(Entry key value) (Entry key value)]` ─

impl<K, V> NotaEncode for std::collections::BTreeMap<K, V>
where
    K: NotaEncode,
    V: NotaEncode,
{
    fn encode(&self, encoder: &mut Encoder) -> Result<()> {
        encoder.start_seq()?;
        for (key, value) in self {
            encoder.start_record("Entry")?;
            key.encode(encoder)?;
            value.encode(encoder)?;
            encoder.end_record()?;
        }
        encoder.end_seq()
    }
}

impl<K, V> NotaDecode for std::collections::BTreeMap<K, V>
where
    K: NotaDecode + Ord,
    V: NotaDecode,
{
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self> {
        decoder.expect_seq_start()?;
        let mut map = std::collections::BTreeMap::new();
        while !decoder.peek_is_seq_end()? {
            decoder.expect_record_head("Entry")?;
            let key = K::decode(decoder)?;
            let value = V::decode(decoder)?;
            decoder.expect_record_end()?;
            map.insert(key, value);
        }
        decoder.expect_seq_end()?;
        Ok(map)
    }
}

// ─── Vec<T> — `[a b c]` sequence form ───────────────────────

impl<T: NotaEncode> NotaEncode for Vec<T> {
    fn encode(&self, encoder: &mut Encoder) -> Result<()> {
        encoder.start_seq()?;
        for element in self {
            element.encode(encoder)?;
        }
        encoder.end_seq()
    }
}

impl<T: NotaDecode> NotaDecode for Vec<T> {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self> {
        decoder.expect_seq_start()?;
        let mut elements = Vec::new();
        while !decoder.peek_is_seq_end()? {
            elements.push(T::decode(decoder)?);
        }
        decoder.expect_seq_end()?;
        Ok(elements)
    }
}

// TODO: i64, f64 as the derives that need them land.
