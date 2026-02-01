//! # oxide-sql-core
//!
//! A type-safe SQL parser and builder with compile-time validation.
//!
//! This crate provides:
//! - A hand-written recursive descent parser with Pratt expression parsing
//! - A type-safe SQL builder using the typestate pattern
//! - Protection against SQL injection through parameterized queries
//!
//! ## Type-Safe SQL Building
//!
//! The builder API uses Rust's type system to prevent invalid SQL at compile time:
//!
//! ```rust
//! use oxide_sql_core::builder::{Select, col};
//!
//! // Valid: Complete SELECT statement
//! let query = Select::new()
//!     .columns(&["id", "name"])
//!     .from("users")
//!     .where_clause(col("active").eq(true))
//!     .build();
//!
//! // This would NOT compile:
//! // let query = Select::new()
//! //     .columns(&["id", "name"])
//! //     .build();  // Error: missing FROM clause
//! ```
//!
//! ## SQL Injection Prevention
//!
//! All values are automatically escaped and parameterized:
//!
//! ```rust
//! use oxide_sql_core::builder::{Select, col};
//!
//! let user_input = "'; DROP TABLE users; --";
//! let (sql, params) = Select::new()
//!     .columns(&["id"])
//!     .from("users")
//!     .where_clause(col("name").eq(user_input))
//!     .build();
//!
//! // sql = "SELECT id FROM users WHERE name = ?"
//! // params = vec![SqlValue::Text("'; DROP TABLE users; --")]
//! ```

pub mod ast;
pub mod builder;
pub mod dialect;
pub mod lexer;
pub mod parser;
pub mod schema;

pub use ast::{Expr, Statement};
pub use builder::{Delete, Insert, Select, Update, col};
pub use lexer::{Lexer, Token, TokenKind};
pub use parser::{ParseError, Parser};
pub use schema::{Column, Selectable, Table, TypedColumn};
