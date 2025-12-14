//! Pattern parsing module for the query parser
//!
//! This module implements parsing for pattern matching in the query language.

use crate::query::parser::ast::statement::{OrderByItem, ReturnItem};
use crate::query::parser::ast::*;
use crate::query::parser::core::error::ParseError;
use crate::query::parser::core::token::TokenKind;

impl super::Parser {
    pub fn parse_match_patterns(&mut self) -> Result<Vec<MatchPath>, ParseError> {
        let mut patterns = Vec::new();

        // For now, just parse a simple path pattern
        // In a real implementation, we'd have more complex pattern parsing
        let path = self.parse_match_path()?;
        patterns.push(path);

        Ok(patterns)
    }

    pub fn parse_match_path(&mut self) -> Result<MatchPath, ParseError> {
        let mut path = Vec::new();

        // Parse nodes and edges in the path
        loop {
            // Parse a node
            if self.current_token.kind == TokenKind::LParen {
                path.push(MatchPathSegment::Node(self.parse_match_node()?));
            } else {
                break;
            }

            // Check if there's an edge following
            if self.current_token.kind == TokenKind::Arrow
                || self.current_token.kind == TokenKind::BackArrow
                || matches!(self.current_token.kind, TokenKind::Minus)
            {
                path.push(MatchPathSegment::Edge(self.parse_match_edge()?));
            } else {
                break;
            }
        }

        Ok(MatchPath { path })
    }

    pub fn parse_match_node(&mut self) -> Result<MatchNode, ParseError> {
        self.expect_token(TokenKind::LParen)?;

        // Parse optional identifier
        let identifier = if matches!(self.current_token.kind, TokenKind::Identifier(_)) {
            let id = self.parse_identifier()?;
            if self.current_token.kind == TokenKind::Colon {
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
        if self.current_token.kind == TokenKind::Colon {
            self.next_token();
            labels.push(Label {
                name: self.parse_identifier()?,
            });
        }

        // Parse optional properties
        let properties = if self.current_token.kind == TokenKind::LBrace {
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

    pub fn parse_match_edge(&mut self) -> Result<MatchEdge, ParseError> {
        let direction = match self.current_token.kind {
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
                    format!(
                        "Expected edge direction (->, <-, -), got {:?}",
                        self.current_token.kind
                    ),
                    self.current_token.line,
                    self.current_token.column,
                ));
            }
        };

        // Check if it's followed by an edge type in brackets [type]
        let mut types = Vec::new();
        let mut identifier = None;
        let mut properties = None;

        if self.current_token.kind == TokenKind::LBracket {
            self.next_token();

            // Parse optional identifier or type
            if matches!(self.current_token.kind, TokenKind::Identifier(_)) {
                let id = self.parse_identifier()?;

                // Check if it's an identifier with type or just a type
                if self.current_token.kind == TokenKind::Colon {
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
            if self.current_token.kind == TokenKind::LBrace {
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

    pub fn parse_where_clause(&mut self) -> Result<WhereClause, ParseError> {
        self.next_token(); // Skip WHERE
        let condition = self.parse_expression()?;
        Ok(WhereClause {
            expression: condition,
        })
    }

    pub fn parse_return_clause(
        &mut self,
    ) -> Result<crate::query::parser::ast::compat::ReturnClause, ParseError> {
        self.next_token(); // Skip RETURN

        let distinct = if self.current_token.kind == TokenKind::Distinct {
            self.next_token();
            true
        } else {
            false
        };

        let mut items = Vec::new();

        // Parse return items
        loop {
            if self.current_token.kind == TokenKind::Eof
                || matches!(
                    self.current_token.kind,
                    TokenKind::Semicolon | TokenKind::Order | TokenKind::Limit | TokenKind::Skip
                )
            {
                break;
            }

            if self.current_token.kind == TokenKind::Star {
                items.push(ReturnItem::All);
                self.next_token();
            } else {
                let expr = self.parse_expression()?;

                let alias = if self.current_token.kind == TokenKind::As {
                    self.next_token();
                    Some(self.parse_identifier()?)
                } else if matches!(self.current_token.kind, TokenKind::Identifier(_))
                    && self.peek_token() != TokenKind::Comma
                {
                    // Potential alias without AS
                    Some(self.parse_identifier()?)
                } else {
                    None
                };

                items.push(ReturnItem::Expression(expr, alias));
            }

            if self.current_token.kind != TokenKind::Comma {
                break;
            }
            self.next_token(); // Skip comma
        }

        Ok(ReturnClause { distinct, items })
    }

    pub fn parse_order_by_clause(&mut self) -> Result<OrderByClause, ParseError> {
        self.expect_token(TokenKind::Order)?;
        self.expect_token(TokenKind::By)?;

        let mut items = Vec::new();

        loop {
            let expr = self.parse_expression()?;

            let order = if self.current_token.kind == TokenKind::Asc
                || self.current_token.kind == TokenKind::Ascending
            {
                self.next_token();
                "ASC"
            } else if self.current_token.kind == TokenKind::Desc
                || self.current_token.kind == TokenKind::Descending
            {
                self.next_token();
                "DESC"
            } else {
                "ASC" // Default to ascending
            };

            items.push(OrderByItem {
                expression: expr,
                ascending: order == "ASC",
            });

            if self.current_token.kind != TokenKind::Comma {
                break;
            }
            self.next_token(); // Skip comma
        }

        Ok(OrderByClause { items })
    }

    pub fn parse_limit_clause(&mut self) -> Result<Box<dyn Expression>, ParseError> {
        self.next_token(); // Skip LIMIT
        self.parse_expression()
    }

    pub fn parse_skip_clause(&mut self) -> Result<Box<dyn Expression>, ParseError> {
        self.next_token(); // Skip SKIP
        self.parse_expression()
    }
}
