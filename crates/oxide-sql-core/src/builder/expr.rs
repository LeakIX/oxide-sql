//! Type-safe expression builder.

use super::value::{SqlValue, ToSqlValue};

/// Creates a column reference.
#[must_use]
pub fn col(name: &str) -> Column {
    Column {
        table: None,
        name: String::from(name),
    }
}

/// A column reference.
#[derive(Debug, Clone)]
pub struct Column {
    /// Optional table qualifier.
    pub table: Option<String>,
    /// Column name.
    pub name: String,
}

impl Column {
    /// Creates a qualified column reference.
    #[must_use]
    pub fn qualified(table: &str, name: &str) -> Self {
        Self {
            table: Some(String::from(table)),
            name: String::from(name),
        }
    }

    /// Returns the SQL representation.
    #[must_use]
    pub fn to_sql(&self) -> String {
        match &self.table {
            Some(t) => format!("{t}.{}", self.name),
            None => self.name.clone(),
        }
    }

    /// Creates an equality expression.
    #[must_use]
    pub fn eq<T: ToSqlValue>(self, value: T) -> ExprBuilder {
        ExprBuilder::binary(self.into(), "=", value.to_sql_value().into())
    }

    /// Creates an inequality expression.
    #[must_use]
    pub fn not_eq<T: ToSqlValue>(self, value: T) -> ExprBuilder {
        ExprBuilder::binary(self.into(), "!=", value.to_sql_value().into())
    }

    /// Creates a less-than expression.
    #[must_use]
    pub fn lt<T: ToSqlValue>(self, value: T) -> ExprBuilder {
        ExprBuilder::binary(self.into(), "<", value.to_sql_value().into())
    }

    /// Creates a less-than-or-equal expression.
    #[must_use]
    pub fn lt_eq<T: ToSqlValue>(self, value: T) -> ExprBuilder {
        ExprBuilder::binary(self.into(), "<=", value.to_sql_value().into())
    }

    /// Creates a greater-than expression.
    #[must_use]
    pub fn gt<T: ToSqlValue>(self, value: T) -> ExprBuilder {
        ExprBuilder::binary(self.into(), ">", value.to_sql_value().into())
    }

    /// Creates a greater-than-or-equal expression.
    #[must_use]
    pub fn gt_eq<T: ToSqlValue>(self, value: T) -> ExprBuilder {
        ExprBuilder::binary(self.into(), ">=", value.to_sql_value().into())
    }

    /// Creates an IS NULL expression.
    #[must_use]
    pub fn is_null(self) -> ExprBuilder {
        ExprBuilder::postfix(self.into(), "IS NULL")
    }

    /// Creates an IS NOT NULL expression.
    #[must_use]
    pub fn is_not_null(self) -> ExprBuilder {
        ExprBuilder::postfix(self.into(), "IS NOT NULL")
    }

    /// Creates a LIKE expression.
    #[must_use]
    pub fn like<T: ToSqlValue>(self, pattern: T) -> ExprBuilder {
        ExprBuilder::binary(self.into(), "LIKE", pattern.to_sql_value().into())
    }

    /// Creates a NOT LIKE expression.
    #[must_use]
    pub fn not_like<T: ToSqlValue>(self, pattern: T) -> ExprBuilder {
        ExprBuilder::binary(self.into(), "NOT LIKE", pattern.to_sql_value().into())
    }

    /// Creates a BETWEEN expression.
    #[must_use]
    pub fn between<T: ToSqlValue, U: ToSqlValue>(self, low: T, high: U) -> ExprBuilder {
        ExprBuilder::between(self.into(), low.to_sql_value(), high.to_sql_value(), false)
    }

    /// Creates a NOT BETWEEN expression.
    #[must_use]
    pub fn not_between<T: ToSqlValue, U: ToSqlValue>(self, low: T, high: U) -> ExprBuilder {
        ExprBuilder::between(self.into(), low.to_sql_value(), high.to_sql_value(), true)
    }

    /// Creates an IN expression.
    #[must_use]
    pub fn in_list<T: ToSqlValue>(self, values: Vec<T>) -> ExprBuilder {
        let sql_values: Vec<SqlValue> = values.into_iter().map(ToSqlValue::to_sql_value).collect();
        ExprBuilder::in_list_impl(self.into(), sql_values, false)
    }

    /// Creates a NOT IN expression.
    #[must_use]
    pub fn not_in_list<T: ToSqlValue>(self, values: Vec<T>) -> ExprBuilder {
        let sql_values: Vec<SqlValue> = values.into_iter().map(ToSqlValue::to_sql_value).collect();
        ExprBuilder::in_list_impl(self.into(), sql_values, true)
    }
}

/// A type-safe expression builder.
#[derive(Debug, Clone)]
pub struct ExprBuilder {
    sql: String,
    params: Vec<SqlValue>,
}

impl ExprBuilder {
    /// Creates a new expression from raw SQL.
    ///
    /// **Warning**: Only use this for SQL fragments that don't contain user input.
    #[must_use]
    pub fn raw(sql: impl Into<String>) -> Self {
        Self {
            sql: sql.into(),
            params: vec![],
        }
    }

    /// Creates a column reference expression.
    ///
    /// This is used internally by typed column accessors.
    #[must_use]
    pub fn column(name: &str) -> Self {
        Self {
            sql: String::from(name),
            params: vec![],
        }
    }

    /// Creates an expression from a value (parameterized).
    #[must_use]
    pub fn value<T: ToSqlValue>(value: T) -> Self {
        Self {
            sql: String::from("?"),
            params: vec![value.to_sql_value()],
        }
    }

    /// Creates a binary expression.
    fn binary(left: Self, op: &str, right: Self) -> Self {
        let mut params = left.params;
        params.extend(right.params);
        Self {
            sql: format!("{} {op} {}", left.sql, right.sql),
            params,
        }
    }

    /// Creates a postfix expression.
    fn postfix(operand: Self, op: &str) -> Self {
        Self {
            sql: format!("{} {op}", operand.sql),
            params: operand.params,
        }
    }

    /// Creates a BETWEEN expression.
    fn between(expr: Self, low: SqlValue, high: SqlValue, negated: bool) -> Self {
        let keyword = if negated { "NOT BETWEEN" } else { "BETWEEN" };
        let mut params = expr.params;
        params.push(low);
        params.push(high);
        Self {
            sql: format!("{} {keyword} ? AND ?", expr.sql),
            params,
        }
    }

    /// Creates an IN expression (internal).
    fn in_list_impl(expr: Self, values: Vec<SqlValue>, negated: bool) -> Self {
        let keyword = if negated { "NOT IN" } else { "IN" };
        let placeholders: Vec<&str> = values.iter().map(|_| "?").collect();
        let mut params = expr.params;
        params.extend(values);
        Self {
            sql: format!("{} {keyword} ({})", expr.sql, placeholders.join(", ")),
            params,
        }
    }

    /// Creates an AND expression.
    #[must_use]
    pub fn and(self, other: Self) -> Self {
        Self::binary(self, "AND", other)
    }

    /// Creates an OR expression.
    #[must_use]
    pub fn or(self, other: Self) -> Self {
        Self::binary(self, "OR", other)
    }

    /// Wraps the expression in parentheses.
    #[must_use]
    pub fn paren(self) -> Self {
        Self {
            sql: format!("({})", self.sql),
            params: self.params,
        }
    }

    /// Negates the expression with NOT.
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn not(self) -> Self {
        Self {
            sql: format!("NOT {}", self.sql),
            params: self.params,
        }
    }

    /// Creates an equality expression.
    #[must_use]
    pub fn eq<T: ToSqlValue>(self, value: T) -> Self {
        Self::binary(self, "=", value.to_sql_value().into())
    }

    /// Creates an inequality expression.
    #[must_use]
    pub fn not_eq<T: ToSqlValue>(self, value: T) -> Self {
        Self::binary(self, "!=", value.to_sql_value().into())
    }

    /// Creates a less-than expression.
    #[must_use]
    pub fn lt<T: ToSqlValue>(self, value: T) -> Self {
        Self::binary(self, "<", value.to_sql_value().into())
    }

    /// Creates a less-than-or-equal expression.
    #[must_use]
    pub fn lt_eq<T: ToSqlValue>(self, value: T) -> Self {
        Self::binary(self, "<=", value.to_sql_value().into())
    }

    /// Creates a greater-than expression.
    #[must_use]
    pub fn gt<T: ToSqlValue>(self, value: T) -> Self {
        Self::binary(self, ">", value.to_sql_value().into())
    }

    /// Creates a greater-than-or-equal expression.
    #[must_use]
    pub fn gt_eq<T: ToSqlValue>(self, value: T) -> Self {
        Self::binary(self, ">=", value.to_sql_value().into())
    }

    /// Creates an IS NULL expression.
    #[must_use]
    pub fn is_null(self) -> Self {
        Self::postfix(self, "IS NULL")
    }

    /// Creates an IS NOT NULL expression.
    #[must_use]
    pub fn is_not_null(self) -> Self {
        Self::postfix(self, "IS NOT NULL")
    }

    /// Creates a LIKE expression.
    #[must_use]
    pub fn like<T: ToSqlValue>(self, pattern: T) -> Self {
        Self::binary(self, "LIKE", pattern.to_sql_value().into())
    }

    /// Creates an IN expression.
    #[must_use]
    pub fn in_list<T: ToSqlValue>(self, values: Vec<T>) -> Self {
        let sql_values: Vec<SqlValue> = values.into_iter().map(ToSqlValue::to_sql_value).collect();
        Self::in_list_impl(self, sql_values, false)
    }

    /// Creates a NOT IN expression.
    #[must_use]
    pub fn not_in_list<T: ToSqlValue>(self, values: Vec<T>) -> Self {
        let sql_values: Vec<SqlValue> = values.into_iter().map(ToSqlValue::to_sql_value).collect();
        Self::in_list_impl(self, sql_values, true)
    }

    /// Returns the SQL string.
    #[must_use]
    pub fn sql(&self) -> &str {
        &self.sql
    }

    /// Returns the parameters.
    #[must_use]
    pub fn params(&self) -> &[SqlValue] {
        &self.params
    }

    /// Consumes the builder and returns the SQL and parameters.
    #[must_use]
    pub fn build(self) -> (String, Vec<SqlValue>) {
        (self.sql, self.params)
    }
}

impl From<Column> for ExprBuilder {
    fn from(col: Column) -> Self {
        Self {
            sql: col.to_sql(),
            params: vec![],
        }
    }
}

impl From<SqlValue> for ExprBuilder {
    fn from(value: SqlValue) -> Self {
        Self {
            sql: String::from("?"),
            params: vec![value],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_eq() {
        let expr = col("name").eq("Alice");
        assert_eq!(expr.sql(), "name = ?");
        assert_eq!(expr.params().len(), 1);
    }

    #[test]
    fn test_column_comparison() {
        assert_eq!(col("age").gt(18).sql(), "age > ?");
        assert_eq!(col("age").lt_eq(65).sql(), "age <= ?");
    }

    #[test]
    fn test_is_null() {
        let expr = col("deleted_at").is_null();
        assert_eq!(expr.sql(), "deleted_at IS NULL");
        assert!(expr.params().is_empty());
    }

    #[test]
    fn test_like() {
        let expr = col("email").like("%@example.com");
        assert_eq!(expr.sql(), "email LIKE ?");
    }

    #[test]
    fn test_between() {
        let expr = col("price").between(10, 100);
        assert_eq!(expr.sql(), "price BETWEEN ? AND ?");
        assert_eq!(expr.params().len(), 2);
    }

    #[test]
    fn test_in_list() {
        let expr = col("status").in_list(vec!["active", "pending"]);
        assert_eq!(expr.sql(), "status IN (?, ?)");
        assert_eq!(expr.params().len(), 2);
    }

    #[test]
    fn test_and_or() {
        let expr = col("active")
            .eq(true)
            .and(col("age").gt(18).or(col("verified").eq(true)).paren());
        assert_eq!(expr.sql(), "active = ? AND (age > ? OR verified = ?)");
        assert_eq!(expr.params().len(), 3);
    }

    #[test]
    fn test_qualified_column() {
        let expr = Column::qualified("users", "name").eq("Bob");
        assert_eq!(expr.sql(), "users.name = ?");
    }

    #[test]
    fn test_sql_injection_prevention() {
        let malicious = "'; DROP TABLE users; --";
        let expr = col("name").eq(malicious);
        // The value is parameterized, not interpolated
        assert_eq!(expr.sql(), "name = ?");
        // The malicious input is stored safely as a parameter
        assert!(matches!(&expr.params()[0], SqlValue::Text(s) if s == malicious));
    }
}
