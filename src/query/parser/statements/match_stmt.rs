//! MATCH语句解析器

use crate::query::parser::ast::*;
use crate::query::parser::expressions::ExpressionParser;
use crate::query::parser::{ParseError, TokenKind};

pub trait MatchStmtParser: ExpressionParser {
    /// 解析MATCH语句
    fn parse_match_statement(&mut self) -> Result<Option<Stmt>, ParseError> {
        // Parse the pattern part of MATCH
        let patterns = self.parse_match_patterns()?;
        let where_clause = if self.current_token().kind == TokenKind::Where {
            Some(self.parse_expression()?)
        } else {
            None
        };

        // Parse optional RETURN clause
        let return_clause = if self.current_token().kind == TokenKind::Return {
            Some(self.parse_return_clause()?)
        } else {
            None
        };

        Ok(Some(Stmt::Match(MatchStmt {
            span: Span::default(),
            patterns,
            where_clause,
            return_clause,
            order_by: None,
            limit: None,
            skip: None,
        })))
    }

    fn parse_match_patterns(&mut self) -> Result<Vec<Pattern>, ParseError> {
        let mut patterns = Vec::new();

        // For now, just parse a simple path pattern
        // In a real implementation, we'd have more complex pattern parsing
        let path = self.parse_match_path()?;
        patterns.push(path);

        Ok(patterns)
    }

    fn parse_match_path(&mut self) -> Result<Pattern, ParseError> {
        // 简化实现：只解析一个节点模式
        let node_pattern = self.parse_match_node()?;
        Ok(Pattern::Node(node_pattern))
    }

    fn parse_match_node(&mut self) -> Result<NodePattern, ParseError> {
        self.expect_token(TokenKind::LParen)?;

        // Parse optional identifier
        let variable = if matches!(self.current_token().kind, TokenKind::Identifier(_)) {
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
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.expect_token(TokenKind::RParen)?;

        Ok(NodePattern::new(
            variable,
            labels,
            properties,
            vec![],
            Span::default(),
        ))
    }

    fn parse_match_edge(&mut self) -> Result<EdgePattern, ParseError> {
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
        let mut edge_types = Vec::new();
        let mut variable = None;
        let mut properties = None;

        if self.current_token().kind == TokenKind::LBracket {
            self.next_token();

            // Parse optional identifier or type
            if matches!(self.current_token().kind, TokenKind::Identifier(_)) {
                let id = self.parse_identifier()?;

                // Check if it's an identifier with type or just a type
                if self.current_token().kind == TokenKind::Colon {
                    // It's identifier:type format
                    variable = Some(id);
                    self.next_token();
                    edge_types.push(self.parse_identifier()?);
                } else {
                    // Just a type
                    edge_types.push(id);
                }
            }

            // Parse optional properties
            if self.current_token().kind == TokenKind::LBrace {
                properties = Some(self.parse_expression()?);
            }

            self.expect_token(TokenKind::RBracket)?;
        }

        Ok(EdgePattern::new(
            variable,
            edge_types,
            properties,
            vec![],
            direction,
            None,
            Span::default(),
        ))
    }

    fn parse_where_clause(&mut self) -> Result<Expr, ParseError>;
    fn parse_return_clause(&mut self) -> Result<ReturnClause, ParseError>;
}
