//! Source location tracking for tokens and AST nodes.

/// Represents a span in the source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// Start byte offset (inclusive).
    pub start: usize,
    /// End byte offset (exclusive).
    pub end: usize,
}

impl Span {
    /// Creates a new span.
    #[must_use]
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Returns the length of the span in bytes.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.end - self.start
    }

    /// Returns true if the span is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Merges two spans into one that covers both.
    #[must_use]
    pub const fn merge(self, other: Self) -> Self {
        let start = if self.start < other.start {
            self.start
        } else {
            other.start
        };
        let end = if self.end > other.end {
            self.end
        } else {
            other.end
        };
        Self { start, end }
    }
}

impl Default for Span {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_new() {
        let span = Span::new(5, 10);
        assert_eq!(span.start, 5);
        assert_eq!(span.end, 10);
    }

    #[test]
    fn test_span_len() {
        let span = Span::new(5, 10);
        assert_eq!(span.len(), 5);
    }

    #[test]
    fn test_span_is_empty() {
        let empty = Span::new(5, 5);
        let non_empty = Span::new(5, 10);
        assert!(empty.is_empty());
        assert!(!non_empty.is_empty());
    }

    #[test]
    fn test_span_merge() {
        let span1 = Span::new(5, 10);
        let span2 = Span::new(8, 15);
        let merged = span1.merge(span2);
        assert_eq!(merged.start, 5);
        assert_eq!(merged.end, 15);
    }
}
