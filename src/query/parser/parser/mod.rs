//! 解析器模块

mod expr_parser;
mod pattern_parser;
mod utils;
mod stmt_parser;
mod clause_parser;

pub use expr_parser::ExprParser;
pub use stmt_parser::StmtParser;

use crate::query::parser::lexer::Lexer;
use crate::query::parser::Token;
use crate::query::parser::core::error::ParseErrorKind;
use crate::query::parser::{TokenKind, ParseError, Span, Position};
use crate::query::parser::ast::stmt::{FromClause, OverClause};
use crate::query::parser::ast::expr::Expr;

pub struct Parser {
    lexer: Lexer,
    expr_parser: ExprParser,
    compat_mode: bool,
    current_token: Token,
    recursion_depth: usize,
    max_recursion_depth: usize,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.peek().unwrap_or_else(|_| {
            Token::new(crate::query::parser::TokenKind::Eof, String::new(), 0, 0)
        });

        Self {
            lexer,
            expr_parser: ExprParser::new(input),
            compat_mode: false,
            current_token,
            recursion_depth: 0,
            max_recursion_depth: 100,
        }
    }

    pub fn set_compat_mode(&mut self, enabled: bool) {
        self.compat_mode = enabled;
    }

    pub fn enter_recursion(&mut self) -> Result<(), crate::query::parser::core::error::ParseError> {
        self.recursion_depth += 1;
        if self.recursion_depth > self.max_recursion_depth {
            let pos = self.lexer.current_position();
            Err(crate::query::parser::core::error::ParseError::new(
                ParseErrorKind::SyntaxError,
                "Recursion limit exceeded".to_string(),
                pos.line,
                pos.column,
            ))
        } else {
            Ok(())
        }
    }

    pub fn exit_recursion(&mut self) {
        if self.recursion_depth > 0 {
            self.recursion_depth -= 1;
        }
    }

    pub fn parser_current_span(&self) -> Span {
        let pos = self.lexer.current_position();
        Span::new(
            Position::new(pos.line, pos.column),
            Position::new(pos.line, pos.column),
        )
    }

    pub fn current_span(&self) -> Span {
        self.parser_current_span()
    }

    pub fn parse_from_clause(&mut self) -> Result<FromClause, ParseError> {
        let span = self.current_span();
        self.expect_token(TokenKind::From)?;
        let vertices = self.parse_expression_list()?;
        Ok(FromClause { span, vertices })
    }

    pub fn parse_over_clause(&mut self) -> Result<OverClause, ParseError> {
        let span = self.current_span();
        self.expect_token(TokenKind::Over)?;
        let mut edge_types = Vec::new();
        let mut direction = crate::core::types::graph::EdgeDirection::Outgoing;
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
            direction = crate::core::types::graph::EdgeDirection::Outgoing;
        } else if self.current_token().kind == TokenKind::In {
            self.next_token();
            direction = crate::core::types::graph::EdgeDirection::Incoming;
        } else if self.current_token().kind == TokenKind::Both {
            self.next_token();
            direction = crate::core::types::graph::EdgeDirection::Both;
        }
        Ok(OverClause { span, edge_types, direction })
    }

    pub fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.expr_parser.parse_expression()
    }

    fn parse_expression_list(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut expressions = Vec::new();
        loop {
            expressions.push(self.parse_expression()?);
            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }
        Ok(expressions)
    }

    fn match_token(&mut self, expected: TokenKind) -> bool {
        if self.current_token.kind == expected {
            self.next_token();
            true
        } else {
            false
        }
    }

    fn expect_token(&mut self, expected: TokenKind) -> Result<(), ParseError> {
        if self.current_token.kind == expected {
            self.next_token();
            Ok(())
        } else {
            let span = self.parser_current_span();
            Err(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                format!(
                    "Expected {:?}, found {:?}",
                    expected, self.current_token.kind
                ),
                span.start.line,
                span.start.column,
            ))
        }
    }

    fn parse_identifier(&mut self) -> Result<String, ParseError> {
        match &self.current_token.kind {
            TokenKind::Identifier(s) => {
                let id = s.clone();
                self.next_token();
                Ok(id)
            }
            _ => Err(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                format!("Expected identifier, found {:?}", self.current_token.kind),
                self.current_token.line,
                self.current_token.column,
            )),
        }
    }

    fn current_token(&self) -> &Token {
        &self.current_token
    }

    fn next_token(&mut self) {
        let token = self.lexer.next_token();
        self.current_token = token;
    }
}
