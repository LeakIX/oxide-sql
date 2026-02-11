//! Tests for binary operators (arithmetic, comparison, logical,
//! bitwise), unary operators, and operator precedence.

mod common;
use common::*;

use oxide_sql_core::ast::{BinaryOp, Expr, Literal, UnaryOp};

// ===================================================================
// Arithmetic operators
// ===================================================================

#[test]
fn binop_add() {
    let s = parse_select("SELECT 1 + 2");
    assert!(matches!(
        &s.columns[0].expr,
        Expr::Binary {
            op: BinaryOp::Add,
            ..
        }
    ));
    round_trip("SELECT 1 + 2");
}

#[test]
fn binop_sub() {
    let s = parse_select("SELECT 5 - 3");
    assert!(matches!(
        &s.columns[0].expr,
        Expr::Binary {
            op: BinaryOp::Sub,
            ..
        }
    ));
    round_trip("SELECT 5 - 3");
}

#[test]
fn binop_mul() {
    let s = parse_select("SELECT 4 * 2");
    assert!(matches!(
        &s.columns[0].expr,
        Expr::Binary {
            op: BinaryOp::Mul,
            ..
        }
    ));
    round_trip("SELECT 4 * 2");
}

#[test]
fn binop_div() {
    let s = parse_select("SELECT 8 / 2");
    assert!(matches!(
        &s.columns[0].expr,
        Expr::Binary {
            op: BinaryOp::Div,
            ..
        }
    ));
    round_trip("SELECT 8 / 2");
}

#[test]
fn binop_mod() {
    let s = parse_select("SELECT 7 % 3");
    assert!(matches!(
        &s.columns[0].expr,
        Expr::Binary {
            op: BinaryOp::Mod,
            ..
        }
    ));
    round_trip("SELECT 7 % 3");
}

#[test]
fn binop_concat() {
    let s = parse_select("SELECT 'a' || 'b'");
    assert!(matches!(
        &s.columns[0].expr,
        Expr::Binary {
            op: BinaryOp::Concat,
            ..
        }
    ));
    round_trip("SELECT 'a' || 'b'");
}

// ===================================================================
// Comparison operators
// ===================================================================

#[test]
fn binop_eq() {
    let s = parse_select("SELECT * FROM t WHERE x = 1");
    assert!(matches!(
        &s.where_clause,
        Some(Expr::Binary {
            op: BinaryOp::Eq,
            ..
        })
    ));
    round_trip("SELECT * FROM t WHERE x = 1");
}

#[test]
fn binop_not_eq() {
    let s = parse_select("SELECT * FROM t WHERE x != 1");
    assert!(matches!(
        &s.where_clause,
        Some(Expr::Binary {
            op: BinaryOp::NotEq,
            ..
        })
    ));
    round_trip("SELECT * FROM t WHERE x != 1");
}

#[test]
fn binop_lt() {
    let s = parse_select("SELECT * FROM t WHERE x < 1");
    assert!(matches!(
        &s.where_clause,
        Some(Expr::Binary {
            op: BinaryOp::Lt,
            ..
        })
    ));
    round_trip("SELECT * FROM t WHERE x < 1");
}

#[test]
fn binop_lt_eq() {
    let s = parse_select("SELECT * FROM t WHERE x <= 1");
    assert!(matches!(
        &s.where_clause,
        Some(Expr::Binary {
            op: BinaryOp::LtEq,
            ..
        })
    ));
    round_trip("SELECT * FROM t WHERE x <= 1");
}

#[test]
fn binop_gt() {
    let s = parse_select("SELECT * FROM t WHERE x > 1");
    assert!(matches!(
        &s.where_clause,
        Some(Expr::Binary {
            op: BinaryOp::Gt,
            ..
        })
    ));
    round_trip("SELECT * FROM t WHERE x > 1");
}

#[test]
fn binop_gt_eq() {
    let s = parse_select("SELECT * FROM t WHERE x >= 1");
    assert!(matches!(
        &s.where_clause,
        Some(Expr::Binary {
            op: BinaryOp::GtEq,
            ..
        })
    ));
    round_trip("SELECT * FROM t WHERE x >= 1");
}

// ===================================================================
// Logical & LIKE operators
// ===================================================================

#[test]
fn binop_and() {
    let s = parse_select("SELECT * FROM t WHERE a = 1 AND b = 2");
    assert!(matches!(
        &s.where_clause,
        Some(Expr::Binary {
            op: BinaryOp::And,
            ..
        })
    ));
    round_trip("SELECT * FROM t WHERE a = 1 AND b = 2");
}

#[test]
fn binop_or() {
    let s = parse_select("SELECT * FROM t WHERE a = 1 OR b = 2");
    assert!(matches!(
        &s.where_clause,
        Some(Expr::Binary {
            op: BinaryOp::Or,
            ..
        })
    ));
    round_trip("SELECT * FROM t WHERE a = 1 OR b = 2");
}

#[test]
fn binop_like() {
    let s = parse_select("SELECT * FROM t WHERE name LIKE '%test%'");
    assert!(matches!(
        &s.where_clause,
        Some(Expr::Binary {
            op: BinaryOp::Like,
            ..
        })
    ));
    round_trip("SELECT * FROM t WHERE name LIKE '%test%'");
}

// ===================================================================
// Bitwise operators
// ===================================================================

#[test]
fn binop_bit_and() {
    let s = parse_select("SELECT 5 & 3");
    assert!(matches!(
        &s.columns[0].expr,
        Expr::Binary {
            op: BinaryOp::BitAnd,
            ..
        }
    ));
    round_trip("SELECT 5 & 3");
}

#[test]
fn binop_bit_or() {
    let s = parse_select("SELECT 5 | 3");
    assert!(matches!(
        &s.columns[0].expr,
        Expr::Binary {
            op: BinaryOp::BitOr,
            ..
        }
    ));
    round_trip("SELECT 5 | 3");
}

#[test]
fn binop_left_shift() {
    let s = parse_select("SELECT 1 << 4");
    assert!(matches!(
        &s.columns[0].expr,
        Expr::Binary {
            op: BinaryOp::LeftShift,
            ..
        }
    ));
    round_trip("SELECT 1 << 4");
}

#[test]
fn binop_right_shift() {
    let s = parse_select("SELECT 16 >> 2");
    assert!(matches!(
        &s.columns[0].expr,
        Expr::Binary {
            op: BinaryOp::RightShift,
            ..
        }
    ));
    round_trip("SELECT 16 >> 2");
}

// ===================================================================
// Unary operators
// ===================================================================

#[test]
fn unary_neg() {
    let s = parse_select("SELECT -x FROM t");
    assert!(matches!(
        &s.columns[0].expr,
        Expr::Unary {
            op: UnaryOp::Neg,
            ..
        }
    ));
    round_trip("SELECT -x FROM t");
}

#[test]
fn unary_not() {
    let s = parse_select("SELECT * FROM t WHERE NOT active");
    assert!(matches!(
        &s.where_clause,
        Some(Expr::Unary {
            op: UnaryOp::Not,
            ..
        })
    ));
    round_trip("SELECT * FROM t WHERE NOT active");
}

#[test]
fn unary_bit_not() {
    let s = parse_select("SELECT ~flags FROM t");
    assert!(matches!(
        &s.columns[0].expr,
        Expr::Unary {
            op: UnaryOp::BitNot,
            ..
        }
    ));
    round_trip("SELECT ~flags FROM t");
}

// ===================================================================
// Operator precedence
// ===================================================================

#[test]
fn precedence_mul_over_add() {
    let s = parse_select("SELECT 1 + 2 * 3");
    if let Expr::Binary { op, left, right } = &s.columns[0].expr {
        assert_eq!(*op, BinaryOp::Add);
        assert!(matches!(left.as_ref(), Expr::Literal(Literal::Integer(1))));
        assert!(matches!(
            right.as_ref(),
            Expr::Binary {
                op: BinaryOp::Mul,
                ..
            }
        ));
    } else {
        panic!("Expected binary");
    }
    round_trip("SELECT 1 + 2 * 3");
}

#[test]
fn precedence_left_associativity() {
    let s = parse_select("SELECT 1 - 2 - 3");
    if let Expr::Binary { op, left, .. } = &s.columns[0].expr {
        assert_eq!(*op, BinaryOp::Sub);
        assert!(matches!(
            left.as_ref(),
            Expr::Binary {
                op: BinaryOp::Sub,
                ..
            }
        ));
    } else {
        panic!("Expected binary");
    }
    round_trip("SELECT 1 - 2 - 3");
}

#[test]
fn precedence_comparison_over_and() {
    let s = parse_select("SELECT * FROM t WHERE a = 1 AND b = 2");
    if let Some(Expr::Binary { op, left, right }) = &s.where_clause {
        assert_eq!(*op, BinaryOp::And);
        assert!(matches!(
            left.as_ref(),
            Expr::Binary {
                op: BinaryOp::Eq,
                ..
            }
        ));
        assert!(matches!(
            right.as_ref(),
            Expr::Binary {
                op: BinaryOp::Eq,
                ..
            }
        ));
    } else {
        panic!("Expected AND");
    }
    round_trip("SELECT * FROM t WHERE a = 1 AND b = 2");
}

#[test]
fn precedence_and_over_or() {
    let s = parse_select("SELECT * FROM t WHERE a = 1 OR b = 2 AND c = 3");
    if let Some(Expr::Binary { op, right, .. }) = &s.where_clause {
        assert_eq!(*op, BinaryOp::Or);
        assert!(matches!(
            right.as_ref(),
            Expr::Binary {
                op: BinaryOp::And,
                ..
            }
        ));
    } else {
        panic!("Expected OR");
    }
    round_trip("SELECT * FROM t WHERE a = 1 OR b = 2 AND c = 3");
}

#[test]
fn precedence_parens_override() {
    let s = parse_select("SELECT (1 + 2) * 3");
    if let Expr::Binary { op, left, .. } = &s.columns[0].expr {
        assert_eq!(*op, BinaryOp::Mul);
        assert!(matches!(left.as_ref(), Expr::Paren(_)));
    } else {
        panic!("Expected binary");
    }
    round_trip("SELECT (1 + 2) * 3");
}

#[test]
fn precedence_nested_parens() {
    let s = parse_select("SELECT ((1 + 2)) * 3");
    if let Expr::Binary { op, left, .. } = &s.columns[0].expr {
        assert_eq!(*op, BinaryOp::Mul);
        if let Expr::Paren(inner) = left.as_ref() {
            assert!(matches!(inner.as_ref(), Expr::Paren(_)));
        } else {
            panic!("Expected nested parens");
        }
    } else {
        panic!("Expected binary");
    }
    round_trip("SELECT ((1 + 2)) * 3");
}

#[test]
fn precedence_unary_neg_high_binding() {
    let s = parse_select("SELECT -x * y FROM t");
    if let Expr::Unary { op, operand } = &s.columns[0].expr {
        assert_eq!(*op, UnaryOp::Neg);
        assert!(matches!(
            operand.as_ref(),
            Expr::Binary {
                op: BinaryOp::Mul,
                ..
            }
        ));
    } else {
        panic!("Expected unary neg");
    }
    round_trip("SELECT -x * y FROM t");
}
