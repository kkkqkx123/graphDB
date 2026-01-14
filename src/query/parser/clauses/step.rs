//! STEP 子句

use crate::query::parser::ast::*;

/// STEP 子句
#[derive(Debug, Clone, PartialEq)]
pub struct StepClause {
    pub span: Span,
    pub steps: Steps,
}

/// 步数定义
#[derive(Debug, Clone, PartialEq)]
pub enum Steps {
    Fixed(usize),
    Range { min: usize, max: usize },
    Variable(String),
}

/// STEP 子句解析器
pub trait StepParser {
    fn parse_step_clause(&mut self) -> Result<StepClause, ParseError>;
}
