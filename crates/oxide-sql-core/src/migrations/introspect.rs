//! Schema introspection trait.
//!
//! Driver crates (oxide-sql-sqlite, etc.) implement [`Introspect`]
//! to read the current database schema at runtime. The core crate
//! defines only the trait so it stays driver-agnostic.

use super::snapshot::SchemaSnapshot;

/// Introspects a live database connection to produce a
/// [`SchemaSnapshot`] of the current schema.
///
/// Implementations live in driver crates (e.g. oxide-sql-sqlite).
pub trait Introspect {
    /// Error type for introspection failures.
    type Error: std::error::Error;

    /// Reads the current database schema and returns a snapshot.
    fn introspect_schema(&self) -> Result<SchemaSnapshot, Self::Error>;
}

/// Helper constants and functions for implementing [`Introspect`]
/// on SQLite connections. No driver dependency — just SQL strings
/// and type-mapping logic that any SQLite driver crate can use.
pub mod sqlite_helpers {
    use crate::ast::DataType;
    use crate::migrations::column_builder::DefaultValue;
    use crate::migrations::snapshot::ColumnSnapshot;

    /// SQL to list all user tables (excludes internal SQLite
    /// tables).
    pub const LIST_TABLES: &str = "SELECT name FROM sqlite_master WHERE type='table' \
         AND name NOT LIKE 'sqlite_%' ORDER BY name";

    /// PRAGMA to get column info for a table.
    /// Replace `{table}` with the actual table name.
    pub const TABLE_INFO: &str = "PRAGMA table_info({table})";

    /// PRAGMA to get the index list for a table.
    /// Replace `{table}` with the actual table name.
    pub const INDEX_LIST: &str = "PRAGMA index_list({table})";

    /// PRAGMA to get the columns of an index.
    /// Replace `{index}` with the actual index name.
    pub const INDEX_INFO: &str = "PRAGMA index_info({index})";

    /// PRAGMA to get foreign key list for a table.
    /// Replace `{table}` with the actual table name.
    pub const FOREIGN_KEY_LIST: &str = "PRAGMA foreign_key_list({table})";

    /// Maps a SQLite type affinity string to a [`DataType`].
    ///
    /// SQLite is flexible about type names; this function handles
    /// the most common forms returned by `PRAGMA table_info`.
    #[must_use]
    pub fn parse_sqlite_type(type_str: &str) -> DataType {
        let upper = type_str.to_uppercase();
        let upper = upper.trim();
        match upper.as_ref() {
            "INTEGER" | "INT" => DataType::Integer,
            "BIGINT" => DataType::Bigint,
            "SMALLINT" | "TINYINT" => DataType::Smallint,
            "REAL" | "FLOAT" => DataType::Real,
            "DOUBLE" | "DOUBLE PRECISION" => DataType::Double,
            "TEXT" | "CLOB" => DataType::Text,
            "BLOB" => DataType::Blob,
            "BOOLEAN" | "BOOL" => DataType::Integer,
            "DATE" => DataType::Date,
            "DATETIME" | "TIMESTAMP" => DataType::Datetime,
            s if s.starts_with("VARCHAR") => {
                let len = extract_length(s);
                DataType::Varchar(len)
            }
            s if s.starts_with("CHAR") => {
                let len = extract_length(s);
                DataType::Char(len)
            }
            s if s.starts_with("NUMERIC") || s.starts_with("DECIMAL") => DataType::Real,
            _ => DataType::Text,
        }
    }

    /// Extracts the length from a type like "VARCHAR(255)".
    fn extract_length(s: &str) -> Option<u32> {
        s.find('(')
            .and_then(|start| s.find(')').map(|end| (start, end)))
            .and_then(|(start, end)| s[start + 1..end].trim().parse::<u32>().ok())
    }

    /// Builds a [`ColumnSnapshot`] from raw `PRAGMA table_info`
    /// row data.
    ///
    /// # Arguments
    ///
    /// * `name` — Column name.
    /// * `type_str` — Type string from SQLite (e.g. "INTEGER",
    ///   "VARCHAR(255)").
    /// * `notnull` — Whether NOT NULL is set.
    /// * `default_value` — Default value expression, if any.
    /// * `pk` — Whether this column is part of the primary key.
    #[must_use]
    pub fn column_from_pragma(
        name: &str,
        type_str: &str,
        notnull: bool,
        default_value: Option<&str>,
        pk: bool,
    ) -> ColumnSnapshot {
        let data_type = parse_sqlite_type(type_str);
        let default = default_value.map(|v| {
            if v == "NULL" {
                DefaultValue::Null
            } else if v == "TRUE" || v == "FALSE" {
                DefaultValue::Expression(v.to_string())
            } else if let Ok(i) = v.parse::<i64>() {
                DefaultValue::Integer(i)
            } else if let Ok(f) = v.parse::<f64>() {
                DefaultValue::Float(f)
            } else if v.starts_with('\'') && v.ends_with('\'') {
                DefaultValue::String(v[1..v.len() - 1].replace("''", "'"))
            } else {
                DefaultValue::Expression(v.to_string())
            }
        });
        ColumnSnapshot {
            name: name.to_string(),
            data_type,
            nullable: !notnull,
            primary_key: pk,
            unique: false,
            autoincrement: false,
            default,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn parse_common_types() {
            assert_eq!(parse_sqlite_type("INTEGER"), DataType::Integer);
            assert_eq!(parse_sqlite_type("BIGINT"), DataType::Bigint);
            assert_eq!(parse_sqlite_type("TEXT"), DataType::Text);
            assert_eq!(parse_sqlite_type("BLOB"), DataType::Blob);
            assert_eq!(parse_sqlite_type("REAL"), DataType::Real);
            assert_eq!(
                parse_sqlite_type("VARCHAR(255)"),
                DataType::Varchar(Some(255))
            );
            assert_eq!(parse_sqlite_type("CHAR(10)"), DataType::Char(Some(10)));
        }

        #[test]
        fn column_from_pragma_basic() {
            let col = column_from_pragma("id", "INTEGER", true, None, true);
            assert_eq!(col.name, "id");
            assert_eq!(col.data_type, DataType::Integer);
            assert!(!col.nullable);
            assert!(col.primary_key);
            assert!(col.default.is_none());
        }

        #[test]
        fn column_from_pragma_with_default() {
            let col = column_from_pragma("active", "INTEGER", true, Some("TRUE"), false);
            assert_eq!(col.default, Some(DefaultValue::Expression("TRUE".into())));

            let col = column_from_pragma("count", "INTEGER", false, Some("42"), false);
            assert_eq!(col.default, Some(DefaultValue::Integer(42)));
        }
    }
}
