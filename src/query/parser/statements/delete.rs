//! DELETE语句解析器

use crate::query::parser::core::error::ParseError;
use crate::query::parser::core::token::{Token, TokenKind};
use crate::query::parser::ast::*;
use crate::query::parser::expressions::{ExpressionParser, TokenParser};

pub trait DeleteStatementParser: ExpressionParser {
    /// 解析DELETE语句
    fn parse_delete_statement(&mut self) -> Result<Option<Statement>, ParseError> {
        let delete_vertices = match self.current_token().kind {
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
                    format!("Expected VERTEX or EDGE after DELETE, got {:?}", self.current_token().kind),
                    self.current_token().line,
                    self.current_token().column,
                ));
            }
        };

        // For simplicity, just parsing expression list
        let mut vertex_exprs = Vec::new();
        loop {
            vertex_exprs.push(self.parse_expression()?);

            if self.current_token().kind != TokenKind::Comma {
                break;
            }
            self.next_token(); // Skip comma
        }

        // Optionally parse WHERE clause
        let where_clause = if self.current_token().kind == TokenKind::Where {
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

        Ok(Some(Statement::Delete(DeleteStatement {
            delete_vertices,
            vertex_exprs,
            edge_exprs: None,  // Simplified for now
            where_clause,
            yield_clause,
        })))
    }

    fn parse_where_clause(&mut self) -> Result<WhereClause, ParseError>;
    fn parse_yield_clause(&mut self) -> Result<YieldClause, ParseError>;
}