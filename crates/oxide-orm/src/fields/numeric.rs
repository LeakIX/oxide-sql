//! Numeric field types.

use super::{Field, FieldOptions};

/// A small integer field (16-bit).
#[derive(Debug, Clone)]
pub struct SmallIntField {
    /// Field options.
    pub options: FieldOptions,
}

impl SmallIntField {
    /// Creates a new SmallIntField.
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

impl Default for SmallIntField {
    fn default() -> Self {
        Self::new()
    }
}

impl Field for SmallIntField {
    fn sql_type(&self) -> &'static str {
        "SMALLINT"
    }

    fn options(&self) -> &FieldOptions {
        &self.options
    }

    fn validate(&self, value: &str) -> Result<(), String> {
        value
            .parse::<i16>()
            .map(|_| ())
            .map_err(|_| "Invalid small integer value".to_string())
    }
}

/// A standard integer field (32-bit).
#[derive(Debug, Clone)]
pub struct IntegerField {
    /// Field options.
    pub options: FieldOptions,
}

impl IntegerField {
    /// Creates a new IntegerField.
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

impl Default for IntegerField {
    fn default() -> Self {
        Self::new()
    }
}

impl Field for IntegerField {
    fn sql_type(&self) -> &'static str {
        "INTEGER"
    }

    fn options(&self) -> &FieldOptions {
        &self.options
    }

    fn validate(&self, value: &str) -> Result<(), String> {
        value
            .parse::<i32>()
            .map(|_| ())
            .map_err(|_| "Invalid integer value".to_string())
    }
}

/// A big integer field (64-bit).
#[derive(Debug, Clone)]
pub struct BigIntField {
    /// Field options.
    pub options: FieldOptions,
}

impl BigIntField {
    /// Creates a new BigIntField.
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

impl Default for BigIntField {
    fn default() -> Self {
        Self::new()
    }
}

impl Field for BigIntField {
    fn sql_type(&self) -> &'static str {
        "BIGINT"
    }

    fn options(&self) -> &FieldOptions {
        &self.options
    }

    fn validate(&self, value: &str) -> Result<(), String> {
        value
            .parse::<i64>()
            .map(|_| ())
            .map_err(|_| "Invalid big integer value".to_string())
    }
}

/// A floating-point field.
#[derive(Debug, Clone)]
pub struct FloatField {
    /// Field options.
    pub options: FieldOptions,
}

impl FloatField {
    /// Creates a new FloatField.
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

impl Default for FloatField {
    fn default() -> Self {
        Self::new()
    }
}

impl Field for FloatField {
    fn sql_type(&self) -> &'static str {
        "REAL"
    }

    fn options(&self) -> &FieldOptions {
        &self.options
    }

    fn validate(&self, value: &str) -> Result<(), String> {
        value
            .parse::<f64>()
            .map(|_| ())
            .map_err(|_| "Invalid float value".to_string())
    }
}

/// A decimal field with fixed precision.
#[derive(Debug, Clone)]
pub struct DecimalField {
    /// Maximum number of digits.
    pub max_digits: u8,
    /// Number of decimal places.
    pub decimal_places: u8,
    /// Field options.
    pub options: FieldOptions,
}

impl DecimalField {
    /// Creates a new DecimalField.
    pub fn new(max_digits: u8, decimal_places: u8) -> Self {
        Self {
            max_digits,
            decimal_places,
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

impl Field for DecimalField {
    fn sql_type(&self) -> &'static str {
        "DECIMAL"
    }

    fn options(&self) -> &FieldOptions {
        &self.options
    }

    fn validate(&self, value: &str) -> Result<(), String> {
        // Parse as float first to validate format
        let num: f64 = value
            .parse()
            .map_err(|_| "Invalid decimal value".to_string())?;

        // Check total digits
        let parts: Vec<&str> = value.split('.').collect();
        let integer_part = parts[0].trim_start_matches('-');
        let decimal_part = parts.get(1).unwrap_or(&"");

        let total_digits = integer_part.len() + decimal_part.len();
        if total_digits > self.max_digits as usize {
            return Err(format!(
                "Value has too many digits (max {})",
                self.max_digits
            ));
        }

        if decimal_part.len() > self.decimal_places as usize {
            return Err(format!(
                "Value has too many decimal places (max {})",
                self.decimal_places
            ));
        }

        // Check for NaN or infinity
        if num.is_nan() || num.is_infinite() {
            return Err("Value must be a finite number".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer_field_validation() {
        let field = IntegerField::new();
        assert!(field.validate("123").is_ok());
        assert!(field.validate("-456").is_ok());
        assert!(field.validate("not a number").is_err());
    }

    #[test]
    fn test_float_field_validation() {
        let field = FloatField::new();
        assert!(field.validate("123.456").is_ok());
        assert!(field.validate("-789.012").is_ok());
        assert!(field.validate("not a number").is_err());
    }

    #[test]
    fn test_decimal_field_validation() {
        let field = DecimalField::new(5, 2);
        assert!(field.validate("123.45").is_ok());
        assert!(field.validate("12.3").is_ok());
        assert!(field.validate("123.456").is_err()); // Too many decimal places
        assert!(field.validate("123456.78").is_err()); // Too many total digits
    }
}
