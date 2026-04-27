//! `Decoder<'input>` — typed reading from a token stream.
//!
//! Protocol surface for the derives in
//! [`nota-derive`](https://github.com/LiGoldragon/nota-derive).
//! Methods land here as the derives need them.

use crate::lexer::{Dialect, Lexer};

pub struct Decoder<'input> {
    #[allow(dead_code)]
    lexer: Lexer<'input>,
}

impl<'input> Decoder<'input> {
    /// Open a decoder over nexus-dialect input.
    pub fn nexus(input: &'input str) -> Self {
        Self {
            lexer: Lexer::with_dialect(input, Dialect::Nexus),
        }
    }

    /// Open a decoder over nota-dialect input.
    pub fn nota(input: &'input str) -> Self {
        Self {
            lexer: Lexer::with_dialect(input, Dialect::Nota),
        }
    }
}
