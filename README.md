# Oxide SQL

A type-safe SQL parser and builder for Rust with compile-time validation and
SQL injection prevention.

## Features

- **Type-Safe SQL Building**: Invalid SQL constructs are caught at compile time
  using the typestate pattern
- **SQL Injection Prevention**: All user input is automatically parameterized
- **Hand-Written Parser**: Recursive descent parser with Pratt expression
  parsing
- **SQLite Extensions**: SQLite-specific syntax like UPSERT

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
oxide-sql-core = "0.1"
oxide-sql-sqlite = "0.1"  # Optional, for SQLite-specific features
```

## Quick Start

See the [API reference](https://leakix.github.io/oxide-sql/rustdoc/oxide_sql_core/)
for complete examples with compile-time validation. Key modules:

- [`oxide_sql_core::builder`](https://leakix.github.io/oxide-sql/rustdoc/oxide_sql_core/builder/)
  -- type-safe and dynamic query builders
- [`oxide_sql_core::builder::typed`](https://leakix.github.io/oxide-sql/rustdoc/oxide_sql_core/builder/typed/)
  -- compile-time column validation with `#[derive(Table)]`
- [`oxide_sql_sqlite::builder`](https://leakix.github.io/oxide-sql/rustdoc/oxide_sql_sqlite/builder/)
  -- SQLite-specific extensions (UPSERT)

## Crates

- **oxide-sql-core**: Core parser and type-safe builders
- **oxide-sql-sqlite**: SQLite-specific extensions
- **oxide-sql-derive**: Derive macros for type-safe tables

## Web Framework

Looking for ORM, admin interface, authentication, forms, routing, and
migrations? See [Corrode](https://github.com/LeakIX/corrode), a Django-like
web framework for Rust built on oxide-sql.

## Documentation

- [Online Documentation](https://leakix.github.io/oxide-sql/)
- [API Reference (rustdoc)](https://leakix.github.io/oxide-sql/rustdoc/oxide_sql_core/)

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
