//! Tests for SELECT FROM clause: table references, aliases,
//! schema-qualified tables, subqueries, and all JOIN types.

mod common;
use common::*;

use oxide_sql_core::ast::{JoinType, TableRef};

#[test]
fn from_simple_table() {
    let s = parse_select("SELECT * FROM users");
    assert!(matches!(
        &s.from,
        Some(TableRef::Table { name, schema: None, alias: None })
            if name == "users"
    ));
    round_trip("SELECT * FROM users");
}

#[test]
fn from_table_with_as_alias() {
    let s = parse_select("SELECT * FROM users AS u");
    assert!(matches!(
        &s.from,
        Some(TableRef::Table { name, alias: Some(a), .. })
            if name == "users" && a == "u"
    ));
    round_trip("SELECT * FROM users AS u");
}

#[test]
fn from_table_with_bare_alias() {
    let s = parse_select("SELECT * FROM users u");
    assert!(matches!(
        &s.from,
        Some(TableRef::Table { name, alias: Some(a), .. })
            if name == "users" && a == "u"
    ));
    round_trip("SELECT * FROM users AS u");
}

#[test]
fn from_schema_qualified_table() {
    let s = parse_select("SELECT * FROM public.users");
    assert!(matches!(
        &s.from,
        Some(TableRef::Table { schema: Some(sc), name, .. })
            if sc == "public" && name == "users"
    ));
    round_trip("SELECT * FROM public.users");
}

#[test]
fn from_subquery_with_alias() {
    let s = parse_select("SELECT t.id FROM (SELECT id FROM users) AS t");
    assert!(matches!(&s.from, Some(TableRef::Subquery { alias, .. }) if alias == "t"));
    round_trip("SELECT t.id FROM (SELECT id FROM users) AS t");
}

#[test]
fn from_subquery_with_where() {
    let s = parse_select("SELECT t.id FROM (SELECT id FROM users WHERE active = 1) AS t");
    if let Some(TableRef::Subquery { query, alias }) = &s.from {
        assert_eq!(alias, "t");
        assert!(query.where_clause.is_some());
    } else {
        panic!("Expected subquery");
    }
    round_trip("SELECT t.id FROM (SELECT id FROM users WHERE active = 1) AS t");
}

#[test]
fn join_inner() {
    let s = parse_select("SELECT * FROM a INNER JOIN b ON a.id = b.a_id");
    if let Some(TableRef::Join { join, .. }) = &s.from {
        assert_eq!(join.join_type, JoinType::Inner);
        assert!(join.on.is_some());
    } else {
        panic!("Expected JOIN");
    }
    round_trip("SELECT * FROM a INNER JOIN b ON a.id = b.a_id");
}

#[test]
fn join_left() {
    let s = parse_select("SELECT * FROM a LEFT JOIN b ON a.id = b.a_id");
    if let Some(TableRef::Join { join, .. }) = &s.from {
        assert_eq!(join.join_type, JoinType::Left);
    } else {
        panic!("Expected JOIN");
    }
    round_trip("SELECT * FROM a LEFT JOIN b ON a.id = b.a_id");
}

#[test]
fn join_right() {
    let s = parse_select("SELECT * FROM a RIGHT JOIN b ON a.id = b.a_id");
    if let Some(TableRef::Join { join, .. }) = &s.from {
        assert_eq!(join.join_type, JoinType::Right);
    } else {
        panic!("Expected JOIN");
    }
    round_trip("SELECT * FROM a RIGHT JOIN b ON a.id = b.a_id");
}

#[test]
fn join_full() {
    let s = parse_select("SELECT * FROM a FULL JOIN b ON a.id = b.a_id");
    if let Some(TableRef::Join { join, .. }) = &s.from {
        assert_eq!(join.join_type, JoinType::Full);
    } else {
        panic!("Expected JOIN");
    }
    round_trip("SELECT * FROM a FULL JOIN b ON a.id = b.a_id");
}

#[test]
fn join_cross() {
    let s = parse_select("SELECT * FROM a CROSS JOIN b");
    if let Some(TableRef::Join { join, .. }) = &s.from {
        assert_eq!(join.join_type, JoinType::Cross);
        assert!(join.on.is_none());
        assert!(join.using.is_empty());
    } else {
        panic!("Expected CROSS JOIN");
    }
    round_trip("SELECT * FROM a CROSS JOIN b");
}

#[test]
fn join_left_outer() {
    let s = parse_select("SELECT * FROM a LEFT OUTER JOIN b ON a.id = b.a_id");
    if let Some(TableRef::Join { join, .. }) = &s.from {
        assert_eq!(join.join_type, JoinType::Left);
    } else {
        panic!("Expected LEFT OUTER JOIN");
    }
    round_trip("SELECT * FROM a LEFT JOIN b ON a.id = b.a_id");
}

#[test]
fn join_right_outer() {
    let s = parse_select("SELECT * FROM a RIGHT OUTER JOIN b ON a.id = b.a_id");
    if let Some(TableRef::Join { join, .. }) = &s.from {
        assert_eq!(join.join_type, JoinType::Right);
    } else {
        panic!("Expected RIGHT OUTER JOIN");
    }
    round_trip("SELECT * FROM a RIGHT JOIN b ON a.id = b.a_id");
}

#[test]
fn join_full_outer() {
    let s = parse_select("SELECT * FROM a FULL OUTER JOIN b ON a.id = b.a_id");
    if let Some(TableRef::Join { join, .. }) = &s.from {
        assert_eq!(join.join_type, JoinType::Full);
    } else {
        panic!("Expected FULL OUTER JOIN");
    }
    round_trip("SELECT * FROM a FULL JOIN b ON a.id = b.a_id");
}

#[test]
fn join_bare_defaults_to_inner() {
    let s = parse_select("SELECT * FROM a JOIN b ON a.id = b.a_id");
    if let Some(TableRef::Join { join, .. }) = &s.from {
        assert_eq!(join.join_type, JoinType::Inner);
    } else {
        panic!("Expected bare JOIN");
    }
    round_trip("SELECT * FROM a INNER JOIN b ON a.id = b.a_id");
}

#[test]
fn join_using_single_column() {
    let s = parse_select("SELECT * FROM a JOIN b USING (id)");
    if let Some(TableRef::Join { join, .. }) = &s.from {
        assert!(join.on.is_none());
        assert_eq!(join.using, vec!["id"]);
    } else {
        panic!("Expected JOIN USING");
    }
    round_trip("SELECT * FROM a INNER JOIN b USING (id)");
}

#[test]
fn join_using_multiple_columns() {
    let s = parse_select("SELECT * FROM a JOIN b USING (id, name)");
    if let Some(TableRef::Join { join, .. }) = &s.from {
        assert_eq!(join.using, vec!["id", "name"]);
    } else {
        panic!("Expected JOIN USING");
    }
    round_trip("SELECT * FROM a INNER JOIN b USING (id, name)");
}

#[test]
fn join_chained_three_tables() {
    let s = parse_select(
        "SELECT * FROM a \
         JOIN b ON a.id = b.a_id \
         JOIN c ON b.id = c.b_id",
    );
    if let Some(TableRef::Join { left, join: outer }) = &s.from {
        assert_eq!(outer.join_type, JoinType::Inner);
        assert!(matches!(
            &outer.table,
            TableRef::Table { name, .. } if name == "c"
        ));
        assert!(matches!(left.as_ref(), TableRef::Join { .. }));
    } else {
        panic!("Expected chained JOIN");
    }
    round_trip("SELECT * FROM a INNER JOIN b ON a.id = b.a_id INNER JOIN c ON b.id = c.b_id");
}
