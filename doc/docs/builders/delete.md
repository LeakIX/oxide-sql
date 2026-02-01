---
sidebar_position: 4
---

# DELETE Builder

The `Delete` builder creates DELETE statements. Oxide SQL provides both a
standard builder and a safe variant that requires a WHERE clause.

## Basic Usage

```rust
use oxide_sql_core::builder::{Delete, col};

let (sql, params) = Delete::new()
    .from("users")
    .where_clause(col("id").eq(1_i32))
    .build();

assert_eq!(sql, "DELETE FROM users WHERE id = ?");
```

## Complex WHERE Conditions

```rust
use oxide_sql_core::builder::{Delete, col};

let (sql, params) = Delete::new()
    .from("sessions")
    .where_clause(
        col("expired_at").lt("2024-01-01")
            .or(col("user_id").is_null())
    )
    .build();
```

## SafeDelete Builder

The `SafeDelete` builder **requires** a WHERE clause at compile time,
preventing accidental deletion of all rows:

```rust
use oxide_sql_core::builder::{SafeDelete, col};

// This compiles - WHERE clause is required
let (sql, params) = SafeDelete::new()
    .from("users")
    .where_clause(col("id").eq(1_i32))
    .build();

// This would NOT compile - missing WHERE clause
// let query = SafeDelete::new()
//     .from("users")
//     .build();  // Error: method `build` not found
```

## When to Use Each Builder

| Builder | Use Case |
|---------|----------|
| `Delete` | When you might intentionally delete all rows (e.g., clearing temp tables) |
| `SafeDelete` | For most cases - prevents accidental data loss |

## SQL Injection Prevention

```rust
use oxide_sql_core::builder::{Delete, col};

let malicious = "1; DROP TABLE users; --";

let (sql, params) = Delete::new()
    .from("users")
    .where_clause(col("id").eq(malicious))
    .build();

// The SQL is safe - malicious input is parameterized
assert_eq!(sql, "DELETE FROM users WHERE id = ?");
```
