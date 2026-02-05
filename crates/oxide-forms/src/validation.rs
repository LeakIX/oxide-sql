//! Form field validators.

use regex::Regex;

/// Trait for field validators.
pub trait Validator: Send + Sync {
    /// Validates a value and returns an error message if invalid.
    fn validate(&self, value: &str) -> Result<(), String>;

    /// Returns the error message for this validator.
    fn message(&self) -> &str;
}

/// Validator that requires a non-empty value.
#[derive(Debug, Clone)]
pub struct RequiredValidator {
    message: String,
}

impl RequiredValidator {
    /// Creates a new RequiredValidator with default message.
    pub fn new() -> Self {
        Self {
            message: "This field is required.".to_string(),
        }
    }

    /// Creates a new RequiredValidator with custom message.
    pub fn with_message(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Default for RequiredValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Validator for RequiredValidator {
    fn validate(&self, value: &str) -> Result<(), String> {
        if value.trim().is_empty() {
            Err(self.message.clone())
        } else {
            Ok(())
        }
    }

    fn message(&self) -> &str {
        &self.message
    }
}

/// Validator that enforces a maximum length.
#[derive(Debug, Clone)]
pub struct MaxLengthValidator {
    max_length: usize,
    message: String,
}

impl MaxLengthValidator {
    /// Creates a new MaxLengthValidator.
    pub fn new(max_length: usize) -> Self {
        Self {
            max_length,
            message: format!("Ensure this value has at most {max_length} characters."),
        }
    }

    /// Creates a new MaxLengthValidator with custom message.
    pub fn with_message(max_length: usize, message: impl Into<String>) -> Self {
        Self {
            max_length,
            message: message.into(),
        }
    }
}

impl Validator for MaxLengthValidator {
    fn validate(&self, value: &str) -> Result<(), String> {
        if value.len() > self.max_length {
            Err(self.message.clone())
        } else {
            Ok(())
        }
    }

    fn message(&self) -> &str {
        &self.message
    }
}

/// Validator that enforces a minimum length.
#[derive(Debug, Clone)]
pub struct MinLengthValidator {
    min_length: usize,
    message: String,
}

impl MinLengthValidator {
    /// Creates a new MinLengthValidator.
    pub fn new(min_length: usize) -> Self {
        Self {
            min_length,
            message: format!("Ensure this value has at least {min_length} characters."),
        }
    }

    /// Creates a new MinLengthValidator with custom message.
    pub fn with_message(min_length: usize, message: impl Into<String>) -> Self {
        Self {
            min_length,
            message: message.into(),
        }
    }
}

impl Validator for MinLengthValidator {
    fn validate(&self, value: &str) -> Result<(), String> {
        if value.len() < self.min_length {
            Err(self.message.clone())
        } else {
            Ok(())
        }
    }

    fn message(&self) -> &str {
        &self.message
    }
}

/// Validator for email addresses.
#[derive(Debug, Clone)]
pub struct EmailValidator {
    message: String,
}

impl EmailValidator {
    /// Creates a new EmailValidator with default message.
    pub fn new() -> Self {
        Self {
            message: "Enter a valid email address.".to_string(),
        }
    }

    /// Creates a new EmailValidator with custom message.
    pub fn with_message(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Default for EmailValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Validator for EmailValidator {
    fn validate(&self, value: &str) -> Result<(), String> {
        // Basic email validation regex
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();

        if email_regex.is_match(value) {
            Ok(())
        } else {
            Err(self.message.clone())
        }
    }

    fn message(&self) -> &str {
        &self.message
    }
}

/// Validator for URL values.
#[derive(Debug, Clone)]
pub struct UrlValidator {
    message: String,
}

impl UrlValidator {
    /// Creates a new UrlValidator with default message.
    pub fn new() -> Self {
        Self {
            message: "Enter a valid URL.".to_string(),
        }
    }

    /// Creates a new UrlValidator with custom message.
    pub fn with_message(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Default for UrlValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Validator for UrlValidator {
    fn validate(&self, value: &str) -> Result<(), String> {
        if value.starts_with("http://") || value.starts_with("https://") {
            Ok(())
        } else {
            Err(self.message.clone())
        }
    }

    fn message(&self) -> &str {
        &self.message
    }
}

/// Validator using a custom regex pattern.
#[derive(Debug, Clone)]
pub struct RegexValidator {
    pattern: Regex,
    message: String,
}

impl RegexValidator {
    /// Creates a new RegexValidator.
    pub fn new(pattern: &str, message: impl Into<String>) -> Result<Self, regex::Error> {
        Ok(Self {
            pattern: Regex::new(pattern)?,
            message: message.into(),
        })
    }
}

impl Validator for RegexValidator {
    fn validate(&self, value: &str) -> Result<(), String> {
        if self.pattern.is_match(value) {
            Ok(())
        } else {
            Err(self.message.clone())
        }
    }

    fn message(&self) -> &str {
        &self.message
    }
}

/// Validator for numeric range.
#[derive(Debug, Clone)]
pub struct RangeValidator {
    min: Option<f64>,
    max: Option<f64>,
    message: String,
}

impl RangeValidator {
    /// Creates a new RangeValidator with min and max bounds.
    pub fn new(min: Option<f64>, max: Option<f64>) -> Self {
        let message = match (min, max) {
            (Some(min), Some(max)) => format!("Value must be between {min} and {max}."),
            (Some(min), None) => format!("Value must be at least {min}."),
            (None, Some(max)) => format!("Value must be at most {max}."),
            (None, None) => "Invalid value.".to_string(),
        };
        Self { min, max, message }
    }

    /// Creates a new RangeValidator with custom message.
    pub fn with_message(min: Option<f64>, max: Option<f64>, message: impl Into<String>) -> Self {
        Self {
            min,
            max,
            message: message.into(),
        }
    }
}

impl Validator for RangeValidator {
    fn validate(&self, value: &str) -> Result<(), String> {
        let num: f64 = value
            .parse()
            .map_err(|_| "Enter a valid number.".to_string())?;

        if let Some(min) = self.min {
            if num < min {
                return Err(self.message.clone());
            }
        }

        if let Some(max) = self.max {
            if num > max {
                return Err(self.message.clone());
            }
        }

        Ok(())
    }

    fn message(&self) -> &str {
        &self.message
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_validator() {
        let v = RequiredValidator::new();
        assert!(v.validate("hello").is_ok());
        assert!(v.validate("").is_err());
        assert!(v.validate("   ").is_err());
    }

    #[test]
    fn test_max_length_validator() {
        let v = MaxLengthValidator::new(5);
        assert!(v.validate("hello").is_ok());
        assert!(v.validate("hi").is_ok());
        assert!(v.validate("hello world").is_err());
    }

    #[test]
    fn test_min_length_validator() {
        let v = MinLengthValidator::new(5);
        assert!(v.validate("hello").is_ok());
        assert!(v.validate("hello world").is_ok());
        assert!(v.validate("hi").is_err());
    }

    #[test]
    fn test_email_validator() {
        let v = EmailValidator::new();
        assert!(v.validate("user@example.com").is_ok());
        assert!(v.validate("user.name@domain.co.uk").is_ok());
        assert!(v.validate("invalid").is_err());
        assert!(v.validate("@example.com").is_err());
    }

    #[test]
    fn test_url_validator() {
        let v = UrlValidator::new();
        assert!(v.validate("https://example.com").is_ok());
        assert!(v.validate("http://example.com/path").is_ok());
        assert!(v.validate("example.com").is_err());
    }

    #[test]
    fn test_regex_validator() {
        let v = RegexValidator::new(r"^\d{4}-\d{2}-\d{2}$", "Enter a valid date.").unwrap();
        assert!(v.validate("2024-01-15").is_ok());
        assert!(v.validate("not a date").is_err());
    }

    #[test]
    fn test_range_validator() {
        let v = RangeValidator::new(Some(0.0), Some(100.0));
        assert!(v.validate("50").is_ok());
        assert!(v.validate("0").is_ok());
        assert!(v.validate("100").is_ok());
        assert!(v.validate("-1").is_err());
        assert!(v.validate("101").is_err());
    }
}
