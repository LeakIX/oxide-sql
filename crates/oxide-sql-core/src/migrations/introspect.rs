//! Schema introspection trait.
//!
//! Driver crates (oxide-sql-sqlite, etc.) implement [`Introspect`]
//! to read the current database schema at runtime. The core crate
//! defines only the trait so it stays driver-agnostic.

use super::snapshot::SchemaSnapshot;

/// Introspects a live database connection to produce a
/// [`SchemaSnapshot`] of the current schema.
///
/// Implementations live in driver crates (e.g. oxide-sql-sqlite).
pub trait Introspect {
    /// Error type for introspection failures.
    type Error: std::error::Error;

    /// Reads the current database schema and returns a snapshot.
    fn introspect_schema(&self) -> Result<SchemaSnapshot, Self::Error>;
}
