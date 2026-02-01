//! SQL values and parameter handling.
//!
//! This module provides safe handling of SQL values to prevent SQL injection.

/// A SQL value that can be used as a parameter.
///
/// All values are properly escaped or parameterized to prevent SQL injection.
#[derive(Debug, Clone, PartialEq)]
pub enum SqlValue {
    /// NULL value.
    Null,
    /// Boolean value.
    Bool(bool),
    /// Integer value.
    Int(i64),
    /// Float value.
    Float(f64),
    /// Text value.
    Text(String),
    /// Binary blob value.
    Blob(Vec<u8>),
}

impl SqlValue {
    /// Returns the SQL representation for inline use (escaped).
    ///
    /// **Warning**: Prefer using parameterized queries instead.
    #[must_use]
    pub fn to_sql_inline(&self) -> String {
        match self {
            Self::Null => String::from("NULL"),
            Self::Bool(b) => {
                if *b {
                    String::from("TRUE")
                } else {
                    String::from("FALSE")
                }
            }
            Self::Int(n) => format!("{n}"),
            Self::Float(f) => format!("{f}"),
            Self::Text(s) => {
                // Escape single quotes by doubling them
                let escaped = s.replace('\'', "''");
                format!("'{escaped}'")
            }
            Self::Blob(b) => {
                let hex: String = b.iter().map(|byte| format!("{byte:02X}")).collect();
                format!("X'{hex}'")
            }
        }
    }

    /// Returns the parameter placeholder.
    #[must_use]
    pub const fn placeholder() -> &'static str {
        "?"
    }
}

/// Trait for types that can be converted to SQL values.
pub trait ToSqlValue {
    /// Converts the value to a `SqlValue`.
    fn to_sql_value(self) -> SqlValue;
}

impl ToSqlValue for SqlValue {
    fn to_sql_value(self) -> SqlValue {
        self
    }
}

impl ToSqlValue for bool {
    fn to_sql_value(self) -> SqlValue {
        SqlValue::Bool(self)
    }
}

impl ToSqlValue for i64 {
    fn to_sql_value(self) -> SqlValue {
        SqlValue::Int(self)
    }
}

impl ToSqlValue for i32 {
    fn to_sql_value(self) -> SqlValue {
        SqlValue::Int(i64::from(self))
    }
}

impl ToSqlValue for i16 {
    fn to_sql_value(self) -> SqlValue {
        SqlValue::Int(i64::from(self))
    }
}

impl ToSqlValue for i8 {
    fn to_sql_value(self) -> SqlValue {
        SqlValue::Int(i64::from(self))
    }
}

impl ToSqlValue for u32 {
    fn to_sql_value(self) -> SqlValue {
        SqlValue::Int(i64::from(self))
    }
}

impl ToSqlValue for u16 {
    fn to_sql_value(self) -> SqlValue {
        SqlValue::Int(i64::from(self))
    }
}

impl ToSqlValue for u8 {
    fn to_sql_value(self) -> SqlValue {
        SqlValue::Int(i64::from(self))
    }
}

impl ToSqlValue for f64 {
    fn to_sql_value(self) -> SqlValue {
        SqlValue::Float(self)
    }
}

impl ToSqlValue for f32 {
    fn to_sql_value(self) -> SqlValue {
        SqlValue::Float(f64::from(self))
    }
}

impl ToSqlValue for String {
    fn to_sql_value(self) -> SqlValue {
        SqlValue::Text(self)
    }
}

impl ToSqlValue for &str {
    fn to_sql_value(self) -> SqlValue {
        SqlValue::Text(String::from(self))
    }
}

impl<T: ToSqlValue> ToSqlValue for Option<T> {
    fn to_sql_value(self) -> SqlValue {
        match self {
            Some(v) => v.to_sql_value(),
            None => SqlValue::Null,
        }
    }
}

impl ToSqlValue for Vec<u8> {
    fn to_sql_value(self) -> SqlValue {
        SqlValue::Blob(self)
    }
}

impl ToSqlValue for &[u8] {
    fn to_sql_value(self) -> SqlValue {
        SqlValue::Blob(self.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_value_inline_null() {
        assert_eq!(SqlValue::Null.to_sql_inline(), "NULL");
    }

    #[test]
    fn test_sql_value_inline_bool() {
        assert_eq!(SqlValue::Bool(true).to_sql_inline(), "TRUE");
        assert_eq!(SqlValue::Bool(false).to_sql_inline(), "FALSE");
    }

    #[test]
    fn test_sql_value_inline_int() {
        assert_eq!(SqlValue::Int(42).to_sql_inline(), "42");
        assert_eq!(SqlValue::Int(-100).to_sql_inline(), "-100");
    }

    #[test]
    fn test_sql_value_inline_text() {
        assert_eq!(
            SqlValue::Text(String::from("hello")).to_sql_inline(),
            "'hello'"
        );
    }

    #[test]
    fn test_sql_value_inline_text_escaping() {
        // Single quotes are escaped by doubling
        assert_eq!(
            SqlValue::Text(String::from("it's")).to_sql_inline(),
            "'it''s'"
        );
        assert_eq!(
            SqlValue::Text(String::from("O'Brien")).to_sql_inline(),
            "'O''Brien'"
        );
    }

    #[test]
    fn test_sql_injection_prevention() {
        // Attempt SQL injection
        let malicious = "'; DROP TABLE users; --";
        let value = SqlValue::Text(String::from(malicious));
        let escaped = value.to_sql_inline();
        // The single quote is escaped, preventing the injection
        assert_eq!(escaped, "'''; DROP TABLE users; --'");
    }

    #[test]
    fn test_sql_value_inline_blob() {
        assert_eq!(
            SqlValue::Blob(vec![0x48, 0x45, 0x4C, 0x4C, 0x4F]).to_sql_inline(),
            "X'48454C4C4F'"
        );
    }

    #[test]
    fn test_to_sql_value_conversions() {
        assert_eq!(true.to_sql_value(), SqlValue::Bool(true));
        assert_eq!(42_i32.to_sql_value(), SqlValue::Int(42));
        assert_eq!(2.5_f64.to_sql_value(), SqlValue::Float(2.5));
        assert_eq!(
            "hello".to_sql_value(),
            SqlValue::Text(String::from("hello"))
        );
        assert_eq!(None::<i32>.to_sql_value(), SqlValue::Null);
        assert_eq!(Some(42_i32).to_sql_value(), SqlValue::Int(42));
    }
}
