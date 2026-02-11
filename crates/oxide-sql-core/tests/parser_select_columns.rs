//! Tests for SELECT column selection: wildcards, named columns,
//! qualified columns, aliases, DISTINCT, ALL, and no-FROM selects.

mod common;
use common::*;

use oxide_sql_core::ast::{BinaryOp, Expr};

#[test]
fn select_star() {
    let s = parse_select("SELECT * FROM users");
    assert_eq!(s.columns.len(), 1);
    assert!(matches!(s.columns[0].expr, Expr::Wildcard { table: None }));
    round_trip("SELECT * FROM users");
}

#[test]
fn select_qualified_star() {
    let s = parse_select("SELECT u.* FROM users u");
    assert_eq!(s.columns.len(), 1);
    assert!(matches!(
        &s.columns[0].expr,
        Expr::Wildcard { table: Some(t) } if t == "u"
    ));
    round_trip("SELECT u.* FROM users AS u");
}

#[test]
fn select_named_columns() {
    let s = parse_select("SELECT id, name, email FROM users");
    assert_eq!(s.columns.len(), 3);
    assert!(matches!(
        &s.columns[0].expr,
        Expr::Column { name, table: None, .. } if name == "id"
    ));
    assert!(matches!(
        &s.columns[2].expr,
        Expr::Column { name, table: None, .. } if name == "email"
    ));
    round_trip("SELECT id, name, email FROM users");
}

#[test]
fn select_qualified_columns() {
    let s = parse_select("SELECT u.id, u.name FROM users u");
    assert_eq!(s.columns.len(), 2);
    assert!(matches!(
        &s.columns[0].expr,
        Expr::Column { table: Some(t), name, .. }
            if t == "u" && name == "id"
    ));
    round_trip("SELECT u.id, u.name FROM users AS u");
}

#[test]
fn select_alias_with_as() {
    let s = parse_select("SELECT id AS user_id FROM users");
    assert_eq!(s.columns[0].alias.as_deref(), Some("user_id"));
    round_trip("SELECT id AS user_id FROM users");
}

#[test]
fn select_bare_alias() {
    let s = parse_select("SELECT id uid FROM users");
    assert_eq!(s.columns[0].alias.as_deref(), Some("uid"));
    round_trip("SELECT id AS uid FROM users");
}

#[test]
fn select_expression_alias() {
    let s = parse_select("SELECT 1 + 2 AS total");
    assert_eq!(s.columns[0].alias.as_deref(), Some("total"));
    assert!(matches!(
        &s.columns[0].expr,
        Expr::Binary {
            op: BinaryOp::Add,
            ..
        }
    ));
    round_trip("SELECT 1 + 2 AS total");
}

#[test]
fn select_distinct() {
    let s = parse_select("SELECT DISTINCT status FROM orders");
    assert!(s.distinct);
    assert_eq!(s.columns.len(), 1);
    round_trip("SELECT DISTINCT status FROM orders");
}

#[test]
fn select_all() {
    let s = parse_select("SELECT ALL status FROM orders");
    assert!(!s.distinct);
    round_trip("SELECT status FROM orders");
}

#[test]
fn select_without_from() {
    let s = parse_select("SELECT 1 + 1");
    assert!(s.from.is_none());
    assert!(matches!(
        &s.columns[0].expr,
        Expr::Binary {
            op: BinaryOp::Add,
            ..
        }
    ));
    round_trip("SELECT 1 + 1");
}
