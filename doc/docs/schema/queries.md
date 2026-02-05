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

## TypedInsert

Type-safe INSERT with compile-time column validation:

```rust
use oxide_sql_derive::Table;
use oxide_sql_core::builder::TypedInsert;

#[derive(Table)]
#[table(name = "users")]
struct User {
    #[column(primary_key)]
    id: i32,
    name: String,
    email: String,
}

let (sql, params) = TypedInsert::<UserTable, _>::new()
    .set(User::name(), "Alice")
    .set(User::email(), "alice@example.com")
    .build();

assert_eq!(sql, "INSERT INTO users (name, email) VALUES (?, ?)");

// This would NOT compile - invalid column:
// TypedInsert::<UserTable, _>::new()
//     .set(User::invalid_column(), "value")  // Error!
```

## TypedUpdate

Type-safe UPDATE with compile-time validation of both SET and WHERE columns:

```rust
use oxide_sql_core::builder::{TypedUpdate, typed_col};

let (sql, params) = TypedUpdate::<UserTable, _>::new()
    .set(User::name(), "Bob")
    .set(User::email(), "bob@example.com")
    .where_clause(typed_col(User::id()).eq(1_i32))
    .build();

assert_eq!(sql, "UPDATE users SET name = ?, email = ? WHERE id = ?");
```

## TypedDelete

Type-safe DELETE with compile-time WHERE clause validation:

```rust
use oxide_sql_core::builder::{TypedDelete, typed_col};

let (sql, params) = TypedDelete::<UserTable>::new()
    .where_clause(typed_col(User::id()).eq(1_i32))
    .build();

assert_eq!(sql, "DELETE FROM users WHERE id = ?");

// Delete with complex conditions
let (sql, _) = TypedDelete::<UserTable>::new()
    .where_clause(
        typed_col(User::name()).eq("test")
            .and(typed_col(User::email()).like("%@test.com"))
    )
    .build();
```

## Complete Example

```rust
use oxide_sql_derive::Table;
use oxide_sql_core::builder::{TypedSelect, TypedInsert, TypedUpdate, TypedDelete, typed_col};

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

// SELECT
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

// INSERT
let (sql, _) = TypedInsert::<ProductTable, _>::new()
    .set(Product::name(), "Widget")
    .set(Product::price(), 29.99_f64)
    .set(Product::category_id(), 1_i32)
    .set(Product::active(), true)
    .build();

// UPDATE
let (sql, _) = TypedUpdate::<ProductTable, _>::new()
    .set(Product::price(), 24.99_f64)
    .set(Product::active(), false)
    .where_clause(typed_col(Product::id()).eq(1_i32))
    .build();

// DELETE
let (sql, _) = TypedDelete::<ProductTable>::new()
    .where_clause(typed_col(Product::active()).eq(false))
    .build();
```
