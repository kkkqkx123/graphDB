//! STEP 子句
//!
//! 步数定义已移至 ast/stmt.rs，此文件仅保留解析器 trait

use crate::query::parser::ast::*;

/// STEP 子句解析器
pub trait StepParser {
    fn parse_step_clause(&mut self) -> Result<StepClause, ParseError>;
}
