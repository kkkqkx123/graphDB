//! YIELD 子句

use crate::query::parser::ast::*;

/// YIELD 子句
#[derive(Debug, Clone, PartialEq)]
pub struct YieldClause {
    pub span: Span,
    pub items: Vec<YieldItem>,
}

/// YIELD 项
#[derive(Debug, Clone, PartialEq)]
pub struct YieldItem {
    pub expr: Expr,
    pub alias: Option<String>,
}

/// YIELD 子句解析器
pub trait YieldParser {
    fn parse_yield_clause(&mut self) -> Result<YieldClause, ParseError>;
}
