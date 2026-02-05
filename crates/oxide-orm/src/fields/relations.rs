//! Relational field types for foreign keys and many-to-many relationships.

use super::{Field, FieldOptions};
use std::marker::PhantomData;

/// A foreign key field that references another model.
///
/// # Example
///
/// ```ignore
/// #[derive(Model)]
/// struct Comment {
///     #[field(primary_key, auto)]
///     id: i64,
///     #[field(foreign_key = "Post")]
///     post_id: i64,
///     content: String,
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ForeignKey<T> {
    /// The related model name.
    pub related_model: String,
    /// The column on the related model to reference.
    pub to_field: String,
    /// What to do when the referenced object is deleted.
    pub on_delete: OnDelete,
    /// Field options.
    pub options: FieldOptions,
    /// Phantom data for the related type.
    _marker: PhantomData<T>,
}

/// Behavior when a referenced object is deleted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OnDelete {
    /// Cascade the deletion to related objects.
    #[default]
    Cascade,
    /// Protect from deletion (raise an error).
    Protect,
    /// Set the foreign key to NULL.
    SetNull,
    /// Set the foreign key to a default value.
    SetDefault,
    /// Do nothing (leave invalid references).
    DoNothing,
}

impl OnDelete {
    /// Returns the SQL representation.
    pub fn to_sql(self) -> &'static str {
        match self {
            Self::Cascade => "CASCADE",
            Self::Protect => "RESTRICT",
            Self::SetNull => "SET NULL",
            Self::SetDefault => "SET DEFAULT",
            Self::DoNothing => "NO ACTION",
        }
    }
}

impl<T> ForeignKey<T> {
    /// Creates a new ForeignKey to the given model.
    pub fn new(related_model: &str) -> Self {
        Self {
            related_model: related_model.to_string(),
            to_field: "id".to_string(),
            on_delete: OnDelete::Cascade,
            options: FieldOptions::new(),
            _marker: PhantomData,
        }
    }

    /// Sets the field to reference on the related model.
    #[must_use]
    pub fn to_field(mut self, field: &str) -> Self {
        self.to_field = field.to_string();
        self
    }

    /// Sets the on_delete behavior.
    #[must_use]
    pub fn on_delete(mut self, behavior: OnDelete) -> Self {
        self.on_delete = behavior;
        self
    }

    /// Sets field options.
    #[must_use]
    pub fn options(mut self, options: FieldOptions) -> Self {
        self.options = options;
        self
    }
}

impl<T> Field for ForeignKey<T> {
    fn sql_type(&self) -> &'static str {
        "BIGINT"
    }

    fn options(&self) -> &FieldOptions {
        &self.options
    }

    fn validate(&self, value: &str) -> Result<(), String> {
        // Foreign key values are just integers
        value
            .parse::<i64>()
            .map(|_| ())
            .map_err(|_| "Foreign key must be an integer".to_string())
    }
}

/// A many-to-many relationship field.
///
/// This field doesn't create a column directly but rather a junction table.
///
/// # Example
///
/// ```ignore
/// #[derive(Model)]
/// struct Article {
///     #[field(primary_key, auto)]
///     id: i64,
///     title: String,
///     #[field(many_to_many = "Tag")]
///     tags: Vec<Tag>,
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ManyToMany<T> {
    /// The related model name.
    pub related_model: String,
    /// Custom junction table name (optional).
    pub through: Option<String>,
    /// Field name on this model in the junction table.
    pub source_field: String,
    /// Field name on the related model in the junction table.
    pub target_field: String,
    /// Phantom data for the related type.
    _marker: PhantomData<T>,
}

impl<T> ManyToMany<T> {
    /// Creates a new ManyToMany relationship to the given model.
    pub fn new(related_model: &str) -> Self {
        Self {
            related_model: related_model.to_string(),
            through: None,
            source_field: String::new(),
            target_field: String::new(),
            _marker: PhantomData,
        }
    }

    /// Sets a custom junction table.
    #[must_use]
    pub fn through(mut self, table: &str) -> Self {
        self.through = Some(table.to_string());
        self
    }

    /// Sets the source field name in the junction table.
    #[must_use]
    pub fn source_field(mut self, field: &str) -> Self {
        self.source_field = field.to_string();
        self
    }

    /// Sets the target field name in the junction table.
    #[must_use]
    pub fn target_field(mut self, field: &str) -> Self {
        self.target_field = field.to_string();
        self
    }

    /// Returns the junction table name.
    pub fn junction_table(&self, source_model: &str) -> String {
        self.through.clone().unwrap_or_else(|| {
            // Default junction table name: model1_model2 (alphabetically sorted)
            let mut names = [
                source_model.to_lowercase(),
                self.related_model.to_lowercase(),
            ];
            names.sort();
            format!("{}_{}", names[0], names[1])
        })
    }
}

impl<T> Field for ManyToMany<T> {
    fn sql_type(&self) -> &'static str {
        // ManyToMany doesn't have a direct SQL type
        ""
    }

    fn options(&self) -> &FieldOptions {
        // ManyToMany doesn't have standard field options
        static DEFAULT_OPTIONS: FieldOptions = FieldOptions {
            null: false,
            blank: false,
            default: None,
            db_index: false,
            unique: false,
            verbose_name: None,
            help_text: None,
            editable: true,
            primary_key: false,
        };
        &DEFAULT_OPTIONS
    }

    fn validate(&self, _value: &str) -> Result<(), String> {
        // ManyToMany fields are validated through the junction table
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_foreign_key_validation() {
        let field: ForeignKey<()> = ForeignKey::new("User");
        assert!(field.validate("123").is_ok());
        assert!(field.validate("not a number").is_err());
    }

    #[test]
    fn test_on_delete_sql() {
        assert_eq!(OnDelete::Cascade.to_sql(), "CASCADE");
        assert_eq!(OnDelete::Protect.to_sql(), "RESTRICT");
        assert_eq!(OnDelete::SetNull.to_sql(), "SET NULL");
    }

    #[test]
    fn test_many_to_many_junction_table() {
        let field: ManyToMany<()> = ManyToMany::new("Tag");
        assert_eq!(field.junction_table("Article"), "article_tag");

        let field_custom: ManyToMany<()> = ManyToMany::new("Tag").through("article_tags");
        assert_eq!(field_custom.junction_table("Article"), "article_tags");
    }
}
