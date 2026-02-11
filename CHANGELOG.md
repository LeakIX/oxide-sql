# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

### Added

- Add `Display` impls for all AST types with round-trip assertions ([#15])

[#15]: https://github.com/LeakIX/oxide-sql/issues/15

## 0.1.0

### Added

- Initial release of oxide-sql workspace
- `oxide-sql-core`: type-safe SQL parser and query builder with
  compile-time validation
- `oxide-sql-derive`: derive macros for type-safe SQL table definitions
- `oxide-sql-sqlite`: SQLite-specific SQL extensions (upsert, etc.)
