//! WITH 子句

use crate::query::parser::ast::*;

/// WITH 子句
#[derive(Debug, Clone, PartialEq)]
pub struct WithClause {
    pub span: Span,
    pub items: Vec<ReturnItem>,
    pub where_clause: Option<Expr>,
}

/// WITH 子句解析器
pub trait WithParser {
    fn parse_with_clause(&mut self) -> Result<WithClause, ParseError>;
}
