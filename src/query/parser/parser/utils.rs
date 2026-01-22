//! 解析器工具函数模块
//!
//! 提供解析器使用的通用工具函数和辅助方法。

use crate::query::parser::ast::expr::*;
use crate::query::parser::ast::stmt::{
    PropertyDef,
};
use crate::query::parser::ast::types::DataType;
use crate::query::parser::core::ParseError;
use crate::query::parser::core::error::ParseErrorKind;
use crate::query::parser::lexer::TokenKind as LexerToken;

impl super::Parser {
    /// 检查 token 类型
    pub fn check_token(&mut self, expected: LexerToken) -> bool {
        self.current_token.kind == expected
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

        if self.current_token.kind == LexerToken::LParen {
            self.next_token();

            loop {
                let tag_name = self.parse_identifier()?;
                tags.push(tag_name);

                if self.current_token.kind != LexerToken::Comma {
                    break;
                }
                self.next_token();
            }

            self.expect_token(LexerToken::RParen)?;
        } else {
            let tag_name = self.parse_identifier()?;
            tags.push(tag_name);
        }

        Ok(tags)
    }

    /// 解析属性列表
    pub fn parse_property_list(&mut self) -> Result<Vec<PropertyDef>, ParseError> {
        let mut properties = Vec::new();

        if self.current_token.kind == LexerToken::LBrace {
            self.next_token();

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
                    self.next_token();
                }
            }

            self.expect_token(LexerToken::RBrace)?;
        } else {
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
                self.next_token();
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
