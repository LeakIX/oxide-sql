//! Tests for SELECT clauses: WHERE, GROUP BY, HAVING, ORDER BY,
//! LIMIT, and OFFSET.

mod common;
use common::*;

use oxide_sql_core::ast::{BinaryOp, Expr, Literal, OrderDirection};

#[test]
fn where_simple() {
    let s = parse_select("SELECT * FROM users WHERE id = 1");
    assert!(matches!(
        &s.where_clause,
        Some(Expr::Binary {
            op: BinaryOp::Eq,
            ..
        })
    ));
    round_trip("SELECT * FROM users WHERE id = 1");
}

#[test]
fn where_compound_and_or() {
    let s = parse_select("SELECT * FROM users WHERE (age > 18 AND active = 1) OR admin = 1");
    assert!(matches!(
        &s.where_clause,
        Some(Expr::Binary {
            op: BinaryOp::Or,
            ..
        })
    ));
    round_trip("SELECT * FROM users WHERE (age > 18 AND active = 1) OR admin = 1");
}

#[test]
fn group_by_single() {
    let s = parse_select("SELECT status, COUNT(*) FROM orders GROUP BY status");
    assert_eq!(s.group_by.len(), 1);
    assert!(matches!(
        &s.group_by[0],
        Expr::Column { name, .. } if name == "status"
    ));
    round_trip("SELECT status, COUNT(*) FROM orders GROUP BY status");
}

#[test]
fn group_by_multiple() {
    let s = parse_select(
        "SELECT status, region, COUNT(*) \
         FROM orders GROUP BY status, region",
    );
    assert_eq!(s.group_by.len(), 2);
    round_trip("SELECT status, region, COUNT(*) FROM orders GROUP BY status, region");
}

#[test]
fn having_with_aggregate() {
    let s = parse_select(
        "SELECT status, COUNT(*) AS cnt \
         FROM orders GROUP BY status HAVING COUNT(*) > 5",
    );
    assert!(s.having.is_some());
    assert!(matches!(
        &s.having,
        Some(Expr::Binary {
            op: BinaryOp::Gt,
            ..
        })
    ));
    round_trip("SELECT status, COUNT(*) AS cnt FROM orders GROUP BY status HAVING COUNT(*) > 5");
}

#[test]
fn where_group_by_having_combined() {
    let s = parse_select(
        "SELECT department, AVG(salary) \
         FROM employees \
         WHERE active = 1 \
         GROUP BY department \
         HAVING AVG(salary) > 50000",
    );
    assert!(s.where_clause.is_some());
    assert_eq!(s.group_by.len(), 1);
    assert!(s.having.is_some());
    round_trip(
        "SELECT department, AVG(salary) FROM employees WHERE active = 1 GROUP BY department HAVING AVG(salary) > 50000",
    );
}

#[test]
fn order_by_default_asc() {
    let s = parse_select("SELECT * FROM users ORDER BY name");
    assert_eq!(s.order_by.len(), 1);
    assert_eq!(s.order_by[0].direction, OrderDirection::Asc);
    round_trip("SELECT * FROM users ORDER BY name");
}

#[test]
fn order_by_explicit_asc() {
    let s = parse_select("SELECT * FROM users ORDER BY name ASC");
    assert_eq!(s.order_by[0].direction, OrderDirection::Asc);
    round_trip("SELECT * FROM users ORDER BY name ASC");
}

#[test]
fn order_by_desc() {
    let s = parse_select("SELECT * FROM users ORDER BY created_at DESC");
    assert_eq!(s.order_by[0].direction, OrderDirection::Desc);
    round_trip("SELECT * FROM users ORDER BY created_at DESC");
}

#[test]
fn order_by_multiple_columns() {
    let s = parse_select("SELECT * FROM users ORDER BY last_name ASC, first_name DESC");
    assert_eq!(s.order_by.len(), 2);
    assert_eq!(s.order_by[0].direction, OrderDirection::Asc);
    assert_eq!(s.order_by[1].direction, OrderDirection::Desc);
    round_trip("SELECT * FROM users ORDER BY last_name ASC, first_name DESC");
}

#[test]
fn limit_only() {
    let s = parse_select("SELECT * FROM users LIMIT 10");
    assert!(matches!(
        &s.limit,
        Some(Expr::Literal(Literal::Integer(10)))
    ));
    assert!(s.offset.is_none());
    round_trip("SELECT * FROM users LIMIT 10");
}

#[test]
fn limit_and_offset() {
    let s = parse_select("SELECT * FROM users LIMIT 10 OFFSET 20");
    assert!(matches!(
        &s.limit,
        Some(Expr::Literal(Literal::Integer(10)))
    ));
    assert!(matches!(
        &s.offset,
        Some(Expr::Literal(Literal::Integer(20)))
    ));
    round_trip("SELECT * FROM users LIMIT 10 OFFSET 20");
}
