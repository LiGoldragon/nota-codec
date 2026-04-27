//! `Encoder` — typed writing into a `String` buffer.
//!
//! Dialect-aware: refuses to emit nexus-only forms when in
//! nota mode. Protocol surface for the derives lands here as
//! the derives need it.

use std::fmt::Write;

use crate::error::Result;
use crate::lexer::Dialect;

pub struct Encoder {
    output: String,
    #[allow(dead_code)]
    dialect: Dialect,
}

impl Encoder {
    /// Open an encoder targeting nexus-dialect output.
    pub fn nexus() -> Self {
        Self { output: String::new(), dialect: Dialect::Nexus }
    }

    /// Open an encoder targeting nota-dialect output.
    pub fn nota() -> Self {
        Self { output: String::new(), dialect: Dialect::Nota }
    }

    /// Consume the encoder and return the accumulated text.
    pub fn into_string(self) -> String {
        self.output
    }

    /// Write a `u64` literal.
    pub fn write_u64(&mut self, value: u64) -> Result<()> {
        write!(self.output, "{value}").expect("write to String cannot fail");
        Ok(())
    }
}
