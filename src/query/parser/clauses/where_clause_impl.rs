//! WHERE 子句解析器实现

use crate::query::parser::ast::*;
use crate::query::parser::core::error::ParseError;
use crate::query::parser::TokenKind;

impl crate::query::parser::Parser {
    pub fn parse_where_clause(&mut self) -> Result<WhereClause, ParseError> {
        self.expect_token(TokenKind::Where)?;
        
        let condition = self.parse_expression()?;
        
        Ok(WhereClause {
            span: self.current_span(),
            condition,
        })
    }
}
