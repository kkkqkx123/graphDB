//! OVER 子句

use crate::query::parser::ast::*;

/// OVER 子句
#[derive(Debug, Clone, PartialEq)]
pub struct OverClause {
    pub span: Span,
    pub edge_types: Vec<String>,
    pub direction: EdgeDirection,
}

/// OVER 子句解析器
pub trait OverParser {
    fn parse_over_clause(&mut self) -> Result<OverClause, ParseError>;
}
