//! # oxide-orm
//!
//! A Django-like ORM for Rust with type-safe queries and async support.
//!
//! This crate provides:
//! - `Model` trait for database models
//! - `Manager` for database access patterns
//! - `QuerySet` for lazy, chainable queries
//! - `Q` objects for complex filter expressions
//! - Field types with validation
//!
//! ## Quick Start
//!
//! ```ignore
//! use oxide_orm::{Model, Q};
//! use sqlx::SqlitePool;
//!
//! #[derive(Model)]
//! #[model(table = "users")]
//! struct User {
//!     #[field(primary_key, auto)]
//!     id: i64,
//!     #[field(max_length = 150)]
//!     username: String,
//!     email: String,
//!     is_active: bool,
//! }
//!
//! async fn example(pool: &SqlitePool) -> oxide_orm::Result<()> {
//!     // Get all active users
//!     let users = User::objects()
//!         .filter(Q::eq("is_active", true))
//!         .order_by("-id")
//!         .execute(pool)
//!         .await?;
//!
//!     // Get a specific user
//!     let user = User::objects().get(pool, 1).await?;
//!
//!     // Count users
//!     let count = User::objects().all().count(pool).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## QuerySet Operations
//!
//! QuerySets are lazy and chainable:
//!
//! ```ignore
//! // Chaining operations
//! let qs = User::objects()
//!     .filter(Q::eq("is_active", true))
//!     .exclude(Q::eq("role", "banned"))
//!     .order_by("-created_at")
//!     .limit(10);
//!
//! // Execute when needed
//! let users = qs.execute(&pool).await?;
//!
//! // Or get just the first result
//! let first = qs.first(&pool).await?;
//!
//! // Or check existence
//! let exists = qs.exists(&pool).await?;
//! ```
//!
//! ## Complex Filters with Q Objects
//!
//! ```ignore
//! use oxide_orm::Q;
//!
//! // AND conditions
//! let filter = Q::eq("status", "active").and(Q::gt("age", 18));
//!
//! // OR conditions
//! let filter = Q::eq("role", "admin").or(Q::eq("role", "moderator"));
//!
//! // NOT conditions
//! let filter = Q::eq("deleted", true).not();
//!
//! // Complex combinations
//! let filter = Q::eq("status", "active")
//!     .and(Q::gt("age", 18).or(Q::eq("verified", true)));
//! ```

mod error;
pub mod fields;
mod manager;
mod model;
pub mod query;
mod queryset;

pub use error::{OrmError, Result};
pub use manager::Manager;
pub use model::{Model, ModelInstance};
pub use query::{avg, count, count_all, count_distinct, max, min, sum, Aggregate, Q};
pub use queryset::{OrderBy, OrderDirection, QuerySet};

// Re-export commonly used types from oxide-sql-core
pub use oxide_sql_core::builder::value::{SqlValue, ToSqlValue};
pub use oxide_sql_core::schema::Table;
