//! Database dialect implementations.
//!
//! Each dialect knows how to generate SQL for migration operations
//! specific to that database system.

mod sqlite;

pub use sqlite::SqliteDialect;

use crate::operations::MigrationOperation;
use crate::schema::{ColumnSchema, SqlType};

/// Trait for database-specific SQL generation.
pub trait MigrationDialect: Send + Sync {
    /// Returns the dialect name.
    fn name(&self) -> &'static str;

    /// Generates SQL for a migration operation.
    fn generate_sql(&self, operation: &MigrationOperation) -> Vec<String>;

    /// Returns the SQL type name for the given type.
    fn type_name(&self, sql_type: &SqlType) -> String;

    /// Returns whether this dialect supports ALTER COLUMN.
    fn supports_alter_column(&self) -> bool;

    /// Returns whether this dialect supports DROP COLUMN.
    fn supports_drop_column(&self) -> bool;

    /// Returns whether this dialect supports adding constraints after table creation.
    fn supports_add_constraint(&self) -> bool;

    /// Generates column definition SQL.
    fn column_definition(&self, column: &ColumnSchema) -> String {
        let mut parts = vec![
            format!("\"{}\"", column.name),
            self.type_name(&column.sql_type),
        ];

        if column.primary_key {
            parts.push("PRIMARY KEY".to_string());
            if column.auto_increment {
                parts.push(self.auto_increment_keyword().to_string());
            }
        }

        if !column.nullable && !column.primary_key {
            parts.push("NOT NULL".to_string());
        }

        if column.unique && !column.primary_key {
            parts.push("UNIQUE".to_string());
        }

        if let Some(default_sql) = column.default.to_sql() {
            parts.push(format!("DEFAULT {}", default_sql));
        }

        if let Some(ref check) = column.check {
            parts.push(format!("CHECK ({})", check));
        }

        parts.join(" ")
    }

    /// Returns the auto-increment keyword for this dialect.
    fn auto_increment_keyword(&self) -> &'static str;

    /// Quote an identifier (table name, column name, etc.).
    fn quote_identifier(&self, name: &str) -> String {
        format!("\"{}\"", name)
    }
}
