//! nota-codec — typed `Decoder` + `Encoder` for the
//! [nota](https://github.com/LiGoldragon/nota) and
//! [nexus](https://github.com/LiGoldragon/nexus) text dialects.
//!
//! See [`README.md`](https://github.com/LiGoldragon/nota-codec)
//! for the high-level shape and
//! [mentci/reports/099](https://github.com/LiGoldragon/mentci/blob/main/reports/099-custom-derive-design-2026-04-27.md)
//! for the design.

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
pub use nota_derive::{NexusPattern, NexusVerb, NotaEnum, NotaRecord, NotaTransparent};
