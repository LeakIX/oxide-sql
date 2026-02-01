//! Token types for the SQL lexer.

#[cfg(feature = "alloc")]
use alloc::string::String;

use super::Span;

/// SQL keywords.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    // Data Query Language (DQL)
    Select,
    From,
    Where,
    Order,
    By,
    Group,
    Having,
    Limit,
    Offset,
    Distinct,
    All,

    // Joins
    Join,
    Inner,
    Left,
    Right,
    Full,
    Outer,
    Cross,
    On,
    Using,

    // Set operations
    Union,
    Intersect,
    Except,

    // Data Manipulation Language (DML)
    Insert,
    Into,
    Values,
    Update,
    Set,
    Delete,

    // Data Definition Language (DDL)
    Create,
    Drop,
    Alter,
    Table,
    Index,
    View,
    Database,
    Schema,
    Trigger,

    // Constraints
    Primary,
    Key,
    Foreign,
    References,
    Unique,
    Check,
    Default,
    Constraint,
    Cascade,
    Restrict,

    // Logical operators
    And,
    Or,
    Not,
    In,
    Between,
    Like,
    Is,
    Null,
    True,
    False,
    Exists,

    // Ordering
    Asc,
    Desc,
    Nulls,
    First,
    Last,

    // Aggregates
    Count,
    Sum,
    Avg,
    Min,
    Max,

    // Data types
    Int,
    Integer,
    Smallint,
    Bigint,
    Real,
    Double,
    Float,
    Decimal,
    Numeric,
    Char,
    Varchar,
    Text,
    Blob,
    Boolean,
    Date,
    Time,
    Timestamp,
    Datetime,

    // SQLite specific
    Autoincrement,
    If,
    Temporary,
    Temp,
    Conflict,
    Replace,
    Abort,
    Rollback,
    Fail,
    Ignore,

    // Common clauses
    As,
    Case,
    When,
    Then,
    Else,
    End,
    Cast,
    Coalesce,
    Nullif,

    // Transaction
    Begin,
    Commit,
    Transaction,

    // Misc
    With,
    Recursive,
    Over,
    Partition,
    Window,
    Rows,
    Range,
    Unbounded,
    Preceding,
    Following,
    Current,
    Row,
}

impl Keyword {
    /// Attempts to parse a keyword from a string (case-insensitive).
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        // Convert to uppercase for comparison
        match s.to_ascii_uppercase().as_str() {
            "SELECT" => Some(Self::Select),
            "FROM" => Some(Self::From),
            "WHERE" => Some(Self::Where),
            "ORDER" => Some(Self::Order),
            "BY" => Some(Self::By),
            "GROUP" => Some(Self::Group),
            "HAVING" => Some(Self::Having),
            "LIMIT" => Some(Self::Limit),
            "OFFSET" => Some(Self::Offset),
            "DISTINCT" => Some(Self::Distinct),
            "ALL" => Some(Self::All),
            "JOIN" => Some(Self::Join),
            "INNER" => Some(Self::Inner),
            "LEFT" => Some(Self::Left),
            "RIGHT" => Some(Self::Right),
            "FULL" => Some(Self::Full),
            "OUTER" => Some(Self::Outer),
            "CROSS" => Some(Self::Cross),
            "ON" => Some(Self::On),
            "USING" => Some(Self::Using),
            "UNION" => Some(Self::Union),
            "INTERSECT" => Some(Self::Intersect),
            "EXCEPT" => Some(Self::Except),
            "INSERT" => Some(Self::Insert),
            "INTO" => Some(Self::Into),
            "VALUES" => Some(Self::Values),
            "UPDATE" => Some(Self::Update),
            "SET" => Some(Self::Set),
            "DELETE" => Some(Self::Delete),
            "CREATE" => Some(Self::Create),
            "DROP" => Some(Self::Drop),
            "ALTER" => Some(Self::Alter),
            "TABLE" => Some(Self::Table),
            "INDEX" => Some(Self::Index),
            "VIEW" => Some(Self::View),
            "DATABASE" => Some(Self::Database),
            "SCHEMA" => Some(Self::Schema),
            "TRIGGER" => Some(Self::Trigger),
            "PRIMARY" => Some(Self::Primary),
            "KEY" => Some(Self::Key),
            "FOREIGN" => Some(Self::Foreign),
            "REFERENCES" => Some(Self::References),
            "UNIQUE" => Some(Self::Unique),
            "CHECK" => Some(Self::Check),
            "DEFAULT" => Some(Self::Default),
            "CONSTRAINT" => Some(Self::Constraint),
            "CASCADE" => Some(Self::Cascade),
            "RESTRICT" => Some(Self::Restrict),
            "AND" => Some(Self::And),
            "OR" => Some(Self::Or),
            "NOT" => Some(Self::Not),
            "IN" => Some(Self::In),
            "BETWEEN" => Some(Self::Between),
            "LIKE" => Some(Self::Like),
            "IS" => Some(Self::Is),
            "NULL" => Some(Self::Null),
            "TRUE" => Some(Self::True),
            "FALSE" => Some(Self::False),
            "EXISTS" => Some(Self::Exists),
            "ASC" => Some(Self::Asc),
            "DESC" => Some(Self::Desc),
            "NULLS" => Some(Self::Nulls),
            "FIRST" => Some(Self::First),
            "LAST" => Some(Self::Last),
            "COUNT" => Some(Self::Count),
            "SUM" => Some(Self::Sum),
            "AVG" => Some(Self::Avg),
            "MIN" => Some(Self::Min),
            "MAX" => Some(Self::Max),
            "INT" => Some(Self::Int),
            "INTEGER" => Some(Self::Integer),
            "SMALLINT" => Some(Self::Smallint),
            "BIGINT" => Some(Self::Bigint),
            "REAL" => Some(Self::Real),
            "DOUBLE" => Some(Self::Double),
            "FLOAT" => Some(Self::Float),
            "DECIMAL" => Some(Self::Decimal),
            "NUMERIC" => Some(Self::Numeric),
            "CHAR" => Some(Self::Char),
            "VARCHAR" => Some(Self::Varchar),
            "TEXT" => Some(Self::Text),
            "BLOB" => Some(Self::Blob),
            "BOOLEAN" => Some(Self::Boolean),
            "DATE" => Some(Self::Date),
            "TIME" => Some(Self::Time),
            "TIMESTAMP" => Some(Self::Timestamp),
            "DATETIME" => Some(Self::Datetime),
            "AUTOINCREMENT" => Some(Self::Autoincrement),
            "IF" => Some(Self::If),
            "TEMPORARY" => Some(Self::Temporary),
            "TEMP" => Some(Self::Temp),
            "CONFLICT" => Some(Self::Conflict),
            "REPLACE" => Some(Self::Replace),
            "ABORT" => Some(Self::Abort),
            "ROLLBACK" => Some(Self::Rollback),
            "FAIL" => Some(Self::Fail),
            "IGNORE" => Some(Self::Ignore),
            "AS" => Some(Self::As),
            "CASE" => Some(Self::Case),
            "WHEN" => Some(Self::When),
            "THEN" => Some(Self::Then),
            "ELSE" => Some(Self::Else),
            "END" => Some(Self::End),
            "CAST" => Some(Self::Cast),
            "COALESCE" => Some(Self::Coalesce),
            "NULLIF" => Some(Self::Nullif),
            "BEGIN" => Some(Self::Begin),
            "COMMIT" => Some(Self::Commit),
            "TRANSACTION" => Some(Self::Transaction),
            "WITH" => Some(Self::With),
            "RECURSIVE" => Some(Self::Recursive),
            "OVER" => Some(Self::Over),
            "PARTITION" => Some(Self::Partition),
            "WINDOW" => Some(Self::Window),
            "ROWS" => Some(Self::Rows),
            "RANGE" => Some(Self::Range),
            "UNBOUNDED" => Some(Self::Unbounded),
            "PRECEDING" => Some(Self::Preceding),
            "FOLLOWING" => Some(Self::Following),
            "CURRENT" => Some(Self::Current),
            "ROW" => Some(Self::Row),
            _ => None,
        }
    }

    /// Returns the keyword as a string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Select => "SELECT",
            Self::From => "FROM",
            Self::Where => "WHERE",
            Self::Order => "ORDER",
            Self::By => "BY",
            Self::Group => "GROUP",
            Self::Having => "HAVING",
            Self::Limit => "LIMIT",
            Self::Offset => "OFFSET",
            Self::Distinct => "DISTINCT",
            Self::All => "ALL",
            Self::Join => "JOIN",
            Self::Inner => "INNER",
            Self::Left => "LEFT",
            Self::Right => "RIGHT",
            Self::Full => "FULL",
            Self::Outer => "OUTER",
            Self::Cross => "CROSS",
            Self::On => "ON",
            Self::Using => "USING",
            Self::Union => "UNION",
            Self::Intersect => "INTERSECT",
            Self::Except => "EXCEPT",
            Self::Insert => "INSERT",
            Self::Into => "INTO",
            Self::Values => "VALUES",
            Self::Update => "UPDATE",
            Self::Set => "SET",
            Self::Delete => "DELETE",
            Self::Create => "CREATE",
            Self::Drop => "DROP",
            Self::Alter => "ALTER",
            Self::Table => "TABLE",
            Self::Index => "INDEX",
            Self::View => "VIEW",
            Self::Database => "DATABASE",
            Self::Schema => "SCHEMA",
            Self::Trigger => "TRIGGER",
            Self::Primary => "PRIMARY",
            Self::Key => "KEY",
            Self::Foreign => "FOREIGN",
            Self::References => "REFERENCES",
            Self::Unique => "UNIQUE",
            Self::Check => "CHECK",
            Self::Default => "DEFAULT",
            Self::Constraint => "CONSTRAINT",
            Self::Cascade => "CASCADE",
            Self::Restrict => "RESTRICT",
            Self::And => "AND",
            Self::Or => "OR",
            Self::Not => "NOT",
            Self::In => "IN",
            Self::Between => "BETWEEN",
            Self::Like => "LIKE",
            Self::Is => "IS",
            Self::Null => "NULL",
            Self::True => "TRUE",
            Self::False => "FALSE",
            Self::Exists => "EXISTS",
            Self::Asc => "ASC",
            Self::Desc => "DESC",
            Self::Nulls => "NULLS",
            Self::First => "FIRST",
            Self::Last => "LAST",
            Self::Count => "COUNT",
            Self::Sum => "SUM",
            Self::Avg => "AVG",
            Self::Min => "MIN",
            Self::Max => "MAX",
            Self::Int => "INT",
            Self::Integer => "INTEGER",
            Self::Smallint => "SMALLINT",
            Self::Bigint => "BIGINT",
            Self::Real => "REAL",
            Self::Double => "DOUBLE",
            Self::Float => "FLOAT",
            Self::Decimal => "DECIMAL",
            Self::Numeric => "NUMERIC",
            Self::Char => "CHAR",
            Self::Varchar => "VARCHAR",
            Self::Text => "TEXT",
            Self::Blob => "BLOB",
            Self::Boolean => "BOOLEAN",
            Self::Date => "DATE",
            Self::Time => "TIME",
            Self::Timestamp => "TIMESTAMP",
            Self::Datetime => "DATETIME",
            Self::Autoincrement => "AUTOINCREMENT",
            Self::If => "IF",
            Self::Temporary => "TEMPORARY",
            Self::Temp => "TEMP",
            Self::Conflict => "CONFLICT",
            Self::Replace => "REPLACE",
            Self::Abort => "ABORT",
            Self::Rollback => "ROLLBACK",
            Self::Fail => "FAIL",
            Self::Ignore => "IGNORE",
            Self::As => "AS",
            Self::Case => "CASE",
            Self::When => "WHEN",
            Self::Then => "THEN",
            Self::Else => "ELSE",
            Self::End => "END",
            Self::Cast => "CAST",
            Self::Coalesce => "COALESCE",
            Self::Nullif => "NULLIF",
            Self::Begin => "BEGIN",
            Self::Commit => "COMMIT",
            Self::Transaction => "TRANSACTION",
            Self::With => "WITH",
            Self::Recursive => "RECURSIVE",
            Self::Over => "OVER",
            Self::Partition => "PARTITION",
            Self::Window => "WINDOW",
            Self::Rows => "ROWS",
            Self::Range => "RANGE",
            Self::Unbounded => "UNBOUNDED",
            Self::Preceding => "PRECEDING",
            Self::Following => "FOLLOWING",
            Self::Current => "CURRENT",
            Self::Row => "ROW",
        }
    }
}

/// The kind of token.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    /// Integer literal (e.g., 42)
    Integer(i64),
    /// Float literal (e.g., 3.14)
    Float(f64),
    /// String literal (e.g., 'hello')
    #[cfg(feature = "alloc")]
    String(String),
    /// Blob literal (e.g., X'1234')
    #[cfg(feature = "alloc")]
    Blob(alloc::vec::Vec<u8>),

    // Identifiers and keywords
    /// Identifier (e.g., column_name)
    #[cfg(feature = "alloc")]
    Identifier(String),
    /// SQL keyword
    Keyword(Keyword),

    // Operators
    /// +
    Plus,
    /// -
    Minus,
    /// *
    Star,
    /// /
    Slash,
    /// %
    Percent,
    /// =
    Eq,
    /// != or <>
    NotEq,
    /// <
    Lt,
    /// <=
    LtEq,
    /// >
    Gt,
    /// >=
    GtEq,
    /// ||
    Concat,
    /// &
    BitAnd,
    /// |
    BitOr,
    /// ~
    BitNot,
    /// <<
    LeftShift,
    /// >>
    RightShift,

    // Delimiters
    /// (
    LeftParen,
    /// )
    RightParen,
    /// [
    LeftBracket,
    /// ]
    RightBracket,
    /// ,
    Comma,
    /// ;
    Semicolon,
    /// .
    Dot,
    /// :
    Colon,
    /// ::
    DoubleColon,
    /// ?
    Question,
    /// @
    At,

    // Special
    /// End of input
    Eof,
    /// Invalid/unknown token
    #[cfg(feature = "alloc")]
    Error(String),
}

/// A token with its span in the source code.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// The kind of token.
    pub kind: TokenKind,
    /// The location in the source code.
    pub span: Span,
}

impl Token {
    /// Creates a new token.
    #[must_use]
    pub const fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Returns true if this is an EOF token.
    #[must_use]
    pub const fn is_eof(&self) -> bool {
        matches!(self.kind, TokenKind::Eof)
    }

    /// Returns true if this is a keyword.
    #[must_use]
    pub const fn is_keyword(&self) -> bool {
        matches!(self.kind, TokenKind::Keyword(_))
    }

    /// Returns the keyword if this is a keyword token.
    #[must_use]
    pub const fn as_keyword(&self) -> Option<Keyword> {
        match &self.kind {
            TokenKind::Keyword(kw) => Some(*kw),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_from_str() {
        assert_eq!(Keyword::from_str("SELECT"), Some(Keyword::Select));
        assert_eq!(Keyword::from_str("select"), Some(Keyword::Select));
        assert_eq!(Keyword::from_str("SeLeCt"), Some(Keyword::Select));
        assert_eq!(Keyword::from_str("not_a_keyword"), None);
    }

    #[test]
    fn test_keyword_as_str() {
        assert_eq!(Keyword::Select.as_str(), "SELECT");
        assert_eq!(Keyword::From.as_str(), "FROM");
        assert_eq!(Keyword::Where.as_str(), "WHERE");
    }

    #[test]
    fn test_token_is_eof() {
        let eof = Token::new(TokenKind::Eof, Span::new(0, 0));
        let select = Token::new(TokenKind::Keyword(Keyword::Select), Span::new(0, 6));
        assert!(eof.is_eof());
        assert!(!select.is_eof());
    }

    #[test]
    fn test_token_as_keyword() {
        let select = Token::new(TokenKind::Keyword(Keyword::Select), Span::new(0, 6));
        let plus = Token::new(TokenKind::Plus, Span::new(0, 1));
        assert_eq!(select.as_keyword(), Some(Keyword::Select));
        assert_eq!(plus.as_keyword(), None);
    }
}
