//! Crate-wide error type. Typed variants only — no `Custom(String)`.
//!
//! The lexer's catch-all `Lexer { … }` variant is a transitional
//! holding pen; the lexer's failure cases will refactor into
//! typed variants in a follow-up pass.

use thiserror::Error;

use crate::lexer::Token;

#[derive(Debug, Error)]
pub enum Error {
    /// Lexer error. Will refactor into typed variants
    /// (UnexpectedChar / UnexpectedEnd / InvalidEscape /
    /// InvalidNumber / …) once the decoder-side types are
    /// stable enough to share their error vocabulary.
    #[error("lexer: {0}")]
    Lexer(String),

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

    /// A unit-variant enum decoder read an identifier that
    /// is not a variant of the enum.
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
}

pub type Result<T> = std::result::Result<T, Error>;
