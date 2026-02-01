//! Generic SQL dialect.

use super::Dialect;

/// A generic SQL dialect using ANSI SQL standards.
#[derive(Debug, Default, Clone, Copy)]
pub struct GenericDialect;

impl GenericDialect {
    /// Creates a new generic dialect.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Dialect for GenericDialect {
    fn name(&self) -> &'static str {
        "generic"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_dialect() {
        let dialect = GenericDialect::new();
        assert_eq!(dialect.name(), "generic");
        assert_eq!(dialect.identifier_quote(), '"');
        assert_eq!(dialect.parameter_placeholder(), "?");
        assert!(!dialect.supports_returning());
        assert!(!dialect.supports_upsert());
    }
}
