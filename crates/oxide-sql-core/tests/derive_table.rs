//! Tests for the `#[derive(Table)]` macro output.
//!
//! These tests verify that the derive macro generates correct:
//! - `{Struct}Table` struct implementing the `Table` trait
//! - `{Struct}Columns` module with column types
//! - Column types implementing the `Column` trait
//! - Column accessor methods on both table and row types

use oxide_sql_core::ast::DataType;
use oxide_sql_core::migrations::{
    CreateTableOp, DefaultValue, DuckDbDialect, MigrationDialect,
    PostgresDialect, SqliteDialect,
};
use oxide_sql_core::schema::{Column, Table, TableSchema};
use oxide_sql_derive::Table;

// =============================================================================
// Test: Basic struct with default table name (snake_case)
// =============================================================================

#[allow(dead_code)]
#[derive(Debug, Clone, Table)]
pub struct User {
    #[column(primary_key)]
    pub id: i64,
    pub name: String,
    #[column(nullable)]
    pub email: Option<String>,
}

#[test]
fn test_user_table_name() {
    assert_eq!(UserTable::NAME, "user");
    assert_eq!(UserTable::table_name(), "user");
}

#[test]
fn test_user_table_columns() {
    assert_eq!(UserTable::COLUMNS, &["id", "name", "email"]);
}

#[test]
fn test_user_table_primary_key() {
    assert_eq!(UserTable::PRIMARY_KEY, Some("id"));
}

#[test]
fn test_user_column_id_metadata() {
    assert_eq!(UserColumns::Id::NAME, "id");
    const { assert!(UserColumns::Id::PRIMARY_KEY) };
    const { assert!(!UserColumns::Id::NULLABLE) };
}

#[test]
fn test_user_column_name_metadata() {
    assert_eq!(UserColumns::Name::NAME, "name");
    const { assert!(!UserColumns::Name::PRIMARY_KEY) };
    const { assert!(!UserColumns::Name::NULLABLE) };
}

#[test]
fn test_user_column_email_metadata() {
    assert_eq!(UserColumns::Email::NAME, "email");
    const { assert!(!UserColumns::Email::PRIMARY_KEY) };
    const { assert!(UserColumns::Email::NULLABLE) };
}

#[test]
fn test_user_column_accessors_on_table() {
    let id_col = UserTable::id();
    let name_col = UserTable::name();
    let email_col = UserTable::email();

    assert_eq!(<UserColumns::Id as Column>::NAME, "id");
    assert_eq!(<UserColumns::Name as Column>::NAME, "name");
    assert_eq!(<UserColumns::Email as Column>::NAME, "email");

    let _: UserColumns::Id = id_col;
    let _: UserColumns::Name = name_col;
    let _: UserColumns::Email = email_col;
}

#[test]
fn test_user_column_accessors_on_struct() {
    let id_col = User::id();
    let name_col = User::name();
    let email_col = User::email();

    let _: UserColumns::Id = id_col;
    let _: UserColumns::Name = name_col;
    let _: UserColumns::Email = email_col;
}

#[test]
fn test_user_table_method_on_struct() {
    let table = User::table();
    let _: UserTable = table;
}

// =============================================================================
// Test: Custom table name with #[table(name = "...")]
// =============================================================================

#[allow(dead_code)]
#[derive(Debug, Clone, Table)]
#[table(name = "products")]
pub struct Product {
    #[column(primary_key)]
    pub id: i64,
    pub title: String,
    pub price_cents: i64,
}

#[test]
fn test_custom_table_name() {
    assert_eq!(ProductTable::NAME, "products");
    assert_eq!(ProductTable::table_name(), "products");
}

#[test]
fn test_product_columns() {
    assert_eq!(ProductTable::COLUMNS, &["id", "title", "price_cents"]);
}

// =============================================================================
// Test: Custom column name with #[column(name = "...")]
// =============================================================================

#[allow(dead_code)]
#[derive(Debug, Clone, Table)]
#[table(name = "orders")]
pub struct Order {
    #[column(primary_key)]
    pub id: i64,
    #[column(name = "customer_id")]
    pub customer: i64,
    #[column(name = "total_amount")]
    pub total: i64,
}

#[test]
fn test_custom_column_names() {
    assert_eq!(OrderTable::COLUMNS, &["id", "customer_id", "total_amount"]);
    assert_eq!(OrderColumns::Id::NAME, "id");
    assert_eq!(OrderColumns::Customer::NAME, "customer_id");
    assert_eq!(OrderColumns::Total::NAME, "total_amount");
}

// =============================================================================
// Test: Table without primary key
// =============================================================================

#[allow(dead_code)]
#[derive(Debug, Clone, Table)]
#[table(name = "log_entries")]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

#[test]
fn test_table_without_primary_key() {
    assert_eq!(LogEntryTable::PRIMARY_KEY, None);
}

#[test]
fn test_log_entry_all_columns_not_primary() {
    const { assert!(!LogEntryColumns::Timestamp::PRIMARY_KEY) };
    const { assert!(!LogEntryColumns::Level::PRIMARY_KEY) };
    const { assert!(!LogEntryColumns::Message::PRIMARY_KEY) };
}

// =============================================================================
// Test: All columns nullable
// =============================================================================

#[allow(dead_code)]
#[derive(Debug, Clone, Table)]
pub struct NullableRecord {
    #[column(nullable)]
    pub field_a: Option<String>,
    #[column(nullable)]
    pub field_b: Option<i64>,
    #[column(nullable)]
    pub field_c: Option<f64>,
}

#[test]
fn test_all_nullable_columns() {
    const { assert!(NullableRecordColumns::FieldA::NULLABLE) };
    const { assert!(NullableRecordColumns::FieldB::NULLABLE) };
    const { assert!(NullableRecordColumns::FieldC::NULLABLE) };
}

// =============================================================================
// Test: Column type associations
// =============================================================================

#[allow(dead_code)]
#[derive(Debug, Clone, Table)]
pub struct TypedFields {
    #[column(primary_key)]
    pub id: i64,
    pub count: i32,
    pub ratio: f64,
    pub flag: bool,
    pub label: String,
}

#[test]
fn test_column_type_associations() {
    fn assert_column_type<C: Column<Type = T>, T>() {}

    assert_column_type::<TypedFieldsColumns::Id, i64>();
    assert_column_type::<TypedFieldsColumns::Count, i32>();
    assert_column_type::<TypedFieldsColumns::Ratio, f64>();
    assert_column_type::<TypedFieldsColumns::Flag, bool>();
    assert_column_type::<TypedFieldsColumns::Label, String>();
}

#[test]
fn test_column_table_association() {
    fn assert_column_table<C: Column<Table = T>, T: Table>() {}

    assert_column_table::<TypedFieldsColumns::Id, TypedFieldsTable>();
    assert_column_table::<TypedFieldsColumns::Count, TypedFieldsTable>();
    assert_column_table::<TypedFieldsColumns::Ratio, TypedFieldsTable>();
    assert_column_table::<TypedFieldsColumns::Flag, TypedFieldsTable>();
    assert_column_table::<TypedFieldsColumns::Label, TypedFieldsTable>();
}

// =============================================================================
// Test: Table::Row association
// =============================================================================

#[test]
fn test_table_row_association() {
    fn assert_table_row<T: Table<Row = R>, R>() {}

    assert_table_row::<UserTable, User>();
    assert_table_row::<ProductTable, Product>();
    assert_table_row::<OrderTable, Order>();
    assert_table_row::<LogEntryTable, LogEntry>();
}

// =============================================================================
// Test: Snake_case to PascalCase conversion for column types
// =============================================================================

#[allow(dead_code)]
#[derive(Debug, Clone, Table)]
pub struct SnakeCaseFields {
    #[column(primary_key)]
    pub user_id: i64,
    pub first_name: String,
    pub last_name: String,
    pub email_address: String,
}

#[test]
fn test_snake_case_to_pascal_case_conversion() {
    let _: SnakeCaseFieldsColumns::UserId = SnakeCaseFields::user_id();
    let _: SnakeCaseFieldsColumns::FirstName = SnakeCaseFields::first_name();
    let _: SnakeCaseFieldsColumns::LastName = SnakeCaseFields::last_name();
    let _: SnakeCaseFieldsColumns::EmailAddress = SnakeCaseFields::email_address();
}

#[test]
fn test_column_names_preserve_snake_case() {
    assert_eq!(SnakeCaseFieldsColumns::UserId::NAME, "user_id");
    assert_eq!(SnakeCaseFieldsColumns::FirstName::NAME, "first_name");
    assert_eq!(SnakeCaseFieldsColumns::LastName::NAME, "last_name");
    assert_eq!(SnakeCaseFieldsColumns::EmailAddress::NAME, "email_address");
}

// =============================================================================
// Test: Multiple primary key attributes (only first should be primary)
// =============================================================================

#[allow(dead_code)]
#[derive(Debug, Clone, Table)]
pub struct MultiPrimaryKey {
    #[column(primary_key)]
    pub id: i64,
    #[column(primary_key)]
    pub other_id: i64,
    pub data: String,
}

#[test]
fn test_multiple_primary_key_columns() {
    const { assert!(MultiPrimaryKeyColumns::Id::PRIMARY_KEY) };
    const { assert!(MultiPrimaryKeyColumns::OtherId::PRIMARY_KEY) };
    assert_eq!(MultiPrimaryKeyTable::PRIMARY_KEY, Some("id"));
}

// =============================================================================
// Test: Combined attributes
// =============================================================================

#[allow(dead_code)]
#[derive(Debug, Clone, Table)]
#[table(name = "audit_logs")]
pub struct AuditLog {
    #[column(primary_key)]
    pub id: i64,
    #[column(name = "user_ref", nullable)]
    pub user_id: Option<i64>,
    pub action: String,
    #[column(nullable)]
    pub details: Option<String>,
}

#[test]
fn test_combined_attributes() {
    assert_eq!(AuditLogTable::NAME, "audit_logs");
    assert_eq!(AuditLogTable::PRIMARY_KEY, Some("id"));

    assert_eq!(AuditLogColumns::UserId::NAME, "user_ref");
    const { assert!(AuditLogColumns::UserId::NULLABLE) };
    const { assert!(!AuditLogColumns::UserId::PRIMARY_KEY) };

    assert_eq!(AuditLogColumns::Details::NAME, "details");
    const { assert!(AuditLogColumns::Details::NULLABLE) };
}

// =============================================================================
// Test: Generated types are Copy and Clone
// =============================================================================

#[test]
fn test_table_struct_is_copy_clone() {
    fn assert_copy_clone<T: Copy + Clone>() {}

    assert_copy_clone::<UserTable>();
    assert_copy_clone::<ProductTable>();
    assert_copy_clone::<OrderTable>();
}

#[test]
fn test_column_types_are_copy_clone() {
    fn assert_copy_clone<T: Copy + Clone>() {}

    assert_copy_clone::<UserColumns::Id>();
    assert_copy_clone::<UserColumns::Name>();
    assert_copy_clone::<UserColumns::Email>();
}

// =============================================================================
// Test: Column types are Debug
// =============================================================================

#[test]
fn test_column_types_are_debug() {
    fn assert_debug<T: std::fmt::Debug>() {}

    assert_debug::<UserColumns::Id>();
    assert_debug::<UserColumns::Name>();
    assert_debug::<UserColumns::Email>();
    assert_debug::<UserTable>();
}

// =============================================================================
// Test: TableSchema is generated for all derive(Table) structs
// =============================================================================

#[test]
fn test_table_schema_generated() {
    let schema = UserTable::SCHEMA;
    assert_eq!(schema.len(), 3);

    assert_eq!(schema[0].name, "id");
    assert_eq!(schema[0].rust_type, "i64");
    assert!(schema[0].primary_key);
    assert!(!schema[0].nullable);
    assert!(!schema[0].unique);
    assert!(!schema[0].autoincrement);
    assert!(schema[0].default_expr.is_none());

    assert_eq!(schema[1].name, "name");
    assert_eq!(schema[1].rust_type, "String");
    assert!(!schema[1].primary_key);
    assert!(!schema[1].nullable);

    assert_eq!(schema[2].name, "email");
    assert_eq!(schema[2].rust_type, "Option<String>");
    assert!(schema[2].nullable);
}

// =============================================================================
// Test: New column attributes (unique, autoincrement, default)
// =============================================================================

#[allow(dead_code)]
#[derive(Debug, Clone, Table)]
#[table(name = "users_full")]
pub struct UserFull {
    #[column(primary_key, autoincrement)]
    pub id: i64,
    #[column(unique)]
    pub name: String,
    #[column(nullable)]
    pub email: Option<String>,
    #[column(default = "TRUE")]
    pub active: bool,
    pub created_at: String,
}

#[test]
fn test_new_column_attrs_in_schema() {
    let schema = UserFullTable::SCHEMA;
    assert_eq!(schema.len(), 5);

    // id: primary_key + autoincrement
    assert!(schema[0].primary_key);
    assert!(schema[0].autoincrement);
    assert!(!schema[0].unique);

    // name: unique
    assert!(schema[1].unique);
    assert!(!schema[1].primary_key);

    // email: nullable
    assert!(schema[2].nullable);

    // active: default
    assert_eq!(schema[3].default_expr, Some("TRUE"));

    // created_at: no special attrs
    assert!(!schema[4].primary_key);
    assert!(!schema[4].nullable);
    assert!(!schema[4].unique);
    assert!(!schema[4].autoincrement);
    assert!(schema[4].default_expr.is_none());
}

// =============================================================================
// Test: CreateTableOp::from_table with SQLite dialect
// =============================================================================

#[test]
fn test_from_table_sqlite() {
    let dialect = SqliteDialect::new();
    let op = CreateTableOp::from_table::<UserFullTable>(&dialect);

    assert_eq!(op.name, "users_full");
    assert_eq!(op.columns.len(), 5);
    assert!(!op.if_not_exists);

    let id = &op.columns[0];
    assert_eq!(id.name, "id");
    assert_eq!(id.data_type, DataType::Bigint);
    assert!(id.primary_key);
    assert!(id.autoincrement);
    assert!(!id.nullable);

    let name = &op.columns[1];
    assert_eq!(name.data_type, DataType::Text);
    assert!(name.unique);

    let email = &op.columns[2];
    assert!(email.nullable);
    assert_eq!(email.data_type, DataType::Text);

    let active = &op.columns[3];
    assert_eq!(active.data_type, DataType::Integer);
    assert_eq!(
        active.default,
        Some(DefaultValue::Expression("TRUE".into()))
    );

    let created_at = &op.columns[4];
    assert_eq!(created_at.data_type, DataType::Text);
}

// =============================================================================
// Test: CreateTableOp::from_table with PostgreSQL dialect
// =============================================================================

#[test]
fn test_from_table_postgres() {
    let dialect = PostgresDialect::new();
    let op = CreateTableOp::from_table::<UserFullTable>(&dialect);

    assert_eq!(op.name, "users_full");

    let id = &op.columns[0];
    assert_eq!(id.data_type, DataType::Bigint);

    let name = &op.columns[1];
    assert_eq!(name.data_type, DataType::Varchar(Some(255)));

    let email = &op.columns[2];
    assert_eq!(email.data_type, DataType::Varchar(Some(255)));

    let active = &op.columns[3];
    assert_eq!(active.data_type, DataType::Boolean);

    let created_at = &op.columns[4];
    assert_eq!(created_at.data_type, DataType::Varchar(Some(255)));
}

// =============================================================================
// Test: CreateTableOp::from_table with DuckDB dialect
// =============================================================================

#[test]
fn test_from_table_duckdb() {
    let dialect = DuckDbDialect::new();
    let op = CreateTableOp::from_table::<UserFullTable>(&dialect);

    assert_eq!(op.name, "users_full");

    let name = &op.columns[1];
    assert_eq!(name.data_type, DataType::Varchar(None));

    let active = &op.columns[3];
    assert_eq!(active.data_type, DataType::Boolean);
}

// =============================================================================
// Test: from_table_if_not_exists
// =============================================================================

#[test]
fn test_from_table_if_not_exists() {
    let dialect = SqliteDialect::new();
    let op =
        CreateTableOp::from_table_if_not_exists::<UserFullTable>(
            &dialect,
        );
    assert!(op.if_not_exists);
    assert_eq!(op.name, "users_full");
}

// =============================================================================
// Test: from_table roundtrip to SQL
// =============================================================================

#[test]
fn test_from_table_roundtrip_sql() {
    let dialect = SqliteDialect::new();
    let op = CreateTableOp::from_table::<UserFullTable>(&dialect);
    let sql = dialect.create_table(&op);
    assert!(sql.contains("CREATE TABLE"));
    assert!(sql.contains("\"users_full\""));
    assert!(sql.contains("\"id\""));
    assert!(sql.contains("\"name\""));
    assert!(sql.contains("AUTOINCREMENT"));
    assert!(sql.contains("UNIQUE"));
    assert!(sql.contains("DEFAULT TRUE"));
}

// =============================================================================
// Test: Option<T> is stripped for type mapping
// =============================================================================

#[test]
fn test_option_stripped_for_type_mapping() {
    let dialect = PostgresDialect::new();
    let op = CreateTableOp::from_table::<UserFullTable>(&dialect);

    // email is Option<String>, should map to VARCHAR(255) not Text
    let email = &op.columns[2];
    assert_eq!(email.data_type, DataType::Varchar(Some(255)));
    assert!(email.nullable);
}
