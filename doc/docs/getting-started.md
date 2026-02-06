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

The simplest way to use Oxide SQL is with the string-based builder. See the
[builder module rustdoc](pathname:///oxide-sql/rustdoc/oxide_sql_core/builder/) for examples.

### Type-Safe Queries with Derive Macros

For compile-time validation of column names, use the derive macro. See the
[typed builder module rustdoc](pathname:///oxide-sql/rustdoc/oxide_sql_core/builder/typed/) for
examples.

## SQL Injection Prevention

Oxide SQL automatically parameterizes all user input. Even malicious input is
safely parameterized -- the SQL structure is fixed at compile time and user
input can never modify the query structure.

## Compile-Time Safety

The typestate pattern ensures that invalid SQL cannot be constructed. For
example, a SELECT without a FROM clause will not compile.

## API Reference

See the [crate overview rustdoc](pathname:///oxide-sql/rustdoc/oxide_sql_core/) for the full API
documentation with code examples.

## Next Steps

- Learn about the [Builder API](./builders/)
- Explore [Type-Safe Schema Definitions](./schema/)
- Read about [SQL Security](./security/)
