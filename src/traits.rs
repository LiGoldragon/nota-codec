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

// ─── Option<T> — trailing-omission ──────────────────────────
//
// `Some(value)` encodes as the inner value. `None` writes
// nothing. The derive enforces that `Option<T>` fields appear
// only at the end of records — see reports/099 §6.1 for the
// design rationale.

impl<T: NotaEncode> NotaEncode for Option<T> {
    fn encode(&self, encoder: &mut Encoder) -> Result<()> {
        match self {
            Some(value) => value.encode(encoder),
            None => Ok(()),
        }
    }
}

impl<T: NotaDecode> NotaDecode for Option<T> {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self> {
        if decoder.peek_is_record_end()? {
            Ok(None)
        } else {
            Ok(Some(T::decode(decoder)?))
        }
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

// TODO: i64, f64, bool as the derives that need them land.
