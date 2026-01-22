//! ORDER BY 子句
//!
//! 子句结构定义已移至 ast/stmt.rs，此文件仅保留解析器 trait

use crate::query::parser::ast::*;

/// ORDER BY 子句解析器
pub trait OrderByParser {
    fn parse_order_by_clause(&mut self) -> Result<OrderByClause, ParseError>;
}
