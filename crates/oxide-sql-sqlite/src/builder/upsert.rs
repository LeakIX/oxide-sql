//! SQLite UPSERT (INSERT ... ON CONFLICT) builder.

#[cfg(feature = "alloc")]
use alloc::{format, string::String, vec, vec::Vec};

use core::marker::PhantomData;

use oxide_sql_core::builder::value::{SqlValue, ToSqlValue};

// Typestate markers

/// Marker: No table specified yet.
pub struct NoTable;
/// Marker: Table has been specified.
pub struct HasTable;
/// Marker: No values specified yet.
pub struct NoValues;
/// Marker: Values have been specified.
pub struct HasValues;
/// Marker: No conflict target specified yet.
pub struct NoConflict;
/// Marker: Conflict target has been specified.
pub struct HasConflict;

/// A type-safe UPSERT (INSERT ... ON CONFLICT) builder for SQLite.
#[cfg(feature = "alloc")]
pub struct UpsertBuilder<Table, Values, Conflict> {
    table: Option<String>,
    columns: Vec<String>,
    values: Vec<SqlValue>,
    conflict_columns: Vec<String>,
    update_columns: Vec<String>,
    do_nothing: bool,
    _state: PhantomData<(Table, Values, Conflict)>,
}

#[cfg(feature = "alloc")]
impl UpsertBuilder<NoTable, NoValues, NoConflict> {
    /// Creates a new UPSERT builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            table: None,
            columns: vec![],
            values: vec![],
            conflict_columns: vec![],
            update_columns: vec![],
            do_nothing: false,
            _state: PhantomData,
        }
    }
}

#[cfg(feature = "alloc")]
impl Default for UpsertBuilder<NoTable, NoValues, NoConflict> {
    fn default() -> Self {
        Self::new()
    }
}

// Transition: NoTable -> HasTable
#[cfg(feature = "alloc")]
impl<Values, Conflict> UpsertBuilder<NoTable, Values, Conflict> {
    /// Specifies the table to insert into.
    #[must_use]
    pub fn into_table(self, table: &str) -> UpsertBuilder<HasTable, Values, Conflict> {
        UpsertBuilder {
            table: Some(String::from(table)),
            columns: self.columns,
            values: self.values,
            conflict_columns: self.conflict_columns,
            update_columns: self.update_columns,
            do_nothing: self.do_nothing,
            _state: PhantomData,
        }
    }
}

// Methods available after specifying table
#[cfg(feature = "alloc")]
impl<Values, Conflict> UpsertBuilder<HasTable, Values, Conflict> {
    /// Specifies the columns to insert into.
    #[must_use]
    pub fn columns(mut self, cols: &[&str]) -> Self {
        self.columns = cols.iter().map(|s| String::from(*s)).collect();
        self
    }
}

// Transition: NoValues -> HasValues
#[cfg(feature = "alloc")]
impl<Conflict> UpsertBuilder<HasTable, NoValues, Conflict> {
    /// Adds values to insert.
    #[must_use]
    pub fn values<T: ToSqlValue>(
        self,
        vals: Vec<T>,
    ) -> UpsertBuilder<HasTable, HasValues, Conflict> {
        let sql_values: Vec<SqlValue> = vals.into_iter().map(ToSqlValue::to_sql_value).collect();
        UpsertBuilder {
            table: self.table,
            columns: self.columns,
            values: sql_values,
            conflict_columns: self.conflict_columns,
            update_columns: self.update_columns,
            do_nothing: self.do_nothing,
            _state: PhantomData,
        }
    }
}

// Transition: NoConflict -> HasConflict
#[cfg(feature = "alloc")]
impl UpsertBuilder<HasTable, HasValues, NoConflict> {
    /// Specifies the conflict target columns.
    #[must_use]
    pub fn on_conflict(self, cols: &[&str]) -> UpsertBuilder<HasTable, HasValues, HasConflict> {
        UpsertBuilder {
            table: self.table,
            columns: self.columns,
            values: self.values,
            conflict_columns: cols.iter().map(|s| String::from(*s)).collect(),
            update_columns: self.update_columns,
            do_nothing: self.do_nothing,
            _state: PhantomData,
        }
    }
}

// Methods available after ON CONFLICT
#[cfg(feature = "alloc")]
impl UpsertBuilder<HasTable, HasValues, HasConflict> {
    /// Sets DO NOTHING action.
    #[must_use]
    pub fn do_nothing(mut self) -> Self {
        self.do_nothing = true;
        self
    }

    /// Sets DO UPDATE with specified columns.
    #[must_use]
    pub fn do_update(mut self, cols: &[&str]) -> Self {
        self.update_columns = cols.iter().map(|s| String::from(*s)).collect();
        self.do_nothing = false;
        self
    }

    /// Builds the UPSERT statement and returns SQL with parameters.
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

        sql.push_str(" VALUES (");
        let placeholders: Vec<&str> = self.values.iter().map(|_| "?").collect();
        sql.push_str(&placeholders.join(", "));
        sql.push(')');

        params.extend(self.values.clone());

        sql.push_str(" ON CONFLICT (");
        sql.push_str(&self.conflict_columns.join(", "));
        sql.push(')');

        if self.do_nothing {
            sql.push_str(" DO NOTHING");
        } else if !self.update_columns.is_empty() {
            sql.push_str(" DO UPDATE SET ");
            let updates: Vec<String> = self
                .update_columns
                .iter()
                .map(|col| format!("{col} = excluded.{col}"))
                .collect();
            sql.push_str(&updates.join(", "));
        }

        (sql, params)
    }

    /// Builds the UPSERT statement and returns only the SQL string.
    #[must_use]
    pub fn build_sql(self) -> String {
        let (sql, _) = self.build();
        sql
    }
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;

    #[test]
    fn test_upsert_do_nothing() {
        let (sql, params) = UpsertBuilder::new()
            .into_table("users")
            .columns(&["id", "name"])
            .values(vec![1_i64.to_sql_value(), "Alice".to_sql_value()])
            .on_conflict(&["id"])
            .do_nothing()
            .build();

        assert_eq!(
            sql,
            "INSERT INTO users (id, name) VALUES (?, ?) ON CONFLICT (id) DO NOTHING"
        );
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_upsert_do_update() {
        let (sql, params) = UpsertBuilder::new()
            .into_table("users")
            .columns(&["id", "name", "email"])
            .values(vec![
                1_i64.to_sql_value(),
                "Alice".to_sql_value(),
                "alice@example.com".to_sql_value(),
            ])
            .on_conflict(&["id"])
            .do_update(&["name", "email"])
            .build();

        assert_eq!(
            sql,
            "INSERT INTO users (id, name, email) VALUES (?, ?, ?) \
             ON CONFLICT (id) DO UPDATE SET name = excluded.name, email = excluded.email"
        );
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn test_upsert_composite_key() {
        let (sql, _) = UpsertBuilder::new()
            .into_table("user_roles")
            .columns(&["user_id", "role_id", "granted_at"])
            .values(vec![
                1_i64.to_sql_value(),
                2_i64.to_sql_value(),
                "2024-01-01".to_sql_value(),
            ])
            .on_conflict(&["user_id", "role_id"])
            .do_update(&["granted_at"])
            .build();

        assert!(sql.contains("ON CONFLICT (user_id, role_id)"));
        assert!(sql.contains("DO UPDATE SET granted_at = excluded.granted_at"));
    }

    #[test]
    fn test_upsert_sql_injection_prevention() {
        let malicious = "'; DROP TABLE users; --";
        let (sql, params) = UpsertBuilder::new()
            .into_table("users")
            .columns(&["id", "name"])
            .values(vec![1_i64.to_sql_value(), malicious.to_sql_value()])
            .on_conflict(&["id"])
            .do_update(&["name"])
            .build();

        // SQL uses parameterized placeholders
        assert!(sql.contains("VALUES (?, ?)"));
        // Malicious input is safely stored as parameter
        assert!(matches!(&params[1], SqlValue::Text(s) if s == malicious));
    }
}
