//! MATCH语句解析器

use crate::query::parser::core::error::ParseError;
use crate::query::parser::core::token::TokenKind;
use crate::query::parser::ast::*;
use crate::query::parser::expressions::ExpressionParser;

pub trait MatchStatementParser: ExpressionParser {
    /// 解析MATCH语句
    fn parse_match_statement(&mut self) -> Result<Option<Box<dyn Statement>>, ParseError> {
        // Parse match patterns
        let mut clauses = Vec::new();

        // Parse the pattern part of MATCH
        let patterns = self.parse_match_patterns()?;
        let where_clause = if self.current_token().kind == TokenKind::Where {
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
        if self.current_token().kind == TokenKind::Return {
            clauses.push(MatchClause::Return(self.parse_return_clause()?));
        }

        Ok(Some(Box::new(MatchStatement {
            base: BaseStatement::new(Span::default(), StatementType::Match),
            clauses,
        })))
    }

    fn parse_match_patterns(&mut self) -> Result<Vec<MatchPath>, ParseError> {
        let mut patterns = Vec::new();

        // For now, just parse a simple path pattern
        // In a real implementation, we'd have more complex pattern parsing
        let path = self.parse_match_path()?;
        patterns.push(path);

        Ok(patterns)
    }

    fn parse_match_path(&mut self) -> Result<MatchPath, ParseError> {
        let mut path = Vec::new();

        // Parse nodes and edges in the path
        loop {
            // Parse a node
            if self.current_token().kind == TokenKind::LParen {
                path.push(MatchPathSegment::Node(self.parse_match_node()?));
            } else {
                break;
            }

            // Check if there's an edge following
            if self.current_token().kind == TokenKind::Arrow ||
               self.current_token().kind == TokenKind::BackArrow ||
               matches!(self.current_token().kind, TokenKind::Minus) {
                path.push(MatchPathSegment::Edge(self.parse_match_edge()?));
            } else {
                break;
            }
        }

        Ok(MatchPath { path })
    }

    fn parse_match_node(&mut self) -> Result<MatchNode, ParseError> {
        self.expect_token(TokenKind::LParen)?;

        // Parse optional identifier
        let identifier = if matches!(self.current_token().kind, TokenKind::Identifier(_)) {
            let id = self.parse_identifier()?;
            if self.current_token().kind == TokenKind::Colon {
                // There's a label following
                Some(id)
            } else {
                // No label, just identifier
                self.expect_token(TokenKind::RParen)?;
                return Ok(MatchNode {
                    identifier: Some(id),
                    labels: vec![],
                    properties: None,
                    predicates: vec![],
                });
            }
        } else {
            None
        };

        // Parse optional label
        let mut labels = Vec::new();
        if self.current_token().kind == TokenKind::Colon {
            self.next_token();
            labels.push(Label { name: self.parse_identifier()? });
        }

        // Parse optional properties
        let properties = if self.current_token().kind == TokenKind::LBrace {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.expect_token(TokenKind::RParen)?;

        Ok(MatchNode {
            identifier,
            labels,
            properties,
            predicates: vec![],
        })
    }

    fn parse_match_edge(&mut self) -> Result<MatchEdge, ParseError> {
        let direction = match self.current_token().kind {
            TokenKind::Arrow => {
                self.next_token();
                EdgeDirection::Outbound
            }
            TokenKind::BackArrow => {
                self.next_token();
                EdgeDirection::Inbound
            }
            TokenKind::Minus => {
                self.next_token();
                EdgeDirection::Bidirectional
            }
            _ => {
                return Err(ParseError::syntax_error(
                    format!("Expected edge direction (->, <-, -), got {:?}", self.current_token().kind),
                    self.current_token().line,
                    self.current_token().column,
                ));
            }
        };

        // Check if it's followed by an edge type in brackets [type]
        let mut types = Vec::new();
        let mut identifier = None;
        let mut properties = None;

        if self.current_token().kind == TokenKind::LBracket {
            self.next_token();

            // Parse optional identifier or type
            if matches!(self.current_token().kind, TokenKind::Identifier(_)) {
                let id = self.parse_identifier()?;

                // Check if it's an identifier with type or just a type
                if self.current_token().kind == TokenKind::Colon {
                    // It's identifier:type format
                    identifier = Some(id);
                    self.next_token();
                    types.push(self.parse_identifier()?);
                } else {
                    // Just a type
                    types.push(id);
                }
            }

            // Parse optional properties
            if self.current_token().kind == TokenKind::LBrace {
                properties = Some(self.parse_expression()?);
            }

            self.expect_token(TokenKind::RBracket)?;
        }

        Ok(MatchEdge {
            direction,
            identifier,
            types,
            relationship: None,
            properties,
            predicates: vec![],
            range: None,
        })
    }

    fn parse_where_clause(&mut self) -> Result<crate::query::parser::ast::compat::WhereClause, ParseError>;
    fn parse_return_clause(&mut self) -> Result<crate::query::parser::ast::compat::ReturnClause, ParseError>;
}