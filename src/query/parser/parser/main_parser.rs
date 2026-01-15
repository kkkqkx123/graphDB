//! 主解析器实现

use crate::query::parser::ast::*;
use crate::query::parser::core::error::ParseError;
use crate::query::parser::TokenKind;
use crate::core::types::graph::EdgeDirection;

impl crate::query::parser::Parser {
    pub fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        match self.current_token().kind {
            TokenKind::Match => self.parse_match_statement(),
            TokenKind::Go => self.parse_go_statement(),
            TokenKind::Create => self.parse_create_statement(),
            TokenKind::Delete => self.parse_delete_statement(),
            TokenKind::Update => self.parse_update_statement(),
            TokenKind::Use => self.parse_use_statement(),
            TokenKind::Show => self.parse_show_statement(),
            TokenKind::Explain => self.parse_explain_statement(),
            TokenKind::Lookup => self.parse_lookup_statement(),
            TokenKind::Fetch => self.parse_fetch_statement(),
            TokenKind::Unwind => self.parse_unwind_statement(),
            TokenKind::Merge => self.parse_merge_statement(),
            TokenKind::Insert => self.parse_insert_statement(),
            TokenKind::Return => self.parse_return_statement(),
            TokenKind::With => self.parse_with_statement(),
            TokenKind::Set => self.parse_set_statement(),
            TokenKind::Remove => self.parse_remove_statement(),
            TokenKind::Pipe => self.parse_pipe_statement(),
            _ => Err(ParseError::syntax_error(
                format!("Unexpected token: {:?}", self.current_token().kind),
                self.current_token().line,
                self.current_token().column,
            )),
        }
    }
    
    pub fn parse_match_statement(&mut self) -> Result<Stmt, ParseError> {
        self.expect_token(TokenKind::Match)?;
        
        let patterns = self.parse_patterns()?;
        
        let where_clause = if self.current_token().kind == TokenKind::Where {
            Some(self.parse_where_clause()?.condition)
        } else {
            None
        };
        
        let return_clause = if self.current_token().kind == TokenKind::Return {
            Some(self.parse_return_clause()?)
        } else {
            None
        };
        
        let order_by = if self.current_token().kind == TokenKind::Order {
            Some(self.parse_order_by_clause()?)
        } else {
            None
        };
        
        let _skip = if self.current_token().kind == TokenKind::Skip {
            Some(self.parse_skip_clause()?.count)
        } else {
            None
        };
        
        let _limit = if self.current_token().kind == TokenKind::Limit {
            Some(self.parse_limit_clause()?.count)
        } else {
            None
        };
        
        Ok(Stmt::Match(MatchStmt {
            span: self.current_span(),
            patterns,
            where_clause,
            return_clause,
            order_by,
            skip: None,
            limit: None,
        }))
    }
    
    fn parse_patterns(&mut self) -> Result<Vec<Pattern>, ParseError> {
        let mut patterns = Vec::new();
        
        patterns.push(self.parse_pattern()?);
        
        while self.current_token().kind == TokenKind::Comma {
            self.next_token();
            patterns.push(self.parse_pattern()?);
        }
        
        Ok(patterns)
    }
    
    pub fn parse_update_statement(&mut self) -> Result<Stmt, ParseError> {
        self.expect_token(TokenKind::Update)?;
        
        let target = self.parse_update_target()?;
        
        let set_clause = self.parse_set_clause()?;
        
        let where_clause = if self.current_token().kind == TokenKind::Where {
            Some(self.parse_where_clause()?.condition)
        } else {
            None
        };
        
        Ok(Stmt::Update(UpdateStmt {
            span: self.current_span(),
            target,
            set_clause,
            where_clause,
        }))
    }
    
    fn parse_update_target(&mut self) -> Result<UpdateTarget, ParseError> {
        let expr = self.parse_expression()?;
        
        Ok(UpdateTarget::Vertex(expr))
    }
    
    pub fn parse_use_statement(&mut self) -> Result<Stmt, ParseError> {
        self.expect_token(TokenKind::Use)?;
        
        let space = self.parse_identifier()?;
        
        Ok(Stmt::Use(UseStmt {
            span: self.current_span(),
            space,
        }))
    }
    
    pub fn parse_show_statement(&mut self) -> Result<Stmt, ParseError> {
        self.expect_token(TokenKind::Show)?;
        
        let target = match self.current_token().kind {
            TokenKind::Spaces => {
                self.next_token();
                ShowTarget::Spaces
            }
            TokenKind::Tags => {
                self.next_token();
                ShowTarget::Tags
            }
            TokenKind::Edges => {
                self.next_token();
                ShowTarget::Edges
            }
            TokenKind::Tag => {
                self.next_token();
                let name = self.parse_identifier()?;
                ShowTarget::Tag(name)
            }
            TokenKind::Edge => {
                self.next_token();
                let name = self.parse_identifier()?;
                ShowTarget::Edge(name)
            }
            TokenKind::Indexes => {
                self.next_token();
                ShowTarget::Indexes
            }
            _ => {
                let name = self.parse_identifier()?;
                ShowTarget::Tag(name)
            }
        };
        
        Ok(Stmt::Show(ShowStmt {
            span: self.current_span(),
            target,
        }))
    }
    
    pub fn parse_explain_statement(&mut self) -> Result<Stmt, ParseError> {
        self.expect_token(TokenKind::Explain)?;
        
        let statement = Box::new(self.parse_statement()?);
        
        Ok(Stmt::Explain(ExplainStmt {
            span: self.current_span(),
            statement,
        }))
    }
    
    pub fn parse_lookup_statement(&mut self) -> Result<Stmt, ParseError> {
        self.expect_token(TokenKind::Lookup)?;
        
        self.expect_token(TokenKind::On)?;
        
        let target = match self.current_token().kind {
            TokenKind::Tag => {
                self.next_token();
                let name = self.parse_identifier()?;
                LookupTarget::Tag(name)
            }
            TokenKind::Edge => {
                self.next_token();
                let name = self.parse_identifier()?;
                LookupTarget::Edge(name)
            }
            _ => {
                let name = self.parse_identifier()?;
                LookupTarget::Tag(name)
            }
        };
        
        let where_clause = if self.current_token().kind == TokenKind::Where {
            Some(self.parse_where_clause()?.condition)
        } else {
            None
        };
        
        let yield_clause = if self.current_token().kind == TokenKind::Yield {
            Some(self.parse_yield_clause()?)
        } else {
            None
        };
        
        Ok(Stmt::Lookup(LookupStmt {
            span: self.current_span(),
            target,
            where_clause,
            yield_clause,
        }))
    }
    
    pub fn parse_fetch_statement(&mut self) -> Result<Stmt, ParseError> {
        self.expect_token(TokenKind::Fetch)?;
        
        self.expect_token(TokenKind::Prop)?;
        
        self.expect_token(TokenKind::On)?;
        
        let ids = self.parse_expression_list()?;
        
        let yield_clause = if self.current_token().kind == TokenKind::Yield {
            Some(self.parse_yield_clause()?)
        } else {
            None
        };
        
        Ok(Stmt::Fetch(FetchStmt {
            span: self.current_span(),
            target: FetchTarget::Vertices {
                ids,
                properties: None,
            },
        }))
    }
    
    fn parse_expression_list(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut expressions = Vec::new();
        
        expressions.push(self.parse_expression()?);
        
        while self.current_token().kind == TokenKind::Comma {
            self.next_token();
            expressions.push(self.parse_expression()?);
        }
        
        Ok(expressions)
    }
    
    pub fn parse_unwind_statement(&mut self) -> Result<Stmt, ParseError> {
        self.expect_token(TokenKind::Unwind)?;
        
        let expression = self.parse_expression()?;
        
        self.expect_token(TokenKind::As)?;
        
        let variable = self.parse_identifier()?;
        
        Ok(Stmt::Unwind(UnwindStmt {
            span: self.current_span(),
            expression,
            variable,
        }))
    }
    
    pub fn parse_merge_statement(&mut self) -> Result<Stmt, ParseError> {
        self.expect_token(TokenKind::Merge)?;
        
        let pattern = self.parse_pattern()?;
        
        Ok(Stmt::Merge(MergeStmt {
            span: self.current_span(),
            pattern,
        }))
    }
    
    pub fn parse_insert_statement(&mut self) -> Result<Stmt, ParseError> {
        self.expect_token(TokenKind::Insert)?;
        
        let target = self.parse_insert_target()?;
        
        Ok(Stmt::Insert(InsertStmt {
            span: self.current_span(),
            target,
        }))
    }
    
    fn parse_insert_target(&mut self) -> Result<InsertTarget, ParseError> {
        match self.current_token().kind {
            TokenKind::Vertex => {
                self.next_token();
                let ids = self.parse_expression_list()?;
                Ok(InsertTarget::Vertices { ids })
            }
            TokenKind::Edge => {
                self.next_token();
                let src = self.parse_expression()?;
                let dst = self.parse_expression()?;
                Ok(InsertTarget::Edge { src, dst })
            }
            _ => Err(ParseError::syntax_error(
                format!("Expected VERTEX or EDGE, got {:?}", self.current_token().kind),
                self.current_token().line,
                self.current_token().column,
            )),
        }
    }
    
    pub fn parse_return_statement(&mut self) -> Result<Stmt, ParseError> {
        self.expect_token(TokenKind::Return)?;
        
        let distinct = if self.current_token().kind == TokenKind::Distinct {
            self.next_token();
            true
        } else {
            false
        };
        
        let mut items = Vec::new();
        
        loop {
            if self.current_token().kind == TokenKind::Eof
                || matches!(
                    self.current_token().kind,
                    TokenKind::Semicolon
                        | TokenKind::Order
                        | TokenKind::Limit
                        | TokenKind::Skip
                )
            {
                break;
            }
            
            if self.current_token().kind == TokenKind::Star {
                items.push(ReturnItem::All);
                self.next_token();
            } else {
                let expr = self.parse_expression()?;
                
                let alias = if self.current_token().kind == TokenKind::As {
                    self.next_token();
                    Some(self.parse_identifier()?)
                } else if matches!(self.current_token().kind, TokenKind::Identifier(_))
                    && self.peek_token() != TokenKind::Comma
                {
                    Some(self.parse_identifier()?)
                } else {
                    None
                };
                
                items.push(ReturnItem::Expression { expr, alias });
            }
            
            if self.current_token().kind != TokenKind::Comma {
                break;
            }
            self.next_token();
        }
        
        Ok(Stmt::Return(ReturnStmt {
            span: self.current_span(),
            items,
            distinct,
        }))
    }
    
    pub fn parse_with_statement(&mut self) -> Result<Stmt, ParseError> {
        self.expect_token(TokenKind::With)?;
        
        let mut items = Vec::new();
        
        loop {
            if self.current_token().kind == TokenKind::Eof
                || matches!(
                    self.current_token().kind,
                    TokenKind::Semicolon | TokenKind::Where
                )
            {
                break;
            }
            
            if self.current_token().kind == TokenKind::Star {
                items.push(ReturnItem::All);
                self.next_token();
            } else {
                let expr = self.parse_expression()?;
                
                let alias = if self.current_token().kind == TokenKind::As {
                    self.next_token();
                    Some(self.parse_identifier()?)
                } else if matches!(self.current_token().kind, TokenKind::Identifier(_))
                    && self.peek_token() != TokenKind::Comma
                {
                    Some(self.parse_identifier()?)
                } else {
                    None
                };
                
                items.push(ReturnItem::Expression { expr, alias });
            }
            
            if self.current_token().kind != TokenKind::Comma {
                break;
            }
            self.next_token();
        }
        
        let where_clause = if self.current_token().kind == TokenKind::Where {
            Some(self.parse_where_clause()?.condition)
        } else {
            None
        };
        
        Ok(Stmt::With(WithStmt {
            span: self.current_span(),
            items,
            where_clause,
        }))
    }
    
    pub fn parse_set_statement(&mut self) -> Result<Stmt, ParseError> {
        self.expect_token(TokenKind::Set)?;
        
        let mut assignments = Vec::new();
        
        loop {
            let property = self.parse_identifier()?;
            self.expect_token(TokenKind::Assign)?;
            let value = self.parse_expression()?;
            
            assignments.push(Assignment { property, value });
            
            if self.current_token().kind != TokenKind::Comma {
                break;
            }
            self.next_token();
        }
        
        Ok(Stmt::Set(SetStmt {
            span: self.current_span(),
            assignments,
        }))
    }
    
    pub fn parse_remove_statement(&mut self) -> Result<Stmt, ParseError> {
        self.expect_token(TokenKind::Remove)?;
        
        let items = self.parse_expression_list()?;
        
        Ok(Stmt::Remove(RemoveStmt {
            span: self.current_span(),
            items,
        }))
    }
    
    pub fn parse_pipe_statement(&mut self) -> Result<Stmt, ParseError> {
        self.expect_token(TokenKind::Pipe)?;
        
        let expression = self.parse_expression()?;
        
        Ok(Stmt::Pipe(PipeStmt {
            span: self.current_span(),
            expression,
        }))
    }
    
    pub fn parse_from_clause(&mut self) -> Result<FromClause, ParseError> {
        let span = self.current_span();
        self.expect_token(TokenKind::From)?;

        let mut vertices = Vec::new();

        loop {
            let vertex = self.parse_expression()?;
            vertices.push(vertex);

            if self.current_token().kind != TokenKind::Comma {
                break;
            }
            self.next_token();
        }

        Ok(FromClause { span, vertices })
    }
    
    pub fn parse_over_clause(&mut self) -> Result<OverClause, ParseError> {
        let span = self.current_span();
        self.expect_token(TokenKind::Over)?;

        let mut edge_types = Vec::new();
        let mut direction = EdgeDirection::Outgoing;

        loop {
            let edge_type = self.parse_identifier()?;
            edge_types.push(edge_type);

            if self.current_token().kind != TokenKind::Comma {
                break;
            }
            self.next_token();
        }

        if self.current_token().kind == TokenKind::Out {
            self.next_token();
            direction = EdgeDirection::Outgoing;
        } else if self.current_token().kind == TokenKind::In {
            self.next_token();
            direction = EdgeDirection::Incoming;
        } else if self.current_token().kind == TokenKind::Both {
            self.next_token();
            direction = EdgeDirection::Both;
        }

        Ok(OverClause {
            span,
            edge_types,
            direction,
        })
    }
}
