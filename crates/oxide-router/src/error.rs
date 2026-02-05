//! Error types for routing.

use thiserror::Error;

/// Router-specific errors.
#[derive(Debug, Error)]
pub enum RouterError {
    /// No route matched the request.
    #[error("no route matched: {method} {path}")]
    NotFound { method: String, path: String },

    /// Method not allowed for this route.
    #[error("method not allowed: {method} for {path}")]
    MethodNotAllowed { method: String, path: String },

    /// Invalid path pattern.
    #[error("invalid path pattern: {0}")]
    InvalidPattern(String),

    /// Route name not found.
    #[error("route not found: {0}")]
    RouteNotFound(String),

    /// Middleware rejected the request.
    #[error("middleware rejected request: {0}")]
    MiddlewareRejection(String),
}

/// Result type alias for router operations.
pub type Result<T> = std::result::Result<T, RouterError>;
