//! Django-like database migrations for Rust.
//!
//! `oxide-migrate` provides a compile-time validated migration system inspired by Django,
//! where:
//! - Invalid migrations fail to compile (missing columns, wrong types, etc.)
//! - Operations are reversible with typed forward/backward methods
//! - SQL generation is dialect-aware (SQLite, PostgreSQL)
//!
//! # Architecture
//!
//! The migration system consists of several components:
//!
//! - **Operations** - Schema changes like `CreateTable`, `AddColumn`, `DropIndex`, etc.
//! - **Executor** - Applies migrations to a database, tracking history
//! - **Autodetector** - Diffs schemas to generate migration operations
//! - **Writer** - Generates Rust migration files
//! - **Dialect** - Database-specific SQL generation
//!
//! # Example
//!
//! ```rust,ignore
//! use oxide_migrate::prelude::*;
//!
//! // Define a migration
//! pub struct Migration0001;
//!
//! impl OxideMigration for Migration0001 {
//!     const APP: &'static str = "users";
//!     const NAME: &'static str = "0001_initial";
//!     const DEPENDENCIES: &'static [(&'static str, &'static str)] = &[];
//!
//!     fn operations() -> Vec<MigrationOperation> {
//!         vec![
//!             MigrationOperation::create_table(
//!                 "users",
//!                 vec![
//!                     ColumnSchema::new("id", SqlType::BigInt)
//!                         .primary_key()
//!                         .auto_increment(),
//!                     ColumnSchema::new("username", SqlType::Varchar(255))
//!                         .not_null()
//!                         .unique(),
//!                     ColumnSchema::new("email", SqlType::Varchar(255)),
//!                     ColumnSchema::new("created_at", SqlType::Timestamp)
//!                         .not_null()
//!                         .default(DefaultValue::Expression("CURRENT_TIMESTAMP".into())),
//!                 ],
//!                 vec!["id".to_string()],
//!             ),
//!         ]
//!     }
//! }
//! ```
//!
//! # CLI Usage
//!
//! ```bash
//! # Generate migrations from model changes
//! oxide-migrate makemigrations
//!
//! # Apply pending migrations
//! oxide-migrate migrate
//!
//! # Show migration status
//! oxide-migrate showmigrations
//!
//! # Rollback the last migration
//! oxide-migrate migrate --reverse
//! ```

pub mod autodetector;
pub mod dialect;
pub mod error;
pub mod executor;
pub mod history;
pub mod operations;
pub mod schema;
pub mod state;
pub mod writer;

/// Prelude for convenient imports.
pub mod prelude {
    pub use crate::autodetector::{Autodetector, AutodetectorOptions};
    pub use crate::dialect::{MigrationDialect, SqliteDialect};
    pub use crate::error::{MigrateError, Result};
    pub use crate::executor::{ExecutableMigration, MigrationExecutor};
    pub use crate::history::MigrationHistory;
    pub use crate::operations::{
        ColumnChanges, ForeignKeyBuilder, IndexBuilder, MigrationOperation,
    };
    pub use crate::schema::{
        ColumnSchema, DatabaseSchema, DefaultValue, ForeignKeyAction, ForeignKeySchema,
        IndexSchema, SqlType, TableSchema, UniqueConstraint,
    };
    pub use crate::state::SchemaState;
    pub use crate::writer::{generate_migration_name, MigrationWriter};
}

/// Trait for migrations defined in Rust code.
///
/// This trait is implemented by migration structs to define schema changes.
pub trait OxideMigration {
    /// Application/module name (e.g., "users", "posts").
    const APP: &'static str;

    /// Migration name (e.g., "0001_initial", "0002_add_email").
    const NAME: &'static str;

    /// Dependencies on other migrations.
    ///
    /// Each tuple is (app, name) of a migration that must run before this one.
    const DEPENDENCIES: &'static [(&'static str, &'static str)] = &[];

    /// Returns the migration operations.
    fn operations() -> Vec<operations::MigrationOperation>;

    /// Converts to an executable migration.
    fn to_executable() -> executor::ExecutableMigration {
        let mut migration = executor::ExecutableMigration::new(Self::APP, Self::NAME);
        for (dep_app, dep_name) in Self::DEPENDENCIES {
            migration = migration.depends_on(*dep_app, *dep_name);
        }
        migration.operations(Self::operations())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    struct TestMigration;

    impl OxideMigration for TestMigration {
        const APP: &'static str = "test";
        const NAME: &'static str = "0001_initial";

        fn operations() -> Vec<MigrationOperation> {
            vec![MigrationOperation::create_table(
                "test_table",
                vec![ColumnSchema::new("id", SqlType::BigInt).primary_key()],
                vec!["id".to_string()],
            )]
        }
    }

    #[test]
    fn test_migration_trait() {
        assert_eq!(TestMigration::APP, "test");
        assert_eq!(TestMigration::NAME, "0001_initial");

        let ops = TestMigration::operations();
        assert_eq!(ops.len(), 1);
    }

    #[test]
    fn test_to_executable() {
        let executable = TestMigration::to_executable();
        assert_eq!(executable.app, "test");
        assert_eq!(executable.name, "0001_initial");
        assert_eq!(executable.operations.len(), 1);
    }
}
