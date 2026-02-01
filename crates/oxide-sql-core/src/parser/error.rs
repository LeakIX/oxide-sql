//! Parser error types.

#[cfg(feature = "alloc")]
use alloc::string::String;

use crate::lexer::{Span, TokenKind};

/// A parse error.
#[derive(Debug, Clone, PartialEq)]
#[cfg(feature = "alloc")]
pub struct ParseError {
    /// The error message.
    pub message: String,
    /// The location of the error.
    pub span: Span,
    /// Expected tokens (if applicable).
    pub expected: Option<String>,
    /// The actual token found.
    pub found: Option<TokenKind>,
}

#[cfg(feature = "alloc")]
impl ParseError {
    /// Creates a new parse error.
    #[must_use]
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
            expected: None,
            found: None,
        }
    }

    /// Creates an "unexpected token" error.
    #[must_use]
    pub fn unexpected(expected: impl Into<String>, found: TokenKind, span: Span) -> Self {
        let expected_str: String = expected.into();
        Self {
            message: alloc::format!(
                "Unexpected token: expected {}, found {:?}",
                expected_str,
                found
            ),
            span,
            expected: Some(expected_str),
            found: Some(found),
        }
    }

    /// Creates an "unexpected end of input" error.
    #[must_use]
    pub fn unexpected_eof(expected: impl Into<String>, span: Span) -> Self {
        let expected_str: String = expected.into();
        Self {
            message: alloc::format!("Unexpected end of input: expected {}", expected_str),
            span,
            expected: Some(expected_str),
            found: Some(TokenKind::Eof),
        }
    }
}

#[cfg(feature = "alloc")]
impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{} at position {}..{}",
            self.message, self.span.start, self.span.end
        )
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ParseError {}
