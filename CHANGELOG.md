# Changelog

All notable changes to this project will be documented in this file.

## 0.2.0

### Added

- Add `Display` impls for all AST types with round-trip assertions
  ([ba9e0eb], [#15])
- Add DuckDB migration dialect with sequence-backed autoincrement
  ([5c0ceab])
- Add DuckDB e2e integration tests with in-memory database ([4eb4de9])
- Add comprehensive SQL parser integration tests — 154 tests across 12
  focused test files ([6bb2e2a], [a60c960])
- Add `derive(Table)` tutorial to oxide-sql-core rustdoc ([f20b841])
- Add dialect-aware `CreateTableOp::from_table` from `derive(Table)`
  structs ([8279ca7])
- Add schema diff engine for auto-migration generation with
  `auto_diff_table` and `auto_diff_schema` ([80d4128])
- Add `SetUnique(bool)` and `SetAutoincrement(bool)` to
  `AlterColumnChange` with dialect-specific SQL generation ([119564b])
- Add N:M rename heuristic using Levenshtein similarity scoring for
  column and table renames ([119564b])
- Add `IndexSnapshot` and `ForeignKeySnapshot` types with index and
  foreign key diff support ([119564b])
- Add `SchemaDiff::to_sql()` convenience method for generating SQL from
  diffs ([119564b])
- Add migration code generation (`codegen.rs`) with
  `generate_migration_code()` producing complete `Migration` trait impls
  ([119564b])
- Add SQLite introspection helpers (`sqlite_helpers` module) with PRAGMA
  constants, type affinity parsing, and `column_from_pragma()` ([119564b])
- Add reversible diffs with `SchemaDiff::reverse()`,
  `is_reversible()`, and `non_reversible_operations()` ([119564b])
- Add `DiffWarning` enum for column ordering detection
  (`ColumnOrderChanged`), primary key changes, and autoincrement changes
  ([119564b])

### Changed

- **BREAKING**: Upgrade to Rust edition 2024 — requires a Rust toolchain
  that supports edition 2024 ([4034f8b])
- **BREAKING**: `TableSnapshot` struct gained two required fields:
  `indexes: Vec<IndexSnapshot>` and `foreign_keys: Vec<ForeignKeySnapshot>`.
  Add `indexes: vec![], foreign_keys: vec![]` to existing struct literals
  ([119564b])
- **BREAKING**: `SchemaDiff` struct gained a required `warnings:
  Vec<DiffWarning>` field. Add `warnings: vec![]` to existing struct
  literals ([119564b])
- **BREAKING**: `AmbiguousChange::PossibleRename` and
  `PossibleTableRename` variants gained a `similarity: f64` field.
  Update pattern matches to include `similarity` or use `..` ([119564b])
- **BREAKING**: `AlterColumnChange` enum gained `SetUnique(bool)` and
  `SetAutoincrement(bool)` variants. Exhaustive match statements must
  handle the new variants ([119564b])
- **BREAKING**: `SchemaDiff::is_empty()` now also returns `false` when
  `warnings` is non-empty ([119564b])
- Rename `parser/parser.rs` to `parser/core.rs` with module docs
  ([d359403])

### Fixed

- Fix typed `Select`/`Update`/`Delete` to propagate WHERE params
  ([c330464])

<!-- Commit links -->
[119564b]: https://github.com/LeakIX/oxide-sql/commit/119564b
[80d4128]: https://github.com/LeakIX/oxide-sql/commit/80d4128
[8279ca7]: https://github.com/LeakIX/oxide-sql/commit/8279ca7
[a135dde]: https://github.com/LeakIX/oxide-sql/commit/a135dde
[4034f8b]: https://github.com/LeakIX/oxide-sql/commit/4034f8b
[a60c960]: https://github.com/LeakIX/oxide-sql/commit/a60c960
[d359403]: https://github.com/LeakIX/oxide-sql/commit/d359403
[ba9e0eb]: https://github.com/LeakIX/oxide-sql/commit/ba9e0eb
[6bb2e2a]: https://github.com/LeakIX/oxide-sql/commit/6bb2e2a
[f20b841]: https://github.com/LeakIX/oxide-sql/commit/f20b841
[4eb4de9]: https://github.com/LeakIX/oxide-sql/commit/4eb4de9
[c330464]: https://github.com/LeakIX/oxide-sql/commit/c330464
[5c0ceab]: https://github.com/LeakIX/oxide-sql/commit/5c0ceab

<!-- PR/Issue links -->
[#15]: https://github.com/LeakIX/oxide-sql/issues/15

## 0.1.0

### Added

- Initial release of oxide-sql workspace
- `oxide-sql-core`: type-safe SQL parser and query builder with
  compile-time validation
- `oxide-sql-derive`: derive macros for type-safe SQL table definitions
- `oxide-sql-sqlite`: SQLite-specific SQL extensions (upsert, etc.)
