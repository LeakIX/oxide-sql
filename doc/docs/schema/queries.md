---
sidebar_position: 2
---

# Typed Queries

Use `TypedSelect` for compile-time validated queries against your schema.

## Basic Query

```rust
use oxide_sql_derive::Table;
use oxide_sql_core::builder::TypedSelect;

#[derive(Table)]
#[table(name = "users")]
struct User {
    id: i32,
    name: String,
    email: String,
}

// Select specific columns
let (sql, params) = TypedSelect::<UserTable, _, _>::new()
    .select::<(UserColumns::Id, UserColumns::Name)>()
    .from_table()
    .build();

assert_eq!(sql, "SELECT id, name FROM users");
```

## Select All Columns

```rust
let (sql, _) = TypedSelect::<UserTable, _, _>::new()
    .select_all()
    .from_table()
    .build();

assert_eq!(sql, "SELECT id, name, email FROM users");
```

## Compile-Time Validation

The type system prevents invalid queries:

```rust
// This compiles - valid column
let query = TypedSelect::<UserTable, _, _>::new()
    .select::<UserColumns::Name>()
    .from_table();

// This would NOT compile - NonExistent is not a valid column
// let query = TypedSelect::<UserTable, _, _>::new()
//     .select::<UserColumns::NonExistent>()  // Error!
//     .from_table();
```

## WHERE Clause

Use `typed_col` for type-safe column references:

```rust
use oxide_sql_core::builder::{TypedSelect, typed_col};

let (sql, params) = TypedSelect::<UserTable, _, _>::new()
    .select_all()
    .from_table()
    .where_col(User::id(), typed_col(User::id()).eq(1_i32))
    .build();
```

## ORDER BY

```rust
let (sql, _) = TypedSelect::<UserTable, _, _>::new()
    .select_all()
    .from_table()
    .order_by(User::name(), true)  // ascending
    .order_by(User::id(), false)   // descending
    .build();

assert_eq!(sql, "SELECT id, name, email FROM users ORDER BY name, id DESC");
```

## LIMIT and OFFSET

```rust
let (sql, _) = TypedSelect::<UserTable, _, _>::new()
    .select_all()
    .from_table()
    .limit(10)
    .offset(20)
    .build();

assert_eq!(sql, "SELECT id, name, email FROM users LIMIT 10 OFFSET 20");
```

## Complete Example

```rust
use oxide_sql_derive::Table;
use oxide_sql_core::builder::{TypedSelect, typed_col};

#[derive(Table)]
#[table(name = "products")]
struct Product {
    #[column(primary_key)]
    id: i32,
    name: String,
    price: f64,
    category_id: i32,
    active: bool,
}

let (sql, params) = TypedSelect::<ProductTable, _, _>::new()
    .select::<(ProductColumns::Id, ProductColumns::Name, ProductColumns::Price)>()
    .from_table()
    .where_clause(
        typed_col(Product::active()).eq(true)
            .and(typed_col(Product::price()).lt(100.0_f64))
    )
    .order_by(Product::price(), true)
    .limit(20)
    .build();
```
