---
sidebar_position: 3
---

# Column Attributes

Configure column behavior using the `#[column]` attribute.

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

Attributes can be combined, e.g.
`#[column(primary_key, name = "user_id")]`.

## Generated Trait Implementations

Each column type implements the `Column` trait with associated constants:

- `NAME` - the SQL column name (or field name if not specified)
- `NULLABLE` - whether the column is nullable
- `PRIMARY_KEY` - whether the column is a primary key

## API Reference

See the [`#[derive(Table)]` rustdoc](pathname:///oxide-sql/rustdoc/oxide_sql_derive/) for the full
attribute documentation with code examples.
