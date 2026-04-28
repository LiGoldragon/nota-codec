//! `Decoder<'input>` — typed reading from a token stream.
//!
//! Protocol surface for the derives in
//! [`nota-derive`](https://github.com/LiGoldragon/nota-derive).
//! The methods here are what the derive-emitted code calls into.

use std::collections::VecDeque;

use crate::error::{Error, Result};
use crate::lexer::{Dialect, Lexer, Token};

pub struct Decoder<'input> {
    lexer: Lexer<'input>,
    /// Tokens that were lexed but pushed back so a later
    /// `next_token` will replay them. Holds at most a few
    /// tokens at a time (currently used by `peek_record_head`
    /// to look two tokens ahead without committing).
    pushback: VecDeque<Token>,
}

impl<'input> Decoder<'input> {
    /// Open a decoder over nexus-dialect input.
    pub fn nexus(input: &'input str) -> Self {
        Self {
            lexer: Lexer::with_dialect(input, Dialect::Nexus),
            pushback: VecDeque::new(),
        }
    }

    /// Open a decoder over nota-dialect input.
    pub fn nota(input: &'input str) -> Self {
        Self {
            lexer: Lexer::with_dialect(input, Dialect::Nota),
            pushback: VecDeque::new(),
        }
    }

    /// Read the next token, erroring on EOF.
    fn next_token(&mut self) -> Result<Token> {
        if let Some(token) = self.pushback.pop_front() {
            return Ok(token);
        }
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

    /// Read an unsigned integer that fits in `u32`.
    pub fn read_u32(&mut self) -> Result<u32> {
        let value = self.read_u64()?;
        u32::try_from(value).map_err(|_| Error::IntegerOutOfRange {
            target: "u32",
            value: value as i128,
        })
    }

    /// Read an unsigned integer that fits in `u16`.
    pub fn read_u16(&mut self) -> Result<u16> {
        let value = self.read_u64()?;
        u16::try_from(value).map_err(|_| Error::IntegerOutOfRange {
            target: "u16",
            value: value as i128,
        })
    }

    /// Read an unsigned integer that fits in `u8`.
    pub fn read_u8(&mut self) -> Result<u8> {
        let value = self.read_u64()?;
        u8::try_from(value).map_err(|_| Error::IntegerOutOfRange {
            target: "u8",
            value: value as i128,
        })
    }

    /// Read a signed integer that fits in `i64`. Accepts
    /// `Int` in range; `UInt` only if it fits as a positive
    /// `i64`.
    pub fn read_i64(&mut self) -> Result<i64> {
        match self.next_token()? {
            Token::Int(value) if value >= i64::MIN as i128 && value <= i64::MAX as i128 => {
                Ok(value as i64)
            }
            Token::UInt(value) if value <= i64::MAX as u128 => Ok(value as i64),
            other => Err(Error::UnexpectedToken { expected: "i64 integer literal", got: other }),
        }
    }

    /// Read a signed integer that fits in `i32`.
    pub fn read_i32(&mut self) -> Result<i32> {
        let value = self.read_i64()?;
        i32::try_from(value).map_err(|_| Error::IntegerOutOfRange {
            target: "i32",
            value: value as i128,
        })
    }

    /// Read a signed integer that fits in `i16`.
    pub fn read_i16(&mut self) -> Result<i16> {
        let value = self.read_i64()?;
        i16::try_from(value).map_err(|_| Error::IntegerOutOfRange {
            target: "i16",
            value: value as i128,
        })
    }

    /// Read a signed integer that fits in `i8`.
    pub fn read_i8(&mut self) -> Result<i8> {
        let value = self.read_i64()?;
        i8::try_from(value).map_err(|_| Error::IntegerOutOfRange {
            target: "i8",
            value: value as i128,
        })
    }

    /// Read a 64-bit float. Accepts `Float` directly; an
    /// integer literal also decodes (cast to `f64`).
    pub fn read_f64(&mut self) -> Result<f64> {
        match self.next_token()? {
            Token::Float(value) => Ok(value),
            Token::Int(value) => Ok(value as f64),
            Token::UInt(value) => Ok(value as f64),
            other => Err(Error::UnexpectedToken { expected: "float or integer literal", got: other }),
        }
    }

    /// Read a 32-bit float. Same shape as `read_f64`; cast to
    /// `f32` after reading.
    pub fn read_f32(&mut self) -> Result<f32> {
        Ok(self.read_f64()? as f32)
    }

    /// Read a byte vector (`#hex…` literal).
    pub fn read_bytes(&mut self) -> Result<Vec<u8>> {
        match self.next_token()? {
            Token::Bytes(bytes) => Ok(bytes),
            other => Err(Error::UnexpectedToken { expected: "byte literal `#…`", got: other }),
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

    /// Read a string value. Accepts either a quoted string
    /// literal (`"foo"`) or a bare identifier (`foo`,
    /// `kebab-case`, `PascalCase`) per the nota grammar's
    /// "bare idents where strings are expected" rule.
    ///
    /// Note the ambiguity around `Option<String>` and the
    /// literal value `"None"`: with bare-ident-as-string in
    /// effect, a wire `None` decodes as `PatternField::None`
    /// when the surrounding type is `Option<String>` (the
    /// outer `Option::decode` peeks for explicit `None`
    /// first). To round-trip the literal string `"None"`,
    /// quote it.
    pub fn read_string(&mut self) -> Result<String> {
        match self.next_token()? {
            Token::Str(value) => Ok(value),
            Token::Ident(name) => Ok(name),
            other => Err(Error::UnexpectedToken {
                expected: "string literal or bare identifier",
                got: other,
            }),
        }
    }

    /// Read a `bool` literal (`true` or `false` keyword).
    pub fn read_bool(&mut self) -> Result<bool> {
        match self.next_token()? {
            Token::Bool(value) => Ok(value),
            other => Err(Error::UnexpectedToken {
                expected: "`true` or `false`",
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

    // ─── Pattern bracketing (nexus-only) ────────────────────

    /// Expect `(| Name`, consuming all three tokens. Errors
    /// if the opening pattern delimiter is missing or the head
    /// identifier doesn't match `expected`.
    pub fn expect_pattern_record_head(&mut self, expected: &'static str) -> Result<()> {
        match self.next_token()? {
            Token::LParenPipe => {}
            other => {
                return Err(Error::UnexpectedToken {
                    expected: "`(|` opening a pattern record",
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

    /// Expect `|)` closing the current pattern record.
    pub fn expect_pattern_record_end(&mut self) -> Result<()> {
        match self.next_token()? {
            Token::RParenPipe => Ok(()),
            other => Err(Error::UnexpectedToken {
                expected: "`|)` closing a pattern record",
                got: other,
            }),
        }
    }

    /// Decode a `PatternField<T>` at a known schema field
    /// position. `expected_bind_name` is the schema field name;
    /// if the input is `@<name>` and `<name>` does not match,
    /// returns `Error::WrongBindName`.
    pub fn decode_pattern_field<T: crate::traits::NotaDecode>(
        &mut self,
        expected_bind_name: &'static str,
    ) -> Result<crate::pattern_field::PatternField<T>> {
        if self.peek_is_wildcard()? {
            self.consume_wildcard()?;
            Ok(crate::pattern_field::PatternField::Wildcard)
        } else if self.peek_is_bind_marker()? {
            self.next_token()?; // consume @
            let bind_name = match self.next_token()? {
                Token::Ident(name) if crate::lexer::is_lowercase_identifier(&name) => name,
                other => {
                    return Err(Error::UnexpectedToken {
                        expected: "lowercase bind-name identifier",
                        got: other,
                    });
                }
            };
            if bind_name != expected_bind_name {
                return Err(Error::WrongBindName {
                    expected: expected_bind_name,
                    got: bind_name,
                });
            }
            Ok(crate::pattern_field::PatternField::Bind)
        } else {
            Ok(crate::pattern_field::PatternField::Match(T::decode(self)?))
        }
    }

    /// Helper for `PatternField`'s out-of-context `NotaDecode`
    /// impl: returns true if the next token is the wildcard
    /// `Token::Ident("_")`.
    pub fn peek_is_wildcard(&mut self) -> Result<bool> {
        // Peek without consuming.
        let token = self.next_token()?;
        let is_wildcard = matches!(&token, Token::Ident(name) if name == "_");
        self.pushback.push_front(token);
        Ok(is_wildcard)
    }

    /// Consume the wildcard token. Caller is responsible for
    /// having checked `peek_is_wildcard` returned true.
    pub fn consume_wildcard(&mut self) -> Result<()> {
        match self.next_token()? {
            Token::Ident(name) if name == "_" => Ok(()),
            other => Err(Error::UnexpectedToken {
                expected: "wildcard `_`",
                got: other,
            }),
        }
    }

    /// Helper for `PatternField`'s out-of-context `NotaDecode`
    /// impl: returns true if the next token is `@`.
    pub fn peek_is_bind_marker(&mut self) -> Result<bool> {
        let token = self.next_token()?;
        let is_bind = matches!(&token, Token::At);
        self.pushback.push_front(token);
        Ok(is_bind)
    }

    /// Returns true if the next token is `)` or `|)` —
    /// indicating no more fields remain in the current record.
    /// Used by `Option<T>::decode` to implement trailing-
    /// omission (an absent `)` boundary means the optional was
    /// `None`).
    pub fn peek_is_record_end(&mut self) -> Result<bool> {
        let token = self.next_token()?;
        let is_end = matches!(&token, Token::RParen | Token::RParenPipe);
        self.pushback.push_front(token);
        Ok(is_end)
    }

    /// Returns true if the next token is the bare identifier
    /// `None` — used by `Option<T>::decode` to detect the
    /// explicit-`None`-ident sentinel (legacy nota dialect).
    pub fn peek_is_explicit_none(&mut self) -> Result<bool> {
        let token = self.next_token()?;
        let is_none = matches!(&token, Token::Ident(name) if name == "None");
        self.pushback.push_front(token);
        Ok(is_none)
    }

    /// Consume the explicit `None` identifier. Caller is
    /// responsible for having checked `peek_is_explicit_none`
    /// returned true.
    pub fn consume_explicit_none(&mut self) -> Result<()> {
        match self.next_token()? {
            Token::Ident(name) if name == "None" => Ok(()),
            other => Err(Error::UnexpectedToken {
                expected: "explicit `None` identifier",
                got: other,
            }),
        }
    }

    // ─── Sequence bracketing ────────────────────────────────

    /// Expect `[` opening a sequence.
    pub fn expect_seq_start(&mut self) -> Result<()> {
        match self.next_token()? {
            Token::LBracket => Ok(()),
            other => Err(Error::UnexpectedToken {
                expected: "`[` opening a sequence",
                got: other,
            }),
        }
    }

    /// Expect `]` closing a sequence.
    pub fn expect_seq_end(&mut self) -> Result<()> {
        match self.next_token()? {
            Token::RBracket => Ok(()),
            other => Err(Error::UnexpectedToken {
                expected: "`]` closing a sequence",
                got: other,
            }),
        }
    }

    /// Returns true if the next token is `]` — used by
    /// `Vec<T>::decode` to detect end-of-sequence.
    pub fn peek_is_seq_end(&mut self) -> Result<bool> {
        let token = self.next_token()?;
        let is_end = matches!(&token, Token::RBracket);
        self.pushback.push_front(token);
        Ok(is_end)
    }

    /// Peek at the next token without consuming it. Returns
    /// `Ok(None)` at end-of-input — useful for top-level
    /// dispatchers (the nexus daemon's sigil-and-delimiter
    /// parser) where end-of-input is a normal termination
    /// condition, not an error. Existing `peek_is_*` methods
    /// error on EOF because they're called inside record bodies
    /// where EOF *is* an error.
    pub fn peek_token(&mut self) -> Result<Option<Token>> {
        if let Some(token) = self.pushback.front() {
            return Ok(Some(token.clone()));
        }
        match self.lexer.next_token()? {
            Some(token) => {
                self.pushback.push_back(token.clone());
                Ok(Some(token))
            }
            None => Ok(None),
        }
    }

    /// Look at the head identifier of the next record (or
    /// pattern record) without consuming any tokens. Used by
    /// closed-enum dispatchers (`NexusVerb`) that need to know
    /// which variant to delegate to before the variant's full
    /// `decode` runs `expect_record_head` (or `expect_pattern_record_head`).
    ///
    /// Accepts both `(Name …)` and `(| Name … |)` openers — a
    /// single `NexusVerb` enum may dispatch over kinds whose
    /// payloads use either form (e.g. `QueryOperation` whose
    /// variants are `NexusPattern` types).
    ///
    /// On success, the opener and the identifier remain queued
    /// as the next two tokens so the variant's full decode reads
    /// them normally.
    pub fn peek_record_head(&mut self) -> Result<String> {
        let opener = self.next_token()?;
        if !matches!(opener, Token::LParen | Token::LParenPipe) {
            // Push back what we read so the caller's error
            // message can come from its own dispatch site.
            self.pushback.push_front(opener.clone());
            return Err(Error::UnexpectedToken {
                expected: "`(` or `(|` opening a record",
                got: opener,
            });
        }
        let head_token = self.next_token()?;
        let head_name = match &head_token {
            Token::Ident(name) if crate::lexer::is_pascal_case(name) => name.clone(),
            _ => {
                self.pushback.push_front(head_token.clone());
                self.pushback.push_front(opener);
                return Err(Error::UnexpectedToken {
                    expected: "PascalCase record-head identifier",
                    got: head_token,
                });
            }
        };
        // Push back so the variant's full `decode` reads them.
        self.pushback.push_front(head_token);
        self.pushback.push_front(opener);
        Ok(head_name)
    }
}
