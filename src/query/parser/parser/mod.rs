//! 解析器模块
//!
//! 基于新 AST 设计的完整解析器，提供完整的查询语言解析功能。

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
use crate::query::parser::{TokenKind, ParseError};
use crate::query::parser::ast::stmt::{FromClause, OverClause};

/// 解析器
pub struct Parser {
    lexer: Lexer,
    compat_mode: bool,
    current_token: Token,
    recursion_depth: usize,
    max_recursion_depth: usize,
}

impl Parser {
    /// 创建解析器
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.peek().unwrap_or_else(|_| {
            Token::new(crate::query::parser::TokenKind::Eof, String::new(), 0, 0)
        });

        Self {
            lexer,
            compat_mode: false,
            current_token,
            recursion_depth: 0,
            max_recursion_depth: 100,
        }
    }

    /// 设置兼容模式
    pub fn set_compat_mode(&mut self, enabled: bool) {
        self.compat_mode = enabled;
    }

    /// 进入递归
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

    /// 退出递归
    pub fn exit_recursion(&mut self) {
        if self.recursion_depth > 0 {
            self.recursion_depth -= 1;
        }
    }

    /// 获取当前 span
    pub fn parser_current_span(&self) -> crate::query::parser::ast::types::Span {
        let pos = self.lexer.current_position();
        crate::query::parser::ast::types::Span::new(
            crate::query::parser::ast::types::Position::new(pos.line, pos.column),
            crate::query::parser::ast::types::Position::new(pos.line, pos.column),
        )
    }

    /// 获取当前 span（别名）
    pub fn current_span(&self) -> crate::query::parser::ast::types::Span {
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
}
