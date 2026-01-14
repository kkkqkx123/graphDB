//! 遍历语句模块
//!
//! 包含所有遍历相关的语句：GO, FIND PATH

use crate::query::parser::ast::*;

/// 遍历语句解析器
pub trait TraverseParser {
    fn parse_traverse_statement(&mut self) -> Result<Stmt, ParseError>;
}
