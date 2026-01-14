//! MATCH 子句

use crate::query::parser::ast::*;

/// MATCH 子句
#[derive(Debug, Clone, PartialEq)]
pub struct MatchClause {
    pub span: Span,
    pub patterns: Vec<Pattern>,
    pub optional: bool,
}

/// MATCH 子句解析器
pub trait MatchClauseParser {
    fn parse_match_clause(&mut self) -> Result<MatchClause, ParseError>;
}
