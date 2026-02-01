---
sidebar_position: 3
---

# Type-Safe Schema Definitions

Oxide SQL provides derive macros for defining database tables as Rust structs.
This enables compile-time validation of column names, preventing typos and
ensuring queries are valid before runtime.

## Overview

Instead of using string column names, you define your schema as Rust structs:

```rust
use oxide_sql_derive::Table;

#[derive(Table)]
#[table(name = "users")]
struct User {
    #[column(primary_key)]
    id: i32,
    name: String,
    email: String,
    active: bool,
}
```

The `#[derive(Table)]` macro generates:

- `UserTable` - Table metadata type
- `UserColumns::Id`, `UserColumns::Name`, etc. - Column types
- Accessor methods for type-safe queries

## Benefits

1. **Compile-time validation** - Misspelled column names fail to compile
2. **IDE support** - Autocomplete for column names
3. **Refactoring safety** - Renaming columns is checked by the compiler
4. **Self-documenting** - Schema is visible in your Rust code

## Usage

See the following guides:

- [Defining Tables](./tables) - How to define table schemas
- [Typed Queries](./queries) - Building queries with type safety
- [Column Attributes](./attributes) - Available column configurations
