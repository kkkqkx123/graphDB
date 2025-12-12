//! Statement parsing module for the query parser
//!
//! This module implements parsing for statements in the query language.

use crate::query::parser::lexer::lexer::Lexer;
use crate::query::parser::core::token::{Token, TokenKind};
use crate::query::parser::ast::*;
use crate::query::parser::core::error::{ParseError, ParseErrors};

impl super::Parser {
    pub fn parse(&mut self) -> Result<Vec<Statement>, ParseErrors> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            if let Some(stmt) = self.parse_statement().map_err(|e| ParseErrors::from(vec![e]))? {
                statements.push(stmt);
            }
        }

        if !self.errors.is_empty() {
            return Err(self.errors.clone());
        }

        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<Option<Statement>, ParseError> {
        match self.current_token.kind {
            TokenKind::Create => {
                self.next_token();
                self.parse_create_statement()
            }
            TokenKind::Match => {
                self.next_token();
                self.parse_match_statement()
            }
            TokenKind::Delete => {
                self.next_token();
                self.parse_delete_statement()
            }
            TokenKind::Update => {
                self.next_token();
                self.parse_update_statement()
            }
            TokenKind::Use => {
                self.next_token();
                self.parse_use_statement()
            }
            TokenKind::Show => {
                self.next_token();
                self.parse_show_statement()
            }
            TokenKind::Explain => {
                self.next_token();
                self.parse_explain_statement()
            }
            TokenKind::Semicolon => {
                // Skip standalone semicolons
                self.next_token();
                Ok(None)
            }
            TokenKind::Eof => Ok(None),
            _ => {
                let error = ParseError::syntax_error(
                    format!("Unexpected token: {:?}", self.current_token.kind),
                    self.current_token.line,
                    self.current_token.column,
                );
                self.errors.add(error.clone());
                Err(error)
            }
        }
    }

    fn parse_create_statement(&mut self) -> Result<Option<Statement>, ParseError> {
        match self.current_token.kind {
            TokenKind::Vertex | TokenKind::Vertices => {
                self.next_token();
                self.parse_create_node_statement()
            }
            TokenKind::Edge | TokenKind::Edges => {
                self.next_token();
                self.parse_create_edge_statement()
            }
            _ => {
                let error = ParseError::syntax_error(
                    format!("Expected VERTEX or EDGE after CREATE, got {:?}", self.current_token.kind),
                    self.current_token.line,
                    self.current_token.column,
                );
                self.errors.add(error.clone());
                Err(error)
            }
        }
    }

    fn parse_create_node_statement(&mut self) -> Result<Option<Statement>, ParseError> {
        let if_not_exists = self.check_and_skip_keyword(TokenKind::If);

        // Skip 'EXISTS' if we found 'IF'
        if if_not_exists {
            self.expect_token(TokenKind::Exists)?;
        }

        // Parse tag list
        let tags = self.parse_tag_list()?;

        // Parse SET clause
        self.expect_token(TokenKind::Set)?;

        // Properties can be in two forms: SET prop = value or SET {prop: value}
        let properties = if self.current_token.kind == TokenKind::LBrace {
            // Handle SET {prop: value} form
            let map = self.parse_property_map()?;
            // Convert HashMap to Vec<Property>
            map.into_iter()
                .map(|(name, value)| Property { name, value })
                .collect()
        } else {
            // Handle SET prop = value form
            self.parse_property_list()?
        };

        // Optionally parse YIELD clause
        let yield_clause = if self.current_token.kind == TokenKind::Yield {
            Some(self.parse_yield_clause()?)
        } else {
            None
        };

        Ok(Some(Statement::CreateNode(CreateNodeStatement {
            if_not_exists,
            tags,
            properties,
            yield_clause,
        })))
    }

    fn parse_create_edge_statement(&mut self) -> Result<Option<Statement>, ParseError> {
        let if_not_exists = self.check_and_skip_keyword(TokenKind::If);

        // Skip 'EXISTS' if we found 'IF'
        if if_not_exists {
            self.expect_token(TokenKind::Exists)?;
        }

        // Parse edge type
        let edge_type = self.parse_identifier()?;

        // Parse source and destination
        self.expect_token(TokenKind::LParen)?;
        let src = self.parse_expression()?;
        self.expect_token(TokenKind::RParen)?;

        // Parse edge pattern -> or <-
        let direction = if self.current_token.kind == TokenKind::Arrow { // ->
            self.next_token();
            EdgeDirection::Outbound
        } else if self.current_token.kind == TokenKind::BackArrow { // <-
            self.next_token();
            EdgeDirection::Inbound
        } else {
            return Err(ParseError::syntax_error(
                format!("Expected -> or <-, got {:?}", self.current_token.kind),
                self.current_token.line,
                self.current_token.column,
            ));
        };

        self.expect_token(TokenKind::LParen)?;
        let dst = self.parse_expression()?;
        self.expect_token(TokenKind::RParen)?;

        // Parse SET clause
        self.expect_token(TokenKind::Set)?;
        let properties = self.parse_property_list()?;

        // Optionally parse YIELD clause
        let yield_clause = if self.current_token.kind == TokenKind::Yield {
            Some(self.parse_yield_clause()?)
        } else {
            None
        };

        Ok(Some(Statement::CreateEdge(CreateEdgeStatement {
            if_not_exists,
            edge_type,
            src,
            dst,
            ranking: None, // No ranking in basic implementation
            properties,
            yield_clause,
        })))
    }

    fn parse_match_statement(&mut self) -> Result<Option<Statement>, ParseError> {
        // Parse match patterns
        let mut clauses = Vec::new();

        // Parse the pattern part of MATCH
        let patterns = self.parse_match_patterns()?;
        let where_clause = if self.current_token.kind == TokenKind::Where {
            Some(self.parse_where_clause()?)
        } else {
            None
        };

        clauses.push(MatchClause::Match(MatchClauseDetail {
            patterns,
            where_clause,
            with_clause: None,
        }));

        // Parse optional RETURN clause
        if self.current_token.kind == TokenKind::Return {
            clauses.push(MatchClause::Return(self.parse_return_clause()?));
        }

        Ok(Some(Statement::Match(MatchStatement {
            clauses,
            return_clause: None
        })))
    }

    fn parse_delete_statement(&mut self) -> Result<Option<Statement>, ParseError> {
        let delete_vertices = match self.current_token.kind {
            TokenKind::Vertex | TokenKind::Vertices => {
                self.next_token();
                true
            }
            TokenKind::Edge | TokenKind::Edges => {
                self.next_token();
                false
            }
            _ => {
                return Err(ParseError::syntax_error(
                    format!("Expected VERTEX or EDGE after DELETE, got {:?}", self.current_token.kind),
                    self.current_token.line,
                    self.current_token.column,
                ));
            }
        };

        // For simplicity, just parsing expression list
        let mut vertex_exprs = Vec::new();
        loop {
            vertex_exprs.push(self.parse_expression()?);

            if self.current_token.kind != TokenKind::Comma {
                break;
            }
            self.next_token(); // Skip comma
        }

        // Optionally parse WHERE clause
        let where_clause = if self.current_token.kind == TokenKind::Where {
            Some(self.parse_where_clause()?)
        } else {
            None
        };

        // Optionally parse YIELD clause
        let yield_clause = if self.current_token.kind == TokenKind::Yield {
            Some(self.parse_yield_clause()?)
        } else {
            None
        };

        Ok(Some(Statement::Delete(DeleteStatement {
            delete_vertices,
            vertex_exprs,
            edge_exprs: None,  // Simplified for now
            where_clause,
            yield_clause,
        })))
    }

    fn parse_update_statement(&mut self) -> Result<Option<Statement>, ParseError> {
        let update_vertices = match self.current_token.kind {
            TokenKind::Vertex => {
                self.next_token();
                true
            }
            TokenKind::Edge => {
                self.next_token();
                false
            }
            _ => {
                return Err(ParseError::syntax_error(
                    format!("Expected VERTEX or EDGE after UPDATE, got {:?}", self.current_token.kind),
                    self.current_token.line,
                    self.current_token.column,
                ));
            }
        };

        // Parse vertex/edge reference
        let vertex_ref = Some(self.parse_expression()?);

        // Parse SET clause
        self.expect_token(TokenKind::Set)?;
        let mut update_items = Vec::new();

        loop {
            let prop = self.parse_property_ref()?;
            self.expect_token(TokenKind::Assign)?;
            let value = self.parse_expression()?;

            update_items.push(Assignment { prop, value });

            if self.current_token.kind != TokenKind::Comma {
                break;
            }
            self.next_token(); // Skip comma
        }

        // Optionally parse WHERE clause
        let condition = if self.current_token.kind == TokenKind::Where {
            Some(self.parse_where_clause()?)
        } else {
            None
        };

        // Optionally parse YIELD clause
        let yield_clause = if self.current_token.kind == TokenKind::Yield {
            Some(self.parse_yield_clause()?)
        } else {
            None
        };

        Ok(Some(Statement::Update(UpdateStatement {
            update_vertices,
            vertex_ref,
            edge_ref: None,  // Simplified for now
            update_items,
            condition,
            yield_clause,
        })))
    }

    fn parse_use_statement(&mut self) -> Result<Option<Statement>, ParseError> {
        self.next_token(); // Skip USE
        let space = self.parse_identifier()?;
        Ok(Some(Statement::Use(UseStatement { space })))
    }

    fn parse_show_statement(&mut self) -> Result<Option<Statement>, ParseError> {
        self.next_token(); // Skip SHOW

        let show_stmt = match self.current_token.kind {
            TokenKind::Spaces => {
                self.next_token();
                ShowStatement::ShowSpaces
            }
            TokenKind::Tags => {
                self.next_token();
                ShowStatement::ShowTags
            }
            TokenKind::Edges => {
                self.next_token();
                ShowStatement::ShowEdges
            }
            TokenKind::Tag => {
                self.next_token();
                ShowStatement::ShowTags
            }
            TokenKind::Edge => {
                self.next_token();
                ShowStatement::ShowEdges
            }
            TokenKind::Users => {
                self.next_token();
                ShowStatement::ShowUsers
            }
            TokenKind::Roles => {
                self.next_token();
                let role = if matches!(self.current_token.kind, TokenKind::Identifier(_)) {
                    Some(self.parse_identifier()?)
                } else {
                    None
                };
                ShowStatement::ShowRoles(role)
            }
            TokenKind::Hosts => {
                self.next_token();
                ShowStatement::ShowHosts
            }
            _ => {
                return Err(ParseError::syntax_error(
                    format!("Unexpected token in SHOW statement: {:?}", self.current_token.kind),
                    self.current_token.line,
                    self.current_token.column,
                ));
            }
        };

        Ok(Some(Statement::Show(show_stmt)))
    }

    fn parse_explain_statement(&mut self) -> Result<Option<Statement>, ParseError> {
        self.next_token(); // Skip EXPLAIN

        // Parse the statement to explain
        let stmt = self.parse_statement()?;
        if let Some(stmt) = stmt {
            Ok(Some(Statement::Explain(ExplainStatement {
                stmt: Box::new(stmt),
            })))
        } else {
            Err(ParseError::syntax_error(
                "Expected statement after EXPLAIN".to_string(),
                self.current_token.line,
                self.current_token.column,
            ))
        }
    }
}