---
sidebar_position: 3
---

# Type-Safe Schema Definitions

Oxide SQL provides derive macros for defining database tables as Rust structs.
This enables compile-time validation of column names, preventing typos and
ensuring queries are valid before runtime.

## Overview

Instead of using string column names, you define your schema as Rust structs
with `#[derive(Table)]`. The macro generates table metadata types, column
types, and accessor methods for type-safe queries.

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

## API Reference

See the [derive macro rustdoc](pathname:///oxide-sql/rustdoc/oxide_sql_derive/) for macro
documentation and the
[typed builder module rustdoc](pathname:///oxide-sql/rustdoc/oxide_sql_core/builder/typed/) for the
typed query builders.
