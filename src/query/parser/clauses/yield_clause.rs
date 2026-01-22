//! YIELD 子句
//!
//! 子句结构定义已移至 ast/stmt.rs，此文件仅保留解析器 trait

use crate::query::parser::ast::*;
use crate::query::parser::ast::types::{LimitClause, SampleClause, SkipClause};

/// YIELD 子句解析器
pub trait YieldParser {
    fn parse_yield_clause(&mut self) -> Result<YieldClause, ParseError>;
}
