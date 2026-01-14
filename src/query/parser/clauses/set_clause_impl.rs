//! SET 子句解析器实现

use crate::query::parser::ast::*;
use crate::query::parser::core::error::ParseError;
use crate::query::parser::TokenKind;

impl crate::query::parser::Parser {
    pub fn parse_set_clause(&mut self) -> Result<SetClause, ParseError> {
        self.expect_token(TokenKind::Set)?;
        
        let mut assignments = Vec::new();
        
        loop {
            let property = self.parse_property_path()?;
            
            self.expect_token(TokenKind::Assign)?;
            
            let value = self.parse_expression()?;
            
            assignments.push(Assignment { property, value });
            
            if self.current_token().kind != TokenKind::Comma {
                break;
            }
            self.next_token();
        }
        
        Ok(SetClause {
            span: self.current_span(),
            assignments,
        })
    }
    
    fn parse_property_path(&mut self) -> Result<String, ParseError> {
        let mut path = String::new();
        
        match &self.current_token().kind {
            TokenKind::Identifier(s) => {
                path.push_str(s);
                self.next_token();
            }
            TokenKind::Dollar => {
                path.push('$');
                self.next_token();
                match &self.current_token().kind {
                    TokenKind::Identifier(s) => {
                        path.push_str(s);
                        self.next_token();
                    }
                    _ => return Err(ParseError::syntax_error(
                        "Expected identifier after $".to_string(),
                        self.current_token().line,
                        self.current_token().column,
                    )),
                }
            }
            _ => return Err(ParseError::syntax_error(
                format!("Expected property path, got {:?}", self.current_token().kind),
                self.current_token().line,
                self.current_token().column,
            )),
        }
        
        Ok(path)
    }
}
