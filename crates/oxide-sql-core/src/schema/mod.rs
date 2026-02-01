//! Schema traits for type-safe table and column definitions.
//!
//! This module provides traits that are implemented by the `#[derive(Table)]`
//! macro to enable compile-time checked SQL queries.

/// Trait for table metadata.
///
/// Implemented by types generated from `#[derive(Table)]` to provide
/// table-level information.
pub trait Table {
    /// The row type (the original struct).
    type Row;

    /// The SQL table name.
    const NAME: &'static str;

    /// List of all column names.
    const COLUMNS: &'static [&'static str];

    /// The primary key column name, if any.
    const PRIMARY_KEY: Option<&'static str>;
}

/// Trait for column metadata.
///
/// Implemented by column types generated from `#[derive(Table)]` to provide
/// column-level information and enable type-safe queries.
pub trait Column {
    /// The table this column belongs to.
    type Table: Table;

    /// The Rust type of this column.
    type Type;

    /// The SQL column name.
    const NAME: &'static str;

    /// Whether this column is nullable.
    const NULLABLE: bool;

    /// Whether this column is the primary key.
    const PRIMARY_KEY: bool;
}

/// Marker trait for columns with a specific Rust type.
///
/// Used for compile-time type checking of values in queries.
pub trait TypedColumn<T>: Column<Type = T> {}

/// Trait for selecting specific columns from a table.
///
/// Implemented for tuples of column types to enable type-safe SELECT queries.
pub trait Selectable<T: Table> {
    /// Returns the column names to select.
    fn column_names() -> &'static [&'static str];
}

// Implement Selectable for single columns
impl<T: Table, C: Column<Table = T>> Selectable<T> for C {
    fn column_names() -> &'static [&'static str] {
        // Return as a slice containing just this column
        // This is a compile-time constant
        &[C::NAME]
    }
}

// Implement Selectable for tuples of columns (up to 12)
macro_rules! impl_selectable_tuple {
    ($($idx:tt: $col:ident),+) => {
        impl<T: Table, $($col: Column<Table = T>),+> Selectable<T> for ($($col,)+) {
            fn column_names() -> &'static [&'static str] {
                &[$($col::NAME),+]
            }
        }
    };
}

impl_selectable_tuple!(0: C0);
impl_selectable_tuple!(0: C0, 1: C1);
impl_selectable_tuple!(0: C0, 1: C1, 2: C2);
impl_selectable_tuple!(0: C0, 1: C1, 2: C2, 3: C3);
impl_selectable_tuple!(0: C0, 1: C1, 2: C2, 3: C3, 4: C4);
impl_selectable_tuple!(0: C0, 1: C1, 2: C2, 3: C3, 4: C4, 5: C5);
impl_selectable_tuple!(0: C0, 1: C1, 2: C2, 3: C3, 4: C4, 5: C5, 6: C6);
impl_selectable_tuple!(0: C0, 1: C1, 2: C2, 3: C3, 4: C4, 5: C5, 6: C6, 7: C7);
impl_selectable_tuple!(0: C0, 1: C1, 2: C2, 3: C3, 4: C4, 5: C5, 6: C6, 7: C7, 8: C8);
impl_selectable_tuple!(0: C0, 1: C1, 2: C2, 3: C3, 4: C4, 5: C5, 6: C6, 7: C7, 8: C8, 9: C9);
impl_selectable_tuple!(0: C0, 1: C1, 2: C2, 3: C3, 4: C4, 5: C5, 6: C6, 7: C7, 8: C8, 9: C9, 10: C10);
impl_selectable_tuple!(0: C0, 1: C1, 2: C2, 3: C3, 4: C4, 5: C5, 6: C6, 7: C7, 8: C8, 9: C9, 10: C10, 11: C11);
