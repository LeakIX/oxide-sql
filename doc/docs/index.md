---
sidebar_position: 0
---

# Oxide SQL

A type-safe SQL parser and builder for Rust with compile-time validation and
SQL injection prevention.

## Features

- **Type-Safe SQL Building**: Invalid SQL constructs are caught at compile time
  using the typestate pattern
- **SQL Injection Prevention**: All user input is automatically parameterized
- **Django-like ORM**: Familiar QuerySet API, Model trait, and Managers
- **Admin Interface**: Automatic CRUD admin interface
- **Database Migrations**: Django-style migrations with auto-detection
- **Authentication**: User management, sessions, and permissions
- **Hand-Written Parser**: Recursive descent parser with Pratt expression parsing
- **SQLite Extensions**: SQLite-specific syntax like UPSERT

## Try the Admin Interface

```bash
cargo run -p oxide-admin --example blog_admin
```

Then open http://localhost:3000/admin/ and login with `admin` / `admin123`.

## Quick Start

Add the dependencies to your `Cargo.toml`:

```toml
[dependencies]
oxide-sql-core = "0.1"
oxide-sql-sqlite = "0.1"  # Optional, for SQLite-specific features
```

### Building Type-Safe Queries

```rust
use oxide_sql_core::builder::{Select, col};

// This compiles - valid SELECT statement
let (sql, params) = Select::new()
    .columns(&["id", "name"])
    .from("users")
    .where_clause(col("active").eq(true))
    .build();

assert_eq!(sql, "SELECT id, name FROM users WHERE active = ?");

// This would NOT compile - missing FROM clause
// let query = Select::new()
//     .columns(&["id", "name"])
//     .build();  // Error: method `build` not found
```

### SQL Injection Prevention

User input is always parameterized, never interpolated:

```rust
use oxide_sql_core::builder::{Select, col};

let user_input = "'; DROP TABLE users; --";
let (sql, params) = Select::new()
    .columns(&["id"])
    .from("users")
    .where_clause(col("name").eq(user_input))
    .build();

// sql = "SELECT id FROM users WHERE name = ?"
// The malicious input is safely stored as a parameter
```

## Why Oxide SQL?

1. **Compile-Time Safety**: Catch SQL syntax errors before runtime
2. **Security First**: SQL injection is prevented by design
3. **Zero Runtime Overhead**: Type states are zero-sized types
4. **Extensible**: Support for database-specific dialects

## Documentation

- [Getting Started](./getting-started) - Installation and basic usage
- [SQL Builders](./builders/) - SELECT, INSERT, UPDATE, DELETE builders
- [Type-Safe Schema](./schema/) - Define tables with derive macros
- [Admin Interface](./admin/) - Django-like admin for your models
- [SQL Security Guide](./security/) - SQL injection and prevention
