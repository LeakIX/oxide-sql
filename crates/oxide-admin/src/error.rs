//! Error types for the admin interface.

use thiserror::Error;

/// Admin-specific errors.
#[derive(Debug, Error)]
pub enum AdminError {
    /// Database error.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    /// ORM error.
    #[error("ORM error: {0}")]
    Orm(#[from] oxide_orm::OrmError),

    /// Authentication error.
    #[error("authentication error: {0}")]
    Auth(#[from] oxide_auth::AuthError),

    /// Model not registered.
    #[error("model not registered: {0}")]
    ModelNotRegistered(String),

    /// Object not found.
    #[error("object not found")]
    NotFound,

    /// Permission denied.
    #[error("permission denied")]
    PermissionDenied,

    /// Validation error.
    #[error("validation error: {0}")]
    Validation(String),

    /// Template rendering error.
    #[error("template error: {0}")]
    TemplateError(String),
}

/// Result type alias for admin operations.
pub type Result<T> = std::result::Result<T, AdminError>;
