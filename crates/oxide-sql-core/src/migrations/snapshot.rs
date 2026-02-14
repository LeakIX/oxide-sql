//! Schema snapshot types for diff-based migration generation.
//!
//! Dialect-resolved representations of database schemas used for
//! comparison. Unlike [`ColumnSchema`](crate::schema::ColumnSchema)
//! (which stores Rust type strings), snapshots store resolved
//! [`DataType`](crate::ast::DataType) values.

use std::collections::BTreeMap;

use crate::ast::DataType;
use crate::schema::{RustTypeMapping, TableSchema};

use super::column_builder::{DefaultValue, ForeignKeyAction};
use super::operation::{IndexType, strip_option};

/// A snapshot of a database index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexSnapshot {
    /// Index name.
    pub name: String,
    /// Columns covered by the index.
    pub columns: Vec<String>,
    /// Whether this is a UNIQUE index.
    pub unique: bool,
    /// Index type (BTree, Hash, etc.).
    pub index_type: IndexType,
    /// Partial index condition (WHERE clause), if any.
    pub condition: Option<String>,
}

/// A snapshot of a foreign key constraint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForeignKeySnapshot {
    /// Optional constraint name.
    pub name: Option<String>,
    /// Columns in this table.
    pub columns: Vec<String>,
    /// Referenced table.
    pub references_table: String,
    /// Referenced columns.
    pub references_columns: Vec<String>,
    /// ON DELETE action.
    pub on_delete: Option<ForeignKeyAction>,
    /// ON UPDATE action.
    pub on_update: Option<ForeignKeyAction>,
}

/// A snapshot of a single column's resolved schema.
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnSnapshot {
    /// Column name.
    pub name: String,
    /// Resolved SQL data type.
    pub data_type: DataType,
    /// Whether the column is nullable.
    pub nullable: bool,
    /// Whether this column is a primary key.
    pub primary_key: bool,
    /// Whether this column has a UNIQUE constraint.
    pub unique: bool,
    /// Whether this column auto-increments.
    pub autoincrement: bool,
    /// Default value, if any.
    pub default: Option<DefaultValue>,
}

/// A snapshot of a single table's resolved schema.
#[derive(Debug, Clone, PartialEq)]
pub struct TableSnapshot {
    /// Table name.
    pub name: String,
    /// Columns in declaration order.
    pub columns: Vec<ColumnSnapshot>,
    /// Indexes on this table.
    pub indexes: Vec<IndexSnapshot>,
    /// Foreign key constraints on this table.
    pub foreign_keys: Vec<ForeignKeySnapshot>,
}

impl TableSnapshot {
    /// Builds a snapshot from a `#[derive(Table)]` struct, resolving
    /// Rust types to SQL `DataType` via the dialect's
    /// `RustTypeMapping`.
    pub fn from_table_schema<T: TableSchema>(dialect: &impl RustTypeMapping) -> Self {
        let columns = T::SCHEMA
            .iter()
            .map(|col| {
                let inner = strip_option(col.rust_type);
                let data_type = dialect.map_type(inner);
                let default = col
                    .default_expr
                    .map(|expr| DefaultValue::Expression(expr.to_string()));
                ColumnSnapshot {
                    name: col.name.to_string(),
                    data_type,
                    nullable: col.nullable,
                    primary_key: col.primary_key,
                    unique: col.unique,
                    autoincrement: col.autoincrement,
                    default,
                }
            })
            .collect();
        Self {
            name: T::NAME.to_string(),
            columns,
            indexes: vec![],
            foreign_keys: vec![],
        }
    }

    /// Looks up a column by name.
    #[must_use]
    pub fn column(&self, name: &str) -> Option<&ColumnSnapshot> {
        self.columns.iter().find(|c| c.name == name)
    }
}

/// A snapshot of an entire database schema (multiple tables).
#[derive(Debug, Clone, PartialEq)]
pub struct SchemaSnapshot {
    /// Tables keyed by name, sorted for deterministic iteration.
    pub tables: BTreeMap<String, TableSnapshot>,
}

impl SchemaSnapshot {
    /// Creates an empty schema snapshot.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tables: BTreeMap::new(),
        }
    }

    /// Adds a table snapshot.
    pub fn add_table(&mut self, table: TableSnapshot) {
        self.tables.insert(table.name.clone(), table);
    }

    /// Adds a table snapshot built from a `#[derive(Table)]` struct.
    pub fn add_from_table_schema<T: TableSchema>(&mut self, dialect: &impl RustTypeMapping) {
        let snapshot = TableSnapshot::from_table_schema::<T>(dialect);
        self.add_table(snapshot);
    }
}

impl Default for SchemaSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migrations::{DuckDbDialect, PostgresDialect, SqliteDialect};
    use crate::schema::{ColumnSchema, Table};

    // Minimal test table
    struct TestTable;
    struct TestRow;

    impl Table for TestTable {
        type Row = TestRow;
        const NAME: &'static str = "test_items";
        const COLUMNS: &'static [&'static str] = &["id", "name", "score", "active"];
        const PRIMARY_KEY: Option<&'static str> = Some("id");
    }

    impl TableSchema for TestTable {
        const SCHEMA: &'static [ColumnSchema] = &[
            ColumnSchema {
                name: "id",
                rust_type: "i64",
                nullable: false,
                primary_key: true,
                unique: false,
                autoincrement: true,
                default_expr: None,
            },
            ColumnSchema {
                name: "name",
                rust_type: "String",
                nullable: false,
                primary_key: false,
                unique: true,
                autoincrement: false,
                default_expr: None,
            },
            ColumnSchema {
                name: "score",
                rust_type: "Option<f64>",
                nullable: true,
                primary_key: false,
                unique: false,
                autoincrement: false,
                default_expr: None,
            },
            ColumnSchema {
                name: "active",
                rust_type: "bool",
                nullable: false,
                primary_key: false,
                unique: false,
                autoincrement: false,
                default_expr: Some("TRUE"),
            },
        ];
    }

    #[test]
    fn from_table_schema_sqlite() {
        let dialect = SqliteDialect::new();
        let snap = TableSnapshot::from_table_schema::<TestTable>(&dialect);

        assert_eq!(snap.name, "test_items");
        assert_eq!(snap.columns.len(), 4);

        let id = snap.column("id").unwrap();
        assert_eq!(id.data_type, DataType::Bigint);
        assert!(id.primary_key);
        assert!(id.autoincrement);
        assert!(!id.nullable);

        let name_col = snap.column("name").unwrap();
        assert_eq!(name_col.data_type, DataType::Text);
        assert!(name_col.unique);

        // Option<f64> -> f64 -> Double (SQLite maps f64 to Double)
        let score = snap.column("score").unwrap();
        assert_eq!(score.data_type, DataType::Double);
        assert!(score.nullable);

        let active = snap.column("active").unwrap();
        assert_eq!(active.data_type, DataType::Integer);
        assert_eq!(
            active.default,
            Some(DefaultValue::Expression("TRUE".into()))
        );
    }

    #[test]
    fn from_table_schema_postgres() {
        let dialect = PostgresDialect::new();
        let snap = TableSnapshot::from_table_schema::<TestTable>(&dialect);

        let name_col = snap.column("name").unwrap();
        assert_eq!(name_col.data_type, DataType::Varchar(Some(255)));

        let active = snap.column("active").unwrap();
        assert_eq!(active.data_type, DataType::Boolean);
    }

    #[test]
    fn from_table_schema_duckdb() {
        let dialect = DuckDbDialect::new();
        let snap = TableSnapshot::from_table_schema::<TestTable>(&dialect);

        let name_col = snap.column("name").unwrap();
        assert_eq!(name_col.data_type, DataType::Varchar(None));

        let active = snap.column("active").unwrap();
        assert_eq!(active.data_type, DataType::Boolean);
    }

    #[test]
    fn column_lookup_by_name() {
        let dialect = SqliteDialect::new();
        let snap = TableSnapshot::from_table_schema::<TestTable>(&dialect);

        assert!(snap.column("id").is_some());
        assert!(snap.column("name").is_some());
        assert!(snap.column("nonexistent").is_none());
    }

    #[test]
    fn schema_snapshot_add_tables() {
        let dialect = SqliteDialect::new();
        let mut schema = SchemaSnapshot::new();
        schema.add_from_table_schema::<TestTable>(&dialect);

        assert_eq!(schema.tables.len(), 1);
        assert!(schema.tables.contains_key("test_items"));
    }
}
