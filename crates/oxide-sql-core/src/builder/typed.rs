//! Type-safe query builder using schema traits.
//!
//! This module provides query builders that use compile-time type checking
//! to ensure that column names are valid for the table being queried.

#[cfg(feature = "alloc")]
use alloc::{string::String, vec, vec::Vec};

use core::marker::PhantomData;

use crate::schema::{Column, Selectable, Table};

use super::expr::ExprBuilder;
use super::value::SqlValue;

// Typestate markers
/// Marker: No columns selected yet.
pub struct NoColumns;
/// Marker: Columns have been selected.
pub struct HasColumns;
/// Marker: No table specified yet.
pub struct NoFrom;
/// Marker: Table has been specified.
pub struct HasFrom;

/// A type-safe SELECT query builder that validates column names at compile time.
///
/// This builder uses generic type parameters to ensure that:
/// - Only valid columns for the table can be selected
/// - Column types are known at compile time
/// - Invalid queries fail to compile
#[cfg(feature = "alloc")]
pub struct TypedSelect<T, Cols, From>
where
    T: Table,
{
    columns: Vec<&'static str>,
    from: Option<&'static str>,
    where_clause: Option<ExprBuilder>,
    order_by: Vec<(&'static str, bool)>,
    limit: Option<i64>,
    offset: Option<i64>,
    _table: PhantomData<T>,
    _cols: PhantomData<Cols>,
    _from: PhantomData<From>,
}

#[cfg(feature = "alloc")]
impl<T: Table> TypedSelect<T, NoColumns, NoFrom> {
    /// Creates a new typed SELECT builder for the given table.
    #[must_use]
    pub fn new() -> Self {
        Self {
            columns: vec![],
            from: None,
            where_clause: None,
            order_by: vec![],
            limit: None,
            offset: None,
            _table: PhantomData,
            _cols: PhantomData,
            _from: PhantomData,
        }
    }
}

#[cfg(feature = "alloc")]
impl<T: Table> Default for TypedSelect<T, NoColumns, NoFrom> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "alloc")]
impl<T: Table, From> TypedSelect<T, NoColumns, From> {
    /// Selects specific columns from the table.
    ///
    /// The columns must implement `Selectable<T>` and belong to the table `T`.
    /// This is enforced at compile time.
    #[must_use]
    pub fn select<S: Selectable<T>>(self) -> TypedSelect<T, HasColumns, From> {
        let column_names = S::column_names();
        TypedSelect {
            columns: column_names.to_vec(),
            from: self.from,
            where_clause: self.where_clause,
            order_by: self.order_by,
            limit: self.limit,
            offset: self.offset,
            _table: PhantomData,
            _cols: PhantomData,
            _from: PhantomData,
        }
    }

    /// Selects all columns from the table.
    #[must_use]
    pub fn select_all(self) -> TypedSelect<T, HasColumns, From> {
        TypedSelect {
            columns: T::COLUMNS.to_vec(),
            from: self.from,
            where_clause: self.where_clause,
            order_by: self.order_by,
            limit: self.limit,
            offset: self.offset,
            _table: PhantomData,
            _cols: PhantomData,
            _from: PhantomData,
        }
    }
}

#[cfg(feature = "alloc")]
impl<T: Table, Cols> TypedSelect<T, Cols, NoFrom> {
    /// Specifies the table to query from.
    ///
    /// The table name is automatically derived from the `Table` trait.
    #[must_use]
    pub fn from_table(self) -> TypedSelect<T, Cols, HasFrom> {
        TypedSelect {
            columns: self.columns,
            from: Some(T::NAME),
            where_clause: self.where_clause,
            order_by: self.order_by,
            limit: self.limit,
            offset: self.offset,
            _table: PhantomData,
            _cols: PhantomData,
            _from: PhantomData,
        }
    }
}

#[cfg(feature = "alloc")]
impl<T: Table, Cols> TypedSelect<T, Cols, HasFrom> {
    /// Adds a WHERE clause with a type-safe column expression.
    #[must_use]
    pub fn where_col<C: Column<Table = T>>(mut self, _col: C, expr: ExprBuilder) -> Self {
        // The column type C ensures the column belongs to table T
        self.where_clause = Some(expr);
        self
    }

    /// Adds a WHERE clause with a raw expression.
    #[must_use]
    pub fn where_clause(mut self, expr: ExprBuilder) -> Self {
        self.where_clause = Some(expr);
        self
    }

    /// Adds an ORDER BY clause for a column.
    #[must_use]
    pub fn order_by<C: Column<Table = T>>(mut self, _col: C, ascending: bool) -> Self {
        self.order_by.push((C::NAME, ascending));
        self
    }

    /// Sets the LIMIT clause.
    #[must_use]
    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the OFFSET clause.
    #[must_use]
    pub fn offset(mut self, offset: i64) -> Self {
        self.offset = Some(offset);
        self
    }
}

#[cfg(feature = "alloc")]
impl<T: Table> TypedSelect<T, HasColumns, HasFrom> {
    /// Builds the query and returns (SQL, parameters).
    #[must_use]
    pub fn build(self) -> (String, Vec<SqlValue>) {
        let mut sql = String::from("SELECT ");
        let params = vec![];

        // Columns
        sql.push_str(&self.columns.join(", "));

        // FROM
        if let Some(table) = self.from {
            sql.push_str(" FROM ");
            sql.push_str(table);
        }

        // WHERE
        if let Some(ref where_expr) = self.where_clause {
            sql.push_str(" WHERE ");
            sql.push_str(where_expr.sql());
        }

        // ORDER BY
        if !self.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            let orders: Vec<String> = self
                .order_by
                .iter()
                .map(|(col, asc)| {
                    if *asc {
                        (*col).to_string()
                    } else {
                        format!("{} DESC", col)
                    }
                })
                .collect();
            sql.push_str(&orders.join(", "));
        }

        // LIMIT
        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        // OFFSET
        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        (sql, params)
    }

    /// Builds the query and returns only the SQL string.
    #[must_use]
    pub fn build_sql(self) -> String {
        let (sql, _) = self.build();
        sql
    }
}

/// Creates a type-safe column expression for use in WHERE clauses.
///
/// This function takes a column type and creates an expression builder
/// that references the column by its SQL name.
#[cfg(feature = "alloc")]
pub fn typed_col<C: Column>(_col: C) -> ExprBuilder {
    ExprBuilder::column(C::NAME)
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    // Note: Tests require the derive macro to be available,
    // which creates a circular dependency. Integration tests
    // should be used instead.
}
