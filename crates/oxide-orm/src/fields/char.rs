//! Character/string field types.

use super::{Field, FieldOptions};

/// A character field with a maximum length.
///
/// # Example
///
/// ```ignore
/// #[derive(Model)]
/// struct User {
///     #[field(max_length = 150)]
///     username: String,
/// }
/// ```
#[derive(Debug, Clone)]
pub struct CharField {
    /// Maximum length of the field.
    pub max_length: usize,
    /// Field options.
    pub options: FieldOptions,
}

impl CharField {
    /// Creates a new CharField with the given max length.
    pub fn new(max_length: usize) -> Self {
        Self {
            max_length,
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

impl Field for CharField {
    fn sql_type(&self) -> &'static str {
        "VARCHAR"
    }

    fn options(&self) -> &FieldOptions {
        &self.options
    }

    fn validate(&self, value: &str) -> Result<(), String> {
        if value.len() > self.max_length {
            return Err(format!(
                "Value exceeds maximum length of {} characters",
                self.max_length
            ));
        }
        Ok(())
    }
}

/// A text field for large strings (no max length).
#[derive(Debug, Clone)]
pub struct TextField {
    /// Field options.
    pub options: FieldOptions,
}

impl TextField {
    /// Creates a new TextField.
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

impl Default for TextField {
    fn default() -> Self {
        Self::new()
    }
}

impl Field for TextField {
    fn sql_type(&self) -> &'static str {
        "TEXT"
    }

    fn options(&self) -> &FieldOptions {
        &self.options
    }

    fn validate(&self, _value: &str) -> Result<(), String> {
        Ok(())
    }
}

/// An email field with validation.
#[derive(Debug, Clone)]
pub struct EmailField {
    /// Maximum length (default 254).
    pub max_length: usize,
    /// Field options.
    pub options: FieldOptions,
}

impl EmailField {
    /// Creates a new EmailField.
    pub fn new() -> Self {
        Self {
            max_length: 254,
            options: FieldOptions::new(),
        }
    }

    /// Sets the maximum length.
    #[must_use]
    pub fn max_length(mut self, length: usize) -> Self {
        self.max_length = length;
        self
    }

    /// Sets field options.
    #[must_use]
    pub fn options(mut self, options: FieldOptions) -> Self {
        self.options = options;
        self
    }
}

impl Default for EmailField {
    fn default() -> Self {
        Self::new()
    }
}

impl Field for EmailField {
    fn sql_type(&self) -> &'static str {
        "VARCHAR"
    }

    fn options(&self) -> &FieldOptions {
        &self.options
    }

    fn validate(&self, value: &str) -> Result<(), String> {
        if value.len() > self.max_length {
            return Err(format!(
                "Value exceeds maximum length of {} characters",
                self.max_length
            ));
        }

        // Basic email validation
        if !value.contains('@') || !value.contains('.') {
            return Err("Invalid email address".to_string());
        }

        let parts: Vec<&str> = value.split('@').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err("Invalid email address".to_string());
        }

        Ok(())
    }
}

/// A URL field with validation.
#[derive(Debug, Clone)]
pub struct UrlField {
    /// Maximum length (default 200).
    pub max_length: usize,
    /// Field options.
    pub options: FieldOptions,
}

impl UrlField {
    /// Creates a new UrlField.
    pub fn new() -> Self {
        Self {
            max_length: 200,
            options: FieldOptions::new(),
        }
    }

    /// Sets the maximum length.
    #[must_use]
    pub fn max_length(mut self, length: usize) -> Self {
        self.max_length = length;
        self
    }

    /// Sets field options.
    #[must_use]
    pub fn options(mut self, options: FieldOptions) -> Self {
        self.options = options;
        self
    }
}

impl Default for UrlField {
    fn default() -> Self {
        Self::new()
    }
}

impl Field for UrlField {
    fn sql_type(&self) -> &'static str {
        "VARCHAR"
    }

    fn options(&self) -> &FieldOptions {
        &self.options
    }

    fn validate(&self, value: &str) -> Result<(), String> {
        if value.len() > self.max_length {
            return Err(format!(
                "Value exceeds maximum length of {} characters",
                self.max_length
            ));
        }

        // Basic URL validation
        if !value.starts_with("http://") && !value.starts_with("https://") {
            return Err("URL must start with http:// or https://".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_field_validation() {
        let field = CharField::new(10);
        assert!(field.validate("short").is_ok());
        assert!(field.validate("this is too long").is_err());
    }

    #[test]
    fn test_email_field_validation() {
        let field = EmailField::new();
        assert!(field.validate("user@example.com").is_ok());
        assert!(field.validate("invalid").is_err());
        assert!(field.validate("@example.com").is_err());
        assert!(field.validate("user@").is_err());
    }

    #[test]
    fn test_url_field_validation() {
        let field = UrlField::new();
        assert!(field.validate("https://example.com").is_ok());
        assert!(field.validate("http://example.com").is_ok());
        assert!(field.validate("example.com").is_err());
    }
}
