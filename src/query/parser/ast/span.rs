//! 位置信息模块
//!
//! 提供 AST 节点的位置信息，用于错误报告和调试。

use std::fmt;

/// 位置信息 - 表示源代码中的一个位置
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub line: u32,
    pub column: u32,
    pub offset: usize,
}

impl Position {
    pub fn new(line: u32, column: u32, offset: usize) -> Self {
        Self { line, column, offset }
    }
    
    pub fn start() -> Self {
        Self {
            line: 1,
            column: 1,
            offset: 0,
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// 跨度信息 - 表示源代码中的一个范围
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
    
    pub fn from_positions(start: Position, end: Position) -> Self {
        Self { start, end }
    }
    
    pub fn single(position: Position) -> Self {
        Self {
            start: position,
            end: position,
        }
    }
    
    pub fn merge(&self, other: &Span) -> Self {
        Self {
            start: if self.start.offset < other.start.offset {
                self.start
            } else {
                other.start
            },
            end: if self.end.offset > other.end.offset {
                self.end
            } else {
                other.end
            },
        }
    }
    
    pub fn contains(&self, position: Position) -> bool {
        position.offset >= self.start.offset && position.offset <= self.end.offset
    }
    
    pub fn overlaps(&self, other: &Span) -> bool {
        self.start.offset <= other.end.offset && self.end.offset >= other.start.offset
    }
    
    pub fn is_empty(&self) -> bool {
        self.start.offset == self.end.offset
    }
    
    pub fn length(&self) -> usize {
        self.end.offset.saturating_sub(self.start.offset)
    }
}

impl Default for Span {
    fn default() -> Self {
        Self {
            start: Position::start(),
            end: Position::start(),
        }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start == self.end {
            write!(f, "{}", self.start)
        } else {
            write!(f, "{}-{} ({}:{})", self.start, self.end, self.start.line, self.start.column)
        }
    }
}

/// 带位置信息的包装器
#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }
    
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Spanned<U> {
        Spanned {
            node: f(self.node),
            span: self.span,
        }
    }
    
    pub fn as_ref(&self) -> Spanned<&T> {
        Spanned {
            node: &self.node,
            span: self.span,
        }
    }
    
    pub fn span(&self) -> Span {
        self.span
    }
}

/// 用于构建位置信息的宏
#[macro_export]
macro_rules! span {
    ($start:expr, $end:expr) => {
        $crate::query::parser::ast::Span::new($start, $end)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_position_creation() {
        let pos = Position::new(1, 10, 100);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 10);
        assert_eq!(pos.offset, 100);
    }
    
    #[test]
    fn test_span_creation() {
        let start = Position::new(1, 1, 0);
        let end = Position::new(1, 10, 9);
        let span = Span::new(start, end);
        
        assert_eq!(span.start.line, 1);
        assert_eq!(span.end.column, 10);
        assert_eq!(span.length(), 9);
    }
    
    #[test]
    fn test_span_merge() {
        let span1 = Span::new(
            Position::new(1, 1, 0),
            Position::new(1, 5, 4),
        );
        let span2 = Span::new(
            Position::new(1, 3, 2),
            Position::new(1, 10, 9),
        );
        
        let merged = span1.merge(&span2);
        assert_eq!(merged.start.offset, 0);
        assert_eq!(merged.end.offset, 9);
    }
}