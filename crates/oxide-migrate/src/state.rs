//! State reconstruction from migrations.
//!
//! This module can reconstruct the expected database schema by replaying
//! migration operations. This is used by the autodetector to compare
//! the current model definitions against what migrations have created.

use crate::error::{MigrateError, Result};
use crate::executor::ExecutableMigration;
use crate::operations::MigrationOperation;
use crate::schema::{DatabaseSchema, IndexSchema, TableSchema, UniqueConstraint};

/// Reconstructs database schema from a list of migrations.
#[derive(Debug, Default)]
pub struct SchemaState {
    schema: DatabaseSchema,
}

impl SchemaState {
    /// Creates a new empty schema state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the current schema.
    #[must_use]
    pub fn schema(&self) -> &DatabaseSchema {
        &self.schema
    }

    /// Consumes and returns the schema.
    #[must_use]
    pub fn into_schema(self) -> DatabaseSchema {
        self.schema
    }

    /// Applies a migration's operations to the schema state.
    pub fn apply_migration(&mut self, migration: &ExecutableMigration) -> Result<()> {
        for operation in &migration.operations {
            self.apply_operation(operation)?;
        }
        Ok(())
    }

    /// Applies a single operation to the schema state.
    pub fn apply_operation(&mut self, operation: &MigrationOperation) -> Result<()> {
        match operation {
            MigrationOperation::CreateTable {
                name,
                columns,
                primary_key,
                ..
            } => {
                if self.schema.get_table(name).is_some() {
                    return Err(MigrateError::InvalidState(format!(
                        "Table '{}' already exists",
                        name
                    )));
                }

                let table = TableSchema {
                    name: name.clone(),
                    columns: columns.clone(),
                    primary_key: primary_key.clone(),
                    indexes: Vec::new(),
                    foreign_keys: Vec::new(),
                    unique_constraints: Vec::new(),
                };
                self.schema.tables.push(table);
            }

            MigrationOperation::DropTable { name, .. } => {
                let idx = self
                    .schema
                    .tables
                    .iter()
                    .position(|t| t.name == *name)
                    .ok_or_else(|| {
                        MigrateError::InvalidState(format!("Table '{}' does not exist", name))
                    })?;
                self.schema.tables.remove(idx);
            }

            MigrationOperation::RenameTable { old_name, new_name } => {
                let table = self.schema.get_table_mut(old_name).ok_or_else(|| {
                    MigrateError::InvalidState(format!("Table '{}' does not exist", old_name))
                })?;
                table.name = new_name.clone();
            }

            MigrationOperation::AddColumn { table, column } => {
                let t = self.schema.get_table_mut(table).ok_or_else(|| {
                    MigrateError::InvalidState(format!("Table '{}' does not exist", table))
                })?;

                if t.get_column(&column.name).is_some() {
                    return Err(MigrateError::InvalidState(format!(
                        "Column '{}' already exists in table '{}'",
                        column.name, table
                    )));
                }

                t.columns.push(column.clone());
            }

            MigrationOperation::DropColumn { table, column_name } => {
                let t = self.schema.get_table_mut(table).ok_or_else(|| {
                    MigrateError::InvalidState(format!("Table '{}' does not exist", table))
                })?;

                let idx = t
                    .columns
                    .iter()
                    .position(|c| c.name == *column_name)
                    .ok_or_else(|| {
                        MigrateError::InvalidState(format!(
                            "Column '{}' does not exist in table '{}'",
                            column_name, table
                        ))
                    })?;
                t.columns.remove(idx);
            }

            MigrationOperation::RenameColumn {
                table,
                old_name,
                new_name,
            } => {
                let t = self.schema.get_table_mut(table).ok_or_else(|| {
                    MigrateError::InvalidState(format!("Table '{}' does not exist", table))
                })?;

                let col = t.get_column_mut(old_name).ok_or_else(|| {
                    MigrateError::InvalidState(format!(
                        "Column '{}' does not exist in table '{}'",
                        old_name, table
                    ))
                })?;
                col.name = new_name.clone();
            }

            MigrationOperation::AlterColumn {
                table,
                column_name,
                changes,
                ..
            } => {
                let t = self.schema.get_table_mut(table).ok_or_else(|| {
                    MigrateError::InvalidState(format!("Table '{}' does not exist", table))
                })?;

                let col = t.get_column_mut(column_name).ok_or_else(|| {
                    MigrateError::InvalidState(format!(
                        "Column '{}' does not exist in table '{}'",
                        column_name, table
                    ))
                })?;

                if let Some(ref sql_type) = changes.sql_type {
                    col.sql_type = sql_type.clone();
                }
                if let Some(nullable) = changes.nullable {
                    col.nullable = nullable;
                }
                if let Some(ref default) = changes.default {
                    col.default = default.clone();
                }
                if let Some(unique) = changes.unique {
                    col.unique = unique;
                }
            }

            MigrationOperation::CreateIndex {
                name,
                table,
                columns,
                unique,
                condition,
                ..
            } => {
                let t = self.schema.get_table_mut(table).ok_or_else(|| {
                    MigrateError::InvalidState(format!("Table '{}' does not exist", table))
                })?;

                t.indexes.push(IndexSchema {
                    name: name.clone(),
                    columns: columns.clone(),
                    unique: *unique,
                    condition: condition.clone(),
                });
            }

            MigrationOperation::DropIndex { name, table, .. } => {
                // Try to find the index in any table if table is not specified
                let mut found = false;
                for t in &mut self.schema.tables {
                    if let Some(table_name) = table
                        && t.name != *table_name
                    {
                        continue;
                    }
                    if let Some(idx) = t.indexes.iter().position(|i| i.name == *name) {
                        t.indexes.remove(idx);
                        found = true;
                        break;
                    }
                }
                if !found {
                    return Err(MigrateError::InvalidState(format!(
                        "Index '{}' does not exist",
                        name
                    )));
                }
            }

            MigrationOperation::AddForeignKey { table, foreign_key } => {
                let t = self.schema.get_table_mut(table).ok_or_else(|| {
                    MigrateError::InvalidState(format!("Table '{}' does not exist", table))
                })?;

                t.foreign_keys.push(foreign_key.clone());
            }

            MigrationOperation::DropForeignKey {
                table,
                constraint_name,
            } => {
                let t = self.schema.get_table_mut(table).ok_or_else(|| {
                    MigrateError::InvalidState(format!("Table '{}' does not exist", table))
                })?;

                let idx = t
                    .foreign_keys
                    .iter()
                    .position(|fk| fk.name == *constraint_name)
                    .ok_or_else(|| {
                        MigrateError::InvalidState(format!(
                            "Foreign key '{}' does not exist in table '{}'",
                            constraint_name, table
                        ))
                    })?;
                t.foreign_keys.remove(idx);
            }

            MigrationOperation::AddUniqueConstraint {
                table,
                name,
                columns,
            } => {
                let t = self.schema.get_table_mut(table).ok_or_else(|| {
                    MigrateError::InvalidState(format!("Table '{}' does not exist", table))
                })?;

                t.unique_constraints.push(UniqueConstraint {
                    name: name.clone(),
                    columns: columns.clone(),
                });
            }

            MigrationOperation::DropUniqueConstraint { table, name } => {
                let t = self.schema.get_table_mut(table).ok_or_else(|| {
                    MigrateError::InvalidState(format!("Table '{}' does not exist", table))
                })?;

                let idx = t
                    .unique_constraints
                    .iter()
                    .position(|uc| uc.name == *name)
                    .ok_or_else(|| {
                        MigrateError::InvalidState(format!(
                            "Unique constraint '{}' does not exist in table '{}'",
                            name, table
                        ))
                    })?;
                t.unique_constraints.remove(idx);
            }

            MigrationOperation::RunSql { .. } => {
                // Raw SQL doesn't affect the tracked schema state
            }
        }

        Ok(())
    }

    /// Applies multiple migrations in order.
    pub fn apply_migrations(&mut self, migrations: &[ExecutableMigration]) -> Result<()> {
        for migration in migrations {
            self.apply_migration(migration)?;
        }
        Ok(())
    }

    /// Reconstructs schema from a list of migrations.
    pub fn from_migrations(migrations: &[ExecutableMigration]) -> Result<Self> {
        let mut state = Self::new();
        state.apply_migrations(migrations)?;
        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{ColumnSchema, SqlType};

    fn create_users_migration() -> ExecutableMigration {
        ExecutableMigration::new("users", "0001_initial").operation(
            MigrationOperation::CreateTable {
                name: "users".to_string(),
                columns: vec![
                    ColumnSchema::new("id", SqlType::BigInt)
                        .primary_key()
                        .auto_increment(),
                    ColumnSchema::new("username", SqlType::Varchar(255)).not_null(),
                ],
                primary_key: vec!["id".to_string()],
                if_not_exists: false,
            },
        )
    }

    fn add_email_migration() -> ExecutableMigration {
        ExecutableMigration::new("users", "0002_add_email").operation(
            MigrationOperation::AddColumn {
                table: "users".to_string(),
                column: ColumnSchema::new("email", SqlType::Varchar(255)),
            },
        )
    }

    #[test]
    fn test_create_table() {
        let mut state = SchemaState::new();
        state.apply_migration(&create_users_migration()).unwrap();

        let schema = state.schema();
        assert_eq!(schema.tables.len(), 1);

        let users = schema.get_table("users").unwrap();
        assert_eq!(users.columns.len(), 2);
        assert_eq!(users.primary_key, vec!["id"]);
    }

    #[test]
    fn test_add_column() {
        let mut state = SchemaState::new();
        state.apply_migration(&create_users_migration()).unwrap();
        state.apply_migration(&add_email_migration()).unwrap();

        let schema = state.schema();
        let users = schema.get_table("users").unwrap();
        assert_eq!(users.columns.len(), 3);
        assert!(users.get_column("email").is_some());
    }

    #[test]
    fn test_drop_column() {
        let mut state = SchemaState::new();
        state.apply_migration(&create_users_migration()).unwrap();
        state.apply_migration(&add_email_migration()).unwrap();

        state
            .apply_operation(&MigrationOperation::DropColumn {
                table: "users".to_string(),
                column_name: "email".to_string(),
            })
            .unwrap();

        let users = state.schema().get_table("users").unwrap();
        assert_eq!(users.columns.len(), 2);
        assert!(users.get_column("email").is_none());
    }

    #[test]
    fn test_rename_table() {
        let mut state = SchemaState::new();
        state.apply_migration(&create_users_migration()).unwrap();

        state
            .apply_operation(&MigrationOperation::RenameTable {
                old_name: "users".to_string(),
                new_name: "accounts".to_string(),
            })
            .unwrap();

        assert!(state.schema().get_table("users").is_none());
        assert!(state.schema().get_table("accounts").is_some());
    }

    #[test]
    fn test_rename_column() {
        let mut state = SchemaState::new();
        state.apply_migration(&create_users_migration()).unwrap();

        state
            .apply_operation(&MigrationOperation::RenameColumn {
                table: "users".to_string(),
                old_name: "username".to_string(),
                new_name: "name".to_string(),
            })
            .unwrap();

        let users = state.schema().get_table("users").unwrap();
        assert!(users.get_column("username").is_none());
        assert!(users.get_column("name").is_some());
    }

    #[test]
    fn test_drop_table() {
        let mut state = SchemaState::new();
        state.apply_migration(&create_users_migration()).unwrap();

        state
            .apply_operation(&MigrationOperation::DropTable {
                name: "users".to_string(),
                if_exists: false,
            })
            .unwrap();

        assert!(state.schema().get_table("users").is_none());
    }

    #[test]
    fn test_create_index() {
        let mut state = SchemaState::new();
        state.apply_migration(&create_users_migration()).unwrap();

        state
            .apply_operation(&MigrationOperation::CreateIndex {
                name: "idx_username".to_string(),
                table: "users".to_string(),
                columns: vec!["username".to_string()],
                unique: true,
                condition: None,
                if_not_exists: false,
            })
            .unwrap();

        let users = state.schema().get_table("users").unwrap();
        assert_eq!(users.indexes.len(), 1);
        assert_eq!(users.indexes[0].name, "idx_username");
        assert!(users.indexes[0].unique);
    }

    #[test]
    fn test_from_migrations() {
        let migrations = vec![create_users_migration(), add_email_migration()];
        let state = SchemaState::from_migrations(&migrations).unwrap();

        let users = state.schema().get_table("users").unwrap();
        assert_eq!(users.columns.len(), 3);
    }

    #[test]
    fn test_duplicate_table_error() {
        let mut state = SchemaState::new();
        state.apply_migration(&create_users_migration()).unwrap();

        let result = state.apply_migration(&create_users_migration());
        assert!(matches!(result, Err(MigrateError::InvalidState(_))));
    }

    #[test]
    fn test_missing_table_error() {
        let mut state = SchemaState::new();

        let result = state.apply_operation(&MigrationOperation::AddColumn {
            table: "nonexistent".to_string(),
            column: ColumnSchema::new("col", SqlType::Text),
        });
        assert!(matches!(result, Err(MigrateError::InvalidState(_))));
    }
}
