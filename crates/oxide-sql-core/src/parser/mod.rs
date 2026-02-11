//! SQL Parser
//!
//! A hand-written recursive descent parser with Pratt expression
//! parsing for a subset of SQL:2016 (ISO/IEC 9075) covering DML/DQL
//! operations.
//!
//! # Parsing approach
//!
//! Statements (`SELECT`, `INSERT`, `UPDATE`, `DELETE`) are parsed by
//! dedicated recursive-descent methods. Expressions use a Pratt
//! (top-down operator precedence) parser that handles prefix, infix,
//! and postfix operators with correct precedence and associativity.
//!
//! # Supported statements
//!
//! | Statement | Notes |
//! |-----------|-------|
//! | `SELECT`  | Full DQL with all clauses listed below |
//! | `INSERT`  | `VALUES`, `DEFAULT VALUES`, sub-`SELECT`, `ON CONFLICT` |
//! | `UPDATE`  | `SET`, optional `FROM`, optional alias |
//! | `DELETE`  | Optional alias, `WHERE` |
//!
//! # SELECT clauses
//!
//! `DISTINCT` / `ALL`, column list with aliases, `FROM` (table,
//! schema-qualified table, subquery, aliases), `WHERE`, `GROUP BY`,
//! `HAVING`, `ORDER BY` (with `ASC` / `DESC` and
//! `NULLS FIRST` / `NULLS LAST`), `LIMIT`, `OFFSET`.
//!
//! # JOINs
//!
//! `INNER`, `LEFT [OUTER]`, `RIGHT [OUTER]`, `FULL [OUTER]`,
//! `CROSS`, with `ON` or `USING` conditions. Chained (multi-table)
//! joins are left-associative.
//!
//! # Expressions
//!
//! - **Literals**: integers, floats, strings, blobs (`X'…'`),
//!   booleans (`TRUE`/`FALSE`), `NULL`
//! - **Column references**: unqualified (`col`), qualified (`t.col`),
//!   wildcards (`*`, `t.*`)
//! - **Binary operators**: `+`, `-`, `*`, `/`, `%`, `||`, `&`, `|`,
//!   `<<`, `>>`, `=`, `!=`/`<>`, `<`, `<=`, `>`, `>=`, `AND`, `OR`,
//!   `LIKE`
//! - **Unary operators**: `-` (negate), `NOT`, `~` (bitwise NOT)
//! - **Special forms**: `IS [NOT] NULL`, `BETWEEN … AND …`,
//!   `IN (…)`, `CASE`/`WHEN`/`THEN`/`ELSE`/`END`,
//!   `CAST(… AS <type>)`, `EXISTS(…)`
//! - **Function calls**: named functions with optional `DISTINCT`
//!   (e.g. `COUNT(DISTINCT col)`)
//! - **Subqueries**: scalar `(SELECT …)` in expressions
//! - **Parameters**: positional (`?`) and named (`:name`)
//!
//! # Data types (via CAST)
//!
//! `SMALLINT`, `INTEGER`/`INT`, `BIGINT`, `REAL`, `DOUBLE`/`FLOAT`,
//! `DECIMAL(p, s)`, `NUMERIC(p, s)`, `CHAR(n)`, `VARCHAR(n)`,
//! `TEXT`, `BLOB`, `BINARY(n)`, `VARBINARY(n)`, `DATE`, `TIME`,
//! `TIMESTAMP`, `DATETIME`, `BOOLEAN`.
//!
//! # INSERT extensions
//!
//! `ON CONFLICT DO NOTHING` and `ON CONFLICT DO UPDATE SET …` for
//! upsert semantics.
//!
//! # Not supported
//!
//! DDL (`CREATE` / `ALTER` / `DROP`), transactions
//! (`BEGIN` / `COMMIT` / `ROLLBACK`), set operations
//! (`UNION` / `INTERSECT` / `EXCEPT`), window functions
//! (`OVER` / `PARTITION BY`), common table expressions (`WITH`),
//! `NATURAL JOIN`.

mod core;
mod error;
mod pratt;

pub use core::Parser;
pub use error::ParseError;
