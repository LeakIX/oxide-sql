---
sidebar_position: 3
---

# UPDATE Builder

The `Update` builder creates UPDATE statements with type-safe expressions.

## Basic Usage

```rust
use oxide_sql_core::builder::{Update, col};

let (sql, params) = Update::new()
    .table("users")
    .set("name", "Alice")
    .set("updated_at", "2024-01-15")
    .where_clause(col("id").eq(1_i32))
    .build();

assert_eq!(
    sql,
    "UPDATE users SET name = ?, updated_at = ? WHERE id = ?"
);
```

## Multiple SET Clauses

```rust
use oxide_sql_core::builder::{Update, col};

let (sql, params) = Update::new()
    .table("products")
    .set("price", 29.99_f64)
    .set("quantity", 50_i32)
    .set("updated_at", "2024-01-15")
    .where_clause(col("id").eq(123_i32))
    .build();
```

## Complex WHERE Conditions

```rust
use oxide_sql_core::builder::{Update, col};

let (sql, params) = Update::new()
    .table("orders")
    .set("status", "cancelled")
    .where_clause(
        col("status").eq("pending")
            .and(col("created_at").lt("2024-01-01"))
    )
    .build();
```

## Safety Warning

**Always use a WHERE clause with UPDATE** to avoid updating all rows.
Consider using the expression builder to construct precise conditions:

```rust
use oxide_sql_core::builder::{Update, col};

// Good: Specific WHERE clause
let (sql, _) = Update::new()
    .table("users")
    .set("active", false)
    .where_clause(col("id").eq(specific_user_id))
    .build();

// Dangerous: No WHERE clause updates ALL rows!
// let (sql, _) = Update::new()
//     .table("users")
//     .set("active", false)
//     .build();  // This updates every user!
```
