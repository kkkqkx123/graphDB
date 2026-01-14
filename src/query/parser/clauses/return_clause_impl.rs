//! RETURN 子句解析器实现

use crate::query::parser::ast::*;
use crate::query::parser::core::error::ParseError;
use crate::query::parser::TokenKind;

impl crate::query::parser::Parser {
    pub fn parse_return_clause(&mut self) -> Result<ReturnClause, ParseError> {
        self.expect_token(TokenKind::Return)?;
        
        let distinct = if self.current_token().kind == TokenKind::Distinct {
            self.next_token();
            true
        } else {
            false
        };
        
        let mut items = Vec::new();
        
        if self.current_token().kind == TokenKind::Star {
            self.next_token();
            items.push(ReturnItem::All);
        } else {
            loop {
                let expr = self.parse_expression()?;
                
                let alias = if self.current_token().kind == TokenKind::As {
                    self.next_token();
                    Some(self.parse_identifier()?)
                } else {
                    None
                };
                
                items.push(ReturnItem::Expression { expr, alias });
                
                if self.current_token().kind != TokenKind::Comma {
                    break;
                }
                self.next_token();
            }
        }
        
        Ok(ReturnClause {
            span: self.current_span(),
            items,
            distinct,
        })
    }
}
