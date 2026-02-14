//! Schema diff engine for auto-migration generation.
//!
//! Compares an "old" (current DB) and "new" (desired from code)
//! [`SchemaSnapshot`] and produces a `Vec<Operation>` representing
//! the DDL changes needed to migrate from old to new.

use std::collections::BTreeSet;

use crate::schema::{RustTypeMapping, TableSchema};

use super::column_builder::ColumnDefinition;
use super::dialect::MigrationDialect;
use super::operation::{
    AddColumnOp, AddForeignKeyOp, AlterColumnChange, AlterColumnOp, CreateIndexOp, CreateTableOp,
    DropColumnOp, DropForeignKeyOp, DropIndexOp, DropTableOp, Operation,
};
use super::snapshot::{
    ColumnSnapshot, ForeignKeySnapshot, IndexSnapshot, SchemaSnapshot, TableSnapshot,
};

/// Minimum normalized similarity score (0.0–1.0) for a
/// (dropped, added) column pair to be flagged as a possible rename.
const RENAME_SIMILARITY_THRESHOLD: f64 = 0.4;

// ================================================================
// String similarity helpers
// ================================================================

/// Computes the Levenshtein edit distance between two strings.
fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let m = a.len();
    let n = b.len();
    let mut prev = (0..=n).collect::<Vec<_>>();
    let mut curr = vec![0; n + 1];
    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[n]
}

/// Returns a normalized similarity score in `[0.0, 1.0]`.
/// 1.0 means identical, 0.0 means completely different.
fn similarity(a: &str, b: &str) -> f64 {
    let max_len = a.len().max(b.len());
    if max_len == 0 {
        return 1.0;
    }
    1.0 - (levenshtein(a, b) as f64 / max_len as f64)
}

// ================================================================
// Public types
// ================================================================

/// A change that cannot be automatically resolved and requires
/// user intervention.
#[derive(Debug, Clone, PartialEq)]
pub enum AmbiguousChange {
    /// A dropped and added column with the same type and similar
    /// names, suggesting a possible rename.
    PossibleRename {
        /// Table containing the columns.
        table: String,
        /// The column that was dropped.
        old_column: String,
        /// The column that was added.
        new_column: String,
        /// Name similarity score (0.0–1.0).
        similarity: f64,
    },
    /// A dropped and added table with the same column structure,
    /// suggesting a possible rename.
    PossibleTableRename {
        /// The table that was dropped.
        old_table: String,
        /// The table that was added.
        new_table: String,
        /// Name similarity score (0.0–1.0).
        similarity: f64,
    },
}

/// Informational warnings about changes that the diff engine
/// detected but cannot (or should not) translate into DDL
/// operations automatically.
#[derive(Debug, Clone, PartialEq)]
pub enum DiffWarning {
    /// A column's primary key status changed. This typically
    /// requires table recreation on most databases.
    PrimaryKeyChange {
        /// Table name.
        table: String,
        /// Column name.
        column: String,
        /// New value of the primary_key flag.
        new_value: bool,
    },
    /// A column's autoincrement status changed. Most databases
    /// cannot alter this without recreating the table.
    AutoincrementChange {
        /// Table name.
        table: String,
        /// Column name.
        column: String,
        /// New value of the autoincrement flag.
        new_value: bool,
    },
    /// The relative ordering of columns changed. Most databases
    /// cannot reorder columns without recreating the table.
    ColumnOrderChanged {
        /// Table name.
        table: String,
        /// Column names in the old order.
        old_order: Vec<String>,
        /// Column names in the new order.
        new_order: Vec<String>,
    },
}

/// Result of comparing two schema snapshots.
#[derive(Debug, Clone, PartialEq)]
pub struct SchemaDiff {
    /// The migration operations to apply.
    pub operations: Vec<Operation>,
    /// Changes that may be renames and need user confirmation.
    pub ambiguous: Vec<AmbiguousChange>,
    /// Informational warnings (destructive changes that require
    /// manual intervention, column order changes, etc.).
    pub warnings: Vec<DiffWarning>,
}

impl SchemaDiff {
    /// Returns `true` if there are no changes at all (no ops,
    /// no ambiguous changes, and no warnings).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty() && self.ambiguous.is_empty() && self.warnings.is_empty()
    }

    /// Convenience: generates SQL for every operation using the
    /// given dialect.
    #[must_use]
    pub fn to_sql(&self, dialect: &impl MigrationDialect) -> Vec<String> {
        self.operations
            .iter()
            .map(|op| dialect.generate_sql(op))
            .collect()
    }

    /// Attempts to reverse the entire diff. Returns `None` if any
    /// operation is non-reversible.
    #[must_use]
    pub fn reverse(&self) -> Option<Self> {
        let mut reversed = Vec::new();
        for op in self.operations.iter().rev() {
            reversed.push(op.reverse()?);
        }
        Some(Self {
            operations: reversed,
            ambiguous: vec![],
            warnings: vec![],
        })
    }

    /// Returns `true` if every operation is reversible.
    #[must_use]
    pub fn is_reversible(&self) -> bool {
        self.operations.iter().all(Operation::is_reversible)
    }

    /// Returns references to the non-reversible operations.
    #[must_use]
    pub fn non_reversible_operations(&self) -> Vec<&Operation> {
        self.operations
            .iter()
            .filter(|op| !op.is_reversible())
            .collect()
    }
}

// ================================================================
// Table-level diff
// ================================================================

/// Compares a single table's current and desired snapshots,
/// producing the operations needed to migrate.
fn diff_table(table_name: &str, old: &TableSnapshot, new: &TableSnapshot) -> SchemaDiff {
    let old_names: BTreeSet<&str> = old.columns.iter().map(|c| c.name.as_str()).collect();
    let new_names: BTreeSet<&str> = new.columns.iter().map(|c| c.name.as_str()).collect();

    let dropped: Vec<&str> = old_names.difference(&new_names).copied().collect();
    let added: Vec<&str> = new_names.difference(&old_names).copied().collect();
    let common: BTreeSet<&str> = old_names.intersection(&new_names).copied().collect();

    let mut operations = Vec::new();
    let mut ambiguous = Vec::new();
    let mut warnings = Vec::new();

    // ---- N:M rename detection with similarity scoring ----------
    let mut rename_dropped: BTreeSet<&str> = BTreeSet::new();
    let mut rename_added: BTreeSet<&str> = BTreeSet::new();

    // Build candidate pairs: (dropped, added, similarity)
    let mut candidates: Vec<(&str, &str, f64)> = Vec::new();
    for &d in &dropped {
        let old_col = old.column(d).unwrap();
        for &a in &added {
            let new_col = new.column(a).unwrap();
            if old_col.data_type == new_col.data_type {
                let sim = similarity(d, a);
                if sim >= RENAME_SIMILARITY_THRESHOLD {
                    candidates.push((d, a, sim));
                }
            }
        }
    }
    // Greedy matching: highest similarity first.
    candidates.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
    for (d, a, sim) in &candidates {
        if rename_dropped.contains(d) || rename_added.contains(a) {
            continue;
        }
        ambiguous.push(AmbiguousChange::PossibleRename {
            table: table_name.to_string(),
            old_column: d.to_string(),
            new_column: a.to_string(),
            similarity: *sim,
        });
        rename_dropped.insert(d);
        rename_added.insert(a);
    }

    // ---- AddColumn for truly new columns -----------------------
    for &name in &added {
        if rename_added.contains(name) {
            continue;
        }
        let col = new.column(name).unwrap();
        operations.push(Operation::AddColumn(AddColumnOp {
            table: table_name.to_string(),
            column: snapshot_to_column_def(col),
        }));
    }

    // ---- AlterColumn for changed properties on common cols -----
    for &name in &common {
        let old_col = old.column(name).unwrap();
        let new_col = new.column(name).unwrap();

        if old_col.data_type != new_col.data_type {
            operations.push(Operation::AlterColumn(AlterColumnOp {
                table: table_name.to_string(),
                column: name.to_string(),
                change: AlterColumnChange::SetDataType(new_col.data_type.clone()),
            }));
        }

        if old_col.nullable != new_col.nullable {
            operations.push(Operation::AlterColumn(AlterColumnOp {
                table: table_name.to_string(),
                column: name.to_string(),
                change: AlterColumnChange::SetNullable(new_col.nullable),
            }));
        }

        if old_col.unique != new_col.unique {
            operations.push(Operation::AlterColumn(AlterColumnOp {
                table: table_name.to_string(),
                column: name.to_string(),
                change: AlterColumnChange::SetUnique(new_col.unique),
            }));
        }

        if old_col.primary_key != new_col.primary_key {
            warnings.push(DiffWarning::PrimaryKeyChange {
                table: table_name.to_string(),
                column: name.to_string(),
                new_value: new_col.primary_key,
            });
        }

        if old_col.autoincrement != new_col.autoincrement {
            warnings.push(DiffWarning::AutoincrementChange {
                table: table_name.to_string(),
                column: name.to_string(),
                new_value: new_col.autoincrement,
            });
        }

        match (&old_col.default, &new_col.default) {
            (None, Some(new_default)) => {
                operations.push(Operation::AlterColumn(AlterColumnOp {
                    table: table_name.to_string(),
                    column: name.to_string(),
                    change: AlterColumnChange::SetDefault(new_default.clone()),
                }));
            }
            (Some(_), None) => {
                operations.push(Operation::AlterColumn(AlterColumnOp {
                    table: table_name.to_string(),
                    column: name.to_string(),
                    change: AlterColumnChange::DropDefault,
                }));
            }
            (Some(old_def), Some(new_def)) if old_def != new_def => {
                operations.push(Operation::AlterColumn(AlterColumnOp {
                    table: table_name.to_string(),
                    column: name.to_string(),
                    change: AlterColumnChange::SetDefault(new_def.clone()),
                }));
            }
            _ => {}
        }
    }

    // ---- DropColumn for truly removed columns ------------------
    for &name in &dropped {
        if rename_dropped.contains(name) {
            continue;
        }
        operations.push(Operation::DropColumn(DropColumnOp {
            table: table_name.to_string(),
            column: name.to_string(),
        }));
    }

    // ---- Index diff --------------------------------------------
    diff_indexes(table_name, old, new, &mut operations);

    // ---- Foreign key diff --------------------------------------
    diff_foreign_keys(table_name, old, new, &mut operations);

    // ---- Column ordering detection -----------------------------
    detect_column_order_change(table_name, old, new, &common, &mut warnings);

    SchemaDiff {
        operations,
        ambiguous,
        warnings,
    }
}

// ================================================================
// Index / FK diffing helpers
// ================================================================

/// Two indexes are considered equivalent if they cover the same
/// columns, uniqueness, and type. Names are ignored because they
/// may differ between environments.
fn indexes_equivalent(a: &IndexSnapshot, b: &IndexSnapshot) -> bool {
    a.columns == b.columns
        && a.unique == b.unique
        && a.index_type == b.index_type
        && a.condition == b.condition
}

/// Diffs indexes between old and new table snapshots.
fn diff_indexes(
    table_name: &str,
    old: &TableSnapshot,
    new: &TableSnapshot,
    operations: &mut Vec<Operation>,
) {
    // Indexes present in old but not in new → DropIndex.
    for old_idx in &old.indexes {
        let still_exists = new.indexes.iter().any(|n| indexes_equivalent(old_idx, n));
        if !still_exists {
            operations.push(Operation::DropIndex(DropIndexOp {
                name: old_idx.name.clone(),
                table: Some(table_name.to_string()),
                if_exists: false,
            }));
        }
    }
    // Indexes present in new but not in old → CreateIndex.
    for new_idx in &new.indexes {
        let already_exists = old.indexes.iter().any(|o| indexes_equivalent(o, new_idx));
        if !already_exists {
            operations.push(Operation::CreateIndex(CreateIndexOp {
                name: new_idx.name.clone(),
                table: table_name.to_string(),
                columns: new_idx.columns.clone(),
                unique: new_idx.unique,
                index_type: new_idx.index_type,
                if_not_exists: false,
                condition: new_idx.condition.clone(),
            }));
        }
    }
}

/// Two foreign keys are equivalent if they reference the same
/// columns, target table, target columns, and actions.
fn fks_equivalent(a: &ForeignKeySnapshot, b: &ForeignKeySnapshot) -> bool {
    a.columns == b.columns
        && a.references_table == b.references_table
        && a.references_columns == b.references_columns
        && a.on_delete == b.on_delete
        && a.on_update == b.on_update
}

/// Diffs foreign keys between old and new table snapshots.
fn diff_foreign_keys(
    table_name: &str,
    old: &TableSnapshot,
    new: &TableSnapshot,
    operations: &mut Vec<Operation>,
) {
    // FKs present in old but not in new → DropForeignKey.
    for old_fk in &old.foreign_keys {
        let still_exists = new.foreign_keys.iter().any(|n| fks_equivalent(old_fk, n));
        if !still_exists {
            if let Some(ref name) = old_fk.name {
                operations.push(Operation::DropForeignKey(DropForeignKeyOp {
                    table: table_name.to_string(),
                    name: name.clone(),
                }));
            }
        }
    }
    // FKs present in new but not in old → AddForeignKey.
    for new_fk in &new.foreign_keys {
        let already_exists = old.foreign_keys.iter().any(|o| fks_equivalent(o, new_fk));
        if !already_exists {
            operations.push(Operation::AddForeignKey(AddForeignKeyOp {
                table: table_name.to_string(),
                name: new_fk.name.clone(),
                columns: new_fk.columns.clone(),
                references_table: new_fk.references_table.clone(),
                references_columns: new_fk.references_columns.clone(),
                on_delete: new_fk.on_delete,
                on_update: new_fk.on_update,
            }));
        }
    }
}

// ================================================================
// Column ordering
// ================================================================

/// Detects whether the relative order of common columns changed
/// between old and new snapshots.
fn detect_column_order_change(
    table_name: &str,
    old: &TableSnapshot,
    new: &TableSnapshot,
    common: &BTreeSet<&str>,
    warnings: &mut Vec<DiffWarning>,
) {
    let old_order: Vec<String> = old
        .columns
        .iter()
        .filter(|c| common.contains(c.name.as_str()))
        .map(|c| c.name.clone())
        .collect();
    let new_order: Vec<String> = new
        .columns
        .iter()
        .filter(|c| common.contains(c.name.as_str()))
        .map(|c| c.name.clone())
        .collect();

    if old_order != new_order {
        warnings.push(DiffWarning::ColumnOrderChanged {
            table: table_name.to_string(),
            old_order,
            new_order,
        });
    }
}

// ================================================================
// Helpers
// ================================================================

/// Converts a `ColumnSnapshot` into a `ColumnDefinition` for use
/// in `AddColumnOp`.
fn snapshot_to_column_def(col: &ColumnSnapshot) -> ColumnDefinition {
    ColumnDefinition {
        name: col.name.clone(),
        data_type: col.data_type.clone(),
        nullable: col.nullable,
        default: col.default.clone(),
        primary_key: col.primary_key,
        unique: col.unique,
        autoincrement: col.autoincrement,
        references: None,
        check: None,
        collation: None,
    }
}

// ================================================================
// Schema-level diff
// ================================================================

/// Compares two full schema snapshots and produces the operations
/// needed to migrate from `current` to `desired`.
///
/// Operation ordering: CreateTable > AddColumn > AlterColumn >
/// DropColumn > DropTable (avoids FK constraint violations).
pub fn auto_diff_schema(current: &SchemaSnapshot, desired: &SchemaSnapshot) -> SchemaDiff {
    let current_tables: BTreeSet<&str> = current.tables.keys().map(String::as_str).collect();
    let desired_tables: BTreeSet<&str> = desired.tables.keys().map(String::as_str).collect();

    let dropped_tables: Vec<&str> = current_tables
        .difference(&desired_tables)
        .copied()
        .collect();
    let added_tables: Vec<&str> = desired_tables
        .difference(&current_tables)
        .copied()
        .collect();
    let common_tables: Vec<&str> = current_tables
        .intersection(&desired_tables)
        .copied()
        .collect();

    let mut create_ops = Vec::new();
    let mut add_ops = Vec::new();
    let mut alter_ops = Vec::new();
    let mut drop_col_ops = Vec::new();
    let mut drop_table_ops = Vec::new();
    let mut ambiguous = Vec::new();
    let mut warnings = Vec::new();

    // ---- N:M table rename detection ----------------------------
    let mut rename_dropped: BTreeSet<&str> = BTreeSet::new();
    let mut rename_added: BTreeSet<&str> = BTreeSet::new();

    // Build candidate pairs: (dropped, added, similarity)
    let mut candidates: Vec<(&str, &str, f64)> = Vec::new();
    for &d in &dropped_tables {
        let old_table = &current.tables[d];
        for &a in &added_tables {
            let new_table = &desired.tables[a];
            if tables_have_same_columns(old_table, new_table) {
                let sim = similarity(d, a);
                candidates.push((d, a, sim));
            }
        }
    }
    candidates.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
    for (d, a, sim) in &candidates {
        if rename_dropped.contains(d) || rename_added.contains(a) {
            continue;
        }
        ambiguous.push(AmbiguousChange::PossibleTableRename {
            old_table: d.to_string(),
            new_table: a.to_string(),
            similarity: *sim,
        });
        rename_dropped.insert(d);
        rename_added.insert(a);
    }

    // ---- New tables -> CreateTable -----------------------------
    for &name in &added_tables {
        if rename_added.contains(name) {
            continue;
        }
        let table = &desired.tables[name];
        let columns = table.columns.iter().map(snapshot_to_column_def).collect();
        create_ops.push(Operation::CreateTable(CreateTableOp {
            name: name.to_string(),
            columns,
            constraints: vec![],
            if_not_exists: false,
        }));
    }

    // ---- Existing tables -> diff columns -----------------------
    for &name in &common_tables {
        let old_table = &current.tables[name];
        let new_table = &desired.tables[name];
        let table_diff = diff_table(name, old_table, new_table);

        for op in table_diff.operations {
            match &op {
                Operation::AddColumn(_) => add_ops.push(op),
                Operation::AlterColumn(_) => {
                    alter_ops.push(op);
                }
                Operation::DropColumn(_) => {
                    drop_col_ops.push(op);
                }
                _ => add_ops.push(op),
            }
        }
        ambiguous.extend(table_diff.ambiguous);
        warnings.extend(table_diff.warnings);
    }

    // ---- Dropped tables -> DropTable ---------------------------
    for &name in &dropped_tables {
        if rename_dropped.contains(name) {
            continue;
        }
        drop_table_ops.push(Operation::DropTable(DropTableOp {
            name: name.to_string(),
            if_exists: false,
            cascade: false,
        }));
    }

    // Assemble in safe order.
    let mut operations = Vec::new();
    operations.extend(create_ops);
    operations.extend(add_ops);
    operations.extend(alter_ops);
    operations.extend(drop_col_ops);
    operations.extend(drop_table_ops);

    SchemaDiff {
        operations,
        ambiguous,
        warnings,
    }
}

/// Compares a single table's current snapshot against the desired
/// schema derived from a `#[derive(Table)]` struct.
pub fn auto_diff_table<T: TableSchema>(
    current: &TableSnapshot,
    dialect: &impl RustTypeMapping,
) -> SchemaDiff {
    let desired = TableSnapshot::from_table_schema::<T>(dialect);
    diff_table(&desired.name, current, &desired)
}

/// Returns `true` if two table snapshots have identical column
/// structure (names, types, nullable, etc.).
fn tables_have_same_columns(a: &TableSnapshot, b: &TableSnapshot) -> bool {
    if a.columns.len() != b.columns.len() {
        return false;
    }
    a.columns.iter().zip(b.columns.iter()).all(|(ac, bc)| {
        ac.name == bc.name
            && ac.data_type == bc.data_type
            && ac.nullable == bc.nullable
            && ac.primary_key == bc.primary_key
            && ac.unique == bc.unique
            && ac.autoincrement == bc.autoincrement
            && ac.default == bc.default
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::DataType;
    use crate::migrations::column_builder::{DefaultValue, ForeignKeyAction};
    use crate::migrations::operation::IndexType;

    // ============================================================
    // Helpers
    // ============================================================

    fn col(name: &str, data_type: DataType, nullable: bool) -> ColumnSnapshot {
        ColumnSnapshot {
            name: name.to_string(),
            data_type,
            nullable,
            primary_key: false,
            unique: false,
            autoincrement: false,
            default: None,
        }
    }

    fn pk_col(name: &str, data_type: DataType) -> ColumnSnapshot {
        ColumnSnapshot {
            name: name.to_string(),
            data_type,
            nullable: false,
            primary_key: true,
            unique: false,
            autoincrement: true,
            default: None,
        }
    }

    fn table(name: &str, columns: Vec<ColumnSnapshot>) -> TableSnapshot {
        TableSnapshot {
            name: name.to_string(),
            columns,
            indexes: vec![],
            foreign_keys: vec![],
        }
    }

    fn schema(tables: Vec<TableSnapshot>) -> SchemaSnapshot {
        let mut s = SchemaSnapshot::new();
        for t in tables {
            s.add_table(t);
        }
        s
    }

    // ============================================================
    // Levenshtein / similarity unit tests
    // ============================================================

    #[test]
    fn levenshtein_basic() {
        assert_eq!(levenshtein("", ""), 0);
        assert_eq!(levenshtein("abc", "abc"), 0);
        assert_eq!(levenshtein("abc", ""), 3);
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("kitten", "sitting"), 3);
    }

    #[test]
    fn similarity_basic() {
        assert!((similarity("abc", "abc") - 1.0).abs() < f64::EPSILON);
        assert!((similarity("", "") - 1.0).abs() < f64::EPSILON);
        // "name" vs "full_name": dist 5, max 9, sim ~0.44
        let s = similarity("name", "full_name");
        assert!(s > 0.4 && s < 0.5, "sim={s}");
    }

    // ============================================================
    // Original diff tests (updated for new SchemaDiff fields)
    // ============================================================

    #[test]
    fn no_changes_produces_empty_diff() {
        let t = table(
            "users",
            vec![
                pk_col("id", DataType::Bigint),
                col("name", DataType::Text, false),
            ],
        );
        let diff = diff_table("users", &t, &t);
        assert!(diff.is_empty());
    }

    #[test]
    fn new_table_detected() {
        let current = schema(vec![]);
        let desired = schema(vec![table("users", vec![pk_col("id", DataType::Bigint)])]);
        let diff = auto_diff_schema(&current, &desired);
        assert_eq!(diff.operations.len(), 1);
        assert!(matches!(
            &diff.operations[0],
            Operation::CreateTable(op) if op.name == "users"
        ));
    }

    #[test]
    fn dropped_table_detected() {
        let current = schema(vec![table("users", vec![pk_col("id", DataType::Bigint)])]);
        let desired = schema(vec![]);
        let diff = auto_diff_schema(&current, &desired);
        assert_eq!(diff.operations.len(), 1);
        assert!(matches!(
            &diff.operations[0],
            Operation::DropTable(op) if op.name == "users"
        ));
    }

    #[test]
    fn added_column_detected() {
        let old = table("users", vec![pk_col("id", DataType::Bigint)]);
        let new = table(
            "users",
            vec![
                pk_col("id", DataType::Bigint),
                col("email", DataType::Text, true),
            ],
        );
        let diff = diff_table("users", &old, &new);
        assert_eq!(diff.operations.len(), 1);
        assert!(matches!(
            &diff.operations[0],
            Operation::AddColumn(op)
                if op.table == "users"
                    && op.column.name == "email"
        ));
    }

    #[test]
    fn dropped_column_detected() {
        let old = table(
            "users",
            vec![
                pk_col("id", DataType::Bigint),
                col("email", DataType::Text, true),
            ],
        );
        let new = table("users", vec![pk_col("id", DataType::Bigint)]);
        let diff = diff_table("users", &old, &new);
        assert_eq!(diff.operations.len(), 1);
        assert!(matches!(
            &diff.operations[0],
            Operation::DropColumn(op)
                if op.table == "users" && op.column == "email"
        ));
    }

    #[test]
    fn type_change_detected() {
        let old = table(
            "users",
            vec![
                pk_col("id", DataType::Bigint),
                col("score", DataType::Integer, false),
            ],
        );
        let new = table(
            "users",
            vec![
                pk_col("id", DataType::Bigint),
                col("score", DataType::Bigint, false),
            ],
        );
        let diff = diff_table("users", &old, &new);
        assert_eq!(diff.operations.len(), 1);
        assert!(matches!(
            &diff.operations[0],
            Operation::AlterColumn(op)
                if op.column == "score"
                    && op.change
                        == AlterColumnChange::SetDataType(
                            DataType::Bigint
                        )
        ));
    }

    #[test]
    fn nullable_change_detected() {
        let old = table(
            "users",
            vec![
                pk_col("id", DataType::Bigint),
                col("email", DataType::Text, false),
            ],
        );
        let new = table(
            "users",
            vec![
                pk_col("id", DataType::Bigint),
                col("email", DataType::Text, true),
            ],
        );
        let diff = diff_table("users", &old, &new);
        assert_eq!(diff.operations.len(), 1);
        assert!(matches!(
            &diff.operations[0],
            Operation::AlterColumn(op)
                if op.column == "email"
                    && op.change
                        == AlterColumnChange::SetNullable(true)
        ));
    }

    #[test]
    fn default_added() {
        let old = table("t", vec![col("active", DataType::Boolean, false)]);
        let mut new_col = col("active", DataType::Boolean, false);
        new_col.default = Some(DefaultValue::Expression("TRUE".into()));
        let new = table("t", vec![new_col]);
        let diff = diff_table("t", &old, &new);
        assert_eq!(diff.operations.len(), 1);
        assert!(matches!(
            &diff.operations[0],
            Operation::AlterColumn(op)
                if matches!(
                    &op.change,
                    AlterColumnChange::SetDefault(
                        DefaultValue::Expression(s)
                    ) if s == "TRUE"
                )
        ));
    }

    #[test]
    fn default_changed() {
        let mut old_col = col("count", DataType::Integer, false);
        old_col.default = Some(DefaultValue::Integer(0));
        let old = table("t", vec![old_col]);

        let mut new_col = col("count", DataType::Integer, false);
        new_col.default = Some(DefaultValue::Integer(1));
        let new = table("t", vec![new_col]);

        let diff = diff_table("t", &old, &new);
        assert_eq!(diff.operations.len(), 1);
        assert!(matches!(
            &diff.operations[0],
            Operation::AlterColumn(op)
                if op.change
                    == AlterColumnChange::SetDefault(
                        DefaultValue::Integer(1)
                    )
        ));
    }

    #[test]
    fn default_removed() {
        let mut old_col = col("active", DataType::Boolean, false);
        old_col.default = Some(DefaultValue::Expression("TRUE".into()));
        let old = table("t", vec![old_col]);
        let new = table("t", vec![col("active", DataType::Boolean, false)]);
        let diff = diff_table("t", &old, &new);
        assert_eq!(diff.operations.len(), 1);
        assert!(matches!(
            &diff.operations[0],
            Operation::AlterColumn(op)
                if op.change == AlterColumnChange::DropDefault
        ));
    }

    // ============================================================
    // Rename detection (N:M with similarity)
    // ============================================================

    #[test]
    fn ambiguous_rename_detected() {
        // "name" -> "full_name" (sim ~0.44, above threshold 0.4)
        let old = table(
            "users",
            vec![
                pk_col("id", DataType::Bigint),
                col("name", DataType::Text, false),
            ],
        );
        let new = table(
            "users",
            vec![
                pk_col("id", DataType::Bigint),
                col("full_name", DataType::Text, false),
            ],
        );
        let diff = diff_table("users", &old, &new);

        assert!(diff.operations.is_empty());
        assert_eq!(diff.ambiguous.len(), 1);
        match &diff.ambiguous[0] {
            AmbiguousChange::PossibleRename {
                table,
                old_column,
                new_column,
                ..
            } => {
                assert_eq!(table, "users");
                assert_eq!(old_column, "name");
                assert_eq!(new_column, "full_name");
            }
            other => {
                panic!("Expected PossibleRename, got {other:?}")
            }
        }
    }

    #[test]
    fn ambiguous_rename_not_triggered_different_types() {
        let old = table(
            "users",
            vec![
                pk_col("id", DataType::Bigint),
                col("name", DataType::Text, false),
            ],
        );
        let new = table(
            "users",
            vec![
                pk_col("id", DataType::Bigint),
                col("full_name", DataType::Integer, false),
            ],
        );
        let diff = diff_table("users", &old, &new);
        assert!(diff.ambiguous.is_empty());
        assert_eq!(diff.operations.len(), 2);
    }

    #[test]
    fn low_similarity_produces_add_drop_not_rename() {
        // "body" vs "summary" — similarity ~0.14, below threshold.
        let old = table(
            "t",
            vec![
                pk_col("id", DataType::Bigint),
                col("body", DataType::Text, false),
            ],
        );
        let new = table(
            "t",
            vec![
                pk_col("id", DataType::Bigint),
                col("summary", DataType::Text, false),
            ],
        );
        let diff = diff_table("t", &old, &new);

        // Below threshold → no rename, just drop + add.
        assert!(diff.ambiguous.is_empty());
        assert_eq!(diff.operations.len(), 2);
    }

    #[test]
    fn n_m_rename_detection() {
        // Two drops + two adds with matching types.
        // "user_name" → "username" (high sim), "addr" → "address"
        // (high sim).
        let old = table(
            "t",
            vec![
                pk_col("id", DataType::Bigint),
                col("user_name", DataType::Text, false),
                col("addr", DataType::Text, false),
            ],
        );
        let new = table(
            "t",
            vec![
                pk_col("id", DataType::Bigint),
                col("username", DataType::Text, false),
                col("address", DataType::Text, false),
            ],
        );
        let diff = diff_table("t", &old, &new);

        assert!(diff.operations.is_empty());
        assert_eq!(diff.ambiguous.len(), 2);
    }

    #[test]
    fn multiple_changes_combined() {
        let old = table(
            "users",
            vec![
                pk_col("id", DataType::Bigint),
                col("name", DataType::Text, false),
                col("old_field", DataType::Integer, false),
            ],
        );
        let new = table(
            "users",
            vec![
                pk_col("id", DataType::Bigint),
                col("name", DataType::Varchar(Some(255)), true),
                col("new_field", DataType::Boolean, false),
            ],
        );
        let diff = diff_table("users", &old, &new);

        // name: type + nullable = 2 alters
        // old_field→new_field: different types → no rename →
        // add + drop = 2
        assert!(diff.ambiguous.is_empty());
        assert_eq!(diff.operations.len(), 4);
    }

    #[test]
    fn operation_ordering_in_schema_diff() {
        let current = schema(vec![
            table(
                "to_drop",
                vec![
                    pk_col("id", DataType::Bigint),
                    col("legacy", DataType::Text, false),
                ],
            ),
            table(
                "to_alter",
                vec![
                    pk_col("id", DataType::Bigint),
                    col("alpha", DataType::Text, false),
                    col("beta", DataType::Integer, false),
                ],
            ),
        ]);
        let desired = schema(vec![
            table("to_create", vec![pk_col("id", DataType::Bigint)]),
            table(
                "to_alter",
                vec![
                    pk_col("id", DataType::Bigint),
                    col("xxx", DataType::Boolean, false),
                    col("yyy", DataType::Real, false),
                ],
            ),
        ]);
        let diff = auto_diff_schema(&current, &desired);

        let mut saw_create = false;
        let mut saw_add = false;
        let mut saw_drop_col = false;
        let mut saw_drop_table = false;

        for op in &diff.operations {
            match op {
                Operation::CreateTable(_) => {
                    assert!(!saw_add && !saw_drop_col && !saw_drop_table);
                    saw_create = true;
                }
                Operation::AddColumn(_) => {
                    assert!(
                        !saw_drop_col && !saw_drop_table,
                        "AddColumn before DropColumn/DropTable"
                    );
                    saw_add = true;
                }
                Operation::DropColumn(_) => {
                    assert!(!saw_drop_table, "DropColumn before DropTable");
                    saw_drop_col = true;
                }
                Operation::DropTable(_) => {
                    saw_drop_table = true;
                }
                _ => {}
            }
        }

        assert!(saw_create);
        assert!(saw_add);
        assert!(saw_drop_col);
        assert!(saw_drop_table);
    }

    #[test]
    fn possible_table_rename_detected() {
        let current = schema(vec![table(
            "users",
            vec![
                pk_col("id", DataType::Bigint),
                col("name", DataType::Text, false),
            ],
        )]);
        let desired = schema(vec![table(
            "accounts",
            vec![
                pk_col("id", DataType::Bigint),
                col("name", DataType::Text, false),
            ],
        )]);
        let diff = auto_diff_schema(&current, &desired);

        assert!(diff.operations.is_empty());
        assert_eq!(diff.ambiguous.len(), 1);
        match &diff.ambiguous[0] {
            AmbiguousChange::PossibleTableRename {
                old_table,
                new_table,
                ..
            } => {
                assert_eq!(old_table, "users");
                assert_eq!(new_table, "accounts");
            }
            other => {
                panic!("Expected PossibleTableRename, got {other:?}")
            }
        }
    }

    #[test]
    fn auto_diff_table_works() {
        use crate::migrations::SqliteDialect;
        use crate::schema::{ColumnSchema, Table};

        struct MyTable;
        struct MyRow;

        impl Table for MyTable {
            type Row = MyRow;
            const NAME: &'static str = "items";
            const COLUMNS: &'static [&'static str] = &["id", "title"];
            const PRIMARY_KEY: Option<&'static str> = Some("id");
        }

        impl TableSchema for MyTable {
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
                    name: "title",
                    rust_type: "String",
                    nullable: false,
                    primary_key: false,
                    unique: false,
                    autoincrement: false,
                    default_expr: None,
                },
            ];
        }

        let dialect = SqliteDialect::new();
        let current = table("items", vec![pk_col("id", DataType::Bigint)]);
        let diff = auto_diff_table::<MyTable>(&current, &dialect);

        assert_eq!(diff.operations.len(), 1);
        assert!(matches!(
            &diff.operations[0],
            Operation::AddColumn(op)
                if op.column.name == "title"
                    && op.column.data_type == DataType::Text
        ));
    }

    // ============================================================
    // Unique change detection
    // ============================================================

    #[test]
    fn unique_change_detected() {
        let mut old_col = col("email", DataType::Text, false);
        old_col.unique = false;
        let old = table("users", vec![old_col]);

        let mut new_col = col("email", DataType::Text, false);
        new_col.unique = true;
        let new = table("users", vec![new_col]);

        let diff = diff_table("users", &old, &new);
        assert!(diff.operations.iter().any(|op| matches!(
            op,
            Operation::AlterColumn(a)
                if a.column == "email"
                    && a.change
                        == AlterColumnChange::SetUnique(true)
        )));
    }

    // ============================================================
    // Warning detection
    // ============================================================

    #[test]
    fn primary_key_change_emits_warning() {
        let mut old_col = col("email", DataType::Text, false);
        old_col.primary_key = false;
        let old = table("t", vec![old_col]);

        let mut new_col = col("email", DataType::Text, false);
        new_col.primary_key = true;
        let new = table("t", vec![new_col]);

        let diff = diff_table("t", &old, &new);
        assert!(diff.warnings.iter().any(|w| matches!(
            w,
            DiffWarning::PrimaryKeyChange {
                column,
                new_value: true,
                ..
            } if column == "email"
        )));
    }

    #[test]
    fn autoincrement_change_emits_warning() {
        let mut old_col = col("id", DataType::Bigint, false);
        old_col.autoincrement = false;
        let old = table("t", vec![old_col]);

        let mut new_col = col("id", DataType::Bigint, false);
        new_col.autoincrement = true;
        let new = table("t", vec![new_col]);

        let diff = diff_table("t", &old, &new);
        assert!(diff.warnings.iter().any(|w| matches!(
            w,
            DiffWarning::AutoincrementChange {
                column,
                new_value: true,
                ..
            } if column == "id"
        )));
    }

    #[test]
    fn column_order_change_emits_warning() {
        let old = table(
            "t",
            vec![
                col("a", DataType::Text, false),
                col("b", DataType::Text, false),
            ],
        );
        let new = table(
            "t",
            vec![
                col("b", DataType::Text, false),
                col("a", DataType::Text, false),
            ],
        );
        let diff = diff_table("t", &old, &new);
        assert!(
            diff.warnings
                .iter()
                .any(|w| matches!(w, DiffWarning::ColumnOrderChanged { .. }))
        );
    }

    // ============================================================
    // Index diff
    // ============================================================

    #[test]
    fn index_added_detected() {
        let old = table("t", vec![col("a", DataType::Text, false)]);
        let mut new = table("t", vec![col("a", DataType::Text, false)]);
        new.indexes.push(IndexSnapshot {
            name: "idx_a".into(),
            columns: vec!["a".into()],
            unique: false,
            index_type: IndexType::BTree,
            condition: None,
        });
        let diff = diff_table("t", &old, &new);
        assert!(
            diff.operations
                .iter()
                .any(|op| matches!(op, Operation::CreateIndex(ci) if ci.name == "idx_a"))
        );
    }

    #[test]
    fn index_dropped_detected() {
        let mut old = table("t", vec![col("a", DataType::Text, false)]);
        old.indexes.push(IndexSnapshot {
            name: "idx_a".into(),
            columns: vec!["a".into()],
            unique: false,
            index_type: IndexType::BTree,
            condition: None,
        });
        let new = table("t", vec![col("a", DataType::Text, false)]);
        let diff = diff_table("t", &old, &new);
        assert!(
            diff.operations
                .iter()
                .any(|op| matches!(op, Operation::DropIndex(di) if di.name == "idx_a"))
        );
    }

    // ============================================================
    // Foreign key diff
    // ============================================================

    #[test]
    fn fk_added_detected() {
        let old = table("t", vec![col("a", DataType::Bigint, false)]);
        let mut new = table("t", vec![col("a", DataType::Bigint, false)]);
        new.foreign_keys.push(ForeignKeySnapshot {
            name: Some("fk_a".into()),
            columns: vec!["a".into()],
            references_table: "other".into(),
            references_columns: vec!["id".into()],
            on_delete: Some(ForeignKeyAction::Cascade),
            on_update: None,
        });
        let diff = diff_table("t", &old, &new);
        assert!(diff.operations.iter().any(
            |op| matches!(op, Operation::AddForeignKey(fk) if fk.name == Some("fk_a".into()))
        ));
    }

    #[test]
    fn fk_dropped_detected() {
        let mut old = table("t", vec![col("a", DataType::Bigint, false)]);
        old.foreign_keys.push(ForeignKeySnapshot {
            name: Some("fk_a".into()),
            columns: vec!["a".into()],
            references_table: "other".into(),
            references_columns: vec!["id".into()],
            on_delete: None,
            on_update: None,
        });
        let new = table("t", vec![col("a", DataType::Bigint, false)]);
        let diff = diff_table("t", &old, &new);
        assert!(
            diff.operations
                .iter()
                .any(|op| matches!(op, Operation::DropForeignKey(fk) if fk.name == "fk_a"))
        );
    }

    // ============================================================
    // Reversibility
    // ============================================================

    #[test]
    fn reversible_diff() {
        let old = table("t", vec![pk_col("id", DataType::Bigint)]);
        let new = table(
            "t",
            vec![
                pk_col("id", DataType::Bigint),
                col("email", DataType::Text, true),
            ],
        );
        let diff = diff_table("t", &old, &new);
        assert!(diff.is_reversible());

        let reversed = diff.reverse().unwrap();
        assert_eq!(reversed.operations.len(), 1);
        assert!(matches!(
            &reversed.operations[0],
            Operation::DropColumn(dc)
                if dc.column == "email"
        ));
    }

    #[test]
    fn non_reversible_diff() {
        let old = table(
            "t",
            vec![
                pk_col("id", DataType::Bigint),
                col("email", DataType::Text, true),
            ],
        );
        let new = table("t", vec![pk_col("id", DataType::Bigint)]);
        let diff = diff_table("t", &old, &new);

        // DropColumn is not reversible (no column definition).
        assert!(!diff.is_reversible());
        assert_eq!(diff.non_reversible_operations().len(), 1);
        assert!(diff.reverse().is_none());
    }

    // ============================================================
    // to_sql convenience
    // ============================================================

    #[test]
    fn to_sql_produces_output() {
        use crate::migrations::SqliteDialect;

        let old = table("t", vec![pk_col("id", DataType::Bigint)]);
        let new = table(
            "t",
            vec![
                pk_col("id", DataType::Bigint),
                col("name", DataType::Text, false),
            ],
        );
        let diff = diff_table("t", &old, &new);
        let sqls = diff.to_sql(&SqliteDialect::new());
        assert_eq!(sqls.len(), 1);
        assert!(sqls[0].contains("ADD COLUMN"));
    }
}
