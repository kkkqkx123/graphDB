//! ORDER BY 子句解析器实现

use crate::query::parser::ast::*;
use crate::query::parser::core::error::ParseError;
use crate::query::parser::TokenKind;

impl crate::query::parser::Parser {
    pub fn parse_order_by_clause(&mut self) -> Result<OrderByClause, ParseError> {
        self.expect_token(TokenKind::Order)?;
        self.expect_token(TokenKind::By)?;
        
        let mut items = Vec::new();
        
        loop {
            let expr = self.parse_expression()?;
            
            let direction = match self.current_token().kind {
                TokenKind::Asc | TokenKind::Ascending => {
                    self.next_token();
                    OrderDirection::Asc
                }
                TokenKind::Desc | TokenKind::Descending => {
                    self.next_token();
                    OrderDirection::Desc
                }
                _ => OrderDirection::Asc,
            };
            
            items.push(OrderByItem { expr, direction });
            
            if self.current_token().kind != TokenKind::Comma {
                break;
            }
            self.next_token();
        }
        
        Ok(OrderByClause {
            span: self.current_span(),
            items,
        })
    }
}
