//! 解析器工具函数模块
//!
//! 提供解析器使用的通用工具函数和辅助方法。

use crate::query::parser::ast::expr::*;
use crate::query::parser::ast::stmt::{
    PropertyDef,
};
use crate::query::parser::ast::types::{DataType, ParseError};
use crate::query::parser::core::error::ParseErrorKind;
use crate::query::parser::lexer::TokenKind as LexerToken;
use crate::query::parser::{Token, TokenKind};

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
            let span = self.parser_current_span();
            Err(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                format!("Expected identifier, found {:?}", self.current_token.kind),
                span.start.line,
                span.start.column,
            ))
        }
    }

    /// 获取当前 token
    pub fn current_token(&self) -> &Token {
        &self.current_token
    }

    /// 获取下一个 token
    pub fn next_token(&mut self) {
        let token = self.lexer.next_token();
        self.current_token = token;
    }

    /// 查看下一个 token 但不移动位置
    pub fn peek_token(&self) -> TokenKind {
        self.current_token.kind.clone()
    }

    /// 查看下一个 token 但不移动位置（返回整个 Token）
    pub fn peek_next_token(&self) -> Token {
        Token::new(TokenKind::Eof, String::new(), 0, 0)
    }

    /// 解析标识符
    pub fn parse_identifier(&mut self) -> Result<String, ParseError> {
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

    /// 检查并跳过关键字
    pub fn check_and_skip_keyword(&mut self, expected: LexerToken) -> bool {
        if self.current_token.kind == expected {
            self.next_token();
            true
        } else {
            false
        }
    }

    /// 解析标签列表
    pub fn parse_tag_list(&mut self) -> Result<Vec<String>, ParseError> {
        let mut tags = Vec::new();

        // If we start with a parenthesis, we have tag list: (tag1, tag2, ...)
        if self.current_token.kind == LexerToken::LParen {
            self.next_token(); // Skip '('

            loop {
                let tag_name = self.parse_identifier()?;
                tags.push(tag_name);

                if self.current_token.kind != LexerToken::Comma {
                    break;
                }
                self.next_token(); // Skip comma
            }

            self.expect_token(LexerToken::RParen)?;
        } else {
            // Just a single tag
            let tag_name = self.parse_identifier()?;
            tags.push(tag_name);
        }

        Ok(tags)
    }

    /// 解析属性列表
    pub fn parse_property_list(&mut self) -> Result<Vec<PropertyDef>, ParseError> {
        let mut properties = Vec::new();

        if self.current_token.kind == LexerToken::LBrace {
            self.next_token(); // Skip '{'

            if self.current_token.kind != LexerToken::RBrace {
                loop {
                    let prop_name = self.parse_identifier()?;
                    self.expect_token(LexerToken::Colon)?;
                    let _value = self.parse_expression()?;

                    properties.push(PropertyDef {
                        name: prop_name,
                        data_type: DataType::String,
                        nullable: false,
                        default: None,
                    });

                    if self.current_token.kind != LexerToken::Comma {
                        break;
                    }
                    self.next_token(); // Skip comma
                }
            }

            self.expect_token(LexerToken::RBrace)?;
        } else {
            // Parse as assignment list: prop1 = value1, prop2 = value2, ...
            loop {
                let prop_name = self.parse_identifier()?;
                self.expect_token(LexerToken::Assign)?;
                let _value = self.parse_expression()?;

                properties.push(PropertyDef {
                    name: prop_name,
                    data_type: DataType::String,
                    nullable: false,
                    default: None,
                });

                if self.current_token.kind != LexerToken::Comma {
                    break;
                }
                self.next_token(); // Skip comma
            }
        }

        Ok(properties)
    }

    /// 解析属性映射
    pub fn parse_property_map(
        &mut self,
    ) -> Result<std::collections::HashMap<String, Expr>, ParseError> {
        let mut map = std::collections::HashMap::new();

        self.expect_token(LexerToken::LBrace)?;

        if self.current_token.kind != LexerToken::RBrace {
            loop {
                let key = self.parse_identifier()?;
                self.expect_token(LexerToken::Colon)?;
                let value = self.parse_expression()?;

                map.insert(key, value);

                if self.current_token.kind != LexerToken::Comma {
                    break;
                }
                self.next_token();
            }
        }

        self.expect_token(LexerToken::RBrace)?;
        Ok(map)
    }
}
