//! FROM 子句

use crate::query::parser::ast::*;

/// FROM 子句
#[derive(Debug, Clone, PartialEq)]
pub struct FromClause {
    pub span: Span,
    pub vertices: Vec<Expr>,
}

/// FROM 子句解析器
pub trait FromParser {
    fn parse_from_clause(&mut self) -> Result<FromClause, ParseError>;
}
