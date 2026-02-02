//! Migration state tracking.
//!
//! This module provides functionality to track which migrations have been
//! applied to the database, similar to Django's migration tracking.

use std::collections::HashSet;

/// SQL for creating the migrations tracking table (SQLite/PostgreSQL compatible).
pub const MIGRATIONS_TABLE_SQL: &str = r"
CREATE TABLE IF NOT EXISTS _oxide_migrations (
    id VARCHAR(255) PRIMARY KEY,
    applied_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
)
";

/// SQL for inserting a migration record.
pub const INSERT_MIGRATION_SQL: &str =
    "INSERT INTO _oxide_migrations (id, applied_at) VALUES (?, CURRENT_TIMESTAMP)";

/// SQL for deleting a migration record.
pub const DELETE_MIGRATION_SQL: &str = "DELETE FROM _oxide_migrations WHERE id = ?";

/// SQL for checking if a migration is applied.
pub const CHECK_MIGRATION_SQL: &str = "SELECT 1 FROM _oxide_migrations WHERE id = ?";

/// SQL for listing all applied migrations.
pub const LIST_MIGRATIONS_SQL: &str =
    "SELECT id, applied_at FROM _oxide_migrations ORDER BY applied_at";

/// Tracks which migrations have been applied.
///
/// This struct provides an in-memory representation of the migration state.
/// In a real application, you would load this from the database using the
/// SQL constants provided in this module.
///
/// # Example
///
/// ```rust
/// use oxide_sql_core::migrations::MigrationState;
///
/// let mut state = MigrationState::new();
///
/// // Check if a migration is applied
/// assert!(!state.is_applied("0001_initial"));
///
/// // Mark a migration as applied
/// state.mark_applied("0001_initial");
/// assert!(state.is_applied("0001_initial"));
///
/// // Mark a migration as unapplied (rolled back)
/// state.mark_unapplied("0001_initial");
/// assert!(!state.is_applied("0001_initial"));
/// ```
#[derive(Debug, Clone, Default)]
pub struct MigrationState {
    /// Set of applied migration IDs.
    applied: HashSet<String>,
}

impl MigrationState {
    /// Creates a new empty migration state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a migration state from a list of applied migration IDs.
    ///
    /// This is useful for loading state from the database.
    #[must_use]
    pub fn from_applied(applied: impl IntoIterator<Item = String>) -> Self {
        Self {
            applied: applied.into_iter().collect(),
        }
    }

    /// Checks if a migration has been applied.
    #[must_use]
    pub fn is_applied(&self, id: &str) -> bool {
        self.applied.contains(id)
    }

    /// Marks a migration as applied.
    pub fn mark_applied(&mut self, id: impl Into<String>) {
        self.applied.insert(id.into());
    }

    /// Marks a migration as unapplied (rolled back).
    pub fn mark_unapplied(&mut self, id: &str) {
        self.applied.remove(id);
    }

    /// Returns an iterator over all applied migration IDs.
    pub fn applied_migrations(&self) -> impl Iterator<Item = &str> {
        self.applied.iter().map(String::as_str)
    }

    /// Returns the number of applied migrations.
    #[must_use]
    pub fn applied_count(&self) -> usize {
        self.applied.len()
    }

    /// Returns the SQL to create the migrations tracking table.
    #[must_use]
    pub const fn create_table_sql() -> &'static str {
        MIGRATIONS_TABLE_SQL
    }

    /// Returns the SQL to insert a migration record.
    #[must_use]
    pub const fn insert_sql() -> &'static str {
        INSERT_MIGRATION_SQL
    }

    /// Returns the SQL to delete a migration record.
    #[must_use]
    pub const fn delete_sql() -> &'static str {
        DELETE_MIGRATION_SQL
    }

    /// Returns the SQL to check if a migration is applied.
    #[must_use]
    pub const fn check_sql() -> &'static str {
        CHECK_MIGRATION_SQL
    }

    /// Returns the SQL to list all applied migrations.
    #[must_use]
    pub const fn list_sql() -> &'static str {
        LIST_MIGRATIONS_SQL
    }
}

/// Information about a migration's application status.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct AppliedMigration {
    /// The migration ID.
    pub id: String,
    /// When the migration was applied (ISO 8601 format).
    pub applied_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_state_is_empty() {
        let state = MigrationState::new();
        assert_eq!(state.applied_count(), 0);
        assert!(!state.is_applied("anything"));
    }

    #[test]
    fn test_mark_applied() {
        let mut state = MigrationState::new();
        state.mark_applied("0001_initial");

        assert!(state.is_applied("0001_initial"));
        assert!(!state.is_applied("0002_add_users"));
        assert_eq!(state.applied_count(), 1);
    }

    #[test]
    fn test_mark_unapplied() {
        let mut state = MigrationState::new();
        state.mark_applied("0001_initial");
        state.mark_applied("0002_add_users");

        assert_eq!(state.applied_count(), 2);

        state.mark_unapplied("0002_add_users");
        assert!(!state.is_applied("0002_add_users"));
        assert!(state.is_applied("0001_initial"));
        assert_eq!(state.applied_count(), 1);
    }

    #[test]
    fn test_from_applied() {
        let state = MigrationState::from_applied(vec![
            "0001_initial".to_string(),
            "0002_add_users".to_string(),
        ]);

        assert!(state.is_applied("0001_initial"));
        assert!(state.is_applied("0002_add_users"));
        assert!(!state.is_applied("0003_something"));
        assert_eq!(state.applied_count(), 2);
    }

    #[test]
    fn test_applied_migrations_iterator() {
        let mut state = MigrationState::new();
        state.mark_applied("0001_initial");
        state.mark_applied("0002_add_users");

        let applied: HashSet<&str> = state.applied_migrations().collect();
        assert!(applied.contains("0001_initial"));
        assert!(applied.contains("0002_add_users"));
    }

    #[test]
    fn test_sql_constants() {
        assert!(MigrationState::create_table_sql().contains("CREATE TABLE"));
        assert!(MigrationState::insert_sql().contains("INSERT"));
        assert!(MigrationState::delete_sql().contains("DELETE"));
        assert!(MigrationState::check_sql().contains("SELECT"));
        assert!(MigrationState::list_sql().contains("SELECT"));
    }
}
