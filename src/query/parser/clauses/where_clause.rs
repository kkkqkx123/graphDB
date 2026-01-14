//! WHERE 子句

use crate::query::parser::ast::*;

/// WHERE 子句
#[derive(Debug, Clone, PartialEq)]
pub struct WhereClause {
    pub span: Span,
    pub condition: Expr,
}

/// WHERE 子句解析器
pub trait WhereParser {
    fn parse_where_clause(&mut self) -> Result<WhereClause, ParseError>;
}
