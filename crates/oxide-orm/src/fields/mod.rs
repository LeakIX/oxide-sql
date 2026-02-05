//! Field types for model definitions.
//!
//! These field types provide Django-like field definitions with validation
//! and metadata for the ORM.

mod boolean;
mod char;
mod numeric;
mod relations;
mod temporal;

pub use boolean::BooleanField;
pub use char::{CharField, EmailField, TextField, UrlField};
pub use numeric::{BigIntField, DecimalField, FloatField, IntegerField, SmallIntField};
pub use relations::{ForeignKey, ManyToMany};
pub use temporal::{DateField, DateTimeField, TimeField};

/// Common field options.
#[derive(Debug, Clone, Default)]
pub struct FieldOptions {
    /// Whether the field can be null.
    pub null: bool,
    /// Whether the field is blank (empty string allowed).
    pub blank: bool,
    /// Default value for the field.
    pub default: Option<String>,
    /// Whether to create a database index.
    pub db_index: bool,
    /// Whether the field must be unique.
    pub unique: bool,
    /// Human-readable name for the field.
    pub verbose_name: Option<String>,
    /// Help text for forms.
    pub help_text: Option<String>,
    /// Whether the field is editable in forms.
    pub editable: bool,
    /// Whether this is the primary key.
    pub primary_key: bool,
}

impl FieldOptions {
    /// Creates new field options with defaults.
    pub fn new() -> Self {
        Self {
            editable: true,
            ..Default::default()
        }
    }

    /// Sets the null option.
    #[must_use]
    pub fn null(mut self, value: bool) -> Self {
        self.null = value;
        self
    }

    /// Sets the blank option.
    #[must_use]
    pub fn blank(mut self, value: bool) -> Self {
        self.blank = value;
        self
    }

    /// Sets the default value.
    #[must_use]
    pub fn default(mut self, value: impl Into<String>) -> Self {
        self.default = Some(value.into());
        self
    }

    /// Sets the db_index option.
    #[must_use]
    pub fn db_index(mut self, value: bool) -> Self {
        self.db_index = value;
        self
    }

    /// Sets the unique option.
    #[must_use]
    pub fn unique(mut self, value: bool) -> Self {
        self.unique = value;
        self
    }

    /// Sets the verbose_name option.
    #[must_use]
    pub fn verbose_name(mut self, value: impl Into<String>) -> Self {
        self.verbose_name = Some(value.into());
        self
    }

    /// Sets the help_text option.
    #[must_use]
    pub fn help_text(mut self, value: impl Into<String>) -> Self {
        self.help_text = Some(value.into());
        self
    }

    /// Sets the editable option.
    #[must_use]
    pub fn editable(mut self, value: bool) -> Self {
        self.editable = value;
        self
    }

    /// Sets the primary_key option.
    #[must_use]
    pub fn primary_key(mut self, value: bool) -> Self {
        self.primary_key = value;
        self
    }
}

/// Trait for field types that can generate SQL column definitions.
pub trait Field {
    /// Returns the SQL type for this field.
    fn sql_type(&self) -> &'static str;

    /// Returns the field options.
    fn options(&self) -> &FieldOptions;

    /// Validates a value for this field.
    fn validate(&self, value: &str) -> Result<(), String>;
}
