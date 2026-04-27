//! `Decoder<'input>` — typed reading from a token stream.
//!
//! Protocol surface for the derives in
//! [`nota-derive`](https://github.com/LiGoldragon/nota-derive).
//! The methods here are what the derive-emitted code calls into.

use crate::error::{Error, Result};
use crate::lexer::{Dialect, Lexer, Token};

pub struct Decoder<'input> {
    lexer: Lexer<'input>,
}

impl<'input> Decoder<'input> {
    /// Open a decoder over nexus-dialect input.
    pub fn nexus(input: &'input str) -> Self {
        Self { lexer: Lexer::with_dialect(input, Dialect::Nexus) }
    }

    /// Open a decoder over nota-dialect input.
    pub fn nota(input: &'input str) -> Self {
        Self { lexer: Lexer::with_dialect(input, Dialect::Nota) }
    }

    /// Read the next token, erroring on EOF.
    fn next_token(&mut self) -> Result<Token> {
        self.lexer
            .next_token()?
            .ok_or(Error::UnexpectedEnd { while_parsing: "value" })
    }

    /// Read a `u64`. Accepts a non-negative `Int` or any `UInt`
    /// that fits in `u64`.
    pub fn read_u64(&mut self) -> Result<u64> {
        match self.next_token()? {
            Token::Int(value) if value >= 0 && value <= u64::MAX as i128 => Ok(value as u64),
            Token::UInt(value) if value <= u64::MAX as u128 => Ok(value as u64),
            other => Err(Error::UnexpectedToken { expected: "u64 integer literal", got: other }),
        }
    }
}
