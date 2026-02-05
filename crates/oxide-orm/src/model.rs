//! Model trait and related types.
//!
//! The `Model` trait extends the `Table` trait from oxide-sql-core to provide
//! Django-like ORM functionality including Manager access and instance methods.

use oxide_sql_core::builder::value::ToSqlValue;
use oxide_sql_core::schema::Table;

use crate::manager::Manager;

/// A database model with ORM capabilities.
///
/// This trait extends `Table` to provide Django-like ORM features:
/// - Access to a `Manager` for database operations
/// - Primary key accessor
/// - Instance methods for save/delete operations
///
/// # Example
///
/// ```ignore
/// use oxide_orm::Model;
///
/// #[derive(Model)]
/// #[model(table = "users")]
/// struct User {
///     #[field(primary_key, auto)]
///     id: i64,
///     #[field(max_length = 255)]
///     username: String,
///     email: String,
/// }
///
/// // Access the manager
/// let users = User::objects().all();
///
/// // Get by primary key
/// let user = User::objects().get(&pool, 1).await?;
/// ```
pub trait Model: Sized + Send + Sync + 'static {
    /// The table type implementing `Table`.
    type Table: Table<Row = Self>;

    /// The primary key type.
    type PrimaryKey: ToSqlValue + Clone + Send + Sync;

    /// Returns the table name.
    fn table_name() -> &'static str {
        Self::Table::NAME
    }

    /// Returns the primary key column name.
    fn pk_column() -> &'static str;

    /// Returns all column names.
    fn columns() -> &'static [&'static str] {
        Self::Table::COLUMNS
    }

    /// Returns the primary key value for this instance.
    fn pk(&self) -> Self::PrimaryKey;

    /// Returns a new Manager for this model.
    fn objects() -> Manager<Self> {
        Manager::new()
    }

    /// Returns whether this instance has been saved to the database.
    ///
    /// By default, this checks if the primary key is non-zero/non-empty.
    /// Override this for custom primary key behavior.
    fn is_saved(&self) -> bool;
}

/// Trait for models that can be saved and deleted.
///
/// This trait is typically implemented by the derive macro and provides
/// async methods for persisting changes to the database.
#[allow(async_fn_in_trait)]
pub trait ModelInstance: Model {
    /// Saves this instance to the database.
    ///
    /// If the instance is new (not yet saved), performs an INSERT.
    /// If the instance already exists, performs an UPDATE.
    async fn save(&mut self, pool: &sqlx::SqlitePool) -> crate::Result<()>;

    /// Deletes this instance from the database.
    async fn delete(&self, pool: &sqlx::SqlitePool) -> crate::Result<()>;

    /// Refreshes this instance from the database.
    async fn refresh_from_db(&mut self, pool: &sqlx::SqlitePool) -> crate::Result<()>;
}
