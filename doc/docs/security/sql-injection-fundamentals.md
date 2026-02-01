---
sidebar_position: 2
---

# SQL Injection Fundamentals

SQL injection is a code injection technique that exploits security
vulnerabilities in an application's database layer. Attackers insert malicious
SQL code through user input fields, which is then executed by the database.

## How SQL Injection Works

Consider a simple login form that constructs a query by concatenating user
input:

```sql
-- Vulnerable code (pseudocode)
query = "SELECT * FROM users WHERE username = '" + username + "' AND password = '" + password + "'"
```

If a user enters:

- Username: `admin`
- Password: `' OR '1'='1`

The resulting query becomes:

```sql
SELECT * FROM users WHERE username = 'admin' AND password = '' OR '1'='1'
```

Since `'1'='1'` is always true, this query returns all users, bypassing
authentication.

## Classic SQL Injection Examples

### Authentication Bypass

```sql
-- Normal query
SELECT * FROM users WHERE username = 'admin' AND password = 'password123'

-- Injected: username = admin'--
SELECT * FROM users WHERE username = 'admin'--' AND password = 'anything'
```

The `--` comments out the password check entirely.

### Data Extraction

```sql
-- Injected: id = 1 UNION SELECT username, password FROM users--
SELECT name, price FROM products WHERE id = 1 UNION SELECT username, password FROM users--
```

### Database Manipulation

```sql
-- Injected: id = 1; DROP TABLE users;--
SELECT * FROM products WHERE id = 1; DROP TABLE users;--
```

## Why Applications Are Vulnerable

1. **String Concatenation**: Building SQL queries by concatenating user input
2. **Insufficient Input Validation**: Trusting user input without sanitization
3. **Excessive Database Privileges**: Application accounts with admin access
4. **Error Message Exposure**: Revealing database structure in error messages

## Impact of SQL Injection

SQL injection can lead to:

- **Data Theft**: Stealing sensitive information (PII, credentials, financial
  data)
- **Authentication Bypass**: Logging in as any user, including administrators
- **Data Modification**: Altering or deleting critical data
- **Privilege Escalation**: Gaining administrative access to the database
- **Server Compromise**: Executing operating system commands via SQL

## How Oxide SQL Prevents This

Oxide SQL uses parameterized queries by default. User input is never
interpolated into SQL strings:

```rust
use oxide_sql_core::builder::{Select, col};

// User input is automatically parameterized
let user_input = "admin'--";
let (sql, params) = Select::new()
    .columns(&["id", "name"])
    .from("users")
    .where_clause(col("username").eq(user_input))
    .build();

// sql = "SELECT id, name FROM users WHERE username = ?"
// params = [SqlValue::Text("admin'--")]
```

The malicious input `admin'--` is treated as a literal string value, not as
SQL code.

## References

- [OWASP SQL Injection](https://owasp.org/www-community/attacks/SQL_Injection)
- [PortSwigger SQL Injection](https://portswigger.net/web-security/sql-injection)
