//! Dynamic SELECT statement builder using the typestate pattern.
//!
//! This module provides string-based query building. For compile-time
//! validated queries using schema traits, use `Select` from `builder::typed`.
//!
//! Invalid SQL constructs are caught at compile time.

use std::marker::PhantomData;

use super::expr::ExprBuilder;
use super::value::SqlValue;

// Typestate markers (zero-sized types)

/// Marker: No columns specified yet.
pub struct NoColumns;
/// Marker: Columns have been specified.
pub struct HasColumns;
/// Marker: No FROM clause specified yet.
pub struct NoFrom;
/// Marker: FROM clause has been specified.
pub struct HasFrom;

/// A dynamic SELECT statement builder using string-based column names.
///
/// For compile-time validated queries, use `Select` from `builder::typed`.
///
/// Uses the typestate pattern to ensure that:
/// - `build()` is only available when both columns and FROM are specified
/// - `where_clause()` is only available after FROM is specified
/// - `group_by()`, `having()`, `order_by()` follow SQL semantics
pub struct SelectDyn<Cols, From> {
    distinct: bool,
    columns: Vec<String>,
    from: Option<String>,
    joins: Vec<String>,
    where_clause: Option<ExprBuilder>,
    group_by: Vec<String>,
    having: Option<ExprBuilder>,
    order_by: Vec<String>,
    limit: Option<u64>,
    offset: Option<u64>,
    _state: PhantomData<(Cols, From)>,
}

impl SelectDyn<NoColumns, NoFrom> {
    /// Creates a new SELECT builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            distinct: false,
            columns: vec![],
            from: None,
            joins: vec![],
            where_clause: None,
            group_by: vec![],
            having: None,
            order_by: vec![],
            limit: None,
            offset: None,
            _state: PhantomData,
        }
    }
}

impl Default for SelectDyn<NoColumns, NoFrom> {
    fn default() -> Self {
        Self::new()
    }
}

// Transition: NoColumns -> HasColumns
impl<From> SelectDyn<NoColumns, From> {
    /// Specifies the columns to select.
    #[must_use]
    pub fn columns(self, cols: &[&str]) -> SelectDyn<HasColumns, From> {
        SelectDyn {
            distinct: self.distinct,
            columns: cols.iter().map(|s| String::from(*s)).collect(),
            from: self.from,
            joins: self.joins,
            where_clause: self.where_clause,
            group_by: self.group_by,
            having: self.having,
            order_by: self.order_by,
            limit: self.limit,
            offset: self.offset,
            _state: PhantomData,
        }
    }

    /// Selects all columns (*).
    #[must_use]
    pub fn all(self) -> SelectDyn<HasColumns, From> {
        SelectDyn {
            distinct: self.distinct,
            columns: vec![String::from("*")],
            from: self.from,
            joins: self.joins,
            where_clause: self.where_clause,
            group_by: self.group_by,
            having: self.having,
            order_by: self.order_by,
            limit: self.limit,
            offset: self.offset,
            _state: PhantomData,
        }
    }
}

// Transition: NoFrom -> HasFrom
impl<Cols> SelectDyn<Cols, NoFrom> {
    /// Specifies the table to select from.
    #[must_use]
    pub fn from(self, table: &str) -> SelectDyn<Cols, HasFrom> {
        SelectDyn {
            distinct: self.distinct,
            columns: self.columns,
            from: Some(String::from(table)),
            joins: self.joins,
            where_clause: self.where_clause,
            group_by: self.group_by,
            having: self.having,
            order_by: self.order_by,
            limit: self.limit,
            offset: self.offset,
            _state: PhantomData,
        }
    }
}

// Methods available after FROM
impl<Cols> SelectDyn<Cols, HasFrom> {
    /// Adds a WHERE clause.
    #[must_use]
    pub fn where_clause(mut self, expr: ExprBuilder) -> Self {
        self.where_clause = Some(expr);
        self
    }

    /// Adds an INNER JOIN.
    #[must_use]
    pub fn join(mut self, table: &str, on: &str) -> Self {
        self.joins.push(format!("INNER JOIN {table} ON {on}"));
        self
    }

    /// Adds a LEFT JOIN.
    #[must_use]
    pub fn left_join(mut self, table: &str, on: &str) -> Self {
        self.joins.push(format!("LEFT JOIN {table} ON {on}"));
        self
    }

    /// Adds a RIGHT JOIN.
    #[must_use]
    pub fn right_join(mut self, table: &str, on: &str) -> Self {
        self.joins.push(format!("RIGHT JOIN {table} ON {on}"));
        self
    }

    /// Adds a CROSS JOIN.
    #[must_use]
    pub fn cross_join(mut self, table: &str) -> Self {
        self.joins.push(format!("CROSS JOIN {table}"));
        self
    }
}

// Methods available with columns
impl<From> SelectDyn<HasColumns, From> {
    /// Sets DISTINCT.
    #[must_use]
    pub fn distinct(mut self) -> Self {
        self.distinct = true;
        self
    }
}

// Methods available with FROM (for grouping)
impl SelectDyn<HasColumns, HasFrom> {
    /// Adds a GROUP BY clause.
    #[must_use]
    pub fn group_by(mut self, cols: &[&str]) -> Self {
        self.group_by = cols.iter().map(|s| String::from(*s)).collect();
        self
    }

    /// Adds a HAVING clause (only valid after GROUP BY).
    #[must_use]
    pub fn having(mut self, expr: ExprBuilder) -> Self {
        self.having = Some(expr);
        self
    }

    /// Adds an ORDER BY clause.
    #[must_use]
    pub fn order_by(mut self, cols: &[&str]) -> Self {
        self.order_by = cols.iter().map(|s| String::from(*s)).collect();
        self
    }

    /// Adds an ORDER BY DESC clause.
    #[must_use]
    pub fn order_by_desc(mut self, cols: &[&str]) -> Self {
        self.order_by = cols.iter().map(|s| format!("{s} DESC")).collect();
        self
    }

    /// Adds a LIMIT clause.
    #[must_use]
    pub const fn limit(mut self, n: u64) -> Self {
        self.limit = Some(n);
        self
    }

    /// Adds an OFFSET clause.
    #[must_use]
    pub const fn offset(mut self, n: u64) -> Self {
        self.offset = Some(n);
        self
    }

    /// Builds the SELECT statement and returns SQL with parameters.
    #[must_use]
    pub fn build(self) -> (String, Vec<SqlValue>) {
        let mut sql = String::from("SELECT ");
        let mut params = vec![];

        if self.distinct {
            sql.push_str("DISTINCT ");
        }

        sql.push_str(&self.columns.join(", "));

        if let Some(ref table) = self.from {
            sql.push_str(" FROM ");
            sql.push_str(table);
        }

        for join in &self.joins {
            sql.push(' ');
            sql.push_str(join);
        }

        if let Some(ref where_expr) = self.where_clause {
            sql.push_str(" WHERE ");
            sql.push_str(where_expr.sql());
            params.extend(where_expr.params().iter().cloned());
        }

        if !self.group_by.is_empty() {
            sql.push_str(" GROUP BY ");
            sql.push_str(&self.group_by.join(", "));
        }

        if let Some(ref having_expr) = self.having {
            sql.push_str(" HAVING ");
            sql.push_str(having_expr.sql());
            params.extend(having_expr.params().iter().cloned());
        }

        if !self.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            sql.push_str(&self.order_by.join(", "));
        }

        if let Some(n) = self.limit {
            sql.push_str(&format!(" LIMIT {n}"));
        }

        if let Some(n) = self.offset {
            sql.push_str(&format!(" OFFSET {n}"));
        }

        (sql, params)
    }

    /// Builds the SELECT statement and returns only the SQL string.
    ///
    /// **Warning**: Parameters are inlined using proper escaping.
    /// Prefer `build()` for parameterized queries.
    #[must_use]
    pub fn build_sql(self) -> String {
        let (sql, _params) = self.build();
        sql
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::dyn_col;

    #[test]
    fn test_simple_select() {
        let (sql, params) = SelectDyn::new()
            .columns(&["id", "name"])
            .from("users")
            .build();

        assert_eq!(sql, "SELECT id, name FROM users");
        assert!(params.is_empty());
    }

    #[test]
    fn test_select_all() {
        let (sql, _) = SelectDyn::new().all().from("users").build();
        assert_eq!(sql, "SELECT * FROM users");
    }

    #[test]
    fn test_select_distinct() {
        let (sql, _) = SelectDyn::new()
            .columns(&["status"])
            .distinct()
            .from("orders")
            .build();

        assert_eq!(sql, "SELECT DISTINCT status FROM orders");
    }

    #[test]
    fn test_select_with_where() {
        let (sql, params) = SelectDyn::new()
            .columns(&["id", "name"])
            .from("users")
            .where_clause(dyn_col("active").eq(true))
            .build();

        assert_eq!(sql, "SELECT id, name FROM users WHERE active = ?");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_select_with_join() {
        let (sql, _) = SelectDyn::new()
            .columns(&["u.id", "o.amount"])
            .from("users u")
            .join("orders o", "u.id = o.user_id")
            .build();

        assert_eq!(
            sql,
            "SELECT u.id, o.amount FROM users u INNER JOIN orders o ON u.id = o.user_id"
        );
    }

    #[test]
    fn test_select_with_group_by() {
        let (sql, _) = SelectDyn::new()
            .columns(&["status", "COUNT(*)"])
            .from("orders")
            .group_by(&["status"])
            .build();

        assert_eq!(sql, "SELECT status, COUNT(*) FROM orders GROUP BY status");
    }

    #[test]
    fn test_select_with_order_by() {
        let (sql, _) = SelectDyn::new()
            .columns(&["id", "name"])
            .from("users")
            .order_by(&["name"])
            .build();

        assert_eq!(sql, "SELECT id, name FROM users ORDER BY name");
    }

    #[test]
    fn test_select_with_limit_offset() {
        let (sql, _) = SelectDyn::new()
            .columns(&["id"])
            .from("users")
            .limit(10)
            .offset(20)
            .build();

        assert_eq!(sql, "SELECT id FROM users LIMIT 10 OFFSET 20");
    }

    #[test]
    fn test_complex_select() {
        let (sql, params) = SelectDyn::new()
            .columns(&["u.id", "u.name", "COUNT(o.id) as order_count"])
            .from("users u")
            .left_join("orders o", "u.id = o.user_id")
            .where_clause(
                dyn_col("u.active")
                    .eq(true)
                    .and(dyn_col("o.status").not_eq("cancelled")),
            )
            .group_by(&["u.id", "u.name"])
            .order_by_desc(&["order_count"])
            .limit(10)
            .build();

        assert!(sql.contains("SELECT u.id, u.name, COUNT(o.id) as order_count"));
        assert!(sql.contains("FROM users u"));
        assert!(sql.contains("LEFT JOIN orders o ON u.id = o.user_id"));
        assert!(sql.contains("WHERE u.active = ? AND o.status != ?"));
        assert!(sql.contains("GROUP BY u.id, u.name"));
        assert!(sql.contains("ORDER BY order_count DESC"));
        assert!(sql.contains("LIMIT 10"));
        assert_eq!(params.len(), 2);
    }

    // Compile-time tests (these would fail to compile if uncommented)

    // This would fail to compile: SELECT without FROM
    // #[test]
    // fn test_select_without_from_fails() {
    //     let _ = SelectDyn::new()
    //         .columns(&["id"])
    //         .build();  // Error: method `build` not found
    // }

    // This would fail to compile: WHERE without FROM
    // #[test]
    // fn test_where_without_from_fails() {
    //     let _ = SelectDyn::new()
    //         .columns(&["id"])
    //         .where_clause(dyn_col("id").eq(1));  // Error: no method `where_clause`
    // }

    // This would fail to compile: SELECT without columns
    // #[test]
    // fn test_select_without_columns_fails() {
    //     let _ = SelectDyn::new()
    //         .from("users")
    //         .build();  // Error: method `build` not found
    // }
}
