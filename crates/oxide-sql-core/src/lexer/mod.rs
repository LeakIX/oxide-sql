//! SQL Lexer/Tokenizer
//!
//! This module provides a hand-written lexer for SQL that produces a stream of tokens.

mod span;
mod token;
mod tokenizer;

pub use span::Span;
pub use token::{Keyword, Token, TokenKind};
pub use tokenizer::Lexer;
