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

// TODO: i64, f64, bool, Vec<T>, Option<T> as the derives that
// need them land.
