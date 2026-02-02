//! # oxide-sql-core
//!
//! A type-safe SQL parser and builder with compile-time validation.
//!
//! This crate provides:
//! - A hand-written recursive descent parser with Pratt expression parsing
//! - A type-safe SQL builder using the typestate pattern
//! - Protection against SQL injection through parameterized queries
//! - A type-safe migrations system (Django-like)
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
//!
//! ## Type-Safe Migrations
//!
//! The migrations module provides Django-like database migrations with compile-time
//! validation:
//!
//! ```rust
//! use oxide_sql_core::migrations::{
//!     Migration, Operation, CreateTableBuilder,
//!     bigint, varchar, timestamp,
//! };
//!
//! pub struct Migration0001;
//!
//! impl Migration for Migration0001 {
//!     const ID: &'static str = "0001_create_users";
//!
//!     fn up() -> Vec<Operation> {
//!         vec![
//!             CreateTableBuilder::new()
//!                 .name("users")
//!                 .column(bigint("id").primary_key().autoincrement().build())
//!                 .column(varchar("username", 255).not_null().unique().build())
//!                 .column(timestamp("created_at").not_null().default_expr("CURRENT_TIMESTAMP").build())
//!                 .build()
//!                 .into(),
//!         ]
//!     }
//!
//!     fn down() -> Vec<Operation> {
//!         vec![Operation::drop_table("users")]
//!     }
//! }
//! ```

pub mod ast;
pub mod builder;
pub mod dialect;
pub mod lexer;
pub mod migrations;
pub mod parser;
pub mod schema;

pub use ast::{Expr, Statement};
pub use builder::{Delete, Insert, Select, Update, col};
pub use lexer::{Lexer, Token, TokenKind};
pub use parser::{ParseError, Parser};
pub use schema::{Column, Selectable, Table, TypedColumn};
