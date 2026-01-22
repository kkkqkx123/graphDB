//! RETURN 子句
//!
//! 子句结构定义已移至 ast/stmt.rs，此文件仅保留解析器 trait

use crate::query::parser::ast::*;
use crate::query::parser::ast::types::{LimitClause, SampleClause, SkipClause};

/// RETURN 子句解析器
pub trait ReturnParser {
    fn parse_return_clause(&mut self) -> Result<ReturnClause, ParseError>;
}
