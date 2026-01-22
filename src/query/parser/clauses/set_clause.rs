//! SET 子句
//!
//! 子句结构定义已移至 ast/stmt.rs，此文件仅保留解析器 trait

use crate::query::parser::ast::*;

/// SET 子句解析器
pub trait SetParser {
    fn parse_set_clause(&mut self) -> Result<SetClause, ParseError>;
}
