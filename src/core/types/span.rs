//! 源码位置类型定义
//!
//! 本模块定义通用的源码位置类型，用于表示 token 和 AST 节点在源码中的位置。

use serde::{Deserialize, Serialize};
use std::fmt;

/// 源码位置
///
/// 表示源码中的一个点位置，包含行号和列号。
/// 行号和列号都从 1 开始计数。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct Position {
    /// 行号，从 1 开始
    pub line: usize,
    /// 列号，从 1 开始
    pub column: usize,
}

impl Position {
    /// 创建新位置
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    /// 将位置转换为字符偏移量
    ///
    /// # 参数
    ///
    /// * `line_lengths` - 每行的字符长度数组
    ///
    /// # 返回值
    ///
    /// 如果行号有效，返回对应的字符偏移量；否则返回 None
    pub fn to_offset(&self, line_lengths: &[usize]) -> Option<usize> {
        if self.line == 0 || self.line > line_lengths.len() {
            return None;
        }

        let mut offset = 0;
        for i in 0..self.line - 1 {
            offset += line_lengths[i] + 1;
        }
        offset += self.column.saturating_sub(1);

        Some(offset)
    }

    /// 将位置转换为 usize（用于简单比较）
    pub fn to_usize(&self) -> usize {
        self.line * 1000 + self.column
    }

    /// 检查位置是否有效（行号和列号都大于 0）
    pub fn is_valid(&self) -> bool {
        self.line > 0 && self.column > 0
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.line, self.column)
    }
}

/// 源码跨度
///
/// 表示源码中的一个范围，从起始位置到结束位置。
/// 用于表示 token、表达式、语句等在源码中的位置范围。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Span {
    /// 起始位置（包含）
    pub start: Position,
    /// 结束位置（包含）
    pub end: Position,
}

impl Span {
    /// 创建新跨度
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// 从四个坐标创建跨度
    pub fn from_coords(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Self {
        Self {
            start: Position::new(start_line, start_col),
            end: Position::new(end_line, end_col),
        }
    }

    /// 从单个位置创建跨度（用于单个 token）
    pub fn from_position(pos: Position) -> Self {
        Self {
            start: pos,
            end: pos,
        }
    }

    /// 扩展跨度的结束位置
    pub fn extend(&mut self, other: Span) {
        self.end = other.end;
    }

    /// 合并两个跨度
    ///
    /// # 参数
    ///
    /// * `other` - 要合并的另一个跨度
    ///
    /// # 返回值
    ///
    /// 新的跨度，起始位置为当前跨度的起始，结束位置为两跨度中较大的结束位置
    pub fn merge(&self, other: Span) -> Span {
        Span::new(
            self.start,
            if self.end >= other.end { self.end } else { other.end },
        )
    }

    /// 检查跨度是否为空（起始位置等于结束位置）
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// 检查跨度是否包含指定位置
    pub fn contains(&self, pos: Position) -> bool {
        self.start <= pos && pos <= self.end
    }

    /// 获取跨度的行号范围
    pub fn line_range(&self) -> (usize, usize) {
        (self.start.line, self.end.line)
    }

    /// 获取跨度的列号范围（仅当在同一行时有效）
    pub fn column_range(&self) -> (usize, usize) {
        (self.start.column, self.end.column)
    }

    /// 创建默认跨度（用于内部生成的表达式）
    pub fn default() -> Self {
        Self {
            start: Position::new(0, 0),
            end: Position::new(0, 0),
        }
    }

    /// 转换为字符串表示
    pub fn to_string(&self) -> String {
        format!("{}:{} - {}:{}", self.start.line, self.start.column, self.end.line, self.end.column)
    }

    /// 获取起始位置的行号
    pub fn start_line(&self) -> usize {
        self.start.line
    }

    /// 获取起始位置的列号
    pub fn start_column(&self) -> usize {
        self.start.column
    }

    /// 获取结束位置的行号
    pub fn end_line(&self) -> usize {
        self.end.line
    }

    /// 获取结束位置的列号
    pub fn end_column(&self) -> usize {
        self.end.column
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}:{} - {}:{}]", self.start.line, self.start.column, self.end.line, self.end.column)
    }
}

/// 转换为 Span 的 trait
///
/// 用于方便地将位置相关类型转换为 Span
pub trait ToSpan {
    fn to_span(&self) -> Span;
}

impl ToSpan for Position {
    fn to_span(&self) -> Span {
        Span::from_position(*self)
    }
}

impl ToSpan for Span {
    fn to_span(&self) -> Span {
        *self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_creation() {
        let pos = Position::new(10, 5);
        assert_eq!(pos.line, 10);
        assert_eq!(pos.column, 5);
    }

    #[test]
    fn test_position_display() {
        let pos = Position::new(1, 1);
        assert_eq!(pos.to_string(), "(1, 1)");
    }

    #[test]
    fn test_position_to_usize() {
        let pos = Position::new(2, 3);
        assert_eq!(pos.to_usize(), 2003);
    }

    #[test]
    fn test_span_creation() {
        let span = Span::new(Position::new(1, 1), Position::new(1, 10));
        assert_eq!(span.start.line, 1);
        assert_eq!(span.end.column, 10);
    }

    #[test]
    fn test_span_from_coords() {
        let span = Span::from_coords(1, 5, 2, 10);
        assert_eq!(span.start.line, 1);
        assert_eq!(span.start.column, 5);
        assert_eq!(span.end.line, 2);
        assert_eq!(span.end.column, 10);
    }

    #[test]
    fn test_span_from_position() {
        let pos = Position::new(5, 10);
        let span = Span::from_position(pos);
        assert!(span.is_empty());
    }

    #[test]
    fn test_span_extend() {
        let mut span = Span::new(Position::new(1, 1), Position::new(1, 5));
        span.extend(Span::new(Position::new(1, 6), Position::new(2, 10)));
        assert_eq!(span.end.line, 2);
        assert_eq!(span.end.column, 10);
    }

    #[test]
    fn test_span_merge() {
        let span1 = Span::new(Position::new(1, 1), Position::new(1, 5));
        let span2 = Span::new(Position::new(1, 6), Position::new(2, 10));
        let merged = span1.merge(span2);
        assert_eq!(merged.start, Position::new(1, 1));
        assert_eq!(merged.end, Position::new(2, 10));
    }

    #[test]
    fn test_span_is_empty() {
        let span = Span::new(Position::new(1, 1), Position::new(1, 1));
        assert!(span.is_empty());

        let span = Span::new(Position::new(1, 1), Position::new(1, 2));
        assert!(!span.is_empty());
    }

    #[test]
    fn test_span_contains() {
        let span = Span::new(Position::new(1, 1), Position::new(2, 10));
        assert!(span.contains(Position::new(1, 5)));
        assert!(span.contains(Position::new(2, 5)));
        assert!(!span.contains(Position::new(3, 1)));
        assert!(!span.contains(Position::new(0, 1)));
    }

    #[test]
    fn test_span_default() {
        let span = Span::default();
        assert_eq!(span.start.line, 0);
        assert_eq!(span.end.column, 0);
    }

    #[test]
    fn test_span_to_string() {
        let span = Span::new(Position::new(1, 1), Position::new(1, 10));
        assert_eq!(span.to_string(), "1:1 - 1:10");
    }

    #[test]
    fn test_serde_serialize() {
        let span = Span::new(Position::new(1, 5), Position::new(2, 10));
        let json = serde_json::to_string(&span).expect("序列化Span应该成功");
        assert!(json.contains("start"));
        assert!(json.contains("end"));
    }

    #[test]
    fn test_serde_deserialize() {
        let json = r#"{"start":{"line":1,"column":5},"end":{"line":2,"column":10}}"#;
        let span: Span = serde_json::from_str(json).expect("反序列化Span应该成功");
        assert_eq!(span.start.line, 1);
        assert_eq!(span.end.column, 10);
    }
}
