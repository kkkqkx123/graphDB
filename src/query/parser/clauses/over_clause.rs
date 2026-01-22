//! OVER 子句
//!
//! 子句结构定义已移至 ast/stmt.rs，此文件仅保留解析器 trait

use crate::query::parser::ast::*;

/// OVER 子句解析器
pub trait OverParser {
    fn parse_over_clause(&mut self) -> Result<OverClause, ParseError>;
}
