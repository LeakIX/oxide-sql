//! Tests for INSERT statements: with/without columns, multiple rows,
//! INSERT ... SELECT, DEFAULT VALUES, schema-qualified, expressions,
//! and parameters.

mod common;
use common::*;

use oxide_sql_core::ast::{BinaryOp, Expr, InsertSource};

#[test]
fn insert_with_columns() {
    let i = parse_insert(
        "INSERT INTO users (name, email) \
         VALUES ('Alice', 'alice@example.com')",
    );
    assert_eq!(i.table, "users");
    assert_eq!(i.columns, vec!["name", "email"]);
    if let InsertSource::Values(rows) = &i.values {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].len(), 2);
    } else {
        panic!("Expected VALUES");
    }
    round_trip("INSERT INTO users (name, email) VALUES ('Alice', 'alice@example.com')");
}

#[test]
fn insert_without_columns() {
    let i = parse_insert("INSERT INTO users VALUES (1, 'Bob', 'b@x.com')");
    assert!(i.columns.is_empty());
    if let InsertSource::Values(rows) = &i.values {
        assert_eq!(rows[0].len(), 3);
    } else {
        panic!("Expected VALUES");
    }
    round_trip("INSERT INTO users VALUES (1, 'Bob', 'b@x.com')");
}

#[test]
fn insert_multiple_rows() {
    let i = parse_insert("INSERT INTO users (name) VALUES ('A'), ('B'), ('C')");
    if let InsertSource::Values(rows) = &i.values {
        assert_eq!(rows.len(), 3);
    } else {
        panic!("Expected VALUES");
    }
    round_trip("INSERT INTO users (name) VALUES ('A'), ('B'), ('C')");
}

#[test]
fn insert_select() {
    let i = parse_insert(
        "INSERT INTO archive (id, name) \
         SELECT id, name FROM users WHERE active = 0",
    );
    assert!(matches!(i.values, InsertSource::Query(_)));
    round_trip("INSERT INTO archive (id, name) SELECT id, name FROM users WHERE active = 0");
}

#[test]
fn insert_default_values() {
    let i = parse_insert("INSERT INTO counters DEFAULT VALUES");
    assert!(matches!(i.values, InsertSource::DefaultValues));
    round_trip("INSERT INTO counters DEFAULT VALUES");
}

#[test]
fn insert_schema_qualified() {
    let i = parse_insert("INSERT INTO public.users (name) VALUES ('Eve')");
    assert_eq!(i.schema.as_deref(), Some("public"));
    assert_eq!(i.table, "users");
    round_trip("INSERT INTO public.users (name) VALUES ('Eve')");
}

#[test]
fn insert_with_expressions() {
    let i = parse_insert("INSERT INTO stats (value) VALUES (1 + 2)");
    if let InsertSource::Values(rows) = &i.values {
        assert!(matches!(
            &rows[0][0],
            Expr::Binary {
                op: BinaryOp::Add,
                ..
            }
        ));
    } else {
        panic!("Expected VALUES");
    }
    round_trip("INSERT INTO stats (value) VALUES (1 + 2)");
}

#[test]
fn insert_with_parameters() {
    let i = parse_insert("INSERT INTO users (name, email) VALUES (?, ?)");
    if let InsertSource::Values(rows) = &i.values {
        assert!(matches!(
            &rows[0][0],
            Expr::Parameter {
                position: 1,
                name: None
            }
        ));
        assert!(matches!(
            &rows[0][1],
            Expr::Parameter {
                position: 2,
                name: None
            }
        ));
    } else {
        panic!("Expected VALUES");
    }
    round_trip("INSERT INTO users (name, email) VALUES (?, ?)");
}
