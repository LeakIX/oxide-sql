//! # oxide-sql-sqlite
//!
//! SQLite-specific extensions for `oxide-sql-core`.
//!
//! # How SQLite differs from other dialects
//!
//! - **[UPSERT]**: SQLite supports
//!   `INSERT ... ON CONFLICT DO NOTHING` and
//!   `ON CONFLICT DO UPDATE SET ...` (since SQLite 3.24.0). This
//!   crate provides [`UpsertBuilder`] for type-safe upsert
//!   construction.
//! - **[RETURNING]**: SQLite supports `RETURNING` clauses on
//!   INSERT, UPDATE, and DELETE (since SQLite 3.35.0).
//! - **Identifier quoting**: SQLite uses double quotes (`"`) as
//!   the standard quoting style, though it also accepts backticks
//!   and square brackets. See [SQLite keywords].
//! - **[Type affinity]**: SQLite uses a type-affinity system rather
//!   than strict column types. Any column can store any value
//!   regardless of declared type (unless [`STRICT` tables] are
//!   used).
//! - **Limited [ALTER TABLE]**: SQLite only supports
//!   `RENAME TABLE`, `RENAME COLUMN`, and `ADD COLUMN`. It does
//!   not support `DROP COLUMN` (before 3.35.0), `ALTER COLUMN`,
//!   or `ADD CONSTRAINT`.
//! - **[AUTOINCREMENT]**: SQLite uses the `AUTOINCREMENT` keyword
//!   (not sequences or `SERIAL` like PostgreSQL/DuckDB).
//!
//! [UPSERT]: https://www.sqlite.org/lang_upsert.html
//! [RETURNING]: https://www.sqlite.org/lang_returning.html
//! [SQLite keywords]: https://www.sqlite.org/lang_keywords.html
//! [Type affinity]: https://www.sqlite.org/datatype3.html
//! [`STRICT` tables]: https://www.sqlite.org/stricttables.html
//! [ALTER TABLE]: https://www.sqlite.org/lang_altertable.html
//! [AUTOINCREMENT]: https://www.sqlite.org/autoinc.html
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
