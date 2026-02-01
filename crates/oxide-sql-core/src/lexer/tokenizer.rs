//! SQL Tokenizer implementation.

#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};

use super::{Keyword, Span, Token, TokenKind};

/// A lexer that tokenizes SQL input.
pub struct Lexer<'a> {
    /// The input source code.
    input: &'a str,
    /// The current byte position.
    pos: usize,
    /// The byte position of the start of the current token.
    start: usize,
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer for the given input.
    #[must_use]
    pub const fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            start: 0,
        }
    }

    /// Returns the current character without advancing.
    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    /// Returns the next character without advancing.
    fn peek_next(&self) -> Option<char> {
        let mut chars = self.input[self.pos..].chars();
        chars.next();
        chars.next()
    }

    /// Advances to the next character and returns it.
    fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    /// Skips whitespace and comments.
    fn skip_whitespace_and_comments(&mut self) {
        loop {
            // Skip whitespace
            while self.peek().is_some_and(|c| c.is_whitespace()) {
                self.advance();
            }

            // Skip single-line comments (-- ...)
            if self.peek() == Some('-') && self.peek_next() == Some('-') {
                self.advance(); // -
                self.advance(); // -
                while self.peek().is_some_and(|c| c != '\n') {
                    self.advance();
                }
                continue;
            }

            // Skip multi-line comments (/* ... */)
            if self.peek() == Some('/') && self.peek_next() == Some('*') {
                self.advance(); // /
                self.advance(); // *
                loop {
                    match self.advance() {
                        Some('*') if self.peek() == Some('/') => {
                            self.advance();
                            break;
                        }
                        None => break,
                        _ => {}
                    }
                }
                continue;
            }

            break;
        }
    }

    /// Creates a span from start to current position.
    fn make_span(&self) -> Span {
        Span::new(self.start, self.pos)
    }

    /// Creates a token with the current span.
    fn make_token(&self, kind: TokenKind) -> Token {
        Token::new(kind, self.make_span())
    }

    /// Scans an identifier or keyword.
    #[cfg(feature = "alloc")]
    fn scan_identifier(&mut self) -> Token {
        while self.peek().is_some_and(|c| c.is_alphanumeric() || c == '_') {
            self.advance();
        }

        let text = &self.input[self.start..self.pos];

        // Check if it's a keyword
        if let Some(keyword) = Keyword::from_str(text) {
            self.make_token(TokenKind::Keyword(keyword))
        } else {
            self.make_token(TokenKind::Identifier(String::from(text)))
        }
    }

    /// Scans a quoted identifier (e.g., "column name" or `column name`).
    #[cfg(feature = "alloc")]
    fn scan_quoted_identifier(&mut self, quote: char) -> Token {
        self.advance(); // consume opening quote
        let content_start = self.pos;

        loop {
            match self.peek() {
                Some(c) if c == quote => {
                    // Check for escaped quote (double quote)
                    if self.peek_next() == Some(quote) {
                        self.advance();
                        self.advance();
                    } else {
                        break;
                    }
                }
                Some(_) => {
                    self.advance();
                }
                None => {
                    return self.make_token(TokenKind::Error(String::from(
                        "Unterminated quoted identifier",
                    )));
                }
            }
        }

        let content = &self.input[content_start..self.pos];
        self.advance(); // consume closing quote

        // Handle escaped quotes
        let unescaped = content.replace(&format!("{quote}{quote}"), &quote.to_string());
        self.make_token(TokenKind::Identifier(unescaped))
    }

    /// Scans a number (integer or float).
    fn scan_number(&mut self) -> Token {
        let mut is_float = false;

        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.advance();
        }

        // Check for decimal point
        if self.peek() == Some('.') && self.peek_next().is_some_and(|c| c.is_ascii_digit()) {
            is_float = true;
            self.advance(); // consume .
            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.advance();
            }
        }

        // Check for exponent
        if self.peek().is_some_and(|c| c == 'e' || c == 'E') {
            is_float = true;
            self.advance(); // consume e/E
            if self.peek().is_some_and(|c| c == '+' || c == '-') {
                self.advance();
            }
            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.advance();
            }
        }

        let text = &self.input[self.start..self.pos];

        if is_float {
            match text.parse::<f64>() {
                Ok(f) => self.make_token(TokenKind::Float(f)),
                #[cfg(feature = "alloc")]
                Err(e) => self.make_token(TokenKind::Error(alloc::format!("Invalid float: {e}"))),
                #[cfg(not(feature = "alloc"))]
                Err(_) => self.make_token(TokenKind::Eof), // Fallback without alloc
            }
        } else {
            match text.parse::<i64>() {
                Ok(i) => self.make_token(TokenKind::Integer(i)),
                #[cfg(feature = "alloc")]
                Err(e) => self.make_token(TokenKind::Error(alloc::format!("Invalid integer: {e}"))),
                #[cfg(not(feature = "alloc"))]
                Err(_) => self.make_token(TokenKind::Eof),
            }
        }
    }

    /// Scans a string literal.
    #[cfg(feature = "alloc")]
    fn scan_string(&mut self, quote: char) -> Token {
        self.advance(); // consume opening quote
        let mut value = String::new();

        loop {
            match self.peek() {
                Some(c) if c == quote => {
                    // Check for escaped quote (double quote)
                    if self.peek_next() == Some(quote) {
                        value.push(quote);
                        self.advance();
                        self.advance();
                    } else {
                        break;
                    }
                }
                Some(c) => {
                    value.push(c);
                    self.advance();
                }
                None => {
                    return self.make_token(TokenKind::Error(String::from(
                        "Unterminated string literal",
                    )));
                }
            }
        }

        self.advance(); // consume closing quote
        self.make_token(TokenKind::String(value))
    }

    /// Scans a blob literal (X'...' or x'...').
    #[cfg(feature = "alloc")]
    fn scan_blob(&mut self) -> Token {
        self.advance(); // consume X/x
        if self.peek() != Some('\'') {
            return self.scan_identifier();
        }
        self.advance(); // consume opening quote

        let mut bytes = Vec::new();
        let mut hex_chars = String::new();

        loop {
            match self.peek() {
                Some('\'') => break,
                Some(c) if c.is_ascii_hexdigit() => {
                    hex_chars.push(c);
                    self.advance();

                    if hex_chars.len() == 2 {
                        if let Ok(byte) = u8::from_str_radix(&hex_chars, 16) {
                            bytes.push(byte);
                        }
                        hex_chars.clear();
                    }
                }
                Some(c) if c.is_whitespace() => {
                    self.advance();
                }
                Some(_) => {
                    return self.make_token(TokenKind::Error(String::from(
                        "Invalid character in blob literal",
                    )));
                }
                None => {
                    return self
                        .make_token(TokenKind::Error(String::from("Unterminated blob literal")));
                }
            }
        }

        if !hex_chars.is_empty() {
            return self.make_token(TokenKind::Error(String::from(
                "Odd number of hex digits in blob literal",
            )));
        }

        self.advance(); // consume closing quote
        self.make_token(TokenKind::Blob(bytes))
    }

    /// Scans the next token.
    #[must_use]
    #[cfg(feature = "alloc")]
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();
        self.start = self.pos;

        let c = match self.advance() {
            Some(c) => c,
            None => return self.make_token(TokenKind::Eof),
        };

        match c {
            // Single-character tokens
            '(' => self.make_token(TokenKind::LeftParen),
            ')' => self.make_token(TokenKind::RightParen),
            '[' => self.make_token(TokenKind::LeftBracket),
            ']' => self.make_token(TokenKind::RightBracket),
            ',' => self.make_token(TokenKind::Comma),
            ';' => self.make_token(TokenKind::Semicolon),
            '+' => self.make_token(TokenKind::Plus),
            '-' => self.make_token(TokenKind::Minus),
            '*' => self.make_token(TokenKind::Star),
            '/' => self.make_token(TokenKind::Slash),
            '%' => self.make_token(TokenKind::Percent),
            '~' => self.make_token(TokenKind::BitNot),
            '?' => self.make_token(TokenKind::Question),
            '@' => self.make_token(TokenKind::At),

            // Potentially multi-character tokens
            '.' => self.make_token(TokenKind::Dot),
            ':' => {
                if self.peek() == Some(':') {
                    self.advance();
                    self.make_token(TokenKind::DoubleColon)
                } else {
                    self.make_token(TokenKind::Colon)
                }
            }
            '=' => self.make_token(TokenKind::Eq),
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    self.make_token(TokenKind::LtEq)
                } else if self.peek() == Some('>') {
                    self.advance();
                    self.make_token(TokenKind::NotEq)
                } else if self.peek() == Some('<') {
                    self.advance();
                    self.make_token(TokenKind::LeftShift)
                } else {
                    self.make_token(TokenKind::Lt)
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    self.make_token(TokenKind::GtEq)
                } else if self.peek() == Some('>') {
                    self.advance();
                    self.make_token(TokenKind::RightShift)
                } else {
                    self.make_token(TokenKind::Gt)
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    self.make_token(TokenKind::NotEq)
                } else {
                    self.make_token(TokenKind::Error(String::from("Unexpected character: !")))
                }
            }
            '|' => {
                if self.peek() == Some('|') {
                    self.advance();
                    self.make_token(TokenKind::Concat)
                } else {
                    self.make_token(TokenKind::BitOr)
                }
            }
            '&' => self.make_token(TokenKind::BitAnd),

            // String literals
            '\'' => {
                self.pos = self.start; // Reset position to scan from quote
                self.scan_string('\'')
            }

            // Quoted identifiers
            '"' => {
                self.pos = self.start;
                self.scan_quoted_identifier('"')
            }
            '`' => {
                self.pos = self.start;
                self.scan_quoted_identifier('`')
            }

            // Blob literals
            'X' | 'x' if self.peek() == Some('\'') => {
                self.pos = self.start;
                self.scan_blob()
            }

            // Numbers
            c if c.is_ascii_digit() => {
                self.pos = self.start;
                self.scan_number()
            }

            // Identifiers and keywords
            c if c.is_alphabetic() || c == '_' => {
                self.pos = self.start;
                self.scan_identifier()
            }

            _ => self.make_token(TokenKind::Error(alloc::format!(
                "Unexpected character: {c}"
            ))),
        }
    }

    /// Tokenizes the entire input and returns all tokens.
    #[must_use]
    #[cfg(feature = "alloc")]
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            let is_eof = token.is_eof();
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        tokens
    }
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;

    fn tokenize(input: &str) -> Vec<Token> {
        Lexer::new(input).tokenize()
    }

    fn token_kinds(input: &str) -> Vec<TokenKind> {
        tokenize(input).into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn test_empty_input() {
        let tokens = tokenize("");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].kind, TokenKind::Eof));
    }

    #[test]
    fn test_whitespace_only() {
        let tokens = tokenize("   \n\t  ");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].kind, TokenKind::Eof));
    }

    #[test]
    fn test_single_line_comment() {
        let tokens = tokenize("SELECT -- this is a comment\nFROM");
        assert_eq!(
            token_kinds("SELECT -- comment\nFROM"),
            vec![
                TokenKind::Keyword(Keyword::Select),
                TokenKind::Keyword(Keyword::From),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_multi_line_comment() {
        let tokens = tokenize("SELECT /* comment */ FROM");
        assert_eq!(
            token_kinds("SELECT /* comment */ FROM"),
            vec![
                TokenKind::Keyword(Keyword::Select),
                TokenKind::Keyword(Keyword::From),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_keywords() {
        assert_eq!(
            token_kinds("SELECT FROM WHERE"),
            vec![
                TokenKind::Keyword(Keyword::Select),
                TokenKind::Keyword(Keyword::From),
                TokenKind::Keyword(Keyword::Where),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_keywords_case_insensitive() {
        assert_eq!(
            token_kinds("select FROM wHeRe"),
            vec![
                TokenKind::Keyword(Keyword::Select),
                TokenKind::Keyword(Keyword::From),
                TokenKind::Keyword(Keyword::Where),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_identifiers() {
        assert_eq!(
            token_kinds("foo bar_baz _qux"),
            vec![
                TokenKind::Identifier(String::from("foo")),
                TokenKind::Identifier(String::from("bar_baz")),
                TokenKind::Identifier(String::from("_qux")),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_quoted_identifiers() {
        assert_eq!(
            token_kinds("\"column name\" `another`"),
            vec![
                TokenKind::Identifier(String::from("column name")),
                TokenKind::Identifier(String::from("another")),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_integers() {
        assert_eq!(
            token_kinds("42 0 123456789"),
            vec![
                TokenKind::Integer(42),
                TokenKind::Integer(0),
                TokenKind::Integer(123_456_789),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_floats() {
        assert_eq!(
            token_kinds("3.14 0.5 1e10 2.5e-3"),
            vec![
                TokenKind::Float(3.14),
                TokenKind::Float(0.5),
                TokenKind::Float(1e10),
                TokenKind::Float(2.5e-3),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_strings() {
        assert_eq!(
            token_kinds("'hello' 'world'"),
            vec![
                TokenKind::String(String::from("hello")),
                TokenKind::String(String::from("world")),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_string_with_escaped_quote() {
        assert_eq!(
            token_kinds("'it''s'"),
            vec![TokenKind::String(String::from("it's")), TokenKind::Eof,]
        );
    }

    #[test]
    fn test_blob() {
        let tokens = tokenize("X'48454C4C4F'");
        assert_eq!(tokens.len(), 2);
        assert!(
            matches!(&tokens[0].kind, TokenKind::Blob(b) if b == &[0x48, 0x45, 0x4C, 0x4C, 0x4F])
        );
    }

    #[test]
    fn test_operators() {
        assert_eq!(
            token_kinds("+ - * / % = != <> < <= > >="),
            vec![
                TokenKind::Plus,
                TokenKind::Minus,
                TokenKind::Star,
                TokenKind::Slash,
                TokenKind::Percent,
                TokenKind::Eq,
                TokenKind::NotEq,
                TokenKind::NotEq,
                TokenKind::Lt,
                TokenKind::LtEq,
                TokenKind::Gt,
                TokenKind::GtEq,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_delimiters() {
        assert_eq!(
            token_kinds("( ) [ ] , ; . : ::"),
            vec![
                TokenKind::LeftParen,
                TokenKind::RightParen,
                TokenKind::LeftBracket,
                TokenKind::RightBracket,
                TokenKind::Comma,
                TokenKind::Semicolon,
                TokenKind::Dot,
                TokenKind::Colon,
                TokenKind::DoubleColon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_concat_operator() {
        assert_eq!(
            token_kinds("a || b"),
            vec![
                TokenKind::Identifier(String::from("a")),
                TokenKind::Concat,
                TokenKind::Identifier(String::from("b")),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_bitwise_operators() {
        assert_eq!(
            token_kinds("a & b | c ~ d << e >> f"),
            vec![
                TokenKind::Identifier(String::from("a")),
                TokenKind::BitAnd,
                TokenKind::Identifier(String::from("b")),
                TokenKind::BitOr,
                TokenKind::Identifier(String::from("c")),
                TokenKind::BitNot,
                TokenKind::Identifier(String::from("d")),
                TokenKind::LeftShift,
                TokenKind::Identifier(String::from("e")),
                TokenKind::RightShift,
                TokenKind::Identifier(String::from("f")),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_simple_select() {
        let sql = "SELECT id, name FROM users WHERE active = 1";
        assert_eq!(
            token_kinds(sql),
            vec![
                TokenKind::Keyword(Keyword::Select),
                TokenKind::Identifier(String::from("id")),
                TokenKind::Comma,
                TokenKind::Identifier(String::from("name")),
                TokenKind::Keyword(Keyword::From),
                TokenKind::Identifier(String::from("users")),
                TokenKind::Keyword(Keyword::Where),
                TokenKind::Identifier(String::from("active")),
                TokenKind::Eq,
                TokenKind::Integer(1),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_span_tracking() {
        let tokens = tokenize("SELECT id");
        assert_eq!(tokens[0].span, Span::new(0, 6));
        assert_eq!(tokens[1].span, Span::new(7, 9));
    }

    #[test]
    fn test_parameter_placeholder() {
        assert_eq!(
            token_kinds("? ?1 @param :param"),
            vec![
                TokenKind::Question,
                TokenKind::Question,
                TokenKind::Integer(1),
                TokenKind::At,
                TokenKind::Identifier(String::from("param")),
                TokenKind::Colon,
                TokenKind::Identifier(String::from("param")),
                TokenKind::Eof,
            ]
        );
    }
}
