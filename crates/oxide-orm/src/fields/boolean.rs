//! Boolean field type.

use super::{Field, FieldOptions};

/// A boolean field.
#[derive(Debug, Clone)]
pub struct BooleanField {
    /// Field options.
    pub options: FieldOptions,
}

impl BooleanField {
    /// Creates a new BooleanField.
    pub fn new() -> Self {
        Self {
            options: FieldOptions::new(),
        }
    }

    /// Sets field options.
    #[must_use]
    pub fn options(mut self, options: FieldOptions) -> Self {
        self.options = options;
        self
    }
}

impl Default for BooleanField {
    fn default() -> Self {
        Self::new()
    }
}

impl Field for BooleanField {
    fn sql_type(&self) -> &'static str {
        "BOOLEAN"
    }

    fn options(&self) -> &FieldOptions {
        &self.options
    }

    fn validate(&self, value: &str) -> Result<(), String> {
        let lower = value.to_lowercase();
        match lower.as_str() {
            "true" | "false" | "1" | "0" | "yes" | "no" | "on" | "off" => Ok(()),
            _ => Err("Value must be a boolean".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boolean_field_validation() {
        let field = BooleanField::new();
        assert!(field.validate("true").is_ok());
        assert!(field.validate("false").is_ok());
        assert!(field.validate("True").is_ok());
        assert!(field.validate("FALSE").is_ok());
        assert!(field.validate("1").is_ok());
        assert!(field.validate("0").is_ok());
        assert!(field.validate("yes").is_ok());
        assert!(field.validate("no").is_ok());
        assert!(field.validate("maybe").is_err());
    }
}
