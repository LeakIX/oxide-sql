---
sidebar_position: 1
---

# SELECT Builder

The `Select` builder creates SELECT queries with compile-time validation.

## Basic Usage

```rust
use oxide_sql_core::builder::{Select, col};

let (sql, params) = Select::new()
    .columns(&["id", "name", "email"])
    .from("users")
    .build();

assert_eq!(sql, "SELECT id, name, email FROM users");
```

## Select All Columns

```rust
use oxide_sql_core::builder::Select;

let (sql, _) = Select::new()
    .all()
    .from("users")
    .build();

assert_eq!(sql, "SELECT * FROM users");
```

## WHERE Clause

```rust
use oxide_sql_core::builder::{Select, col};

let (sql, params) = Select::new()
    .columns(&["id", "name"])
    .from("users")
    .where_clause(col("active").eq(true))
    .build();

assert_eq!(sql, "SELECT id, name FROM users WHERE active = ?");
```

### Complex WHERE Conditions

```rust
use oxide_sql_core::builder::{Select, col};

let (sql, params) = Select::new()
    .columns(&["id"])
    .from("users")
    .where_clause(
        col("age").gte(18)
            .and(col("status").eq("active"))
            .and(col("country").in_list(&["US", "CA", "UK"]))
    )
    .build();
```

## ORDER BY

```rust
use oxide_sql_core::builder::Select;

let (sql, _) = Select::new()
    .columns(&["id", "name"])
    .from("users")
    .order_by("name", true)   // ascending
    .order_by("id", false)    // descending
    .build();

assert_eq!(sql, "SELECT id, name FROM users ORDER BY name, id DESC");
```

## LIMIT and OFFSET

```rust
use oxide_sql_core::builder::Select;

let (sql, _) = Select::new()
    .columns(&["id"])
    .from("users")
    .limit(10)
    .offset(20)
    .build();

assert_eq!(sql, "SELECT id FROM users LIMIT 10 OFFSET 20");
```

## DISTINCT

```rust
use oxide_sql_core::builder::Select;

let (sql, _) = Select::new()
    .distinct()
    .columns(&["country"])
    .from("users")
    .build();

assert_eq!(sql, "SELECT DISTINCT country FROM users");
```

## Complete Example

```rust
use oxide_sql_core::builder::{Select, col};

let (sql, params) = Select::new()
    .distinct()
    .columns(&["id", "name", "email"])
    .from("users")
    .where_clause(
        col("active").eq(true)
            .and(col("created_at").gte("2024-01-01"))
    )
    .order_by("created_at", false)
    .limit(50)
    .offset(0)
    .build();
```
