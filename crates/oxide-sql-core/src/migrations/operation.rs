//! Migration operations.
//!
//! Defines all possible migration operations like CREATE TABLE, ADD COLUMN, etc.

use super::column_builder::{ColumnDefinition, DefaultValue};
use crate::schema::{RustTypeMapping, TableSchema};

/// All possible migration operations.
#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    /// Create a new table.
    CreateTable(CreateTableOp),
    /// Drop an existing table.
    DropTable(DropTableOp),
    /// Rename a table.
    RenameTable(RenameTableOp),
    /// Add a column to an existing table.
    AddColumn(AddColumnOp),
    /// Drop a column from a table.
    DropColumn(DropColumnOp),
    /// Alter a column definition.
    AlterColumn(AlterColumnOp),
    /// Rename a column.
    RenameColumn(RenameColumnOp),
    /// Create an index.
    CreateIndex(CreateIndexOp),
    /// Drop an index.
    DropIndex(DropIndexOp),
    /// Add a foreign key constraint.
    AddForeignKey(AddForeignKeyOp),
    /// Drop a foreign key constraint.
    DropForeignKey(DropForeignKeyOp),
    /// Run raw SQL.
    RunSql(RawSqlOp),
}

impl Operation {
    /// Creates a drop table operation.
    #[must_use]
    pub fn drop_table(name: impl Into<String>) -> Self {
        Self::DropTable(DropTableOp {
            name: name.into(),
            if_exists: false,
            cascade: false,
        })
    }

    /// Creates a drop table if exists operation.
    #[must_use]
    pub fn drop_table_if_exists(name: impl Into<String>) -> Self {
        Self::DropTable(DropTableOp {
            name: name.into(),
            if_exists: true,
            cascade: false,
        })
    }

    /// Creates a rename table operation.
    #[must_use]
    pub fn rename_table(old_name: impl Into<String>, new_name: impl Into<String>) -> Self {
        Self::RenameTable(RenameTableOp {
            old_name: old_name.into(),
            new_name: new_name.into(),
        })
    }

    /// Creates an add column operation.
    #[must_use]
    pub fn add_column(table: impl Into<String>, column: ColumnDefinition) -> Self {
        Self::AddColumn(AddColumnOp {
            table: table.into(),
            column,
        })
    }

    /// Creates a drop column operation.
    #[must_use]
    pub fn drop_column(table: impl Into<String>, column: impl Into<String>) -> Self {
        Self::DropColumn(DropColumnOp {
            table: table.into(),
            column: column.into(),
        })
    }

    /// Creates a rename column operation.
    #[must_use]
    pub fn rename_column(
        table: impl Into<String>,
        old_name: impl Into<String>,
        new_name: impl Into<String>,
    ) -> Self {
        Self::RenameColumn(RenameColumnOp {
            table: table.into(),
            old_name: old_name.into(),
            new_name: new_name.into(),
        })
    }

    /// Creates a raw SQL operation.
    #[must_use]
    pub fn run_sql(sql: impl Into<String>) -> Self {
        Self::RunSql(RawSqlOp {
            up_sql: sql.into(),
            down_sql: None,
        })
    }

    /// Creates a raw SQL operation with both up and down SQL.
    #[must_use]
    pub fn run_sql_reversible(up_sql: impl Into<String>, down_sql: impl Into<String>) -> Self {
        Self::RunSql(RawSqlOp {
            up_sql: up_sql.into(),
            down_sql: Some(down_sql.into()),
        })
    }

    /// Attempts to generate the reverse operation.
    ///
    /// Returns `None` if the operation is not reversible.
    #[must_use]
    pub fn reverse(&self) -> Option<Self> {
        match self {
            Self::CreateTable(op) => Some(Self::drop_table(&op.name)),
            Self::DropTable(_) => None, // Cannot reverse without knowing the schema
            Self::RenameTable(op) => {
                Some(Self::rename_table(op.new_name.clone(), op.old_name.clone()))
            }
            Self::AddColumn(op) => Some(Self::drop_column(&op.table, &op.column.name)),
            Self::DropColumn(_) => None, // Cannot reverse without knowing the column definition
            Self::AlterColumn(_) => None, // Cannot reverse without knowing the old definition
            Self::RenameColumn(op) => Some(Self::rename_column(
                &op.table,
                op.new_name.clone(),
                op.old_name.clone(),
            )),
            Self::CreateIndex(op) => Some(Self::DropIndex(DropIndexOp {
                name: op.name.clone(),
                table: Some(op.table.clone()),
                if_exists: false,
            })),
            Self::DropIndex(_) => None, // Cannot reverse without knowing the index definition
            Self::AddForeignKey(op) => op.name.as_ref().map(|name| {
                Self::DropForeignKey(DropForeignKeyOp {
                    table: op.table.clone(),
                    name: name.clone(),
                })
            }),
            Self::DropForeignKey(_) => None, // Cannot reverse without knowing the FK definition
            Self::RunSql(op) => op.down_sql.as_ref().map(|down| Self::run_sql(down.clone())),
        }
    }

    /// Returns whether this operation is reversible.
    #[must_use]
    pub fn is_reversible(&self) -> bool {
        self.reverse().is_some()
    }
}

/// Create table operation.
#[derive(Debug, Clone, PartialEq)]
pub struct CreateTableOp {
    /// Table name.
    pub name: String,
    /// Column definitions.
    pub columns: Vec<ColumnDefinition>,
    /// Table-level constraints.
    pub constraints: Vec<TableConstraint>,
    /// Whether to use IF NOT EXISTS.
    pub if_not_exists: bool,
}

impl CreateTableOp {
    /// Builds a `CreateTableOp` from a `#[derive(Table)]` struct
    /// using the given dialect for Rust-to-SQL type mapping.
    pub fn from_table<T: TableSchema>(dialect: &impl RustTypeMapping) -> Self {
        let columns = T::SCHEMA
            .iter()
            .map(|col| {
                let inner = strip_option(col.rust_type);
                let data_type = dialect.map_type(inner);
                let mut def = ColumnDefinition::new(col.name, data_type);
                def.nullable = col.nullable;
                def.primary_key = col.primary_key;
                def.unique = col.unique;
                def.autoincrement = col.autoincrement;
                if let Some(expr) = col.default_expr {
                    def.default = Some(DefaultValue::Expression(expr.to_string()));
                }
                def
            })
            .collect();
        Self {
            name: T::NAME.to_string(),
            columns,
            constraints: vec![],
            if_not_exists: false,
        }
    }

    /// Same as `from_table` but with `IF NOT EXISTS`.
    pub fn from_table_if_not_exists<T: TableSchema>(dialect: &impl RustTypeMapping) -> Self {
        let mut op = Self::from_table::<T>(dialect);
        op.if_not_exists = true;
        op
    }
}

/// Strips `Option<T>` wrapper from a Rust type string, returning
/// the inner type. Nullability is tracked separately via
/// `ColumnSchema::nullable`.
pub(super) fn strip_option(rust_type: &str) -> &str {
    rust_type
        .strip_prefix("Option<")
        .and_then(|s| s.strip_suffix('>'))
        .unwrap_or(rust_type)
}

impl From<CreateTableOp> for Operation {
    fn from(op: CreateTableOp) -> Self {
        Self::CreateTable(op)
    }
}

/// Table-level constraint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableConstraint {
    /// Primary key constraint on multiple columns.
    PrimaryKey {
        /// Optional constraint name.
        name: Option<String>,
        /// Column names.
        columns: Vec<String>,
    },
    /// Unique constraint on multiple columns.
    Unique {
        /// Optional constraint name.
        name: Option<String>,
        /// Column names.
        columns: Vec<String>,
    },
    /// Foreign key constraint.
    ForeignKey {
        /// Optional constraint name.
        name: Option<String>,
        /// Columns in this table.
        columns: Vec<String>,
        /// Referenced table.
        references_table: String,
        /// Referenced columns.
        references_columns: Vec<String>,
        /// ON DELETE action.
        on_delete: Option<super::column_builder::ForeignKeyAction>,
        /// ON UPDATE action.
        on_update: Option<super::column_builder::ForeignKeyAction>,
    },
    /// Check constraint.
    Check {
        /// Optional constraint name.
        name: Option<String>,
        /// Check expression.
        expression: String,
    },
}

/// Drop table operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropTableOp {
    /// Table name.
    pub name: String,
    /// Whether to use IF EXISTS.
    pub if_exists: bool,
    /// Whether to cascade.
    pub cascade: bool,
}

impl From<DropTableOp> for Operation {
    fn from(op: DropTableOp) -> Self {
        Self::DropTable(op)
    }
}

/// Rename table operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameTableOp {
    /// Current table name.
    pub old_name: String,
    /// New table name.
    pub new_name: String,
}

impl From<RenameTableOp> for Operation {
    fn from(op: RenameTableOp) -> Self {
        Self::RenameTable(op)
    }
}

/// Add column operation.
#[derive(Debug, Clone, PartialEq)]
pub struct AddColumnOp {
    /// Table name.
    pub table: String,
    /// Column definition.
    pub column: ColumnDefinition,
}

impl From<AddColumnOp> for Operation {
    fn from(op: AddColumnOp) -> Self {
        Self::AddColumn(op)
    }
}

/// Drop column operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropColumnOp {
    /// Table name.
    pub table: String,
    /// Column name.
    pub column: String,
}

impl From<DropColumnOp> for Operation {
    fn from(op: DropColumnOp) -> Self {
        Self::DropColumn(op)
    }
}

/// Column alteration type.
#[derive(Debug, Clone, PartialEq)]
pub enum AlterColumnChange {
    /// Change the data type.
    SetDataType(crate::ast::DataType),
    /// Set or remove NOT NULL constraint.
    SetNullable(bool),
    /// Set a new default value.
    SetDefault(super::column_builder::DefaultValue),
    /// Remove the default value.
    DropDefault,
}

/// Alter column operation.
#[derive(Debug, Clone, PartialEq)]
pub struct AlterColumnOp {
    /// Table name.
    pub table: String,
    /// Column name.
    pub column: String,
    /// The change to apply.
    pub change: AlterColumnChange,
}

impl From<AlterColumnOp> for Operation {
    fn from(op: AlterColumnOp) -> Self {
        Self::AlterColumn(op)
    }
}

/// Rename column operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameColumnOp {
    /// Table name.
    pub table: String,
    /// Current column name.
    pub old_name: String,
    /// New column name.
    pub new_name: String,
}

impl From<RenameColumnOp> for Operation {
    fn from(op: RenameColumnOp) -> Self {
        Self::RenameColumn(op)
    }
}

/// Index type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IndexType {
    /// B-tree index (default).
    #[default]
    BTree,
    /// Hash index.
    Hash,
    /// GiST index (PostgreSQL).
    Gist,
    /// GIN index (PostgreSQL).
    Gin,
}

/// Create index operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateIndexOp {
    /// Index name.
    pub name: String,
    /// Table name.
    pub table: String,
    /// Columns to index.
    pub columns: Vec<String>,
    /// Whether this is a unique index.
    pub unique: bool,
    /// Index type.
    pub index_type: IndexType,
    /// Whether to use IF NOT EXISTS.
    pub if_not_exists: bool,
    /// Partial index condition (WHERE clause).
    pub condition: Option<String>,
}

impl From<CreateIndexOp> for Operation {
    fn from(op: CreateIndexOp) -> Self {
        Self::CreateIndex(op)
    }
}

/// Drop index operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropIndexOp {
    /// Index name.
    pub name: String,
    /// Table name (required for some dialects).
    pub table: Option<String>,
    /// Whether to use IF EXISTS.
    pub if_exists: bool,
}

impl From<DropIndexOp> for Operation {
    fn from(op: DropIndexOp) -> Self {
        Self::DropIndex(op)
    }
}

/// Add foreign key operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddForeignKeyOp {
    /// Table name.
    pub table: String,
    /// Optional constraint name.
    pub name: Option<String>,
    /// Columns in this table.
    pub columns: Vec<String>,
    /// Referenced table.
    pub references_table: String,
    /// Referenced columns.
    pub references_columns: Vec<String>,
    /// ON DELETE action.
    pub on_delete: Option<super::column_builder::ForeignKeyAction>,
    /// ON UPDATE action.
    pub on_update: Option<super::column_builder::ForeignKeyAction>,
}

impl From<AddForeignKeyOp> for Operation {
    fn from(op: AddForeignKeyOp) -> Self {
        Self::AddForeignKey(op)
    }
}

/// Drop foreign key operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropForeignKeyOp {
    /// Table name.
    pub table: String,
    /// Constraint name.
    pub name: String,
}

impl From<DropForeignKeyOp> for Operation {
    fn from(op: DropForeignKeyOp) -> Self {
        Self::DropForeignKey(op)
    }
}

/// Raw SQL operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawSqlOp {
    /// SQL to run for the up migration.
    pub up_sql: String,
    /// SQL to run for the down migration (if reversible).
    pub down_sql: Option<String>,
}

impl From<RawSqlOp> for Operation {
    fn from(op: RawSqlOp) -> Self {
        Self::RunSql(op)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migrations::column_builder::{ForeignKeyAction, bigint, varchar};

    #[test]
    fn test_drop_table_operation() {
        let op = Operation::drop_table("users");
        match op {
            Operation::DropTable(drop) => {
                assert_eq!(drop.name, "users");
                assert!(!drop.if_exists);
                assert!(!drop.cascade);
            }
            _ => panic!("Expected DropTable operation"),
        }
    }

    #[test]
    fn test_rename_table_operation() {
        let op = Operation::rename_table("old_name", "new_name");
        match op {
            Operation::RenameTable(rename) => {
                assert_eq!(rename.old_name, "old_name");
                assert_eq!(rename.new_name, "new_name");
            }
            _ => panic!("Expected RenameTable operation"),
        }
    }

    #[test]
    fn test_add_column_operation() {
        let col = varchar("email", 255).not_null().build();
        let op = Operation::add_column("users", col);
        match op {
            Operation::AddColumn(add) => {
                assert_eq!(add.table, "users");
                assert_eq!(add.column.name, "email");
            }
            _ => panic!("Expected AddColumn operation"),
        }
    }

    #[test]
    fn test_reverse_operations() {
        // Create table can be reversed to drop table
        let create = CreateTableOp {
            name: "users".to_string(),
            columns: vec![bigint("id").primary_key().build()],
            constraints: vec![],
            if_not_exists: false,
        };
        let op = Operation::CreateTable(create);
        let reversed = op.reverse().expect("Should be reversible");
        match reversed {
            Operation::DropTable(drop) => assert_eq!(drop.name, "users"),
            _ => panic!("Expected DropTable"),
        }

        // Rename table is reversible
        let rename = Operation::rename_table("old", "new");
        let reversed = rename.reverse().expect("Should be reversible");
        match reversed {
            Operation::RenameTable(r) => {
                assert_eq!(r.old_name, "new");
                assert_eq!(r.new_name, "old");
            }
            _ => panic!("Expected RenameTable"),
        }

        // Add column can be reversed to drop column
        let add = Operation::add_column("users", varchar("email", 255).build());
        let reversed = add.reverse().expect("Should be reversible");
        match reversed {
            Operation::DropColumn(drop) => {
                assert_eq!(drop.table, "users");
                assert_eq!(drop.column, "email");
            }
            _ => panic!("Expected DropColumn"),
        }

        // Drop table is NOT reversible (no schema info)
        let drop = Operation::drop_table("users");
        assert!(drop.reverse().is_none());
    }

    #[test]
    fn test_raw_sql_reversibility() {
        // Non-reversible raw SQL
        let op = Operation::run_sql("INSERT INTO config VALUES ('key', 'value')");
        assert!(!op.is_reversible());

        // Reversible raw SQL
        let op = Operation::run_sql_reversible(
            "INSERT INTO config VALUES ('key', 'value')",
            "DELETE FROM config WHERE key = 'key'",
        );
        assert!(op.is_reversible());
    }

    #[test]
    fn test_table_constraint() {
        let pk = TableConstraint::PrimaryKey {
            name: Some("pk_users".to_string()),
            columns: vec!["id".to_string()],
        };
        match pk {
            TableConstraint::PrimaryKey { name, columns } => {
                assert_eq!(name, Some("pk_users".to_string()));
                assert_eq!(columns, vec!["id"]);
            }
            _ => panic!("Expected PrimaryKey"),
        }

        let fk = TableConstraint::ForeignKey {
            name: Some("fk_user_company".to_string()),
            columns: vec!["company_id".to_string()],
            references_table: "companies".to_string(),
            references_columns: vec!["id".to_string()],
            on_delete: Some(ForeignKeyAction::Cascade),
            on_update: None,
        };
        match fk {
            TableConstraint::ForeignKey {
                references_table,
                on_delete,
                ..
            } => {
                assert_eq!(references_table, "companies");
                assert_eq!(on_delete, Some(ForeignKeyAction::Cascade));
            }
            _ => panic!("Expected ForeignKey"),
        }
    }
}
