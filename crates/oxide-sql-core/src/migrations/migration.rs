//! Migration trait and runner.
//!
//! Provides the `Migration` trait that all migrations implement, and the
//! `MigrationRunner` that executes migrations in dependency order.

use std::collections::{HashMap, HashSet, VecDeque};

use super::dialect::MigrationDialect;
use super::operation::Operation;
use super::state::MigrationState;

/// A database migration with typed up/down operations.
///
/// Implement this trait for each migration in your application.
///
/// # Example
///
/// ```rust
/// use oxide_sql_core::migrations::{
///     Migration, Operation, CreateTableBuilder,
///     bigint, varchar, timestamp,
/// };
///
/// pub struct Migration0001;
///
/// impl Migration for Migration0001 {
///     const ID: &'static str = "0001_create_users";
///
///     fn up() -> Vec<Operation> {
///         vec![
///             CreateTableBuilder::new()
///                 .name("users")
///                 .column(bigint("id").primary_key().autoincrement().build())
///                 .column(varchar("username", 255).not_null().unique().build())
///                 .build()
///                 .into(),
///         ]
///     }
///
///     fn down() -> Vec<Operation> {
///         vec![
///             Operation::drop_table("users"),
///         ]
///     }
/// }
/// ```
pub trait Migration {
    /// Unique migration identifier (e.g., "0001_initial", "0002_add_email").
    ///
    /// This ID is stored in the migrations table to track which migrations
    /// have been applied.
    const ID: &'static str;

    /// Dependencies on other migrations (must run first).
    ///
    /// Each string should be the `ID` of another migration.
    const DEPENDENCIES: &'static [&'static str] = &[];

    /// Apply the migration (forward).
    ///
    /// Returns a list of operations to execute.
    fn up() -> Vec<Operation>;

    /// Reverse the migration (backward).
    ///
    /// Returns a list of operations to execute to undo the migration.
    /// Return an empty vec if the migration is not reversible.
    fn down() -> Vec<Operation>;
}

/// A registered migration with runtime-accessible metadata.
pub struct RegisteredMigration {
    /// Migration ID.
    pub id: &'static str,
    /// Dependencies.
    pub dependencies: &'static [&'static str],
    /// Function to get up operations.
    pub up: fn() -> Vec<Operation>,
    /// Function to get down operations.
    pub down: fn() -> Vec<Operation>,
}

impl RegisteredMigration {
    /// Creates a new registered migration from a `Migration` implementor.
    #[must_use]
    pub const fn new<M: Migration>() -> Self {
        Self {
            id: M::ID,
            dependencies: M::DEPENDENCIES,
            up: M::up,
            down: M::down,
        }
    }
}

/// Status of a migration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationStatus {
    /// The migration ID.
    pub id: &'static str,
    /// Whether the migration has been applied.
    pub applied: bool,
    /// When the migration was applied (if known).
    pub applied_at: Option<String>,
}

/// Runs migrations in dependency order.
///
/// The runner tracks which migrations are registered and uses the provided
/// `MigrationState` to determine which migrations need to be applied.
///
/// # Example
///
/// ```rust
/// use oxide_sql_core::migrations::{
///     Migration, MigrationRunner, MigrationState, Operation,
///     CreateTableBuilder, SqliteDialect, bigint,
/// };
///
/// // Define a migration
/// pub struct Migration0001;
/// impl Migration for Migration0001 {
///     const ID: &'static str = "0001_initial";
///     fn up() -> Vec<Operation> {
///         vec![CreateTableBuilder::new()
///             .name("test")
///             .column(bigint("id").primary_key().build())
///             .build()
///             .into()]
///     }
///     fn down() -> Vec<Operation> {
///         vec![Operation::drop_table("test")]
///     }
/// }
///
/// // Create runner
/// let mut runner = MigrationRunner::new(SqliteDialect::new());
/// runner.register::<Migration0001>();
///
/// // Check status
/// let state = MigrationState::new();
/// let pending = runner.pending_migrations(&state);
/// assert_eq!(pending.len(), 1);
/// ```
pub struct MigrationRunner<D: MigrationDialect> {
    migrations: Vec<RegisteredMigration>,
    dialect: D,
}

impl<D: MigrationDialect> MigrationRunner<D> {
    /// Creates a new migration runner with the given dialect.
    #[must_use]
    pub fn new(dialect: D) -> Self {
        Self {
            migrations: Vec::new(),
            dialect,
        }
    }

    /// Registers a migration.
    pub fn register<M: Migration>(&mut self) -> &mut Self {
        self.migrations.push(RegisteredMigration::new::<M>());
        self
    }

    /// Returns all registered migrations.
    #[must_use]
    pub fn migrations(&self) -> &[RegisteredMigration] {
        &self.migrations
    }

    /// Returns the dialect.
    #[must_use]
    pub fn dialect(&self) -> &D {
        &self.dialect
    }

    /// Returns migrations that haven't been applied yet.
    #[must_use]
    pub fn pending_migrations(&self, state: &MigrationState) -> Vec<&RegisteredMigration> {
        self.migrations
            .iter()
            .filter(|m| !state.is_applied(m.id))
            .collect()
    }

    /// Returns the status of all migrations.
    #[must_use]
    pub fn status(&self, state: &MigrationState) -> Vec<MigrationStatus> {
        self.migrations
            .iter()
            .map(|m| MigrationStatus {
                id: m.id,
                applied: state.is_applied(m.id),
                applied_at: None, // Would need to query the DB for this
            })
            .collect()
    }

    /// Returns migrations in dependency order (topological sort).
    ///
    /// Returns `Err` if there's a circular dependency.
    pub fn sorted_migrations(&self) -> Result<Vec<&RegisteredMigration>, MigrationError> {
        // Build dependency graph
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();
        let migration_map: HashMap<&str, &RegisteredMigration> =
            self.migrations.iter().map(|m| (m.id, m)).collect();

        for m in &self.migrations {
            in_degree.entry(m.id).or_insert(0);
            for dep in m.dependencies {
                *in_degree.entry(m.id).or_insert(0) += 1;
                dependents.entry(*dep).or_default().push(m.id);
            }
        }

        // Kahn's algorithm for topological sort
        let mut queue: VecDeque<&str> = in_degree
            .iter()
            .filter(|(_, deg)| **deg == 0)
            .map(|(id, _)| *id)
            .collect();
        let mut result = Vec::new();

        while let Some(id) = queue.pop_front() {
            if let Some(m) = migration_map.get(id) {
                result.push(*m);
            }

            if let Some(deps) = dependents.get(id) {
                for dep in deps {
                    if let Some(deg) = in_degree.get_mut(dep) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(dep);
                        }
                    }
                }
            }
        }

        if result.len() != self.migrations.len() {
            return Err(MigrationError::CircularDependency);
        }

        Ok(result)
    }

    /// Generates SQL for all pending migrations.
    ///
    /// Returns a list of (migration_id, sql_statements) pairs.
    pub fn sql_for_pending(
        &self,
        state: &MigrationState,
    ) -> Result<Vec<(&'static str, Vec<String>)>, MigrationError> {
        let sorted = self.sorted_migrations()?;
        let pending: Vec<_> = sorted
            .into_iter()
            .filter(|m| !state.is_applied(m.id))
            .collect();

        let mut result = Vec::new();
        for migration in pending {
            let operations = (migration.up)();
            let sqls: Vec<String> = operations
                .iter()
                .map(|op| self.dialect.generate_sql(op))
                .collect();
            result.push((migration.id, sqls));
        }

        Ok(result)
    }

    /// Generates SQL for rolling back migrations.
    ///
    /// Returns a list of (migration_id, sql_statements) pairs in reverse order.
    pub fn sql_for_rollback(
        &self,
        state: &MigrationState,
        count: usize,
    ) -> Result<Vec<(&'static str, Vec<String>)>, MigrationError> {
        let sorted = self.sorted_migrations()?;

        // Get applied migrations in reverse order
        let applied: Vec<_> = sorted
            .into_iter()
            .rev()
            .filter(|m| state.is_applied(m.id))
            .take(count)
            .collect();

        let mut result = Vec::new();
        for migration in applied {
            let operations = (migration.down)();
            if operations.is_empty() {
                return Err(MigrationError::NotReversible(migration.id.to_string()));
            }
            let sqls: Vec<String> = operations
                .iter()
                .map(|op| self.dialect.generate_sql(op))
                .collect();
            result.push((migration.id, sqls));
        }

        Ok(result)
    }

    /// Validates that all dependencies exist and are registered.
    pub fn validate(&self) -> Result<(), MigrationError> {
        let ids: HashSet<&str> = self.migrations.iter().map(|m| m.id).collect();

        for m in &self.migrations {
            for dep in m.dependencies {
                if !ids.contains(dep) {
                    return Err(MigrationError::MissingDependency {
                        migration: m.id.to_string(),
                        dependency: (*dep).to_string(),
                    });
                }
            }
        }

        // Check for circular dependencies
        let _ = self.sorted_migrations()?;

        Ok(())
    }
}

/// Errors that can occur during migration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationError {
    /// A migration has a circular dependency.
    CircularDependency,
    /// A migration depends on another that doesn't exist.
    MissingDependency {
        /// The migration with the missing dependency.
        migration: String,
        /// The dependency that's missing.
        dependency: String,
    },
    /// A migration is not reversible.
    NotReversible(String),
    /// Database error.
    DatabaseError(String),
}

impl std::fmt::Display for MigrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CircularDependency => write!(f, "Circular dependency detected in migrations"),
            Self::MissingDependency {
                migration,
                dependency,
            } => write!(
                f,
                "Migration '{}' depends on '{}' which doesn't exist",
                migration, dependency
            ),
            Self::NotReversible(id) => write!(f, "Migration '{}' is not reversible", id),
            Self::DatabaseError(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl std::error::Error for MigrationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migrations::column_builder::{bigint, boolean, varchar};
    use crate::migrations::dialect::SqliteDialect;
    use crate::migrations::table_builder::CreateTableBuilder;

    // Test migrations
    struct Migration0001;
    impl Migration for Migration0001 {
        const ID: &'static str = "0001_initial";
        fn up() -> Vec<Operation> {
            vec![
                CreateTableBuilder::new()
                    .name("users")
                    .column(bigint("id").primary_key().autoincrement().build())
                    .column(varchar("username", 255).not_null().build())
                    .build()
                    .into(),
            ]
        }
        fn down() -> Vec<Operation> {
            vec![Operation::drop_table("users")]
        }
    }

    struct Migration0002;
    impl Migration for Migration0002 {
        const ID: &'static str = "0002_add_email";
        const DEPENDENCIES: &'static [&'static str] = &["0001_initial"];
        fn up() -> Vec<Operation> {
            vec![Operation::add_column(
                "users",
                varchar("email", 255).build(),
            )]
        }
        fn down() -> Vec<Operation> {
            vec![Operation::drop_column("users", "email")]
        }
    }

    struct Migration0003;
    impl Migration for Migration0003 {
        const ID: &'static str = "0003_add_active";
        const DEPENDENCIES: &'static [&'static str] = &["0002_add_email"];
        fn up() -> Vec<Operation> {
            vec![Operation::add_column(
                "users",
                boolean("active").not_null().default_bool(true).build(),
            )]
        }
        fn down() -> Vec<Operation> {
            vec![Operation::drop_column("users", "active")]
        }
    }

    #[test]
    fn test_register_migrations() {
        let mut runner = MigrationRunner::new(SqliteDialect::new());
        runner.register::<Migration0001>();
        runner.register::<Migration0002>();

        assert_eq!(runner.migrations().len(), 2);
    }

    #[test]
    fn test_pending_migrations() {
        let mut runner = MigrationRunner::new(SqliteDialect::new());
        runner.register::<Migration0001>();
        runner.register::<Migration0002>();

        let state = MigrationState::new();
        let pending = runner.pending_migrations(&state);
        assert_eq!(pending.len(), 2);

        let mut state = MigrationState::new();
        state.mark_applied("0001_initial");
        let pending = runner.pending_migrations(&state);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, "0002_add_email");
    }

    #[test]
    fn test_topological_sort() {
        let mut runner = MigrationRunner::new(SqliteDialect::new());
        // Register in reverse order
        runner.register::<Migration0003>();
        runner.register::<Migration0001>();
        runner.register::<Migration0002>();

        let sorted = runner.sorted_migrations().unwrap();
        let ids: Vec<_> = sorted.iter().map(|m| m.id).collect();

        // 0001 must come before 0002, 0002 must come before 0003
        let pos_0001 = ids.iter().position(|&id| id == "0001_initial").unwrap();
        let pos_0002 = ids.iter().position(|&id| id == "0002_add_email").unwrap();
        let pos_0003 = ids.iter().position(|&id| id == "0003_add_active").unwrap();

        assert!(pos_0001 < pos_0002);
        assert!(pos_0002 < pos_0003);
    }

    #[test]
    fn test_sql_generation() {
        let mut runner = MigrationRunner::new(SqliteDialect::new());
        runner.register::<Migration0001>();

        let state = MigrationState::new();
        let sql = runner.sql_for_pending(&state).unwrap();

        assert_eq!(sql.len(), 1);
        assert_eq!(sql[0].0, "0001_initial");
        assert!(!sql[0].1.is_empty());
        assert!(sql[0].1[0].contains("CREATE TABLE"));
    }

    #[test]
    fn test_rollback_sql() {
        let mut runner = MigrationRunner::new(SqliteDialect::new());
        runner.register::<Migration0001>();
        runner.register::<Migration0002>();

        let mut state = MigrationState::new();
        state.mark_applied("0001_initial");
        state.mark_applied("0002_add_email");

        let sql = runner.sql_for_rollback(&state, 1).unwrap();
        assert_eq!(sql.len(), 1);
        assert_eq!(sql[0].0, "0002_add_email");
        assert!(sql[0].1[0].contains("DROP COLUMN"));
    }

    #[test]
    fn test_missing_dependency() {
        struct BadMigration;
        impl Migration for BadMigration {
            const ID: &'static str = "bad_migration";
            const DEPENDENCIES: &'static [&'static str] = &["nonexistent"];
            fn up() -> Vec<Operation> {
                vec![]
            }
            fn down() -> Vec<Operation> {
                vec![]
            }
        }

        let mut runner = MigrationRunner::new(SqliteDialect::new());
        runner.register::<BadMigration>();

        let result = runner.validate();
        assert!(matches!(
            result,
            Err(MigrationError::MissingDependency { .. })
        ));
    }

    #[test]
    fn test_status() {
        let mut runner = MigrationRunner::new(SqliteDialect::new());
        runner.register::<Migration0001>();
        runner.register::<Migration0002>();

        let mut state = MigrationState::new();
        state.mark_applied("0001_initial");

        let status = runner.status(&state);
        assert_eq!(status.len(), 2);

        let s1 = status.iter().find(|s| s.id == "0001_initial").unwrap();
        assert!(s1.applied);

        let s2 = status.iter().find(|s| s.id == "0002_add_email").unwrap();
        assert!(!s2.applied);
    }
}
