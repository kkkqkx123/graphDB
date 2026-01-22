//! SKIP/LIMIT 子句

use crate::query::parser::ast::*;
use crate::query::parser::ast::types::{LimitClause, SampleClause, SkipClause};

/// SKIP 子句解析器
pub trait SkipParser {
    fn parse_skip_clause(&mut self) -> Result<SkipClause, ParseError>;
}

/// LIMIT 子句解析器
pub trait LimitParser {
    fn parse_limit_clause(&mut self) -> Result<LimitClause, ParseError>;
}

/// SAMPLE 子句解析器
pub trait SampleParser {
    fn parse_sample_clause(&mut self) -> Result<SampleClause, ParseError>;
}
