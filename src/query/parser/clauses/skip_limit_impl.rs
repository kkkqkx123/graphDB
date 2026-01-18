//! SKIP/LIMIT 子句解析器实现

use crate::query::parser::clauses::{SkipClause, LimitClause};
use crate::query::parser::core::error::ParseError;
use crate::query::parser::TokenKind;

impl crate::query::parser::Parser {
    pub fn parse_skip_clause(&mut self) -> Result<SkipClause, ParseError> {
        self.expect_token(TokenKind::Skip)?;
        
        let count = self.parse_expression()?;
        
        Ok(SkipClause {
            span: self.current_span(),
            count,
        })
    }
    
    pub fn parse_limit_clause(&mut self) -> Result<LimitClause, ParseError> {
        self.expect_token(TokenKind::Limit)?;
        
        let count = self.parse_expression()?;
        
        Ok(LimitClause {
            span: self.current_span(),
            count,
        })
    }
}
