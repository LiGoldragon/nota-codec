//! Crate-wide error type. Typed variants only — every failure
//! mode is named so callers can pattern-match on what went wrong
//! and the diagnostic-rendering layer can render rich messages.

use thiserror::Error;

use crate::lexer::{Dialect, Token};

#[derive(Debug, Error)]
pub enum Error {
    // ─── Decoder errors ─────────────────────────────────────

    /// The decoder expected a particular token shape and got
    /// something else.
    #[error("expected {expected}, got {got:?}")]
    UnexpectedToken {
        expected: &'static str,
        got: Token,
    },

    /// The decoder expected a record header `(Foo …)` for a
    /// specific kind name and read a different name.
    #[error("expected record `({expected} …)`, got `({got} …)`")]
    ExpectedRecordHead {
        expected: &'static str,
        got: String,
    },

    /// A pattern bind name `@name` did not match the schema
    /// field name at this position.
    #[error("bind name `@{got}` does not match the schema field name `{expected}` at this position")]
    WrongBindName {
        expected: &'static str,
        got: String,
    },

    /// A unit-variant enum decoder read an identifier that is
    /// not a variant of the enum.
    #[error("unknown variant `{got}` for enum `{enum_name}`")]
    UnknownVariant {
        enum_name: &'static str,
        got: String,
    },

    /// A verb-payload decoder read a record-head identifier
    /// that is not one of the closed kind set the verb
    /// dispatches over.
    #[error("unknown kind `{got}` for verb `{verb}`")]
    UnknownKindForVerb {
        verb: &'static str,
        got: String,
    },

    /// Hit end-of-input while parsing a value.
    #[error("unexpected end of input while parsing {while_parsing}")]
    UnexpectedEnd {
        while_parsing: &'static str,
    },

    /// A validating newtype's `try_new` rejected the decoded
    /// inner value. Used by the `NotaTryTransparent` derive.
    #[error("validation rejected `{type_name}`: {message}")]
    Validation {
        type_name: &'static str,
        message: String,
    },

    /// An integer literal didn't fit in the target type.
    #[error("integer {value} out of range for {target}")]
    IntegerOutOfRange {
        target: &'static str,
        value: i128,
    },

    /// `PatternField::Bind` was reached through the flat
    /// `NotaEncode` / `NotaDecode` path. Bind names are
    /// schema-field-position-specific; encoding or decoding
    /// `Bind` requires the contextual `encode_pattern_field` /
    /// `decode_pattern_field` protocol method, which the
    /// `NexusPattern` derive emits.
    #[error("PatternField::Bind cannot be used outside a NexusPattern field position — the bind name is contextual; use encode_pattern_field / decode_pattern_field instead")]
    PatternBindOutOfContext,

    // ─── Lexer errors ───────────────────────────────────────

    /// Lexer encountered a character it cannot tokenize for the
    /// active dialect.
    #[error("unexpected character {character:?} at byte offset {offset} ({dialect:?} dialect)")]
    UnexpectedChar {
        character: char,
        offset: usize,
        dialect: Dialect,
    },

    /// One of the comparison/equality tokens (`<`, `>`, `<=`,
    /// `>=`, `!=`) reserved for future operator design per
    /// nexus/spec/grammar.md §"Reserved tokens". Currently a
    /// hard parse error in both dialects.
    #[error("reserved token {token:?} at byte offset {offset} — `<` `>` `<=` `>=` `!=` are reserved for future comparison operators")]
    ReservedComparisonToken {
        token: char,
        offset: usize,
    },

    /// `"…"` string opened but never closed before EOI.
    #[error("unterminated inline string — missing closing `\"`")]
    UnterminatedInlineString,

    /// `\"\"\"…\"\"\"` string opened but never closed before EOI.
    #[error("unterminated multiline string — missing closing `\"\"\"`")]
    UnterminatedMultilineString,

    /// Bare newline inside a `"…"` inline string. Use the
    /// `\"\"\" \"\"\"` form for content that spans lines.
    #[error("unexpected newline in inline string — use `\"\"\" \"\"\"` for multiline content")]
    NewlineInInlineString,

    /// `\X` escape with X not in the supported set
    /// (`\\`, `\"`, `\n`, `\t`, `\r`).
    #[error("unknown escape `\\{found}` in inline string — supported: `\\\\`, `\\\"`, `\\n`, `\\t`, `\\r`")]
    UnknownEscape {
        found: char,
    },

    /// `\` immediately before EOI inside an inline string.
    #[error("unterminated escape at end of input")]
    UnterminatedEscape,

    /// `|` followed by something other than `)`, `}}`, or `]`
    /// (the only valid pipe-closer continuations in nexus).
    #[error("unexpected `|` followed by {following:?} — expected `|)`, `|}}`, or `|]`")]
    UnexpectedPipeContinuation {
        following: char,
    },

    /// `|` immediately before EOI.
    #[error("unexpected `|` at end of input")]
    UnexpectedPipeAtEnd,

    /// Multi-byte UTF-8 sequence cut off by EOI inside an
    /// inline string.
    #[error("truncated UTF-8 sequence in inline string")]
    TruncatedUtf8InString,

    /// `#` byte-literal prefix not followed by any hex digits.
    #[error("`#` byte literal must be followed by lowercase hex digits")]
    EmptyByteLiteral,

    /// `#hex…` byte literal with an odd number of hex digits.
    #[error("byte literal must have an even number of hex digits, got {length}")]
    OddByteLiteralLength {
        length: usize,
    },

    /// A number literal failed to parse.
    #[error("invalid {kind:?} literal {raw:?}: {detail}")]
    InvalidNumber {
        kind: NumberKind,
        raw: String,
        detail: String,
    },
}

/// Which numeric form the lexer was attempting when the parse
/// failed. Used by [`Error::InvalidNumber`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberKind {
    Float,
    Integer,
    /// Radix-prefixed integer (`0x…`, `0b…`, `0o…`).
    RadixInt(u32),
}

pub type Result<T> = std::result::Result<T, Error>;
