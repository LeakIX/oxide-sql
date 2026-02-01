//! Pratt expression parser for operator precedence.

use crate::ast::{BinaryOp, UnaryOp};
use crate::lexer::{Keyword, TokenKind};

/// Returns the prefix binding power for a token.
///
/// Returns `None` if the token cannot start an expression.
#[must_use]
pub const fn prefix_binding_power(kind: &TokenKind) -> Option<u8> {
    match kind {
        // Unary minus
        TokenKind::Minus => Some(15),
        // Bitwise NOT
        TokenKind::BitNot => Some(15),
        // NOT keyword
        TokenKind::Keyword(Keyword::Not) => Some(3),
        // Primary expressions (literals, identifiers, etc.)
        TokenKind::Integer(_)
        | TokenKind::Float(_)
        | TokenKind::LeftParen
        | TokenKind::Question
        | TokenKind::Colon => Some(0),
        #[cfg(feature = "alloc")]
        TokenKind::String(_) | TokenKind::Blob(_) | TokenKind::Identifier(_) | TokenKind::Star => {
            Some(0)
        }
        // Keywords that can start expressions
        TokenKind::Keyword(
            Keyword::Null
            | Keyword::True
            | Keyword::False
            | Keyword::Case
            | Keyword::Cast
            | Keyword::Exists
            | Keyword::Count
            | Keyword::Sum
            | Keyword::Avg
            | Keyword::Min
            | Keyword::Max
            | Keyword::Coalesce
            | Keyword::Nullif,
        ) => Some(0),
        _ => None,
    }
}

/// Returns the infix binding power for a token.
///
/// Returns `(left_bp, right_bp)` where:
/// - Higher binding power = binds tighter
/// - Left associative: left_bp < right_bp
/// - Right associative: left_bp > right_bp
///
/// Returns `None` if the token is not an infix operator.
#[must_use]
pub const fn infix_binding_power(kind: &TokenKind) -> Option<(u8, u8)> {
    match kind {
        // Logical OR (lowest precedence)
        TokenKind::Keyword(Keyword::Or) => Some((1, 2)),

        // Logical AND
        TokenKind::Keyword(Keyword::And) => Some((3, 4)),

        // Comparison operators
        TokenKind::Eq
        | TokenKind::NotEq
        | TokenKind::Lt
        | TokenKind::LtEq
        | TokenKind::Gt
        | TokenKind::GtEq => Some((5, 6)),

        // IS, IN, BETWEEN, LIKE
        TokenKind::Keyword(Keyword::Is | Keyword::In | Keyword::Between | Keyword::Like) => {
            Some((5, 6))
        }

        // Bitwise OR
        TokenKind::BitOr => Some((7, 8)),

        // Bitwise AND
        TokenKind::BitAnd => Some((9, 10)),

        // Bit shifts
        TokenKind::LeftShift | TokenKind::RightShift => Some((11, 12)),

        // Additive (string concat has same precedence as addition)
        TokenKind::Plus | TokenKind::Minus | TokenKind::Concat => Some((13, 14)),

        // Multiplicative
        TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Some((15, 16)),

        _ => None,
    }
}

/// Returns the postfix binding power for a token.
///
/// Returns `None` if the token is not a postfix operator.
#[must_use]
#[allow(dead_code)]
pub const fn postfix_binding_power(kind: &TokenKind) -> Option<u8> {
    match kind {
        // IS NULL, IS NOT NULL
        TokenKind::Keyword(Keyword::Is) => Some(17),
        // COLLATE
        TokenKind::Keyword(Keyword::As) => Some(17),
        _ => None,
    }
}

/// Converts a token to a binary operator.
#[must_use]
pub const fn token_to_binary_op(kind: &TokenKind) -> Option<BinaryOp> {
    match kind {
        TokenKind::Plus => Some(BinaryOp::Add),
        TokenKind::Minus => Some(BinaryOp::Sub),
        TokenKind::Star => Some(BinaryOp::Mul),
        TokenKind::Slash => Some(BinaryOp::Div),
        TokenKind::Percent => Some(BinaryOp::Mod),
        TokenKind::Eq => Some(BinaryOp::Eq),
        TokenKind::NotEq => Some(BinaryOp::NotEq),
        TokenKind::Lt => Some(BinaryOp::Lt),
        TokenKind::LtEq => Some(BinaryOp::LtEq),
        TokenKind::Gt => Some(BinaryOp::Gt),
        TokenKind::GtEq => Some(BinaryOp::GtEq),
        TokenKind::Keyword(Keyword::And) => Some(BinaryOp::And),
        TokenKind::Keyword(Keyword::Or) => Some(BinaryOp::Or),
        TokenKind::Concat => Some(BinaryOp::Concat),
        TokenKind::Keyword(Keyword::Like) => Some(BinaryOp::Like),
        TokenKind::BitAnd => Some(BinaryOp::BitAnd),
        TokenKind::BitOr => Some(BinaryOp::BitOr),
        TokenKind::LeftShift => Some(BinaryOp::LeftShift),
        TokenKind::RightShift => Some(BinaryOp::RightShift),
        _ => None,
    }
}

/// Converts a token to a unary operator.
#[must_use]
pub const fn token_to_unary_op(kind: &TokenKind) -> Option<UnaryOp> {
    match kind {
        TokenKind::Minus => Some(UnaryOp::Neg),
        TokenKind::Keyword(Keyword::Not) => Some(UnaryOp::Not),
        TokenKind::BitNot => Some(UnaryOp::BitNot),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precedence_ordering() {
        // Multiplication should bind tighter than addition
        let add_bp = infix_binding_power(&TokenKind::Plus).unwrap();
        let mul_bp = infix_binding_power(&TokenKind::Star).unwrap();
        assert!(mul_bp.0 > add_bp.0);

        // AND should bind tighter than OR
        let and_bp = infix_binding_power(&TokenKind::Keyword(Keyword::And)).unwrap();
        let or_bp = infix_binding_power(&TokenKind::Keyword(Keyword::Or)).unwrap();
        assert!(and_bp.0 > or_bp.0);

        // Comparison should bind tighter than logical operators
        let eq_bp = infix_binding_power(&TokenKind::Eq).unwrap();
        assert!(eq_bp.0 > and_bp.0);
    }

    #[test]
    fn test_left_associativity() {
        // Binary operators should be left-associative
        let (left, right) = infix_binding_power(&TokenKind::Plus).unwrap();
        assert!(left < right);
    }

    #[test]
    fn test_token_to_binary_op() {
        assert_eq!(token_to_binary_op(&TokenKind::Plus), Some(BinaryOp::Add));
        assert_eq!(token_to_binary_op(&TokenKind::Minus), Some(BinaryOp::Sub));
        assert_eq!(token_to_binary_op(&TokenKind::Eq), Some(BinaryOp::Eq));
        assert_eq!(token_to_binary_op(&TokenKind::LeftParen), None);
    }

    #[test]
    fn test_token_to_unary_op() {
        assert_eq!(token_to_unary_op(&TokenKind::Minus), Some(UnaryOp::Neg));
        assert_eq!(
            token_to_unary_op(&TokenKind::Keyword(Keyword::Not)),
            Some(UnaryOp::Not)
        );
        assert_eq!(token_to_unary_op(&TokenKind::Plus), None);
    }
}
