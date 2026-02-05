//! Manager for database access.
//!
//! The Manager provides the primary interface for database operations,
//! similar to Django's Manager class.

use oxide_sql_core::builder::value::{SqlValue, ToSqlValue};
use sqlx::{FromRow, Row, SqlitePool};
use std::marker::PhantomData;

use crate::error::{OrmError, Result};
use crate::model::Model;
use crate::query::Q;
use crate::queryset::QuerySet;

/// A Manager provides database access methods for a Model.
///
/// Each Model has a default Manager accessible via `Model::objects()`.
/// Managers are lightweight and can be created freely.
///
/// # Example
///
/// ```ignore
/// use oxide_orm::Model;
///
/// // Get all users
/// let users = User::objects().all().execute(&pool).await?;
///
/// // Get a specific user by primary key
/// let user = User::objects().get(&pool, 1).await?;
///
/// // Create a new user
/// let user = User::objects().create(&pool, User {
///     id: 0,
///     username: "alice".to_string(),
///     email: "alice@example.com".to_string(),
/// }).await?;
/// ```
#[derive(Debug)]
pub struct Manager<M: Model> {
    _marker: PhantomData<M>,
}

impl<M: Model> Clone for Manager<M> {
    fn clone(&self) -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<M: Model> Copy for Manager<M> {}

impl<M: Model> Default for Manager<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: Model> Manager<M> {
    /// Creates a new Manager.
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    /// Returns a QuerySet for all objects.
    pub fn all(&self) -> QuerySet<M> {
        QuerySet::new()
    }

    /// Returns a QuerySet filtered by the given Q expression.
    pub fn filter(&self, q: Q) -> QuerySet<M> {
        QuerySet::new().filter(q)
    }

    /// Returns a QuerySet excluding objects matching the Q expression.
    pub fn exclude(&self, q: Q) -> QuerySet<M> {
        QuerySet::new().exclude(q)
    }

    /// Returns a QuerySet with no results.
    pub fn none(&self) -> QuerySet<M> {
        QuerySet::none()
    }
}

/// Async methods for Manager.
impl<M: Model + for<'r> FromRow<'r, sqlx::sqlite::SqliteRow> + Unpin> Manager<M> {
    /// Gets an object by its primary key.
    pub async fn get(&self, pool: &SqlitePool, pk: M::PrimaryKey) -> Result<M> {
        let pk_column = M::pk_column();
        let columns = M::columns().join(", ");
        let table = M::table_name();

        let sql = format!("SELECT {columns} FROM {table} WHERE {pk_column} = ?");
        let mut query = sqlx::query_as::<_, M>(&sql);

        query = bind_param(query, pk.to_sql_value());

        let result = query.fetch_optional(pool).await?;
        result.ok_or(OrmError::NotFound)
    }

    /// Gets an object by its primary key, returning None if not found.
    pub async fn get_or_none(&self, pool: &SqlitePool, pk: M::PrimaryKey) -> Result<Option<M>> {
        let pk_column = M::pk_column();
        let columns = M::columns().join(", ");
        let table = M::table_name();

        let sql = format!("SELECT {columns} FROM {table} WHERE {pk_column} = ?");
        let mut query = sqlx::query_as::<_, M>(&sql);

        query = bind_param(query, pk.to_sql_value());

        let result = query.fetch_optional(pool).await?;
        Ok(result)
    }

    /// Returns the count of all objects.
    pub async fn count(&self, pool: &SqlitePool) -> Result<i64> {
        let table = M::table_name();
        let sql = format!("SELECT COUNT(*) FROM {table}");

        let row = sqlx::query(&sql).fetch_one(pool).await?;
        let count: i64 = row.get(0);
        Ok(count)
    }

    /// Returns whether any objects exist.
    pub async fn exists(&self, pool: &SqlitePool) -> Result<bool> {
        let count = self.count(pool).await?;
        Ok(count > 0)
    }

    /// Returns the first object, or None if no objects exist.
    pub async fn first(&self, pool: &SqlitePool) -> Result<Option<M>> {
        self.all().first(pool).await
    }

    /// Returns the last object based on primary key, or None if no objects exist.
    pub async fn last(&self, pool: &SqlitePool) -> Result<Option<M>> {
        let pk_column = M::pk_column();
        self.all()
            .order_by(&format!("-{pk_column}"))
            .first(pool)
            .await
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

#[cfg(test)]
mod tests {
    use super::*;

    // Tests are in integration tests since Manager needs database access
}
