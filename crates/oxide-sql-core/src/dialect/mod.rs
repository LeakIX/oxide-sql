//! SQL Dialect support.
//!
//! Different databases have slightly different SQL syntax. This module provides
//! a trait for dialect-specific behavior.

mod generic;

pub use generic::GenericDialect;

/// Trait for SQL dialect-specific behavior.
pub trait Dialect {
    /// Returns the name of the dialect.
    fn name(&self) -> &'static str;

    /// Returns the identifier quote character (e.g., `"` for standard SQL, `` ` `` for MySQL).
    fn identifier_quote(&self) -> char {
        '"'
    }

    /// Returns the string escape character.
    fn string_escape(&self) -> &'static str {
        "''"
    }

    /// Returns the parameter placeholder style.
    fn parameter_placeholder(&self) -> &'static str {
        "?"
    }

    /// Returns whether the dialect supports RETURNING clause.
    fn supports_returning(&self) -> bool {
        false
    }

    /// Returns whether the dialect supports UPSERT (ON CONFLICT).
    fn supports_upsert(&self) -> bool {
        false
    }

    /// Returns whether the dialect supports LIMIT with OFFSET.
    fn supports_limit_offset(&self) -> bool {
        true
    }

    /// Quotes an identifier if necessary.
    fn quote_identifier(&self, name: &str) -> String {
        let quote = self.identifier_quote();
        format!("{quote}{name}{quote}")
    }
}
