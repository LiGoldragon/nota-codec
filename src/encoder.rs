//! `Encoder` — typed writing into a `String` buffer.
//!
//! Tracks a single `needs_space` flag so callers don't have to
//! think about field separators. Each atomic-write method emits
//! a leading space when a previous token has been written into
//! the same delimiter context; the start of a fresh record (the
//! position right after `(Name`) is the only place no leading
//! space appears.
//!
//! Dialect-aware: refuses to emit nexus-only forms when in nota
//! mode (sigils, pattern delimiters — wired in as those derives
//! land).

use std::fmt::Write;

use crate::error::Result;
use crate::lexer::Dialect;

pub struct Encoder {
    output: String,
    #[allow(dead_code)]
    dialect: Dialect,
    /// True when the next atomic write should be preceded by a
    /// space. Set after every written token; cleared right after
    /// `(Name` is written so the first field appears with no
    /// leading space.
    needs_space: bool,
}

impl Encoder {
    /// Open an encoder targeting nexus-dialect output.
    pub fn nexus() -> Self {
        Self { output: String::new(), dialect: Dialect::Nexus, needs_space: false }
    }

    /// Open an encoder targeting nota-dialect output.
    pub fn nota() -> Self {
        Self { output: String::new(), dialect: Dialect::Nota, needs_space: false }
    }

    /// Consume the encoder and return the accumulated text.
    pub fn into_string(self) -> String {
        self.output
    }

    fn write_separator_if_needed(&mut self) {
        if self.needs_space {
            self.output.push(' ');
        }
    }

    // ─── Atomic value writes ─────────────────────────────────

    /// Write a `u64` literal.
    pub fn write_u64(&mut self, value: u64) -> Result<()> {
        self.write_separator_if_needed();
        write!(self.output, "{value}").expect("write to String cannot fail");
        self.needs_space = true;
        Ok(())
    }

    /// Write a PascalCase identifier verbatim. Used for unit-
    /// variant enum values and record-head names. The caller is
    /// responsible for the identifier already being PascalCase
    /// (in derived code, it comes from a Rust variant or type
    /// name, which is PascalCase by convention).
    pub fn write_pascal_identifier(&mut self, name: &str) -> Result<()> {
        self.write_separator_if_needed();
        self.output.push_str(name);
        self.needs_space = true;
        Ok(())
    }

    /// Write a quoted string literal. For MVP, strings must not
    /// contain `"` or `\`; richer escaping lands when a real
    /// caller needs it.
    pub fn write_string(&mut self, value: &str) -> Result<()> {
        self.write_separator_if_needed();
        self.output.push('"');
        self.output.push_str(value);
        self.output.push('"');
        self.needs_space = true;
        Ok(())
    }

    // ─── Record bracketing ──────────────────────────────────

    /// Open a record: write `(Name`. The first subsequent
    /// atomic-write inside this record will appear with no
    /// leading space; later writes inside the same record get
    /// the space separator.
    pub fn start_record(&mut self, name: &str) -> Result<()> {
        self.write_separator_if_needed();
        self.output.push('(');
        self.output.push_str(name);
        self.needs_space = true;
        Ok(())
    }

    /// Close the most recently opened record: write `)`.
    pub fn end_record(&mut self) -> Result<()> {
        self.output.push(')');
        self.needs_space = true;
        Ok(())
    }

    // ─── Pattern bracketing (nexus-only) ────────────────────

    /// Open a pattern record: write `(| Name`. The first
    /// subsequent field-write inside this pattern record will
    /// appear with no leading space; later writes get the
    /// space separator.
    pub fn start_pattern_record(&mut self, name: &str) -> Result<()> {
        // TODO: error in nota dialect (pattern delimiters do
        // not exist in nota). Will land alongside dialect-aware
        // tests.
        self.write_separator_if_needed();
        self.output.push_str("(|");
        self.output.push(' ');
        self.output.push_str(name);
        self.needs_space = true;
        Ok(())
    }

    /// Close the most recently opened pattern record: write ` |)`.
    /// The leading space is symmetric with `start_pattern_record`'s
    /// `(| ` opening — keeps the wire form readable.
    pub fn end_pattern_record(&mut self) -> Result<()> {
        self.write_separator_if_needed();
        self.output.push_str("|)");
        self.needs_space = true;
        Ok(())
    }

    /// Encode a `PatternField<T>` at a known schema field
    /// position. `bind_name` is the schema field name; if the
    /// pattern is `Bind`, the wire form is `@<bind_name>`.
    pub fn encode_pattern_field<T>(
        &mut self,
        field: &crate::pattern_field::PatternField<T>,
        bind_name: &'static str,
    ) -> Result<()>
    where
        T: crate::traits::NotaEncode,
    {
        match field {
            crate::pattern_field::PatternField::Wildcard => self.write_wildcard(),
            crate::pattern_field::PatternField::Bind => self.write_bind(bind_name),
            crate::pattern_field::PatternField::Match(value) => value.encode(self),
        }
    }

    /// Write a wildcard `_`.
    pub fn write_wildcard(&mut self) -> Result<()> {
        self.write_separator_if_needed();
        self.output.push('_');
        self.needs_space = true;
        Ok(())
    }

    /// Write a bind reference `@<name>`. The caller is
    /// responsible for `name` being a valid camelCase or
    /// kebab-case identifier (which it is when sourced from a
    /// `*Query` struct field name).
    pub fn write_bind(&mut self, name: &str) -> Result<()> {
        self.write_separator_if_needed();
        self.output.push('@');
        self.output.push_str(name);
        self.needs_space = true;
        Ok(())
    }
}
