//! SKIP/LIMIT 子句解析器实现

use crate::query::parser::ast::types::{LimitClause, SkipClause, SampleClause};
use crate::query::parser::ast::Expr;
use crate::query::parser::core::error::ParseError;
use crate::query::parser::TokenKind;

impl crate::query::parser::Parser {
    pub fn parse_skip_clause(&mut self) -> Result<SkipClause, ParseError> {
        self.expect_token(TokenKind::Skip)?;
        
        let count = self.parse_expression()?;
        
        let count_value = Self::evaluate_expression_to_usize(&count)?;
        
        Ok(SkipClause {
            span: self.current_span(),
            count: count_value,
        })
    }
    
    pub fn parse_limit_clause(&mut self) -> Result<LimitClause, ParseError> {
        self.expect_token(TokenKind::Limit)?;
        
        let count = self.parse_expression()?;
        
        let count_value = Self::evaluate_expression_to_usize(&count)?;
        
        Ok(LimitClause {
            span: self.current_span(),
            count: count_value,
        })
    }
    
    pub fn parse_sample_clause(&mut self) -> Result<SampleClause, ParseError> {
        self.expect_token(TokenKind::Sample)?;
        
        let count = self.parse_expression()?;
        
        let count_value = Self::evaluate_expression_to_usize(&count)?;
        
        Ok(SampleClause {
            span: self.current_span(),
            count: count_value,
            percentage: None,
        })
    }
    
    fn evaluate_expression_to_usize(expr: &Expr) -> Result<usize, ParseError> {
        match expr {
            Expr::Constant(c) => match &c.value {
                crate::core::Value::Int(n) => Ok(*n as usize),
                crate::core::Value::String(s) => s.parse::<usize>().map_err(|_| {
                    ParseError::new(
                        crate::query::parser::core::error::ParseErrorKind::SyntaxError,
                        format!("Cannot convert '{}' to integer", s),
                        c.span.start.line,
                        c.span.start.column,
                    )
                }),
                _ => Err(ParseError::new(
                    crate::query::parser::core::error::ParseErrorKind::SyntaxError,
                    "Expected integer literal".to_string(),
                    expr.span().start.line,
                    expr.span().start.column,
                )),
            },
            _ => Err(ParseError::new(
                crate::query::parser::core::error::ParseErrorKind::SyntaxError,
                "Expected integer literal".to_string(),
                expr.span().start.line,
                expr.span().start.column,
            )),
        }
    }
}
