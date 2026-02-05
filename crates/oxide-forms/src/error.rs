//! Error types for forms.

use std::collections::HashMap;
use thiserror::Error;

/// Form-specific errors.
#[derive(Debug, Error)]
pub enum FormError {
    /// Validation failed with errors.
    #[error("validation errors: {0:?}")]
    ValidationErrors(ValidationErrors),

    /// Missing required field.
    #[error("missing required field: {0}")]
    MissingField(String),

    /// Invalid field value.
    #[error("invalid value for field {field}: {message}")]
    InvalidValue { field: String, message: String },

    /// Form data parsing error.
    #[error("failed to parse form data: {0}")]
    ParseError(String),
}

/// Collection of validation errors by field.
#[derive(Debug, Clone, Default)]
pub struct ValidationErrors {
    /// Errors keyed by field name.
    pub errors: HashMap<String, Vec<String>>,
}

impl ValidationErrors {
    /// Creates a new empty ValidationErrors.
    pub fn new() -> Self {
        Self {
            errors: HashMap::new(),
        }
    }

    /// Adds an error for a field.
    pub fn add(&mut self, field: &str, message: impl Into<String>) {
        self.errors
            .entry(field.to_string())
            .or_default()
            .push(message.into());
    }

    /// Returns whether there are any errors.
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns the number of fields with errors.
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Returns errors for a specific field.
    pub fn get(&self, field: &str) -> Option<&Vec<String>> {
        self.errors.get(field)
    }

    /// Returns all errors as a flat list.
    pub fn all_errors(&self) -> Vec<(&str, &str)> {
        self.errors
            .iter()
            .flat_map(|(field, messages)| {
                messages
                    .iter()
                    .map(move |msg| (field.as_str(), msg.as_str()))
            })
            .collect()
    }
}

impl std::fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (field, messages) in &self.errors {
            for message in messages {
                writeln!(f, "{field}: {message}")?;
            }
        }
        Ok(())
    }
}

/// Result type alias for form operations.
pub type Result<T> = std::result::Result<T, FormError>;
