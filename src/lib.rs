//! nota-codec — typed `Decoder` + `Encoder` for the
//! nota and
//! nexus text dialects.
//!
//! See `README.md`
//! for the high-level shape; `ARCHITECTURE.md`
//! for the per-module roles.

pub mod decoder;
pub mod encoder;
pub mod error;
pub mod lexer;
pub mod pattern_field;
pub mod traits;

pub use decoder::Decoder;
pub use encoder::Encoder;
pub use error::{Error, Result};
pub use lexer::{Dialect, Lexer, Token};
pub use pattern_field::PatternField;
pub use traits::{NotaDecode, NotaEncode};

// Re-export derives so users only depend on this crate.
pub use nota_derive::{
    NexusPattern, NexusVerb, NotaEnum, NotaRecord, NotaTransparent, NotaTryTransparent,
};
