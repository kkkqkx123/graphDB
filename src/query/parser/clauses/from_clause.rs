//! FROM 子句
//!
//! 子句结构定义已移至 ast/stmt.rs，此文件仅保留解析器 trait

use crate::query::parser::ast::*;

/// FROM 子句解析器
pub trait FromParser {
    fn parse_from_clause(&mut self) -> Result<FromClause, ParseError>;
}
