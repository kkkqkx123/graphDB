//! Expression parsing trait for the query parser
//!
//! This module defines the interface for expression parsing that can be
//! implemented by different parser components.

use crate::query::parser::core::error::{ParseError, ParseErrors};
use crate::query::parser::core::token::{Token, TokenKind};
use crate::query::parser::ast::*;

pub trait TokenParser {
    fn expect_token(&mut self, expected: TokenKind) -> Result<Token, ParseError>;
    fn peek_token(&self) -> TokenKind;
    fn current_token(&self) -> &Token;
    fn next_token(&mut self);
}

pub trait ExpressionParser: TokenParser {
    /// 解析表达式
    fn parse_expression(&mut self) -> Result<Expression, ParseError>;

    /// 解析逻辑或表达式
    fn parse_logical_or(&mut self) -> Result<Expression, ParseError>;

    /// 解析逻辑与表达式
    fn parse_logical_and(&mut self) -> Result<Expression, ParseError>;

    /// 解析相等性表达式
    fn parse_equality(&mut self) -> Result<Expression, ParseError>;

    /// 解析比较表达式
    fn parse_comparison(&mut self) -> Result<Expression, ParseError>;

    /// 解析加法表达式
    fn parse_addition(&mut self) -> Result<Expression, ParseError>;

    /// 解析乘法表达式
    fn parse_multiplication(&mut self) -> Result<Expression, ParseError>;

    /// 解析一元表达式
    fn parse_unary(&mut self) -> Result<Expression, ParseError>;

    /// 解析指数表达式
    fn parse_exponentiation(&mut self) -> Result<Expression, ParseError>;

    /// 解析基本表达式
    fn parse_primary(&mut self) -> Result<Expression, ParseError>;

    fn parse_identifier(&mut self) -> Result<String, ParseError>;
}