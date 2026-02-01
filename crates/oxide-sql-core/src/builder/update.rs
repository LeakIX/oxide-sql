//! Type-safe UPDATE statement builder using the typestate pattern.

#[cfg(feature = "alloc")]
use alloc::{format, string::String, vec, vec::Vec};

use core::marker::PhantomData;

use super::expr::ExprBuilder;
use super::value::{SqlValue, ToSqlValue};

// Typestate markers

/// Marker: No table specified yet.
pub struct NoTable;
/// Marker: Table has been specified.
pub struct HasTable;
/// Marker: No SET clause specified yet.
pub struct NoSet;
/// Marker: SET clause has been specified.
pub struct HasSet;

/// An assignment in the SET clause.
#[cfg(feature = "alloc")]
struct Assignment {
    column: String,
    value: SqlValue,
}

/// A type-safe UPDATE statement builder.
#[cfg(feature = "alloc")]
pub struct Update<Table, Set> {
    table: Option<String>,
    assignments: Vec<Assignment>,
    where_clause: Option<ExprBuilder>,
    _state: PhantomData<(Table, Set)>,
}

#[cfg(feature = "alloc")]
impl Update<NoTable, NoSet> {
    /// Creates a new UPDATE builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            table: None,
            assignments: vec![],
            where_clause: None,
            _state: PhantomData,
        }
    }
}

#[cfg(feature = "alloc")]
impl Default for Update<NoTable, NoSet> {
    fn default() -> Self {
        Self::new()
    }
}

// Transition: NoTable -> HasTable
#[cfg(feature = "alloc")]
impl<Set> Update<NoTable, Set> {
    /// Specifies the table to update.
    #[must_use]
    pub fn table(self, table: &str) -> Update<HasTable, Set> {
        Update {
            table: Some(String::from(table)),
            assignments: self.assignments,
            where_clause: self.where_clause,
            _state: PhantomData,
        }
    }
}

// Transition: NoSet -> HasSet (requires table)
#[cfg(feature = "alloc")]
impl Update<HasTable, NoSet> {
    /// Adds a SET assignment.
    #[must_use]
    pub fn set<T: ToSqlValue>(self, column: &str, value: T) -> Update<HasTable, HasSet> {
        Update {
            table: self.table,
            assignments: vec![Assignment {
                column: String::from(column),
                value: value.to_sql_value(),
            }],
            where_clause: self.where_clause,
            _state: PhantomData,
        }
    }
}

// Methods available after SET
#[cfg(feature = "alloc")]
impl Update<HasTable, HasSet> {
    /// Adds another SET assignment.
    #[must_use]
    pub fn set<T: ToSqlValue>(mut self, column: &str, value: T) -> Self {
        self.assignments.push(Assignment {
            column: String::from(column),
            value: value.to_sql_value(),
        });
        self
    }

    /// Adds a WHERE clause.
    #[must_use]
    pub fn where_clause(mut self, expr: ExprBuilder) -> Self {
        self.where_clause = Some(expr);
        self
    }

    /// Builds the UPDATE statement and returns SQL with parameters.
    #[must_use]
    pub fn build(self) -> (String, Vec<SqlValue>) {
        let mut sql = String::from("UPDATE ");
        let mut params = vec![];

        if let Some(ref table) = self.table {
            sql.push_str(table);
        }

        sql.push_str(" SET ");

        let set_parts: Vec<String> = self
            .assignments
            .iter()
            .map(|a| format!("{} = ?", a.column))
            .collect();
        sql.push_str(&set_parts.join(", "));

        for assignment in self.assignments {
            params.push(assignment.value);
        }

        if let Some(ref where_expr) = self.where_clause {
            sql.push_str(" WHERE ");
            sql.push_str(where_expr.sql());
            params.extend(where_expr.params().iter().cloned());
        }

        (sql, params)
    }

    /// Builds the UPDATE statement and returns only the SQL string.
    #[must_use]
    pub fn build_sql(self) -> String {
        let (sql, _) = self.build();
        sql
    }
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;
    use crate::builder::col;

    #[test]
    fn test_simple_update() {
        let (sql, params) = Update::new().table("users").set("name", "Bob").build();

        assert_eq!(sql, "UPDATE users SET name = ?");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_update_multiple_columns() {
        let (sql, params) = Update::new()
            .table("users")
            .set("name", "Bob")
            .set("email", "bob@example.com")
            .set("age", 30_i32)
            .build();

        assert_eq!(sql, "UPDATE users SET name = ?, email = ?, age = ?");
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn test_update_with_where() {
        let (sql, params) = Update::new()
            .table("users")
            .set("active", false)
            .where_clause(col("id").eq(1_i32))
            .build();

        assert_eq!(sql, "UPDATE users SET active = ? WHERE id = ?");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_update_sql_injection_prevention() {
        let malicious = "'; DROP TABLE users; --";
        let (sql, params) = Update::new()
            .table("users")
            .set("name", malicious)
            .where_clause(col("id").eq(1_i32))
            .build();

        assert_eq!(sql, "UPDATE users SET name = ? WHERE id = ?");
        assert!(matches!(&params[0], SqlValue::Text(s) if s == malicious));
    }
}
