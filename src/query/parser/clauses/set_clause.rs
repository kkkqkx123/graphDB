//! SET 子句

use crate::query::parser::ast::*;

/// SET 子句
#[derive(Debug, Clone, PartialEq)]
pub struct SetClause {
    pub span: Span,
    pub assignments: Vec<Assignment>,
}

/// 赋值操作
#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub property: String,
    pub value: Expr,
}

/// SET 子句解析器
pub trait SetParser {
    fn parse_set_clause(&mut self) -> Result<SetClause, ParseError>;
}
