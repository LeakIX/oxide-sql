//! Type-Safe SQL Builder
//!
//! This module provides a type-safe SQL builder using the typestate pattern.
//! Invalid SQL constructs are caught at compile time.
//!
//! # Example
//!
//! ```rust
//! use oxide_sql_core::builder::{Select, col};
//!
//! // Valid: Complete SELECT statement
//! let (sql, params) = Select::new()
//!     .columns(&["id", "name"])
//!     .from("users")
//!     .where_clause(col("active").eq(true))
//!     .build();
//!
//! assert_eq!(sql, "SELECT id, name FROM users WHERE active = ?");
//! ```

mod delete;
mod expr;
mod insert;
mod select;
pub mod typed;
mod update;
pub mod value;

pub use delete::{Delete, SafeDelete, SafeDeleteWithWhere};
pub use expr::{col, Column, ExprBuilder};
pub use insert::Insert;
pub use select::Select;
pub use typed::{typed_col, TypedDelete, TypedInsert, TypedSelect, TypedUpdate};
pub use update::Update;
pub use value::{SqlValue, ToSqlValue};
