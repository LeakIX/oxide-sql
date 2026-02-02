//! Autodetector for generating migrations from schema changes.
//!
//! This module compares two database schemas and generates the migration
//! operations needed to transform one into the other.

use std::collections::{HashMap, HashSet};

use crate::operations::{ColumnChanges, MigrationOperation};
use crate::schema::{
    ColumnSchema, DatabaseSchema, ForeignKeySchema, IndexSchema, TableSchema, UniqueConstraint,
};

/// Options for the autodetector.
#[derive(Debug, Clone, Default)]
pub struct AutodetectorOptions {
    /// Whether to detect renamed tables (heuristic).
    pub detect_renames: bool,
    /// Similarity threshold for rename detection (0.0 to 1.0).
    pub rename_threshold: f64,
}

impl AutodetectorOptions {
    /// Creates default options.
    #[must_use]
    pub fn new() -> Self {
        Self {
            detect_renames: false,
            rename_threshold: 0.7,
        }
    }

    /// Enables rename detection.
    #[must_use]
    pub fn with_rename_detection(mut self) -> Self {
        self.detect_renames = true;
        self
    }
}

/// Detects schema changes and generates migration operations.
#[derive(Debug)]
pub struct Autodetector {
    options: AutodetectorOptions,
}

impl Default for Autodetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Autodetector {
    /// Creates a new autodetector with default options.
    #[must_use]
    pub fn new() -> Self {
        Self {
            options: AutodetectorOptions::default(),
        }
    }

    /// Creates a new autodetector with custom options.
    #[must_use]
    pub fn with_options(options: AutodetectorOptions) -> Self {
        Self { options }
    }

    /// Compares two schemas and returns the operations needed to transform
    /// `from` into `to`.
    #[must_use]
    pub fn diff(&self, from: &DatabaseSchema, to: &DatabaseSchema) -> Vec<MigrationOperation> {
        let mut operations = Vec::new();

        let from_tables: HashMap<&str, &TableSchema> =
            from.tables.iter().map(|t| (t.name.as_str(), t)).collect();
        let to_tables: HashMap<&str, &TableSchema> =
            to.tables.iter().map(|t| (t.name.as_str(), t)).collect();

        let from_names: HashSet<&str> = from_tables.keys().copied().collect();
        let to_names: HashSet<&str> = to_tables.keys().copied().collect();

        // Tables to create (in to but not in from)
        let new_tables: Vec<&str> = to_names.difference(&from_names).copied().collect();

        // Tables to drop (in from but not in to)
        let dropped_tables: Vec<&str> = from_names.difference(&to_names).copied().collect();

        // Tables that exist in both (may have changes)
        let common_tables: Vec<&str> = from_names.intersection(&to_names).copied().collect();

        // Handle table renames (heuristic)
        let mut renamed_tables: HashMap<&str, &str> = HashMap::new();
        if self.options.detect_renames && !new_tables.is_empty() && !dropped_tables.is_empty() {
            for &dropped in &dropped_tables {
                let dropped_table = from_tables[dropped];
                for &new in &new_tables {
                    if renamed_tables.values().any(|&v| v == new) {
                        continue; // Already matched
                    }
                    let new_table = to_tables[new];
                    if self.tables_similar(dropped_table, new_table) {
                        renamed_tables.insert(dropped, new);
                        break;
                    }
                }
            }
        }

        // Generate rename operations
        for (&old_name, &new_name) in &renamed_tables {
            operations.push(MigrationOperation::rename_table(old_name, new_name));
        }

        // Generate create table operations (excluding renamed tables)
        for &name in &new_tables {
            if renamed_tables.values().any(|&v| v == name) {
                continue;
            }
            let table = to_tables[name];
            operations.push(MigrationOperation::create_table(
                name.to_string(),
                table.columns.clone(),
                table.primary_key.clone(),
            ));

            // Add foreign keys
            for fk in &table.foreign_keys {
                operations.push(MigrationOperation::add_foreign_key(name, fk.clone()));
            }

            // Add indexes (non-primary, non-unique-constraint)
            for idx in &table.indexes {
                operations.push(MigrationOperation::CreateIndex {
                    name: idx.name.clone(),
                    table: name.to_string(),
                    columns: idx.columns.clone(),
                    unique: idx.unique,
                    condition: idx.condition.clone(),
                    if_not_exists: false,
                });
            }

            // Add unique constraints
            for uc in &table.unique_constraints {
                operations.push(MigrationOperation::add_unique_constraint(
                    name,
                    uc.name.clone(),
                    uc.columns.clone(),
                ));
            }
        }

        // Generate drop table operations (excluding renamed tables)
        for &name in &dropped_tables {
            if renamed_tables.contains_key(name) {
                continue;
            }
            operations.push(MigrationOperation::drop_table(name));
        }

        // Generate operations for modified tables
        for &name in &common_tables {
            let from_table = from_tables[name];
            let to_table = to_tables[name];
            operations.extend(self.diff_table(from_table, to_table));
        }

        // Also diff renamed tables (comparing old to new)
        for (&old_name, &new_name) in &renamed_tables {
            let from_table = from_tables[old_name];
            let to_table = to_tables[new_name];

            // Create a modified from_table with the new name for comparison
            let from_renamed = TableSchema {
                name: new_name.to_string(),
                ..from_table.clone()
            };
            operations.extend(self.diff_table(&from_renamed, to_table));
        }

        operations
    }

    /// Compares two tables and returns the operations needed.
    fn diff_table(&self, from: &TableSchema, to: &TableSchema) -> Vec<MigrationOperation> {
        let mut operations = Vec::new();
        let table_name = &to.name;

        // Compare columns
        let from_cols: HashMap<&str, &ColumnSchema> =
            from.columns.iter().map(|c| (c.name.as_str(), c)).collect();
        let to_cols: HashMap<&str, &ColumnSchema> =
            to.columns.iter().map(|c| (c.name.as_str(), c)).collect();

        let from_col_names: HashSet<&str> = from_cols.keys().copied().collect();
        let to_col_names: HashSet<&str> = to_cols.keys().copied().collect();

        // New columns
        for &col_name in to_col_names.difference(&from_col_names) {
            let col = to_cols[col_name];
            operations.push(MigrationOperation::add_column(table_name, col.clone()));
        }

        // Dropped columns
        for &col_name in from_col_names.difference(&to_col_names) {
            operations.push(MigrationOperation::drop_column(table_name, col_name));
        }

        // Modified columns
        for &col_name in from_col_names.intersection(&to_col_names) {
            let from_col = from_cols[col_name];
            let to_col = to_cols[col_name];

            if let Some(changes) = self.diff_column(from_col, to_col) {
                operations.push(MigrationOperation::AlterColumn {
                    table: table_name.clone(),
                    column_name: col_name.to_string(),
                    changes,
                    original: Some(from_col.clone()),
                });
            }
        }

        // Compare indexes
        operations.extend(self.diff_indexes(table_name, &from.indexes, &to.indexes));

        // Compare foreign keys
        operations.extend(self.diff_foreign_keys(table_name, &from.foreign_keys, &to.foreign_keys));

        // Compare unique constraints
        operations.extend(self.diff_unique_constraints(
            table_name,
            &from.unique_constraints,
            &to.unique_constraints,
        ));

        operations
    }

    /// Compares two columns and returns changes if any.
    fn diff_column(&self, from: &ColumnSchema, to: &ColumnSchema) -> Option<ColumnChanges> {
        let mut changes = ColumnChanges::new();

        if from.sql_type != to.sql_type {
            changes.sql_type = Some(to.sql_type.clone());
        }

        if from.nullable != to.nullable {
            changes.nullable = Some(to.nullable);
        }

        if from.default != to.default {
            changes.default = Some(to.default.clone());
        }

        if from.unique != to.unique {
            changes.unique = Some(to.unique);
        }

        if changes.is_empty() {
            None
        } else {
            Some(changes)
        }
    }

    /// Compares indexes and returns operations.
    fn diff_indexes(
        &self,
        table: &str,
        from: &[IndexSchema],
        to: &[IndexSchema],
    ) -> Vec<MigrationOperation> {
        let mut operations = Vec::new();

        let from_map: HashMap<&str, &IndexSchema> =
            from.iter().map(|i| (i.name.as_str(), i)).collect();
        let to_map: HashMap<&str, &IndexSchema> = to.iter().map(|i| (i.name.as_str(), i)).collect();

        let from_names: HashSet<&str> = from_map.keys().copied().collect();
        let to_names: HashSet<&str> = to_map.keys().copied().collect();

        // New indexes
        for &name in to_names.difference(&from_names) {
            let idx = to_map[name];
            operations.push(MigrationOperation::CreateIndex {
                name: name.to_string(),
                table: table.to_string(),
                columns: idx.columns.clone(),
                unique: idx.unique,
                condition: idx.condition.clone(),
                if_not_exists: false,
            });
        }

        // Dropped indexes
        for &name in from_names.difference(&to_names) {
            operations.push(MigrationOperation::DropIndex {
                name: name.to_string(),
                table: Some(table.to_string()),
                if_exists: false,
            });
        }

        // Modified indexes (drop + recreate)
        for &name in from_names.intersection(&to_names) {
            let from_idx = from_map[name];
            let to_idx = to_map[name];

            if from_idx != to_idx {
                operations.push(MigrationOperation::DropIndex {
                    name: name.to_string(),
                    table: Some(table.to_string()),
                    if_exists: false,
                });
                operations.push(MigrationOperation::CreateIndex {
                    name: name.to_string(),
                    table: table.to_string(),
                    columns: to_idx.columns.clone(),
                    unique: to_idx.unique,
                    condition: to_idx.condition.clone(),
                    if_not_exists: false,
                });
            }
        }

        operations
    }

    /// Compares foreign keys and returns operations.
    fn diff_foreign_keys(
        &self,
        table: &str,
        from: &[ForeignKeySchema],
        to: &[ForeignKeySchema],
    ) -> Vec<MigrationOperation> {
        let mut operations = Vec::new();

        let from_map: HashMap<&str, &ForeignKeySchema> =
            from.iter().map(|fk| (fk.name.as_str(), fk)).collect();
        let to_map: HashMap<&str, &ForeignKeySchema> =
            to.iter().map(|fk| (fk.name.as_str(), fk)).collect();

        let from_names: HashSet<&str> = from_map.keys().copied().collect();
        let to_names: HashSet<&str> = to_map.keys().copied().collect();

        // New foreign keys
        for &name in to_names.difference(&from_names) {
            let fk = to_map[name];
            operations.push(MigrationOperation::add_foreign_key(table, fk.clone()));
        }

        // Dropped foreign keys
        for &name in from_names.difference(&to_names) {
            operations.push(MigrationOperation::drop_foreign_key(table, name));
        }

        // Modified foreign keys (drop + recreate)
        for &name in from_names.intersection(&to_names) {
            let from_fk = from_map[name];
            let to_fk = to_map[name];

            if from_fk != to_fk {
                operations.push(MigrationOperation::drop_foreign_key(table, name));
                operations.push(MigrationOperation::add_foreign_key(table, to_fk.clone()));
            }
        }

        operations
    }

    /// Compares unique constraints and returns operations.
    fn diff_unique_constraints(
        &self,
        table: &str,
        from: &[UniqueConstraint],
        to: &[UniqueConstraint],
    ) -> Vec<MigrationOperation> {
        let mut operations = Vec::new();

        let from_map: HashMap<&str, &UniqueConstraint> =
            from.iter().map(|uc| (uc.name.as_str(), uc)).collect();
        let to_map: HashMap<&str, &UniqueConstraint> =
            to.iter().map(|uc| (uc.name.as_str(), uc)).collect();

        let from_names: HashSet<&str> = from_map.keys().copied().collect();
        let to_names: HashSet<&str> = to_map.keys().copied().collect();

        // New unique constraints
        for &name in to_names.difference(&from_names) {
            let uc = to_map[name];
            operations.push(MigrationOperation::add_unique_constraint(
                table,
                name,
                uc.columns.clone(),
            ));
        }

        // Dropped unique constraints
        for &name in from_names.difference(&to_names) {
            operations.push(MigrationOperation::drop_unique_constraint(table, name));
        }

        // Modified unique constraints (drop + recreate)
        for &name in from_names.intersection(&to_names) {
            let from_uc = from_map[name];
            let to_uc = to_map[name];

            if from_uc != to_uc {
                operations.push(MigrationOperation::drop_unique_constraint(table, name));
                operations.push(MigrationOperation::add_unique_constraint(
                    table,
                    name,
                    to_uc.columns.clone(),
                ));
            }
        }

        operations
    }

    /// Checks if two tables are similar (for rename detection).
    fn tables_similar(&self, a: &TableSchema, b: &TableSchema) -> bool {
        // Compare column names (ignoring order)
        let a_cols: HashSet<&str> = a.columns.iter().map(|c| c.name.as_str()).collect();
        let b_cols: HashSet<&str> = b.columns.iter().map(|c| c.name.as_str()).collect();

        let common = a_cols.intersection(&b_cols).count();
        let total = a_cols.union(&b_cols).count();

        if total == 0 {
            return false;
        }

        let similarity = common as f64 / total as f64;
        similarity >= self.options.rename_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::SqlType;

    fn detector() -> Autodetector {
        Autodetector::new()
    }

    #[test]
    fn test_detect_new_table() {
        let from = DatabaseSchema::new();
        let to = DatabaseSchema::new().table(
            TableSchema::new("users")
                .column(ColumnSchema::new("id", SqlType::BigInt).primary_key())
                .column(ColumnSchema::new("name", SqlType::Text)),
        );

        let ops = detector().diff(&from, &to);
        assert_eq!(ops.len(), 1);
        assert!(matches!(ops[0], MigrationOperation::CreateTable { .. }));
    }

    #[test]
    fn test_detect_dropped_table() {
        let from = DatabaseSchema::new().table(
            TableSchema::new("users")
                .column(ColumnSchema::new("id", SqlType::BigInt).primary_key()),
        );
        let to = DatabaseSchema::new();

        let ops = detector().diff(&from, &to);
        assert_eq!(ops.len(), 1);
        assert!(matches!(ops[0], MigrationOperation::DropTable { .. }));
    }

    #[test]
    fn test_detect_new_column() {
        let from = DatabaseSchema::new().table(
            TableSchema::new("users")
                .column(ColumnSchema::new("id", SqlType::BigInt).primary_key()),
        );
        let to = DatabaseSchema::new().table(
            TableSchema::new("users")
                .column(ColumnSchema::new("id", SqlType::BigInt).primary_key())
                .column(ColumnSchema::new("email", SqlType::Text)),
        );

        let ops = detector().diff(&from, &to);
        assert_eq!(ops.len(), 1);
        assert!(matches!(ops[0], MigrationOperation::AddColumn { .. }));
    }

    #[test]
    fn test_detect_dropped_column() {
        let from = DatabaseSchema::new().table(
            TableSchema::new("users")
                .column(ColumnSchema::new("id", SqlType::BigInt).primary_key())
                .column(ColumnSchema::new("email", SqlType::Text)),
        );
        let to = DatabaseSchema::new().table(
            TableSchema::new("users")
                .column(ColumnSchema::new("id", SqlType::BigInt).primary_key()),
        );

        let ops = detector().diff(&from, &to);
        assert_eq!(ops.len(), 1);
        assert!(matches!(ops[0], MigrationOperation::DropColumn { .. }));
    }

    #[test]
    fn test_detect_column_type_change() {
        let from = DatabaseSchema::new().table(
            TableSchema::new("users")
                .column(ColumnSchema::new("id", SqlType::BigInt).primary_key())
                .column(ColumnSchema::new("age", SqlType::Integer)),
        );
        let to = DatabaseSchema::new().table(
            TableSchema::new("users")
                .column(ColumnSchema::new("id", SqlType::BigInt).primary_key())
                .column(ColumnSchema::new("age", SqlType::BigInt)),
        );

        let ops = detector().diff(&from, &to);
        assert_eq!(ops.len(), 1);
        match &ops[0] {
            MigrationOperation::AlterColumn { changes, .. } => {
                assert_eq!(changes.sql_type, Some(SqlType::BigInt));
            }
            _ => panic!("Expected AlterColumn"),
        }
    }

    #[test]
    fn test_detect_column_nullability_change() {
        let from = DatabaseSchema::new().table(
            TableSchema::new("users")
                .column(ColumnSchema::new("id", SqlType::BigInt).primary_key())
                .column(ColumnSchema::new("name", SqlType::Text)),
        );
        let to = DatabaseSchema::new().table(
            TableSchema::new("users")
                .column(ColumnSchema::new("id", SqlType::BigInt).primary_key())
                .column(ColumnSchema::new("name", SqlType::Text).not_null()),
        );

        let ops = detector().diff(&from, &to);
        assert_eq!(ops.len(), 1);
        match &ops[0] {
            MigrationOperation::AlterColumn { changes, .. } => {
                assert_eq!(changes.nullable, Some(false));
            }
            _ => panic!("Expected AlterColumn"),
        }
    }

    #[test]
    fn test_detect_new_index() {
        let from = DatabaseSchema::new().table(
            TableSchema::new("users")
                .column(ColumnSchema::new("id", SqlType::BigInt).primary_key())
                .column(ColumnSchema::new("email", SqlType::Text)),
        );
        let to = DatabaseSchema::new().table(
            TableSchema::new("users")
                .column(ColumnSchema::new("id", SqlType::BigInt).primary_key())
                .column(ColumnSchema::new("email", SqlType::Text))
                .index(IndexSchema {
                    name: "idx_email".to_string(),
                    columns: vec!["email".to_string()],
                    unique: true,
                    condition: None,
                }),
        );

        let ops = detector().diff(&from, &to);
        assert_eq!(ops.len(), 1);
        assert!(matches!(ops[0], MigrationOperation::CreateIndex { .. }));
    }

    #[test]
    fn test_detect_table_rename() {
        let from = DatabaseSchema::new().table(
            TableSchema::new("users")
                .column(ColumnSchema::new("id", SqlType::BigInt).primary_key())
                .column(ColumnSchema::new("name", SqlType::Text)),
        );
        let to = DatabaseSchema::new().table(
            TableSchema::new("accounts")
                .column(ColumnSchema::new("id", SqlType::BigInt).primary_key())
                .column(ColumnSchema::new("name", SqlType::Text)),
        );

        let detector =
            Autodetector::with_options(AutodetectorOptions::new().with_rename_detection());
        let ops = detector.diff(&from, &to);

        // Should detect rename instead of drop+create
        assert_eq!(ops.len(), 1);
        match &ops[0] {
            MigrationOperation::RenameTable { old_name, new_name } => {
                assert_eq!(old_name, "users");
                assert_eq!(new_name, "accounts");
            }
            _ => panic!("Expected RenameTable, got {:?}", ops[0]),
        }
    }

    #[test]
    fn test_no_changes() {
        let schema = DatabaseSchema::new().table(
            TableSchema::new("users")
                .column(ColumnSchema::new("id", SqlType::BigInt).primary_key())
                .column(ColumnSchema::new("name", SqlType::Text)),
        );

        let ops = detector().diff(&schema, &schema);
        assert!(ops.is_empty());
    }

    #[test]
    fn test_complex_changes() {
        let from = DatabaseSchema::new()
            .table(
                TableSchema::new("users")
                    .column(ColumnSchema::new("id", SqlType::BigInt).primary_key())
                    .column(ColumnSchema::new("name", SqlType::Text))
                    .column(ColumnSchema::new("old_field", SqlType::Text)),
            )
            .table(
                TableSchema::new("to_drop")
                    .column(ColumnSchema::new("id", SqlType::BigInt).primary_key()),
            );

        let to = DatabaseSchema::new()
            .table(
                TableSchema::new("users")
                    .column(ColumnSchema::new("id", SqlType::BigInt).primary_key())
                    .column(ColumnSchema::new("name", SqlType::Varchar(255)).not_null())
                    .column(ColumnSchema::new("email", SqlType::Text)),
            )
            .table(
                TableSchema::new("posts")
                    .column(ColumnSchema::new("id", SqlType::BigInt).primary_key())
                    .column(ColumnSchema::new("title", SqlType::Text)),
            );

        let ops = detector().diff(&from, &to);

        // Should have:
        // - Create posts table
        // - Drop to_drop table
        // - Alter users.name (type and nullability)
        // - Add users.email
        // - Drop users.old_field
        assert!(ops.len() >= 4);
    }
}
