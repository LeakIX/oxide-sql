//! Migration code generation.
//!
//! Generates Rust source code implementing the [`Migration`] trait
//! from a [`SchemaDiff`], enabling `makemigrations`-style tooling.

use super::column_builder::DefaultValue;
use super::diff::SchemaDiff;
use super::operation::{AlterColumnChange, CreateTableOp, Operation};
use crate::ast::DataType;

/// Generates a Rust source string containing a `Migration` impl
/// for the given diff.
///
/// # Arguments
///
/// * `id` — The migration ID (e.g. `"0002_add_email"`).
/// * `diff` — The schema diff to translate into code.
///
/// # Returns
///
/// A Rust source string that, when compiled, produces a struct
/// implementing the `Migration` trait with `up()` and `down()`
/// methods.
#[must_use]
pub fn generate_migration_code(id: &str, diff: &SchemaDiff) -> String {
    let struct_name = id_to_struct_name(id);
    let up_body = render_operations(&diff.operations);
    let down_body = render_down(&diff.operations);

    format!(
        "use oxide_sql_core::migrations::{{\n\
         \x20   Migration, Operation, CreateTableBuilder,\n\
         \x20   bigint, varchar, text, integer, smallint,\n\
         \x20   boolean, timestamp, datetime, date, time,\n\
         \x20   real, double, decimal, numeric, blob, binary,\n\
         \x20   varbinary, char,\n\
         }};\n\
         \n\
         pub struct {struct_name};\n\
         \n\
         impl Migration for {struct_name} {{\n\
         \x20   const ID: &'static str = \"{id}\";\n\
         \n\
         \x20   fn up() -> Vec<Operation> {{\n\
         \x20       vec![\n\
         {up_body}\
         \x20       ]\n\
         \x20   }}\n\
         \n\
         \x20   fn down() -> Vec<Operation> {{\n\
         \x20       vec![\n\
         {down_body}\
         \x20       ]\n\
         \x20   }}\n\
         }}\n"
    )
}

// ================================================================
// Internal helpers
// ================================================================

/// Converts a migration ID like "0002_add_email" into a struct
/// name like "Migration0002AddEmail".
fn id_to_struct_name(id: &str) -> String {
    let mut result = String::from("Migration");
    let mut capitalize_next = true;
    for ch in id.chars() {
        if ch == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(ch.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }
    result
}

/// Renders a list of operations as Rust expressions.
fn render_operations(ops: &[Operation]) -> String {
    let mut out = String::new();
    for op in ops {
        out.push_str(&format!("            {},\n", render_operation(op)));
    }
    out
}

/// Renders the `down()` body from the up operations.
fn render_down(ops: &[Operation]) -> String {
    let mut out = String::new();
    for op in ops.iter().rev() {
        match op.reverse() {
            Some(rev) => {
                out.push_str(&format!("            {},\n", render_operation(&rev)));
            }
            None => {
                out.push_str(&format!(
                    "            // TODO: cannot auto-reverse: \
                     {:?}\n",
                    op_summary(op)
                ));
            }
        }
    }
    out
}

/// Short human-readable summary for comments.
fn op_summary(op: &Operation) -> String {
    match op {
        Operation::CreateTable(ct) => {
            format!("CreateTable({})", ct.name)
        }
        Operation::DropTable(dt) => {
            format!("DropTable({})", dt.name)
        }
        Operation::RenameTable(rt) => {
            format!("RenameTable({} -> {})", rt.old_name, rt.new_name)
        }
        Operation::AddColumn(ac) => {
            format!("AddColumn({}.{})", ac.table, ac.column.name)
        }
        Operation::DropColumn(dc) => {
            format!("DropColumn({}.{})", dc.table, dc.column)
        }
        Operation::AlterColumn(ac) => {
            format!("AlterColumn({}.{})", ac.table, ac.column)
        }
        Operation::RenameColumn(rc) => {
            format!(
                "RenameColumn({}.{} -> {})",
                rc.table, rc.old_name, rc.new_name
            )
        }
        Operation::CreateIndex(ci) => {
            format!("CreateIndex({})", ci.name)
        }
        Operation::DropIndex(di) => {
            format!("DropIndex({})", di.name)
        }
        Operation::AddForeignKey(fk) => {
            format!("AddForeignKey({} -> {})", fk.table, fk.references_table)
        }
        Operation::DropForeignKey(fk) => {
            format!("DropForeignKey({}.{})", fk.table, fk.name)
        }
        Operation::RunSql(_) => "RunSql(...)".to_string(),
    }
}

/// Renders a single Operation as a Rust expression string.
fn render_operation(op: &Operation) -> String {
    match op {
        Operation::CreateTable(ct) => render_create_table(ct),
        Operation::DropTable(dt) => {
            format!("Operation::drop_table(\"{}\")", dt.name)
        }
        Operation::RenameTable(rt) => {
            format!(
                "Operation::rename_table(\"{}\", \"{}\")",
                rt.old_name, rt.new_name
            )
        }
        Operation::AddColumn(ac) => {
            format!(
                "Operation::add_column(\"{}\", {})",
                ac.table,
                render_column_builder(&ac.column.name, &ac.column)
            )
        }
        Operation::DropColumn(dc) => {
            format!(
                "Operation::drop_column(\"{}\", \"{}\")",
                dc.table, dc.column
            )
        }
        Operation::RenameColumn(rc) => {
            format!(
                "Operation::rename_column(\"{}\", \"{}\", \"{}\")",
                rc.table, rc.old_name, rc.new_name
            )
        }
        Operation::AlterColumn(ac) => render_alter_column(ac),
        Operation::CreateIndex(ci) => {
            format!(
                "Operation::CreateIndex(CreateIndexOp {{ \
                 name: \"{}\".into(), \
                 table: \"{}\".into(), \
                 columns: vec![{}], \
                 unique: {}, \
                 index_type: IndexType::BTree, \
                 if_not_exists: false, \
                 condition: None \
                 }})",
                ci.name,
                ci.table,
                ci.columns
                    .iter()
                    .map(|c| format!("\"{c}\".into()"))
                    .collect::<Vec<_>>()
                    .join(", "),
                ci.unique,
            )
        }
        Operation::DropIndex(di) => {
            format!(
                "Operation::DropIndex(DropIndexOp {{ \
                 name: \"{}\".into(), table: None, \
                 if_exists: false }})",
                di.name
            )
        }
        Operation::AddForeignKey(_) | Operation::DropForeignKey(_) => {
            format!("// TODO: manually write FK operation: {:?}", op_summary(op))
        }
        Operation::RunSql(rs) => {
            if let Some(ref down) = rs.down_sql {
                format!(
                    "Operation::run_sql_reversible(\"{}\", \"{}\")",
                    escape_str(&rs.up_sql),
                    escape_str(down)
                )
            } else {
                format!("Operation::run_sql(\"{}\")", escape_str(&rs.up_sql))
            }
        }
    }
}

/// Renders a `CreateTableBuilder` chain.
fn render_create_table(ct: &CreateTableOp) -> String {
    let mut s = String::from("CreateTableBuilder::new()\n");
    s.push_str(&format!("                .name(\"{}\")\n", ct.name));
    for col in &ct.columns {
        s.push_str(&format!(
            "                .column({})\n",
            render_column_builder(&col.name, col)
        ));
    }
    if ct.if_not_exists {
        s.push_str("                .if_not_exists()\n");
    }
    s.push_str("                .build()\n");
    s.push_str("                .into()");
    s
}

/// Renders a column builder expression.
fn render_column_builder(_name: &str, col: &super::column_builder::ColumnDefinition) -> String {
    let type_fn = match &col.data_type {
        DataType::Bigint => {
            format!("bigint(\"{}\")", col.name)
        }
        DataType::Integer => {
            format!("integer(\"{}\")", col.name)
        }
        DataType::Smallint => {
            format!("smallint(\"{}\")", col.name)
        }
        DataType::Text => {
            format!("text(\"{}\")", col.name)
        }
        DataType::Varchar(Some(len)) => {
            format!("varchar(\"{}\", {len})", col.name)
        }
        DataType::Varchar(None) => {
            format!("text(\"{}\")", col.name)
        }
        DataType::Boolean => {
            format!("boolean(\"{}\")", col.name)
        }
        DataType::Timestamp => {
            format!("timestamp(\"{}\")", col.name)
        }
        DataType::Datetime => {
            format!("datetime(\"{}\")", col.name)
        }
        DataType::Date => {
            format!("date(\"{}\")", col.name)
        }
        DataType::Time => {
            format!("time(\"{}\")", col.name)
        }
        DataType::Real => {
            format!("real(\"{}\")", col.name)
        }
        DataType::Double => {
            format!("double(\"{}\")", col.name)
        }
        DataType::Blob => {
            format!("blob(\"{}\")", col.name)
        }
        DataType::Decimal {
            precision: Some(p),
            scale: Some(s),
        } => {
            format!("decimal(\"{}\", {p}, {s})", col.name)
        }
        DataType::Numeric {
            precision: Some(p),
            scale: Some(s),
        } => {
            format!("numeric(\"{}\", {p}, {s})", col.name)
        }
        DataType::Char(Some(len)) => {
            format!("char(\"{}\", {len})", col.name)
        }
        _ => format!("text(\"{}\")", col.name),
    };

    let mut chain = type_fn;
    if col.primary_key {
        chain.push_str(".primary_key()");
    }
    if col.autoincrement {
        chain.push_str(".autoincrement()");
    }
    if !col.nullable && !col.primary_key {
        chain.push_str(".not_null()");
    }
    if col.unique {
        chain.push_str(".unique()");
    }
    if let Some(ref default) = col.default {
        match default {
            DefaultValue::Boolean(b) => {
                chain.push_str(&format!(".default_bool({b})"));
            }
            DefaultValue::Integer(i) => {
                chain.push_str(&format!(".default_int({i})"));
            }
            DefaultValue::Float(f) => {
                chain.push_str(&format!(".default_float({f})"));
            }
            DefaultValue::String(s) => {
                chain.push_str(&format!(".default_str(\"{}\")", escape_str(s)));
            }
            DefaultValue::Null => {
                chain.push_str(".default_null()");
            }
            DefaultValue::Expression(expr) => {
                chain.push_str(&format!(".default_expr(\"{}\")", escape_str(expr)));
            }
        }
    }
    chain.push_str(".build()");
    chain
}

/// Renders an AlterColumn operation as Rust code.
fn render_alter_column(ac: &super::operation::AlterColumnOp) -> String {
    let change = match &ac.change {
        AlterColumnChange::SetDataType(dt) => {
            format!("AlterColumnChange::SetDataType(DataType::{:?})", dt)
        }
        AlterColumnChange::SetNullable(n) => {
            format!("AlterColumnChange::SetNullable({n})")
        }
        AlterColumnChange::SetDefault(d) => {
            format!("AlterColumnChange::SetDefault({})", render_default_value(d))
        }
        AlterColumnChange::DropDefault => "AlterColumnChange::DropDefault".to_string(),
        AlterColumnChange::SetUnique(u) => {
            format!("AlterColumnChange::SetUnique({u})")
        }
        AlterColumnChange::SetAutoincrement(a) => {
            format!("AlterColumnChange::SetAutoincrement({a})")
        }
    };
    format!(
        "Operation::AlterColumn(AlterColumnOp {{ \
         table: \"{}\".into(), \
         column: \"{}\".into(), \
         change: {} }})",
        ac.table, ac.column, change
    )
}

/// Renders a DefaultValue as Rust code.
fn render_default_value(dv: &DefaultValue) -> String {
    match dv {
        DefaultValue::Null => "DefaultValue::Null".to_string(),
        DefaultValue::Boolean(b) => {
            format!("DefaultValue::Boolean({b})")
        }
        DefaultValue::Integer(i) => {
            format!("DefaultValue::Integer({i})")
        }
        DefaultValue::Float(f) => {
            format!("DefaultValue::Float({f})")
        }
        DefaultValue::String(s) => {
            format!("DefaultValue::String(\"{}\".into())", escape_str(s))
        }
        DefaultValue::Expression(e) => {
            format!("DefaultValue::Expression(\"{}\".into())", escape_str(e))
        }
    }
}

/// Escapes a string for inclusion in a Rust string literal.
fn escape_str(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migrations::column_builder::varchar;
    use crate::migrations::diff::SchemaDiff;
    use crate::migrations::operation::Operation;
    use crate::migrations::table_builder::CreateTableBuilder;

    #[test]
    fn id_to_struct_name_works() {
        assert_eq!(
            id_to_struct_name("0001_create_users"),
            "Migration0001CreateUsers"
        );
        assert_eq!(id_to_struct_name("0002_add_email"), "Migration0002AddEmail");
    }

    #[test]
    fn generate_simple_migration() {
        let diff = SchemaDiff {
            operations: vec![Operation::add_column(
                "users",
                varchar("email", 255).not_null().build(),
            )],
            ambiguous: vec![],
            warnings: vec![],
        };

        let code = generate_migration_code("0002_add_email", &diff);
        assert!(code.contains("struct Migration0002AddEmail"));
        assert!(code.contains("fn up()"));
        assert!(code.contains("fn down()"));
        assert!(code.contains("add_column"));
        assert!(code.contains("varchar"));
        assert!(code.contains("drop_column"));
    }

    #[test]
    fn generate_create_table_migration() {
        let op: Operation = CreateTableBuilder::new()
            .name("users")
            .column(
                crate::migrations::column_builder::bigint("id")
                    .primary_key()
                    .autoincrement()
                    .build(),
            )
            .column(varchar("name", 255).not_null().unique().build())
            .build()
            .into();

        let diff = SchemaDiff {
            operations: vec![op],
            ambiguous: vec![],
            warnings: vec![],
        };

        let code = generate_migration_code("0001_create_users", &diff);
        assert!(code.contains("CreateTableBuilder::new()"));
        assert!(code.contains(".primary_key()"));
        assert!(code.contains(".autoincrement()"));
        assert!(code.contains(".unique()"));
        // down should have drop_table
        assert!(code.contains("drop_table"));
    }
}
