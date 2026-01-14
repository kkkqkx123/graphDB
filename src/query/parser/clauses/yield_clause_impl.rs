//! YIELD 子句解析器实现

use crate::query::parser::ast::*;
use crate::query::parser::core::error::ParseError;
use crate::query::parser::TokenKind;

impl crate::query::parser::Parser {
    pub fn parse_yield_clause(&mut self) -> Result<YieldClause, ParseError> {
        self.expect_token(TokenKind::Yield)?;
        
        let mut items = Vec::new();
        
        loop {
            let expr = self.parse_expression()?;
            
            let alias = if self.current_token().kind == TokenKind::As {
                self.next_token();
                Some(self.parse_identifier()?)
            } else {
                None
            };
            
            items.push(YieldItem { expr, alias });
            
            if self.current_token().kind != TokenKind::Comma {
                break;
            }
            self.next_token();
        }
        
        Ok(YieldClause {
            span: self.current_span(),
            items,
        })
    }
}
