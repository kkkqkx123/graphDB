//! DELETE 语句解析器实现

use crate::query::parser::ast::*;
use crate::query::parser::core::error::ParseError;
use crate::query::parser::TokenKind;

impl crate::query::parser::Parser {
    pub fn parse_delete_statement(&mut self) -> Result<Stmt, ParseError> {
        self.expect_token(TokenKind::Delete)?;
        
        let target = self.parse_delete_target()?;
        
        let where_clause = if self.current_token().kind == TokenKind::Where {
            Some(self.parse_where_clause()?.condition)
        } else {
            None
        };
        
        Ok(Stmt::Delete(DeleteStmt {
            span: self.current_span(),
            target,
            where_clause,
        }))
    }
    
    fn parse_delete_target(&mut self) -> Result<DeleteTarget, ParseError> {
        match self.current_token().kind {
            TokenKind::Tag => {
                self.next_token();
                let name = self.parse_identifier()?;
                Ok(DeleteTarget::Tag(name))
            }
            TokenKind::Index => {
                self.next_token();
                let name = self.parse_identifier()?;
                Ok(DeleteTarget::Index(name))
            }
            _ => {
                let mut vertices = Vec::new();
                
                vertices.push(self.parse_expression()?);
                
                while self.current_token().kind == TokenKind::Comma {
                    self.next_token();
                    vertices.push(self.parse_expression()?);
                }
                
                Ok(DeleteTarget::Vertices(vertices))
            }
        }
    }
}
