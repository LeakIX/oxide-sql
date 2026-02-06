---
sidebar_position: 5
---

# SQL Injection Prevention Techniques

This page covers the primary defense strategies recommended by OWASP and
security experts.

## 1. Parameterized Queries (Prepared Statements)

**The most effective defense against SQL injection.**

Parameterized queries ensure user input is treated as data, never as executable
SQL code.

### How They Work

1. The SQL statement structure is defined with placeholders
2. User input is bound to placeholders separately
3. The database treats input as literal values, not SQL code

### Examples by Language

**Java (JDBC):**

```java
String query = "SELECT * FROM users WHERE username = ? AND password = ?";
PreparedStatement stmt = connection.prepareStatement(query);
stmt.setString(1, username);
stmt.setString(2, password);
ResultSet results = stmt.executeQuery();
```

**Python (SQLite/psycopg2):**

```python
# SQLite uses ? placeholders
cursor.execute(
    "SELECT * FROM users WHERE username = ? AND password = ?",
    (username, password)
)

# PostgreSQL uses %s placeholders
cursor.execute(
    "SELECT * FROM users WHERE username = %s AND password = %s",
    (username, password)
)
```

**PHP (PDO):**

```php
$stmt = $pdo->prepare("SELECT * FROM users WHERE username = ?");
$stmt->execute([$username]);

// With named parameters
$stmt = $pdo->prepare("SELECT * FROM users WHERE username = :username");
$stmt->execute(['username' => $username]);
```

**Rust (with Oxide SQL):**

See the [builder module rustdoc](pathname:///oxide-sql/rustdoc/oxide_sql_core/builder/) for
parameterized query examples.

## 2. Stored Procedures

Stored procedures can provide protection when implemented correctly.

### Safe Implementation

```sql
CREATE PROCEDURE GetUserByUsername
    @Username NVARCHAR(50)
AS
BEGIN
    SELECT * FROM Users WHERE Username = @Username
END
```

### Unsafe Implementation (Still Vulnerable!)

```sql
-- DON'T DO THIS - still vulnerable to injection
CREATE PROCEDURE GetUserByUsername
    @Username NVARCHAR(50)
AS
BEGIN
    EXEC('SELECT * FROM Users WHERE Username = ''' + @Username + '''')
END
```

### Best Practices

- Use `sp_executesql` instead of `EXEC` for dynamic SQL in SQL Server
- Never concatenate parameters into SQL strings
- Validate input within the procedure
- Apply least privilege to procedure execution

## 3. Input Validation

**Input validation should be a secondary defense, not the primary one.**

### Allowlist Validation

```python
import re

def validate_username(username):
    # Accept only alphanumeric characters and underscores
    if not re.match(r'^[a-zA-Z0-9_]{3,20}$', username):
        raise ValueError("Invalid username format")
    return username

def validate_id(user_id):
    # Force to integer
    return int(user_id)
```

### Type Enforcement

```javascript
function sanitizeInput(input) {
  // Force to string and remove dangerous characters
  return String(input).replace(/['"\\;]/g, '');
}
```

<!-- prettier-ignore-start -->
:::warning
Input validation alone is NOT sufficient. Always use parameterized queries as
the primary defense.
:::
<!-- prettier-ignore-end -->

## 4. Least Privilege Principle

Configure database accounts with minimal necessary permissions.

```sql
-- Create a read-only user for web application
CREATE USER 'webapp_readonly'@'localhost' IDENTIFIED BY 'secure_password';
GRANT SELECT ON mydb.products TO 'webapp_readonly'@'localhost';
GRANT SELECT ON mydb.categories TO 'webapp_readonly'@'localhost';

-- Create a user with limited write access
CREATE USER 'webapp_writer'@'localhost' IDENTIFIED BY 'secure_password';
GRANT SELECT, INSERT, UPDATE ON mydb.orders TO 'webapp_writer'@'localhost';
-- No DELETE, DROP, or ALTER privileges
```

### Key Principles

- Never use admin accounts (`sa`, `root`) for web applications
- Separate read and write accounts where possible
- Restrict access to specific tables needed
- Never grant `FILE`, `PROCESS`, or `SUPER` privileges

## 5. Web Application Firewalls (WAFs)

WAFs provide a defense-in-depth layer by blocking known attack patterns.

### How WAFs Help

- Pattern matching against SQL keywords
- Anomaly detection for unusual requests
- Signature database for known exploits

### Limitations

WAFs should NOT be your only defense:

- Known bypass techniques exist
- False positives can block legitimate traffic
- New attack patterns may not be recognized

## 6. Error Handling

Never expose database errors to users.

```python
try:
    cursor.execute(query, params)
except DatabaseError as e:
    # Log the detailed error internally
    logger.error(f"Database error: {e}")
    # Return generic message to user
    return "An error occurred. Please try again later."
```

## OWASP Recommended Priority

1. **Parameterized Queries** - Primary defense
2. **Stored Procedures** - When implemented safely
3. **Allowlist Input Validation** - For untrusted input
4. **Escaping User Input** - Last resort

## References

- [OWASP SQL Injection Prevention Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/SQL_Injection_Prevention_Cheat_Sheet.html)
- [OWASP Query Parameterization Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Query_Parameterization_Cheat_Sheet.html)
