//! Error types for the ORM.

use thiserror::Error;

/// ORM-specific errors.
#[derive(Debug, Error)]
pub enum OrmError {
    /// Database error from sqlx.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    /// No object found matching the query.
    #[error("object not found")]
    NotFound,

    /// Multiple objects found when exactly one was expected.
    #[error("multiple objects returned when one was expected")]
    MultipleObjectsReturned,

    /// Validation error.
    #[error("validation error: {0}")]
    Validation(String),

    /// Invalid field name.
    #[error("invalid field: {0}")]
    InvalidField(String),

    /// Query building error.
    #[error("query error: {0}")]
    QueryError(String),
}

/// Result type alias for ORM operations.
pub type Result<T> = std::result::Result<T, OrmError>;
