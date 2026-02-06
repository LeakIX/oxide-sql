---
sidebar_position: 7
---

# NoSQL and ORM Injection

SQL injection techniques extend beyond traditional SQL databases to NoSQL
databases and Object-Relational Mappers (ORMs).

## NoSQL Injection

NoSQL databases like MongoDB are vulnerable to injection attacks through
operator manipulation.

### MongoDB Operator Injection

**Vulnerable authentication check:**

```javascript
// Vulnerable code
db.users.find({
  username: req.body.username,
  password: req.body.password,
});
```

**Malicious payload:**

```json
{
  "username": { "$ne": null },
  "password": { "$ne": null }
}
```

This matches any document where username and password exist, bypassing
authentication.

### JavaScript Injection in $where

```javascript
// Vulnerable query using $where
db.users.find({ $where: "this.username == '" + username + "'" });

// Malicious input: ' || '1'=='1
// Becomes: this.username == '' || '1'=='1'
```

### Prevention for NoSQL

```javascript
// Use mongo-sanitize library
const sanitize = require('mongo-sanitize');
const cleanUsername = sanitize(req.body.username);

// Enforce explicit operators
db.users.find({
  username: { $eq: username },
});

// Disable JavaScript execution
// mongod --noscripting
```

## ORM Injection

ORM layers can be vulnerable when developers bypass the ORM's protections.

### Vulnerable Patterns

**Rails ActiveRecord (vulnerable):**

```ruby
# DON'T DO THIS - vulnerable to injection
Orders.find_all(
  "customer_id = 123 AND order_date = '#{params[:order_date]}'"
)

# SAFE - parameterized
Orders.where(
  "customer_id = ? AND order_date = ?",
  customer_id, order_date
)
```

**Django ORM (vulnerable):**

```python
# DON'T DO THIS - vulnerable
User.objects.raw(f"SELECT * FROM users WHERE username = '{username}'")

# SAFE - ORM query
User.objects.filter(username=username)
```

**Hibernate HQL (vulnerable):**

```java
// DON'T DO THIS - vulnerable
session.createQuery("from Orders where id = " + orderId)

// SAFE - positional parameters
session.createQuery("from Orders where id = ?1")
       .setParameter(1, orderId)
```

**SQLAlchemy (vulnerable):**

```python
# DON'T DO THIS - vulnerable
session.execute(f"SELECT * FROM users WHERE id = {user_id}")

# SAFE - parameterized
session.execute(text("SELECT * FROM users WHERE id = :id"), {"id": user_id})
```

### Recent ORM Vulnerabilities (2020-2024)

| CVE | Framework | Issue |
| --- | --------- | ----- |
| CVE-2024-42005 | Django | JSON field SQL injection |
| CVE-2023-22794 | Rails ActiveRecord | Injection via `accepts_nested_attributes` |
| CVE-2020-25638 | Hibernate | Criteria API injection |

### Best Practices for ORMs

1. **Avoid raw SQL methods** when possible
2. **Keep ORM libraries updated** - security patches are released regularly
3. **Never concatenate user input** into queries
4. **Use parameterized queries** even within ORM raw methods
5. **Review generated SQL** during development

## Command Injection via SQL

Attackers can escalate SQL injection to execute operating system commands.

### xp_cmdshell (MS SQL Server)

```sql
-- Enable xp_cmdshell (requires admin)
EXEC sp_configure 'show advanced options', 1;
RECONFIGURE;
EXEC sp_configure 'xp_cmdshell', 1;
RECONFIGURE;

-- Execute commands
EXEC xp_cmdshell 'net user';
EXEC xp_cmdshell 'whoami';
```

### INTO OUTFILE (MySQL)

```sql
-- Write webshell to server
SELECT '<?php system($_GET["cmd"]); ?>'
INTO OUTFILE '/var/www/html/shell.php';

-- Read sensitive files
LOAD DATA INFILE '/etc/passwd' INTO TABLE temp_table;
```

### Prevention

- Disable `xp_cmdshell` when not needed
- Set `secure_file_priv` to restrict file operations in MySQL
- Run database services under limited-privilege accounts
- Never expose sysadmin credentials to web applications

## How Oxide SQL Helps

Oxide SQL prevents ORM-like injection vulnerabilities by enforcing
parameterization at the type level. The type system prevents unsafe string
concatenation, and there is no way to inject raw SQL -- the builder API only
accepts structured components. Unlike ORMs that provide "escape hatches" for
raw SQL, Oxide SQL's builder pattern makes injection impossible through its API
design.

See the [builder module rustdoc](pathname:///oxide-sql/rustdoc/oxide_sql_core/builder/) for
examples.

## References

- [Imperva - What Is NoSQL Injection](https://www.imperva.com/learn/application-security/nosql-injection/)
- [OWASP - Testing for ORM Injection](https://owasp.org/www-project-web-security-testing-guide/latest/4-Web_Application_Security_Testing/07-Input_Validation_Testing/05.7-Testing_for_ORM_Injection)
- [Propel Code - SQL Injection in ORMs 2025](https://www.propelcode.ai/blog/sql-injection-orm-vulnerabilities-modern-frameworks-2025)
