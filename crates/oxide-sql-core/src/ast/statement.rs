//! SQL statement AST types.

use core::fmt;

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

impl fmt::Display for OrderDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
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

impl fmt::Display for NullOrdering {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
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

impl fmt::Display for JoinType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
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

// ===================================================================
// Display implementations
// ===================================================================

impl fmt::Display for OrderBy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.expr, self.direction)?;
        if let Some(nulls) = &self.nulls {
            write!(f, " {nulls}")?;
        }
        Ok(())
    }
}

impl fmt::Display for JoinClause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.join_type, self.table)?;
        if let Some(on) = &self.on {
            write!(f, " ON {on}")?;
        }
        if !self.using.is_empty() {
            write!(f, " USING (")?;
            for (i, col) in self.using.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{col}")?;
            }
            write!(f, ")")?;
        }
        Ok(())
    }
}

impl fmt::Display for TableRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Table {
                schema,
                name,
                alias,
            } => {
                if let Some(s) = schema {
                    write!(f, "{s}.")?;
                }
                write!(f, "{name}")?;
                if let Some(a) = alias {
                    write!(f, " AS {a}")?;
                }
                Ok(())
            }
            Self::Subquery { query, alias } => {
                write!(f, "({query}) AS {alias}")
            }
            Self::Join { left, join } => {
                write!(f, "{left} {join}")
            }
        }
    }
}

impl fmt::Display for SelectColumn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.expr)?;
        if let Some(a) = &self.alias {
            write!(f, " AS {a}")?;
        }
        Ok(())
    }
}

impl fmt::Display for SelectStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SELECT")?;
        if self.distinct {
            write!(f, " DISTINCT")?;
        }
        for (i, col) in self.columns.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, " {col}")?;
        }
        if let Some(from) = &self.from {
            write!(f, " FROM {from}")?;
        }
        if let Some(w) = &self.where_clause {
            write!(f, " WHERE {w}")?;
        }
        if !self.group_by.is_empty() {
            write!(f, " GROUP BY")?;
            for (i, g) in self.group_by.iter().enumerate() {
                if i > 0 {
                    write!(f, ",")?;
                }
                write!(f, " {g}")?;
            }
        }
        if let Some(h) = &self.having {
            write!(f, " HAVING {h}")?;
        }
        if !self.order_by.is_empty() {
            write!(f, " ORDER BY")?;
            for (i, o) in self.order_by.iter().enumerate() {
                if i > 0 {
                    write!(f, ",")?;
                }
                write!(f, " {o}")?;
            }
        }
        if let Some(l) = &self.limit {
            write!(f, " LIMIT {l}")?;
        }
        if let Some(o) = &self.offset {
            write!(f, " OFFSET {o}")?;
        }
        Ok(())
    }
}

impl fmt::Display for InsertSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Values(rows) => {
                write!(f, "VALUES")?;
                for (i, row) in rows.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, " (")?;
                    for (j, val) in row.iter().enumerate() {
                        if j > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{val}")?;
                    }
                    write!(f, ")")?;
                }
                Ok(())
            }
            Self::Query(q) => write!(f, "{q}"),
            Self::DefaultValues => write!(f, "DEFAULT VALUES"),
        }
    }
}

impl fmt::Display for OnConflict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ON CONFLICT (")?;
        for (i, col) in self.columns.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{col}")?;
        }
        write!(f, ") {}", self.action)
    }
}

impl fmt::Display for ConflictAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DoNothing => write!(f, "DO NOTHING"),
            Self::DoUpdate(assignments) => {
                write!(f, "DO UPDATE SET")?;
                for (i, a) in assignments.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, " {a}")?;
                }
                Ok(())
            }
        }
    }
}

impl fmt::Display for InsertStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "INSERT INTO ")?;
        if let Some(s) = &self.schema {
            write!(f, "{s}.")?;
        }
        write!(f, "{}", self.table)?;
        if !self.columns.is_empty() {
            write!(f, " (")?;
            for (i, col) in self.columns.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{col}")?;
            }
            write!(f, ")")?;
        }
        write!(f, " {}", self.values)?;
        if let Some(oc) = &self.on_conflict {
            write!(f, " {oc}")?;
        }
        Ok(())
    }
}

impl fmt::Display for UpdateAssignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.column, self.value)
    }
}

impl fmt::Display for UpdateStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UPDATE ")?;
        if let Some(s) = &self.schema {
            write!(f, "{s}.")?;
        }
        write!(f, "{}", self.table)?;
        if let Some(a) = &self.alias {
            write!(f, " AS {a}")?;
        }
        write!(f, " SET")?;
        for (i, a) in self.assignments.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, " {a}")?;
        }
        if let Some(from) = &self.from {
            write!(f, " FROM {from}")?;
        }
        if let Some(w) = &self.where_clause {
            write!(f, " WHERE {w}")?;
        }
        Ok(())
    }
}

impl fmt::Display for DeleteStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DELETE FROM ")?;
        if let Some(s) = &self.schema {
            write!(f, "{s}.")?;
        }
        write!(f, "{}", self.table)?;
        if let Some(a) = &self.alias {
            write!(f, " AS {a}")?;
        }
        if let Some(w) = &self.where_clause {
            write!(f, " WHERE {w}")?;
        }
        Ok(())
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Select(s) => write!(f, "{s}"),
            Self::Insert(i) => write!(f, "{i}"),
            Self::Update(u) => write!(f, "{u}"),
            Self::Delete(d) => write!(f, "{d}"),
        }
    }
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
