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
//! Define a model by implementing the [`Model`] trait (or use a derive
//! macro), then use [`Manager`] and [`QuerySet`] for database operations.
//! See the [`Model`] trait docs for the full API.
//!
//! ## QuerySet Operations
//!
//! QuerySets are lazy and chainable. See [`QuerySet`] for the full API.
//!
//! ## Complex Filters with Q Objects
//!
//! ```rust
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
