//! Cypher解析器核心模块
//!
//! 提供解析器的基础结构和通用方法

use super::ast::*;
use super::lexer::{CypherLexer, Token, TokenType};
use std::collections::HashMap;

/// Cypher解析器核心结构
#[derive(Debug)]
pub struct CypherParserCore {
    pub tokens: Vec<Token>,
    pub current_token_index: usize,
}

impl CypherParserCore {
    /// 创建新的Cypher解析器
    pub fn new(input: String) -> Self {
        let mut lexer = CypherLexer::new(input);
        let tokens = lexer.tokenize().unwrap_or_else(|_| {
            vec![Token {
                token_type: TokenType::EOF,
                value: "".to_string(),
                position: 0,
            }]
        });

        Self {
            tokens,
            current_token_index: 0,
        }
    }

    /// 获取当前标记
    pub fn current_token(&self) -> &Token {
        &self.tokens[self.current_token_index]
    }

    /// 查看下一个标记
    pub fn peek_token(&self, offset: usize) -> Option<&Token> {
        let index = self.current_token_index + offset;
        if index < self.tokens.len() {
            Some(&self.tokens[index])
        } else {
            None
        }
    }

    /// 消费当前标记并移动到下一个
    pub fn consume_token(&mut self) -> &Token {
        let token = &self.tokens[self.current_token_index];
        if self.current_token_index < self.tokens.len() - 1 {
            self.current_token_index += 1;
        }
        token
    }

    /// 检查当前标记是否为指定类型
    pub fn is_current_token_type(&self, token_type: TokenType) -> bool {
        self.current_token().token_type == token_type
    }

    /// 检查当前标记是否为指定值
    pub fn is_current_token_value(&self, value: &str) -> bool {
        self.current_token().value == value
    }

    /// 检查当前标记是否为关键字
    pub fn is_current_keyword(&self, keyword: &str) -> bool {
        self.is_current_token_type(TokenType::Keyword)
            && self.current_token().value.to_uppercase() == keyword.to_uppercase()
    }

    /// 期望当前标记为指定类型，否则返回错误
    pub fn expect_token_type(&mut self, token_type: TokenType) -> Result<&Token, String> {
        if self.is_current_token_type(token_type.clone()) {
            Ok(self.consume_token())
        } else {
            Err(format!(
                "期望标记类型 {:?}，但得到 {:?} 在位置 {}",
                token_type,
                self.current_token().token_type,
                self.current_token().position
            ))
        }
    }

    /// 期望当前标记为指定值，否则返回错误
    pub fn expect_token_value(&mut self, value: &str) -> Result<&Token, String> {
        if self.is_current_token_value(value) {
            Ok(self.consume_token())
        } else {
            Err(format!(
                "期望标记 '{}'，但得到 '{}' 在位置 {}",
                value,
                self.current_token().value,
                self.current_token().position
            ))
        }
    }

    /// 期望当前标记为指定关键字，否则返回错误
    pub fn expect_keyword(&mut self, keyword: &str) -> Result<&Token, String> {
        if self.is_current_keyword(keyword) {
            Ok(self.consume_token())
        } else {
            Err(format!(
                "期望关键字 '{}'，但得到 '{}' 在位置 {}",
                keyword,
                self.current_token().value,
                self.current_token().position
            ))
        }
    }

    /// 跳过空白字符标记
    pub fn skip_whitespace(&mut self) {
        while self.is_current_token_type(TokenType::Whitespace) {
            self.consume_token();
        }
    }

    /// 检查是否到达文件末尾
    pub fn is_eof(&self) -> bool {
        self.is_current_token_type(TokenType::EOF)
    }

    /// 解析标识符
    pub fn parse_identifier(&mut self) -> Result<String, String> {
        self.skip_whitespace();

        if self.is_current_token_type(TokenType::Identifier)
            || self.is_current_token_type(TokenType::Keyword)
        {
            Ok(self.consume_token().value.clone())
        } else {
            Err(format!(
                "期望标识符，但得到 '{}' 在位置 {}",
                self.current_token().value,
                self.current_token().position
            ))
        }
    }

    /// 解析字符串字面量
    pub fn parse_string_literal(&mut self) -> Result<String, String> {
        self.skip_whitespace();

        if self.is_current_token_type(TokenType::LiteralString) {
            Ok(self.consume_token().value.clone())
        } else {
            Err(format!(
                "期望字符串字面量，但得到 '{}' 在位置 {}",
                self.current_token().value,
                self.current_token().position
            ))
        }
    }

    /// 解析数字字面量
    pub fn parse_number_literal(&mut self) -> Result<i64, String> {
        self.skip_whitespace();

        if self.is_current_token_type(TokenType::LiteralNumber) {
            let value = self.consume_token().value.clone();
            value.parse().map_err(|e| {
                format!(
                    "解析数字失败: '{}' 在位置 {} - {}",
                    value,
                    self.current_token().position,
                    e
                )
            })
        } else {
            Err(format!(
                "期望数字字面量，但得到 '{}' 在位置 {}",
                self.current_token().value,
                self.current_token().position
            ))
        }
    }

    /// 解析属性映射
    pub fn parse_properties(&mut self) -> Result<Option<HashMap<String, Expression>>, String> {
        self.skip_whitespace();

        if self.is_current_token_value("{") {
            self.consume_token(); // 消费 '{'
            let mut properties = HashMap::new();

            self.skip_whitespace();
            while !self.is_current_token_value("}") && !self.is_eof() {
                let key = self.parse_identifier()?;

                self.skip_whitespace();
                self.expect_token_value(":")?;

                let value = self.parse_expression()?;
                properties.insert(key, value);

                self.skip_whitespace();
                if self.is_current_token_value(",") {
                    self.consume_token(); // 消费 ','
                    self.skip_whitespace();
                } else {
                    break;
                }
            }

            self.expect_token_value("}")?;
            Ok(Some(properties))
        } else {
            Ok(None)
        }
    }

    /// 解析表达式（基础实现，具体实现在expression_parser.rs中）
    pub fn parse_expression(&mut self) -> Result<Expression, String> {
        self.skip_whitespace();

        if self.is_current_token_type(TokenType::LiteralString) {
            let value = self.parse_string_literal()?;
            Ok(Expression::Literal(Literal::String(value)))
        } else if self.is_current_token_type(TokenType::LiteralNumber) {
            let value = self.parse_number_literal()?;
            Ok(Expression::Literal(Literal::Integer(value)))
        } else if self.is_current_token_type(TokenType::Identifier) {
            let identifier = self.parse_identifier()?;

            // 检查是否是属性表达式
            self.skip_whitespace();
            if self.is_current_token_value(".") {
                self.consume_token(); // 消费 '.'
                let property_name = self.parse_identifier()?;
                Ok(Expression::Property(PropertyExpression {
                    expression: Box::new(Expression::Variable(identifier)),
                    property_name,
                }))
            } else {
                Ok(Expression::Variable(identifier))
            }
        } else {
            Err(format!(
                "不支持的表达式类型: '{}' 在位置 {}",
                self.current_token().value,
                self.current_token().position
            ))
        }
    }

    /// 解析标签列表
    pub fn parse_labels(&mut self) -> Result<Vec<String>, String> {
        let mut labels = Vec::new();

        self.skip_whitespace();
        while self.is_current_token_value(":") {
            self.consume_token(); // 消费 ':'
            let label = self.parse_identifier()?;
            labels.push(label);
            self.skip_whitespace();
        }

        Ok(labels)
    }

    /// 解析类型列表
    pub fn parse_types(&mut self) -> Result<Vec<String>, String> {
        let mut types = Vec::new();

        self.skip_whitespace();
        while self.is_current_token_value(":") {
            self.consume_token(); // 消费 ':'
            let type_name = self.parse_identifier()?;
            types.push(type_name);
            self.skip_whitespace();
        }

        Ok(types)
    }

    /// 解析范围
    pub fn parse_range(&mut self) -> Result<Option<Range>, String> {
        self.skip_whitespace();

        if self.is_current_token_value("*") {
            self.consume_token(); // 消费 '*'

            let start = if self.is_current_token_type(TokenType::LiteralNumber) {
                Some(self.parse_number_literal()?)
            } else {
                None
            };

            self.skip_whitespace();
            if self.is_current_token_value(".") {
                self.consume_token(); // 消费 '.'
                self.expect_token_value(".")?; // 消费第二个 '.'

                self.skip_whitespace();
                let end = if self.is_current_token_type(TokenType::LiteralNumber) {
                    Some(self.parse_number_literal()?)
                } else {
                    None
                };

                Ok(Some(Range { start, end }))
            } else {
                Ok(Some(Range { start, end: start }))
            }
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_core_creation() {
        let parser = CypherParserCore::new("MATCH (n)".to_string());
        assert!(!parser.is_eof());
        assert_eq!(parser.current_token().value, "MATCH");
    }

    #[test]
    fn test_identifier_parsing() {
        let mut parser = CypherParserCore::new("myVariable".to_string());
        let identifier = parser
            .parse_identifier()
            .expect("Failed to parse identifier");
        assert_eq!(identifier, "myVariable");
    }

    #[test]
    fn test_string_literal_parsing() {
        let mut parser = CypherParserCore::new("\"Hello, World!\"".to_string());
        let string_literal = parser
            .parse_string_literal()
            .expect("Parser should parse valid string literals");
        assert_eq!(string_literal, "Hello, World!");
    }

    #[test]
    fn test_number_literal_parsing() {
        let mut parser = CypherParserCore::new("42".to_string());
        let number_literal = parser
            .parse_number_literal()
            .expect("Parser should parse valid number literals");
        assert_eq!(number_literal, 42);
    }

    #[test]
    fn test_expect_keyword() {
        let mut parser = CypherParserCore::new("MATCH".to_string());
        let result = parser.expect_keyword("MATCH");
        assert!(result.is_ok());
        assert_eq!(
            result
                .expect("Parser should return valid keyword result")
                .value,
            "MATCH"
        );
    }

    #[test]
    fn test_expect_keyword_error() {
        let mut parser = CypherParserCore::new("RETURN".to_string());
        let result = parser.expect_keyword("MATCH");
        assert!(result.is_err());
    }
}
