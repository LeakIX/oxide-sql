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
//! ```ignore
//! use oxide_auth::{User, DatabaseBackend, create_tables};
//! use sqlx::SqlitePool;
//!
//! async fn example(pool: &SqlitePool) -> oxide_auth::Result<()> {
//!     // Create auth tables
//!     create_tables(pool).await?;
//!
//!     // Create a user
//!     let mut user = User::create("alice", "alice@example.com", "password123")?;
//!     user.save(pool).await?;
//!
//!     // Authenticate and login
//!     let (user, session) = DatabaseBackend::login(pool, "alice", "password123").await?;
//!
//!     // Get user from session
//!     let user = DatabaseBackend::get_user(pool, &session.session_key).await?;
//!
//!     // Logout
//!     DatabaseBackend::logout(pool, &session.session_key).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Password Hashing
//!
//! Passwords are hashed using Argon2id, a memory-hard hashing algorithm that
//! is resistant to GPU and ASIC attacks.
//!
//! ```ignore
//! use oxide_auth::User;
//!
//! let user = User::create("alice", "alice@example.com", "password123")?;
//!
//! // Check password
//! assert!(user.check_password("password123"));
//! assert!(!user.check_password("wrongpassword"));
//!
//! // Change password
//! user.set_password("newpassword123")?;
//! ```
//!
//! ## Sessions
//!
//! Sessions track user authentication state across requests.
//!
//! ```ignore
//! use oxide_auth::Session;
//!
//! // Create a session for a user
//! let session = Session::for_user(&user);
//! session.save(pool).await?;
//!
//! // Store data in session
//! session.set("cart_items", vec![1, 2, 3]);
//!
//! // Retrieve data
//! let items: Option<Vec<i32>> = session.get("cart_items");
//! ```
//!
//! ## Permissions
//!
//! Permissions can be assigned to users directly or through groups.
//!
//! ```ignore
//! use oxide_auth::{Permission, Group, add_user_to_group, user_has_permission};
//!
//! // Create a permission
//! let mut perm = Permission::new("edit_posts", "Can edit posts");
//! perm.save(pool).await?;
//!
//! // Create a group and add permission
//! let mut group = Group::new("Editors");
//! group.save(pool).await?;
//! group.add_permission(pool, perm.id).await?;
//!
//! // Add user to group
//! add_user_to_group(pool, user.id, group.id).await?;
//!
//! // Check permission
//! let can_edit = user_has_permission(pool, user.id, "edit_posts").await?;
//! ```

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
