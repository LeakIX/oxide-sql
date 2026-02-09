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
- **SQLite Extensions**: SQLite-specific syntax like UPSERT

## Quick Start

Add the dependencies to your `Cargo.toml`:

```toml
[dependencies]
oxide-sql-core = "0.1"
oxide-sql-sqlite = "0.1"  # Optional, for SQLite-specific features
```

## Why Oxide SQL?

1. **Compile-Time Safety**: Catch SQL syntax errors before runtime
2. **Security First**: SQL injection is prevented by design
3. **Zero Runtime Overhead**: Type states are zero-sized types
4. **Extensible**: Support for database-specific dialects

## API Reference

See the [crate overview rustdoc](pathname:///oxide-sql/rustdoc/oxide_sql_core/) for the full API
documentation with code examples.

## Documentation

- [Getting Started](./getting-started) - Installation and basic usage
- [SQL Builders](./builders/) - SELECT, INSERT, UPDATE, DELETE builders
- [Type-Safe Schema](./schema/) - Define tables with derive macros
- [SQL Security Guide](./security/) - SQL injection and prevention

## Web Framework

Looking for ORM, admin interface, authentication, forms, routing, and
migrations? See [Corrode](https://github.com/LeakIX/corrode), a Django-like
web framework for Rust built on oxide-sql.
