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

    /// Read a PascalCase identifier (the wire form of a unit-
    /// variant enum value or a record-head name). Returns the
    /// identifier text; the caller matches it against its
    /// expected variant set.
    pub fn read_pascal_identifier(&mut self) -> Result<String> {
        match self.next_token()? {
            Token::Ident(name) if crate::lexer::is_pascal_case(&name) => Ok(name),
            other => Err(Error::UnexpectedToken {
                expected: "PascalCase identifier",
                got: other,
            }),
        }
    }

    /// Read a quoted string literal.
    pub fn read_string(&mut self) -> Result<String> {
        match self.next_token()? {
            Token::Str(value) => Ok(value),
            other => Err(Error::UnexpectedToken {
                expected: "string literal",
                got: other,
            }),
        }
    }

    // ─── Record bracketing ──────────────────────────────────

    /// Expect `(Name`, consuming both tokens. Errors if either
    /// the opening paren is missing or the head identifier
    /// doesn't match `expected`.
    pub fn expect_record_head(&mut self, expected: &'static str) -> Result<()> {
        match self.next_token()? {
            Token::LParen => {}
            other => {
                return Err(Error::UnexpectedToken {
                    expected: "`(` opening a record",
                    got: other,
                });
            }
        }
        let head = self.read_pascal_identifier()?;
        if head != expected {
            return Err(Error::ExpectedRecordHead { expected, got: head });
        }
        Ok(())
    }

    /// Expect `)` closing the current record.
    pub fn expect_record_end(&mut self) -> Result<()> {
        match self.next_token()? {
            Token::RParen => Ok(()),
            other => Err(Error::UnexpectedToken {
                expected: "`)` closing a record",
                got: other,
            }),
        }
    }
}
