//! Comprehensive parser integration tests.
//!
//! Covers every parser feature with realistic and complex SQL.

use oxide_sql_core::ast::{
    BinaryOp, DataType, DeleteStatement, Expr, FunctionCall, InsertSource, InsertStatement,
    JoinType, Literal, OrderDirection, SelectStatement, Statement, TableRef, UnaryOp,
    UpdateStatement,
};
use oxide_sql_core::{ParseError, Parser};

// ===================================================================
// Helper functions
// ===================================================================

fn parse(sql: &str) -> Statement {
    Parser::new(sql)
        .parse_statement()
        .unwrap_or_else(|e| panic!("Failed to parse: {sql}\nError: {e:?}"))
}

fn parse_err(sql: &str) -> ParseError {
    Parser::new(sql)
        .parse_statement()
        .expect_err(&format!("Expected parse error for: {sql}"))
}

fn parse_select(sql: &str) -> SelectStatement {
    match parse(sql) {
        Statement::Select(s) => s,
        other => panic!("Expected SELECT, got {other:?}"),
    }
}

fn parse_insert(sql: &str) -> InsertStatement {
    match parse(sql) {
        Statement::Insert(i) => i,
        other => panic!("Expected INSERT, got {other:?}"),
    }
}

fn parse_update(sql: &str) -> UpdateStatement {
    match parse(sql) {
        Statement::Update(u) => u,
        other => panic!("Expected UPDATE, got {other:?}"),
    }
}

fn parse_delete(sql: &str) -> DeleteStatement {
    match parse(sql) {
        Statement::Delete(d) => d,
        other => panic!("Expected DELETE, got {other:?}"),
    }
}

/// Verifies that `to_string()` produces a fixed point:
/// parse(sql).to_string() can be re-parsed and yields the same
/// string again.
fn round_trip(sql: &str) {
    let ast1 = parse(sql);
    let rendered1 = ast1.to_string();
    let ast2 = parse(&rendered1);
    let rendered2 = ast2.to_string();
    assert_eq!(
        rendered1, rendered2,
        "Round-trip failed.\n  Input:    {sql}\n  First:    {rendered1}\n  Second:   {rendered2}"
    );
}

// ===================================================================
// 1. SELECT — Column selection
// ===================================================================

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

// ===================================================================
// 2. SELECT — FROM clause & table refs
// ===================================================================

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

// ===================================================================
// 3. SELECT — JOIN types
// ===================================================================

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
    // The outer join is left=Join(a,b), right=c
    if let Some(TableRef::Join { left, join: outer }) = &s.from {
        assert_eq!(outer.join_type, JoinType::Inner);
        assert!(matches!(
            &outer.table,
            TableRef::Table { name, .. } if name == "c"
        ));
        // Inner join is left=a, right=b
        assert!(matches!(left.as_ref(), TableRef::Join { .. }));
    } else {
        panic!("Expected chained JOIN");
    }
    round_trip("SELECT * FROM a INNER JOIN b ON a.id = b.a_id INNER JOIN c ON b.id = c.b_id");
}

// ===================================================================
// 4. SELECT — WHERE, GROUP BY, HAVING
// ===================================================================

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
    // Outer is OR because AND binds tighter
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

// ===================================================================
// 5. SELECT — ORDER BY, LIMIT, OFFSET
// ===================================================================

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

// ===================================================================
// 6. INSERT
// ===================================================================

#[test]
fn insert_with_columns() {
    let i = parse_insert(
        "INSERT INTO users (name, email) \
         VALUES ('Alice', 'alice@example.com')",
    );
    assert_eq!(i.table, "users");
    assert_eq!(i.columns, vec!["name", "email"]);
    if let InsertSource::Values(rows) = &i.values {
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].len(), 2);
    } else {
        panic!("Expected VALUES");
    }
    round_trip("INSERT INTO users (name, email) VALUES ('Alice', 'alice@example.com')");
}

#[test]
fn insert_without_columns() {
    let i = parse_insert("INSERT INTO users VALUES (1, 'Bob', 'b@x.com')");
    assert!(i.columns.is_empty());
    if let InsertSource::Values(rows) = &i.values {
        assert_eq!(rows[0].len(), 3);
    } else {
        panic!("Expected VALUES");
    }
    round_trip("INSERT INTO users VALUES (1, 'Bob', 'b@x.com')");
}

#[test]
fn insert_multiple_rows() {
    let i = parse_insert("INSERT INTO users (name) VALUES ('A'), ('B'), ('C')");
    if let InsertSource::Values(rows) = &i.values {
        assert_eq!(rows.len(), 3);
    } else {
        panic!("Expected VALUES");
    }
    round_trip("INSERT INTO users (name) VALUES ('A'), ('B'), ('C')");
}

#[test]
fn insert_select() {
    let i = parse_insert(
        "INSERT INTO archive (id, name) \
         SELECT id, name FROM users WHERE active = 0",
    );
    assert!(matches!(i.values, InsertSource::Query(_)));
    round_trip("INSERT INTO archive (id, name) SELECT id, name FROM users WHERE active = 0");
}

#[test]
fn insert_default_values() {
    let i = parse_insert("INSERT INTO counters DEFAULT VALUES");
    assert!(matches!(i.values, InsertSource::DefaultValues));
    round_trip("INSERT INTO counters DEFAULT VALUES");
}

#[test]
fn insert_schema_qualified() {
    let i = parse_insert("INSERT INTO public.users (name) VALUES ('Eve')");
    assert_eq!(i.schema.as_deref(), Some("public"));
    assert_eq!(i.table, "users");
    round_trip("INSERT INTO public.users (name) VALUES ('Eve')");
}

#[test]
fn insert_with_expressions() {
    let i = parse_insert("INSERT INTO stats (value) VALUES (1 + 2)");
    if let InsertSource::Values(rows) = &i.values {
        assert!(matches!(
            &rows[0][0],
            Expr::Binary {
                op: BinaryOp::Add,
                ..
            }
        ));
    } else {
        panic!("Expected VALUES");
    }
    round_trip("INSERT INTO stats (value) VALUES (1 + 2)");
}

#[test]
fn insert_with_parameters() {
    let i = parse_insert("INSERT INTO users (name, email) VALUES (?, ?)");
    if let InsertSource::Values(rows) = &i.values {
        assert!(matches!(
            &rows[0][0],
            Expr::Parameter {
                position: 1,
                name: None
            }
        ));
        assert!(matches!(
            &rows[0][1],
            Expr::Parameter {
                position: 2,
                name: None
            }
        ));
    } else {
        panic!("Expected VALUES");
    }
    round_trip("INSERT INTO users (name, email) VALUES (?, ?)");
}

// ===================================================================
// 7. UPDATE
// ===================================================================

#[test]
fn update_single_set() {
    let u = parse_update("UPDATE users SET name = 'Bob' WHERE id = 1");
    assert_eq!(u.table, "users");
    assert_eq!(u.assignments.len(), 1);
    assert_eq!(u.assignments[0].column, "name");
    assert!(u.where_clause.is_some());
    round_trip("UPDATE users SET name = 'Bob' WHERE id = 1");
}

#[test]
fn update_multiple_set() {
    let u = parse_update("UPDATE users SET name = 'Bob', email = 'bob@x.com' WHERE id = 1");
    assert_eq!(u.assignments.len(), 2);
    assert_eq!(u.assignments[0].column, "name");
    assert_eq!(u.assignments[1].column, "email");
    round_trip("UPDATE users SET name = 'Bob', email = 'bob@x.com' WHERE id = 1");
}

#[test]
fn update_schema_qualified() {
    let u = parse_update("UPDATE public.users SET name = 'X' WHERE id = 1");
    assert_eq!(u.schema.as_deref(), Some("public"));
    assert_eq!(u.table, "users");
    round_trip("UPDATE public.users SET name = 'X' WHERE id = 1");
}

#[test]
fn update_with_alias() {
    let u = parse_update("UPDATE users u SET name = 'X' WHERE u.id = 1");
    assert_eq!(u.alias.as_deref(), Some("u"));
    round_trip("UPDATE users AS u SET name = 'X' WHERE u.id = 1");
}

#[test]
fn update_with_from_clause() {
    let u = parse_update(
        "UPDATE orders SET total = p.price \
         FROM products p \
         WHERE orders.product_id = p.id",
    );
    assert!(u.from.is_some());
    assert!(u.where_clause.is_some());
    round_trip(
        "UPDATE orders SET total = p.price FROM products AS p WHERE orders.product_id = p.id",
    );
}

#[test]
fn update_without_where() {
    let u = parse_update("UPDATE users SET active = 0");
    assert!(u.where_clause.is_none());
    round_trip("UPDATE users SET active = 0");
}

#[test]
fn update_with_parameters() {
    let u = parse_update("UPDATE users SET name = ?, email = :email WHERE id = ?");
    assert!(matches!(
        &u.assignments[0].value,
        Expr::Parameter {
            position: 1,
            name: None
        }
    ));
    assert!(matches!(
        &u.assignments[1].value,
        Expr::Parameter { name: Some(n), .. } if n == "email"
    ));
    round_trip("UPDATE users SET name = ?, email = :email WHERE id = ?");
}

// ===================================================================
// 8. DELETE
// ===================================================================

#[test]
fn delete_with_where() {
    let d = parse_delete("DELETE FROM users WHERE id = 1");
    assert_eq!(d.table, "users");
    assert!(d.where_clause.is_some());
    round_trip("DELETE FROM users WHERE id = 1");
}

#[test]
fn delete_without_where() {
    let d = parse_delete("DELETE FROM users");
    assert!(d.where_clause.is_none());
    round_trip("DELETE FROM users");
}

#[test]
fn delete_schema_qualified() {
    let d = parse_delete("DELETE FROM public.users WHERE id = 1");
    assert_eq!(d.schema.as_deref(), Some("public"));
    assert_eq!(d.table, "users");
    round_trip("DELETE FROM public.users WHERE id = 1");
}

#[test]
fn delete_with_alias() {
    let d = parse_delete("DELETE FROM users u WHERE u.active = 0");
    assert_eq!(d.alias.as_deref(), Some("u"));
    round_trip("DELETE FROM users AS u WHERE u.active = 0");
}

#[test]
fn delete_complex_where() {
    let d = parse_delete(
        "DELETE FROM logs \
         WHERE created_at < 1000 AND level = 'debug'",
    );
    assert!(matches!(
        &d.where_clause,
        Some(Expr::Binary {
            op: BinaryOp::And,
            ..
        })
    ));
    round_trip("DELETE FROM logs WHERE created_at < 1000 AND level = 'debug'");
}

// ===================================================================
// 9. Literals
// ===================================================================

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
    // Parser produces Unary { Neg, 7 }
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

// ===================================================================
// 10. Binary operators — arithmetic
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
// 11. Binary operators — comparison
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
// 12. Binary operators — logical & LIKE
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
// 13. Binary operators — bitwise
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
// 14. Unary operators
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
// 15. Operator precedence
// ===================================================================

#[test]
fn precedence_mul_over_add() {
    // 1 + 2 * 3 -> Add(1, Mul(2, 3))
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
    // 1 - 2 - 3 -> Sub(Sub(1, 2), 3)
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
    // a = 1 AND b = 2 -> AND(Eq(a,1), Eq(b,2))
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
    // a OR b AND c -> OR(a, AND(b, c))
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
    // (1 + 2) * 3 -> Mul(Paren(Add(1, 2)), 3)
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
    // ((1 + 2)) * 3
    let s = parse_select("SELECT ((1 + 2)) * 3");
    if let Expr::Binary { op, left, .. } = &s.columns[0].expr {
        assert_eq!(*op, BinaryOp::Mul);
        // Outer paren contains inner paren
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
    // Unary neg prefix bp = 15, mul left bp = 15.
    // Since l_bp (15) is NOT less than min_bp (15), * is consumed
    // by the unary operand: -x * y -> -(x * y)
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

// ===================================================================
// 16. IS NULL / IS NOT NULL
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
// 17. BETWEEN
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
// 18. IN
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
// 19. CASE expressions
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
    // Top-level is Eq, left is CASE
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
// 20. CAST
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

// ===================================================================
// 21. Function calls (aggregates)
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
// 22. Custom functions
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
// 23. Subqueries & EXISTS
// ===================================================================

#[test]
fn exists_in_where() {
    let s = parse_select(
        "SELECT * FROM users u \
         WHERE EXISTS (SELECT 1 FROM orders o WHERE o.user_id = u.id)",
    );
    // EXISTS is parsed as a function call
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
// 24. Parameters
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
    // Parse succeeds; positions: ?=1, :name=0, ?=2
    assert!(s.where_clause.is_some());
    round_trip("SELECT * FROM t WHERE a = ? AND b = :name AND c = ?");
}

// ===================================================================
// 25. Data types via CAST
// ===================================================================

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
        // Parser maps FLOAT -> Double
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

// ===================================================================
// 26. Complex realistic queries
// ===================================================================

#[test]
fn complex_report_query() {
    let s = parse_select(
        "SELECT c.name, COUNT(o.id) AS order_count, SUM(o.total) AS revenue \
         FROM customers c \
         LEFT JOIN orders o ON c.id = o.customer_id \
         WHERE c.active = 1 \
         GROUP BY c.name \
         HAVING COUNT(o.id) > 0 \
         ORDER BY revenue DESC \
         LIMIT 100",
    );
    assert_eq!(s.columns.len(), 3);
    assert!(s.where_clause.is_some());
    assert_eq!(s.group_by.len(), 1);
    assert!(s.having.is_some());
    assert_eq!(s.order_by.len(), 1);
    assert_eq!(s.order_by[0].direction, OrderDirection::Desc);
    assert!(s.limit.is_some());
    round_trip("SELECT c.name, COUNT(o.id) AS order_count, SUM(o.total) AS revenue FROM customers AS c LEFT JOIN orders AS o ON c.id = o.customer_id WHERE c.active = 1 GROUP BY c.name HAVING COUNT(o.id) > 0 ORDER BY revenue DESC LIMIT 100");
}

#[test]
fn complex_self_join() {
    let s = parse_select(
        "SELECT e.name, m.name AS manager_name \
         FROM employees e \
         LEFT JOIN employees m ON e.manager_id = m.id",
    );
    if let Some(TableRef::Join { left, join }) = &s.from {
        assert_eq!(join.join_type, JoinType::Left);
        assert!(matches!(
            left.as_ref(),
            TableRef::Table { name, alias: Some(a), .. }
                if name == "employees" && a == "e"
        ));
        assert!(matches!(
            &join.table,
            TableRef::Table { name, alias: Some(a), .. }
                if name == "employees" && a == "m"
        ));
    } else {
        panic!("Expected self-join");
    }
    round_trip("SELECT e.name, m.name AS manager_name FROM employees AS e LEFT JOIN employees AS m ON e.manager_id = m.id");
}

#[test]
fn complex_three_table_join() {
    let s = parse_select(
        "SELECT u.name, o.id, p.title \
         FROM users u \
         JOIN orders o ON u.id = o.user_id \
         JOIN products p ON o.product_id = p.id",
    );
    // Outer: Join(Join(users, orders), products)
    if let Some(TableRef::Join { left, join: outer }) = &s.from {
        assert!(matches!(
            &outer.table,
            TableRef::Table { name, .. } if name == "products"
        ));
        assert!(matches!(left.as_ref(), TableRef::Join { .. }));
    } else {
        panic!("Expected 3-table join");
    }
    round_trip("SELECT u.name, o.id, p.title FROM users AS u INNER JOIN orders AS o ON u.id = o.user_id INNER JOIN products AS p ON o.product_id = p.id");
}

#[test]
fn complex_insert_from_select_with_join() {
    let i = parse_insert(
        "INSERT INTO order_summary (user_name, total) \
         SELECT u.name, SUM(o.amount) \
         FROM users u \
         JOIN orders o ON u.id = o.user_id \
         GROUP BY u.name",
    );
    assert_eq!(i.columns, vec!["user_name", "total"]);
    if let InsertSource::Query(q) = &i.values {
        assert!(q.from.is_some());
        assert_eq!(q.group_by.len(), 1);
    } else {
        panic!("Expected INSERT ... SELECT");
    }
    round_trip("INSERT INTO order_summary (user_name, total) SELECT u.name, SUM(o.amount) FROM users AS u INNER JOIN orders AS o ON u.id = o.user_id GROUP BY u.name");
}

#[test]
fn complex_deeply_nested_arithmetic() {
    let s = parse_select("SELECT ((1 + 2) * (3 - 4)) / 5");
    // Top level: Div
    if let Expr::Binary { op, .. } = &s.columns[0].expr {
        assert_eq!(*op, BinaryOp::Div);
    } else {
        panic!("Expected division");
    }
    round_trip("SELECT ((1 + 2) * (3 - 4)) / 5");
}

#[test]
fn complex_case_with_alias_and_order_by() {
    let s = parse_select(
        "SELECT id, \
            CASE \
                WHEN score >= 90 THEN 'A' \
                WHEN score >= 80 THEN 'B' \
                ELSE 'C' \
            END AS grade \
         FROM students \
         ORDER BY grade ASC",
    );
    assert_eq!(s.columns.len(), 2);
    assert_eq!(s.columns[1].alias.as_deref(), Some("grade"));
    assert!(matches!(&s.columns[1].expr, Expr::Case { .. }));
    assert_eq!(s.order_by.len(), 1);
    round_trip("SELECT id, CASE WHEN score >= 90 THEN 'A' WHEN score >= 80 THEN 'B' ELSE 'C' END AS grade FROM students ORDER BY grade ASC");
}

#[test]
fn complex_where_mixing_operators() {
    let s = parse_select(
        "SELECT * FROM products \
         WHERE (price > 10 AND price < 100) \
            OR (name LIKE '%sale%' AND active = 1)",
    );
    assert!(matches!(
        &s.where_clause,
        Some(Expr::Binary {
            op: BinaryOp::Or,
            ..
        })
    ));
    round_trip(
        "SELECT * FROM products WHERE (price > 10 AND price < 100) OR (name LIKE '%sale%' AND active = 1)",
    );
}

#[test]
fn complex_update_with_subquery_in_set() {
    let u = parse_update(
        "UPDATE users SET rank = (SELECT COUNT(*) FROM scores WHERE scores.user_id = users.id) \
         WHERE active = 1",
    );
    assert_eq!(u.assignments.len(), 1);
    assert!(matches!(&u.assignments[0].value, Expr::Subquery(_)));
    assert!(u.where_clause.is_some());
    round_trip(
        "UPDATE users SET rank = (SELECT COUNT(*) FROM scores WHERE scores.user_id = users.id) WHERE active = 1",
    );
}

// ===================================================================
// 27. Error cases
// ===================================================================

#[test]
fn error_empty_input() {
    let _ = parse_err("");
}

#[test]
fn error_incomplete_select() {
    // SELECT without columns
    let _ = parse_err("SELECT");
}

#[test]
fn error_missing_from_table() {
    // FROM without a table name
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
