//! Type-safe column definition builder.
//!
//! Provides a fluent API for defining columns in migrations with compile-time
//! validation of constraints.

use crate::ast::DataType;

/// A reference to a foreign key in another table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForeignKeyRef {
    /// The referenced table name.
    pub table: String,
    /// The referenced column name.
    pub column: String,
    /// Action on delete.
    pub on_delete: Option<ForeignKeyAction>,
    /// Action on update.
    pub on_update: Option<ForeignKeyAction>,
}

/// Foreign key referential action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForeignKeyAction {
    /// No action.
    NoAction,
    /// Restrict deletion/update.
    Restrict,
    /// Cascade the operation.
    Cascade,
    /// Set to NULL.
    SetNull,
    /// Set to default value.
    SetDefault,
}

impl ForeignKeyAction {
    /// Returns the SQL representation of the action.
    #[must_use]
    pub fn as_sql(self) -> &'static str {
        match self {
            Self::NoAction => "NO ACTION",
            Self::Restrict => "RESTRICT",
            Self::Cascade => "CASCADE",
            Self::SetNull => "SET NULL",
            Self::SetDefault => "SET DEFAULT",
        }
    }
}

/// Default value for a column.
#[derive(Debug, Clone, PartialEq)]
pub enum DefaultValue {
    /// NULL default.
    Null,
    /// Boolean default.
    Boolean(bool),
    /// Integer default.
    Integer(i64),
    /// Float default.
    Float(f64),
    /// String default.
    String(String),
    /// Raw SQL expression (e.g., CURRENT_TIMESTAMP).
    Expression(String),
}

impl DefaultValue {
    /// Returns the SQL representation of the default value.
    #[must_use]
    pub fn to_sql(&self) -> String {
        match self {
            Self::Null => String::from("NULL"),
            Self::Boolean(b) => {
                if *b {
                    String::from("TRUE")
                } else {
                    String::from("FALSE")
                }
            }
            Self::Integer(i) => i.to_string(),
            Self::Float(f) => f.to_string(),
            Self::String(s) => format!("'{}'", s.replace('\'', "''")),
            Self::Expression(expr) => expr.clone(),
        }
    }
}

/// A complete column definition for migrations.
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDefinition {
    /// Column name.
    pub name: String,
    /// Data type.
    pub data_type: DataType,
    /// Whether the column is nullable.
    pub nullable: bool,
    /// Default value.
    pub default: Option<DefaultValue>,
    /// Whether this is a primary key.
    pub primary_key: bool,
    /// Whether this column is unique.
    pub unique: bool,
    /// Whether this column auto-increments.
    pub autoincrement: bool,
    /// Foreign key reference, if any.
    pub references: Option<ForeignKeyRef>,
    /// Check constraint expression, if any.
    pub check: Option<String>,
    /// Collation for string columns.
    pub collation: Option<String>,
}

impl ColumnDefinition {
    /// Creates a new column definition.
    #[must_use]
    pub fn new(name: impl Into<String>, data_type: DataType) -> Self {
        Self {
            name: name.into(),
            data_type,
            nullable: true,
            default: None,
            primary_key: false,
            unique: false,
            autoincrement: false,
            references: None,
            check: None,
            collation: None,
        }
    }
}

/// Type-safe column definition builder.
///
/// Provides a fluent API for building column definitions with all constraints.
#[derive(Debug, Clone)]
pub struct ColumnBuilder {
    name: String,
    data_type: DataType,
    nullable: bool,
    default: Option<DefaultValue>,
    primary_key: bool,
    unique: bool,
    autoincrement: bool,
    references: Option<ForeignKeyRef>,
    check: Option<String>,
    collation: Option<String>,
}

impl ColumnBuilder {
    /// Creates a new column builder with name and type.
    #[must_use]
    pub fn new(name: impl Into<String>, data_type: DataType) -> Self {
        Self {
            name: name.into(),
            data_type,
            nullable: true,
            default: None,
            primary_key: false,
            unique: false,
            autoincrement: false,
            references: None,
            check: None,
            collation: None,
        }
    }

    /// Marks the column as NOT NULL.
    #[must_use]
    pub fn not_null(mut self) -> Self {
        self.nullable = false;
        self
    }

    /// Marks the column as nullable (default).
    #[must_use]
    pub fn nullable(mut self) -> Self {
        self.nullable = true;
        self
    }

    /// Marks the column as PRIMARY KEY.
    #[must_use]
    pub fn primary_key(mut self) -> Self {
        self.primary_key = true;
        self.nullable = false; // Primary keys are implicitly NOT NULL
        self
    }

    /// Marks the column as UNIQUE.
    #[must_use]
    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    /// Marks the column as AUTOINCREMENT.
    #[must_use]
    pub fn autoincrement(mut self) -> Self {
        self.autoincrement = true;
        self
    }

    /// Sets a boolean default value.
    #[must_use]
    pub fn default_bool(mut self, value: bool) -> Self {
        self.default = Some(DefaultValue::Boolean(value));
        self
    }

    /// Sets an integer default value.
    #[must_use]
    pub fn default_int(mut self, value: i64) -> Self {
        self.default = Some(DefaultValue::Integer(value));
        self
    }

    /// Sets a float default value.
    #[must_use]
    pub fn default_float(mut self, value: f64) -> Self {
        self.default = Some(DefaultValue::Float(value));
        self
    }

    /// Sets a string default value.
    #[must_use]
    pub fn default_str(mut self, value: impl Into<String>) -> Self {
        self.default = Some(DefaultValue::String(value.into()));
        self
    }

    /// Sets a NULL default value.
    #[must_use]
    pub fn default_null(mut self) -> Self {
        self.default = Some(DefaultValue::Null);
        self
    }

    /// Sets a raw SQL expression as default (e.g., CURRENT_TIMESTAMP).
    #[must_use]
    pub fn default_expr(mut self, expr: impl Into<String>) -> Self {
        self.default = Some(DefaultValue::Expression(expr.into()));
        self
    }

    /// Sets a foreign key reference.
    #[must_use]
    pub fn references(mut self, table: impl Into<String>, column: impl Into<String>) -> Self {
        self.references = Some(ForeignKeyRef {
            table: table.into(),
            column: column.into(),
            on_delete: None,
            on_update: None,
        });
        self
    }

    /// Sets a foreign key reference with ON DELETE action.
    #[must_use]
    pub fn references_on_delete(
        mut self,
        table: impl Into<String>,
        column: impl Into<String>,
        on_delete: ForeignKeyAction,
    ) -> Self {
        self.references = Some(ForeignKeyRef {
            table: table.into(),
            column: column.into(),
            on_delete: Some(on_delete),
            on_update: None,
        });
        self
    }

    /// Sets a foreign key reference with full options.
    #[must_use]
    pub fn references_full(
        mut self,
        table: impl Into<String>,
        column: impl Into<String>,
        on_delete: Option<ForeignKeyAction>,
        on_update: Option<ForeignKeyAction>,
    ) -> Self {
        self.references = Some(ForeignKeyRef {
            table: table.into(),
            column: column.into(),
            on_delete,
            on_update,
        });
        self
    }

    /// Adds a CHECK constraint.
    #[must_use]
    pub fn check(mut self, expr: impl Into<String>) -> Self {
        self.check = Some(expr.into());
        self
    }

    /// Sets the collation for string columns.
    #[must_use]
    pub fn collation(mut self, collation: impl Into<String>) -> Self {
        self.collation = Some(collation.into());
        self
    }

    /// Builds the column definition.
    #[must_use]
    pub fn build(self) -> ColumnDefinition {
        ColumnDefinition {
            name: self.name,
            data_type: self.data_type,
            nullable: self.nullable,
            default: self.default,
            primary_key: self.primary_key,
            unique: self.unique,
            autoincrement: self.autoincrement,
            references: self.references,
            check: self.check,
            collation: self.collation,
        }
    }
}

// =============================================================================
// Shorthand Functions for Common Types
// =============================================================================

/// Creates an INTEGER column builder.
#[must_use]
pub fn integer(name: impl Into<String>) -> ColumnBuilder {
    ColumnBuilder::new(name, DataType::Integer)
}

/// Creates a SMALLINT column builder.
#[must_use]
pub fn smallint(name: impl Into<String>) -> ColumnBuilder {
    ColumnBuilder::new(name, DataType::Smallint)
}

/// Creates a BIGINT column builder.
#[must_use]
pub fn bigint(name: impl Into<String>) -> ColumnBuilder {
    ColumnBuilder::new(name, DataType::Bigint)
}

/// Creates a REAL column builder.
#[must_use]
pub fn real(name: impl Into<String>) -> ColumnBuilder {
    ColumnBuilder::new(name, DataType::Real)
}

/// Creates a DOUBLE column builder.
#[must_use]
pub fn double(name: impl Into<String>) -> ColumnBuilder {
    ColumnBuilder::new(name, DataType::Double)
}

/// Creates a DECIMAL column builder.
#[must_use]
pub fn decimal(name: impl Into<String>, precision: u16, scale: u16) -> ColumnBuilder {
    ColumnBuilder::new(
        name,
        DataType::Decimal {
            precision: Some(precision),
            scale: Some(scale),
        },
    )
}

/// Creates a NUMERIC column builder.
#[must_use]
pub fn numeric(name: impl Into<String>, precision: u16, scale: u16) -> ColumnBuilder {
    ColumnBuilder::new(
        name,
        DataType::Numeric {
            precision: Some(precision),
            scale: Some(scale),
        },
    )
}

/// Creates a CHAR column builder.
#[must_use]
pub fn char(name: impl Into<String>, len: u32) -> ColumnBuilder {
    ColumnBuilder::new(name, DataType::Char(Some(len)))
}

/// Creates a VARCHAR column builder.
#[must_use]
pub fn varchar(name: impl Into<String>, len: u32) -> ColumnBuilder {
    ColumnBuilder::new(name, DataType::Varchar(Some(len)))
}

/// Creates a TEXT column builder.
#[must_use]
pub fn text(name: impl Into<String>) -> ColumnBuilder {
    ColumnBuilder::new(name, DataType::Text)
}

/// Creates a BLOB column builder.
#[must_use]
pub fn blob(name: impl Into<String>) -> ColumnBuilder {
    ColumnBuilder::new(name, DataType::Blob)
}

/// Creates a BINARY column builder.
#[must_use]
pub fn binary(name: impl Into<String>, len: u32) -> ColumnBuilder {
    ColumnBuilder::new(name, DataType::Binary(Some(len)))
}

/// Creates a VARBINARY column builder.
#[must_use]
pub fn varbinary(name: impl Into<String>, len: u32) -> ColumnBuilder {
    ColumnBuilder::new(name, DataType::Varbinary(Some(len)))
}

/// Creates a DATE column builder.
#[must_use]
pub fn date(name: impl Into<String>) -> ColumnBuilder {
    ColumnBuilder::new(name, DataType::Date)
}

/// Creates a TIME column builder.
#[must_use]
pub fn time(name: impl Into<String>) -> ColumnBuilder {
    ColumnBuilder::new(name, DataType::Time)
}

/// Creates a TIMESTAMP column builder.
#[must_use]
pub fn timestamp(name: impl Into<String>) -> ColumnBuilder {
    ColumnBuilder::new(name, DataType::Timestamp)
}

/// Creates a DATETIME column builder.
#[must_use]
pub fn datetime(name: impl Into<String>) -> ColumnBuilder {
    ColumnBuilder::new(name, DataType::Datetime)
}

/// Creates a BOOLEAN column builder.
#[must_use]
pub fn boolean(name: impl Into<String>) -> ColumnBuilder {
    ColumnBuilder::new(name, DataType::Boolean)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_column() {
        let col = integer("id").build();
        assert_eq!(col.name, "id");
        assert_eq!(col.data_type, DataType::Integer);
        assert!(col.nullable);
        assert!(!col.primary_key);
    }

    #[test]
    fn test_primary_key_column() {
        let col = bigint("id").primary_key().autoincrement().build();
        assert_eq!(col.name, "id");
        assert!(col.primary_key);
        assert!(col.autoincrement);
        assert!(!col.nullable); // Primary key implies NOT NULL
    }

    #[test]
    fn test_varchar_column() {
        let col = varchar("username", 255).not_null().unique().build();
        assert_eq!(col.data_type, DataType::Varchar(Some(255)));
        assert!(!col.nullable);
        assert!(col.unique);
    }

    #[test]
    fn test_column_with_default() {
        let col = boolean("active").not_null().default_bool(true).build();
        assert_eq!(col.default, Some(DefaultValue::Boolean(true)));

        let col = integer("count").default_int(0).build();
        assert_eq!(col.default, Some(DefaultValue::Integer(0)));

        let col = timestamp("created_at")
            .not_null()
            .default_expr("CURRENT_TIMESTAMP")
            .build();
        assert_eq!(
            col.default,
            Some(DefaultValue::Expression("CURRENT_TIMESTAMP".to_string()))
        );
    }

    #[test]
    fn test_foreign_key_column() {
        let col = bigint("user_id")
            .not_null()
            .references_on_delete("users", "id", ForeignKeyAction::Cascade)
            .build();

        assert!(col.references.is_some());
        let fk = col.references.unwrap();
        assert_eq!(fk.table, "users");
        assert_eq!(fk.column, "id");
        assert_eq!(fk.on_delete, Some(ForeignKeyAction::Cascade));
    }

    #[test]
    fn test_column_with_check() {
        let col = integer("age").not_null().check("age >= 0").build();
        assert_eq!(col.check, Some("age >= 0".to_string()));
    }

    #[test]
    fn test_default_value_to_sql() {
        assert_eq!(DefaultValue::Null.to_sql(), "NULL");
        assert_eq!(DefaultValue::Boolean(true).to_sql(), "TRUE");
        assert_eq!(DefaultValue::Boolean(false).to_sql(), "FALSE");
        assert_eq!(DefaultValue::Integer(42).to_sql(), "42");
        assert_eq!(DefaultValue::Float(3.14).to_sql(), "3.14");
        assert_eq!(DefaultValue::String("hello".into()).to_sql(), "'hello'");
        assert_eq!(DefaultValue::String("it's".into()).to_sql(), "'it''s'"); // Escaped
        assert_eq!(
            DefaultValue::Expression("CURRENT_TIMESTAMP".into()).to_sql(),
            "CURRENT_TIMESTAMP"
        );
    }
}
