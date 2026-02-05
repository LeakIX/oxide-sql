//! QuerySet implementation for lazy, chainable database queries.
//!
//! QuerySets are lazy - they don't execute until you iterate over them or
//! call a method that evaluates the query (like `execute()`, `first()`, etc.).

use oxide_sql_core::builder::value::SqlValue;
use sqlx::{FromRow, Row, SqlitePool};
use std::marker::PhantomData;

use crate::error::{OrmError, Result};
use crate::model::Model;
use crate::query::{Aggregate, FilterExpr, Q};

/// Order direction for sorting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderDirection {
    /// Ascending order (ASC)
    Asc,
    /// Descending order (DESC)
    Desc,
}

/// An ordering specification.
#[derive(Debug, Clone)]
pub struct OrderBy {
    /// Column to order by
    pub column: String,
    /// Order direction
    pub direction: OrderDirection,
}

impl OrderBy {
    /// Creates a new ascending order specification.
    pub fn asc(column: &str) -> Self {
        Self {
            column: column.to_string(),
            direction: OrderDirection::Asc,
        }
    }

    /// Creates a new descending order specification.
    pub fn desc(column: &str) -> Self {
        Self {
            column: column.to_string(),
            direction: OrderDirection::Desc,
        }
    }

    /// Parses a Django-style order specification.
    ///
    /// Prefix with `-` for descending order.
    /// Example: `"-created_at"` for descending, `"name"` for ascending.
    pub fn parse(spec: &str) -> Self {
        if let Some(column) = spec.strip_prefix('-') {
            Self::desc(column)
        } else {
            Self::asc(spec)
        }
    }

    /// Returns the SQL representation.
    pub fn to_sql(&self) -> String {
        match self.direction {
            OrderDirection::Asc => format!("{} ASC", self.column),
            OrderDirection::Desc => format!("{} DESC", self.column),
        }
    }
}

/// A lazy, chainable query builder for database operations.
///
/// QuerySets are immutable - each method returns a new QuerySet with the
/// modification applied.
///
/// # Example
///
/// ```ignore
/// use oxide_orm::QuerySet;
///
/// // Chain multiple operations
/// let users = User::objects()
///     .filter(Q::eq("is_active", true))
///     .exclude(Q::eq("role", "banned"))
///     .order_by("-created_at")
///     .limit(10)
///     .execute(&pool)
///     .await?;
/// ```
#[derive(Debug)]
pub struct QuerySet<M: Model> {
    /// Filter expressions (combined with AND)
    filters: Vec<FilterExpr>,
    /// Exclude expressions (combined with AND, then negated)
    excludes: Vec<FilterExpr>,
    /// Ordering specifications
    order_by: Vec<OrderBy>,
    /// LIMIT clause
    limit: Option<i64>,
    /// OFFSET clause
    offset: Option<i64>,
    /// Columns to select (None = all)
    select_columns: Option<Vec<String>>,
    /// Whether to select distinct rows
    distinct: bool,
    /// Phantom data for the model type
    _marker: PhantomData<M>,
}

// Manual Clone implementation to avoid M: Clone bound
impl<M: Model> Clone for QuerySet<M> {
    fn clone(&self) -> Self {
        Self {
            filters: self.filters.clone(),
            excludes: self.excludes.clone(),
            order_by: self.order_by.clone(),
            limit: self.limit,
            offset: self.offset,
            select_columns: self.select_columns.clone(),
            distinct: self.distinct,
            _marker: PhantomData,
        }
    }
}

impl<M: Model> Default for QuerySet<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: Model> QuerySet<M> {
    /// Creates a new empty QuerySet.
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
            excludes: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
            select_columns: None,
            distinct: false,
            _marker: PhantomData,
        }
    }

    /// Adds a filter to the QuerySet.
    ///
    /// Multiple filters are combined with AND.
    #[must_use]
    pub fn filter(mut self, q: Q) -> Self {
        self.filters.push(q.into_expr());
        self
    }

    /// Adds an exclude filter to the QuerySet.
    ///
    /// Excluded rows are those that match the filter.
    #[must_use]
    pub fn exclude(mut self, q: Q) -> Self {
        self.excludes.push(q.into_expr());
        self
    }

    /// Sets the ordering for the QuerySet.
    ///
    /// Use `-` prefix for descending order.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Order by created_at descending, then name ascending
    /// qs.order_by("-created_at").order_by("name")
    /// ```
    #[must_use]
    pub fn order_by(mut self, spec: &str) -> Self {
        self.order_by.push(OrderBy::parse(spec));
        self
    }

    /// Clears all ordering and sets new ordering.
    #[must_use]
    pub fn order_by_clear(mut self, specs: &[&str]) -> Self {
        self.order_by = specs.iter().map(|s| OrderBy::parse(s)).collect();
        self
    }

    /// Limits the number of results.
    #[must_use]
    pub fn limit(mut self, n: i64) -> Self {
        self.limit = Some(n);
        self
    }

    /// Sets the offset for pagination.
    #[must_use]
    pub fn offset(mut self, n: i64) -> Self {
        self.offset = Some(n);
        self
    }

    /// Selects specific columns.
    #[must_use]
    pub fn only(mut self, columns: &[&str]) -> Self {
        self.select_columns = Some(columns.iter().map(|s| (*s).to_string()).collect());
        self
    }

    /// Makes the query return distinct rows.
    #[must_use]
    pub fn distinct(mut self) -> Self {
        self.distinct = true;
        self
    }

    /// Returns a new QuerySet that is a copy of this one.
    #[must_use]
    pub fn all(&self) -> Self {
        self.clone()
    }

    /// Returns a QuerySet with no results.
    #[must_use]
    pub fn none() -> Self {
        // Add an impossible filter
        Self::new().filter(Q::raw("1 = 0", vec![]))
    }

    /// Builds the SQL SELECT query and parameters.
    pub fn build_select(&self) -> (String, Vec<SqlValue>) {
        let mut sql = String::new();
        let mut params = Vec::new();

        // SELECT clause
        sql.push_str("SELECT ");
        if self.distinct {
            sql.push_str("DISTINCT ");
        }

        match &self.select_columns {
            Some(cols) => sql.push_str(&cols.join(", ")),
            None => sql.push_str(&M::columns().join(", ")),
        }

        // FROM clause
        sql.push_str(" FROM ");
        sql.push_str(M::table_name());

        // WHERE clause
        let where_clause = self.build_where_clause(&mut params);
        if !where_clause.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&where_clause);
        }

        // ORDER BY clause
        if !self.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            let order_parts: Vec<String> = self.order_by.iter().map(|o| o.to_sql()).collect();
            sql.push_str(&order_parts.join(", "));
        }

        // LIMIT clause
        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {limit}"));
        }

        // OFFSET clause
        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {offset}"));
        }

        (sql, params)
    }

    /// Builds the SQL COUNT query and parameters.
    pub fn build_count(&self) -> (String, Vec<SqlValue>) {
        let mut sql = String::new();
        let mut params = Vec::new();

        sql.push_str("SELECT COUNT(*) FROM ");
        sql.push_str(M::table_name());

        let where_clause = self.build_where_clause(&mut params);
        if !where_clause.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&where_clause);
        }

        (sql, params)
    }

    /// Builds the SQL DELETE query and parameters.
    pub fn build_delete(&self) -> (String, Vec<SqlValue>) {
        let mut sql = String::new();
        let mut params = Vec::new();

        sql.push_str("DELETE FROM ");
        sql.push_str(M::table_name());

        let where_clause = self.build_where_clause(&mut params);
        if !where_clause.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&where_clause);
        }

        (sql, params)
    }

    /// Builds the WHERE clause from filters and excludes.
    fn build_where_clause(&self, params: &mut Vec<SqlValue>) -> String {
        let mut conditions = Vec::new();

        // Add filter conditions
        for filter in &self.filters {
            let (sql, filter_params) = build_filter_expr(filter);
            conditions.push(sql);
            params.extend(filter_params);
        }

        // Add exclude conditions (negated)
        for exclude in &self.excludes {
            let (sql, exclude_params) = build_filter_expr(exclude);
            conditions.push(format!("NOT ({sql})"));
            params.extend(exclude_params);
        }

        conditions.join(" AND ")
    }

    /// Builds an aggregate query.
    pub fn build_aggregate(&self, aggregate: &Aggregate) -> (String, Vec<SqlValue>) {
        let mut sql = String::new();
        let mut params = Vec::new();

        sql.push_str("SELECT ");
        sql.push_str(&aggregate.to_sql());
        sql.push_str(" FROM ");
        sql.push_str(M::table_name());

        let where_clause = self.build_where_clause(&mut params);
        if !where_clause.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&where_clause);
        }

        (sql, params)
    }
}

/// Async execution methods for QuerySet.
impl<M: Model + for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Unpin> QuerySet<M> {
    /// Executes the query and returns all matching rows.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Vec<M>> {
        let (sql, params) = self.build_select();
        let mut query = sqlx::query_as::<_, M>(&sql);

        for param in params {
            query = bind_param(query, param);
        }

        let results = query.fetch_all(pool).await?;
        Ok(results)
    }

    /// Returns the first matching row, or None if no rows match.
    pub async fn first(&self, pool: &SqlitePool) -> Result<Option<M>> {
        let qs = self.clone().limit(1);
        let (sql, params) = qs.build_select();
        let mut query = sqlx::query_as::<_, M>(&sql);

        for param in params {
            query = bind_param(query, param);
        }

        let result = query.fetch_optional(pool).await?;
        Ok(result)
    }

    /// Returns exactly one matching row.
    ///
    /// Returns an error if zero or more than one row matches.
    pub async fn get(&self, pool: &SqlitePool) -> Result<M> {
        let qs = self.clone().limit(2);
        let (sql, params) = qs.build_select();
        let mut query = sqlx::query_as::<_, M>(&sql);

        for param in params {
            query = bind_param(query, param);
        }

        let results = query.fetch_all(pool).await?;

        match results.len() {
            0 => Err(OrmError::NotFound),
            1 => Ok(results.into_iter().next().unwrap()),
            _ => Err(OrmError::MultipleObjectsReturned),
        }
    }

    /// Returns the count of matching rows.
    pub async fn count(&self, pool: &SqlitePool) -> Result<i64> {
        let (sql, params) = self.build_count();
        let mut query = sqlx::query(&sql);

        for param in params {
            query = bind_param_raw(query, param);
        }

        let row = query.fetch_one(pool).await?;
        let count: i64 = row.get(0);
        Ok(count)
    }

    /// Returns whether any rows match the query.
    pub async fn exists(&self, pool: &SqlitePool) -> Result<bool> {
        let count = self.clone().limit(1).count(pool).await?;
        Ok(count > 0)
    }

    /// Deletes all matching rows and returns the count of deleted rows.
    pub async fn delete(&self, pool: &SqlitePool) -> Result<u64> {
        let (sql, params) = self.build_delete();
        let mut query = sqlx::query(&sql);

        for param in params {
            query = bind_param_raw(query, param);
        }

        let result = query.execute(pool).await?;
        Ok(result.rows_affected())
    }

    /// Executes an aggregate query and returns the result as f64.
    pub async fn aggregate(&self, pool: &SqlitePool, agg: Aggregate) -> Result<Option<f64>> {
        let (sql, params) = self.build_aggregate(&agg);
        let mut query = sqlx::query(&sql);

        for param in params {
            query = bind_param_raw(query, param);
        }

        let row = query.fetch_one(pool).await?;
        let value: Option<f64> = row.get(0);
        Ok(value)
    }
}

/// Builds SQL and parameters from a filter expression.
fn build_filter_expr(expr: &FilterExpr) -> (String, Vec<SqlValue>) {
    use crate::query::CompareOp;

    match expr {
        FilterExpr::Comparison { field, op, value } => {
            let op_str = match op {
                CompareOp::Eq => "=",
                CompareOp::Ne => "!=",
                CompareOp::Gt => ">",
                CompareOp::Gte => ">=",
                CompareOp::Lt => "<",
                CompareOp::Lte => "<=",
            };
            (format!("{field} {op_str} ?"), vec![value.clone()])
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

/// Binds a SqlValue parameter to a query_as query.
fn bind_param<'q, M>(
    query: sqlx::query::QueryAs<'q, sqlx::Sqlite, M, sqlx::sqlite::SqliteArguments<'q>>,
    value: SqlValue,
) -> sqlx::query::QueryAs<'q, sqlx::Sqlite, M, sqlx::sqlite::SqliteArguments<'q>>
where
    M: for<'r> FromRow<'r, sqlx::sqlite::SqliteRow>,
{
    match value {
        SqlValue::Null => query.bind(Option::<i64>::None),
        SqlValue::Bool(b) => query.bind(b),
        SqlValue::Int(i) => query.bind(i),
        SqlValue::Float(f) => query.bind(f),
        SqlValue::Text(s) => query.bind(s),
        SqlValue::Blob(b) => query.bind(b),
    }
}

/// Binds a SqlValue parameter to a raw query.
fn bind_param_raw<'q>(
    query: sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>>,
    value: SqlValue,
) -> sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>> {
    match value {
        SqlValue::Null => query.bind(Option::<i64>::None),
        SqlValue::Bool(b) => query.bind(b),
        SqlValue::Int(i) => query.bind(i),
        SqlValue::Float(f) => query.bind(f),
        SqlValue::Text(s) => query.bind(s),
        SqlValue::Blob(b) => query.bind(b),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock model for testing query building
    struct TestModel;

    impl Model for TestModel {
        type Table = TestModelTable;
        type PrimaryKey = i64;

        fn pk_column() -> &'static str {
            "id"
        }

        fn pk(&self) -> i64 {
            0
        }

        fn is_saved(&self) -> bool {
            false
        }
    }

    struct TestModelTable;

    impl oxide_sql_core::schema::Table for TestModelTable {
        type Row = TestModel;
        const NAME: &'static str = "test_models";
        const COLUMNS: &'static [&'static str] = &["id", "name", "email", "created_at"];
        const PRIMARY_KEY: Option<&'static str> = Some("id");
    }

    #[test]
    fn test_basic_select() {
        let qs: QuerySet<TestModel> = QuerySet::new();
        let (sql, params) = qs.build_select();
        assert_eq!(sql, "SELECT id, name, email, created_at FROM test_models");
        assert!(params.is_empty());
    }

    #[test]
    fn test_select_with_filter() {
        let qs: QuerySet<TestModel> = QuerySet::new().filter(Q::eq("name", "Alice"));
        let (sql, params) = qs.build_select();
        assert_eq!(
            sql,
            "SELECT id, name, email, created_at FROM test_models WHERE name = ?"
        );
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_select_with_multiple_filters() {
        let qs: QuerySet<TestModel> = QuerySet::new()
            .filter(Q::eq("name", "Alice"))
            .filter(Q::gt("id", 10));
        let (sql, params) = qs.build_select();
        assert_eq!(
            sql,
            "SELECT id, name, email, created_at FROM test_models WHERE name = ? AND id > ?"
        );
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_select_with_exclude() {
        let qs: QuerySet<TestModel> = QuerySet::new().exclude(Q::eq("name", "Bob"));
        let (sql, params) = qs.build_select();
        assert_eq!(
            sql,
            "SELECT id, name, email, created_at FROM test_models WHERE NOT (name = ?)"
        );
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_select_with_order_by() {
        let qs: QuerySet<TestModel> = QuerySet::new().order_by("-created_at").order_by("name");
        let (sql, _) = qs.build_select();
        assert!(sql.contains("ORDER BY created_at DESC, name ASC"));
    }

    #[test]
    fn test_select_with_limit_offset() {
        let qs: QuerySet<TestModel> = QuerySet::new().limit(10).offset(20);
        let (sql, _) = qs.build_select();
        assert!(sql.contains("LIMIT 10"));
        assert!(sql.contains("OFFSET 20"));
    }

    #[test]
    fn test_select_with_only() {
        let qs: QuerySet<TestModel> = QuerySet::new().only(&["id", "name"]);
        let (sql, _) = qs.build_select();
        assert_eq!(sql, "SELECT id, name FROM test_models");
    }

    #[test]
    fn test_select_distinct() {
        let qs: QuerySet<TestModel> = QuerySet::new().distinct();
        let (sql, _) = qs.build_select();
        assert!(sql.starts_with("SELECT DISTINCT"));
    }

    #[test]
    fn test_count() {
        let qs: QuerySet<TestModel> = QuerySet::new().filter(Q::eq("name", "Alice"));
        let (sql, params) = qs.build_count();
        assert_eq!(sql, "SELECT COUNT(*) FROM test_models WHERE name = ?");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_delete() {
        let qs: QuerySet<TestModel> = QuerySet::new().filter(Q::eq("name", "Alice"));
        let (sql, params) = qs.build_delete();
        assert_eq!(sql, "DELETE FROM test_models WHERE name = ?");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_order_by_parsing() {
        assert_eq!(
            OrderBy::parse("-created_at").direction,
            OrderDirection::Desc
        );
        assert_eq!(OrderBy::parse("name").direction, OrderDirection::Asc);
    }
}
