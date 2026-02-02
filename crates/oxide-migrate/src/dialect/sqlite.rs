//! SQLite dialect for migrations.
//!
//! SQLite has limited ALTER TABLE support, so some operations require
//! the "table recreation" strategy: create a new table, copy data,
//! drop the old table, rename the new table.

use crate::operations::MigrationOperation;
use crate::schema::{ColumnSchema, ForeignKeyAction, SqlType};

use super::MigrationDialect;

/// SQLite migration dialect.
#[derive(Debug, Clone, Default)]
pub struct SqliteDialect;

impl SqliteDialect {
    /// Creates a new SQLite dialect.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Generates SQL for creating a table.
    fn create_table_sql(
        &self,
        name: &str,
        columns: &[ColumnSchema],
        primary_key: &[String],
        if_not_exists: bool,
    ) -> String {
        let mut sql = String::from("CREATE TABLE ");
        if if_not_exists {
            sql.push_str("IF NOT EXISTS ");
        }
        sql.push_str(&self.quote_identifier(name));
        sql.push_str(" (\n");

        // Column definitions
        let col_defs: Vec<String> = columns.iter().map(|c| self.column_definition(c)).collect();
        sql.push_str("  ");
        sql.push_str(&col_defs.join(",\n  "));

        // Composite primary key (if more than one column or if not defined inline)
        let pk_columns: Vec<&String> = primary_key
            .iter()
            .filter(|pk| {
                // Only include if not already defined inline
                columns
                    .iter()
                    .find(|c| &c.name == *pk)
                    .map(|c| !c.primary_key)
                    .unwrap_or(true)
            })
            .collect();

        if !pk_columns.is_empty() || primary_key.len() > 1 {
            sql.push_str(",\n  PRIMARY KEY (");
            let quoted: Vec<String> = primary_key
                .iter()
                .map(|c| self.quote_identifier(c))
                .collect();
            sql.push_str(&quoted.join(", "));
            sql.push(')');
        }

        sql.push_str("\n)");
        sql
    }

    /// Generates SQL for dropping a table.
    fn drop_table_sql(&self, name: &str, if_exists: bool) -> String {
        let mut sql = String::from("DROP TABLE ");
        if if_exists {
            sql.push_str("IF EXISTS ");
        }
        sql.push_str(&self.quote_identifier(name));
        sql
    }

    /// Generates SQL for renaming a table.
    fn rename_table_sql(&self, old_name: &str, new_name: &str) -> String {
        format!(
            "ALTER TABLE {} RENAME TO {}",
            self.quote_identifier(old_name),
            self.quote_identifier(new_name)
        )
    }

    /// Generates SQL for adding a column.
    fn add_column_sql(&self, table: &str, column: &ColumnSchema) -> String {
        format!(
            "ALTER TABLE {} ADD COLUMN {}",
            self.quote_identifier(table),
            self.column_definition(column)
        )
    }

    /// Generates SQL for dropping a column (SQLite 3.35.0+).
    fn drop_column_sql(&self, table: &str, column_name: &str) -> String {
        format!(
            "ALTER TABLE {} DROP COLUMN {}",
            self.quote_identifier(table),
            self.quote_identifier(column_name)
        )
    }

    /// Generates SQL for renaming a column (SQLite 3.25.0+).
    fn rename_column_sql(&self, table: &str, old_name: &str, new_name: &str) -> String {
        format!(
            "ALTER TABLE {} RENAME COLUMN {} TO {}",
            self.quote_identifier(table),
            self.quote_identifier(old_name),
            self.quote_identifier(new_name)
        )
    }

    /// Generates SQL for creating an index.
    fn create_index_sql(
        &self,
        name: &str,
        table: &str,
        columns: &[String],
        unique: bool,
        condition: Option<&str>,
        if_not_exists: bool,
    ) -> String {
        let mut sql = String::from("CREATE ");
        if unique {
            sql.push_str("UNIQUE ");
        }
        sql.push_str("INDEX ");
        if if_not_exists {
            sql.push_str("IF NOT EXISTS ");
        }
        sql.push_str(&self.quote_identifier(name));
        sql.push_str(" ON ");
        sql.push_str(&self.quote_identifier(table));
        sql.push_str(" (");

        let quoted: Vec<String> = columns.iter().map(|c| self.quote_identifier(c)).collect();
        sql.push_str(&quoted.join(", "));
        sql.push(')');

        if let Some(cond) = condition {
            sql.push_str(" WHERE ");
            sql.push_str(cond);
        }

        sql
    }

    /// Generates SQL for dropping an index.
    fn drop_index_sql(&self, name: &str, if_exists: bool) -> String {
        let mut sql = String::from("DROP INDEX ");
        if if_exists {
            sql.push_str("IF EXISTS ");
        }
        sql.push_str(&self.quote_identifier(name));
        sql
    }

    /// Generates foreign key action SQL.
    #[allow(dead_code)]
    fn foreign_key_action_sql(&self, action: &ForeignKeyAction) -> &'static str {
        match action {
            ForeignKeyAction::NoAction => "NO ACTION",
            ForeignKeyAction::Restrict => "RESTRICT",
            ForeignKeyAction::Cascade => "CASCADE",
            ForeignKeyAction::SetNull => "SET NULL",
            ForeignKeyAction::SetDefault => "SET DEFAULT",
        }
    }
}

impl MigrationDialect for SqliteDialect {
    fn name(&self) -> &'static str {
        "sqlite"
    }

    fn generate_sql(&self, operation: &MigrationOperation) -> Vec<String> {
        match operation {
            MigrationOperation::CreateTable {
                name,
                columns,
                primary_key,
                if_not_exists,
            } => vec![self.create_table_sql(name, columns, primary_key, *if_not_exists)],

            MigrationOperation::DropTable { name, if_exists } => {
                vec![self.drop_table_sql(name, *if_exists)]
            }

            MigrationOperation::RenameTable { old_name, new_name } => {
                vec![self.rename_table_sql(old_name, new_name)]
            }

            MigrationOperation::AddColumn { table, column } => {
                vec![self.add_column_sql(table, column)]
            }

            MigrationOperation::DropColumn { table, column_name } => {
                vec![self.drop_column_sql(table, column_name)]
            }

            MigrationOperation::RenameColumn {
                table,
                old_name,
                new_name,
            } => vec![self.rename_column_sql(table, old_name, new_name)],

            MigrationOperation::AlterColumn {
                table, column_name, ..
            } => {
                // SQLite doesn't support ALTER COLUMN directly.
                // For now, generate a comment explaining the limitation.
                // A full implementation would use the table recreation strategy.
                vec![format!(
                    "-- ALTER COLUMN not directly supported in SQLite. \
                     Table recreation required for: {}.{}",
                    table, column_name
                )]
            }

            MigrationOperation::CreateIndex {
                name,
                table,
                columns,
                unique,
                condition,
                if_not_exists,
            } => vec![self.create_index_sql(
                name,
                table,
                columns,
                *unique,
                condition.as_deref(),
                *if_not_exists,
            )],

            MigrationOperation::DropIndex {
                name, if_exists, ..
            } => vec![self.drop_index_sql(name, *if_exists)],

            MigrationOperation::AddForeignKey { foreign_key, .. } => {
                // SQLite doesn't support ALTER TABLE ADD CONSTRAINT for foreign keys.
                // Foreign keys must be defined at table creation time.
                vec![format!(
                    "-- Foreign key {} cannot be added after table creation in SQLite. \
                     Table recreation required.",
                    foreign_key.name
                )]
            }

            MigrationOperation::DropForeignKey {
                constraint_name, ..
            } => {
                vec![format!(
                    "-- Foreign key {} cannot be dropped in SQLite. \
                     Table recreation required.",
                    constraint_name
                )]
            }

            MigrationOperation::AddUniqueConstraint {
                table,
                name,
                columns,
            } => {
                // In SQLite, unique constraints after creation are done via unique indexes
                vec![self.create_index_sql(name, table, columns, true, None, false)]
            }

            MigrationOperation::DropUniqueConstraint { name, .. } => {
                vec![self.drop_index_sql(name, false)]
            }

            MigrationOperation::RunSql { forward, .. } => {
                vec![forward.clone()]
            }
        }
    }

    fn type_name(&self, sql_type: &SqlType) -> String {
        match sql_type {
            SqlType::Integer | SqlType::SmallInt => "INTEGER".to_string(),
            SqlType::BigInt => "INTEGER".to_string(),
            SqlType::Text => "TEXT".to_string(),
            SqlType::Varchar(_) => "TEXT".to_string(),
            SqlType::Char(_) => "TEXT".to_string(),
            SqlType::Boolean => "INTEGER".to_string(),
            SqlType::DateTime | SqlType::Timestamp => "TEXT".to_string(),
            SqlType::Date => "TEXT".to_string(),
            SqlType::Time => "TEXT".to_string(),
            SqlType::Real => "REAL".to_string(),
            SqlType::Double => "REAL".to_string(),
            SqlType::Decimal(_, _) | SqlType::Numeric(_, _) => "NUMERIC".to_string(),
            SqlType::Blob | SqlType::Binary(_) | SqlType::VarBinary(_) => "BLOB".to_string(),
            SqlType::Json => "TEXT".to_string(),
            SqlType::Uuid => "TEXT".to_string(),
        }
    }

    fn supports_alter_column(&self) -> bool {
        false
    }

    fn supports_drop_column(&self) -> bool {
        // SQLite 3.35.0+ supports DROP COLUMN
        true
    }

    fn supports_add_constraint(&self) -> bool {
        false
    }

    fn auto_increment_keyword(&self) -> &'static str {
        "AUTOINCREMENT"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::DefaultValue;

    fn dialect() -> SqliteDialect {
        SqliteDialect::new()
    }

    #[test]
    fn test_create_table_simple() {
        let op = MigrationOperation::CreateTable {
            name: "users".to_string(),
            columns: vec![
                ColumnSchema::new("id", SqlType::BigInt)
                    .primary_key()
                    .auto_increment(),
                ColumnSchema::new("name", SqlType::Varchar(255)).not_null(),
            ],
            primary_key: vec!["id".to_string()],
            if_not_exists: false,
        };

        let sql = dialect().generate_sql(&op);
        assert_eq!(sql.len(), 1);
        assert!(sql[0].contains("CREATE TABLE"));
        assert!(sql[0].contains("\"users\""));
        assert!(sql[0].contains("\"id\""));
        assert!(sql[0].contains("PRIMARY KEY"));
        assert!(sql[0].contains("AUTOINCREMENT"));
        assert!(sql[0].contains("NOT NULL"));
    }

    #[test]
    fn test_create_table_if_not_exists() {
        let op = MigrationOperation::CreateTable {
            name: "users".to_string(),
            columns: vec![ColumnSchema::new("id", SqlType::BigInt).primary_key()],
            primary_key: vec!["id".to_string()],
            if_not_exists: true,
        };

        let sql = dialect().generate_sql(&op);
        assert!(sql[0].contains("IF NOT EXISTS"));
    }

    #[test]
    fn test_drop_table() {
        let op = MigrationOperation::DropTable {
            name: "users".to_string(),
            if_exists: true,
        };

        let sql = dialect().generate_sql(&op);
        assert_eq!(sql[0], "DROP TABLE IF EXISTS \"users\"");
    }

    #[test]
    fn test_rename_table() {
        let op = MigrationOperation::RenameTable {
            old_name: "users".to_string(),
            new_name: "accounts".to_string(),
        };

        let sql = dialect().generate_sql(&op);
        assert_eq!(sql[0], "ALTER TABLE \"users\" RENAME TO \"accounts\"");
    }

    #[test]
    fn test_add_column() {
        let op = MigrationOperation::AddColumn {
            table: "users".to_string(),
            column: ColumnSchema::new("email", SqlType::Varchar(255))
                .not_null()
                .unique(),
        };

        let sql = dialect().generate_sql(&op);
        assert!(sql[0].contains("ALTER TABLE \"users\" ADD COLUMN"));
        assert!(sql[0].contains("\"email\""));
        assert!(sql[0].contains("NOT NULL"));
        assert!(sql[0].contains("UNIQUE"));
    }

    #[test]
    fn test_add_column_with_default() {
        let op = MigrationOperation::AddColumn {
            table: "users".to_string(),
            column: ColumnSchema::new("is_active", SqlType::Boolean)
                .not_null()
                .default(DefaultValue::Bool(true)),
        };

        let sql = dialect().generate_sql(&op);
        assert!(sql[0].contains("DEFAULT 1"));
    }

    #[test]
    fn test_drop_column() {
        let op = MigrationOperation::DropColumn {
            table: "users".to_string(),
            column_name: "email".to_string(),
        };

        let sql = dialect().generate_sql(&op);
        assert_eq!(sql[0], "ALTER TABLE \"users\" DROP COLUMN \"email\"");
    }

    #[test]
    fn test_rename_column() {
        let op = MigrationOperation::RenameColumn {
            table: "users".to_string(),
            old_name: "name".to_string(),
            new_name: "full_name".to_string(),
        };

        let sql = dialect().generate_sql(&op);
        assert_eq!(
            sql[0],
            "ALTER TABLE \"users\" RENAME COLUMN \"name\" TO \"full_name\""
        );
    }

    #[test]
    fn test_create_index() {
        let op = MigrationOperation::CreateIndex {
            name: "idx_users_email".to_string(),
            table: "users".to_string(),
            columns: vec!["email".to_string()],
            unique: true,
            condition: None,
            if_not_exists: false,
        };

        let sql = dialect().generate_sql(&op);
        assert_eq!(
            sql[0],
            "CREATE UNIQUE INDEX \"idx_users_email\" ON \"users\" (\"email\")"
        );
    }

    #[test]
    fn test_create_partial_index() {
        let op = MigrationOperation::CreateIndex {
            name: "idx_active_users".to_string(),
            table: "users".to_string(),
            columns: vec!["email".to_string()],
            unique: false,
            condition: Some("is_active = 1".to_string()),
            if_not_exists: true,
        };

        let sql = dialect().generate_sql(&op);
        assert!(sql[0].contains("IF NOT EXISTS"));
        assert!(sql[0].contains("WHERE is_active = 1"));
    }

    #[test]
    fn test_drop_index() {
        let op = MigrationOperation::DropIndex {
            name: "idx_users_email".to_string(),
            table: None,
            if_exists: true,
        };

        let sql = dialect().generate_sql(&op);
        assert_eq!(sql[0], "DROP INDEX IF EXISTS \"idx_users_email\"");
    }

    #[test]
    fn test_run_sql() {
        let op = MigrationOperation::RunSql {
            forward: "INSERT INTO config VALUES ('key', 'value')".to_string(),
            backward: Some("DELETE FROM config WHERE key = 'key'".to_string()),
        };

        let sql = dialect().generate_sql(&op);
        assert_eq!(sql[0], "INSERT INTO config VALUES ('key', 'value')");
    }

    #[test]
    fn test_type_names() {
        let d = dialect();
        assert_eq!(d.type_name(&SqlType::BigInt), "INTEGER");
        assert_eq!(d.type_name(&SqlType::Varchar(255)), "TEXT");
        assert_eq!(d.type_name(&SqlType::Boolean), "INTEGER");
        assert_eq!(d.type_name(&SqlType::DateTime), "TEXT");
        assert_eq!(d.type_name(&SqlType::Blob), "BLOB");
    }
}
