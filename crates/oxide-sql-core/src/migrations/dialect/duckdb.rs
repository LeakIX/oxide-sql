//! DuckDB dialect for migrations.

use super::MigrationDialect;
use crate::ast::DataType;
use crate::migrations::column_builder::{ColumnDefinition, DefaultValue};
use crate::migrations::operation::{
    AlterColumnChange, AlterColumnOp, CreateTableOp, DropIndexOp, RenameColumnOp, RenameTableOp,
};
use crate::schema::RustTypeMapping;

/// DuckDB dialect for migration SQL generation.
///
/// DuckDB does not support `AUTOINCREMENT` or `SERIAL`/`BIGSERIAL`.
/// Instead, auto-increment is implemented via `CREATE SEQUENCE` +
/// `DEFAULT nextval('seq_<table>_<column>')`.  The [`create_table`]
/// override emits the sequence DDL automatically for every column
/// marked with `autoincrement`.
#[derive(Debug, Clone, Copy, Default)]
pub struct DuckDbDialect;

impl DuckDbDialect {
    /// Creates a new DuckDB dialect.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Generates a column definition with sequence-backed default for
    /// autoincrement columns, using the given table name to build the
    /// sequence name.
    fn column_def_with_table(&self, col: &ColumnDefinition, table: &str) -> String {
        let data_type = self.map_data_type(&col.data_type);
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

        if col.autoincrement && col.default.is_none() {
            sql.push_str(&format!(" DEFAULT nextval('seq_{}_{}')", table, col.name,));
        } else if let Some(ref default) = col.default {
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
}

impl MigrationDialect for DuckDbDialect {
    fn name(&self) -> &'static str {
        "duckdb"
    }

    fn map_data_type(&self, dt: &DataType) -> String {
        match dt {
            DataType::Smallint => "SMALLINT".to_string(),
            DataType::Integer => "INTEGER".to_string(),
            DataType::Bigint => "BIGINT".to_string(),
            DataType::Real => "REAL".to_string(),
            DataType::Double => "DOUBLE".to_string(),
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
            DataType::Blob => "BLOB".to_string(),
            DataType::Binary(len) => match len {
                Some(n) => format!("BLOB({n})"),
                None => "BLOB".to_string(),
            },
            DataType::Varbinary(len) => match len {
                Some(n) => format!("BLOB({n})"),
                None => "BLOB".to_string(),
            },
            DataType::Date => "DATE".to_string(),
            DataType::Time => "TIME".to_string(),
            DataType::Timestamp => "TIMESTAMP".to_string(),
            DataType::Datetime => "TIMESTAMP".to_string(),
            DataType::Boolean => "BOOLEAN".to_string(),
            DataType::Custom(name) => name.clone(),
        }
    }

    fn autoincrement_keyword(&self) -> String {
        // DuckDB uses CREATE SEQUENCE + DEFAULT nextval() instead.
        String::new()
    }

    fn create_table(&self, op: &CreateTableOp) -> String {
        // Emit CREATE SEQUENCE for every autoincrement column.
        let mut sql = String::new();
        for col in &op.columns {
            if col.autoincrement {
                sql.push_str(&format!(
                    "CREATE SEQUENCE IF NOT EXISTS \
                     \"seq_{table}_{col}\" START 1;\n",
                    table = op.name,
                    col = col.name,
                ));
            }
        }

        sql.push_str("CREATE TABLE ");
        if op.if_not_exists {
            sql.push_str("IF NOT EXISTS ");
        }
        sql.push_str(&self.quote_identifier(&op.name));
        sql.push_str(" (\n");

        let column_defs: Vec<String> = op
            .columns
            .iter()
            .map(|c| format!("    {}", self.column_def_with_table(c, &op.name)))
            .collect();
        sql.push_str(&column_defs.join(",\n"));

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
                    "ALTER TABLE {} ALTER COLUMN {} SET DATA TYPE {}",
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
            AlterColumnChange::SetUnique(true) => {
                format!("ALTER TABLE {} ADD UNIQUE ({})", table, column)
            }
            AlterColumnChange::SetUnique(false) => {
                format!(
                    "ALTER TABLE {} DROP CONSTRAINT \"{}_key\"",
                    table, op.column
                )
            }
            AlterColumnChange::SetAutoincrement(_) => {
                format!(
                    "-- DuckDB cannot ALTER autoincrement \
                     for {}.{}; table recreation required",
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

impl RustTypeMapping for DuckDbDialect {
    fn map_type(&self, rust_type: &str) -> DataType {
        match rust_type {
            "bool" => DataType::Boolean,
            "i8" | "i16" | "u8" | "u16" => DataType::Smallint,
            "i32" | "u32" => DataType::Integer,
            "i64" | "u64" | "i128" | "u128" | "isize" | "usize" => DataType::Bigint,
            "f32" => DataType::Real,
            "f64" => DataType::Double,
            "String" => DataType::Varchar(None),
            "Vec<u8>" => DataType::Blob,
            s if s.contains("DateTime") => DataType::Timestamp,
            s if s.contains("NaiveDate") => DataType::Date,
            _ => DataType::Text,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migrations::column_builder::{integer, varchar};
    use crate::migrations::operation::{DropTableOp, Operation, RenameColumnOp, RenameTableOp};
    use crate::migrations::table_builder::CreateTableBuilder;

    #[test]
    fn test_duckdb_data_types() {
        let d = DuckDbDialect::new();
        assert_eq!(d.map_data_type(&DataType::Integer), "INTEGER");
        assert_eq!(d.map_data_type(&DataType::Bigint), "BIGINT");
        assert_eq!(d.map_data_type(&DataType::Text), "TEXT");
        assert_eq!(
            d.map_data_type(&DataType::Varchar(Some(255))),
            "VARCHAR(255)"
        );
        assert_eq!(d.map_data_type(&DataType::Blob), "BLOB");
        assert_eq!(d.map_data_type(&DataType::Boolean), "BOOLEAN");
        assert_eq!(d.map_data_type(&DataType::Timestamp), "TIMESTAMP");
        assert_eq!(d.map_data_type(&DataType::Double), "DOUBLE");
        assert_eq!(d.map_data_type(&DataType::Real), "REAL");
        assert_eq!(d.map_data_type(&DataType::Date), "DATE");
        assert_eq!(d.map_data_type(&DataType::Time), "TIME");
        assert_eq!(
            d.map_data_type(&DataType::Decimal {
                precision: Some(10),
                scale: Some(2)
            }),
            "DECIMAL(10, 2)"
        );
    }

    #[test]
    fn test_create_table_basic() {
        let d = DuckDbDialect::new();
        let op = CreateTableBuilder::new()
            .name("users")
            .column(varchar("username", 255).not_null().unique().build())
            .build();

        let sql = d.create_table(&op);
        assert_eq!(
            sql,
            "CREATE TABLE \"users\" (\n\
             \x20   \"username\" VARCHAR(255) NOT NULL UNIQUE\n\
             )"
        );
    }

    #[test]
    fn test_create_table_if_not_exists() {
        let d = DuckDbDialect::new();
        let op = CreateTableBuilder::new()
            .if_not_exists()
            .name("users")
            .column(varchar("username", 255).not_null().build())
            .build();

        let sql = d.create_table(&op);
        assert!(sql.contains("CREATE TABLE IF NOT EXISTS \"users\""));
    }

    #[test]
    fn test_autoincrement_generates_sequence() {
        let d = DuckDbDialect::new();
        let op = CreateTableBuilder::new()
            .name("users")
            .column(integer("id").primary_key().autoincrement().build())
            .column(varchar("username", 255).not_null().unique().build())
            .build();

        let sql = d.create_table(&op);

        assert!(
            sql.contains(
                "CREATE SEQUENCE IF NOT EXISTS \
                 \"seq_users_id\" START 1;"
            ),
            "Missing sequence DDL in:\n{sql}"
        );
        assert!(
            sql.contains("DEFAULT nextval('seq_users_id')"),
            "Missing nextval default in:\n{sql}"
        );
        assert!(
            !sql.contains("AUTOINCREMENT"),
            "Should not contain AUTOINCREMENT keyword"
        );
    }

    #[test]
    fn test_varchar_unique_not_null() {
        let d = DuckDbDialect::new();
        let op = CreateTableBuilder::new()
            .name("items")
            .column(varchar("domain", 255).not_null().unique().build())
            .build();

        let sql = d.create_table(&op);
        assert!(
            sql.contains("\"domain\" VARCHAR(255) NOT NULL UNIQUE"),
            "Expected NOT NULL UNIQUE in:\n{sql}"
        );
    }

    #[test]
    fn test_drop_table() {
        let d = DuckDbDialect::new();

        let op = DropTableOp {
            name: "users".to_string(),
            if_exists: false,
            cascade: false,
        };
        assert_eq!(d.drop_table(&op), "DROP TABLE \"users\"");

        let op = DropTableOp {
            name: "users".to_string(),
            if_exists: true,
            cascade: true,
        };
        assert_eq!(d.drop_table(&op), "DROP TABLE IF EXISTS \"users\" CASCADE");
    }

    #[test]
    fn test_rename_table() {
        let d = DuckDbDialect::new();
        let op = RenameTableOp {
            old_name: "old_users".to_string(),
            new_name: "users".to_string(),
        };
        assert_eq!(
            d.rename_table(&op),
            "ALTER TABLE \"old_users\" RENAME TO \"users\""
        );
    }

    #[test]
    fn test_add_column() {
        let d = DuckDbDialect::new();
        let op = Operation::add_column("users", varchar("email", 255).not_null().build());
        if let Operation::AddColumn(ref add_op) = op {
            let sql = d.add_column(add_op);
            assert_eq!(
                sql,
                "ALTER TABLE \"users\" ADD COLUMN \
                 \"email\" VARCHAR(255) NOT NULL"
            );
        }
    }

    #[test]
    fn test_drop_column() {
        let d = DuckDbDialect::new();
        let op = Operation::drop_column("users", "email");
        if let Operation::DropColumn(ref drop_op) = op {
            let sql = d.drop_column(drop_op);
            assert_eq!(sql, "ALTER TABLE \"users\" DROP COLUMN \"email\"");
        }
    }

    #[test]
    fn test_rename_column() {
        let d = DuckDbDialect::new();
        let op = RenameColumnOp {
            table: "users".to_string(),
            old_name: "name".to_string(),
            new_name: "full_name".to_string(),
        };
        assert_eq!(
            d.rename_column(&op),
            "ALTER TABLE \"users\" RENAME COLUMN \
             \"name\" TO \"full_name\""
        );
    }

    #[test]
    fn test_alter_column_set_data_type() {
        let d = DuckDbDialect::new();
        let op = AlterColumnOp {
            table: "users".to_string(),
            column: "age".to_string(),
            change: AlterColumnChange::SetDataType(DataType::Bigint),
        };
        assert_eq!(
            d.alter_column(&op),
            "ALTER TABLE \"users\" ALTER COLUMN \"age\" \
             SET DATA TYPE BIGINT"
        );
    }

    #[test]
    fn test_alter_column_set_not_null() {
        let d = DuckDbDialect::new();
        let op = AlterColumnOp {
            table: "users".to_string(),
            column: "email".to_string(),
            change: AlterColumnChange::SetNullable(false),
        };
        assert_eq!(
            d.alter_column(&op),
            "ALTER TABLE \"users\" ALTER COLUMN \"email\" SET NOT NULL"
        );
    }

    #[test]
    fn test_alter_column_drop_not_null() {
        let d = DuckDbDialect::new();
        let op = AlterColumnOp {
            table: "users".to_string(),
            column: "email".to_string(),
            change: AlterColumnChange::SetNullable(true),
        };
        assert_eq!(
            d.alter_column(&op),
            "ALTER TABLE \"users\" ALTER COLUMN \"email\" DROP NOT NULL"
        );
    }

    #[test]
    fn test_alter_column_set_default() {
        let d = DuckDbDialect::new();
        let op = AlterColumnOp {
            table: "users".to_string(),
            column: "active".to_string(),
            change: AlterColumnChange::SetDefault(DefaultValue::Boolean(true)),
        };
        assert_eq!(
            d.alter_column(&op),
            "ALTER TABLE \"users\" ALTER COLUMN \"active\" \
             SET DEFAULT TRUE"
        );
    }

    #[test]
    fn test_alter_column_drop_default() {
        let d = DuckDbDialect::new();
        let op = AlterColumnOp {
            table: "users".to_string(),
            column: "active".to_string(),
            change: AlterColumnChange::DropDefault,
        };
        assert_eq!(
            d.alter_column(&op),
            "ALTER TABLE \"users\" ALTER COLUMN \"active\" DROP DEFAULT"
        );
    }

    #[test]
    fn test_create_index() {
        let d = DuckDbDialect::new();
        let op = crate::migrations::operation::CreateIndexOp {
            name: "idx_users_email".to_string(),
            table: "users".to_string(),
            columns: vec!["email".to_string()],
            unique: true,
            index_type: crate::migrations::operation::IndexType::BTree,
            if_not_exists: true,
            condition: None,
        };
        assert_eq!(
            d.create_index(&op),
            "CREATE UNIQUE INDEX IF NOT EXISTS \"idx_users_email\" \
             ON \"users\" (\"email\")"
        );
    }

    #[test]
    fn test_drop_index() {
        let d = DuckDbDialect::new();

        let op = crate::migrations::operation::DropIndexOp {
            name: "idx_users_email".to_string(),
            table: None,
            if_exists: false,
        };
        assert_eq!(d.drop_index(&op), "DROP INDEX \"idx_users_email\"");

        let op = crate::migrations::operation::DropIndexOp {
            name: "idx_users_email".to_string(),
            table: None,
            if_exists: true,
        };
        assert_eq!(
            d.drop_index(&op),
            "DROP INDEX IF EXISTS \"idx_users_email\""
        );
    }

    #[test]
    fn test_drop_foreign_key() {
        let d = DuckDbDialect::new();
        let op = crate::migrations::operation::DropForeignKeyOp {
            table: "invoices".to_string(),
            name: "fk_invoices_user".to_string(),
        };
        assert_eq!(
            d.drop_foreign_key(&op),
            "ALTER TABLE \"invoices\" DROP CONSTRAINT \
             \"fk_invoices_user\""
        );
    }

    #[test]
    fn test_consumer_scenario_two_tables_with_sequences() {
        let d = DuckDbDialect::new();

        let ops: Vec<Operation> = vec![
            CreateTableBuilder::new()
                .if_not_exists()
                .name("excluded_domains")
                .column(integer("id").primary_key().autoincrement().build())
                .column(varchar("domain", 255).not_null().unique().build())
                .build()
                .into(),
            CreateTableBuilder::new()
                .if_not_exists()
                .name("excluded_ips")
                .column(integer("id").primary_key().autoincrement().build())
                .column(varchar("cidr", 255).not_null().unique().build())
                .build()
                .into(),
        ];

        let sqls: Vec<String> = ops.iter().map(|op| d.generate_sql(op)).collect();

        // First table
        assert!(
            sqls[0].contains(
                "CREATE SEQUENCE IF NOT EXISTS \
                 \"seq_excluded_domains_id\" START 1;"
            ),
            "Missing sequence for excluded_domains:\n{}",
            sqls[0]
        );
        assert!(
            sqls[0].contains("CREATE TABLE IF NOT EXISTS \"excluded_domains\""),
            "Missing CREATE TABLE:\n{}",
            sqls[0]
        );
        assert!(
            sqls[0].contains("DEFAULT nextval('seq_excluded_domains_id')"),
            "Missing nextval default:\n{}",
            sqls[0]
        );
        assert!(
            sqls[0].contains("\"domain\" VARCHAR(255) NOT NULL UNIQUE"),
            "Missing domain column:\n{}",
            sqls[0]
        );

        // Second table
        assert!(
            sqls[1].contains(
                "CREATE SEQUENCE IF NOT EXISTS \
                 \"seq_excluded_ips_id\" START 1;"
            ),
            "Missing sequence for excluded_ips:\n{}",
            sqls[1]
        );
        assert!(
            sqls[1].contains("DEFAULT nextval('seq_excluded_ips_id')"),
            "Missing nextval default:\n{}",
            sqls[1]
        );
        assert!(
            sqls[1].contains("\"cidr\" VARCHAR(255) NOT NULL UNIQUE"),
            "Missing cidr column:\n{}",
            sqls[1]
        );
    }
}
