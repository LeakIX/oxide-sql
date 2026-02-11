//! Tests for function calls (aggregates and custom), subqueries,
//! EXISTS, and parameters.

mod common;
use common::*;

use oxide_sql_core::ast::{Expr, FunctionCall};

// ===================================================================
// Aggregate functions
// ===================================================================

#[test]
fn function_count_star() {
    let s = parse_select("SELECT COUNT(*) FROM t");
    if let Expr::Function(FunctionCall {
        name,
        args,
        distinct,
    }) = &s.columns[0].expr
    {
        assert_eq!(name, "COUNT");
        assert!(!distinct);
        assert_eq!(args.len(), 1);
        assert!(matches!(args[0], Expr::Wildcard { table: None }));
    } else {
        panic!("Expected COUNT(*)");
    }
    round_trip("SELECT COUNT(*) FROM t");
}

#[test]
fn function_count_column() {
    let s = parse_select("SELECT COUNT(id) FROM t");
    if let Expr::Function(FunctionCall { name, args, .. }) = &s.columns[0].expr {
        assert_eq!(name, "COUNT");
        assert_eq!(args.len(), 1);
        assert!(matches!(
            &args[0],
            Expr::Column { name, .. } if name == "id"
        ));
    } else {
        panic!("Expected COUNT(id)");
    }
    round_trip("SELECT COUNT(id) FROM t");
}

#[test]
fn function_count_distinct() {
    let s = parse_select("SELECT COUNT(DISTINCT status) FROM t");
    if let Expr::Function(FunctionCall { name, distinct, .. }) = &s.columns[0].expr {
        assert_eq!(name, "COUNT");
        assert!(distinct);
    } else {
        panic!("Expected COUNT(DISTINCT ...)");
    }
    round_trip("SELECT COUNT(DISTINCT status) FROM t");
}

#[test]
fn function_sum() {
    let s = parse_select("SELECT SUM(amount) FROM orders");
    assert!(matches!(
        &s.columns[0].expr,
        Expr::Function(FunctionCall { name, .. }) if name == "SUM"
    ));
    round_trip("SELECT SUM(amount) FROM orders");
}

#[test]
fn function_avg() {
    let s = parse_select("SELECT AVG(price) FROM products");
    assert!(matches!(
        &s.columns[0].expr,
        Expr::Function(FunctionCall { name, .. }) if name == "AVG"
    ));
    round_trip("SELECT AVG(price) FROM products");
}

#[test]
fn function_min() {
    let s = parse_select("SELECT MIN(created_at) FROM events");
    assert!(matches!(
        &s.columns[0].expr,
        Expr::Function(FunctionCall { name, .. }) if name == "MIN"
    ));
    round_trip("SELECT MIN(created_at) FROM events");
}

#[test]
fn function_max() {
    let s = parse_select("SELECT MAX(score) FROM results");
    assert!(matches!(
        &s.columns[0].expr,
        Expr::Function(FunctionCall { name, .. }) if name == "MAX"
    ));
    round_trip("SELECT MAX(score) FROM results");
}

#[test]
fn function_coalesce() {
    let s = parse_select("SELECT COALESCE(a, b, 0) FROM t");
    if let Expr::Function(FunctionCall { name, args, .. }) = &s.columns[0].expr {
        assert_eq!(name, "COALESCE");
        assert_eq!(args.len(), 3);
    } else {
        panic!("Expected COALESCE");
    }
    round_trip("SELECT COALESCE(a, b, 0) FROM t");
}

#[test]
fn function_nullif() {
    let s = parse_select("SELECT NULLIF(x, 0) FROM t");
    if let Expr::Function(FunctionCall { name, args, .. }) = &s.columns[0].expr {
        assert_eq!(name, "NULLIF");
        assert_eq!(args.len(), 2);
    } else {
        panic!("Expected NULLIF");
    }
    round_trip("SELECT NULLIF(x, 0) FROM t");
}

// ===================================================================
// Custom functions
// ===================================================================

#[test]
fn custom_function_no_args() {
    let s = parse_select("SELECT now()");
    if let Expr::Function(FunctionCall { name, args, .. }) = &s.columns[0].expr {
        assert_eq!(name, "now");
        assert!(args.is_empty());
    } else {
        panic!("Expected now()");
    }
    round_trip("SELECT now()");
}

#[test]
fn custom_function_multi_args() {
    let s = parse_select("SELECT substr(name, 1, 3) FROM t");
    if let Expr::Function(FunctionCall { name, args, .. }) = &s.columns[0].expr {
        assert_eq!(name, "substr");
        assert_eq!(args.len(), 3);
    } else {
        panic!("Expected substr()");
    }
    round_trip("SELECT substr(name, 1, 3) FROM t");
}

// ===================================================================
// Subqueries & EXISTS
// ===================================================================

#[test]
fn exists_in_where() {
    let s = parse_select(
        "SELECT * FROM users u \
         WHERE EXISTS (SELECT 1 FROM orders o WHERE o.user_id = u.id)",
    );
    assert!(matches!(
        &s.where_clause,
        Some(Expr::Function(FunctionCall { name, .. })) if name == "EXISTS"
    ));
    round_trip(
        "SELECT * FROM users AS u \
         WHERE EXISTS(SELECT 1 FROM orders AS o WHERE o.user_id = u.id)",
    );
}

#[test]
fn scalar_subquery_in_select() {
    let s = parse_select("SELECT (SELECT COUNT(*) FROM orders) AS total");
    assert!(matches!(&s.columns[0].expr, Expr::Subquery(_)));
    assert_eq!(s.columns[0].alias.as_deref(), Some("total"));
    round_trip("SELECT (SELECT COUNT(*) FROM orders) AS total");
}

#[test]
fn subquery_in_where() {
    let s = parse_select(
        "SELECT * FROM users \
         WHERE id = (SELECT MAX(user_id) FROM orders)",
    );
    if let Some(Expr::Binary { right, .. }) = &s.where_clause {
        assert!(matches!(right.as_ref(), Expr::Subquery(_)));
    } else {
        panic!("Expected binary with subquery");
    }
    round_trip("SELECT * FROM users WHERE id = (SELECT MAX(user_id) FROM orders)");
}

// ===================================================================
// Parameters
// ===================================================================

#[test]
fn param_positional() {
    let s = parse_select("SELECT * FROM t WHERE id = ?");
    if let Some(Expr::Binary { right, .. }) = &s.where_clause {
        assert!(matches!(
            right.as_ref(),
            Expr::Parameter {
                name: None,
                position: 1
            }
        ));
    } else {
        panic!("Expected parameter");
    }
    round_trip("SELECT * FROM t WHERE id = ?");
}

#[test]
fn param_multiple_positional() {
    let s = parse_select("SELECT * FROM t WHERE a = ? AND b = ?");
    if let Some(Expr::Binary { left, right, .. }) = &s.where_clause {
        if let Expr::Binary { right: p1, .. } = left.as_ref() {
            assert!(matches!(
                p1.as_ref(),
                Expr::Parameter {
                    position: 1,
                    name: None
                }
            ));
        }
        if let Expr::Binary { right: p2, .. } = right.as_ref() {
            assert!(matches!(
                p2.as_ref(),
                Expr::Parameter {
                    position: 2,
                    name: None
                }
            ));
        }
    } else {
        panic!("Expected AND");
    }
    round_trip("SELECT * FROM t WHERE a = ? AND b = ?");
}

#[test]
fn param_named() {
    let s = parse_select("SELECT * FROM t WHERE name = :user_name");
    if let Some(Expr::Binary { right, .. }) = &s.where_clause {
        assert!(matches!(
            right.as_ref(),
            Expr::Parameter { name: Some(n), position: 0 } if n == "user_name"
        ));
    } else {
        panic!("Expected named parameter");
    }
    round_trip("SELECT * FROM t WHERE name = :user_name");
}

#[test]
fn param_mixed() {
    let s = parse_select("SELECT * FROM t WHERE a = ? AND b = :name AND c = ?");
    assert!(s.where_clause.is_some());
    round_trip("SELECT * FROM t WHERE a = ? AND b = :name AND c = ?");
}
