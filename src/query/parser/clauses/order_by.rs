//! ORDER BY 子句

use crate::query::parser::ast::*;

/// ORDER BY 子句
#[derive(Debug, Clone, PartialEq)]
pub struct OrderByClause {
    pub span: Span,
    pub items: Vec<OrderByItem>,
}

/// ORDER BY 项
#[derive(Debug, Clone, PartialEq)]
pub struct OrderByItem {
    pub expr: Expr,
    pub direction: OrderDirection,
}

/// 排序方向
#[derive(Debug, Clone, PartialEq)]
pub enum OrderDirection {
    Asc,
    Desc,
}

/// ORDER BY 子句解析器
pub trait OrderByParser {
    fn parse_order_by_clause(&mut self) -> Result<OrderByClause, ParseError>;
}
