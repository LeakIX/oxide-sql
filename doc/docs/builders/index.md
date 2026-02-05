---
sidebar_position: 2
---

# SQL Builders

Oxide SQL provides type-safe builders for common SQL statements. Each builder
uses the typestate pattern to ensure valid SQL at compile time.

## Available Builders

- [SELECT](./select) - Query data from tables
- [INSERT](./insert) - Insert new rows
- [UPDATE](./update) - Modify existing rows
- [DELETE](./delete) - Remove rows

## Typed vs String-Based Builders

Oxide SQL offers two levels of type safety:

### String-Based Builders (This Section)

Use column names as strings. Good for dynamic queries or quick prototyping:

```rust
Select::new().columns(&["id", "name"]).from("users").build();
```

### Typed Builders (Recommended)

Use `#[derive(Table)]` for **compile-time column validation**. Invalid column
names won't compile:

```rust
use oxide_sql_derive::Table;
use oxide_sql_core::builder::{TypedSelect, TypedInsert, typed_col};

#[derive(Table)]
#[table(name = "users")]
struct User {
    #[column(primary_key)]
    id: i32,
    name: String,
}

// Columns are validated at compile time
TypedSelect::<UserTable, _, _>::new()
    .select::<(UserColumns::Id, UserColumns::Name)>()
    .from_table()
    .where_clause(typed_col(User::id()).eq(1))
    .build();

// This would NOT compile:
// .select::<(UserColumns::InvalidColumn,)>()  // Error!
```

See [Schema > Typed Queries](../schema/queries) for full typed builder
documentation.

## The Typestate Pattern

All builders use Rust's type system to enforce SQL validity at compile time.
This means:

1. **Required clauses are enforced** - You can't build a SELECT without FROM
2. **Order is enforced** - WHERE must come after FROM
3. **Invalid combinations fail to compile** - No runtime errors for SQL syntax

### Example

```rust
use oxide_sql_core::builder::Select;

// The type changes as you build the query:
let step1 = Select::new();           // Select<NoColumns, NoFrom>
let step2 = step1.columns(&["id"]);  // Select<HasColumns, NoFrom>
let step3 = step2.from("users");     // Select<HasColumns, HasFrom>

// Only step3 has the `build()` method available
let (sql, params) = step3.build();
```

## Expressions

All builders use the same expression system for WHERE clauses:

```rust
use oxide_sql_core::builder::col;

// Column reference
let expr = col("name");

// Comparisons
col("age").gt(18)
col("status").eq("active")
col("name").like("%john%")

// Null checks
col("deleted_at").is_null()
col("email").is_not_null()

// Range checks
col("age").between(18, 65)
col("status").in_list(&["active", "pending"])

// Logical operators
col("active").eq(true).and(col("age").gt(18))
col("status").eq("admin").or(col("status").eq("moderator"))
```

## Parameterized Queries

All values are automatically parameterized to prevent SQL injection:

```rust
use oxide_sql_core::builder::{Select, col};

let (sql, params) = Select::new()
    .columns(&["id"])
    .from("users")
    .where_clause(col("name").eq("Alice"))
    .build();

assert_eq!(sql, "SELECT id FROM users WHERE name = ?");
// params = [SqlValue::Text("Alice")]
```

The `params` vector should be passed to your database driver for safe execution.
