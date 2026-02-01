---
sidebar_position: 1
---

# Getting Started

This guide will help you get started with Oxide SQL, a type-safe SQL parser and
builder for Rust.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
oxide-sql-core = "0.1"
oxide-sql-derive = "0.1"  # For derive macros
oxide-sql-sqlite = "0.1"  # Optional, for SQLite-specific features
```

## Basic Usage

### String-Based Queries

The simplest way to use Oxide SQL is with the string-based builder:

```rust
use oxide_sql_core::builder::{Select, col};

let (sql, params) = Select::new()
    .columns(&["id", "name", "email"])
    .from("users")
    .where_clause(col("active").eq(true))
    .build();

assert_eq!(sql, "SELECT id, name, email FROM users WHERE active = ?");
```

### Type-Safe Queries with Derive Macros

For compile-time validation of column names, use the derive macro:

```rust
use oxide_sql_derive::Table;
use oxide_sql_core::builder::TypedSelect;

#[derive(Table)]
#[table(name = "users")]
struct User {
    #[column(primary_key)]
    id: i32,
    name: String,
    email: String,
    active: bool,
}

// Column names are validated at compile time
let (sql, params) = TypedSelect::<UserTable, _, _>::new()
    .select::<(UserColumns::Id, UserColumns::Name)>()
    .from_table()
    .build();

assert_eq!(sql, "SELECT id, name FROM users");
```

## SQL Injection Prevention

Oxide SQL automatically parameterizes all user input:

```rust
use oxide_sql_core::builder::{Select, col};

// Even malicious input is safely parameterized
let user_input = "'; DROP TABLE users; --";

let (sql, params) = Select::new()
    .columns(&["id"])
    .from("users")
    .where_clause(col("name").eq(user_input))
    .build();

// The SQL is safe - the malicious string is a parameter, not interpolated
assert_eq!(sql, "SELECT id FROM users WHERE name = ?");
// params contains the raw string, to be passed safely to the database driver
```

## Compile-Time Safety

The typestate pattern ensures that invalid SQL cannot be constructed:

```rust
use oxide_sql_core::builder::Select;

// This compiles - valid SELECT with FROM
let query = Select::new()
    .columns(&["id"])
    .from("users")
    .build();

// This would NOT compile - SELECT without FROM
// let query = Select::new()
//     .columns(&["id"])
//     .build();  // Error: method `build` not found
```

## Next Steps

- Learn about the [Builder API](./builders/)
- Explore [Type-Safe Schema Definitions](./schema/)
- Read about [SQL Security](./security/)
