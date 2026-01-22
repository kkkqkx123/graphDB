//! 子句解析器实现
//!
//! 包含所有子句的解析方法实现

use crate::query::parser::ast::stmt::{
    FromClause,
    OverClause,
    WhereClause,
    ReturnClause,
    ReturnItem,
    YieldClause,
    YieldItem,
    SetClause,
    Assignment,
    OrderByClause,
    OrderByItem,
    MatchClause,
    WithClause,
};
use crate::query::parser::ast::types::{LimitClause, SampleClause, SkipClause, OrderDirection};
use crate::query::parser::ast::*;
use crate::query::parser::core::{ParseError, Span};
use crate::query::parser::TokenKind;

impl crate::query::parser::Parser {
    pub fn parse_match_clause(&mut self) -> Result<MatchClause, ParseError> {
        let span = self.current_span();
        self.expect_token(TokenKind::Match)?;
        
        let patterns = self.parse_patterns()?;
        
        let optional = if self.match_token(TokenKind::Optional) {
            true
        } else {
            false
        };
        
        Ok(MatchClause {
            span,
            patterns,
            optional,
        })
    }
    
    pub fn parse_with_clause(&mut self) -> Result<WithClause, ParseError> {
        let span = self.current_span();
        self.expect_token(TokenKind::With)?;
        
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
        
        let where_clause = if self.match_token(TokenKind::Where) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        Ok(WithClause {
            span,
            items,
            where_clause,
        })
    }
    
    pub fn parse_where_clause(&mut self) -> Result<WhereClause, ParseError> {
        self.expect_token(TokenKind::Where)?;
        
        let condition = self.parse_expression()?;
        
        Ok(WhereClause {
            span: self.current_span(),
            condition,
        })
    }
    
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
        
        let limit = if self.current_token().kind == TokenKind::Limit {
            Some(self.parse_limit_clause()?)
        } else {
            None
        };
        
        let skip = if self.current_token().kind == TokenKind::Skip {
            Some(self.parse_skip_clause()?)
        } else {
            None
        };
        
        let sample = if self.current_token().kind == TokenKind::Sample {
            Some(self.parse_sample_clause()?)
        } else {
            None
        };
        
        Ok(ReturnClause {
            span: self.current_span(),
            items,
            distinct,
            limit,
            skip,
            sample,
        })
    }
    
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
        
        let limit = if self.current_token().kind == TokenKind::Limit {
            Some(self.parse_limit_clause()?)
        } else {
            None
        };
        
        let skip = if self.current_token().kind == TokenKind::Skip {
            Some(self.parse_skip_clause()?)
        } else {
            None
        };
        
        let sample = if self.current_token().kind == TokenKind::Sample {
            Some(self.parse_sample_clause()?)
        } else {
            None
        };
        
        Ok(YieldClause {
            span: self.current_span(),
            items,
            limit,
            skip,
            sample,
        })
    }
    
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
                path.push_str(s.as_str());
                self.next_token();
            }
            TokenKind::Dollar => {
                path.push('$');
                self.next_token();
                match &self.current_token().kind {
                    TokenKind::Identifier(s) => {
                        path.push_str(s.as_str());
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
