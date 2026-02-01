---
sidebar_position: 1
---

# Defining Tables

Use the `#[derive(Table)]` macro to define database tables as Rust structs.

## Basic Table Definition

```rust
use oxide_sql_derive::Table;

#[derive(Table)]
struct User {
    id: i32,
    name: String,
    email: String,
}
```

By default, the table name is the snake_case version of the struct name
(`user` in this case).

## Custom Table Name

Use the `#[table]` attribute to specify a custom table name:

```rust
use oxide_sql_derive::Table;

#[derive(Table)]
#[table(name = "app_users")]
struct User {
    id: i32,
    name: String,
}
```

## Generated Types

For a struct `User`, the macro generates:

### UserTable

A zero-sized type implementing the `Table` trait:

```rust
// Generated automatically
pub struct UserTable;

impl Table for UserTable {
    type Row = User;
    const NAME: &'static str = "user";
    const COLUMNS: &'static [&'static str] = &["id", "name", "email"];
    const PRIMARY_KEY: Option<&'static str> = None;
}
```

### UserColumns Module

A module containing column types:

```rust
// Generated automatically
pub mod UserColumns {
    pub struct Id;      // Implements Column<Table = UserTable, Type = i32>
    pub struct Name;    // Implements Column<Table = UserTable, Type = String>
    pub struct Email;   // Implements Column<Table = UserTable, Type = String>
}
```

### Accessor Methods

Both `User` and `UserTable` get accessor methods:

```rust
// Access column types
let id_col = User::id();        // Returns UserColumns::Id
let name_col = UserTable::name();  // Returns UserColumns::Name
```

## Multiple Tables

Define related tables with proper type safety:

```rust
use oxide_sql_derive::Table;

#[derive(Table)]
#[table(name = "users")]
struct User {
    #[column(primary_key)]
    id: i32,
    name: String,
}

#[derive(Table)]
#[table(name = "posts")]
struct Post {
    #[column(primary_key)]
    id: i32,
    user_id: i32,  // Foreign key to users
    title: String,
    content: String,
}
```

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
