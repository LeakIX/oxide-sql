//! Type-safe DELETE statement builder using the typestate pattern.

#[cfg(feature = "alloc")]
use alloc::{string::String, vec, vec::Vec};

use core::marker::PhantomData;

use super::expr::ExprBuilder;
use super::value::SqlValue;

// Typestate markers

/// Marker: No table specified yet.
pub struct NoTable;
/// Marker: Table has been specified.
pub struct HasTable;

/// A type-safe DELETE statement builder.
///
/// Uses the typestate pattern to ensure that:
/// - `build()` is only available when table is specified
/// - `where_clause()` is only available after table is specified
#[cfg(feature = "alloc")]
pub struct Delete<Table> {
    table: Option<String>,
    where_clause: Option<ExprBuilder>,
    _state: PhantomData<Table>,
}

#[cfg(feature = "alloc")]
impl Delete<NoTable> {
    /// Creates a new DELETE builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            table: None,
            where_clause: None,
            _state: PhantomData,
        }
    }
}

#[cfg(feature = "alloc")]
impl Default for Delete<NoTable> {
    fn default() -> Self {
        Self::new()
    }
}

// Transition: NoTable -> HasTable
#[cfg(feature = "alloc")]
impl Delete<NoTable> {
    /// Specifies the table to delete from.
    #[must_use]
    pub fn from(self, table: &str) -> Delete<HasTable> {
        Delete {
            table: Some(String::from(table)),
            where_clause: self.where_clause,
            _state: PhantomData,
        }
    }
}

// Methods available after FROM
#[cfg(feature = "alloc")]
impl Delete<HasTable> {
    /// Adds a WHERE clause.
    ///
    /// **Important**: DELETE without WHERE deletes all rows!
    /// Consider using `where_clause_required()` for safety.
    #[must_use]
    pub fn where_clause(mut self, expr: ExprBuilder) -> Self {
        self.where_clause = Some(expr);
        self
    }

    /// Builds the DELETE statement and returns SQL with parameters.
    ///
    /// **Warning**: If no WHERE clause is specified, this will delete ALL rows.
    #[must_use]
    pub fn build(self) -> (String, Vec<SqlValue>) {
        let mut sql = String::from("DELETE FROM ");
        let mut params = vec![];

        if let Some(ref table) = self.table {
            sql.push_str(table);
        }

        if let Some(ref where_expr) = self.where_clause {
            sql.push_str(" WHERE ");
            sql.push_str(where_expr.sql());
            params.extend(where_expr.params().iter().cloned());
        }

        (sql, params)
    }

    /// Builds the DELETE statement and returns only the SQL string.
    #[must_use]
    pub fn build_sql(self) -> String {
        let (sql, _) = self.build();
        sql
    }

    /// Returns true if a WHERE clause is specified.
    #[must_use]
    pub const fn has_where_clause(&self) -> bool {
        self.where_clause.is_some()
    }
}

/// A safe DELETE builder that requires a WHERE clause.
///
/// This prevents accidental deletion of all rows.
#[cfg(feature = "alloc")]
pub struct SafeDelete<Table> {
    inner: Delete<Table>,
}

#[cfg(feature = "alloc")]
impl SafeDelete<NoTable> {
    /// Creates a new safe DELETE builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Delete::new(),
        }
    }

    /// Specifies the table to delete from.
    #[must_use]
    pub fn from(self, table: &str) -> SafeDelete<HasTable> {
        SafeDelete {
            inner: self.inner.from(table),
        }
    }
}

#[cfg(feature = "alloc")]
impl Default for SafeDelete<NoTable> {
    fn default() -> Self {
        Self::new()
    }
}

// Safe DELETE requires WHERE before build
#[cfg(feature = "alloc")]
pub struct SafeDeleteWithWhere {
    inner: Delete<HasTable>,
}

#[cfg(feature = "alloc")]
impl SafeDelete<HasTable> {
    /// Adds a WHERE clause (required for SafeDelete).
    #[must_use]
    pub fn where_clause(self, expr: ExprBuilder) -> SafeDeleteWithWhere {
        SafeDeleteWithWhere {
            inner: self.inner.where_clause(expr),
        }
    }
}

#[cfg(feature = "alloc")]
impl SafeDeleteWithWhere {
    /// Builds the DELETE statement.
    #[must_use]
    pub fn build(self) -> (String, Vec<SqlValue>) {
        self.inner.build()
    }

    /// Builds the DELETE statement and returns only the SQL string.
    #[must_use]
    pub fn build_sql(self) -> String {
        self.inner.build_sql()
    }
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;
    use crate::builder::col;

    #[test]
    fn test_simple_delete() {
        let (sql, params) = Delete::new()
            .from("users")
            .where_clause(col("id").eq(1_i32))
            .build();

        assert_eq!(sql, "DELETE FROM users WHERE id = ?");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_delete_all() {
        let (sql, params) = Delete::new().from("temp_data").build();

        assert_eq!(sql, "DELETE FROM temp_data");
        assert!(params.is_empty());
    }

    #[test]
    fn test_delete_complex_where() {
        let (sql, params) = Delete::new()
            .from("orders")
            .where_clause(
                col("status")
                    .eq("cancelled")
                    .and(col("created_at").lt("2024-01-01")),
            )
            .build();

        assert_eq!(
            sql,
            "DELETE FROM orders WHERE status = ? AND created_at < ?"
        );
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_safe_delete() {
        let (sql, params) = SafeDelete::new()
            .from("users")
            .where_clause(col("id").eq(1_i32))
            .build();

        assert_eq!(sql, "DELETE FROM users WHERE id = ?");
        assert_eq!(params.len(), 1);
    }

    // This would fail to compile: SafeDelete without WHERE
    // #[test]
    // fn test_safe_delete_without_where_fails() {
    //     let _ = SafeDelete::new()
    //         .from("users")
    //         .build();  // Error: method `build` not found
    // }

    #[test]
    fn test_delete_sql_injection_prevention() {
        let malicious = "1; DROP TABLE users; --";
        let (sql, params) = Delete::new()
            .from("users")
            .where_clause(col("id").eq(malicious))
            .build();

        assert_eq!(sql, "DELETE FROM users WHERE id = ?");
        assert!(matches!(&params[0], SqlValue::Text(s) if s == malicious));
    }
}
