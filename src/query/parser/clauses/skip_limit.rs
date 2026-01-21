//! SKIP/LIMIT 子句

use crate::query::parser::ast::*;

/// SKIP 子句
#[derive(Debug, Clone, PartialEq)]
pub struct SkipClause {
    pub span: Span,
    pub count: Expr,
}

/// LIMIT 子句
#[derive(Debug, Clone, PartialEq)]
pub struct LimitClause {
    pub span: Span,
    pub count: Expr,
}

/// SAMPLE 子句 - 对查询结果进行随机采样
#[derive(Debug, Clone, PartialEq)]
pub struct SampleClause {
    pub span: Span,
    pub count: Expr,
}

/// SKIP 子句解析器
pub trait SkipParser {
    fn parse_skip_clause(&mut self) -> Result<SkipClause, ParseError>;
}

/// LIMIT 子句解析器
pub trait LimitParser {
    fn parse_limit_clause(&mut self) -> Result<LimitClause, ParseError>;
}

/// SAMPLE 子句解析器
pub trait SampleParser {
    fn parse_sample_clause(&mut self) -> Result<SampleClause, ParseError>;
}
