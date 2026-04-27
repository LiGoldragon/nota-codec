//! `NotaEncode` + `NotaDecode` traits + blanket impls for
//! primitives and standard containers.

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

// Blanket impls — populated as the protocol methods on
// Decoder/Encoder land. Stubs for now so the trait surface
// is real.

// TODO: u64, i64, f64, bool, String, Vec<T>, Option<T>.
