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

// ─── Primitive integers ─────────────────────────────────────

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

impl NotaEncode for u32 {
    fn encode(&self, encoder: &mut Encoder) -> Result<()> {
        encoder.write_u64(*self as u64)
    }
}

impl NotaDecode for u32 {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self> {
        decoder.read_u32()
    }
}

impl NotaEncode for u16 {
    fn encode(&self, encoder: &mut Encoder) -> Result<()> {
        encoder.write_u64(*self as u64)
    }
}

impl NotaDecode for u16 {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self> {
        decoder.read_u16()
    }
}

impl NotaEncode for u8 {
    fn encode(&self, encoder: &mut Encoder) -> Result<()> {
        encoder.write_u64(*self as u64)
    }
}

impl NotaDecode for u8 {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self> {
        decoder.read_u8()
    }
}

impl NotaEncode for i64 {
    fn encode(&self, encoder: &mut Encoder) -> Result<()> {
        encoder.write_i64(*self)
    }
}

impl NotaDecode for i64 {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self> {
        decoder.read_i64()
    }
}

impl NotaEncode for i32 {
    fn encode(&self, encoder: &mut Encoder) -> Result<()> {
        encoder.write_i64(*self as i64)
    }
}

impl NotaDecode for i32 {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self> {
        decoder.read_i32()
    }
}

impl NotaEncode for i16 {
    fn encode(&self, encoder: &mut Encoder) -> Result<()> {
        encoder.write_i64(*self as i64)
    }
}

impl NotaDecode for i16 {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self> {
        decoder.read_i16()
    }
}

impl NotaEncode for i8 {
    fn encode(&self, encoder: &mut Encoder) -> Result<()> {
        encoder.write_i64(*self as i64)
    }
}

impl NotaDecode for i8 {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self> {
        decoder.read_i8()
    }
}

// ─── Primitive floats ───────────────────────────────────────

impl NotaEncode for f64 {
    fn encode(&self, encoder: &mut Encoder) -> Result<()> {
        encoder.write_f64(*self)
    }
}

impl NotaDecode for f64 {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self> {
        decoder.read_f64()
    }
}

impl NotaEncode for f32 {
    fn encode(&self, encoder: &mut Encoder) -> Result<()> {
        encoder.write_f64(*self as f64)
    }
}

impl NotaDecode for f32 {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self> {
        decoder.read_f32()
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

// ─── HashMap<K, V> — same Entry-record wire form ───────────

impl<K, V> NotaEncode for std::collections::HashMap<K, V>
where
    K: NotaEncode + Ord + Eq + std::hash::Hash,
    V: NotaEncode,
{
    fn encode(&self, encoder: &mut Encoder) -> Result<()> {
        // Sort keys for deterministic wire output. Round-trip
        // guarantee: encode(decode(x)) == encode(x).
        let mut keys: Vec<&K> = self.keys().collect();
        keys.sort();
        encoder.start_seq()?;
        for key in keys {
            let value = self.get(key).expect("key from self.keys() must be in self");
            encoder.start_record("Entry")?;
            key.encode(encoder)?;
            value.encode(encoder)?;
            encoder.end_record()?;
        }
        encoder.end_seq()
    }
}

impl<K, V> NotaDecode for std::collections::HashMap<K, V>
where
    K: NotaDecode + Eq + std::hash::Hash,
    V: NotaDecode,
{
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self> {
        decoder.expect_seq_start()?;
        let mut map = std::collections::HashMap::new();
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

// ─── BTreeSet<T> + HashSet<T> — `[a b c]` sequence form ────

impl<T: NotaEncode> NotaEncode for std::collections::BTreeSet<T> {
    fn encode(&self, encoder: &mut Encoder) -> Result<()> {
        encoder.start_seq()?;
        for element in self {
            element.encode(encoder)?;
        }
        encoder.end_seq()
    }
}

impl<T: NotaDecode + Ord> NotaDecode for std::collections::BTreeSet<T> {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self> {
        decoder.expect_seq_start()?;
        let mut set = std::collections::BTreeSet::new();
        while !decoder.peek_is_seq_end()? {
            set.insert(T::decode(decoder)?);
        }
        decoder.expect_seq_end()?;
        Ok(set)
    }
}

impl<T> NotaEncode for std::collections::HashSet<T>
where
    T: NotaEncode + Ord + Eq + std::hash::Hash,
{
    fn encode(&self, encoder: &mut Encoder) -> Result<()> {
        // Sort for deterministic wire output.
        let mut elements: Vec<&T> = self.iter().collect();
        elements.sort();
        encoder.start_seq()?;
        for element in elements {
            element.encode(encoder)?;
        }
        encoder.end_seq()
    }
}

impl<T> NotaDecode for std::collections::HashSet<T>
where
    T: NotaDecode + Eq + std::hash::Hash,
{
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self> {
        decoder.expect_seq_start()?;
        let mut set = std::collections::HashSet::new();
        while !decoder.peek_is_seq_end()? {
            set.insert(T::decode(decoder)?);
        }
        decoder.expect_seq_end()?;
        Ok(set)
    }
}

// ─── Box<T> — transparent delegation ───────────────────────

impl<T: NotaEncode + ?Sized> NotaEncode for Box<T> {
    fn encode(&self, encoder: &mut Encoder) -> Result<()> {
        (**self).encode(encoder)
    }
}

impl<T: NotaDecode> NotaDecode for Box<T> {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self> {
        Ok(Box::new(T::decode(decoder)?))
    }
}

// ─── Tuples — `(Tuple a b …)` record form ──────────────────
//
// Encoded with an explicit `Tuple` head so the wire is
// unambiguous (a sequence-of-values would collide with a
// `Vec<T>` of the same shape).

impl<A: NotaEncode, B: NotaEncode> NotaEncode for (A, B) {
    fn encode(&self, encoder: &mut Encoder) -> Result<()> {
        encoder.start_record("Tuple")?;
        self.0.encode(encoder)?;
        self.1.encode(encoder)?;
        encoder.end_record()
    }
}

impl<A: NotaDecode, B: NotaDecode> NotaDecode for (A, B) {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self> {
        decoder.expect_record_head("Tuple")?;
        let first = A::decode(decoder)?;
        let second = B::decode(decoder)?;
        decoder.expect_record_end()?;
        Ok((first, second))
    }
}

impl<A: NotaEncode, B: NotaEncode, C: NotaEncode> NotaEncode for (A, B, C) {
    fn encode(&self, encoder: &mut Encoder) -> Result<()> {
        encoder.start_record("Tuple")?;
        self.0.encode(encoder)?;
        self.1.encode(encoder)?;
        self.2.encode(encoder)?;
        encoder.end_record()
    }
}

impl<A: NotaDecode, B: NotaDecode, C: NotaDecode> NotaDecode for (A, B, C) {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self> {
        decoder.expect_record_head("Tuple")?;
        let first = A::decode(decoder)?;
        let second = B::decode(decoder)?;
        let third = C::decode(decoder)?;
        decoder.expect_record_end()?;
        Ok((first, second, third))
    }
}

impl<A: NotaEncode, B: NotaEncode, C: NotaEncode, D: NotaEncode> NotaEncode for (A, B, C, D) {
    fn encode(&self, encoder: &mut Encoder) -> Result<()> {
        encoder.start_record("Tuple")?;
        self.0.encode(encoder)?;
        self.1.encode(encoder)?;
        self.2.encode(encoder)?;
        self.3.encode(encoder)?;
        encoder.end_record()
    }
}

impl<A: NotaDecode, B: NotaDecode, C: NotaDecode, D: NotaDecode> NotaDecode for (A, B, C, D) {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self> {
        decoder.expect_record_head("Tuple")?;
        let first = A::decode(decoder)?;
        let second = B::decode(decoder)?;
        let third = C::decode(decoder)?;
        let fourth = D::decode(decoder)?;
        decoder.expect_record_end()?;
        Ok((first, second, third, fourth))
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
