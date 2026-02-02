//! Type-safe table builders using the typestate pattern.
//!
//! This module provides builders for table operations that enforce correct
//! usage at compile time. For example, you cannot call `build()` on a
//! `CreateTableBuilder` without first setting a name and at least one column.

use std::marker::PhantomData;

use super::column_builder::ColumnDefinition;
use super::operation::{CreateTableOp, DropTableOp, TableConstraint};

// =============================================================================
// Typestate Markers
// =============================================================================

/// Marker: table has no name set.
#[derive(Debug, Clone, Copy)]
pub struct NoName;

/// Marker: table has a name set.
#[derive(Debug, Clone, Copy)]
pub struct HasName;

/// Marker: table has no columns.
#[derive(Debug, Clone, Copy)]
pub struct NoColumns;

/// Marker: table has at least one column.
#[derive(Debug, Clone, Copy)]
pub struct HasColumns;

// =============================================================================
// CreateTableBuilder
// =============================================================================

/// Type-safe CREATE TABLE builder.
///
/// Uses the typestate pattern to ensure that:
/// - A table name must be set before building
/// - At least one column must be added before building
///
/// # Example
///
/// ```rust
/// use oxide_sql_core::migrations::{CreateTableBuilder, bigint, varchar, timestamp};
///
/// let op = CreateTableBuilder::new()
///     .name("users")
///     .column(bigint("id").primary_key().autoincrement().build())
///     .column(varchar("username", 255).not_null().unique().build())
///     .column(timestamp("created_at").not_null().default_expr("CURRENT_TIMESTAMP").build())
///     .build();
///
/// assert_eq!(op.name, "users");
/// assert_eq!(op.columns.len(), 3);
/// ```
#[derive(Debug, Clone)]
pub struct CreateTableBuilder<Name, Cols> {
    name: Option<String>,
    columns: Vec<ColumnDefinition>,
    constraints: Vec<TableConstraint>,
    if_not_exists: bool,
    _state: PhantomData<(Name, Cols)>,
}

impl Default for CreateTableBuilder<NoName, NoColumns> {
    fn default() -> Self {
        Self::new()
    }
}

impl CreateTableBuilder<NoName, NoColumns> {
    /// Creates a new `CreateTableBuilder`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: None,
            columns: Vec::new(),
            constraints: Vec::new(),
            if_not_exists: false,
            _state: PhantomData,
        }
    }
}

impl<Cols> CreateTableBuilder<NoName, Cols> {
    /// Sets the table name.
    #[must_use]
    pub fn name(self, name: impl Into<String>) -> CreateTableBuilder<HasName, Cols> {
        CreateTableBuilder {
            name: Some(name.into()),
            columns: self.columns,
            constraints: self.constraints,
            if_not_exists: self.if_not_exists,
            _state: PhantomData,
        }
    }
}

impl<Name> CreateTableBuilder<Name, NoColumns> {
    /// Adds the first column to the table.
    #[must_use]
    pub fn column(self, column: ColumnDefinition) -> CreateTableBuilder<Name, HasColumns> {
        CreateTableBuilder {
            name: self.name,
            columns: vec![column],
            constraints: self.constraints,
            if_not_exists: self.if_not_exists,
            _state: PhantomData,
        }
    }
}

impl<Name> CreateTableBuilder<Name, HasColumns> {
    /// Adds another column to the table.
    #[must_use]
    pub fn column(mut self, column: ColumnDefinition) -> Self {
        self.columns.push(column);
        self
    }
}

impl<Name, Cols> CreateTableBuilder<Name, Cols> {
    /// Uses IF NOT EXISTS clause.
    #[must_use]
    pub fn if_not_exists(mut self) -> Self {
        self.if_not_exists = true;
        self
    }
}

impl<Cols> CreateTableBuilder<HasName, Cols> {
    /// Adds a table-level constraint.
    #[must_use]
    pub fn constraint(mut self, constraint: TableConstraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    /// Adds a composite primary key constraint.
    #[must_use]
    pub fn primary_key(mut self, columns: &[&str]) -> Self {
        self.constraints.push(TableConstraint::PrimaryKey {
            name: None,
            columns: columns.iter().map(|&s| s.to_string()).collect(),
        });
        self
    }

    /// Adds a named composite primary key constraint.
    #[must_use]
    pub fn primary_key_named(mut self, name: impl Into<String>, columns: &[&str]) -> Self {
        self.constraints.push(TableConstraint::PrimaryKey {
            name: Some(name.into()),
            columns: columns.iter().map(|&s| s.to_string()).collect(),
        });
        self
    }

    /// Adds a unique constraint on multiple columns.
    #[must_use]
    pub fn unique_constraint(mut self, columns: &[&str]) -> Self {
        self.constraints.push(TableConstraint::Unique {
            name: None,
            columns: columns.iter().map(|&s| s.to_string()).collect(),
        });
        self
    }

    /// Adds a named unique constraint on multiple columns.
    #[must_use]
    pub fn unique_constraint_named(mut self, name: impl Into<String>, columns: &[&str]) -> Self {
        self.constraints.push(TableConstraint::Unique {
            name: Some(name.into()),
            columns: columns.iter().map(|&s| s.to_string()).collect(),
        });
        self
    }

    /// Adds a check constraint.
    #[must_use]
    pub fn check_constraint(mut self, expression: impl Into<String>) -> Self {
        self.constraints.push(TableConstraint::Check {
            name: None,
            expression: expression.into(),
        });
        self
    }

    /// Adds a named check constraint.
    #[must_use]
    pub fn check_constraint_named(
        mut self,
        name: impl Into<String>,
        expression: impl Into<String>,
    ) -> Self {
        self.constraints.push(TableConstraint::Check {
            name: Some(name.into()),
            expression: expression.into(),
        });
        self
    }
}

impl CreateTableBuilder<HasName, HasColumns> {
    /// Builds the `CreateTableOp`.
    #[must_use]
    pub fn build(self) -> CreateTableOp {
        CreateTableOp {
            name: self.name.expect("Name was set"),
            columns: self.columns,
            constraints: self.constraints,
            if_not_exists: self.if_not_exists,
        }
    }
}

// =============================================================================
// DropTableBuilder
// =============================================================================

/// Builder for DROP TABLE operations.
#[derive(Debug, Clone, Default)]
pub struct DropTableBuilder {
    name: Option<String>,
    if_exists: bool,
    cascade: bool,
}

impl DropTableBuilder {
    /// Creates a new `DropTableBuilder`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the table name.
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Uses IF EXISTS clause.
    #[must_use]
    pub fn if_exists(mut self) -> Self {
        self.if_exists = true;
        self
    }

    /// Uses CASCADE (PostgreSQL).
    #[must_use]
    pub fn cascade(mut self) -> Self {
        self.cascade = true;
        self
    }

    /// Builds the `DropTableOp`.
    ///
    /// # Panics
    ///
    /// Panics if no table name was set.
    #[must_use]
    pub fn build(self) -> DropTableOp {
        DropTableOp {
            name: self.name.expect("Table name must be set"),
            if_exists: self.if_exists,
            cascade: self.cascade,
        }
    }
}

// =============================================================================
// IndexBuilder
// =============================================================================

use super::operation::{CreateIndexOp, IndexType};

/// Builder for CREATE INDEX operations.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct CreateIndexBuilder {
    name: Option<String>,
    table: Option<String>,
    columns: Vec<String>,
    unique: bool,
    index_type: IndexType,
    if_not_exists: bool,
    condition: Option<String>,
}

#[allow(dead_code)]
impl CreateIndexBuilder {
    /// Creates a new `CreateIndexBuilder`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the index name.
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the table name.
    #[must_use]
    pub fn on_table(mut self, table: impl Into<String>) -> Self {
        self.table = Some(table.into());
        self
    }

    /// Adds a column to the index.
    #[must_use]
    pub fn column(mut self, column: impl Into<String>) -> Self {
        self.columns.push(column.into());
        self
    }

    /// Adds multiple columns to the index.
    #[must_use]
    pub fn columns(mut self, columns: &[&str]) -> Self {
        self.columns.extend(columns.iter().map(|&s| s.to_string()));
        self
    }

    /// Makes this a unique index.
    #[must_use]
    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    /// Sets the index type.
    #[must_use]
    pub fn index_type(mut self, index_type: IndexType) -> Self {
        self.index_type = index_type;
        self
    }

    /// Uses IF NOT EXISTS clause.
    #[must_use]
    pub fn if_not_exists(mut self) -> Self {
        self.if_not_exists = true;
        self
    }

    /// Adds a partial index condition (WHERE clause).
    #[must_use]
    pub fn where_clause(mut self, condition: impl Into<String>) -> Self {
        self.condition = Some(condition.into());
        self
    }

    /// Builds the `CreateIndexOp`.
    ///
    /// # Panics
    ///
    /// Panics if name, table, or columns are not set.
    #[must_use]
    pub fn build(self) -> CreateIndexOp {
        CreateIndexOp {
            name: self.name.expect("Index name must be set"),
            table: self.table.expect("Table name must be set"),
            columns: self.columns,
            unique: self.unique,
            index_type: self.index_type,
            if_not_exists: self.if_not_exists,
            condition: self.condition,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migrations::column_builder::{bigint, boolean, timestamp, varchar};

    #[test]
    fn test_create_table_builder() {
        let op = CreateTableBuilder::new()
            .name("users")
            .column(bigint("id").primary_key().autoincrement().build())
            .column(varchar("username", 255).not_null().unique().build())
            .column(varchar("email", 255).build())
            .column(
                timestamp("created_at")
                    .not_null()
                    .default_expr("CURRENT_TIMESTAMP")
                    .build(),
            )
            .build();

        assert_eq!(op.name, "users");
        assert_eq!(op.columns.len(), 4);
        assert!(!op.if_not_exists);

        // Check first column
        let id_col = &op.columns[0];
        assert_eq!(id_col.name, "id");
        assert!(id_col.primary_key);
        assert!(id_col.autoincrement);
    }

    #[test]
    fn test_create_table_if_not_exists() {
        let op = CreateTableBuilder::new()
            .if_not_exists()
            .name("users")
            .column(bigint("id").primary_key().build())
            .build();

        assert!(op.if_not_exists);
    }

    #[test]
    fn test_create_table_with_constraints() {
        let op = CreateTableBuilder::new()
            .name("order_items")
            .column(bigint("order_id").not_null().build())
            .column(bigint("product_id").not_null().build())
            .column(bigint("quantity").not_null().build())
            .primary_key(&["order_id", "product_id"])
            .unique_constraint(&["order_id", "product_id"])
            .check_constraint("quantity > 0")
            .build();

        assert_eq!(op.constraints.len(), 3);

        match &op.constraints[0] {
            TableConstraint::PrimaryKey { columns, .. } => {
                assert_eq!(columns, &["order_id", "product_id"]);
            }
            _ => panic!("Expected PrimaryKey constraint"),
        }

        match &op.constraints[1] {
            TableConstraint::Unique { columns, .. } => {
                assert_eq!(columns, &["order_id", "product_id"]);
            }
            _ => panic!("Expected Unique constraint"),
        }

        match &op.constraints[2] {
            TableConstraint::Check { expression, .. } => {
                assert_eq!(expression, "quantity > 0");
            }
            _ => panic!("Expected Check constraint"),
        }
    }

    #[test]
    fn test_drop_table_builder() {
        let op = DropTableBuilder::new().name("users").build();
        assert_eq!(op.name, "users");
        assert!(!op.if_exists);
        assert!(!op.cascade);

        let op = DropTableBuilder::new()
            .name("users")
            .if_exists()
            .cascade()
            .build();
        assert!(op.if_exists);
        assert!(op.cascade);
    }

    #[test]
    fn test_create_index_builder() {
        let op = CreateIndexBuilder::new()
            .name("idx_users_email")
            .on_table("users")
            .column("email")
            .unique()
            .build();

        assert_eq!(op.name, "idx_users_email");
        assert_eq!(op.table, "users");
        assert_eq!(op.columns, vec!["email"]);
        assert!(op.unique);
    }

    #[test]
    fn test_create_composite_index() {
        let op = CreateIndexBuilder::new()
            .name("idx_invoices_company_status")
            .on_table("invoices")
            .columns(&["company_id", "status"])
            .if_not_exists()
            .build();

        assert_eq!(op.columns, vec!["company_id", "status"]);
        assert!(op.if_not_exists);
    }

    #[test]
    fn test_partial_index() {
        let op = CreateIndexBuilder::new()
            .name("idx_active_users")
            .on_table("users")
            .column("email")
            .where_clause("active = true")
            .build();

        assert_eq!(op.condition, Some("active = true".to_string()));
    }

    #[test]
    fn test_fluent_api_order() {
        // Verify we can chain methods in different orders
        let op1 = CreateTableBuilder::new()
            .name("test")
            .column(boolean("flag").build())
            .if_not_exists()
            .build();

        let op2 = CreateTableBuilder::new()
            .if_not_exists()
            .name("test")
            .column(boolean("flag").build())
            .build();

        assert_eq!(op1.name, op2.name);
        assert_eq!(op1.if_not_exists, op2.if_not_exists);
    }
}
