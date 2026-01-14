//! RETURN 子句

use crate::query::parser::ast::*;

/// RETURN 子句
#[derive(Debug, Clone, PartialEq)]
pub struct ReturnClause {
    pub span: Span,
    pub items: Vec<ReturnItem>,
    pub distinct: bool,
}

/// RETURN 项
#[derive(Debug, Clone, PartialEq)]
pub enum ReturnItem {
    All,
    Expression { expr: Expr, alias: Option<String> },
}

/// RETURN 子句解析器
pub trait ReturnParser {
    fn parse_return_clause(&mut self) -> Result<ReturnClause, ParseError>;
}
