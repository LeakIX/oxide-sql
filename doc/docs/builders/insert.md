---
sidebar_position: 2
---

# INSERT Builder

The `Insert` builder creates INSERT statements with parameterized values.

## Basic Usage

```rust
use oxide_sql_core::builder::Insert;

let (sql, params) = Insert::new()
    .into_table("users")
    .columns(&["name", "email"])
    .values(&[&"Alice", &"alice@example.com"])
    .build();

assert_eq!(sql, "INSERT INTO users (name, email) VALUES (?, ?)");
```

## Multiple Rows

```rust
use oxide_sql_core::builder::Insert;

let (sql, params) = Insert::new()
    .into_table("users")
    .columns(&["name", "email"])
    .values(&[&"Alice", &"alice@example.com"])
    .values(&[&"Bob", &"bob@example.com"])
    .build();

assert_eq!(
    sql,
    "INSERT INTO users (name, email) VALUES (?, ?), (?, ?)"
);
```

## Type Safety

The builder accepts any type that implements `ToSqlValue`:

```rust
use oxide_sql_core::builder::Insert;

let (sql, params) = Insert::new()
    .into_table("products")
    .columns(&["name", "price", "quantity", "active"])
    .values(&[&"Widget", &19.99_f64, &100_i32, &true])
    .build();
```

## SQL Injection Prevention

All values are parameterized:

```rust
use oxide_sql_core::builder::Insert;

let malicious = "'); DROP TABLE users; --";

let (sql, params) = Insert::new()
    .into_table("users")
    .columns(&["name"])
    .values(&[&malicious])
    .build();

// The SQL is safe
assert_eq!(sql, "INSERT INTO users (name) VALUES (?)");
// The malicious string is safely stored as a parameter
```
