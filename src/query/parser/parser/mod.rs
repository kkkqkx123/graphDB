//! 解析器模块
//!
//! 基于新 AST 设计的完整解析器，提供完整的查询语言解析功能。

mod expr_parser;
mod pattern_parser;
mod utils;
mod main_parser;

use crate::query::parser::lexer::Lexer;
use crate::query::parser::Token;

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
}
