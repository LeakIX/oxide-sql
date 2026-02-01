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
- **Hand-Written Parser**: Recursive descent parser with Pratt expression parsing
- **no_std Support**: Works in embedded and WebAssembly environments
- **SQLite Extensions**: SQLite-specific syntax like UPSERT

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

- [Getting Started](./getting-started)
- [SQL Security Guide](./security/)
- [API Reference](./api/)
