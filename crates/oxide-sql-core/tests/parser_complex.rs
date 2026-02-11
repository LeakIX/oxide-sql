//! Tests for complex realistic queries combining multiple features.

mod common;
use common::*;

use oxide_sql_core::ast::{BinaryOp, Expr, InsertSource, JoinType, OrderDirection, TableRef};

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
