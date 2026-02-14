//! SQLite dialect for migrations.

use super::MigrationDialect;
use crate::ast::DataType;
use crate::migrations::operation::{
    AlterColumnChange, AlterColumnOp, DropIndexOp, RenameColumnOp, RenameTableOp,
};
use crate::schema::RustTypeMapping;

/// SQLite dialect for migration SQL generation.
#[derive(Debug, Clone, Copy, Default)]
pub struct SqliteDialect;

impl SqliteDialect {
    /// Creates a new SQLite dialect.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl MigrationDialect for SqliteDialect {
    fn name(&self) -> &'static str {
        "sqlite"
    }

    fn map_data_type(&self, dt: &DataType) -> String {
        // SQLite has dynamic typing with type affinity
        match dt {
            DataType::Smallint | DataType::Integer | DataType::Bigint => "INTEGER".to_string(),
            DataType::Real | DataType::Double => "REAL".to_string(),
            DataType::Decimal { .. } | DataType::Numeric { .. } => "REAL".to_string(),
            DataType::Char(_) | DataType::Varchar(_) | DataType::Text => "TEXT".to_string(),
            DataType::Blob | DataType::Binary(_) | DataType::Varbinary(_) => "BLOB".to_string(),
            DataType::Date | DataType::Time | DataType::Timestamp | DataType::Datetime => {
                "TEXT".to_string()
            }
            DataType::Boolean => "INTEGER".to_string(), // SQLite has no bool, use 0/1
            DataType::Custom(name) => name.clone(),
        }
    }

    fn autoincrement_keyword(&self) -> String {
        " AUTOINCREMENT".to_string()
    }

    fn rename_table(&self, op: &RenameTableOp) -> String {
        format!(
            "ALTER TABLE {} RENAME TO {}",
            self.quote_identifier(&op.old_name),
            self.quote_identifier(&op.new_name)
        )
    }

    fn rename_column(&self, op: &RenameColumnOp) -> String {
        // SQLite 3.25.0+ supports RENAME COLUMN
        format!(
            "ALTER TABLE {} RENAME COLUMN {} TO {}",
            self.quote_identifier(&op.table),
            self.quote_identifier(&op.old_name),
            self.quote_identifier(&op.new_name)
        )
    }

    fn alter_column(&self, op: &AlterColumnOp) -> String {
        // SQLite has very limited ALTER TABLE support.
        // Most column alterations require recreating the table.
        // We generate a comment noting this limitation.
        match &op.change {
            AlterColumnChange::SetDataType(_) => {
                format!(
                    "-- SQLite does not support ALTER COLUMN TYPE directly for {}.{}; \
                     table recreation required",
                    op.table, op.column
                )
            }
            AlterColumnChange::SetNullable(_) => {
                format!(
                    "-- SQLite does not support ALTER COLUMN NULL/NOT NULL directly for {}.{}; \
                     table recreation required",
                    op.table, op.column
                )
            }
            AlterColumnChange::SetDefault(default) => {
                // SQLite doesn't support ALTER COLUMN SET DEFAULT either
                format!(
                    "-- SQLite does not support ALTER COLUMN SET DEFAULT directly for {}.{}; \
                     would set to: {}",
                    op.table,
                    op.column,
                    self.render_default(default)
                )
            }
            AlterColumnChange::DropDefault => {
                format!(
                    "-- SQLite does not support ALTER COLUMN DROP DEFAULT directly for {}.{}; \
                     table recreation required",
                    op.table, op.column
                )
            }
            AlterColumnChange::SetUnique(_) => {
                format!(
                    "-- SQLite does not support ALTER COLUMN UNIQUE directly for {}.{}; \
                     table recreation required",
                    op.table, op.column
                )
            }
            AlterColumnChange::SetAutoincrement(_) => {
                format!(
                    "-- SQLite does not support ALTER autoincrement for {}.{}; \
                     table recreation required",
                    op.table, op.column
                )
            }
        }
    }

    fn drop_index(&self, op: &DropIndexOp) -> String {
        let mut sql = String::from("DROP INDEX ");
        if op.if_exists {
            sql.push_str("IF EXISTS ");
        }
        // SQLite index names are global, not per-table
        sql.push_str(&self.quote_identifier(&op.name));
        sql
    }

    fn drop_foreign_key(&self, op: &super::super::operation::DropForeignKeyOp) -> String {
        // SQLite does not support DROP CONSTRAINT; requires table recreation
        format!(
            "-- SQLite does not support DROP CONSTRAINT; \
             table recreation required to remove foreign key {} from {}",
            op.name, op.table
        )
    }
}

impl RustTypeMapping for SqliteDialect {
    fn map_type(&self, rust_type: &str) -> DataType {
        match rust_type {
            "bool" => DataType::Integer,
            "i8" | "i16" | "u8" | "u16" | "i32" | "u32" => DataType::Integer,
            "i64" | "u64" | "i128" | "u128" | "isize" | "usize" => DataType::Bigint,
            "f32" => DataType::Real,
            "f64" => DataType::Double,
            "String" => DataType::Text,
            "Vec<u8>" => DataType::Blob,
            s if s.contains("DateTime") => DataType::Text,
            s if s.contains("NaiveDate") => DataType::Text,
            _ => DataType::Text, // safe fallback for SQLite
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migrations::column_builder::{bigint, boolean, timestamp, varchar};
    use crate::migrations::operation::{DropTableOp, Operation};
    use crate::migrations::table_builder::CreateTableBuilder;

    #[test]
    fn test_sqlite_data_types() {
        let dialect = SqliteDialect::new();
        assert_eq!(dialect.map_data_type(&DataType::Integer), "INTEGER");
        assert_eq!(dialect.map_data_type(&DataType::Bigint), "INTEGER");
        assert_eq!(dialect.map_data_type(&DataType::Text), "TEXT");
        assert_eq!(dialect.map_data_type(&DataType::Varchar(Some(255))), "TEXT");
        assert_eq!(dialect.map_data_type(&DataType::Blob), "BLOB");
        assert_eq!(dialect.map_data_type(&DataType::Boolean), "INTEGER");
        assert_eq!(dialect.map_data_type(&DataType::Timestamp), "TEXT");
    }

    #[test]
    fn test_create_table_sql() {
        let dialect = SqliteDialect::new();
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

        let sql = dialect.create_table(&op);
        assert!(sql.contains("CREATE TABLE \"users\""));
        assert!(sql.contains("\"id\" INTEGER PRIMARY KEY AUTOINCREMENT"));
        assert!(sql.contains("\"username\" TEXT NOT NULL UNIQUE"));
        assert!(sql.contains("DEFAULT CURRENT_TIMESTAMP"));
    }

    #[test]
    fn test_drop_table_sql() {
        let dialect = SqliteDialect::new();

        let op = DropTableOp {
            name: "users".to_string(),
            if_exists: false,
            cascade: false,
        };
        assert_eq!(dialect.drop_table(&op), "DROP TABLE \"users\"");

        let op = DropTableOp {
            name: "users".to_string(),
            if_exists: true,
            cascade: false,
        };
        assert_eq!(dialect.drop_table(&op), "DROP TABLE IF EXISTS \"users\"");
    }

    #[test]
    fn test_add_column_sql() {
        let dialect = SqliteDialect::new();
        let op = Operation::add_column(
            "users",
            boolean("active").not_null().default_bool(true).build(),
        );

        if let Operation::AddColumn(add_op) = op {
            let sql = dialect.add_column(&add_op);
            assert!(sql.contains("ALTER TABLE \"users\" ADD COLUMN"));
            assert!(sql.contains("\"active\" INTEGER NOT NULL DEFAULT TRUE"));
        }
    }

    #[test]
    fn test_rename_table_sql() {
        let dialect = SqliteDialect::new();
        let op = RenameTableOp {
            old_name: "old_users".to_string(),
            new_name: "users".to_string(),
        };
        assert_eq!(
            dialect.rename_table(&op),
            "ALTER TABLE \"old_users\" RENAME TO \"users\""
        );
    }

    #[test]
    fn test_rename_column_sql() {
        let dialect = SqliteDialect::new();
        let op = RenameColumnOp {
            table: "users".to_string(),
            old_name: "name".to_string(),
            new_name: "full_name".to_string(),
        };
        assert_eq!(
            dialect.rename_column(&op),
            "ALTER TABLE \"users\" RENAME COLUMN \"name\" TO \"full_name\""
        );
    }
}
