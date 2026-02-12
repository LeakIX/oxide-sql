//! # oxide-sql-core
//!
//! A type-safe SQL parser and builder with compile-time validation.
//!
//! This crate provides:
//! - A hand-written recursive descent parser with Pratt expression parsing
//! - Type-safe SQL builders using the typestate pattern
//! - Protection against SQL injection through parameterized queries
//! - A type-safe migrations system (Django-like)
//! - A schema diff engine for automatic migration generation
//!
//! ## Defining Tables with `#[derive(Table)]`
//!
//! The `#[derive(Table)]` macro (from [`oxide-sql-derive`]) turns a plain
//! struct into a full schema definition with compile-time checked column
//! names, types, and metadata.
//!
//! ```rust
//! # #![allow(clippy::needless_doctest_main)]
//! use oxide_sql_derive::Table;
//! use oxide_sql_core::schema::{Column, Table};
//!
//! #[derive(Table)]
//! #[table(name = "users")]
//! pub struct User {
//!     #[column(primary_key)]
//!     id: i64,
//!     name: String,
//!     #[column(nullable)]
//!     email: Option<String>,
//! }
//!
//! fn main() {
//! // The macro generates all of the following:
//! //
//! //   UserTable     – unit struct implementing the Table trait
//! //   UserColumns   – module with typed column structs (Id, Name, Email)
//! //   User::id()    – accessor returning UserColumns::Id
//! //   User::name()  – accessor returning UserColumns::Name
//! //   User::email() – accessor returning UserColumns::Email
//! //   UserTable::id(), UserTable::name(), ... (same accessors)
//!
//! // Table metadata
//! assert_eq!(UserTable::NAME, "users");
//! assert_eq!(UserTable::COLUMNS, &["id", "name", "email"]);
//! assert_eq!(UserTable::PRIMARY_KEY, Some("id"));
//!
//! // Column metadata
//! assert_eq!(UserColumns::Id::NAME, "id");
//! assert!(UserColumns::Id::PRIMARY_KEY);
//! assert!(!UserColumns::Id::NULLABLE);
//!
//! assert_eq!(UserColumns::Email::NAME, "email");
//! assert!(UserColumns::Email::NULLABLE);
//! }
//! ```
//!
//! ### Attributes
//!
//! | Attribute | Level | Effect |
//! |---|---|---|
//! | `#[table(name = "...")]` | struct | Sets the SQL table name (default: `snake_case` of struct name) |
//! | `#[column(primary_key)]` | field | Marks the column as the primary key |
//! | `#[column(nullable)]` | field | Marks the column as nullable |
//! | `#[column(name = "...")]` | field | Overrides the SQL column name (default: field name) |
//!
//! ### What the macro generates — under the hood
//!
//! Given the `User` struct above, `#[derive(Table)]` expands to roughly:
//!
//! ```rust
//! use oxide_sql_core::schema::{Column, Table, TypedColumn};
//!
//! pub struct User { id: i64, name: String, email: Option<String> }
//!
//! // 1. A table unit struct that implements the Table trait.
//! #[derive(Debug, Clone, Copy)]
//! pub struct UserTable;
//!
//! impl Table for UserTable {
//!     type Row = User;
//!     const NAME: &'static str = "users";
//!     const COLUMNS: &'static [&'static str] = &["id", "name", "email"];
//!     const PRIMARY_KEY: Option<&'static str> = Some("id");
//! }
//!
//! // 2. A columns module with one zero-sized struct per field.
//! //    Each struct implements Column (with the table, Rust type,
//! //    name, nullable, and primary_key metadata) and TypedColumn<T>.
//! #[allow(non_snake_case)]
//! mod UserColumns {
//!     use super::*;
//!
//!     #[derive(Debug, Clone, Copy)]
//!     pub struct Id;
//!     impl Column for Id {
//!         type Table = super::UserTable;
//!         type Type = i64;
//!         const NAME: &'static str = "id";
//!         const NULLABLE: bool = false;
//!         const PRIMARY_KEY: bool = true;
//!     }
//!     impl TypedColumn<i64> for Id {}
//!
//!     #[derive(Debug, Clone, Copy)]
//!     pub struct Name;
//!     impl Column for Name {
//!         type Table = super::UserTable;
//!         type Type = String;
//!         const NAME: &'static str = "name";
//!         const NULLABLE: bool = false;
//!         const PRIMARY_KEY: bool = false;
//!     }
//!     impl TypedColumn<String> for Name {}
//!
//!     #[derive(Debug, Clone, Copy)]
//!     pub struct Email;
//!     impl Column for Email {
//!         type Table = super::UserTable;
//!         type Type = Option<String>;
//!         const NAME: &'static str = "email";
//!         const NULLABLE: bool = true;
//!         const PRIMARY_KEY: bool = false;
//!     }
//!     impl TypedColumn<Option<String>> for Email {}
//! }
//!
//! // 3. Const accessor methods on both UserTable and User so you
//! //    can write `User::id()` or `UserTable::id()` to obtain
//! //    the zero-sized column type for use in query builders.
//! impl UserTable {
//!     pub const fn id() -> UserColumns::Id { UserColumns::Id }
//!     pub const fn name() -> UserColumns::Name { UserColumns::Name }
//!     pub const fn email() -> UserColumns::Email { UserColumns::Email }
//! }
//! // (User also gets the same accessors and a `table()` method)
//! # fn main() {}
//! ```
//!
//! Because every column is a distinct zero-sized type that carries its
//! table association via `Column::Table`, the typed query builders can
//! verify at compile time that you only reference columns that belong to
//! the table you are querying.
//!
//! ## Type-Safe Queries
//!
//! The typed builders — [`Select`], [`Insert`], [`Update`], [`Delete`] —
//! use the typestate pattern so that incomplete queries (missing columns,
//! missing table, missing SET values) simply do not compile.
//!
//! All examples below reuse the `User` / `UserTable` definition from
//! the section above.
//!
//! ### SELECT
//!
//! ```rust
//! # #![allow(clippy::needless_doctest_main)]
//! use oxide_sql_derive::Table;
//! use oxide_sql_core::builder::{Select, col};
//! use oxide_sql_core::schema::Table;
//!
//! #[derive(Table)]
//! #[table(name = "users")]
//! pub struct User {
//!     #[column(primary_key)]
//!     id: i64,
//!     name: String,
//!     #[column(nullable)]
//!     email: Option<String>,
//! }
//!
//! fn main() {
//! // SELECT all columns
//! let (sql, _params) = Select::<UserTable, _, _>::new()
//!     .select_all()
//!     .from_table()
//!     .build();
//! assert_eq!(sql, "SELECT id, name, email FROM users");
//!
//! // SELECT with WHERE, ORDER BY, LIMIT
//! let (sql, params) = Select::<UserTable, _, _>::new()
//!     .select_all()
//!     .from_table()
//!     .where_col(User::id(), col(User::id()).gt(100))
//!     .order_by(User::name(), true)
//!     .limit(10)
//!     .build();
//! assert_eq!(
//!     sql,
//!     "SELECT id, name, email FROM users \
//!      WHERE id > ? ORDER BY name LIMIT 10"
//! );
//! }
//! ```
//!
//! ### INSERT
//!
//! ```rust
//! # #![allow(clippy::needless_doctest_main)]
//! use oxide_sql_derive::Table;
//! use oxide_sql_core::builder::Insert;
//! use oxide_sql_core::schema::Table;
//!
//! #[derive(Table)]
//! #[table(name = "users")]
//! pub struct User {
//!     #[column(primary_key)]
//!     id: i64,
//!     name: String,
//!     #[column(nullable)]
//!     email: Option<String>,
//! }
//!
//! fn main() {
//! let (sql, params) = Insert::<UserTable, _>::new()
//!     .set(User::name(), "Alice")
//!     .set(User::email(), "alice@example.com")
//!     .build();
//! assert_eq!(sql, "INSERT INTO users (name, email) VALUES (?, ?)");
//! }
//! ```
//!
//! ### UPDATE
//!
//! ```rust
//! # #![allow(clippy::needless_doctest_main)]
//! use oxide_sql_derive::Table;
//! use oxide_sql_core::builder::{Update, col};
//! use oxide_sql_core::schema::Table;
//!
//! #[derive(Table)]
//! #[table(name = "users")]
//! pub struct User {
//!     #[column(primary_key)]
//!     id: i64,
//!     name: String,
//!     #[column(nullable)]
//!     email: Option<String>,
//! }
//!
//! fn main() {
//! let (sql, params) = Update::<UserTable, _>::new()
//!     .set(User::name(), "Bob")
//!     .where_col(User::id(), col(User::id()).eq(42))
//!     .build();
//! assert_eq!(sql, "UPDATE users SET name = ? WHERE id = ?");
//! }
//! ```
//!
//! ### DELETE
//!
//! ```rust
//! # #![allow(clippy::needless_doctest_main)]
//! use oxide_sql_derive::Table;
//! use oxide_sql_core::builder::{Delete, col};
//! use oxide_sql_core::schema::Table;
//!
//! #[derive(Table)]
//! #[table(name = "users")]
//! pub struct User {
//!     #[column(primary_key)]
//!     id: i64,
//!     name: String,
//!     #[column(nullable)]
//!     email: Option<String>,
//! }
//!
//! fn main() {
//! let (sql, params) = Delete::<UserTable>::new()
//!     .where_col(User::id(), col(User::id()).eq(1))
//!     .build();
//! assert_eq!(sql, "DELETE FROM users WHERE id = ?");
//! }
//! ```
//!
//! ## Dynamic SQL Building
//!
//! For string-based queries without compile-time validation, use `SelectDyn`,
//! `InsertDyn`, `UpdateDyn`, `DeleteDyn` with `dyn_col`:
//!
//! ```rust
//! use oxide_sql_core::builder::{SelectDyn, dyn_col};
//!
//! let (sql, params) = SelectDyn::new()
//!     .columns(&["id", "name"])
//!     .from("users")
//!     .where_clause(dyn_col("active").eq(true))
//!     .build();
//!
//! assert_eq!(sql, "SELECT id, name FROM users WHERE active = ?");
//! ```
//!
//! ## SQL Injection Prevention
//!
//! All values are automatically parameterized:
//!
//! ```rust
//! use oxide_sql_core::builder::{SelectDyn, dyn_col};
//!
//! let user_input = "'; DROP TABLE users; --";
//! let (sql, params) = SelectDyn::new()
//!     .columns(&["id"])
//!     .from("users")
//!     .where_clause(dyn_col("name").eq(user_input))
//!     .build();
//!
//! // sql = "SELECT id FROM users WHERE name = ?"
//! // The malicious input is safely parameterized
//! assert_eq!(sql, "SELECT id FROM users WHERE name = ?");
//! ```
//!
//! ## Type-Safe Migrations
//!
//! The migrations module provides a Django-like system for evolving
//! database schemas. Each migration is a struct implementing the
//! [`Migration`] trait with `up()` (apply) and `down()` (rollback)
//! methods that return a list of [`Operation`]s.
//!
//! ### Defining a migration
//!
//! ```rust
//! use oxide_sql_core::migrations::{
//!     Migration, Operation, CreateTableBuilder,
//!     bigint, varchar, timestamp,
//! };
//!
//! pub struct Migration0001;
//!
//! impl Migration for Migration0001 {
//!     const ID: &'static str = "0001_create_users";
//!
//!     fn up() -> Vec<Operation> {
//!         vec![
//!             CreateTableBuilder::new()
//!                 .name("users")
//!                 .column(bigint("id").primary_key().autoincrement().build())
//!                 .column(varchar("username", 255).not_null().unique().build())
//!                 .column(
//!                     timestamp("created_at")
//!                         .not_null()
//!                         .default_expr("CURRENT_TIMESTAMP")
//!                         .build(),
//!                 )
//!                 .build()
//!                 .into(),
//!         ]
//!     }
//!
//!     fn down() -> Vec<Operation> {
//!         vec![Operation::drop_table("users")]
//!     }
//! }
//! ```
//!
//! ### Column helpers
//!
//! Shorthand functions create a [`ColumnBuilder`](migrations::ColumnBuilder)
//! for each SQL type. Chain constraints then call `.build()`:
//!
//! | Function | SQL type |
//! |---|---|
//! | [`bigint`](migrations::bigint), [`integer`](migrations::integer), [`smallint`](migrations::smallint) | `BIGINT`, `INTEGER`, `SMALLINT` |
//! | [`varchar`](migrations::varchar), [`text`](migrations::text), [`char`](migrations::char) | `VARCHAR(n)`, `TEXT`, `CHAR(n)` |
//! | [`boolean`](migrations::boolean) | `BOOLEAN` |
//! | [`timestamp`](migrations::timestamp), [`datetime`](migrations::datetime), [`date`](migrations::date), [`time`](migrations::time) | date/time types |
//! | [`decimal`](migrations::decimal), [`numeric`](migrations::numeric), [`real`](migrations::real), [`double`](migrations::double) | floating-point/decimal types |
//! | [`blob`](migrations::blob), [`binary`](migrations::binary), [`varbinary`](migrations::varbinary) | binary types |
//!
//! ```rust
//! use oxide_sql_core::migrations::{bigint, varchar, boolean, timestamp};
//!
//! // Primary key with auto-increment
//! let id = bigint("id").primary_key().autoincrement().build();
//!
//! // NOT NULL + UNIQUE
//! let email = varchar("email", 255).not_null().unique().build();
//!
//! // Default value
//! let active = boolean("active").not_null().default_bool(true).build();
//!
//! // Default expression
//! let ts = timestamp("created_at")
//!     .not_null()
//!     .default_expr("CURRENT_TIMESTAMP")
//!     .build();
//! ```
//!
//! ### Operations
//!
//! [`Operation`] covers all DDL changes. Beyond `CreateTable`, the most
//! common factory methods are:
//!
//! ```rust
//! use oxide_sql_core::migrations::{Operation, varchar};
//!
//! // Drop a table
//! let _ = Operation::drop_table("users");
//!
//! // Rename a table
//! let _ = Operation::rename_table("users", "accounts");
//!
//! // Add a column to an existing table
//! let _ = Operation::add_column(
//!     "users",
//!     varchar("bio", 1000).nullable().build(),
//! );
//!
//! // Drop a column
//! let _ = Operation::drop_column("users", "bio");
//!
//! // Rename a column
//! let _ = Operation::rename_column("users", "name", "full_name");
//!
//! // Raw SQL (with optional reverse for rollback)
//! let _ = Operation::run_sql_reversible(
//!     "CREATE VIEW active_users AS SELECT * FROM users WHERE active",
//!     "DROP VIEW active_users",
//! );
//! ```
//!
//! ### Dependencies between migrations
//!
//! Migrations can declare dependencies via `DEPENDENCIES`. The runner
//! topologically sorts them so dependees always run first:
//!
//! ```rust
//! use oxide_sql_core::migrations::{
//!     Migration, Operation, CreateTableBuilder, bigint, varchar,
//! };
//!
//! pub struct Migration0001;
//! impl Migration for Migration0001 {
//!     const ID: &'static str = "0001_create_users";
//!     fn up() -> Vec<Operation> {
//!         vec![
//!             CreateTableBuilder::new()
//!                 .name("users")
//!                 .column(bigint("id").primary_key().build())
//!                 .column(varchar("name", 255).not_null().build())
//!                 .build()
//!                 .into(),
//!         ]
//!     }
//!     fn down() -> Vec<Operation> {
//!         vec![Operation::drop_table("users")]
//!     }
//! }
//!
//! pub struct Migration0002;
//! impl Migration for Migration0002 {
//!     const ID: &'static str = "0002_create_posts";
//!     // This migration depends on 0001
//!     const DEPENDENCIES: &'static [&'static str] = &["0001_create_users"];
//!     fn up() -> Vec<Operation> {
//!         vec![
//!             CreateTableBuilder::new()
//!                 .name("posts")
//!                 .column(bigint("id").primary_key().build())
//!                 .column(bigint("user_id").not_null().build())
//!                 .column(varchar("title", 255).not_null().build())
//!                 .build()
//!                 .into(),
//!         ]
//!     }
//!     fn down() -> Vec<Operation> {
//!         vec![Operation::drop_table("posts")]
//!     }
//! }
//! ```
//!
//! ### Running migrations — under the hood
//!
//! [`MigrationRunner`](migrations::MigrationRunner) registers migrations,
//! resolves dependencies, and generates dialect-specific SQL.
//! [`MigrationState`](migrations::MigrationState) tracks which migrations
//! have already been applied (backed by the `_oxide_migrations` table in
//! your database).
//!
//! ```rust
//! use oxide_sql_core::migrations::{
//!     Migration, MigrationRunner, MigrationState,
//!     SqliteDialect, Operation, CreateTableBuilder,
//!     bigint, varchar,
//! };
//!
//! pub struct Mig0001;
//! impl Migration for Mig0001 {
//!     const ID: &'static str = "0001_create_users";
//!     fn up() -> Vec<Operation> {
//!         vec![
//!             CreateTableBuilder::new()
//!                 .name("users")
//!                 .column(bigint("id").primary_key().build())
//!                 .column(varchar("name", 255).not_null().build())
//!                 .build()
//!                 .into(),
//!         ]
//!     }
//!     fn down() -> Vec<Operation> {
//!         vec![Operation::drop_table("users")]
//!     }
//! }
//!
//! // 1. Create a runner with a dialect (SQLite, Postgres, DuckDB)
//! let mut runner = MigrationRunner::new(SqliteDialect::new());
//!
//! // 2. Register all migrations
//! runner.register::<Mig0001>();
//!
//! // 3. Validate dependencies (detects cycles / missing deps)
//! runner.validate().expect("dependency graph is valid");
//!
//! // 4. Build state from the database (here: empty = fresh DB)
//! let state = MigrationState::new();
//!
//! // 5. Generate SQL for pending migrations
//! let pending_sql = runner.sql_for_pending(&state).unwrap();
//! for (id, statements) in &pending_sql {
//!     for sql in statements {
//!         // execute `sql` against your database connection
//!         assert!(!sql.is_empty());
//!     }
//!     // then mark applied: state.mark_applied(id);
//! }
//!
//! // 6. Rollback the last N migrations
//! let mut applied_state = MigrationState::from_applied(
//!     vec!["0001_create_users".to_string()],
//! );
//! let rollback_sql = runner.sql_for_rollback(&applied_state, 1).unwrap();
//! for (id, statements) in &rollback_sql {
//!     for sql in statements {
//!         assert!(!sql.is_empty());
//!     }
//! }
//! ```
//!
//! ### Dialects
//!
//! The same migration operations produce different SQL depending on the
//! dialect:
//!
//! | Dialect | Auto-increment strategy | Notes |
//! |---|---|---|
//! | [`SqliteDialect`](migrations::SqliteDialect) | `AUTOINCREMENT` keyword | Limited `ALTER TABLE`; dates stored as `TEXT` |
//! | [`PostgresDialect`](migrations::PostgresDialect) | `SERIAL` / `BIGSERIAL` types | Full `ALTER COLUMN` support |
//! | [`DuckDbDialect`](migrations::DuckDbDialect) | `CREATE SEQUENCE` + `DEFAULT nextval(...)` | Sequence name: `seq_<table>_<column>` |
//!
//! ## Schema Diff Engine — Automatic Migration Generation
//!
//! The schema diff engine compares two schema snapshots and produces
//! `Vec<Operation>`, enabling Django-like `makemigrations`. Instead
//! of writing migration operations by hand, you can diff the current
//! database state against your `#[derive(Table)]` structs to
//! automatically detect what changed.
//!
//! ### Core types
//!
//! | Type | Purpose |
//! |---|---|
//! | [`TableSnapshot`] | Dialect-resolved snapshot of a single table (columns with `DataType`, not Rust type strings) |
//! | [`ColumnSnapshot`] | A single column's resolved name, type, nullable, primary key, unique, autoincrement, and default |
//! | [`SchemaSnapshot`] | A collection of `TableSnapshot`s keyed by table name |
//! | [`SchemaDiff`] | Result of a diff: `operations: Vec<Operation>` + `ambiguous: Vec<AmbiguousChange>` |
//! | [`AmbiguousChange`] | Changes that cannot be auto-resolved (possible column/table renames) |
//!
//! ### Building snapshots from `#[derive(Table)]` structs
//!
//! [`TableSnapshot::from_table_schema`] resolves Rust types to SQL
//! `DataType` using the dialect's [`RustTypeMapping`], producing a
//! dialect-aware snapshot ready for comparison:
//!
//! ```rust
//! # #![allow(clippy::needless_doctest_main)]
//! use oxide_sql_derive::Table;
//! use oxide_sql_core::migrations::{TableSnapshot, SqliteDialect};
//! use oxide_sql_core::schema::Table;
//!
//! #[derive(Table)]
//! #[table(name = "articles")]
//! pub struct Article {
//!     #[column(primary_key, autoincrement)]
//!     pub id: i64,
//!     pub title: String,
//!     #[column(nullable)]
//!     pub body: Option<String>,
//!     #[column(default = "FALSE")]
//!     pub published: bool,
//! }
//!
//! fn main() {
//! let dialect = SqliteDialect::new();
//! let snapshot = TableSnapshot::from_table_schema::<ArticleTable>(&dialect);
//!
//! assert_eq!(snapshot.name, "articles");
//! assert_eq!(snapshot.columns.len(), 4);
//! assert!(snapshot.column("id").unwrap().primary_key);
//! assert!(snapshot.column("body").unwrap().nullable);
//! }
//! ```
//!
//! ### Diffing a single table
//!
//! [`auto_diff_table`] compares a table's current snapshot against the
//! desired schema from a `#[derive(Table)]` struct:
//!
//! ```rust
//! # #![allow(clippy::needless_doctest_main)]
//! use oxide_sql_derive::Table;
//! use oxide_sql_core::migrations::{
//!     TableSnapshot, SqliteDialect, auto_diff_table, Operation,
//! };
//! use oxide_sql_core::schema::Table;
//! use oxide_sql_core::ast::DataType;
//!
//! #[derive(Table)]
//! #[table(name = "articles")]
//! pub struct ArticleV2 {
//!     #[column(primary_key, autoincrement)]
//!     pub id: i64,
//!     pub title: String,
//!     #[column(nullable)]
//!     pub body: Option<String>,
//!     #[column(default = "FALSE")]
//!     pub published: bool,
//!     #[column(nullable)]
//!     pub category: Option<String>,
//! }
//!
//! fn main() {
//! let dialect = SqliteDialect::new();
//!
//! // Simulate "current" DB state: articles without the category column.
//! let current = TableSnapshot {
//!     name: "articles".to_string(),
//!     columns: vec![
//!         oxide_sql_core::migrations::ColumnSnapshot {
//!             name: "id".into(),
//!             data_type: DataType::Bigint,
//!             nullable: false,
//!             primary_key: true,
//!             unique: false,
//!             autoincrement: true,
//!             default: None,
//!         },
//!         oxide_sql_core::migrations::ColumnSnapshot {
//!             name: "title".into(),
//!             data_type: DataType::Text,
//!             nullable: false,
//!             primary_key: false,
//!             unique: false,
//!             autoincrement: false,
//!             default: None,
//!         },
//!         oxide_sql_core::migrations::ColumnSnapshot {
//!             name: "body".into(),
//!             data_type: DataType::Text,
//!             nullable: true,
//!             primary_key: false,
//!             unique: false,
//!             autoincrement: false,
//!             default: None,
//!         },
//!         oxide_sql_core::migrations::ColumnSnapshot {
//!             name: "published".into(),
//!             data_type: DataType::Integer,
//!             nullable: false,
//!             primary_key: false,
//!             unique: false,
//!             autoincrement: false,
//!             default: Some(oxide_sql_core::migrations::DefaultValue::Expression(
//!                 "FALSE".into(),
//!             )),
//!         },
//!     ],
//! };
//!
//! let diff = auto_diff_table::<ArticleV2Table>(&current, &dialect);
//!
//! // Detects that "category" was added.
//! assert!(diff.operations.iter().any(|op| matches!(
//!     op,
//!     Operation::AddColumn(add) if add.column.name == "category"
//! )));
//! }
//! ```
//!
//! ### Diffing entire schemas
//!
//! [`auto_diff_schema`] compares two [`SchemaSnapshot`]s and detects
//! new tables, dropped tables, and per-table column changes:
//!
//! ```rust
//! use oxide_sql_core::migrations::{
//!     SchemaSnapshot, TableSnapshot, ColumnSnapshot, Operation,
//!     auto_diff_schema,
//! };
//! use oxide_sql_core::ast::DataType;
//!
//! let mut current = SchemaSnapshot::new();
//! let mut desired = SchemaSnapshot::new();
//!
//! // "current" has a "users" table; "desired" adds a "posts" table.
//! current.add_table(TableSnapshot {
//!     name: "users".into(),
//!     columns: vec![ColumnSnapshot {
//!         name: "id".into(),
//!         data_type: DataType::Bigint,
//!         nullable: false,
//!         primary_key: true,
//!         unique: false,
//!         autoincrement: true,
//!         default: None,
//!     }],
//! });
//! desired.add_table(TableSnapshot {
//!     name: "users".into(),
//!     columns: vec![ColumnSnapshot {
//!         name: "id".into(),
//!         data_type: DataType::Bigint,
//!         nullable: false,
//!         primary_key: true,
//!         unique: false,
//!         autoincrement: true,
//!         default: None,
//!     }],
//! });
//! desired.add_table(TableSnapshot {
//!     name: "posts".into(),
//!     columns: vec![ColumnSnapshot {
//!         name: "id".into(),
//!         data_type: DataType::Bigint,
//!         nullable: false,
//!         primary_key: true,
//!         unique: false,
//!         autoincrement: true,
//!         default: None,
//!     }],
//! });
//!
//! let diff = auto_diff_schema(&current, &desired);
//! assert!(diff.operations.iter().any(|op| matches!(
//!     op,
//!     Operation::CreateTable(ct) if ct.name == "posts"
//! )));
//! ```
//!
//! ### What the diff engine detects
//!
//! | Change | Detected? | Mechanism |
//! |---|---|---|
//! | New table | Yes | In desired, not in current |
//! | Dropped table | Yes | In current, not in desired |
//! | Added column | Yes | Column in desired, not in current |
//! | Dropped column | Yes | Column in current, not in desired |
//! | Column type change | Yes | `data_type` differs |
//! | Nullable change | Yes | `nullable` differs |
//! | Default add/change | Yes | `default` differs |
//! | Default removed | Yes | Old has default, new does not |
//! | Column rename | **No** | Flagged as [`AmbiguousChange::PossibleRename`] |
//! | Table rename | **No** | Flagged as [`AmbiguousChange::PossibleTableRename`] |
//! | Foreign key changes | **No** | Not tracked in `ColumnSchema` |
//!
//! ### Ambiguous changes
//!
//! When exactly one column is dropped and one is added with the same
//! type, the diff engine flags this as a possible rename rather than
//! generating a drop+add pair. The same heuristic applies at the table
//! level. Your migration tooling should present these to the user for
//! confirmation:
//!
//! ```rust
//! use oxide_sql_core::migrations::{
//!     AmbiguousChange, TableSnapshot, ColumnSnapshot, auto_diff_schema,
//!     SchemaSnapshot,
//! };
//! use oxide_sql_core::ast::DataType;
//!
//! let mut current = SchemaSnapshot::new();
//! current.add_table(TableSnapshot {
//!     name: "users".into(),
//!     columns: vec![ColumnSnapshot {
//!         name: "id".into(),
//!         data_type: DataType::Bigint,
//!         nullable: false,
//!         primary_key: true,
//!         unique: false,
//!         autoincrement: false,
//!         default: None,
//!     }],
//! });
//!
//! // "desired" has the same structure but the table is named "accounts".
//! let mut desired = SchemaSnapshot::new();
//! desired.add_table(TableSnapshot {
//!     name: "accounts".into(),
//!     columns: vec![ColumnSnapshot {
//!         name: "id".into(),
//!         data_type: DataType::Bigint,
//!         nullable: false,
//!         primary_key: true,
//!         unique: false,
//!         autoincrement: false,
//!         default: None,
//!     }],
//! });
//!
//! let diff = auto_diff_schema(&current, &desired);
//! // No operations generated — flagged as ambiguous instead.
//! assert!(diff.operations.is_empty());
//! assert!(matches!(
//!     &diff.ambiguous[0],
//!     AmbiguousChange::PossibleTableRename {
//!         old_table,
//!         new_table,
//!     } if old_table == "users" && new_table == "accounts"
//! ));
//! ```
//!
//! ### Operation ordering
//!
//! Operations are returned in a safe order to avoid FK constraint
//! violations:
//!
//! 1. `CreateTable` — new tables first
//! 2. `AddColumn` — new columns on existing tables
//! 3. `AlterColumn` — type/nullable/default changes
//! 4. `DropColumn` — removed columns
//! 5. `DropTable` — removed tables last
//!
//! ### Database introspection
//!
//! The [`Introspect`] trait defines how to read the current schema from
//! a live database connection:
//!
//! ```rust,ignore
//! use oxide_sql_core::migrations::{Introspect, SchemaSnapshot};
//!
//! // Driver crates implement this:
//! impl Introspect for MyDatabaseConnection {
//!     type Error = MyError;
//!     fn introspect_schema(&self) -> Result<SchemaSnapshot, MyError> {
//!         // Query information_schema / sqlite_master / etc.
//!     }
//! }
//! ```
//!
//! With introspection + diff, the full `makemigrations` workflow is:
//!
//! ```rust,ignore
//! // 1. Introspect the current database.
//! let current = db.introspect_schema()?;
//!
//! // 2. Build desired schema from derive(Table) structs.
//! let mut desired = SchemaSnapshot::new();
//! desired.add_from_table_schema::<UserTable>(&dialect);
//! desired.add_from_table_schema::<PostTable>(&dialect);
//!
//! // 3. Diff.
//! let diff = auto_diff_schema(&current, &desired);
//!
//! // 4. Generate SQL for each operation.
//! for op in &diff.operations {
//!     let sql = dialect.generate_sql(op);
//!     println!("{sql}");
//! }
//!
//! // 5. Handle ambiguous changes (ask user about renames).
//! for change in &diff.ambiguous {
//!     println!("Ambiguous: {change:?}");
//! }
//! ```

pub mod ast;
pub mod builder;
pub mod dialect;
pub mod lexer;
pub mod migrations;
pub mod parser;
pub mod schema;

pub use ast::{Expr, Statement};
pub use builder::{
    Delete, DeleteDyn, Insert, InsertDyn, Select, SelectDyn, Update, UpdateDyn, col, dyn_col,
};
pub use lexer::{Lexer, Token, TokenKind};
pub use migrations::{
    AmbiguousChange, ColumnSnapshot, Introspect, SchemaDiff, SchemaSnapshot, TableSnapshot,
    auto_diff_schema, auto_diff_table,
};
pub use parser::{ParseError, Parser};
pub use schema::{
    Column, ColumnSchema, RustTypeMapping, Selectable, Table, TableSchema, TypedColumn,
};
