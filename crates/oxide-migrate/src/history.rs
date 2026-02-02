//! Migration history tracking.
//!
//! This module manages the `oxide_migrations` table that tracks which migrations
//! have been applied to the database.

use chrono::{DateTime, Utc};
use sqlx::sqlite::SqlitePool;

use crate::error::{MigrateError, Result};

/// SQL to create the migrations history table (SQLite).
pub const CREATE_MIGRATIONS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS oxide_migrations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    app TEXT NOT NULL,
    name TEXT NOT NULL,
    applied_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(app, name)
)
"#;

/// A record of an applied migration.
#[derive(Debug, Clone)]
pub struct AppliedMigration {
    /// Unique ID in the migrations table.
    pub id: i64,
    /// Application/module name.
    pub app: String,
    /// Migration name.
    pub name: String,
    /// When the migration was applied.
    pub applied_at: DateTime<Utc>,
}

/// Manages the migration history in the database.
pub struct MigrationHistory {
    pool: SqlitePool,
}

impl MigrationHistory {
    /// Creates a new migration history manager.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Ensures the migrations table exists.
    pub async fn ensure_table(&self) -> Result<()> {
        sqlx::query(CREATE_MIGRATIONS_TABLE_SQL)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Records a migration as applied.
    pub async fn record_applied(&self, app: &str, name: &str) -> Result<()> {
        sqlx::query("INSERT INTO oxide_migrations (app, name) VALUES (?, ?)")
            .bind(app)
            .bind(name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Removes a migration record (for rollback).
    pub async fn record_unapplied(&self, app: &str, name: &str) -> Result<()> {
        let result = sqlx::query("DELETE FROM oxide_migrations WHERE app = ? AND name = ?")
            .bind(app)
            .bind(name)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(MigrateError::MigrationNotFound {
                app: app.to_string(),
                name: name.to_string(),
            });
        }

        Ok(())
    }

    /// Checks if a migration has been applied.
    pub async fn is_applied(&self, app: &str, name: &str) -> Result<bool> {
        let row: Option<(i64,)> =
            sqlx::query_as("SELECT 1 FROM oxide_migrations WHERE app = ? AND name = ?")
                .bind(app)
                .bind(name)
                .fetch_optional(&self.pool)
                .await?;

        Ok(row.is_some())
    }

    /// Gets all applied migrations.
    pub async fn get_applied(&self) -> Result<Vec<AppliedMigration>> {
        let rows: Vec<(i64, String, String, String)> =
            sqlx::query_as("SELECT id, app, name, applied_at FROM oxide_migrations ORDER BY id")
                .fetch_all(&self.pool)
                .await?;

        let mut migrations = Vec::new();
        for (id, app, name, applied_at_str) in rows {
            let applied_at = DateTime::parse_from_rfc3339(&applied_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| {
                    // SQLite datetime format fallback
                    chrono::NaiveDateTime::parse_from_str(&applied_at_str, "%Y-%m-%d %H:%M:%S")
                        .map(|dt| dt.and_utc())
                        .unwrap_or_else(|_| Utc::now())
                });

            migrations.push(AppliedMigration {
                id,
                app,
                name,
                applied_at,
            });
        }

        Ok(migrations)
    }

    /// Gets applied migrations for a specific app.
    pub async fn get_applied_for_app(&self, app: &str) -> Result<Vec<AppliedMigration>> {
        let rows: Vec<(i64, String, String, String)> = sqlx::query_as(
            "SELECT id, app, name, applied_at FROM oxide_migrations WHERE app = ? ORDER BY id",
        )
        .bind(app)
        .fetch_all(&self.pool)
        .await?;

        let mut migrations = Vec::new();
        for (id, app, name, applied_at_str) in rows {
            let applied_at = DateTime::parse_from_rfc3339(&applied_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| {
                    chrono::NaiveDateTime::parse_from_str(&applied_at_str, "%Y-%m-%d %H:%M:%S")
                        .map(|dt| dt.and_utc())
                        .unwrap_or_else(|_| Utc::now())
                });

            migrations.push(AppliedMigration {
                id,
                app,
                name,
                applied_at,
            });
        }

        Ok(migrations)
    }

    /// Gets the last applied migration for an app.
    pub async fn get_last_applied(&self, app: &str) -> Result<Option<AppliedMigration>> {
        let row: Option<(i64, String, String, String)> = sqlx::query_as(
            "SELECT id, app, name, applied_at FROM oxide_migrations WHERE app = ? ORDER BY id DESC LIMIT 1",
        )
        .bind(app)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(id, app, name, applied_at_str)| {
            let applied_at = DateTime::parse_from_rfc3339(&applied_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| {
                    chrono::NaiveDateTime::parse_from_str(&applied_at_str, "%Y-%m-%d %H:%M:%S")
                        .map(|dt| dt.and_utc())
                        .unwrap_or_else(|_| Utc::now())
                });

            AppliedMigration {
                id,
                app,
                name,
                applied_at,
            }
        }))
    }

    /// Counts applied migrations.
    pub async fn count_applied(&self) -> Result<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM oxide_migrations")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }

    /// Counts applied migrations for a specific app.
    pub async fn count_applied_for_app(&self, app: &str) -> Result<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM oxide_migrations WHERE app = ?")
            .bind(app)
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }

    /// Gets a set of applied migration identifiers (app/name pairs).
    pub async fn get_applied_set(&self) -> Result<std::collections::HashSet<(String, String)>> {
        let rows: Vec<(String, String)> = sqlx::query_as("SELECT app, name FROM oxide_migrations")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn create_test_pool() -> SqlitePool {
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect(":memory:")
            .await
            .expect("Failed to create in-memory SQLite pool")
    }

    #[tokio::test]
    async fn test_ensure_table() {
        let pool = create_test_pool().await;
        let history = MigrationHistory::new(pool);

        // Should not fail
        history.ensure_table().await.unwrap();
        // Should be idempotent
        history.ensure_table().await.unwrap();
    }

    #[tokio::test]
    async fn test_record_and_check_applied() {
        let pool = create_test_pool().await;
        let history = MigrationHistory::new(pool);
        history.ensure_table().await.unwrap();

        // Initially not applied
        assert!(!history.is_applied("users", "0001_initial").await.unwrap());

        // Record as applied
        history
            .record_applied("users", "0001_initial")
            .await
            .unwrap();

        // Now it should be applied
        assert!(history.is_applied("users", "0001_initial").await.unwrap());
    }

    #[tokio::test]
    async fn test_get_applied() {
        let pool = create_test_pool().await;
        let history = MigrationHistory::new(pool);
        history.ensure_table().await.unwrap();

        history
            .record_applied("users", "0001_initial")
            .await
            .unwrap();
        history
            .record_applied("users", "0002_add_email")
            .await
            .unwrap();
        history
            .record_applied("posts", "0001_initial")
            .await
            .unwrap();

        let all = history.get_applied().await.unwrap();
        assert_eq!(all.len(), 3);

        let users_only = history.get_applied_for_app("users").await.unwrap();
        assert_eq!(users_only.len(), 2);
    }

    #[tokio::test]
    async fn test_record_unapplied() {
        let pool = create_test_pool().await;
        let history = MigrationHistory::new(pool);
        history.ensure_table().await.unwrap();

        history
            .record_applied("users", "0001_initial")
            .await
            .unwrap();
        assert!(history.is_applied("users", "0001_initial").await.unwrap());

        history
            .record_unapplied("users", "0001_initial")
            .await
            .unwrap();
        assert!(!history.is_applied("users", "0001_initial").await.unwrap());
    }

    #[tokio::test]
    async fn test_get_last_applied() {
        let pool = create_test_pool().await;
        let history = MigrationHistory::new(pool);
        history.ensure_table().await.unwrap();

        assert!(history.get_last_applied("users").await.unwrap().is_none());

        history
            .record_applied("users", "0001_initial")
            .await
            .unwrap();
        history
            .record_applied("users", "0002_add_email")
            .await
            .unwrap();

        let last = history.get_last_applied("users").await.unwrap().unwrap();
        assert_eq!(last.name, "0002_add_email");
    }
}
