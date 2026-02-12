//! Integration tests for the schema diff engine.
//!
//! These tests build v1 and v2 snapshots from `#[derive(Table)]`
//! structs, diff them, and verify the resulting operations produce
//! valid SQL via `MigrationDialect::generate_sql()`.

use oxide_sql_core::migrations::{
    AlterColumnChange, AmbiguousChange, MigrationDialect, Operation, SqliteDialect, TableSnapshot,
    auto_diff_table,
};
use oxide_sql_derive::Table;

// =============================================================================
// V1: Initial schema
// =============================================================================

#[allow(dead_code)]
#[derive(Debug, Clone, Table)]
#[table(name = "articles")]
pub struct ArticleV1 {
    #[column(primary_key, autoincrement)]
    pub id: i64,
    pub title: String,
    pub body: String,
    #[column(default = "FALSE")]
    pub published: bool,
}

// =============================================================================
// V2: Evolved schema — added column, changed default, dropped column
// The body->summary change (same type) should be flagged as a
// possible rename rather than a drop+add.
// =============================================================================

#[allow(dead_code)]
#[derive(Debug, Clone, Table)]
#[table(name = "articles")]
pub struct ArticleV2 {
    #[column(primary_key, autoincrement)]
    pub id: i64,
    pub title: String,
    #[column(nullable)]
    pub summary: Option<String>,
    #[column(default = "TRUE")]
    pub published: bool,
}

// =============================================================================
// V3: Unambiguous changes — added column, type stays same, no rename
// =============================================================================

#[allow(dead_code)]
#[derive(Debug, Clone, Table)]
#[table(name = "articles")]
pub struct ArticleV3 {
    #[column(primary_key, autoincrement)]
    pub id: i64,
    pub title: String,
    pub body: String,
    #[column(default = "TRUE")]
    pub published: bool,
    #[column(nullable)]
    pub category: Option<String>,
}

#[test]
fn diff_v1_to_v2_detects_possible_rename() {
    let dialect = SqliteDialect::new();
    let v1 = TableSnapshot::from_table_schema::<ArticleV1Table>(&dialect);
    let diff = auto_diff_table::<ArticleV2Table>(&v1, &dialect);

    assert!(!diff.is_empty());

    // body -> summary is same type (Text), so should be flagged
    // as a possible rename.
    assert_eq!(diff.ambiguous.len(), 1);
    match &diff.ambiguous[0] {
        AmbiguousChange::PossibleRename {
            old_column,
            new_column,
            ..
        } => {
            assert_eq!(old_column, "body");
            assert_eq!(new_column, "summary");
        }
        other => panic!("Expected PossibleRename, got {other:?}"),
    }

    // The default change on published should still be detected.
    let has_set_default = diff.operations.iter().any(|op| {
        matches!(
            op,
            Operation::AlterColumn(alter)
                if alter.column == "published"
                    && matches!(
                        alter.change,
                        AlterColumnChange::SetDefault(_)
                    )
        )
    });
    assert!(
        has_set_default,
        "Should alter 'published' default from FALSE to TRUE"
    );

    // All operations should produce valid SQL.
    for op in &diff.operations {
        let sql = dialect.generate_sql(op);
        assert!(!sql.is_empty(), "Should produce SQL: {op:?}");
    }
}

#[test]
fn diff_v1_to_v3_produces_unambiguous_changes() {
    let dialect = SqliteDialect::new();
    let v1 = TableSnapshot::from_table_schema::<ArticleV1Table>(&dialect);
    let diff = auto_diff_table::<ArticleV3Table>(&v1, &dialect);

    assert!(!diff.is_empty());
    assert!(
        diff.ambiguous.is_empty(),
        "No renames — only additions and alterations"
    );

    // Should add category column.
    let has_add_category = diff.operations.iter().any(|op| {
        matches!(
            op,
            Operation::AddColumn(add)
                if add.column.name == "category"
        )
    });
    assert!(has_add_category, "Should add 'category' column");

    // Should alter published default.
    let has_set_default = diff.operations.iter().any(|op| {
        matches!(
            op,
            Operation::AlterColumn(alter)
                if alter.column == "published"
                    && matches!(
                        alter.change,
                        AlterColumnChange::SetDefault(_)
                    )
        )
    });
    assert!(has_set_default, "Should alter 'published' default");

    // All operations produce valid SQL.
    for op in &diff.operations {
        let sql = dialect.generate_sql(op);
        assert!(!sql.is_empty(), "Should produce SQL: {op:?}");
        assert!(
            sql.contains("articles") || sql.contains("\"articles\""),
            "SQL should reference 'articles': {sql}"
        );
    }
}

#[test]
fn diff_identical_schemas_is_empty() {
    let dialect = SqliteDialect::new();
    let v1 = TableSnapshot::from_table_schema::<ArticleV1Table>(&dialect);
    let diff = auto_diff_table::<ArticleV1Table>(&v1, &dialect);

    assert!(diff.is_empty(), "Identical schemas should produce no diff");
}
