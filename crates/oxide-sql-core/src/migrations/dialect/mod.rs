//! Dialect-specific SQL generation for migrations.
//!
//! Different databases have different SQL syntax for DDL operations.
//! This module provides dialect implementations for generating
//! database-specific migration SQL.

mod postgres;
mod sqlite;

pub use postgres::PostgresDialect;
pub use sqlite::SqliteDialect;

use crate::ast::DataType;

use super::column_builder::{ColumnDefinition, DefaultValue};
use super::operation::{
    AddColumnOp, AlterColumnOp, CreateIndexOp, CreateTableOp, DropColumnOp, DropIndexOp,
    DropTableOp, IndexType, Operation, RenameColumnOp, RenameTableOp, TableConstraint,
};

/// Trait for dialect-specific SQL generation for migrations.
pub trait MigrationDialect {
    /// Returns the dialect name.
    fn name(&self) -> &'static str;

    /// Generates SQL for an operation.
    fn generate_sql(&self, operation: &Operation) -> String {
        match operation {
            Operation::CreateTable(op) => self.create_table(op),
            Operation::DropTable(op) => self.drop_table(op),
            Operation::RenameTable(op) => self.rename_table(op),
            Operation::AddColumn(op) => self.add_column(op),
            Operation::DropColumn(op) => self.drop_column(op),
            Operation::AlterColumn(op) => self.alter_column(op),
            Operation::RenameColumn(op) => self.rename_column(op),
            Operation::CreateIndex(op) => self.create_index(op),
            Operation::DropIndex(op) => self.drop_index(op),
            Operation::AddForeignKey(op) => self.add_foreign_key(op),
            Operation::DropForeignKey(op) => self.drop_foreign_key(op),
            Operation::RunSql(op) => op.up_sql.clone(),
        }
    }

    /// Generates SQL for CREATE TABLE.
    fn create_table(&self, op: &CreateTableOp) -> String {
        let mut sql = String::from("CREATE TABLE ");
        if op.if_not_exists {
            sql.push_str("IF NOT EXISTS ");
        }
        sql.push_str(&self.quote_identifier(&op.name));
        sql.push_str(" (\n");

        // Columns
        let column_defs: Vec<String> = op
            .columns
            .iter()
            .map(|c| format!("    {}", self.column_definition(c)))
            .collect();
        sql.push_str(&column_defs.join(",\n"));

        // Constraints
        if !op.constraints.is_empty() {
            sql.push_str(",\n");
            let constraint_defs: Vec<String> = op
                .constraints
                .iter()
                .map(|c| format!("    {}", self.table_constraint(c)))
                .collect();
            sql.push_str(&constraint_defs.join(",\n"));
        }

        sql.push_str("\n)");
        sql
    }

    /// Generates SQL for DROP TABLE.
    fn drop_table(&self, op: &DropTableOp) -> String {
        let mut sql = String::from("DROP TABLE ");
        if op.if_exists {
            sql.push_str("IF EXISTS ");
        }
        sql.push_str(&self.quote_identifier(&op.name));
        if op.cascade {
            sql.push_str(" CASCADE");
        }
        sql
    }

    /// Generates SQL for RENAME TABLE.
    fn rename_table(&self, op: &RenameTableOp) -> String;

    /// Generates SQL for ADD COLUMN.
    fn add_column(&self, op: &AddColumnOp) -> String {
        format!(
            "ALTER TABLE {} ADD COLUMN {}",
            self.quote_identifier(&op.table),
            self.column_definition(&op.column)
        )
    }

    /// Generates SQL for DROP COLUMN.
    fn drop_column(&self, op: &DropColumnOp) -> String {
        format!(
            "ALTER TABLE {} DROP COLUMN {}",
            self.quote_identifier(&op.table),
            self.quote_identifier(&op.column)
        )
    }

    /// Generates SQL for ALTER COLUMN.
    fn alter_column(&self, op: &AlterColumnOp) -> String;

    /// Generates SQL for RENAME COLUMN.
    fn rename_column(&self, op: &RenameColumnOp) -> String;

    /// Generates SQL for CREATE INDEX.
    fn create_index(&self, op: &CreateIndexOp) -> String {
        let mut sql = String::from("CREATE ");
        if op.unique {
            sql.push_str("UNIQUE ");
        }
        sql.push_str("INDEX ");
        if op.if_not_exists {
            sql.push_str("IF NOT EXISTS ");
        }
        sql.push_str(&self.quote_identifier(&op.name));
        sql.push_str(" ON ");
        sql.push_str(&self.quote_identifier(&op.table));

        // Index type (if supported and not default)
        if op.index_type != IndexType::BTree {
            sql.push_str(&format!(" USING {}", self.index_type_sql(&op.index_type)));
        }

        // Columns
        sql.push_str(" (");
        let cols: Vec<String> = op
            .columns
            .iter()
            .map(|c| self.quote_identifier(c))
            .collect();
        sql.push_str(&cols.join(", "));
        sql.push(')');

        // Partial index condition
        if let Some(ref condition) = op.condition {
            sql.push_str(" WHERE ");
            sql.push_str(condition);
        }

        sql
    }

    /// Generates SQL for DROP INDEX.
    fn drop_index(&self, op: &DropIndexOp) -> String;

    /// Generates SQL for ADD FOREIGN KEY.
    fn add_foreign_key(&self, op: &super::operation::AddForeignKeyOp) -> String {
        let mut sql = format!("ALTER TABLE {} ADD ", self.quote_identifier(&op.table));
        if let Some(ref name) = op.name {
            sql.push_str(&format!("CONSTRAINT {} ", self.quote_identifier(name)));
        }
        sql.push_str("FOREIGN KEY (");
        let cols: Vec<String> = op
            .columns
            .iter()
            .map(|c| self.quote_identifier(c))
            .collect();
        sql.push_str(&cols.join(", "));
        sql.push_str(") REFERENCES ");
        sql.push_str(&self.quote_identifier(&op.references_table));
        sql.push_str(" (");
        let ref_cols: Vec<String> = op
            .references_columns
            .iter()
            .map(|c| self.quote_identifier(c))
            .collect();
        sql.push_str(&ref_cols.join(", "));
        sql.push(')');

        if let Some(action) = op.on_delete {
            sql.push_str(" ON DELETE ");
            sql.push_str(action.as_sql());
        }
        if let Some(action) = op.on_update {
            sql.push_str(" ON UPDATE ");
            sql.push_str(action.as_sql());
        }

        sql
    }

    /// Generates SQL for DROP FOREIGN KEY.
    fn drop_foreign_key(&self, op: &super::operation::DropForeignKeyOp) -> String;

    /// Generates SQL for a column definition.
    fn column_definition(&self, col: &ColumnDefinition) -> String {
        let mut sql = format!(
            "{} {}",
            self.quote_identifier(&col.name),
            self.map_data_type(&col.data_type)
        );

        if col.primary_key {
            sql.push_str(" PRIMARY KEY");
            if col.autoincrement {
                sql.push_str(&self.autoincrement_keyword());
            }
        } else {
            if !col.nullable {
                sql.push_str(" NOT NULL");
            }
            if col.unique {
                sql.push_str(" UNIQUE");
            }
            if col.autoincrement {
                sql.push_str(&self.autoincrement_keyword());
            }
        }

        if let Some(ref default) = col.default {
            sql.push_str(" DEFAULT ");
            sql.push_str(&self.render_default(default));
        }

        if let Some(ref fk) = col.references {
            sql.push_str(" REFERENCES ");
            sql.push_str(&self.quote_identifier(&fk.table));
            sql.push_str(" (");
            sql.push_str(&self.quote_identifier(&fk.column));
            sql.push(')');
            if let Some(action) = fk.on_delete {
                sql.push_str(" ON DELETE ");
                sql.push_str(action.as_sql());
            }
            if let Some(action) = fk.on_update {
                sql.push_str(" ON UPDATE ");
                sql.push_str(action.as_sql());
            }
        }

        if let Some(ref check) = col.check {
            sql.push_str(&format!(" CHECK ({})", check));
        }

        if let Some(ref collation) = col.collation {
            sql.push_str(&format!(" COLLATE {}", collation));
        }

        sql
    }

    /// Generates SQL for a table constraint.
    fn table_constraint(&self, constraint: &TableConstraint) -> String {
        match constraint {
            TableConstraint::PrimaryKey { name, columns } => {
                let mut sql = String::new();
                if let Some(n) = name {
                    sql.push_str(&format!("CONSTRAINT {} ", self.quote_identifier(n)));
                }
                sql.push_str("PRIMARY KEY (");
                let cols: Vec<String> = columns.iter().map(|c| self.quote_identifier(c)).collect();
                sql.push_str(&cols.join(", "));
                sql.push(')');
                sql
            }
            TableConstraint::Unique { name, columns } => {
                let mut sql = String::new();
                if let Some(n) = name {
                    sql.push_str(&format!("CONSTRAINT {} ", self.quote_identifier(n)));
                }
                sql.push_str("UNIQUE (");
                let cols: Vec<String> = columns.iter().map(|c| self.quote_identifier(c)).collect();
                sql.push_str(&cols.join(", "));
                sql.push(')');
                sql
            }
            TableConstraint::ForeignKey {
                name,
                columns,
                references_table,
                references_columns,
                on_delete,
                on_update,
            } => {
                let mut sql = String::new();
                if let Some(n) = name {
                    sql.push_str(&format!("CONSTRAINT {} ", self.quote_identifier(n)));
                }
                sql.push_str("FOREIGN KEY (");
                let cols: Vec<String> = columns.iter().map(|c| self.quote_identifier(c)).collect();
                sql.push_str(&cols.join(", "));
                sql.push_str(") REFERENCES ");
                sql.push_str(&self.quote_identifier(references_table));
                sql.push_str(" (");
                let ref_cols: Vec<String> = references_columns
                    .iter()
                    .map(|c| self.quote_identifier(c))
                    .collect();
                sql.push_str(&ref_cols.join(", "));
                sql.push(')');
                if let Some(action) = on_delete {
                    sql.push_str(" ON DELETE ");
                    sql.push_str(action.as_sql());
                }
                if let Some(action) = on_update {
                    sql.push_str(" ON UPDATE ");
                    sql.push_str(action.as_sql());
                }
                sql
            }
            TableConstraint::Check { name, expression } => {
                let mut sql = String::new();
                if let Some(n) = name {
                    sql.push_str(&format!("CONSTRAINT {} ", self.quote_identifier(n)));
                }
                sql.push_str(&format!("CHECK ({})", expression));
                sql
            }
        }
    }

    /// Maps a `DataType` to the dialect-specific SQL type.
    fn map_data_type(&self, dt: &DataType) -> String;

    /// Renders a default value.
    fn render_default(&self, default: &DefaultValue) -> String {
        default.to_sql()
    }

    /// Returns the identifier quote character.
    fn quote_char(&self) -> char {
        '"'
    }

    /// Quotes an identifier.
    fn quote_identifier(&self, name: &str) -> String {
        let q = self.quote_char();
        format!("{q}{name}{q}")
    }

    /// Returns the AUTOINCREMENT keyword for this dialect.
    fn autoincrement_keyword(&self) -> String;

    /// Maps an index type to SQL.
    fn index_type_sql(&self, index_type: &IndexType) -> &'static str {
        match index_type {
            IndexType::BTree => "BTREE",
            IndexType::Hash => "HASH",
            IndexType::Gist => "GIST",
            IndexType::Gin => "GIN",
        }
    }
}
