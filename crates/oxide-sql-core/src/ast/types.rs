//! SQL data type definitions.

use core::fmt;

/// SQL data types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataType {
    // Integer types
    /// Small integer (2 bytes).
    Smallint,
    /// Integer (4 bytes).
    Integer,
    /// Big integer (8 bytes).
    Bigint,

    // Floating point
    /// Real (4-byte float).
    Real,
    /// Double precision (8-byte float).
    Double,
    /// Decimal with precision and scale.
    Decimal {
        /// Total number of digits.
        precision: Option<u16>,
        /// Number of digits after decimal point.
        scale: Option<u16>,
    },
    /// Numeric (alias for Decimal).
    Numeric {
        /// Total number of digits.
        precision: Option<u16>,
        /// Number of digits after decimal point.
        scale: Option<u16>,
    },

    // String types
    /// Fixed-length character string.
    Char(Option<u32>),
    /// Variable-length character string.
    Varchar(Option<u32>),
    /// Text (variable length, no limit).
    Text,

    // Binary types
    /// Binary large object.
    Blob,
    /// Binary with specified length.
    Binary(Option<u32>),
    /// Variable-length binary.
    Varbinary(Option<u32>),

    // Date/time types
    /// Date.
    Date,
    /// Time.
    Time,
    /// Timestamp.
    Timestamp,
    /// DateTime (SQLite-style).
    Datetime,

    // Boolean
    /// Boolean.
    Boolean,

    // Custom type (for database-specific types).
    Custom(String),
}

impl DataType {
    /// Returns the SQL representation of the data type.
    #[must_use]
    pub fn to_sql(&self) -> String {
        match self {
            Self::Smallint => String::from("SMALLINT"),
            Self::Integer => String::from("INTEGER"),
            Self::Bigint => String::from("BIGINT"),
            Self::Real => String::from("REAL"),
            Self::Double => String::from("DOUBLE"),
            Self::Decimal { precision, scale } => match (precision, scale) {
                (Some(p), Some(s)) => format!("DECIMAL({p}, {s})"),
                (Some(p), None) => format!("DECIMAL({p})"),
                _ => String::from("DECIMAL"),
            },
            Self::Numeric { precision, scale } => match (precision, scale) {
                (Some(p), Some(s)) => format!("NUMERIC({p}, {s})"),
                (Some(p), None) => format!("NUMERIC({p})"),
                _ => String::from("NUMERIC"),
            },
            Self::Char(len) => match len {
                Some(n) => format!("CHAR({n})"),
                None => String::from("CHAR"),
            },
            Self::Varchar(len) => match len {
                Some(n) => format!("VARCHAR({n})"),
                None => String::from("VARCHAR"),
            },
            Self::Text => String::from("TEXT"),
            Self::Blob => String::from("BLOB"),
            Self::Binary(len) => match len {
                Some(n) => format!("BINARY({n})"),
                None => String::from("BINARY"),
            },
            Self::Varbinary(len) => match len {
                Some(n) => format!("VARBINARY({n})"),
                None => String::from("VARBINARY"),
            },
            Self::Date => String::from("DATE"),
            Self::Time => String::from("TIME"),
            Self::Timestamp => String::from("TIMESTAMP"),
            Self::Datetime => String::from("DATETIME"),
            Self::Boolean => String::from("BOOLEAN"),
            Self::Custom(name) => name.clone(),
        }
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_sql())
    }
}

/// A column definition for CREATE TABLE.
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDef {
    /// Column name.
    pub name: String,
    /// Data type.
    pub data_type: DataType,
    /// Whether the column is nullable.
    pub nullable: bool,
    /// Default value expression.
    pub default: Option<super::Expr>,
    /// Whether this is a primary key.
    pub primary_key: bool,
    /// Whether this column is unique.
    pub unique: bool,
    /// Whether this column auto-increments.
    pub autoincrement: bool,
}

impl ColumnDef {
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
        }
    }

    /// Sets the column as NOT NULL.
    #[must_use]
    pub fn not_null(mut self) -> Self {
        self.nullable = false;
        self
    }

    /// Sets the column as PRIMARY KEY.
    #[must_use]
    pub fn primary_key(mut self) -> Self {
        self.primary_key = true;
        self.nullable = false; // Primary keys are implicitly NOT NULL
        self
    }

    /// Sets the column as UNIQUE.
    #[must_use]
    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    /// Sets the column as AUTOINCREMENT.
    #[must_use]
    pub fn autoincrement(mut self) -> Self {
        self.autoincrement = true;
        self
    }

    /// Sets the default value.
    #[must_use]
    pub fn default(mut self, expr: super::Expr) -> Self {
        self.default = Some(expr);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_type_to_sql() {
        assert_eq!(DataType::Integer.to_sql(), "INTEGER");
        assert_eq!(DataType::Varchar(Some(255)).to_sql(), "VARCHAR(255)");
        assert_eq!(
            DataType::Decimal {
                precision: Some(10),
                scale: Some(2)
            }
            .to_sql(),
            "DECIMAL(10, 2)"
        );
    }

    #[test]
    fn test_column_def_builder() {
        let col = ColumnDef::new("id", DataType::Integer)
            .primary_key()
            .autoincrement();

        assert_eq!(col.name, "id");
        assert!(col.primary_key);
        assert!(col.autoincrement);
        assert!(!col.nullable);
    }
}
