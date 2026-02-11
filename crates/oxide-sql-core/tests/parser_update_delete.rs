//! Tests for UPDATE and DELETE statements.

mod common;
use common::*;

use oxide_sql_core::ast::{BinaryOp, Expr};

// ===================================================================
// UPDATE
// ===================================================================

#[test]
fn update_single_set() {
    let u = parse_update("UPDATE users SET name = 'Bob' WHERE id = 1");
    assert_eq!(u.table, "users");
    assert_eq!(u.assignments.len(), 1);
    assert_eq!(u.assignments[0].column, "name");
    assert!(u.where_clause.is_some());
    round_trip("UPDATE users SET name = 'Bob' WHERE id = 1");
}

#[test]
fn update_multiple_set() {
    let u = parse_update("UPDATE users SET name = 'Bob', email = 'bob@x.com' WHERE id = 1");
    assert_eq!(u.assignments.len(), 2);
    assert_eq!(u.assignments[0].column, "name");
    assert_eq!(u.assignments[1].column, "email");
    round_trip("UPDATE users SET name = 'Bob', email = 'bob@x.com' WHERE id = 1");
}

#[test]
fn update_schema_qualified() {
    let u = parse_update("UPDATE public.users SET name = 'X' WHERE id = 1");
    assert_eq!(u.schema.as_deref(), Some("public"));
    assert_eq!(u.table, "users");
    round_trip("UPDATE public.users SET name = 'X' WHERE id = 1");
}

#[test]
fn update_with_alias() {
    let u = parse_update("UPDATE users u SET name = 'X' WHERE u.id = 1");
    assert_eq!(u.alias.as_deref(), Some("u"));
    round_trip("UPDATE users AS u SET name = 'X' WHERE u.id = 1");
}

#[test]
fn update_with_from_clause() {
    let u = parse_update(
        "UPDATE orders SET total = p.price \
         FROM products p \
         WHERE orders.product_id = p.id",
    );
    assert!(u.from.is_some());
    assert!(u.where_clause.is_some());
    round_trip(
        "UPDATE orders SET total = p.price FROM products AS p WHERE orders.product_id = p.id",
    );
}

#[test]
fn update_without_where() {
    let u = parse_update("UPDATE users SET active = 0");
    assert!(u.where_clause.is_none());
    round_trip("UPDATE users SET active = 0");
}

#[test]
fn update_with_parameters() {
    let u = parse_update("UPDATE users SET name = ?, email = :email WHERE id = ?");
    assert!(matches!(
        &u.assignments[0].value,
        Expr::Parameter {
            position: 1,
            name: None
        }
    ));
    assert!(matches!(
        &u.assignments[1].value,
        Expr::Parameter { name: Some(n), .. } if n == "email"
    ));
    round_trip("UPDATE users SET name = ?, email = :email WHERE id = ?");
}

// ===================================================================
// DELETE
// ===================================================================

#[test]
fn delete_with_where() {
    let d = parse_delete("DELETE FROM users WHERE id = 1");
    assert_eq!(d.table, "users");
    assert!(d.where_clause.is_some());
    round_trip("DELETE FROM users WHERE id = 1");
}

#[test]
fn delete_without_where() {
    let d = parse_delete("DELETE FROM users");
    assert!(d.where_clause.is_none());
    round_trip("DELETE FROM users");
}

#[test]
fn delete_schema_qualified() {
    let d = parse_delete("DELETE FROM public.users WHERE id = 1");
    assert_eq!(d.schema.as_deref(), Some("public"));
    assert_eq!(d.table, "users");
    round_trip("DELETE FROM public.users WHERE id = 1");
}

#[test]
fn delete_with_alias() {
    let d = parse_delete("DELETE FROM users u WHERE u.active = 0");
    assert_eq!(d.alias.as_deref(), Some("u"));
    round_trip("DELETE FROM users AS u WHERE u.active = 0");
}

#[test]
fn delete_complex_where() {
    let d = parse_delete(
        "DELETE FROM logs \
         WHERE created_at < 1000 AND level = 'debug'",
    );
    assert!(matches!(
        &d.where_clause,
        Some(Expr::Binary {
            op: BinaryOp::And,
            ..
        })
    ));
    round_trip("DELETE FROM logs WHERE created_at < 1000 AND level = 'debug'");
}
