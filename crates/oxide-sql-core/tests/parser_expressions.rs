//! Tests for special expression forms: IS NULL, BETWEEN, IN, CASE,
//! and CAST.

mod common;
use common::*;

use oxide_sql_core::ast::{BinaryOp, DataType, Expr, Literal};

// ===================================================================
// IS NULL / IS NOT NULL
// ===================================================================

#[test]
fn is_null() {
    let s = parse_select("SELECT * FROM t WHERE x IS NULL");
    assert!(matches!(
        &s.where_clause,
        Some(Expr::IsNull { negated: false, .. })
    ));
    round_trip("SELECT * FROM t WHERE x IS NULL");
}

#[test]
fn is_not_null() {
    let s = parse_select("SELECT * FROM t WHERE x IS NOT NULL");
    assert!(matches!(
        &s.where_clause,
        Some(Expr::IsNull { negated: true, .. })
    ));
    round_trip("SELECT * FROM t WHERE x IS NOT NULL");
}

// ===================================================================
// BETWEEN
// ===================================================================

#[test]
fn between_simple() {
    let s = parse_select("SELECT * FROM t WHERE x BETWEEN 1 AND 10");
    if let Some(Expr::Between {
        negated, low, high, ..
    }) = &s.where_clause
    {
        assert!(!negated);
        assert!(matches!(low.as_ref(), Expr::Literal(Literal::Integer(1))));
        assert!(matches!(high.as_ref(), Expr::Literal(Literal::Integer(10))));
    } else {
        panic!("Expected BETWEEN");
    }
    round_trip("SELECT * FROM t WHERE x BETWEEN 1 AND 10");
}

#[test]
fn between_with_expressions() {
    let s = parse_select("SELECT * FROM t WHERE x BETWEEN 1 + 1 AND 5 * 2");
    if let Some(Expr::Between { low, high, .. }) = &s.where_clause {
        assert!(matches!(
            low.as_ref(),
            Expr::Binary {
                op: BinaryOp::Add,
                ..
            }
        ));
        assert!(matches!(
            high.as_ref(),
            Expr::Binary {
                op: BinaryOp::Mul,
                ..
            }
        ));
    } else {
        panic!("Expected BETWEEN");
    }
    round_trip("SELECT * FROM t WHERE x BETWEEN 1 + 1 AND 5 * 2");
}

// ===================================================================
// IN
// ===================================================================

#[test]
fn in_integers() {
    let s = parse_select("SELECT * FROM t WHERE id IN (1, 2, 3)");
    if let Some(Expr::In { list, negated, .. }) = &s.where_clause {
        assert!(!negated);
        assert_eq!(list.len(), 3);
    } else {
        panic!("Expected IN");
    }
    round_trip("SELECT * FROM t WHERE id IN (1, 2, 3)");
}

#[test]
fn in_strings() {
    let s = parse_select("SELECT * FROM t WHERE name IN ('a', 'b')");
    if let Some(Expr::In { list, .. }) = &s.where_clause {
        assert_eq!(list.len(), 2);
        assert!(matches!(
            &list[0],
            Expr::Literal(Literal::String(v)) if v == "a"
        ));
    } else {
        panic!("Expected IN");
    }
    round_trip("SELECT * FROM t WHERE name IN ('a', 'b')");
}

// ===================================================================
// CASE expressions
// ===================================================================

#[test]
fn case_searched() {
    let s = parse_select(
        "SELECT CASE \
            WHEN x = 1 THEN 'one' \
            WHEN x = 2 THEN 'two' \
            ELSE 'other' \
         END FROM t",
    );
    if let Expr::Case {
        operand,
        when_clauses,
        else_clause,
    } = &s.columns[0].expr
    {
        assert!(operand.is_none());
        assert_eq!(when_clauses.len(), 2);
        assert!(else_clause.is_some());
    } else {
        panic!("Expected CASE");
    }
    round_trip("SELECT CASE WHEN x = 1 THEN 'one' WHEN x = 2 THEN 'two' ELSE 'other' END FROM t");
}

#[test]
fn case_searched_without_else() {
    let s = parse_select("SELECT CASE WHEN x > 0 THEN 'pos' END FROM t");
    if let Expr::Case { else_clause, .. } = &s.columns[0].expr {
        assert!(else_clause.is_none());
    } else {
        panic!("Expected CASE");
    }
    round_trip("SELECT CASE WHEN x > 0 THEN 'pos' END FROM t");
}

#[test]
fn case_simple() {
    let s = parse_select(
        "SELECT CASE status \
            WHEN 1 THEN 'active' \
            WHEN 0 THEN 'inactive' \
         END FROM t",
    );
    if let Expr::Case {
        operand,
        when_clauses,
        ..
    } = &s.columns[0].expr
    {
        assert!(operand.is_some());
        assert_eq!(when_clauses.len(), 2);
    } else {
        panic!("Expected CASE");
    }
    round_trip("SELECT CASE status WHEN 1 THEN 'active' WHEN 0 THEN 'inactive' END FROM t");
}

#[test]
fn case_in_where() {
    let s = parse_select(
        "SELECT * FROM t \
         WHERE CASE WHEN x > 0 THEN 1 ELSE 0 END = 1",
    );
    assert!(matches!(
        &s.where_clause,
        Some(Expr::Binary {
            op: BinaryOp::Eq,
            ..
        })
    ));
    round_trip("SELECT * FROM t WHERE CASE WHEN x > 0 THEN 1 ELSE 0 END = 1");
}

// ===================================================================
// CAST
// ===================================================================

#[test]
fn cast_to_integer() {
    let s = parse_select("SELECT CAST(x AS INTEGER) FROM t");
    if let Expr::Cast { data_type, .. } = &s.columns[0].expr {
        assert_eq!(*data_type, DataType::Integer);
    } else {
        panic!("Expected CAST");
    }
    round_trip("SELECT CAST(x AS INTEGER) FROM t");
}

#[test]
fn cast_to_varchar_n() {
    let s = parse_select("SELECT CAST(x AS VARCHAR(255)) FROM t");
    if let Expr::Cast { data_type, .. } = &s.columns[0].expr {
        assert_eq!(*data_type, DataType::Varchar(Some(255)));
    } else {
        panic!("Expected CAST");
    }
    round_trip("SELECT CAST(x AS VARCHAR(255)) FROM t");
}

#[test]
fn cast_to_decimal_precision_scale() {
    let s = parse_select("SELECT CAST(x AS DECIMAL(10, 2)) FROM t");
    if let Expr::Cast { data_type, .. } = &s.columns[0].expr {
        assert_eq!(
            *data_type,
            DataType::Decimal {
                precision: Some(10),
                scale: Some(2)
            }
        );
    } else {
        panic!("Expected CAST");
    }
    round_trip("SELECT CAST(x AS DECIMAL(10, 2)) FROM t");
}

#[test]
fn cast_to_text() {
    let s = parse_select("SELECT CAST(42 AS TEXT) FROM t");
    if let Expr::Cast { data_type, .. } = &s.columns[0].expr {
        assert_eq!(*data_type, DataType::Text);
    } else {
        panic!("Expected CAST");
    }
    round_trip("SELECT CAST(42 AS TEXT) FROM t");
}
