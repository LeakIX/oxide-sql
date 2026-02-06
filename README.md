# Oxide SQL

A type-safe SQL parser and builder for Rust with compile-time validation, SQL
injection prevention, and Django-like admin interface.

## Features

- **Type-Safe SQL Building**: Invalid SQL constructs are caught at compile time
  using the typestate pattern
- **SQL Injection Prevention**: All user input is automatically parameterized
- **Django-like ORM**: Familiar QuerySet API, Model trait, and Managers
- **Admin Interface**: Automatic CRUD admin with TailwindCSS UI
- **Database Migrations**: Django-style migrations with auto-detection
- **Authentication**: User management, sessions, and permissions
- **Hand-Written Parser**: Recursive descent parser with Pratt expression
  parsing
- **SQLite Extensions**: SQLite-specific syntax like UPSERT

## Try the Admin Interface

Run the blog admin example to see the admin interface in action:

```bash
cargo run -p oxide-admin --example blog_admin
```

Then open http://localhost:3000/admin/ and login with `admin` / `admin123`.

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
- **oxide-orm**: Django-like ORM with QuerySet, Manager, and Model
- **oxide-migrate**: Database migrations with auto-detection
- **oxide-auth**: Authentication, sessions, and permissions
- **oxide-forms**: Form validation and rendering
- **oxide-router**: HTTP routing for web applications
- **oxide-admin**: Django-like admin interface

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
make example-blog   # Run blog admin example
make e2e-install    # Install E2E test dependencies
make e2e-test       # Run E2E tests
```

## License

MIT
