---
sidebar_position: 2
---

# SQL Builders

Oxide SQL provides type-safe builders for common SQL statements. Each builder
uses the typestate pattern to ensure valid SQL at compile time.

## Available Builders

- [SELECT](./select) - Query data from tables
- [INSERT](./insert) - Insert new rows
- [UPDATE](./update) - Modify existing rows
- [DELETE](./delete) - Remove rows

## Typed vs String-Based Builders

Oxide SQL offers two levels of type safety:

### String-Based Builders (This Section)

Use column names as strings. Good for dynamic queries or quick prototyping.

### Typed Builders (Recommended)

Use `#[derive(Table)]` for **compile-time column validation**. Invalid column
names won't compile.

See [Schema > Typed Queries](../schema/queries) for full typed builder
documentation.

## The Typestate Pattern

All builders use Rust's type system to enforce SQL validity at compile time.
This means:

1. **Required clauses are enforced** - You can't build a SELECT without FROM
2. **Order is enforced** - WHERE must come after FROM
3. **Invalid combinations fail to compile** - No runtime errors for SQL syntax

## Expressions

All builders use the same expression system for WHERE clauses, supporting
comparisons, null checks, range checks, and logical operators.

## Parameterized Queries

All values are automatically parameterized to prevent SQL injection. The
`params` vector should be passed to your database driver for safe execution.

## API Reference

See the [builder module rustdoc](pathname:///oxide-sql/rustdoc/oxide_sql_core/builder/) for the
dynamic builders and the
[typed builder module rustdoc](pathname:///oxide-sql/rustdoc/oxide_sql_core/builder/typed/) for
compile-time validated builders with full code examples.
