//! 解析器工具函数模块
//!
//! 提供解析器使用的通用工具函数和辅助方法。

use crate::query::parser::lexer::TokenKind as LexerToken;
use crate::query::parser::ast::types::{ParseError, Span, Position};

impl super::Parser {
    /// 检查并匹配 token
    pub fn match_token(&mut self, expected: LexerToken) -> bool {
        if self.current_token.kind == expected {
            self.next_token();
            true
        } else {
            false
        }
    }

    /// 检查 token 类型
    pub fn check_token(&mut self, expected: LexerToken) -> bool {
        self.current_token.kind == expected
    }

    /// 期望特定的 token
    pub fn expect_token(&mut self, expected: LexerToken) -> Result<(), ParseError> {
        if self.current_token.kind == expected {
            self.next_token();
            Ok(())
        } else {
            Err(ParseError::new(
                format!(
                    "Expected {:?}, found {:?}",
                    expected, self.current_token.kind
                ),
                self.current_span(),
            ))
        }
    }

    /// 期望标识符
    pub fn expect_identifier(&mut self) -> Result<String, ParseError> {
        if let LexerToken::Identifier(_) = self.current_token.kind {
            let text = match &self.current_token.kind {
                LexerToken::Identifier(s) => s.clone(),
                _ => String::new(),
            };
            self.next_token();
            Ok(text)
        } else {
            Err(ParseError::new(
                format!("Expected identifier, found {:?}", self.current_token.kind),
                self.current_span(),
            ))
        }
    }

    /// 获取当前 span
    pub fn current_span(&self) -> Span {
        let pos = self.lexer.current_position();
        Span::new(
            Position::new(pos.line, pos.column),
            Position::new(pos.line, pos.column),
        )
    }

    /// 获取当前 token
    pub fn current_token(&self) -> &crate::query::parser::core::token::Token {
        &self.current_token
    }

    /// 获取下一个 token
    pub fn next_token(&mut self) {
        let token = self.lexer.next_token();
        self.current_token = token;
    }

    /// 查看下一个 token 但不移动位置
    pub fn peek_token(&self) -> crate::query::parser::core::token::TokenKind {
        self.current_token.kind.clone()
    }

    /// 查看下一个 token 但不移动位置（返回整个 Token）
    pub fn peek_next_token(&self) -> crate::query::parser::core::token::Token {
        crate::query::parser::core::token::Token::new(
            crate::query::parser::core::token::TokenKind::Eof,
            String::new(),
            0,
            0,
        )
    }

    /// 解析标识符
    pub fn parse_identifier(&mut self) -> Result<String, ParseError> {
        match &self.current_token.kind {
            crate::query::parser::core::token::TokenKind::Identifier(s) => {
                let id = s.clone();
                self.next_token();
                Ok(id)
            }
            _ => Err(ParseError::new(
                format!("Expected identifier, found {:?}", self.current_token.kind),
                self.current_span(),
            )),
        }
    }
}