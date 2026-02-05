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

### Type-Safe Query Building with Compile-Time Column Validation

Define your schema once with `#[derive(Table)]`, and get compile-time validation
of all column references:

```rust
use oxide_sql_derive::Table;
use oxide_sql_core::builder::{TypedSelect, TypedInsert, typed_col};

// Define your model - generates PostTable, PostColumns module, and accessors
#[derive(Debug, Clone, Table)]
#[table(name = "posts")]
pub struct Post {
    #[column(primary_key)]
    pub id: i64,
    pub title: String,
    pub status: String,
    pub created_at: String,
}

// Type-safe SELECT - invalid columns won't compile!
let (sql, _) = TypedSelect::<PostTable, _, _>::new()
    .select::<(PostColumns::Id, PostColumns::Title, PostColumns::Status)>()
    .from_table()
    .where_clause(
        typed_col(Post::status()).eq("published")
            .and(typed_col(Post::created_at()).gt("2024-01-01"))
    )
    .order_by(Post::created_at(), false)  // descending
    .limit(10)
    .build();
// Output: SELECT id, title, status FROM posts
//         WHERE status = ? AND created_at > ? ORDER BY created_at DESC LIMIT 10

// Type-safe INSERT - column names are validated at compile time
let (sql, _) = TypedInsert::<PostTable, _>::new()
    .set(Post::title(), "Hello World")
    .set(Post::status(), "draft")
    .set(Post::created_at(), "2024-01-15")
    .build();
// Output: INSERT INTO posts (title, status, created_at) VALUES (?, ?, ?)

// This would NOT compile - InvalidColumn doesn't exist on Post:
// TypedSelect::<PostTable, _, _>::new()
//     .select::<(PostColumns::InvalidColumn,)>()  // Compile error!
```

### Basic Query Building (String-Based)

For simpler cases without compile-time column checking:

```rust
use oxide_sql_core::builder::{Select, col};

let (sql, params) = Select::new()
    .columns(&["id", "name"])
    .from("users")
    .where_clause(col("active").eq(true))
    .build();

assert_eq!(sql, "SELECT id, name FROM users WHERE active = ?");

// This would NOT compile - missing FROM clause:
// let query = Select::new()
//     .columns(&["id", "name"])
//     .build();  // Error: method `build` not found
```

### SQL Injection Prevention

User input is always parameterized, never interpolated:

```rust
use oxide_sql_core::builder::{Select, col};

let user_input = "'; DROP TABLE users; --";
let (sql, params) = Select::new()
    .columns(&["id"])
    .from("users")
    .where_clause(col("name").eq(user_input))
    .build();

// sql = "SELECT id FROM users WHERE name = ?"
// The malicious input is safely stored as a parameter
```

### SQLite UPSERT

```rust
use oxide_sql_sqlite::builder::Upsert;
use oxide_sql_core::builder::col;

let (sql, params) = Upsert::new()
    .into_table("users")
    .columns(&["id", "name", "email"])
    .values(&[&1_i32, &"Alice", &"alice@example.com"])
    .on_conflict(&["id"])
    .do_update(&["name", "email"])
    .build();
```

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
- [API Reference](https://docs.rs/oxide-sql-core)

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
