//! Tests for parser error cases.

mod common;
use common::*;

#[test]
fn error_empty_input() {
    let _ = parse_err("");
}

#[test]
fn error_incomplete_select() {
    let _ = parse_err("SELECT");
}

#[test]
fn error_missing_from_table() {
    let _ = parse_err("SELECT * FROM");
}

#[test]
fn error_unexpected_keyword() {
    let _ = parse_err("TRUNCATE users");
}

#[test]
fn error_unclosed_paren() {
    let _ = parse_err("SELECT (1 + 2");
}

#[test]
fn error_join_without_on_or_using() {
    let _ = parse_err("SELECT * FROM a INNER JOIN b WHERE a.id = 1");
}
