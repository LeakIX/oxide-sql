//! # oxide-sql-sqlite
//!
//! SQLite-specific SQL parser and builder extensions.
//!
//! This crate extends `oxide-sql-core` with SQLite-specific features:
//! - SQLite dialect with proper identifier quoting
//! - UPSERT (ON CONFLICT) support
//! - SQLite-specific data types
//!
//! ## Example
//!
//! ```rust
//! use oxide_sql_sqlite::UpsertBuilder;
//! use oxide_sql_core::builder::value::ToSqlValue;
//!
//! // UPSERT example
//! let (sql, params) = UpsertBuilder::new()
//!     .into_table("users")
//!     .columns(&["id", "name", "email"])
//!     .values(vec![
//!         1_i64.to_sql_value(),
//!         "Alice".to_sql_value(),
//!         "alice@example.com".to_sql_value(),
//!     ])
//!     .on_conflict(&["id"])
//!     .do_update(&["name", "email"])
//!     .build();
//! ```

pub mod builder;
mod dialect;

pub use builder::UpsertBuilder;
pub use dialect::SqliteDialect;
