//! Tests for literal parsing: integers, floats, strings, blobs,
//! booleans, and NULL.

mod common;
use common::*;

use oxide_sql_core::ast::{Expr, Literal, UnaryOp};

#[test]
fn literal_integer() {
    let s = parse_select("SELECT 42");
    assert!(matches!(
        &s.columns[0].expr,
        Expr::Literal(Literal::Integer(42))
    ));
    round_trip("SELECT 42");
}

#[test]
fn literal_negative_integer() {
    let s = parse_select("SELECT -7");
    assert!(matches!(
        &s.columns[0].expr,
        Expr::Unary { op: UnaryOp::Neg, operand }
            if matches!(operand.as_ref(), Expr::Literal(Literal::Integer(7)))
    ));
    round_trip("SELECT -7");
}

#[test]
fn literal_float() {
    let s = parse_select("SELECT 9.75");
    if let Expr::Literal(Literal::Float(f)) = &s.columns[0].expr {
        assert!((*f - 9.75).abs() < f64::EPSILON);
    } else {
        panic!("Expected float literal");
    }
    round_trip("SELECT 9.75");
}

#[test]
fn literal_string() {
    let s = parse_select("SELECT 'hello world'");
    assert!(matches!(
        &s.columns[0].expr,
        Expr::Literal(Literal::String(v)) if v == "hello world"
    ));
    round_trip("SELECT 'hello world'");
}

#[test]
fn literal_blob() {
    let s = parse_select("SELECT X'DEADBEEF'");
    assert!(matches!(
        &s.columns[0].expr,
        Expr::Literal(Literal::Blob(_))
    ));
    round_trip("SELECT X'DEADBEEF'");
}

#[test]
fn literal_true() {
    let s = parse_select("SELECT TRUE");
    assert!(matches!(
        &s.columns[0].expr,
        Expr::Literal(Literal::Boolean(true))
    ));
    round_trip("SELECT TRUE");
}

#[test]
fn literal_false() {
    let s = parse_select("SELECT FALSE");
    assert!(matches!(
        &s.columns[0].expr,
        Expr::Literal(Literal::Boolean(false))
    ));
    round_trip("SELECT FALSE");
}

#[test]
fn literal_null() {
    let s = parse_select("SELECT NULL");
    assert!(matches!(&s.columns[0].expr, Expr::Literal(Literal::Null)));
    round_trip("SELECT NULL");
}
