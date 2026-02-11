//! Tests for data type parsing via CAST expressions.

mod common;
use common::*;

use oxide_sql_core::ast::{DataType, Expr};

#[test]
fn datatype_int() {
    let s = parse_select("SELECT CAST(x AS INT) FROM t");
    if let Expr::Cast { data_type, .. } = &s.columns[0].expr {
        assert_eq!(*data_type, DataType::Integer);
    } else {
        panic!("Expected CAST");
    }
    round_trip("SELECT CAST(x AS INTEGER) FROM t");
}

#[test]
fn datatype_smallint() {
    let s = parse_select("SELECT CAST(x AS SMALLINT) FROM t");
    if let Expr::Cast { data_type, .. } = &s.columns[0].expr {
        assert_eq!(*data_type, DataType::Smallint);
    } else {
        panic!("Expected CAST");
    }
    round_trip("SELECT CAST(x AS SMALLINT) FROM t");
}

#[test]
fn datatype_bigint() {
    let s = parse_select("SELECT CAST(x AS BIGINT) FROM t");
    if let Expr::Cast { data_type, .. } = &s.columns[0].expr {
        assert_eq!(*data_type, DataType::Bigint);
    } else {
        panic!("Expected CAST");
    }
    round_trip("SELECT CAST(x AS BIGINT) FROM t");
}

#[test]
fn datatype_real() {
    let s = parse_select("SELECT CAST(x AS REAL) FROM t");
    if let Expr::Cast { data_type, .. } = &s.columns[0].expr {
        assert_eq!(*data_type, DataType::Real);
    } else {
        panic!("Expected CAST");
    }
    round_trip("SELECT CAST(x AS REAL) FROM t");
}

#[test]
fn datatype_double() {
    let s = parse_select("SELECT CAST(x AS DOUBLE) FROM t");
    if let Expr::Cast { data_type, .. } = &s.columns[0].expr {
        assert_eq!(*data_type, DataType::Double);
    } else {
        panic!("Expected CAST");
    }
    round_trip("SELECT CAST(x AS DOUBLE) FROM t");
}

#[test]
fn datatype_float_maps_to_double() {
    let s = parse_select("SELECT CAST(x AS FLOAT) FROM t");
    if let Expr::Cast { data_type, .. } = &s.columns[0].expr {
        assert_eq!(*data_type, DataType::Double);
    } else {
        panic!("Expected CAST");
    }
    round_trip("SELECT CAST(x AS DOUBLE) FROM t");
}

#[test]
fn datatype_numeric() {
    let s = parse_select("SELECT CAST(x AS NUMERIC(8, 3)) FROM t");
    if let Expr::Cast { data_type, .. } = &s.columns[0].expr {
        assert_eq!(
            *data_type,
            DataType::Numeric {
                precision: Some(8),
                scale: Some(3)
            }
        );
    } else {
        panic!("Expected CAST");
    }
    round_trip("SELECT CAST(x AS NUMERIC(8, 3)) FROM t");
}

#[test]
fn datatype_char() {
    let s = parse_select("SELECT CAST(x AS CHAR(10)) FROM t");
    if let Expr::Cast { data_type, .. } = &s.columns[0].expr {
        assert_eq!(*data_type, DataType::Char(Some(10)));
    } else {
        panic!("Expected CAST");
    }
    round_trip("SELECT CAST(x AS CHAR(10)) FROM t");
}

#[test]
fn datatype_varchar_no_length() {
    let s = parse_select("SELECT CAST(x AS VARCHAR) FROM t");
    if let Expr::Cast { data_type, .. } = &s.columns[0].expr {
        assert_eq!(*data_type, DataType::Varchar(None));
    } else {
        panic!("Expected CAST");
    }
    round_trip("SELECT CAST(x AS VARCHAR) FROM t");
}

#[test]
fn datatype_boolean() {
    let s = parse_select("SELECT CAST(x AS BOOLEAN) FROM t");
    if let Expr::Cast { data_type, .. } = &s.columns[0].expr {
        assert_eq!(*data_type, DataType::Boolean);
    } else {
        panic!("Expected CAST");
    }
    round_trip("SELECT CAST(x AS BOOLEAN) FROM t");
}

#[test]
fn datatype_timestamp() {
    let s = parse_select("SELECT CAST(x AS TIMESTAMP) FROM t");
    if let Expr::Cast { data_type, .. } = &s.columns[0].expr {
        assert_eq!(*data_type, DataType::Timestamp);
    } else {
        panic!("Expected CAST");
    }
    round_trip("SELECT CAST(x AS TIMESTAMP) FROM t");
}
