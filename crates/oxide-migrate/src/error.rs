//! Error types for the migration system.

use std::path::PathBuf;

/// Errors that can occur during migration operations.
#[derive(Debug, thiserror::Error)]
pub enum MigrateError {
    /// A migration has a circular dependency.
    #[error("Circular dependency detected in migrations")]
    CircularDependency,

    /// A migration depends on another that doesn't exist.
    #[error("Migration '{migration}' depends on '{dependency}' which doesn't exist")]
    MissingDependency {
        /// The migration with the missing dependency.
        migration: String,
        /// The dependency that's missing.
        dependency: String,
    },

    /// A migration is not reversible.
    #[error("Migration '{0}' is not reversible")]
    NotReversible(String),

    /// Database error during migration execution.
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// IO error (reading/writing migration files).
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Failed to parse migration file.
    #[error("Failed to parse migration file '{path}': {message}")]
    ParseError {
        /// Path to the migration file.
        path: PathBuf,
        /// Error message.
        message: String,
    },

    /// Migration file already exists.
    #[error("Migration file already exists: {0}")]
    MigrationExists(PathBuf),

    /// No migrations directory found.
    #[error("Migrations directory not found: {0}")]
    MigrationsDirNotFound(PathBuf),

    /// Migration not found.
    #[error("Migration not found: {app}/{name}")]
    MigrationNotFound {
        /// Application name.
        app: String,
        /// Migration name.
        name: String,
    },

    /// Invalid migration state.
    #[error("Invalid migration state: {0}")]
    InvalidState(String),

    /// Schema mismatch between model and database.
    #[error("Schema mismatch: {0}")]
    SchemaMismatch(String),

    /// Serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Multiple errors occurred.
    #[error("Multiple errors occurred:\n{}", .0.iter().map(|e| format!("  - {}", e)).collect::<Vec<_>>().join("\n"))]
    Multiple(Vec<MigrateError>),
}

/// Result type for migration operations.
pub type Result<T> = std::result::Result<T, MigrateError>;
