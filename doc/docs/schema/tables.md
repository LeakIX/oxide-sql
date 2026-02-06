---
sidebar_position: 1
---

# Defining Tables

Use the `#[derive(Table)]` macro to define database tables as Rust structs.

## Features

- Automatic table name from struct name (snake_case)
- Custom table names via `#[table(name = "...")]`
- Generated `UserTable` type implementing the `Table` trait
- Generated `UserColumns` module with column types
- Accessor methods on both the struct and table type

## Supported Field Types

The derive macro works with any Rust type. Common mappings:

| Rust Type | SQL Type |
|-----------|----------|
| `i32`, `i64` | INTEGER |
| `f32`, `f64` | REAL / FLOAT |
| `String` | TEXT / VARCHAR |
| `bool` | BOOLEAN / INTEGER |
| `Vec<u8>` | BLOB |
| `Option<T>` | Nullable column |

## API Reference

See the [`#[derive(Table)]` rustdoc](pathname:///oxide-sql/rustdoc/oxide_sql_derive/) for the full
macro documentation with code examples.
