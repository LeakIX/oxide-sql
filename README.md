# Oxide SQL

A type-safe SQL parser and builder for Rust with compile-time validation and
SQL injection prevention.

## Features

- **Type-Safe SQL Building**: Invalid SQL constructs are caught at compile time
  using the typestate pattern
- **SQL Injection Prevention**: All user input is automatically parameterized
- **Hand-Written Parser**: Recursive descent parser with Pratt expression
  parsing
- **no_std Support**: Works in embedded and WebAssembly environments
- **SQLite Extensions**: SQLite-specific syntax like UPSERT

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
oxide-sql-core = "0.1"
oxide-sql-sqlite = "0.1"  # Optional, for SQLite-specific features
```

## Quick Start

### Type-Safe Query Building

```rust
use oxide_sql_core::builder::{Select, col};

// Valid: Complete SELECT statement
let (sql, params) = Select::new()
    .columns(&["id", "name"])
    .from("users")
    .where_clause(col("active").eq(true))
    .build();

assert_eq!(sql, "SELECT id, name FROM users WHERE active = ?");

// This would NOT compile - missing FROM clause:
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

### SQLite UPSERT

```rust
use oxide_sql_sqlite::builder::Upsert;
use oxide_sql_core::builder::col;

let (sql, params) = Upsert::new()
    .into_table("users")
    .columns(&["id", "name", "email"])
    .values(&[&1_i32, &"Alice", &"alice@example.com"])
    .on_conflict(&["id"])
    .do_update(&["name", "email"])
    .build();
```

## Crates

- **oxide-sql-core**: Core parser and type-safe builders
- **oxide-sql-sqlite**: SQLite-specific extensions

## Documentation

- [Online Documentation](https://leakix.github.io/oxide-sql/)
- [API Reference](https://docs.rs/oxide-sql-core)

## Development

```bash
make build          # Build the project
make test           # Run tests
make lint           # Run clippy
make format         # Format code
make doc-dev        # Run documentation dev server
```

## License

MIT
