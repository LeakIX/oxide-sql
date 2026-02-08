//! # oxide-auth
//!
//! Django-like authentication system for Rust with users, sessions, and permissions.
//!
//! This crate provides:
//! - `User` model with password hashing (Argon2)
//! - `Session` management for stateful authentication
//! - `Permission` and `Group` models for authorization
//! - `DatabaseBackend` for authenticating against the database
//!
//! ## Quick Start
//!
//! ```rust
//! use oxide_auth::{User, hash_password, verify_password};
//!
//! // Create a user with hashed password
//! let user = User::create("alice", "alice@example.com", "password123")
//!     .expect("valid user");
//!
//! // Verify the password
//! assert!(user.check_password("password123"));
//! assert!(!user.check_password("wrongpassword"));
//! ```
//!
//! For the full async workflow with database persistence, see the
//! [`DatabaseBackend`] documentation.
//!
//! ## Password Hashing
//!
//! Passwords are hashed using Argon2id, a memory-hard hashing algorithm that
//! is resistant to GPU and ASIC attacks.
//!
//! ```rust
//! use oxide_auth::{hash_password, verify_password};
//!
//! let hash = hash_password("secret123").expect("hashing works");
//! assert!(verify_password("secret123", &hash));
//! assert!(!verify_password("wrong", &hash));
//! ```
//!
//! ## Sessions
//!
//! Sessions track user authentication state across requests.
//! Use [`Session::create`] to generate a new session for a user,
//! and persist it with `Session::save(pool).await`.
//! See the [`Session`] and [`SessionData`] docs for the full API.
//!
//! ## Permissions
//!
//! Permissions can be assigned to users directly or through groups.
//! See [`Permission`], [`Group`], [`add_user_to_group`], and
//! [`user_has_permission`] for the full API.

pub mod backends;
mod error;
mod password;
mod permissions;
mod session;
mod user;

pub use backends::DatabaseBackend;
pub use error::{AuthError, Result};
pub use password::{hash_password, validate_password, verify_password};
pub use permissions::{
    add_user_permission, add_user_to_group, create_permission_tables, get_user_groups,
    get_user_permissions, remove_user_from_group, remove_user_permission, user_has_permission,
    Group, Permission,
};
pub use session::{create_session_table, Session, SessionData};
pub use user::{create_user_table, User};

use sqlx::SqlitePool;

/// Creates all authentication tables.
///
/// This should be called during application setup to ensure all
/// required tables exist.
pub async fn create_tables(pool: &SqlitePool) -> Result<()> {
    create_user_table(pool).await?;
    create_session_table(pool).await?;
    create_permission_tables(pool).await?;
    Ok(())
}
