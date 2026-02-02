//! PostgreSQL dialect for migrations.

use super::MigrationDialect;
use crate::ast::DataType;
use crate::migrations::column_builder::{ColumnDefinition, DefaultValue};
use crate::migrations::operation::{
    AlterColumnChange, AlterColumnOp, DropIndexOp, RenameColumnOp, RenameTableOp,
};

/// PostgreSQL dialect for migration SQL generation.
#[derive(Debug, Clone, Copy, Default)]
pub struct PostgresDialect;

impl PostgresDialect {
    /// Creates a new PostgreSQL dialect.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl MigrationDialect for PostgresDialect {
    fn name(&self) -> &'static str {
        "postgresql"
    }

    fn map_data_type(&self, dt: &DataType) -> String {
        match dt {
            DataType::Smallint => "SMALLINT".to_string(),
            DataType::Integer => "INTEGER".to_string(),
            DataType::Bigint => "BIGINT".to_string(),
            DataType::Real => "REAL".to_string(),
            DataType::Double => "DOUBLE PRECISION".to_string(),
            DataType::Decimal { precision, scale } => match (precision, scale) {
                (Some(p), Some(s)) => format!("DECIMAL({p}, {s})"),
                (Some(p), None) => format!("DECIMAL({p})"),
                _ => "DECIMAL".to_string(),
            },
            DataType::Numeric { precision, scale } => match (precision, scale) {
                (Some(p), Some(s)) => format!("NUMERIC({p}, {s})"),
                (Some(p), None) => format!("NUMERIC({p})"),
                _ => "NUMERIC".to_string(),
            },
            DataType::Char(len) => match len {
                Some(n) => format!("CHAR({n})"),
                None => "CHAR".to_string(),
            },
            DataType::Varchar(len) => match len {
                Some(n) => format!("VARCHAR({n})"),
                None => "VARCHAR".to_string(),
            },
            DataType::Text => "TEXT".to_string(),
            DataType::Blob => "BYTEA".to_string(), // PostgreSQL uses BYTEA
            DataType::Binary(len) => match len {
                Some(n) => format!("BIT({n})"),
                None => "BYTEA".to_string(),
            },
            DataType::Varbinary(len) => match len {
                Some(n) => format!("VARBIT({n})"),
                None => "BYTEA".to_string(),
            },
            DataType::Date => "DATE".to_string(),
            DataType::Time => "TIME".to_string(),
            DataType::Timestamp => "TIMESTAMP".to_string(),
            DataType::Datetime => "TIMESTAMP".to_string(), // PostgreSQL uses TIMESTAMP
            DataType::Boolean => "BOOLEAN".to_string(),
            DataType::Custom(name) => name.clone(),
        }
    }

    fn autoincrement_keyword(&self) -> String {
        // PostgreSQL uses SERIAL types instead of AUTOINCREMENT keyword
        // However, when PRIMARY KEY is specified with BIGINT, we don't change the type
        // The application should use SERIAL/BIGSERIAL types directly
        String::new()
    }

    fn column_definition(&self, col: &ColumnDefinition) -> String {
        // PostgreSQL uses SERIAL/BIGSERIAL for auto-increment
        let data_type = if col.autoincrement && col.primary_key {
            match col.data_type {
                DataType::Integer | DataType::Smallint => "SERIAL".to_string(),
                DataType::Bigint => "BIGSERIAL".to_string(),
                _ => self.map_data_type(&col.data_type),
            }
        } else {
            self.map_data_type(&col.data_type)
        };

        let mut sql = format!("{} {}", self.quote_identifier(&col.name), data_type);

        if col.primary_key {
            sql.push_str(" PRIMARY KEY");
        } else {
            if !col.nullable {
                sql.push_str(" NOT NULL");
            }
            if col.unique {
                sql.push_str(" UNIQUE");
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
            sql.push_str(&format!(" COLLATE \"{}\"", collation));
        }

        sql
    }

    fn render_default(&self, default: &DefaultValue) -> String {
        match default {
            DefaultValue::Boolean(b) => {
                if *b {
                    "TRUE".to_string()
                } else {
                    "FALSE".to_string()
                }
            }
            _ => default.to_sql(),
        }
    }

    fn rename_table(&self, op: &RenameTableOp) -> String {
        format!(
            "ALTER TABLE {} RENAME TO {}",
            self.quote_identifier(&op.old_name),
            self.quote_identifier(&op.new_name)
        )
    }

    fn rename_column(&self, op: &RenameColumnOp) -> String {
        format!(
            "ALTER TABLE {} RENAME COLUMN {} TO {}",
            self.quote_identifier(&op.table),
            self.quote_identifier(&op.old_name),
            self.quote_identifier(&op.new_name)
        )
    }

    fn alter_column(&self, op: &AlterColumnOp) -> String {
        let table = self.quote_identifier(&op.table);
        let column = self.quote_identifier(&op.column);

        match &op.change {
            AlterColumnChange::SetDataType(dt) => {
                format!(
                    "ALTER TABLE {} ALTER COLUMN {} TYPE {}",
                    table,
                    column,
                    self.map_data_type(dt)
                )
            }
            AlterColumnChange::SetNullable(nullable) => {
                if *nullable {
                    format!(
                        "ALTER TABLE {} ALTER COLUMN {} DROP NOT NULL",
                        table, column
                    )
                } else {
                    format!("ALTER TABLE {} ALTER COLUMN {} SET NOT NULL", table, column)
                }
            }
            AlterColumnChange::SetDefault(default) => {
                format!(
                    "ALTER TABLE {} ALTER COLUMN {} SET DEFAULT {}",
                    table,
                    column,
                    self.render_default(default)
                )
            }
            AlterColumnChange::DropDefault => {
                format!("ALTER TABLE {} ALTER COLUMN {} DROP DEFAULT", table, column)
            }
        }
    }

    fn drop_index(&self, op: &DropIndexOp) -> String {
        let mut sql = String::from("DROP INDEX ");
        if op.if_exists {
            sql.push_str("IF EXISTS ");
        }
        sql.push_str(&self.quote_identifier(&op.name));
        sql
    }

    fn drop_foreign_key(&self, op: &super::super::operation::DropForeignKeyOp) -> String {
        format!(
            "ALTER TABLE {} DROP CONSTRAINT {}",
            self.quote_identifier(&op.table),
            self.quote_identifier(&op.name)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migrations::column_builder::{bigint, varchar};
    use crate::migrations::table_builder::CreateTableBuilder;

    #[test]
    fn test_postgres_data_types() {
        let dialect = PostgresDialect::new();
        assert_eq!(dialect.map_data_type(&DataType::Integer), "INTEGER");
        assert_eq!(dialect.map_data_type(&DataType::Bigint), "BIGINT");
        assert_eq!(dialect.map_data_type(&DataType::Text), "TEXT");
        assert_eq!(
            dialect.map_data_type(&DataType::Varchar(Some(255))),
            "VARCHAR(255)"
        );
        assert_eq!(dialect.map_data_type(&DataType::Blob), "BYTEA");
        assert_eq!(dialect.map_data_type(&DataType::Boolean), "BOOLEAN");
        assert_eq!(dialect.map_data_type(&DataType::Timestamp), "TIMESTAMP");
        assert_eq!(
            dialect.map_data_type(&DataType::Decimal {
                precision: Some(10),
                scale: Some(2)
            }),
            "DECIMAL(10, 2)"
        );
    }

    #[test]
    fn test_create_table_with_serial() {
        let dialect = PostgresDialect::new();
        let op = CreateTableBuilder::new()
            .name("users")
            .column(bigint("id").primary_key().autoincrement().build())
            .column(varchar("username", 255).not_null().unique().build())
            .build();

        let sql = dialect.create_table(&op);
        assert!(sql.contains("CREATE TABLE \"users\""));
        assert!(sql.contains("\"id\" BIGSERIAL PRIMARY KEY"));
        assert!(sql.contains("\"username\" VARCHAR(255) NOT NULL UNIQUE"));
    }

    #[test]
    fn test_alter_column_sql() {
        let dialect = PostgresDialect::new();

        // Set NOT NULL
        let op = AlterColumnOp {
            table: "users".to_string(),
            column: "email".to_string(),
            change: AlterColumnChange::SetNullable(false),
        };
        assert_eq!(
            dialect.alter_column(&op),
            "ALTER TABLE \"users\" ALTER COLUMN \"email\" SET NOT NULL"
        );

        // Drop NOT NULL
        let op = AlterColumnOp {
            table: "users".to_string(),
            column: "email".to_string(),
            change: AlterColumnChange::SetNullable(true),
        };
        assert_eq!(
            dialect.alter_column(&op),
            "ALTER TABLE \"users\" ALTER COLUMN \"email\" DROP NOT NULL"
        );

        // Change type
        let op = AlterColumnOp {
            table: "users".to_string(),
            column: "age".to_string(),
            change: AlterColumnChange::SetDataType(DataType::Bigint),
        };
        assert_eq!(
            dialect.alter_column(&op),
            "ALTER TABLE \"users\" ALTER COLUMN \"age\" TYPE BIGINT"
        );
    }

    #[test]
    fn test_drop_foreign_key() {
        let dialect = PostgresDialect::new();
        let op = super::super::super::operation::DropForeignKeyOp {
            table: "invoices".to_string(),
            name: "fk_invoices_user".to_string(),
        };
        assert_eq!(
            dialect.drop_foreign_key(&op),
            "ALTER TABLE \"invoices\" DROP CONSTRAINT \"fk_invoices_user\""
        );
    }
}
