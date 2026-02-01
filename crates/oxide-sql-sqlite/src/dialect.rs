//! SQLite dialect implementation.

use oxide_sql_core::dialect::Dialect;

/// SQLite dialect.
#[derive(Debug, Default, Clone, Copy)]
pub struct SqliteDialect;

impl SqliteDialect {
    /// Creates a new SQLite dialect.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Dialect for SqliteDialect {
    fn name(&self) -> &'static str {
        "sqlite"
    }

    fn identifier_quote(&self) -> char {
        '"' // SQLite also accepts backticks, but double quotes are standard
    }

    fn supports_returning(&self) -> bool {
        true // SQLite 3.35.0+
    }

    fn supports_upsert(&self) -> bool {
        true // SQLite 3.24.0+
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqlite_dialect() {
        let dialect = SqliteDialect::new();
        assert_eq!(dialect.name(), "sqlite");
        assert_eq!(dialect.identifier_quote(), '"');
        assert!(dialect.supports_returning());
        assert!(dialect.supports_upsert());
    }
}
