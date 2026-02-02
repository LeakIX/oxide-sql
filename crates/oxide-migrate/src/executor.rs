//! Migration executor.
//!
//! This module handles applying and rolling back migrations against a database.

use sqlx::sqlite::SqlitePool;
use tracing::{debug, info, warn};

use crate::dialect::MigrationDialect;
use crate::error::{MigrateError, Result};
use crate::history::MigrationHistory;
use crate::operations::MigrationOperation;

/// A migration ready to be executed.
#[derive(Debug, Clone)]
pub struct ExecutableMigration {
    /// Application/module name.
    pub app: String,
    /// Migration name.
    pub name: String,
    /// Migration operations.
    pub operations: Vec<MigrationOperation>,
    /// Dependencies (app/name pairs).
    pub dependencies: Vec<(String, String)>,
}

impl ExecutableMigration {
    /// Creates a new executable migration.
    #[must_use]
    pub fn new(app: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            app: app.into(),
            name: name.into(),
            operations: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    /// Adds an operation to this migration.
    #[must_use]
    pub fn operation(mut self, op: MigrationOperation) -> Self {
        self.operations.push(op);
        self
    }

    /// Adds operations to this migration.
    #[must_use]
    pub fn operations(mut self, ops: Vec<MigrationOperation>) -> Self {
        self.operations.extend(ops);
        self
    }

    /// Adds a dependency.
    #[must_use]
    pub fn depends_on(mut self, app: impl Into<String>, name: impl Into<String>) -> Self {
        self.dependencies.push((app.into(), name.into()));
        self
    }

    /// Returns the full migration identifier.
    #[must_use]
    pub fn id(&self) -> String {
        format!("{}/{}", self.app, self.name)
    }

    /// Returns whether this migration is reversible.
    #[must_use]
    pub fn is_reversible(&self) -> bool {
        self.operations.iter().all(|op| op.is_reversible())
    }

    /// Returns the reverse operations for rollback.
    #[must_use]
    pub fn reverse_operations(&self) -> Option<Vec<MigrationOperation>> {
        let reversed: Vec<Option<MigrationOperation>> = self
            .operations
            .iter()
            .rev()
            .map(|op| op.reverse())
            .collect();

        if reversed.iter().all(|op| op.is_some()) {
            Some(reversed.into_iter().flatten().collect())
        } else {
            None
        }
    }
}

/// Executes migrations against a database.
pub struct MigrationExecutor<D: MigrationDialect> {
    pool: SqlitePool,
    dialect: D,
    history: MigrationHistory,
    dry_run: bool,
}

impl<D: MigrationDialect> MigrationExecutor<D> {
    /// Creates a new migration executor.
    pub fn new(pool: SqlitePool, dialect: D) -> Self {
        let history = MigrationHistory::new(pool.clone());
        Self {
            pool,
            dialect,
            history,
            dry_run: false,
        }
    }

    /// Enables dry-run mode (SQL is printed but not executed).
    #[must_use]
    pub fn dry_run(mut self, enabled: bool) -> Self {
        self.dry_run = enabled;
        self
    }

    /// Ensures the migrations history table exists.
    pub async fn init(&self) -> Result<()> {
        if !self.dry_run {
            self.history.ensure_table().await?;
        }
        Ok(())
    }

    /// Returns the migration history.
    #[must_use]
    pub fn history(&self) -> &MigrationHistory {
        &self.history
    }

    /// Returns the dialect.
    #[must_use]
    pub fn dialect(&self) -> &D {
        &self.dialect
    }

    /// Checks if a migration has been applied.
    pub async fn is_applied(&self, app: &str, name: &str) -> Result<bool> {
        self.history.is_applied(app, name).await
    }

    /// Applies a single migration.
    pub async fn apply(&self, migration: &ExecutableMigration) -> Result<()> {
        info!(
            app = %migration.app,
            name = %migration.name,
            "Applying migration"
        );

        // Check if already applied (skip in dry_run mode)
        if !self.dry_run
            && self
                .history
                .is_applied(&migration.app, &migration.name)
                .await?
        {
            warn!(
                app = %migration.app,
                name = %migration.name,
                "Migration already applied, skipping"
            );
            return Ok(());
        }

        // Check dependencies (skip in dry_run mode)
        if !self.dry_run {
            for (dep_app, dep_name) in &migration.dependencies {
                if !self.history.is_applied(dep_app, dep_name).await? {
                    return Err(MigrateError::MissingDependency {
                        migration: migration.id(),
                        dependency: format!("{}/{}", dep_app, dep_name),
                    });
                }
            }
        }

        // Generate and execute SQL
        for operation in &migration.operations {
            let statements = self.dialect.generate_sql(operation);
            for sql in statements {
                debug!(sql = %sql, "Executing SQL");

                if self.dry_run {
                    println!("{};", sql);
                } else {
                    // Skip comments
                    if sql.starts_with("--") {
                        warn!(comment = %sql, "Skipping comment (unsupported operation)");
                        continue;
                    }
                    sqlx::query(&sql).execute(&self.pool).await?;
                }
            }
        }

        // Record as applied
        if !self.dry_run {
            self.history
                .record_applied(&migration.app, &migration.name)
                .await?;
        }

        info!(
            app = %migration.app,
            name = %migration.name,
            "Migration applied successfully"
        );

        Ok(())
    }

    /// Rolls back a single migration.
    pub async fn rollback(&self, migration: &ExecutableMigration) -> Result<()> {
        info!(
            app = %migration.app,
            name = %migration.name,
            "Rolling back migration"
        );

        // Check if applied
        if !self
            .history
            .is_applied(&migration.app, &migration.name)
            .await?
        {
            warn!(
                app = %migration.app,
                name = %migration.name,
                "Migration not applied, skipping rollback"
            );
            return Ok(());
        }

        // Check if reversible
        let reverse_ops = migration
            .reverse_operations()
            .ok_or_else(|| MigrateError::NotReversible(migration.id()))?;

        // Generate and execute reverse SQL
        for operation in &reverse_ops {
            let statements = self.dialect.generate_sql(operation);
            for sql in statements {
                debug!(sql = %sql, "Executing rollback SQL");

                if self.dry_run {
                    println!("{};", sql);
                } else {
                    if sql.starts_with("--") {
                        warn!(comment = %sql, "Skipping comment (unsupported operation)");
                        continue;
                    }
                    sqlx::query(&sql).execute(&self.pool).await?;
                }
            }
        }

        // Remove from history
        if !self.dry_run {
            self.history
                .record_unapplied(&migration.app, &migration.name)
                .await?;
        }

        info!(
            app = %migration.app,
            name = %migration.name,
            "Migration rolled back successfully"
        );

        Ok(())
    }

    /// Applies multiple migrations in order.
    pub async fn apply_all(&self, migrations: &[ExecutableMigration]) -> Result<()> {
        for migration in migrations {
            self.apply(migration).await?;
        }
        Ok(())
    }

    /// Rolls back multiple migrations in reverse order.
    pub async fn rollback_all(&self, migrations: &[ExecutableMigration]) -> Result<()> {
        for migration in migrations.iter().rev() {
            self.rollback(migration).await?;
        }
        Ok(())
    }

    /// Returns pending migrations (not yet applied).
    pub async fn pending<'a>(
        &self,
        migrations: &'a [ExecutableMigration],
    ) -> Result<Vec<&'a ExecutableMigration>> {
        let applied = self.history.get_applied_set().await?;
        Ok(migrations
            .iter()
            .filter(|m| !applied.contains(&(m.app.clone(), m.name.clone())))
            .collect())
    }

    /// Generates SQL for a migration without executing it.
    #[must_use]
    pub fn sql_for(&self, migration: &ExecutableMigration) -> Vec<String> {
        let mut all_sql = Vec::new();
        for operation in &migration.operations {
            all_sql.extend(self.dialect.generate_sql(operation));
        }
        all_sql
    }

    /// Generates rollback SQL for a migration.
    #[must_use]
    pub fn rollback_sql_for(&self, migration: &ExecutableMigration) -> Option<Vec<String>> {
        let reverse_ops = migration.reverse_operations()?;
        let mut all_sql = Vec::new();
        for operation in &reverse_ops {
            all_sql.extend(self.dialect.generate_sql(operation));
        }
        Some(all_sql)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialect::SqliteDialect;
    use crate::schema::{ColumnSchema, SqlType};
    use sqlx::sqlite::SqlitePoolOptions;

    async fn create_test_pool() -> SqlitePool {
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect(":memory:")
            .await
            .expect("Failed to create in-memory SQLite pool")
    }

    fn create_users_migration() -> ExecutableMigration {
        ExecutableMigration::new("users", "0001_initial").operation(
            MigrationOperation::CreateTable {
                name: "users".to_string(),
                columns: vec![
                    ColumnSchema::new("id", SqlType::BigInt)
                        .primary_key()
                        .auto_increment(),
                    ColumnSchema::new("username", SqlType::Varchar(255)).not_null(),
                ],
                primary_key: vec!["id".to_string()],
                if_not_exists: false,
            },
        )
    }

    fn add_email_migration() -> ExecutableMigration {
        ExecutableMigration::new("users", "0002_add_email")
            .depends_on("users", "0001_initial")
            .operation(MigrationOperation::AddColumn {
                table: "users".to_string(),
                column: ColumnSchema::new("email", SqlType::Varchar(255)),
            })
    }

    #[tokio::test]
    async fn test_apply_migration() {
        let pool = create_test_pool().await;
        let executor = MigrationExecutor::new(pool, SqliteDialect::new());
        executor.init().await.unwrap();

        let migration = create_users_migration();
        executor.apply(&migration).await.unwrap();

        // Verify table was created
        let row: Option<(String,)> =
            sqlx::query_as("SELECT name FROM sqlite_master WHERE type='table' AND name='users'")
                .fetch_optional(&executor.pool)
                .await
                .unwrap();
        assert!(row.is_some());

        // Verify migration was recorded
        assert!(executor.is_applied("users", "0001_initial").await.unwrap());
    }

    #[tokio::test]
    async fn test_apply_idempotent() {
        let pool = create_test_pool().await;
        let executor = MigrationExecutor::new(pool, SqliteDialect::new());
        executor.init().await.unwrap();

        let migration = create_users_migration();

        // Apply twice - should not error
        executor.apply(&migration).await.unwrap();
        executor.apply(&migration).await.unwrap();
    }

    #[tokio::test]
    async fn test_apply_with_dependency() {
        let pool = create_test_pool().await;
        let executor = MigrationExecutor::new(pool, SqliteDialect::new());
        executor.init().await.unwrap();

        let m1 = create_users_migration();
        let m2 = add_email_migration();

        // Apply first migration
        executor.apply(&m1).await.unwrap();

        // Apply second migration (depends on first)
        executor.apply(&m2).await.unwrap();

        // Verify both applied
        assert!(executor.is_applied("users", "0001_initial").await.unwrap());
        assert!(
            executor
                .is_applied("users", "0002_add_email")
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_apply_missing_dependency() {
        let pool = create_test_pool().await;
        let executor = MigrationExecutor::new(pool, SqliteDialect::new());
        executor.init().await.unwrap();

        let m2 = add_email_migration();

        // Should fail because 0001_initial not applied
        let result = executor.apply(&m2).await;
        assert!(matches!(
            result,
            Err(MigrateError::MissingDependency { .. })
        ));
    }

    #[tokio::test]
    async fn test_rollback_migration() {
        let pool = create_test_pool().await;
        let executor = MigrationExecutor::new(pool, SqliteDialect::new());
        executor.init().await.unwrap();

        let migration = create_users_migration();
        executor.apply(&migration).await.unwrap();

        // Verify table exists
        assert!(executor.is_applied("users", "0001_initial").await.unwrap());

        // Rollback
        executor.rollback(&migration).await.unwrap();

        // Verify table was dropped
        let row: Option<(String,)> =
            sqlx::query_as("SELECT name FROM sqlite_master WHERE type='table' AND name='users'")
                .fetch_optional(&executor.pool)
                .await
                .unwrap();
        assert!(row.is_none());

        // Verify migration record removed
        assert!(!executor.is_applied("users", "0001_initial").await.unwrap());
    }

    #[tokio::test]
    async fn test_pending_migrations() {
        let pool = create_test_pool().await;
        let executor = MigrationExecutor::new(pool, SqliteDialect::new());
        executor.init().await.unwrap();

        let m1 = create_users_migration();
        let m2 = add_email_migration();
        let migrations = vec![m1.clone(), m2.clone()];

        // Both should be pending
        let pending = executor.pending(&migrations).await.unwrap();
        assert_eq!(pending.len(), 2);

        // Apply first
        executor.apply(&m1).await.unwrap();

        // Only second should be pending
        let pending = executor.pending(&migrations).await.unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].name, "0002_add_email");
    }

    #[tokio::test]
    async fn test_sql_generation() {
        let pool = create_test_pool().await;
        let executor = MigrationExecutor::new(pool, SqliteDialect::new());

        let migration = create_users_migration();
        let sql = executor.sql_for(&migration);

        assert_eq!(sql.len(), 1);
        assert!(sql[0].contains("CREATE TABLE"));
    }

    #[tokio::test]
    async fn test_dry_run() {
        let pool = create_test_pool().await;
        let executor = MigrationExecutor::new(pool.clone(), SqliteDialect::new()).dry_run(true);

        let migration = create_users_migration();
        executor.apply(&migration).await.unwrap();

        // Table should NOT exist (dry run)
        let row: Option<(String,)> =
            sqlx::query_as("SELECT name FROM sqlite_master WHERE type='table' AND name='users'")
                .fetch_optional(&pool)
                .await
                .unwrap();
        assert!(row.is_none());
    }
}
