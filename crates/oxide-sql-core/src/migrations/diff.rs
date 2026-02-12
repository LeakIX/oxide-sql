//! Schema diff engine for auto-migration generation.
//!
//! Compares an "old" (current DB) and "new" (desired from code)
//! [`SchemaSnapshot`] and produces a `Vec<Operation>` representing
//! the DDL changes needed to migrate from old to new.

use std::collections::BTreeSet;

use crate::schema::{RustTypeMapping, TableSchema};

use super::column_builder::ColumnDefinition;
use super::operation::{
    AddColumnOp, AlterColumnChange, AlterColumnOp, CreateTableOp, DropColumnOp, DropTableOp,
    Operation,
};
use super::snapshot::{ColumnSnapshot, SchemaSnapshot, TableSnapshot};

/// A change that cannot be automatically resolved and requires
/// user intervention.
#[derive(Debug, Clone, PartialEq)]
pub enum AmbiguousChange {
    /// One column was dropped and one was added with the same type,
    /// suggesting a possible rename.
    PossibleRename {
        /// Table containing the columns.
        table: String,
        /// The column that was dropped.
        old_column: String,
        /// The column that was added.
        new_column: String,
    },
    /// One table was dropped and one was added with the same column
    /// structure, suggesting a possible rename.
    PossibleTableRename {
        /// The table that was dropped.
        old_table: String,
        /// The table that was added.
        new_table: String,
    },
}

/// Result of comparing two schema snapshots.
#[derive(Debug, Clone, PartialEq)]
pub struct SchemaDiff {
    /// The migration operations to apply.
    pub operations: Vec<Operation>,
    /// Changes that may be renames and need user confirmation.
    pub ambiguous: Vec<AmbiguousChange>,
}

impl SchemaDiff {
    /// Returns `true` if there are no changes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty() && self.ambiguous.is_empty()
    }
}

/// Compares a single table's current and desired snapshots, producing
/// the operations needed to migrate.
fn diff_table(table_name: &str, old: &TableSnapshot, new: &TableSnapshot) -> SchemaDiff {
    let old_names: BTreeSet<&str> = old.columns.iter().map(|c| c.name.as_str()).collect();
    let new_names: BTreeSet<&str> = new.columns.iter().map(|c| c.name.as_str()).collect();

    let dropped: Vec<&str> = old_names.difference(&new_names).copied().collect();
    let added: Vec<&str> = new_names.difference(&old_names).copied().collect();
    let common: BTreeSet<&str> = old_names.intersection(&new_names).copied().collect();

    let mut operations = Vec::new();
    let mut ambiguous = Vec::new();

    // Check for possible renames: exactly 1 dropped + 1 added with
    // the same type.
    let mut rename_dropped = BTreeSet::new();
    let mut rename_added = BTreeSet::new();

    if dropped.len() == 1 && added.len() == 1 {
        let old_col = old.column(dropped[0]).unwrap();
        let new_col = new.column(added[0]).unwrap();
        if old_col.data_type == new_col.data_type {
            ambiguous.push(AmbiguousChange::PossibleRename {
                table: table_name.to_string(),
                old_column: dropped[0].to_string(),
                new_column: added[0].to_string(),
            });
            rename_dropped.insert(dropped[0]);
            rename_added.insert(added[0]);
        }
    }

    // AddColumn for truly new columns (not flagged as rename).
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

    // AlterColumn for changed properties on existing columns.
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

    // DropColumn for truly removed columns (not flagged as rename).
    for &name in &dropped {
        if rename_dropped.contains(name) {
            continue;
        }
        operations.push(Operation::DropColumn(DropColumnOp {
            table: table_name.to_string(),
            column: name.to_string(),
        }));
    }

    SchemaDiff {
        operations,
        ambiguous,
    }
}

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

    // Check for possible table renames.
    let mut rename_dropped = BTreeSet::new();
    let mut rename_added = BTreeSet::new();

    if dropped_tables.len() == 1 && added_tables.len() == 1 {
        let old_table = &current.tables[dropped_tables[0]];
        let new_table = &desired.tables[added_tables[0]];
        if tables_have_same_columns(old_table, new_table) {
            ambiguous.push(AmbiguousChange::PossibleTableRename {
                old_table: dropped_tables[0].to_string(),
                new_table: added_tables[0].to_string(),
            });
            rename_dropped.insert(dropped_tables[0]);
            rename_added.insert(added_tables[0]);
        }
    }

    // New tables -> CreateTable.
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

    // Existing tables -> diff columns.
    for &name in &common_tables {
        let old_table = &current.tables[name];
        let new_table = &desired.tables[name];
        let table_diff = diff_table(name, old_table, new_table);

        for op in table_diff.operations {
            match &op {
                Operation::AddColumn(_) => add_ops.push(op),
                Operation::AlterColumn(_) => alter_ops.push(op),
                Operation::DropColumn(_) => drop_col_ops.push(op),
                _ => add_ops.push(op),
            }
        }
        ambiguous.extend(table_diff.ambiguous);
    }

    // Dropped tables -> DropTable.
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
    use crate::migrations::column_builder::DefaultValue;

    // ================================================================
    // Helpers
    // ================================================================

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
        }
    }

    fn schema(tables: Vec<TableSnapshot>) -> SchemaSnapshot {
        let mut s = SchemaSnapshot::new();
        for t in tables {
            s.add_table(t);
        }
        s
    }

    // ================================================================
    // Tests
    // ================================================================

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
        match &diff.operations[0] {
            Operation::CreateTable(op) => {
                assert_eq!(op.name, "users");
            }
            other => panic!("Expected CreateTable, got {other:?}"),
        }
    }

    #[test]
    fn dropped_table_detected() {
        let current = schema(vec![table("users", vec![pk_col("id", DataType::Bigint)])]);
        let desired = schema(vec![]);
        let diff = auto_diff_schema(&current, &desired);

        assert_eq!(diff.operations.len(), 1);
        match &diff.operations[0] {
            Operation::DropTable(op) => {
                assert_eq!(op.name, "users");
            }
            other => panic!("Expected DropTable, got {other:?}"),
        }
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
        match &diff.operations[0] {
            Operation::AddColumn(op) => {
                assert_eq!(op.table, "users");
                assert_eq!(op.column.name, "email");
            }
            other => panic!("Expected AddColumn, got {other:?}"),
        }
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
        match &diff.operations[0] {
            Operation::DropColumn(op) => {
                assert_eq!(op.table, "users");
                assert_eq!(op.column, "email");
            }
            other => panic!("Expected DropColumn, got {other:?}"),
        }
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
        match &diff.operations[0] {
            Operation::AlterColumn(op) => {
                assert_eq!(op.column, "score");
                assert_eq!(op.change, AlterColumnChange::SetDataType(DataType::Bigint));
            }
            other => panic!("Expected AlterColumn, got {other:?}"),
        }
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
        match &diff.operations[0] {
            Operation::AlterColumn(op) => {
                assert_eq!(op.column, "email");
                assert_eq!(op.change, AlterColumnChange::SetNullable(true));
            }
            other => panic!("Expected AlterColumn, got {other:?}"),
        }
    }

    #[test]
    fn default_added() {
        let old = table("t", vec![col("active", DataType::Boolean, false)]);
        let mut new_col = col("active", DataType::Boolean, false);
        new_col.default = Some(DefaultValue::Expression("TRUE".into()));
        let new = table("t", vec![new_col]);
        let diff = diff_table("t", &old, &new);

        assert_eq!(diff.operations.len(), 1);
        match &diff.operations[0] {
            Operation::AlterColumn(op) => {
                assert_eq!(
                    op.change,
                    AlterColumnChange::SetDefault(DefaultValue::Expression("TRUE".into()))
                );
            }
            other => panic!("Expected AlterColumn, got {other:?}"),
        }
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
        match &diff.operations[0] {
            Operation::AlterColumn(op) => {
                assert_eq!(
                    op.change,
                    AlterColumnChange::SetDefault(DefaultValue::Integer(1))
                );
            }
            other => panic!("Expected AlterColumn, got {other:?}"),
        }
    }

    #[test]
    fn default_removed() {
        let mut old_col = col("active", DataType::Boolean, false);
        old_col.default = Some(DefaultValue::Expression("TRUE".into()));
        let old = table("t", vec![old_col]);
        let new = table("t", vec![col("active", DataType::Boolean, false)]);
        let diff = diff_table("t", &old, &new);

        assert_eq!(diff.operations.len(), 1);
        match &diff.operations[0] {
            Operation::AlterColumn(op) => {
                assert_eq!(op.change, AlterColumnChange::DropDefault);
            }
            other => panic!("Expected AlterColumn, got {other:?}"),
        }
    }

    #[test]
    fn ambiguous_rename_detected() {
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

        // Should NOT produce add/drop, but flag as ambiguous.
        assert!(diff.operations.is_empty());
        assert_eq!(diff.ambiguous.len(), 1);
        match &diff.ambiguous[0] {
            AmbiguousChange::PossibleRename {
                table,
                old_column,
                new_column,
            } => {
                assert_eq!(table, "users");
                assert_eq!(old_column, "name");
                assert_eq!(new_column, "full_name");
            }
            other => panic!("Expected PossibleRename, got {other:?}"),
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

        // Different types -> regular add+drop, no ambiguity.
        assert!(diff.ambiguous.is_empty());
        assert_eq!(diff.operations.len(), 2);
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

        // name: type change + nullable change = 2 alter ops
        // old_field dropped, new_field added (different types, no
        // rename ambiguity)
        assert!(diff.ambiguous.is_empty());
        // AddColumn(new_field) + AlterColumn(name type) +
        // AlterColumn(name nullable) + DropColumn(old_field) = 4
        assert_eq!(diff.operations.len(), 4);
    }

    #[test]
    fn operation_ordering_in_schema_diff() {
        // Create a scenario with create, alter, drop operations.
        // Use different column structures so table rename detection
        // does not fire. Use multiple column changes so column
        // rename detection does not fire either.
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
                    col("old_a", DataType::Text, false),
                    col("old_b", DataType::Integer, false),
                ],
            ),
        ]);
        let desired = schema(vec![
            table("to_create", vec![pk_col("id", DataType::Bigint)]),
            table(
                "to_alter",
                vec![
                    pk_col("id", DataType::Bigint),
                    col("new_a", DataType::Text, false),
                    col("new_b", DataType::Integer, false),
                ],
            ),
        ]);
        let diff = auto_diff_schema(&current, &desired);

        // Verify ordering: CreateTable first, then
        // AddColumn/AlterColumn, then DropColumn, then DropTable.
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
                        "AddColumn must come before DropColumn/DropTable"
                    );
                    saw_add = true;
                }
                Operation::DropColumn(_) => {
                    assert!(!saw_drop_table, "DropColumn must come before DropTable");
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

        // Should detect that "title" column was added.
        assert_eq!(diff.operations.len(), 1);
        match &diff.operations[0] {
            Operation::AddColumn(op) => {
                assert_eq!(op.column.name, "title");
                assert_eq!(op.column.data_type, DataType::Text);
            }
            other => panic!("Expected AddColumn, got {other:?}"),
        }
    }
}
