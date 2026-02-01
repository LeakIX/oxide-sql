//! SQL statement AST types.

use super::expression::Expr;

/// Order direction for ORDER BY.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OrderDirection {
    /// Ascending order (default).
    #[default]
    Asc,
    /// Descending order.
    Desc,
}

impl OrderDirection {
    /// Returns the SQL representation.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}

/// Null ordering for ORDER BY.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NullOrdering {
    /// NULLs come first.
    First,
    /// NULLs come last.
    Last,
}

impl NullOrdering {
    /// Returns the SQL representation.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::First => "NULLS FIRST",
            Self::Last => "NULLS LAST",
        }
    }
}

/// An ORDER BY clause entry.
#[derive(Debug, Clone, PartialEq)]
pub struct OrderBy {
    /// The expression to order by.
    pub expr: Expr,
    /// The direction (ASC or DESC).
    pub direction: OrderDirection,
    /// Null ordering (optional).
    pub nulls: Option<NullOrdering>,
}

/// Join type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinType {
    /// INNER JOIN.
    Inner,
    /// LEFT OUTER JOIN.
    Left,
    /// RIGHT OUTER JOIN.
    Right,
    /// FULL OUTER JOIN.
    Full,
    /// CROSS JOIN.
    Cross,
}

impl JoinType {
    /// Returns the SQL representation.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Inner => "INNER JOIN",
            Self::Left => "LEFT JOIN",
            Self::Right => "RIGHT JOIN",
            Self::Full => "FULL JOIN",
            Self::Cross => "CROSS JOIN",
        }
    }
}

/// A JOIN clause.
#[derive(Debug, Clone, PartialEq)]
pub struct JoinClause {
    /// The type of join.
    pub join_type: JoinType,
    /// The table to join.
    pub table: TableRef,
    /// The join condition (for non-CROSS joins).
    pub on: Option<Expr>,
    /// USING columns (alternative to ON).
    pub using: Vec<String>,
}

/// A table reference in FROM clause.
#[derive(Debug, Clone, PartialEq)]
pub enum TableRef {
    /// A simple table name.
    Table {
        /// Schema name (optional).
        schema: Option<String>,
        /// Table name.
        name: String,
        /// Alias.
        alias: Option<String>,
    },
    /// A subquery.
    Subquery {
        /// The subquery.
        query: Box<SelectStatement>,
        /// Alias (required for subqueries).
        alias: String,
    },
    /// A joined table.
    Join {
        /// Left side of the join.
        left: Box<TableRef>,
        /// The join clause.
        join: Box<JoinClause>,
    },
}

impl TableRef {
    /// Creates a simple table reference.
    #[must_use]
    pub fn table(name: impl Into<String>) -> Self {
        Self::Table {
            schema: None,
            name: name.into(),
            alias: None,
        }
    }

    /// Creates a table reference with schema.
    #[must_use]
    pub fn with_schema(schema: impl Into<String>, name: impl Into<String>) -> Self {
        Self::Table {
            schema: Some(schema.into()),
            name: name.into(),
            alias: None,
        }
    }

    /// Adds an alias to this table reference.
    #[must_use]
    pub fn alias(self, alias: impl Into<String>) -> Self {
        match self {
            Self::Table { schema, name, .. } => Self::Table {
                schema,
                name,
                alias: Some(alias.into()),
            },
            Self::Subquery { query, .. } => Self::Subquery {
                query,
                alias: alias.into(),
            },
            Self::Join { left, join } => Self::Join {
                left: Box::new((*left).alias(alias)),
                join,
            },
        }
    }
}

/// A SELECT statement.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectStatement {
    /// Whether to select DISTINCT values.
    pub distinct: bool,
    /// The columns to select.
    pub columns: Vec<SelectColumn>,
    /// The FROM clause.
    pub from: Option<TableRef>,
    /// The WHERE clause.
    pub where_clause: Option<Expr>,
    /// GROUP BY expressions.
    pub group_by: Vec<Expr>,
    /// HAVING clause.
    pub having: Option<Expr>,
    /// ORDER BY clauses.
    pub order_by: Vec<OrderBy>,
    /// LIMIT clause.
    pub limit: Option<Expr>,
    /// OFFSET clause.
    pub offset: Option<Expr>,
}

/// A column in SELECT clause.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectColumn {
    /// The expression.
    pub expr: Expr,
    /// Column alias.
    pub alias: Option<String>,
}

impl SelectColumn {
    /// Creates a new select column.
    #[must_use]
    pub fn new(expr: Expr) -> Self {
        Self { expr, alias: None }
    }

    /// Creates a select column with an alias.
    #[must_use]
    pub fn with_alias(expr: Expr, alias: impl Into<String>) -> Self {
        Self {
            expr,
            alias: Some(alias.into()),
        }
    }
}

/// An INSERT statement.
#[derive(Debug, Clone, PartialEq)]
pub struct InsertStatement {
    /// Schema name.
    pub schema: Option<String>,
    /// Table name.
    pub table: String,
    /// Column names (optional).
    pub columns: Vec<String>,
    /// Values to insert.
    pub values: InsertSource,
    /// ON CONFLICT clause (for UPSERT).
    pub on_conflict: Option<OnConflict>,
}

/// Source of data for INSERT.
#[derive(Debug, Clone, PartialEq)]
pub enum InsertSource {
    /// VALUES (...), (...), ...
    Values(Vec<Vec<Expr>>),
    /// SELECT ...
    Query(Box<SelectStatement>),
    /// DEFAULT VALUES
    DefaultValues,
}

/// ON CONFLICT clause for UPSERT.
#[derive(Debug, Clone, PartialEq)]
pub struct OnConflict {
    /// Conflict target columns.
    pub columns: Vec<String>,
    /// Action to take on conflict.
    pub action: ConflictAction,
}

/// Action to take on conflict.
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictAction {
    /// DO NOTHING
    DoNothing,
    /// DO UPDATE SET ...
    DoUpdate(Vec<UpdateAssignment>),
}

/// An UPDATE statement.
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateStatement {
    /// Schema name.
    pub schema: Option<String>,
    /// Table name.
    pub table: String,
    /// Alias.
    pub alias: Option<String>,
    /// SET assignments.
    pub assignments: Vec<UpdateAssignment>,
    /// FROM clause (for joins in UPDATE).
    pub from: Option<TableRef>,
    /// WHERE clause.
    pub where_clause: Option<Expr>,
}

/// An assignment in UPDATE SET.
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateAssignment {
    /// Column name.
    pub column: String,
    /// Value expression.
    pub value: Expr,
}

/// A DELETE statement.
#[derive(Debug, Clone, PartialEq)]
pub struct DeleteStatement {
    /// Schema name.
    pub schema: Option<String>,
    /// Table name.
    pub table: String,
    /// Alias.
    pub alias: Option<String>,
    /// WHERE clause.
    pub where_clause: Option<Expr>,
}

/// A SQL statement.
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// SELECT statement.
    Select(SelectStatement),
    /// INSERT statement.
    Insert(InsertStatement),
    /// UPDATE statement.
    Update(UpdateStatement),
    /// DELETE statement.
    Delete(DeleteStatement),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_direction() {
        assert_eq!(OrderDirection::Asc.as_str(), "ASC");
        assert_eq!(OrderDirection::Desc.as_str(), "DESC");
    }

    #[test]
    fn test_join_type() {
        assert_eq!(JoinType::Inner.as_str(), "INNER JOIN");
        assert_eq!(JoinType::Left.as_str(), "LEFT JOIN");
    }

    #[test]
    fn test_table_ref_builder() {
        let table = TableRef::table("users").alias("u");
        assert!(
            matches!(table, TableRef::Table { name, alias, .. } if name == "users" && alias == Some(String::from("u")))
        );
    }
}
