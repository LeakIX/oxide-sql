---
sidebar_position: 2
---

# Typed Queries

Use `Select`, `Insert`, `Update`, and `Delete` (typed builders) for
compile-time validated queries against your schema.

## Features

- `Select` - Type-safe SELECT with compile-time column validation
- `Insert` - Type-safe INSERT with `.set()` API
- `Update` - Type-safe UPDATE with validated SET and WHERE columns
- `Delete` - Type-safe DELETE with validated WHERE columns
- `typed_col` for type-safe column references in WHERE clauses
- Compile-time rejection of invalid column names

## API Reference

See the
[typed builder module rustdoc](pathname:///oxide-sql/rustdoc/oxide_sql_core/builder/typed/) for the
full API with code examples covering SELECT, INSERT, UPDATE, and DELETE.
