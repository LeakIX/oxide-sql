//! Type-Safe Database Migrations System
//!
//! This module provides a compile-time validated migrations system inspired by Django,
//! where:
//! - Invalid migrations fail to compile (missing columns, wrong types, etc.)
//! - Operations are reversible with typed `up()` and `down()` methods
//! - SQL generation is dialect-aware
//!
//! # Example
//!
//! ```rust
//! use oxide_sql_core::migrations::{
//!     Migration, Operation, CreateTableBuilder,
//!     bigint, varchar, timestamp, boolean,
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
//!                 .column(varchar("email", 255).build())
//!                 .column(timestamp("created_at").not_null().default_expr("CURRENT_TIMESTAMP").build())
//!                 .build()
//!                 .into(),
//!         ]
//!     }
//!
//!     fn down() -> Vec<Operation> {
//!         vec![
//!             Operation::drop_table("users"),
//!         ]
//!     }
//! }
//! ```

mod column_builder;
pub mod dialect;
mod migration;
mod operation;
mod state;
mod table_builder;

pub use column_builder::{
    bigint, binary, blob, boolean, char, date, datetime, decimal, double, integer, numeric, real,
    smallint, text, time, timestamp, varbinary, varchar, ColumnBuilder, ColumnDefinition,
    DefaultValue, ForeignKeyRef,
};
pub use dialect::{DuckDbDialect, MigrationDialect, PostgresDialect, SqliteDialect};
pub use migration::{Migration, MigrationRunner, MigrationStatus};
pub use operation::{
    AddColumnOp, AddForeignKeyOp, AlterColumnOp, CreateIndexOp, CreateTableOp, DropColumnOp,
    DropForeignKeyOp, DropIndexOp, DropTableOp, IndexType, Operation, RawSqlOp, RenameColumnOp,
    RenameTableOp,
};
pub use state::MigrationState;
pub use table_builder::{
    CreateTableBuilder, DropTableBuilder, HasColumns, HasName, NoColumns, NoName,
};
