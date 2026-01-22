//! Pattern parsing module for the query parser
//!
//! This module implements parsing for pattern matching in the query language.

use crate::query::parser::ast::*;
use crate::query::parser::{ParseError, TokenKind};

impl super::Parser {
    pub fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        let path_elements = self.parse_match_path()?;
        
        if path_elements.len() == 1 {
            match &path_elements[0] {
                PathElement::Node(node) => Ok(Pattern::Node(node.clone())),
                PathElement::Edge(edge) => Ok(Pattern::Edge(edge.clone())),
                _ => Err(ParseError::syntax_error(
                    "Invalid pattern".to_string(),
                    self.current_token().line,
                    self.current_token().column,
                )),
            }
        } else {
            Ok(Pattern::Path(PathPattern {
                span: Span::default(),
                elements: path_elements,
            }))
        }
    }

    pub fn parse_match_patterns(&mut self) -> Result<Vec<PathElement>, ParseError> {
        let mut patterns = Vec::new();

        // For now, just parse a simple path pattern
        // In a real implementation, we'd have more complex pattern parsing
        let path = self.parse_match_path()?;
        patterns.extend(path);

        Ok(patterns)
    }

    pub fn parse_match_path(&mut self) -> Result<Vec<PathElement>, ParseError> {
        let mut path = Vec::new();

        // Parse nodes and edges in the path
        loop {
            // Parse a node
            if self.current_token().kind == TokenKind::LParen {
                path.push(PathElement::Node(self.parse_match_node()?));
            } else {
                break;
            }

            // Check if there's an edge following
            if self.current_token().kind == TokenKind::Arrow
                || self.current_token().kind == TokenKind::BackArrow
                || matches!(self.current_token().kind, TokenKind::Minus)
            {
                path.push(PathElement::Edge(self.parse_match_edge()?));
            } else {
                break;
            }
        }

        Ok(path)
    }

    pub fn parse_match_node(&mut self) -> Result<NodePattern, ParseError> {
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
                return Ok(NodePattern::new(
                    Some(id),
                    vec![],
                    None,
                    vec![],
                    Span::default(),
                ));
            }
        } else {
            None
        };

        // Parse optional label
        let mut labels = Vec::new();
        if self.current_token().kind == TokenKind::Colon {
            self.next_token();
            labels.push(self.parse_identifier()?);
        }

        // Parse optional properties
        let properties = if self.current_token().kind == TokenKind::LBrace {
            Some(self.parse_expression().map_err(|e| {
                ParseError::syntax_error(
                    e.message,
                    self.current_token().line,
                    self.current_token().column,
                )
            })?)
        } else {
            None
        };

        self.expect_token(TokenKind::RParen)?;

        Ok(NodePattern::new(
            identifier,
            labels,
            properties,
            vec![],
            Span::default(),
        ))
    }

    pub fn parse_match_edge(&mut self) -> Result<EdgePattern, ParseError> {
        let direction = match self.current_token().kind {
            TokenKind::Arrow => {
                self.next_token();
                EdgeDirection::Outgoing
            }
            TokenKind::BackArrow => {
                self.next_token();
                EdgeDirection::Incoming
            }
            TokenKind::Minus => {
                self.next_token();
                EdgeDirection::Both
            }
            _ => {
                return Err(ParseError::syntax_error(
                    format!(
                        "Expected edge direction (->, <-, -), got {:?}",
                        self.current_token().kind
                    ),
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
                properties = Some(self.parse_expression().map_err(|e| {
                    ParseError::syntax_error(
                        e.message,
                        self.current_token().line,
                        self.current_token().column,
                    )
                })?);
            }

            self.expect_token(TokenKind::RBracket)?;
        }

        Ok(EdgePattern::new(
            identifier,
            types,
            properties,
            vec![],
            direction,
            None,
            Span::default(),
        ))
    }
    
    pub fn parse_patterns(&mut self) -> Result<Vec<Pattern>, ParseError> {
        let mut patterns = Vec::new();
        loop {
            patterns.push(self.parse_pattern()?);
            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }
        Ok(patterns)
    }
}
