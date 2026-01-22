//! WHERE 子句
//!
//! 子句结构定义已移至 ast/stmt.rs，此文件仅保留解析器 trait

use crate::query::parser::ast::*;

/// WHERE 子句解析器
pub trait WhereParser {
    fn parse_where_clause(&mut self) -> Result<WhereClause, ParseError>;
}
