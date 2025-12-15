//! 解析器模块
//!
//! 基于新 AST 设计的简化解析器，提供更好的性能和可维护性。

mod expr_parser;
mod pattern_parser;
mod statement_parser;
mod utils;

pub use expr_parser::*;
pub use pattern_parser::*;
pub use statement_parser::*;
pub use utils::*;

use crate::query::parser::lexer::{Lexer, TokenKind as LexerToken};

/// 解析器
pub struct Parser {
    lexer: Lexer,
    compat_mode: bool,
    current_token: crate::query::parser::core::token::Token,
}

impl Parser {
    /// 创建解析器
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.peek().unwrap_or_else(|_| {
            crate::query::parser::core::token::Token::new(
                crate::query::parser::core::token::TokenKind::Eof,
                String::new(),
                0,
                0,
            )
        });

        Self {
            lexer,
            compat_mode: false,
            current_token,
        }
    }

    /// 设置兼容模式
    pub fn set_compat_mode(&mut self, enabled: bool) {
        self.compat_mode = enabled;
    }
}
