//! Migration operations.
//!
//! This module defines all the schema changes that can be expressed in a migration,
//! along with their forward and backward SQL generation.

use serde::{Deserialize, Serialize};

use crate::schema::{
    ColumnSchema, DefaultValue, ForeignKeyAction, ForeignKeySchema, IndexSchema, SqlType,
};

/// Changes to apply to an existing column.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ColumnChanges {
    /// New data type (if changing).
    pub sql_type: Option<SqlType>,
    /// New nullability (if changing).
    pub nullable: Option<bool>,
    /// New default value (if changing).
    pub default: Option<DefaultValue>,
    /// New unique constraint (if changing).
    pub unique: Option<bool>,
}

impl ColumnChanges {
    /// Creates empty column changes.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a new type.
    #[must_use]
    pub fn set_type(mut self, sql_type: SqlType) -> Self {
        self.sql_type = Some(sql_type);
        self
    }

    /// Sets nullability.
    #[must_use]
    pub fn set_nullable(mut self, nullable: bool) -> Self {
        self.nullable = Some(nullable);
        self
    }

    /// Sets default value.
    #[must_use]
    pub fn set_default(mut self, default: DefaultValue) -> Self {
        self.default = Some(default);
        self
    }

    /// Sets unique constraint.
    #[must_use]
    pub fn set_unique(mut self, unique: bool) -> Self {
        self.unique = Some(unique);
        self
    }

    /// Returns true if no changes are specified.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sql_type.is_none()
            && self.nullable.is_none()
            && self.default.is_none()
            && self.unique.is_none()
    }
}

/// A single migration operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MigrationOperation {
    /// Create a new table.
    CreateTable {
        /// Table name.
        name: String,
        /// Column definitions.
        columns: Vec<ColumnSchema>,
        /// Primary key column(s).
        primary_key: Vec<String>,
        /// Whether to use IF NOT EXISTS.
        if_not_exists: bool,
    },

    /// Drop a table.
    DropTable {
        /// Table name.
        name: String,
        /// Whether to use IF EXISTS.
        if_exists: bool,
    },

    /// Rename a table.
    RenameTable {
        /// Old table name.
        old_name: String,
        /// New table name.
        new_name: String,
    },

    /// Add a column to a table.
    AddColumn {
        /// Table name.
        table: String,
        /// Column definition.
        column: ColumnSchema,
    },

    /// Drop a column from a table.
    DropColumn {
        /// Table name.
        table: String,
        /// Column name.
        column_name: String,
    },

    /// Rename a column.
    RenameColumn {
        /// Table name.
        table: String,
        /// Old column name.
        old_name: String,
        /// New column name.
        new_name: String,
    },

    /// Alter a column's properties.
    AlterColumn {
        /// Table name.
        table: String,
        /// Column name.
        column_name: String,
        /// Changes to apply.
        changes: ColumnChanges,
        /// Original column definition (for reversal).
        original: Option<ColumnSchema>,
    },

    /// Create an index.
    CreateIndex {
        /// Index name.
        name: String,
        /// Table name.
        table: String,
        /// Columns to index.
        columns: Vec<String>,
        /// Whether this is a unique index.
        unique: bool,
        /// Partial index condition.
        condition: Option<String>,
        /// Whether to use IF NOT EXISTS.
        if_not_exists: bool,
    },

    /// Drop an index.
    DropIndex {
        /// Index name.
        name: String,
        /// Table name (needed for some databases).
        table: Option<String>,
        /// Whether to use IF EXISTS.
        if_exists: bool,
    },

    /// Add a foreign key constraint.
    AddForeignKey {
        /// Table name.
        table: String,
        /// Foreign key definition.
        foreign_key: ForeignKeySchema,
    },

    /// Drop a foreign key constraint.
    DropForeignKey {
        /// Table name.
        table: String,
        /// Constraint name.
        constraint_name: String,
    },

    /// Add a unique constraint.
    AddUniqueConstraint {
        /// Table name.
        table: String,
        /// Constraint name.
        name: String,
        /// Columns that form the unique constraint.
        columns: Vec<String>,
    },

    /// Drop a unique constraint.
    DropUniqueConstraint {
        /// Table name.
        table: String,
        /// Constraint name.
        name: String,
    },

    /// Run raw SQL (for custom migrations).
    RunSql {
        /// Forward SQL statement(s).
        forward: String,
        /// Backward SQL statement(s) for rollback.
        backward: Option<String>,
    },
}

impl MigrationOperation {
    // Convenience constructors

    /// Creates a CreateTable operation.
    #[must_use]
    pub fn create_table(
        name: impl Into<String>,
        columns: Vec<ColumnSchema>,
        primary_key: Vec<String>,
    ) -> Self {
        Self::CreateTable {
            name: name.into(),
            columns,
            primary_key,
            if_not_exists: false,
        }
    }

    /// Creates a DropTable operation.
    #[must_use]
    pub fn drop_table(name: impl Into<String>) -> Self {
        Self::DropTable {
            name: name.into(),
            if_exists: false,
        }
    }

    /// Creates a RenameTable operation.
    #[must_use]
    pub fn rename_table(old_name: impl Into<String>, new_name: impl Into<String>) -> Self {
        Self::RenameTable {
            old_name: old_name.into(),
            new_name: new_name.into(),
        }
    }

    /// Creates an AddColumn operation.
    #[must_use]
    pub fn add_column(table: impl Into<String>, column: ColumnSchema) -> Self {
        Self::AddColumn {
            table: table.into(),
            column,
        }
    }

    /// Creates a DropColumn operation.
    #[must_use]
    pub fn drop_column(table: impl Into<String>, column_name: impl Into<String>) -> Self {
        Self::DropColumn {
            table: table.into(),
            column_name: column_name.into(),
        }
    }

    /// Creates a RenameColumn operation.
    #[must_use]
    pub fn rename_column(
        table: impl Into<String>,
        old_name: impl Into<String>,
        new_name: impl Into<String>,
    ) -> Self {
        Self::RenameColumn {
            table: table.into(),
            old_name: old_name.into(),
            new_name: new_name.into(),
        }
    }

    /// Creates an AlterColumn operation.
    #[must_use]
    pub fn alter_column(
        table: impl Into<String>,
        column_name: impl Into<String>,
        changes: ColumnChanges,
    ) -> Self {
        Self::AlterColumn {
            table: table.into(),
            column_name: column_name.into(),
            changes,
            original: None,
        }
    }

    /// Creates a CreateIndex operation.
    #[must_use]
    pub fn create_index(
        name: impl Into<String>,
        table: impl Into<String>,
        columns: Vec<String>,
        unique: bool,
    ) -> Self {
        Self::CreateIndex {
            name: name.into(),
            table: table.into(),
            columns,
            unique,
            condition: None,
            if_not_exists: false,
        }
    }

    /// Creates a DropIndex operation.
    #[must_use]
    pub fn drop_index(name: impl Into<String>) -> Self {
        Self::DropIndex {
            name: name.into(),
            table: None,
            if_exists: false,
        }
    }

    /// Creates an AddForeignKey operation.
    #[must_use]
    pub fn add_foreign_key(table: impl Into<String>, foreign_key: ForeignKeySchema) -> Self {
        Self::AddForeignKey {
            table: table.into(),
            foreign_key,
        }
    }

    /// Creates a DropForeignKey operation.
    #[must_use]
    pub fn drop_foreign_key(table: impl Into<String>, constraint_name: impl Into<String>) -> Self {
        Self::DropForeignKey {
            table: table.into(),
            constraint_name: constraint_name.into(),
        }
    }

    /// Creates an AddUniqueConstraint operation.
    #[must_use]
    pub fn add_unique_constraint(
        table: impl Into<String>,
        name: impl Into<String>,
        columns: Vec<String>,
    ) -> Self {
        Self::AddUniqueConstraint {
            table: table.into(),
            name: name.into(),
            columns,
        }
    }

    /// Creates a DropUniqueConstraint operation.
    #[must_use]
    pub fn drop_unique_constraint(table: impl Into<String>, name: impl Into<String>) -> Self {
        Self::DropUniqueConstraint {
            table: table.into(),
            name: name.into(),
        }
    }

    /// Creates a RunSql operation.
    #[must_use]
    pub fn run_sql(forward: impl Into<String>, backward: Option<String>) -> Self {
        Self::RunSql {
            forward: forward.into(),
            backward,
        }
    }

    /// Returns the reverse operation for rollback.
    ///
    /// Returns `None` if the operation is not reversible.
    #[must_use]
    pub fn reverse(&self) -> Option<Self> {
        match self {
            Self::CreateTable { name, .. } => Some(Self::drop_table(name.clone())),

            Self::DropTable { .. } => {
                // Cannot reverse without knowing the original table definition
                None
            }

            Self::RenameTable { old_name, new_name } => {
                Some(Self::rename_table(new_name.clone(), old_name.clone()))
            }

            Self::AddColumn { table, column } => {
                Some(Self::drop_column(table.clone(), column.name.clone()))
            }

            Self::DropColumn { .. } => {
                // Cannot reverse without knowing the original column definition
                None
            }

            Self::RenameColumn {
                table,
                old_name,
                new_name,
            } => Some(Self::rename_column(
                table.clone(),
                new_name.clone(),
                old_name.clone(),
            )),

            Self::AlterColumn {
                table,
                column_name,
                original,
                ..
            } => {
                // Can only reverse if we have the original column definition
                original.as_ref().map(|orig| Self::AlterColumn {
                    table: table.clone(),
                    column_name: column_name.clone(),
                    changes: ColumnChanges {
                        sql_type: Some(orig.sql_type.clone()),
                        nullable: Some(orig.nullable),
                        default: Some(orig.default.clone()),
                        unique: Some(orig.unique),
                    },
                    original: None,
                })
            }

            Self::CreateIndex { name, .. } => Some(Self::drop_index(name.clone())),

            Self::DropIndex { .. } => {
                // Cannot reverse without knowing the original index definition
                None
            }

            Self::AddForeignKey { table, foreign_key } => Some(Self::drop_foreign_key(
                table.clone(),
                foreign_key.name.clone(),
            )),

            Self::DropForeignKey { .. } => {
                // Cannot reverse without knowing the original foreign key
                None
            }

            Self::AddUniqueConstraint { table, name, .. } => {
                Some(Self::drop_unique_constraint(table.clone(), name.clone()))
            }

            Self::DropUniqueConstraint { .. } => {
                // Cannot reverse without knowing the original constraint
                None
            }

            Self::RunSql { backward, forward } => backward.as_ref().map(|bwd| Self::RunSql {
                forward: bwd.clone(),
                backward: Some(forward.clone()),
            }),
        }
    }

    /// Returns true if this operation can be reversed.
    #[must_use]
    pub fn is_reversible(&self) -> bool {
        match self {
            Self::CreateTable { .. } => true,
            Self::DropTable { .. } => false,
            Self::RenameTable { .. } => true,
            Self::AddColumn { .. } => true,
            Self::DropColumn { .. } => false,
            Self::RenameColumn { .. } => true,
            Self::AlterColumn { original, .. } => original.is_some(),
            Self::CreateIndex { .. } => true,
            Self::DropIndex { .. } => false,
            Self::AddForeignKey { .. } => true,
            Self::DropForeignKey { .. } => false,
            Self::AddUniqueConstraint { .. } => true,
            Self::DropUniqueConstraint { .. } => false,
            Self::RunSql { backward, .. } => backward.is_some(),
        }
    }

    /// Returns a human-readable description of this operation.
    #[must_use]
    pub fn description(&self) -> String {
        match self {
            Self::CreateTable { name, .. } => format!("Create table '{}'", name),
            Self::DropTable { name, .. } => format!("Drop table '{}'", name),
            Self::RenameTable { old_name, new_name } => {
                format!("Rename table '{}' to '{}'", old_name, new_name)
            }
            Self::AddColumn { table, column } => {
                format!("Add column '{}' to table '{}'", column.name, table)
            }
            Self::DropColumn { table, column_name } => {
                format!("Drop column '{}' from table '{}'", column_name, table)
            }
            Self::RenameColumn {
                table,
                old_name,
                new_name,
            } => format!(
                "Rename column '{}' to '{}' in table '{}'",
                old_name, new_name, table
            ),
            Self::AlterColumn {
                table, column_name, ..
            } => format!("Alter column '{}' in table '{}'", column_name, table),
            Self::CreateIndex { name, table, .. } => {
                format!("Create index '{}' on table '{}'", name, table)
            }
            Self::DropIndex { name, .. } => format!("Drop index '{}'", name),
            Self::AddForeignKey { table, foreign_key } => format!(
                "Add foreign key '{}' to table '{}'",
                foreign_key.name, table
            ),
            Self::DropForeignKey {
                table,
                constraint_name,
            } => format!(
                "Drop foreign key '{}' from table '{}'",
                constraint_name, table
            ),
            Self::AddUniqueConstraint { table, name, .. } => {
                format!("Add unique constraint '{}' to table '{}'", name, table)
            }
            Self::DropUniqueConstraint { table, name } => {
                format!("Drop unique constraint '{}' from table '{}'", name, table)
            }
            Self::RunSql { .. } => "Run custom SQL".to_string(),
        }
    }
}

/// Helper to create a foreign key schema.
#[must_use]
pub fn foreign_key(
    name: impl Into<String>,
    columns: Vec<String>,
    references_table: impl Into<String>,
    references_columns: Vec<String>,
) -> ForeignKeySchema {
    ForeignKeySchema {
        name: name.into(),
        columns,
        references_table: references_table.into(),
        references_columns,
        on_delete: ForeignKeyAction::NoAction,
        on_update: ForeignKeyAction::NoAction,
    }
}

/// Builder for foreign key schema.
pub struct ForeignKeyBuilder {
    schema: ForeignKeySchema,
}

impl ForeignKeyBuilder {
    /// Creates a new foreign key builder.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            schema: ForeignKeySchema {
                name: name.into(),
                columns: Vec::new(),
                references_table: String::new(),
                references_columns: Vec::new(),
                on_delete: ForeignKeyAction::NoAction,
                on_update: ForeignKeyAction::NoAction,
            },
        }
    }

    /// Sets the local columns.
    #[must_use]
    pub fn columns(mut self, columns: Vec<String>) -> Self {
        self.schema.columns = columns;
        self
    }

    /// Sets the referenced table and columns.
    #[must_use]
    pub fn references(mut self, table: impl Into<String>, columns: Vec<String>) -> Self {
        self.schema.references_table = table.into();
        self.schema.references_columns = columns;
        self
    }

    /// Sets the ON DELETE action.
    #[must_use]
    pub fn on_delete(mut self, action: ForeignKeyAction) -> Self {
        self.schema.on_delete = action;
        self
    }

    /// Sets the ON UPDATE action.
    #[must_use]
    pub fn on_update(mut self, action: ForeignKeyAction) -> Self {
        self.schema.on_update = action;
        self
    }

    /// Builds the foreign key schema.
    #[must_use]
    pub fn build(self) -> ForeignKeySchema {
        self.schema
    }
}

/// Builder for index schema.
pub struct IndexBuilder {
    schema: IndexSchema,
}

impl IndexBuilder {
    /// Creates a new index builder.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            schema: IndexSchema {
                name: name.into(),
                columns: Vec::new(),
                unique: false,
                condition: None,
            },
        }
    }

    /// Sets the columns to index.
    #[must_use]
    pub fn columns(mut self, columns: Vec<String>) -> Self {
        self.schema.columns = columns;
        self
    }

    /// Makes this a unique index.
    #[must_use]
    pub fn unique(mut self) -> Self {
        self.schema.unique = true;
        self
    }

    /// Sets a partial index condition.
    #[must_use]
    pub fn condition(mut self, condition: impl Into<String>) -> Self {
        self.schema.condition = Some(condition.into());
        self
    }

    /// Builds the index schema.
    #[must_use]
    pub fn build(self) -> IndexSchema {
        self.schema
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_table_reverse() {
        let op = MigrationOperation::create_table(
            "users",
            vec![ColumnSchema::new("id", SqlType::BigInt).primary_key()],
            vec!["id".to_string()],
        );

        let reverse = op.reverse().unwrap();
        match reverse {
            MigrationOperation::DropTable { name, .. } => {
                assert_eq!(name, "users");
            }
            _ => panic!("Expected DropTable"),
        }
    }

    #[test]
    fn test_rename_table_reverse() {
        let op = MigrationOperation::rename_table("old_name", "new_name");
        let reverse = op.reverse().unwrap();

        match reverse {
            MigrationOperation::RenameTable { old_name, new_name } => {
                assert_eq!(old_name, "new_name");
                assert_eq!(new_name, "old_name");
            }
            _ => panic!("Expected RenameTable"),
        }
    }

    #[test]
    fn test_add_column_reverse() {
        let op = MigrationOperation::add_column(
            "users",
            ColumnSchema::new("email", SqlType::Varchar(255)),
        );

        let reverse = op.reverse().unwrap();
        match reverse {
            MigrationOperation::DropColumn { table, column_name } => {
                assert_eq!(table, "users");
                assert_eq!(column_name, "email");
            }
            _ => panic!("Expected DropColumn"),
        }
    }

    #[test]
    fn test_drop_table_not_reversible() {
        let op = MigrationOperation::drop_table("users");
        assert!(op.reverse().is_none());
        assert!(!op.is_reversible());
    }

    #[test]
    fn test_run_sql_reversible() {
        let op = MigrationOperation::run_sql(
            "INSERT INTO config VALUES ('key', 'value')",
            Some("DELETE FROM config WHERE key = 'key'".to_string()),
        );

        assert!(op.is_reversible());
        let reverse = op.reverse().unwrap();
        match reverse {
            MigrationOperation::RunSql { forward, backward } => {
                assert_eq!(forward, "DELETE FROM config WHERE key = 'key'");
                assert!(backward.is_some());
            }
            _ => panic!("Expected RunSql"),
        }
    }

    #[test]
    fn test_column_changes() {
        let changes = ColumnChanges::new()
            .set_type(SqlType::Text)
            .set_nullable(false);

        assert!(!changes.is_empty());
        assert_eq!(changes.sql_type, Some(SqlType::Text));
        assert_eq!(changes.nullable, Some(false));
    }

    #[test]
    fn test_foreign_key_builder() {
        let fk = ForeignKeyBuilder::new("fk_user_org")
            .columns(vec!["organization_id".to_string()])
            .references("organizations", vec!["id".to_string()])
            .on_delete(ForeignKeyAction::Cascade)
            .build();

        assert_eq!(fk.name, "fk_user_org");
        assert_eq!(fk.columns, vec!["organization_id"]);
        assert_eq!(fk.references_table, "organizations");
        assert_eq!(fk.on_delete, ForeignKeyAction::Cascade);
    }
}
