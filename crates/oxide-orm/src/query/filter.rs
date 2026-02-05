//! Q objects for complex query filtering.
//!
//! Q objects allow building complex filter expressions that can be combined
//! with AND, OR, and NOT operators, similar to Django's Q objects.

use oxide_sql_core::builder::value::{SqlValue, ToSqlValue};
use std::fmt;

/// A filter expression that can be combined with other expressions.
///
/// # Example
///
/// ```ignore
/// use oxide_orm::Q;
///
/// // Simple equality
/// let filter = Q::eq("status", "active");
///
/// // Complex boolean logic
/// let filter = Q::eq("status", "active")
///     .and(Q::gt("age", 18).or(Q::eq("verified", true)));
///
/// // NOT expressions
/// let filter = Q::eq("deleted", true).not();
/// ```
#[derive(Debug, Clone)]
pub struct Q {
    expr: FilterExpr,
}

/// Internal filter expression representation.
#[derive(Debug, Clone)]
pub enum FilterExpr {
    /// Simple comparison: field op value
    Comparison {
        field: String,
        op: CompareOp,
        value: SqlValue,
    },
    /// IS NULL check
    IsNull { field: String },
    /// IS NOT NULL check
    IsNotNull { field: String },
    /// IN list check
    InList {
        field: String,
        values: Vec<SqlValue>,
    },
    /// NOT IN list check
    NotInList {
        field: String,
        values: Vec<SqlValue>,
    },
    /// LIKE pattern match
    Like { field: String, pattern: String },
    /// BETWEEN range check
    Between {
        field: String,
        low: SqlValue,
        high: SqlValue,
    },
    /// AND combination
    And(Box<FilterExpr>, Box<FilterExpr>),
    /// OR combination
    Or(Box<FilterExpr>, Box<FilterExpr>),
    /// NOT negation
    Not(Box<FilterExpr>),
    /// Raw SQL expression (use with caution)
    Raw { sql: String, params: Vec<SqlValue> },
}

/// Comparison operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    /// Equal (=)
    Eq,
    /// Not equal (!=)
    Ne,
    /// Greater than (>)
    Gt,
    /// Greater than or equal (>=)
    Gte,
    /// Less than (<)
    Lt,
    /// Less than or equal (<=)
    Lte,
}

impl fmt::Display for CompareOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Eq => write!(f, "="),
            Self::Ne => write!(f, "!="),
            Self::Gt => write!(f, ">"),
            Self::Gte => write!(f, ">="),
            Self::Lt => write!(f, "<"),
            Self::Lte => write!(f, "<="),
        }
    }
}

impl Q {
    /// Creates an equality filter (field = value).
    pub fn eq<V: ToSqlValue>(field: &str, value: V) -> Self {
        Self {
            expr: FilterExpr::Comparison {
                field: field.to_string(),
                op: CompareOp::Eq,
                value: value.to_sql_value(),
            },
        }
    }

    /// Creates an inequality filter (field != value).
    pub fn ne<V: ToSqlValue>(field: &str, value: V) -> Self {
        Self {
            expr: FilterExpr::Comparison {
                field: field.to_string(),
                op: CompareOp::Ne,
                value: value.to_sql_value(),
            },
        }
    }

    /// Creates a greater-than filter (field > value).
    pub fn gt<V: ToSqlValue>(field: &str, value: V) -> Self {
        Self {
            expr: FilterExpr::Comparison {
                field: field.to_string(),
                op: CompareOp::Gt,
                value: value.to_sql_value(),
            },
        }
    }

    /// Creates a greater-than-or-equal filter (field >= value).
    pub fn gte<V: ToSqlValue>(field: &str, value: V) -> Self {
        Self {
            expr: FilterExpr::Comparison {
                field: field.to_string(),
                op: CompareOp::Gte,
                value: value.to_sql_value(),
            },
        }
    }

    /// Creates a less-than filter (field < value).
    pub fn lt<V: ToSqlValue>(field: &str, value: V) -> Self {
        Self {
            expr: FilterExpr::Comparison {
                field: field.to_string(),
                op: CompareOp::Lt,
                value: value.to_sql_value(),
            },
        }
    }

    /// Creates a less-than-or-equal filter (field <= value).
    pub fn lte<V: ToSqlValue>(field: &str, value: V) -> Self {
        Self {
            expr: FilterExpr::Comparison {
                field: field.to_string(),
                op: CompareOp::Lte,
                value: value.to_sql_value(),
            },
        }
    }

    /// Creates an IS NULL filter.
    pub fn is_null(field: &str) -> Self {
        Self {
            expr: FilterExpr::IsNull {
                field: field.to_string(),
            },
        }
    }

    /// Creates an IS NOT NULL filter.
    pub fn is_not_null(field: &str) -> Self {
        Self {
            expr: FilterExpr::IsNotNull {
                field: field.to_string(),
            },
        }
    }

    /// Creates an IN list filter.
    pub fn in_list<V: ToSqlValue>(field: &str, values: Vec<V>) -> Self {
        Self {
            expr: FilterExpr::InList {
                field: field.to_string(),
                values: values.into_iter().map(|v| v.to_sql_value()).collect(),
            },
        }
    }

    /// Creates a NOT IN list filter.
    pub fn not_in_list<V: ToSqlValue>(field: &str, values: Vec<V>) -> Self {
        Self {
            expr: FilterExpr::NotInList {
                field: field.to_string(),
                values: values.into_iter().map(|v| v.to_sql_value()).collect(),
            },
        }
    }

    /// Creates a LIKE filter (case-sensitive pattern match).
    ///
    /// Use `%` for wildcard matching.
    pub fn like(field: &str, pattern: &str) -> Self {
        Self {
            expr: FilterExpr::Like {
                field: field.to_string(),
                pattern: pattern.to_string(),
            },
        }
    }

    /// Creates a contains filter (LIKE %value%).
    pub fn contains(field: &str, value: &str) -> Self {
        Self::like(field, &format!("%{value}%"))
    }

    /// Creates a starts-with filter (LIKE value%).
    pub fn startswith(field: &str, value: &str) -> Self {
        Self::like(field, &format!("{value}%"))
    }

    /// Creates an ends-with filter (LIKE %value).
    pub fn endswith(field: &str, value: &str) -> Self {
        Self::like(field, &format!("%{value}"))
    }

    /// Creates a BETWEEN filter (low <= field <= high).
    pub fn between<V: ToSqlValue>(field: &str, low: V, high: V) -> Self {
        Self {
            expr: FilterExpr::Between {
                field: field.to_string(),
                low: low.to_sql_value(),
                high: high.to_sql_value(),
            },
        }
    }

    /// Creates a raw SQL filter expression.
    ///
    /// **Warning**: Use parameterized values to prevent SQL injection.
    pub fn raw(sql: &str, params: Vec<SqlValue>) -> Self {
        Self {
            expr: FilterExpr::Raw {
                sql: sql.to_string(),
                params,
            },
        }
    }

    /// Combines this filter with another using AND.
    #[must_use]
    pub fn and(self, other: Q) -> Q {
        Q {
            expr: FilterExpr::And(Box::new(self.expr), Box::new(other.expr)),
        }
    }

    /// Combines this filter with another using OR.
    #[must_use]
    pub fn or(self, other: Q) -> Q {
        Q {
            expr: FilterExpr::Or(Box::new(self.expr), Box::new(other.expr)),
        }
    }

    /// Negates this filter with NOT.
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn not(self) -> Q {
        Q {
            expr: FilterExpr::Not(Box::new(self.expr)),
        }
    }

    /// Returns the internal filter expression.
    pub fn into_expr(self) -> FilterExpr {
        self.expr
    }

    /// Builds the SQL WHERE clause and parameters.
    pub fn build(&self) -> (String, Vec<SqlValue>) {
        build_filter_expr(&self.expr)
    }
}

impl From<Q> for FilterExpr {
    fn from(q: Q) -> Self {
        q.expr
    }
}

/// Builds SQL and parameters from a filter expression.
fn build_filter_expr(expr: &FilterExpr) -> (String, Vec<SqlValue>) {
    match expr {
        FilterExpr::Comparison { field, op, value } => {
            (format!("{field} {op} ?"), vec![value.clone()])
        }
        FilterExpr::IsNull { field } => (format!("{field} IS NULL"), vec![]),
        FilterExpr::IsNotNull { field } => (format!("{field} IS NOT NULL"), vec![]),
        FilterExpr::InList { field, values } => {
            let placeholders: Vec<&str> = values.iter().map(|_| "?").collect();
            (
                format!("{field} IN ({})", placeholders.join(", ")),
                values.clone(),
            )
        }
        FilterExpr::NotInList { field, values } => {
            let placeholders: Vec<&str> = values.iter().map(|_| "?").collect();
            (
                format!("{field} NOT IN ({})", placeholders.join(", ")),
                values.clone(),
            )
        }
        FilterExpr::Like { field, pattern } => (
            format!("{field} LIKE ?"),
            vec![SqlValue::Text(pattern.clone())],
        ),
        FilterExpr::Between { field, low, high } => (
            format!("{field} BETWEEN ? AND ?"),
            vec![low.clone(), high.clone()],
        ),
        FilterExpr::And(left, right) => {
            let (left_sql, mut left_params) = build_filter_expr(left);
            let (right_sql, right_params) = build_filter_expr(right);
            left_params.extend(right_params);
            (format!("({left_sql}) AND ({right_sql})"), left_params)
        }
        FilterExpr::Or(left, right) => {
            let (left_sql, mut left_params) = build_filter_expr(left);
            let (right_sql, right_params) = build_filter_expr(right);
            left_params.extend(right_params);
            (format!("({left_sql}) OR ({right_sql})"), left_params)
        }
        FilterExpr::Not(inner) => {
            let (inner_sql, params) = build_filter_expr(inner);
            (format!("NOT ({inner_sql})"), params)
        }
        FilterExpr::Raw { sql, params } => (sql.clone(), params.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_eq() {
        let q = Q::eq("status", "active");
        let (sql, params) = q.build();
        assert_eq!(sql, "status = ?");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_and_combination() {
        let q = Q::eq("status", "active").and(Q::gt("age", 18));
        let (sql, params) = q.build();
        assert_eq!(sql, "(status = ?) AND (age > ?)");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_or_combination() {
        let q = Q::eq("role", "admin").or(Q::eq("role", "moderator"));
        let (sql, params) = q.build();
        assert_eq!(sql, "(role = ?) OR (role = ?)");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_not() {
        let q = Q::eq("deleted", true).not();
        let (sql, params) = q.build();
        assert_eq!(sql, "NOT (deleted = ?)");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_complex_expression() {
        let q = Q::eq("status", "active").and(Q::gt("age", 18).or(Q::eq("verified", true)));
        let (sql, params) = q.build();
        assert_eq!(sql, "(status = ?) AND ((age > ?) OR (verified = ?))");
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn test_in_list() {
        let q = Q::in_list("status", vec!["active", "pending"]);
        let (sql, params) = q.build();
        assert_eq!(sql, "status IN (?, ?)");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_contains() {
        let q = Q::contains("email", "@example.com");
        let (sql, params) = q.build();
        assert_eq!(sql, "email LIKE ?");
        assert_eq!(params[0], SqlValue::Text("%@example.com%".to_string()));
    }

    #[test]
    fn test_between() {
        let q = Q::between("price", 10, 100);
        let (sql, params) = q.build();
        assert_eq!(sql, "price BETWEEN ? AND ?");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_is_null() {
        let q = Q::is_null("deleted_at");
        let (sql, params) = q.build();
        assert_eq!(sql, "deleted_at IS NULL");
        assert!(params.is_empty());
    }
}
