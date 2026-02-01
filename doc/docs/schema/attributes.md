---
sidebar_position: 3
---

# Column Attributes

Configure column behavior using the `#[column]` attribute.

## Primary Key

Mark a column as the primary key:

```rust
use oxide_sql_derive::Table;

#[derive(Table)]
struct User {
    #[column(primary_key)]
    id: i32,
    name: String,
}
```

This sets `UserTable::PRIMARY_KEY` to `Some("id")`.

## Custom Column Name

Override the SQL column name (useful when Rust naming conventions differ):

```rust
use oxide_sql_derive::Table;

#[derive(Table)]
struct User {
    #[column(name = "user_id")]
    id: i32,

    #[column(name = "full_name")]
    name: String,
}
```

The generated SQL will use `user_id` and `full_name` instead of `id` and `name`.

## Nullable Columns

Mark a column as nullable:

```rust
use oxide_sql_derive::Table;

#[derive(Table)]
struct User {
    id: i32,
    name: String,

    #[column(nullable)]
    bio: Option<String>,
}
```

This sets `Column::NULLABLE` to `true` for the `bio` column.

## Combining Attributes

Attributes can be combined:

```rust
use oxide_sql_derive::Table;

#[derive(Table)]
#[table(name = "app_users")]
struct User {
    #[column(primary_key, name = "user_id")]
    id: i32,

    #[column(name = "display_name")]
    name: String,

    #[column(nullable, name = "profile_bio")]
    bio: Option<String>,
}
```

## Available Attributes

### Table Attributes

| Attribute | Description |
|-----------|-------------|
| `#[table(name = "...")]` | Custom SQL table name |

### Column Attributes

| Attribute | Description |
|-----------|-------------|
| `#[column(primary_key)]` | Mark as primary key |
| `#[column(name = "...")]` | Custom SQL column name |
| `#[column(nullable)]` | Mark as nullable |

## Generated Trait Implementations

Each column type implements the `Column` trait:

```rust
pub trait Column {
    type Table: Table;
    type Type;

    const NAME: &'static str;
    const NULLABLE: bool;
    const PRIMARY_KEY: bool;
}
```

For a column marked with `#[column(primary_key, nullable)]`:

- `NAME` = the SQL column name (or field name if not specified)
- `NULLABLE` = `true`
- `PRIMARY_KEY` = `true`
