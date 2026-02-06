//! Dynamic DELETE statement builder using the typestate pattern.
//!
//! This module provides string-based query building. For compile-time
//! validated queries using schema traits, use `Delete` from `builder::typed`.

use std::marker::PhantomData;

use super::expr::ExprBuilder;
use super::value::SqlValue;

// Typestate markers

/// Marker: No table specified yet.
pub struct NoTable;
/// Marker: Table has been specified.
pub struct HasTable;

/// A dynamic DELETE statement builder using string-based column names.
///
/// For compile-time validated queries, use `Delete` from `builder::typed`.
///
/// Uses the typestate pattern to ensure that:
/// - `build()` is only available when table is specified
/// - `where_clause()` is only available after table is specified
pub struct DeleteDyn<Table> {
    table: Option<String>,
    where_clause: Option<ExprBuilder>,
    _state: PhantomData<Table>,
}

impl DeleteDyn<NoTable> {
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

impl Default for DeleteDyn<NoTable> {
    fn default() -> Self {
        Self::new()
    }
}

// Transition: NoTable -> HasTable
impl DeleteDyn<NoTable> {
    /// Specifies the table to delete from.
    #[must_use]
    pub fn from(self, table: &str) -> DeleteDyn<HasTable> {
        DeleteDyn {
            table: Some(String::from(table)),
            where_clause: self.where_clause,
            _state: PhantomData,
        }
    }
}

// Methods available after FROM
impl DeleteDyn<HasTable> {
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
pub struct SafeDeleteDyn<Table> {
    inner: DeleteDyn<Table>,
}

impl SafeDeleteDyn<NoTable> {
    /// Creates a new safe DELETE builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: DeleteDyn::new(),
        }
    }

    /// Specifies the table to delete from.
    #[must_use]
    pub fn from(self, table: &str) -> SafeDeleteDyn<HasTable> {
        SafeDeleteDyn {
            inner: self.inner.from(table),
        }
    }
}

impl Default for SafeDeleteDyn<NoTable> {
    fn default() -> Self {
        Self::new()
    }
}

// Safe DELETE requires WHERE before build
pub struct SafeDeleteDynWithWhere {
    inner: DeleteDyn<HasTable>,
}

impl SafeDeleteDyn<HasTable> {
    /// Adds a WHERE clause (required for SafeDeleteDyn).
    #[must_use]
    pub fn where_clause(self, expr: ExprBuilder) -> SafeDeleteDynWithWhere {
        SafeDeleteDynWithWhere {
            inner: self.inner.where_clause(expr),
        }
    }
}

impl SafeDeleteDynWithWhere {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::dyn_col;

    #[test]
    fn test_simple_delete() {
        let (sql, params) = DeleteDyn::new()
            .from("users")
            .where_clause(dyn_col("id").eq(1_i32))
            .build();

        assert_eq!(sql, "DELETE FROM users WHERE id = ?");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_delete_all() {
        let (sql, params) = DeleteDyn::new().from("temp_data").build();

        assert_eq!(sql, "DELETE FROM temp_data");
        assert!(params.is_empty());
    }

    #[test]
    fn test_delete_complex_where() {
        let (sql, params) = DeleteDyn::new()
            .from("orders")
            .where_clause(
                dyn_col("status")
                    .eq("cancelled")
                    .and(dyn_col("created_at").lt("2024-01-01")),
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
        let (sql, params) = SafeDeleteDyn::new()
            .from("users")
            .where_clause(dyn_col("id").eq(1_i32))
            .build();

        assert_eq!(sql, "DELETE FROM users WHERE id = ?");
        assert_eq!(params.len(), 1);
    }

    // This would fail to compile: SafeDeleteDyn without WHERE
    // #[test]
    // fn test_safe_delete_without_where_fails() {
    //     let _ = SafeDeleteDyn::new()
    //         .from("users")
    //         .build();  // Error: method `build` not found
    // }

    #[test]
    fn test_delete_sql_injection_prevention() {
        let malicious = "1; DROP TABLE users; --";
        let (sql, params) = DeleteDyn::new()
            .from("users")
            .where_clause(dyn_col("id").eq(malicious))
            .build();

        assert_eq!(sql, "DELETE FROM users WHERE id = ?");
        assert!(matches!(&params[0], SqlValue::Text(s) if s == malicious));
    }
}
