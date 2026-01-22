//! Position and Span types for the query parser
//!
//! This module defines source location types for tracking
//! token and AST node positions.

use std::fmt;

/// Represents a position in the source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    pub fn to_offset(&self, line_lengths: &[usize]) -> Option<usize> {
        if self.line == 0 || self.line > line_lengths.len() {
            return None;
        }

        let mut offset = 0;
        for i in 0..self.line - 1 {
            offset += line_lengths[i] + 1; // +1 for newline
        }
        offset += self.column.saturating_sub(1);

        Some(offset)
    }

    pub fn to_usize(&self) -> usize {
        self.line * 1000 + self.column
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.line, self.column)
    }
}

/// Represents a span of source code from start to end position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    pub fn from_tokens(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Self {
        Self {
            start: Position::new(start_line, start_col),
            end: Position::new(end_line, end_col),
        }
    }

    pub fn from_position(pos: Position) -> Self {
        Self {
            start: pos,
            end: pos,
        }
    }

    pub fn extend(&mut self, other: Span) {
        self.end = other.end;
    }

    pub fn merge(&self, other: Span) -> Span {
        Span::new(
            self.start,
            if self.end > other.end { self.end } else { other.end },
        )
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn contains(&self, pos: Position) -> bool {
        self.start <= pos && pos <= self.end
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}:{} - {}:{}]", self.start.line, self.start.column, self.end.line, self.end.column)
    }
}

/// Extension trait for converting line/column to Span
pub trait ToSpan {
    fn to_span(&self) -> Span;
}

impl ToSpan for (usize, usize) {
    fn to_span(&self) -> Span {
        Span::from_position(Position::new(self.0, self.1))
    }
}

impl ToSpan for (usize, usize, usize, usize) {
    fn to_span(&self) -> Span {
        Span::from_tokens(self.0, self.1, self.2, self.3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_new() {
        let pos = Position::new(5, 10);
        assert_eq!(pos.line, 5);
        assert_eq!(pos.column, 10);
    }

    #[test]
    fn test_span_new() {
        let start = Position::new(1, 1);
        let end = Position::new(1, 10);
        let span = Span::new(start, end);
        assert_eq!(span.start, start);
        assert_eq!(span.end, end);
    }

    #[test]
    fn test_span_contains() {
        let span = Span::new(Position::new(1, 1), Position::new(1, 10));
        assert!(span.contains(Position::new(1, 5)));
        assert!(!span.contains(Position::new(2, 1)));
    }

    #[test]
    fn test_span_merge() {
        let span1 = Span::new(Position::new(1, 1), Position::new(1, 5));
        let span2 = Span::new(Position::new(1, 6), Position::new(1, 10));
        let merged = span1.merge(span2);
        assert_eq!(merged.start, Position::new(1, 1));
        assert_eq!(merged.end, Position::new(1, 10));
    }
}
