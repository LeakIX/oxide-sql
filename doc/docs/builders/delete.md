---
sidebar_position: 4
---

# DELETE Builder

The `DeleteDyn` builder creates DELETE statements. Oxide SQL provides both a
standard builder and a safe variant that requires a WHERE clause.

## Features

- Standard `DeleteDyn` builder for general use
- `SafeDelete` builder that **requires** a WHERE clause at compile time
- Complex WHERE conditions
- All values are automatically parameterized

## When to Use Each Builder

| Builder | Use Case |
|---------|----------|
| `DeleteDyn` | When you might intentionally delete all rows (e.g., clearing temp tables) |
| `SafeDelete` | For most cases - prevents accidental data loss |

## API Reference

See the [`DeleteDyn` rustdoc](pathname:///oxide-sql/rustdoc/oxide_sql_core/builder/delete/) for the
full API with code examples.
