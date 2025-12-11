//! UPDATE语句解析器

use crate::query::parser::core::error::ParseError;
use crate::query::parser::core::token::{Token, TokenKind};
use crate::query::parser::ast::*;
use crate::query::parser::expressions::{ExpressionParser, TokenParser};

pub trait UpdateStatementParser: ExpressionParser {
    /// 解析UPDATE语句
    fn parse_update_statement(&mut self) -> Result<Option<Statement>, ParseError> {
        let update_vertices = match self.current_token().kind {
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
                    format!("Expected VERTEX or EDGE after UPDATE, got {:?}", self.current_token().kind),
                    self.current_token().line,
                    self.current_token().column,
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

            if self.current_token().kind != TokenKind::Comma {
                break;
            }
            self.next_token(); // Skip comma
        }

        // Optionally parse WHERE clause
        let condition = if self.current_token().kind == TokenKind::Where {
            Some(self.parse_where_clause()?)
        } else {
            None
        };

        // Optionally parse YIELD clause
        let yield_clause = if self.current_token().kind == TokenKind::Yield {
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

    fn parse_where_clause(&mut self) -> Result<WhereClause, ParseError>;
    fn parse_yield_clause(&mut self) -> Result<YieldClause, ParseError>;
    fn parse_property_ref(&mut self) -> Result<PropertyRef, ParseError>;
}