#![allow(dead_code)]

use oxide_sql_core::ast::{
    DeleteStatement, InsertStatement, SelectStatement, Statement, UpdateStatement,
};
use oxide_sql_core::{ParseError, Parser};

pub fn parse(sql: &str) -> Statement {
    Parser::new(sql)
        .parse_statement()
        .unwrap_or_else(|e| panic!("Failed to parse: {sql}\nError: {e:?}"))
}

pub fn parse_err(sql: &str) -> ParseError {
    Parser::new(sql)
        .parse_statement()
        .expect_err(&format!("Expected parse error for: {sql}"))
}

pub fn parse_select(sql: &str) -> SelectStatement {
    match parse(sql) {
        Statement::Select(s) => s,
        other => panic!("Expected SELECT, got {other:?}"),
    }
}

pub fn parse_insert(sql: &str) -> InsertStatement {
    match parse(sql) {
        Statement::Insert(i) => i,
        other => panic!("Expected INSERT, got {other:?}"),
    }
}

pub fn parse_update(sql: &str) -> UpdateStatement {
    match parse(sql) {
        Statement::Update(u) => u,
        other => panic!("Expected UPDATE, got {other:?}"),
    }
}

pub fn parse_delete(sql: &str) -> DeleteStatement {
    match parse(sql) {
        Statement::Delete(d) => d,
        other => panic!("Expected DELETE, got {other:?}"),
    }
}

/// Verifies that `to_string()` produces a fixed point:
/// parse(sql).to_string() can be re-parsed and yields the same
/// string again.
pub fn round_trip(sql: &str) {
    let ast1 = parse(sql);
    let rendered1 = ast1.to_string();
    let ast2 = parse(&rendered1);
    let rendered2 = ast2.to_string();
    assert_eq!(
        rendered1, rendered2,
        "Round-trip failed.\n  Input:    {sql}\n  First:    {rendered1}\n  Second:   {rendered2}"
    );
}
