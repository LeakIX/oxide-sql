//! Dynamic INSERT statement builder using the typestate pattern.
//!
//! This module provides string-based query building. For compile-time
//! validated queries using schema traits, use `Insert` from `builder::typed`.

use std::marker::PhantomData;

use super::value::{SqlValue, ToSqlValue};

// Typestate markers

/// Marker: No table specified yet.
pub struct NoTable;
/// Marker: Table has been specified.
pub struct HasTable;
/// Marker: No values specified yet.
pub struct NoValues;
/// Marker: Values have been specified.
pub struct HasValues;

/// A dynamic INSERT statement builder using string-based column names.
///
/// For compile-time validated queries, use `Insert` from `builder::typed`.
pub struct InsertDyn<Table, Values> {
    table: Option<String>,
    columns: Vec<String>,
    values: Vec<Vec<SqlValue>>,
    _state: PhantomData<(Table, Values)>,
}

impl InsertDyn<NoTable, NoValues> {
    /// Creates a new INSERT builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            table: None,
            columns: vec![],
            values: vec![],
            _state: PhantomData,
        }
    }
}

impl Default for InsertDyn<NoTable, NoValues> {
    fn default() -> Self {
        Self::new()
    }
}

// Transition: NoTable -> HasTable
impl<Values> InsertDyn<NoTable, Values> {
    /// Specifies the table to insert into.
    #[must_use]
    pub fn into_table(self, table: &str) -> InsertDyn<HasTable, Values> {
        InsertDyn {
            table: Some(String::from(table)),
            columns: self.columns,
            values: self.values,
            _state: PhantomData,
        }
    }
}

// Methods available after specifying table
impl<Values> InsertDyn<HasTable, Values> {
    /// Specifies the columns to insert into.
    #[must_use]
    pub fn columns(mut self, cols: &[&str]) -> Self {
        self.columns = cols.iter().map(|s| String::from(*s)).collect();
        self
    }
}

// Transition: NoValues -> HasValues
impl InsertDyn<HasTable, NoValues> {
    /// Adds a row of values to insert.
    #[must_use]
    pub fn values<T: ToSqlValue>(self, vals: Vec<T>) -> InsertDyn<HasTable, HasValues> {
        let sql_values: Vec<SqlValue> = vals.into_iter().map(ToSqlValue::to_sql_value).collect();
        InsertDyn {
            table: self.table,
            columns: self.columns,
            values: vec![sql_values],
            _state: PhantomData,
        }
    }

    /// Adds multiple rows of values to insert.
    #[must_use]
    pub fn values_many<T: ToSqlValue>(self, rows: Vec<Vec<T>>) -> InsertDyn<HasTable, HasValues> {
        let sql_rows: Vec<Vec<SqlValue>> = rows
            .into_iter()
            .map(|row| row.into_iter().map(ToSqlValue::to_sql_value).collect())
            .collect();
        InsertDyn {
            table: self.table,
            columns: self.columns,
            values: sql_rows,
            _state: PhantomData,
        }
    }
}

// Methods available after adding values
impl InsertDyn<HasTable, HasValues> {
    /// Adds another row of values.
    #[must_use]
    pub fn and_values<T: ToSqlValue>(mut self, vals: Vec<T>) -> Self {
        let sql_values: Vec<SqlValue> = vals.into_iter().map(ToSqlValue::to_sql_value).collect();
        self.values.push(sql_values);
        self
    }

    /// Builds the INSERT statement and returns SQL with parameters.
    #[must_use]
    pub fn build(self) -> (String, Vec<SqlValue>) {
        let mut sql = String::from("INSERT INTO ");
        let mut params = vec![];

        if let Some(ref table) = self.table {
            sql.push_str(table);
        }

        if !self.columns.is_empty() {
            sql.push_str(" (");
            sql.push_str(&self.columns.join(", "));
            sql.push(')');
        }

        sql.push_str(" VALUES ");

        let row_strs: Vec<String> = self
            .values
            .iter()
            .map(|row| {
                let placeholders: Vec<&str> = row.iter().map(|_| "?").collect();
                format!("({})", placeholders.join(", "))
            })
            .collect();

        sql.push_str(&row_strs.join(", "));

        for row in self.values {
            params.extend(row);
        }

        (sql, params)
    }

    /// Builds the INSERT statement and returns only the SQL string.
    ///
    /// **Warning**: Parameters are NOT inlined. Use `build()` to get parameters.
    #[must_use]
    pub fn build_sql(self) -> String {
        let (sql, _) = self.build();
        sql
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_insert() {
        let (sql, params) = InsertDyn::new()
            .into_table("users")
            .columns(&["name", "email"])
            .values(vec!["Alice", "alice@example.com"])
            .build();

        assert_eq!(sql, "INSERT INTO users (name, email) VALUES (?, ?)");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_insert_multiple_rows() {
        let (sql, params) = InsertDyn::new()
            .into_table("users")
            .columns(&["name"])
            .values(vec!["Alice"])
            .and_values(vec!["Bob"])
            .and_values(vec!["Charlie"])
            .build();

        assert_eq!(sql, "INSERT INTO users (name) VALUES (?), (?), (?)");
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn test_insert_without_columns() {
        let (sql, params) = InsertDyn::new()
            .into_table("users")
            .values(vec!["Alice", "alice@example.com"])
            .build();

        assert_eq!(sql, "INSERT INTO users VALUES (?, ?)");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_insert_with_integers() {
        let (sql, params) = InsertDyn::new()
            .into_table("orders")
            .columns(&["user_id", "amount"])
            .values(vec![1_i64.to_sql_value(), 100_i64.to_sql_value()])
            .build();

        assert_eq!(sql, "INSERT INTO orders (user_id, amount) VALUES (?, ?)");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_insert_sql_injection_prevention() {
        let malicious = "'; DROP TABLE users; --";
        let (sql, params) = InsertDyn::new()
            .into_table("users")
            .columns(&["name"])
            .values(vec![malicious])
            .build();

        // SQL uses parameterized placeholder
        assert_eq!(sql, "INSERT INTO users (name) VALUES (?)");
        // Malicious input is safely stored as parameter
        assert!(matches!(&params[0], SqlValue::Text(s) if s == malicious));
    }
}
