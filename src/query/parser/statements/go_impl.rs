//! GO 语句解析器实现

use crate::query::parser::ast::*;
use crate::query::parser::core::error::ParseError;
use crate::query::parser::TokenKind;

impl crate::query::parser::Parser {
    pub fn parse_go_statement(&mut self) -> Result<Stmt, ParseError> {
        self.expect_token(TokenKind::Go)?;
        
        let steps = self.parse_steps()?;
        
        let from_clause = self.parse_from_clause()?;
        
        let over_clause = if self.current_token().kind == TokenKind::Over {
            Some(self.parse_over_clause()?)
        } else {
            None
        };
        
        let where_clause = if self.current_token().kind == TokenKind::Where {
            Some(self.parse_where_clause()?.condition)
        } else {
            None
        };
        
        let _yield_clause = if self.current_token().kind == TokenKind::Yield {
            Some(self.parse_yield_clause()?)
        } else {
            None
        };
        
        Ok(Stmt::Go(GoStmt {
            span: self.current_span(),
            steps,
            from: from_clause,
            over: over_clause,
            where_clause,
            yield_clause: None,
        }))
    }
    
    fn parse_steps(&mut self) -> Result<Steps, ParseError> {
        if self.current_token().kind == TokenKind::Step {
            self.next_token();
            
            if self.current_token().kind == TokenKind::Upto {
                self.next_token();
                let min = self.parse_integer_literal()?;
                self.expect_token(TokenKind::To)?;
                let max = self.parse_integer_literal()?;
                Ok(Steps::Range { min, max })
            } else {
                let steps = self.parse_integer_literal()?;
                Ok(Steps::Fixed(steps))
            }
        } else {
            Ok(Steps::Fixed(1))
        }
    }
    
    fn parse_integer_literal(&mut self) -> Result<usize, ParseError> {
        match &self.current_token().kind {
            TokenKind::IntegerLiteral(n) => {
                let value = *n as usize;
                self.next_token();
                Ok(value)
            }
            _ => Err(ParseError::syntax_error(
                format!("Expected integer literal, got {:?}", self.current_token().kind),
                self.current_token().line,
                self.current_token().column,
            )),
        }
    }
}
