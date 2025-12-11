//! CREATE语句解析器

use crate::query::parser::core::{ParseError, Token, TokenKind};
use crate::query::parser::ast::*;
use crate::query::parser::expressions::ExpressionParser;
use std::collections::HashMap;

pub trait CreateStatementParser: ExpressionParser {
    /// 解析CREATE语句
    fn parse_create_statement(&mut self) -> Result<Option<Statement>, ParseError> {
        match self.current_token().kind {
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
                    format!("Expected VERTEX or EDGE after CREATE, got {:?}", self.current_token().kind),
                    self.current_token().line,
                    self.current_token().column,
                );
                self.add_error(error.clone());
                Err(error)
            }
        }
    }

    /// 解析CREATE VERTEX语句
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
        let properties = self.parse_property_list()?;

        // Optionally parse YIELD clause
        let yield_clause = if self.current_token().kind == TokenKind::Yield {
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

    /// 解析CREATE EDGE语句
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
        let direction = if self.current_token().kind == TokenKind::Arrow { // ->
            self.next_token();
            EdgeDirection::Outbound
        } else if self.current_token().kind == TokenKind::BackArrow { // <-
            self.next_token();
            EdgeDirection::Inbound
        } else {
            return Err(ParseError::syntax_error(
                format!("Expected -> or <-, got {:?}", self.current_token().kind),
                self.current_token().line,
                self.current_token().column,
            ));
        };

        self.expect_token(TokenKind::LParen)?;
        let dst = self.parse_expression()?;
        self.expect_token(TokenKind::RParen)?;

        // Parse SET clause
        self.expect_token(TokenKind::Set)?;
        let properties = self.parse_property_list()?;

        // Optionally parse YIELD clause
        let yield_clause = if self.current_token().kind == TokenKind::Yield {
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

    fn parse_tag_list(&mut self) -> Result<Vec<TagIdentifier>, ParseError>;
    fn parse_property_list(&mut self) -> Result<Vec<Property>, ParseError>;
    fn parse_yield_clause(&mut self) -> Result<YieldClause, ParseError>;
    fn parse_identifier(&mut self) -> Result<String, ParseError>;
    fn check_and_skip_keyword(&mut self, keyword: TokenKind) -> bool;
    fn expect_token(&mut self, expected: TokenKind) -> Result<Token, ParseError>;
    fn add_error(&mut self, error: ParseError);
    fn current_token(&self) -> &Token;
    fn next_token(&mut self);
}