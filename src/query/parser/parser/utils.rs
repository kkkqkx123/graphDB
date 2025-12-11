//! Utility functions for the query parser
//!
//! This module implements common utility functions used across the parser.

use crate::query::parser::lexer::lexer::Lexer;
use crate::query::parser::core::token::{Token, TokenKind};
use crate::query::parser::ast::*;
use crate::query::parser::core::error::{ParseError, ParseErrors};

pub struct Parser {
    pub lexer: Lexer,
    pub current_token: Token,
    pub errors: ParseErrors,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.next_token();

        Self {
            lexer,
            current_token,
            errors: ParseErrors::new(),
        }
    }

    pub fn next_token(&mut self) {
        self.current_token = self.lexer.next_token();
    }

    pub fn peek_token(&mut self) -> TokenKind {
        // Create a copy of the current lexer to peek ahead without changing state
        let mut temp_lexer = self.lexer.clone();
        let next_token = temp_lexer.next_token();
        next_token.kind
    }

    pub fn expect_token(&mut self, expected: TokenKind) -> Result<Token, ParseError> {
        if self.current_token.kind == expected {
            let token = self.current_token.clone();
            self.next_token();
            Ok(token)
        } else {
            let error = ParseError::syntax_error(
                format!(
                    "Expected {:?}, got {:?}",
                    expected, self.current_token.kind
                ),
                self.current_token.line,
                self.current_token.column,
            );
            self.errors.add(error.clone());
            Err(error)
        }
    }

    pub fn is_at_end(&self) -> bool {
        matches!(self.current_token.kind, TokenKind::Eof)
    }

    pub fn parse_identifier(&mut self) -> Result<String, ParseError> {
        match &self.current_token.kind {
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.next_token();
                Ok(name)
            }
            _ => {
                let error = ParseError::syntax_error(
                    format!("Expected identifier, got {:?}", self.current_token.kind),
                    self.current_token.line,
                    self.current_token.column,
                );
                self.errors.add(error.clone());
                Err(error)
            }
        }
    }

    pub fn check_and_skip_keyword(&mut self, expected: TokenKind) -> bool {
        if self.current_token.kind == expected {
            self.next_token();
            true
        } else {
            false
        }
    }

    pub fn parse_tag_list(&mut self) -> Result<Vec<TagIdentifier>, ParseError> {
        let mut tags = Vec::new();

        // If we start with a parenthesis, we have tag list: (tag1, tag2, ...)
        if self.current_token.kind == TokenKind::LParen {
            self.next_token(); // Skip '('

            loop {
                let tag_name = self.parse_identifier()?;
                let properties = if self.current_token.kind == TokenKind::LBrace {
                    Some(self.parse_property_map()?)
                } else {
                    None
                };

                tags.push(TagIdentifier {
                    name: tag_name,
                    properties,
                });

                if self.current_token.kind != TokenKind::Comma {
                    break;
                }
                self.next_token(); // Skip comma
            }

            self.expect_token(TokenKind::RParen)?;
        } else {
            // Just a single tag
            let tag_name = self.parse_identifier()?;
            tags.push(TagIdentifier {
                name: tag_name,
                properties: None,
            });
        }

        Ok(tags)
    }

    pub fn parse_property_list(&mut self) -> Result<Vec<Property>, ParseError> {
        let mut properties = Vec::new();

        if self.current_token.kind == TokenKind::LBrace {
            self.next_token(); // Skip '{'

            if self.current_token.kind != TokenKind::RBrace {
                loop {
                    let prop_name = self.parse_identifier()?;
                    self.expect_token(TokenKind::Colon)?;
                    let value = self.parse_expression()?;

                    properties.push(Property {
                        name: prop_name,
                        value,
                    });

                    if self.current_token.kind != TokenKind::Comma {
                        break;
                    }
                    self.next_token(); // Skip comma
                }
            }

            self.expect_token(TokenKind::RBrace)?;
        } else {
            // Parse as assignment list: prop1 = value1, prop2 = value2, ...
            loop {
                let prop_name = self.parse_identifier()?;
                self.expect_token(TokenKind::Assign)?;
                let value = self.parse_expression()?;

                properties.push(Property {
                    name: prop_name,
                    value,
                });

                if self.current_token.kind != TokenKind::Comma {
                    break;
                }
                self.next_token(); // Skip comma
            }
        }

        Ok(properties)
    }

    pub fn parse_property_ref(&mut self) -> Result<PropertyRef, ParseError> {
        let first_ident = self.parse_identifier()?;

        if self.current_token.kind == TokenKind::Dot {
            self.next_token(); // Skip '.'
            let second_ident = self.parse_identifier()?;
            Ok(PropertyRef::Prop(first_ident, second_ident))
        } else {
            Ok(PropertyRef::InlineProp(first_ident))
        }
    }

    pub fn parse_yield_clause(&mut self) -> Result<YieldClause, ParseError> {
        self.next_token(); // Skip YIELD
        let mut items = Vec::new();

        loop {
            let expr = self.parse_expression()?;
            let alias = if self.current_token.kind == TokenKind::As {
                self.next_token();
                Some(self.parse_identifier()?)
            } else {
                None
            };

            items.push(YieldExpression { expr, alias });

            if self.current_token.kind != TokenKind::Comma {
                break;
            }
            self.next_token();
        }

        Ok(YieldClause { items })
    }

    pub fn parse_property_map(&mut self) -> Result<std::collections::HashMap<String, Expression>, ParseError> {
        let mut map = std::collections::HashMap::new();

        self.expect_token(TokenKind::LBrace)?;

        if self.current_token.kind != TokenKind::RBrace {
            loop {
                let key = self.parse_identifier()?;
                self.expect_token(TokenKind::Colon)?;
                let value = self.parse_expression()?;

                map.insert(key, value);

                if self.current_token.kind != TokenKind::Comma {
                    break;
                }
                self.next_token();
            }
        }

        self.expect_token(TokenKind::RBrace)?;
        Ok(map)
    }
}