//! Expression parsing trait for the query parser
//!
//! This module defines the interface for expression parsing that can be
//! implemented by different parser components.

use crate::query::parser::ast::*;
use crate::query::parser::{ParseError, Token, TokenKind};

pub trait TokenParser {
    fn expect_token(&mut self, expected: TokenKind) -> Result<Token, ParseError>;
    fn peek_token(&self) -> TokenKind;
    fn current_token(&self) -> &Token;
    fn next_token(&mut self);
}

pub trait ExpressionParser: TokenParser {
    /// 解析表达式
    fn parse_expression(&mut self) -> Result<Expr, ParseError>;

    /// 解析逻辑或表达式
    fn parse_logical_or(&mut self) -> Result<Expr, ParseError>;

    /// 解析逻辑与表达式
    fn parse_logical_and(&mut self) -> Result<Expr, ParseError>;

    /// 解析相等性表达式
    fn parse_equality(&mut self) -> Result<Expr, ParseError>;

    /// 解析比较表达式
    fn parse_comparison(&mut self) -> Result<Expr, ParseError>;

    /// 解析加法表达式
    fn parse_addition(&mut self) -> Result<Expr, ParseError>;

    /// 解析乘法表达式
    fn parse_multiplication(&mut self) -> Result<Expr, ParseError>;

    /// 解析一元表达式
    fn parse_unary(&mut self) -> Result<Expr, ParseError>;

    /// 解析指数表达式
    fn parse_exponentiation(&mut self) -> Result<Expr, ParseError>;

    /// 解析基本表达式
    fn parse_primary(&mut self) -> Result<Expr, ParseError>;

    fn parse_identifier(&mut self) -> Result<String, ParseError>;
}

// 包含表达式转换器
pub mod expression_converter;

// 重新导出表达式转换器的公共函数
pub use expression_converter::{convert_ast_to_graph_expression, parse_expression_from_string};
