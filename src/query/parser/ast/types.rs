//! 基础类型定义

use crate::core::types::operators::AggregateFunction as CoreAggregateFunction;

/// 位置信息
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    pub fn from_tokens(start: &Token, end: &Token) -> Self {
        Self {
            start: start.position,
            end: end.position,
        }
    }

    pub fn default() -> Self {
        Self {
            start: Position::new(0, 0),
            end: Position::new(0, 0),
        }
    }
}

/// 位置信息
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

/// 词法单元（简化版）
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub position: Position,
}

/// 词法单元类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    // 标识符和字面量
    Identifier,
    String,
    Integer,
    Float,
    Boolean,

    // 关键字
    Match,
    Create,
    Delete,
    Update,
    Go,
    Fetch,
    Use,
    Show,
    Explain,
    Lookup,
    Subgraph,
    FindPath,

    // 操作符
    Plus,
    Minus,
    Star,
    Slash,
    Percent,

    // 比较操作符
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    // 逻辑操作符
    And,
    Or,
    Not,

    // 特殊符号
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Semicolon,
    Dot,
    Arrow,

    // 其他
    Eof,
    Unknown,
}

// 使用核心操作符类型
pub type BinaryOp = crate::core::types::operators::BinaryOperator;
pub type UnaryOp = crate::core::types::operators::UnaryOperator;

/// 谓词类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredicateType {
    All,
    Any,
    Single,
    None,
    Exists,
}

/// 边方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeDirection {
    Out,
    In,
    Both,
}

// 使用核心数据类型
pub type DataType = crate::core::types::expression::DataType;

/// 标签
#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    pub name: String,
}

/// 属性引用
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyRef {
    pub name: String,
}

/// 排序方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderDirection {
    Asc,
    Desc,
}

// 聚合函数使用核心定义，但为了兼容性提供别名
pub type AggregateFunction = CoreAggregateFunction;

/// 错误类型
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl ParseError {
    pub fn new(message: String, span: Span) -> Self {
        Self { message, span }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Parse error at line {}, column {}: {}",
            self.span.start.line, self.span.start.column, self.message
        )
    }
}

impl std::error::Error for ParseError {}

impl From<String> for ParseError {
    fn from(message: String) -> Self {
        ParseError::new(message, Span::default())
    }
}

impl From<crate::query::parser::core::error::ParseError> for ParseError {
    fn from(error: crate::query::parser::core::error::ParseError) -> Self {
        ParseError::new(error.message, Span::default())
    }
}
