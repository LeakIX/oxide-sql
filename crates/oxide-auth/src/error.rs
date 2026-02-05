//! Error types for authentication.

use thiserror::Error;

/// Authentication-specific errors.
#[derive(Debug, Error)]
pub enum AuthError {
    /// Database error.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    /// ORM error.
    #[error("ORM error: {0}")]
    Orm(#[from] oxide_orm::OrmError),

    /// Invalid credentials.
    #[error("invalid credentials")]
    InvalidCredentials,

    /// User not found.
    #[error("user not found")]
    UserNotFound,

    /// Session not found or expired.
    #[error("session not found or expired")]
    SessionNotFound,

    /// User is inactive.
    #[error("user account is inactive")]
    UserInactive,

    /// Permission denied.
    #[error("permission denied")]
    PermissionDenied,

    /// Password hashing error.
    #[error("password hashing error")]
    PasswordHashError,

    /// Session creation error.
    #[error("failed to create session")]
    SessionCreationError,

    /// Validation error.
    #[error("validation error: {0}")]
    Validation(String),
}

/// Result type alias for authentication operations.
pub type Result<T> = std::result::Result<T, AuthError>;
