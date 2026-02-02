//! Schema representation types.
//!
//! These types describe the structure of database tables and are used by both
//! the model layer (to describe what the code expects) and the migration engine
//! (to track what migrations have created).

use serde::{Deserialize, Serialize};

/// SQL data types supported by the migration system.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SqlType {
    /// Integer (32-bit).
    Integer,
    /// Big integer (64-bit).
    BigInt,
    /// Small integer (16-bit).
    SmallInt,
    /// Text/string with optional max length.
    Text,
    /// Variable-length character string.
    Varchar(usize),
    /// Fixed-length character string.
    Char(usize),
    /// Boolean.
    Boolean,
    /// Date and time.
    DateTime,
    /// Date only.
    Date,
    /// Time only.
    Time,
    /// Timestamp (alias for DateTime in most databases).
    Timestamp,
    /// Floating point (single precision).
    Real,
    /// Floating point (double precision).
    Double,
    /// Decimal with precision and scale.
    Decimal(u8, u8),
    /// Numeric (alias for Decimal).
    Numeric(u8, u8),
    /// Binary large object.
    Blob,
    /// Binary data with max length.
    Binary(usize),
    /// Variable-length binary data.
    VarBinary(usize),
    /// JSON data.
    Json,
    /// UUID.
    Uuid,
}

impl SqlType {
    /// Returns the SQL type name for SQLite.
    #[must_use]
    pub fn sqlite_name(&self) -> &'static str {
        match self {
            Self::Integer | Self::SmallInt => "INTEGER",
            Self::BigInt => "INTEGER", // SQLite uses INTEGER for all ints
            Self::Text | Self::Varchar(_) | Self::Char(_) => "TEXT",
            Self::Boolean => "INTEGER", // SQLite stores booleans as 0/1
            Self::DateTime | Self::Date | Self::Time | Self::Timestamp => "TEXT",
            Self::Real => "REAL",
            Self::Double => "REAL",
            Self::Decimal(_, _) | Self::Numeric(_, _) => "NUMERIC",
            Self::Blob | Self::Binary(_) | Self::VarBinary(_) => "BLOB",
            Self::Json => "TEXT",
            Self::Uuid => "TEXT",
        }
    }

    /// Returns the SQL type name for PostgreSQL.
    #[must_use]
    pub fn postgres_name(&self) -> String {
        match self {
            Self::Integer => "INTEGER".to_string(),
            Self::BigInt => "BIGINT".to_string(),
            Self::SmallInt => "SMALLINT".to_string(),
            Self::Text => "TEXT".to_string(),
            Self::Varchar(len) => format!("VARCHAR({})", len),
            Self::Char(len) => format!("CHAR({})", len),
            Self::Boolean => "BOOLEAN".to_string(),
            Self::DateTime | Self::Timestamp => "TIMESTAMP".to_string(),
            Self::Date => "DATE".to_string(),
            Self::Time => "TIME".to_string(),
            Self::Real => "REAL".to_string(),
            Self::Double => "DOUBLE PRECISION".to_string(),
            Self::Decimal(p, s) => format!("DECIMAL({}, {})", p, s),
            Self::Numeric(p, s) => format!("NUMERIC({}, {})", p, s),
            Self::Blob => "BYTEA".to_string(),
            Self::Binary(len) | Self::VarBinary(len) => format!("BYTEA({})", len),
            Self::Json => "JSONB".to_string(),
            Self::Uuid => "UUID".to_string(),
        }
    }
}

/// Default value for a column.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DefaultValue {
    /// No default value.
    None,
    /// NULL default.
    Null,
    /// Boolean default.
    Bool(bool),
    /// Integer default.
    Integer(i64),
    /// Float default.
    Float(f64),
    /// String default.
    String(String),
    /// SQL expression (e.g., "CURRENT_TIMESTAMP").
    Expression(String),
}

impl DefaultValue {
    /// Returns the SQL representation of this default value.
    #[must_use]
    pub fn to_sql(&self) -> Option<String> {
        match self {
            Self::None => None,
            Self::Null => Some("NULL".to_string()),
            Self::Bool(b) => Some(if *b { "1" } else { "0" }.to_string()),
            Self::Integer(i) => Some(i.to_string()),
            Self::Float(f) => Some(f.to_string()),
            Self::String(s) => Some(format!("'{}'", s.replace('\'', "''"))),
            Self::Expression(expr) => Some(expr.clone()),
        }
    }
}

/// Foreign key action (ON DELETE, ON UPDATE).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ForeignKeyAction {
    /// No action (error if referenced row is deleted/updated).
    #[default]
    NoAction,
    /// Restrict (same as NoAction but checked immediately).
    Restrict,
    /// Cascade the delete/update to referencing rows.
    Cascade,
    /// Set the foreign key column to NULL.
    SetNull,
    /// Set the foreign key column to its default value.
    SetDefault,
}

impl ForeignKeyAction {
    /// Returns the SQL representation of this action.
    #[must_use]
    pub fn to_sql(&self) -> &'static str {
        match self {
            Self::NoAction => "NO ACTION",
            Self::Restrict => "RESTRICT",
            Self::Cascade => "CASCADE",
            Self::SetNull => "SET NULL",
            Self::SetDefault => "SET DEFAULT",
        }
    }
}

/// Schema definition for a column.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnSchema {
    /// Column name.
    pub name: String,
    /// SQL data type.
    pub sql_type: SqlType,
    /// Whether the column allows NULL values.
    pub nullable: bool,
    /// Default value.
    pub default: DefaultValue,
    /// Whether this column is part of the primary key.
    pub primary_key: bool,
    /// Whether this column auto-increments.
    pub auto_increment: bool,
    /// Whether this column has a UNIQUE constraint.
    pub unique: bool,
    /// Check constraint expression (if any).
    pub check: Option<String>,
}

impl ColumnSchema {
    /// Creates a new column schema.
    #[must_use]
    pub fn new(name: impl Into<String>, sql_type: SqlType) -> Self {
        Self {
            name: name.into(),
            sql_type,
            nullable: true,
            default: DefaultValue::None,
            primary_key: false,
            auto_increment: false,
            unique: false,
            check: None,
        }
    }

    /// Sets the column as NOT NULL.
    #[must_use]
    pub fn not_null(mut self) -> Self {
        self.nullable = false;
        self
    }

    /// Sets the column as nullable.
    #[must_use]
    pub fn nullable(mut self) -> Self {
        self.nullable = true;
        self
    }

    /// Sets the default value.
    #[must_use]
    pub fn default(mut self, value: DefaultValue) -> Self {
        self.default = value;
        self
    }

    /// Sets the column as the primary key.
    #[must_use]
    pub fn primary_key(mut self) -> Self {
        self.primary_key = true;
        self.nullable = false; // Primary keys are always NOT NULL
        self
    }

    /// Sets the column to auto-increment.
    #[must_use]
    pub fn auto_increment(mut self) -> Self {
        self.auto_increment = true;
        self
    }

    /// Sets the column as unique.
    #[must_use]
    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    /// Sets a check constraint.
    #[must_use]
    pub fn check(mut self, expr: impl Into<String>) -> Self {
        self.check = Some(expr.into());
        self
    }
}

/// Schema definition for a foreign key constraint.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ForeignKeySchema {
    /// Constraint name.
    pub name: String,
    /// Column(s) in the referencing table.
    pub columns: Vec<String>,
    /// Referenced table name.
    pub references_table: String,
    /// Referenced column(s).
    pub references_columns: Vec<String>,
    /// Action on delete.
    pub on_delete: ForeignKeyAction,
    /// Action on update.
    pub on_update: ForeignKeyAction,
}

/// Schema definition for an index.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IndexSchema {
    /// Index name.
    pub name: String,
    /// Columns included in the index.
    pub columns: Vec<String>,
    /// Whether this is a unique index.
    pub unique: bool,
    /// Partial index condition (WHERE clause).
    pub condition: Option<String>,
}

/// Schema definition for a unique constraint.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UniqueConstraint {
    /// Constraint name.
    pub name: String,
    /// Columns that form the unique constraint.
    pub columns: Vec<String>,
}

/// Complete schema definition for a table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableSchema {
    /// Table name.
    pub name: String,
    /// Column definitions.
    pub columns: Vec<ColumnSchema>,
    /// Primary key column(s).
    pub primary_key: Vec<String>,
    /// Index definitions.
    pub indexes: Vec<IndexSchema>,
    /// Foreign key definitions.
    pub foreign_keys: Vec<ForeignKeySchema>,
    /// Unique constraint definitions.
    pub unique_constraints: Vec<UniqueConstraint>,
}

impl TableSchema {
    /// Creates a new table schema.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            columns: Vec::new(),
            primary_key: Vec::new(),
            indexes: Vec::new(),
            foreign_keys: Vec::new(),
            unique_constraints: Vec::new(),
        }
    }

    /// Adds a column to the table.
    #[must_use]
    pub fn column(mut self, column: ColumnSchema) -> Self {
        if column.primary_key && !self.primary_key.contains(&column.name) {
            self.primary_key.push(column.name.clone());
        }
        self.columns.push(column);
        self
    }

    /// Sets the primary key columns.
    #[must_use]
    pub fn primary_key(mut self, columns: Vec<String>) -> Self {
        self.primary_key = columns;
        self
    }

    /// Adds an index.
    #[must_use]
    pub fn index(mut self, index: IndexSchema) -> Self {
        self.indexes.push(index);
        self
    }

    /// Adds a foreign key.
    #[must_use]
    pub fn foreign_key(mut self, fk: ForeignKeySchema) -> Self {
        self.foreign_keys.push(fk);
        self
    }

    /// Adds a unique constraint.
    #[must_use]
    pub fn unique_constraint(mut self, constraint: UniqueConstraint) -> Self {
        self.unique_constraints.push(constraint);
        self
    }

    /// Gets a column by name.
    #[must_use]
    pub fn get_column(&self, name: &str) -> Option<&ColumnSchema> {
        self.columns.iter().find(|c| c.name == name)
    }

    /// Gets a mutable column by name.
    #[must_use]
    pub fn get_column_mut(&mut self, name: &str) -> Option<&mut ColumnSchema> {
        self.columns.iter_mut().find(|c| c.name == name)
    }
}

/// The complete database schema (all tables).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DatabaseSchema {
    /// All tables in the database.
    pub tables: Vec<TableSchema>,
}

impl DatabaseSchema {
    /// Creates a new empty database schema.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a table to the schema.
    #[must_use]
    pub fn table(mut self, table: TableSchema) -> Self {
        self.tables.push(table);
        self
    }

    /// Gets a table by name.
    #[must_use]
    pub fn get_table(&self, name: &str) -> Option<&TableSchema> {
        self.tables.iter().find(|t| t.name == name)
    }

    /// Gets a mutable table by name.
    #[must_use]
    pub fn get_table_mut(&mut self, name: &str) -> Option<&mut TableSchema> {
        self.tables.iter_mut().find(|t| t.name == name)
    }

    /// Returns table names.
    pub fn table_names(&self) -> impl Iterator<Item = &str> {
        self.tables.iter().map(|t| t.name.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_schema_builder() {
        let col = ColumnSchema::new("id", SqlType::BigInt)
            .primary_key()
            .auto_increment();

        assert_eq!(col.name, "id");
        assert!(col.primary_key);
        assert!(col.auto_increment);
        assert!(!col.nullable); // Primary keys are NOT NULL
    }

    #[test]
    fn test_table_schema_builder() {
        let table = TableSchema::new("users")
            .column(ColumnSchema::new("id", SqlType::BigInt).primary_key())
            .column(ColumnSchema::new("name", SqlType::Varchar(255)).not_null())
            .column(ColumnSchema::new("email", SqlType::Varchar(255)));

        assert_eq!(table.name, "users");
        assert_eq!(table.columns.len(), 3);
        assert_eq!(table.primary_key, vec!["id"]);
    }

    #[test]
    fn test_default_value_to_sql() {
        assert_eq!(DefaultValue::None.to_sql(), None);
        assert_eq!(DefaultValue::Null.to_sql(), Some("NULL".to_string()));
        assert_eq!(DefaultValue::Bool(true).to_sql(), Some("1".to_string()));
        assert_eq!(DefaultValue::Integer(42).to_sql(), Some("42".to_string()));
        assert_eq!(
            DefaultValue::String("hello".to_string()).to_sql(),
            Some("'hello'".to_string())
        );
        assert_eq!(
            DefaultValue::Expression("CURRENT_TIMESTAMP".to_string()).to_sql(),
            Some("CURRENT_TIMESTAMP".to_string())
        );
    }

    #[test]
    fn test_sql_type_names() {
        assert_eq!(SqlType::BigInt.sqlite_name(), "INTEGER");
        assert_eq!(SqlType::BigInt.postgres_name(), "BIGINT");
        assert_eq!(SqlType::Varchar(255).postgres_name(), "VARCHAR(255)");
        assert_eq!(SqlType::Decimal(10, 2).postgres_name(), "DECIMAL(10, 2)");
    }
}
