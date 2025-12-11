//! Expression parsing module for the query parser
//!
//! This module implements parsing for expressions in the query language.

use crate::query::parser::lexer::lexer::Lexer;
use crate::query::parser::core::token::{Token, TokenKind};
use crate::query::parser::ast::*;
use crate::query::parser::core::error::{ParseError, ParseErrors};
use crate::core::Value;

impl super::Parser {
    /// 解析表达式
    pub fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        self.parse_logical_or()
    }

    /// 解析逻辑或表达式
    fn parse_logical_or(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_logical_and()?;

        while self.current_token.kind == TokenKind::Or {
            self.next_token();
            let right = self.parse_logical_and()?;
            expr = Expression::Logical(Box::new(expr), LogicalOp::Or, Box::new(right));
        }

        Ok(expr)
    }

    /// 解析逻辑与表达式
    fn parse_logical_and(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_equality()?;

        while self.current_token.kind == TokenKind::And {
            self.next_token();
            let right = self.parse_equality()?;
            expr = Expression::Logical(Box::new(expr), LogicalOp::And, Box::new(right));
        }

        Ok(expr)
    }

    /// 解析相等性表达式
    fn parse_equality(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_comparison()?;

        loop {
            let op = match self.current_token.kind {
                TokenKind::Eq => { self.next_token(); RelationalOp::Eq },
                TokenKind::Ne => { self.next_token(); RelationalOp::Ne },
                _ => break,
            };

            let right = self.parse_comparison()?;
            expr = Expression::Relational(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    /// 解析比较表达式
    fn parse_comparison(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_addition()?;

        loop {
            let op = match self.current_token.kind {
                TokenKind::Lt => { self.next_token(); RelationalOp::Lt },
                TokenKind::Le => { self.next_token(); RelationalOp::Le },
                TokenKind::Gt => { self.next_token(); RelationalOp::Gt },
                TokenKind::Ge => { self.next_token(); RelationalOp::Ge },
                TokenKind::Regex => { self.next_token(); RelationalOp::Regex },
                _ => break,
            };

            let right = self.parse_addition()?;
            expr = Expression::Relational(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    /// 解析加法表达式
    fn parse_addition(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_multiplication()?;

        loop {
            let op = match self.current_token.kind {
                TokenKind::Plus => { self.next_token(); ArithmeticOp::Add },
                TokenKind::Minus => { self.next_token(); ArithmeticOp::Sub },
                _ => break,
            };

            let right = self.parse_multiplication()?;
            expr = Expression::Arithmetic(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    /// 解析乘法表达式
    fn parse_multiplication(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_unary()?;

        loop {
            let op = match self.current_token.kind {
                TokenKind::Star => { self.next_token(); ArithmeticOp::Mul },
                TokenKind::Div => { self.next_token(); ArithmeticOp::Div },
                TokenKind::Mod => { self.next_token(); ArithmeticOp::Mod },
                _ => break,
            };

            let right = self.parse_unary()?;
            expr = Expression::Arithmetic(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    /// 解析一元表达式
    fn parse_unary(&mut self) -> Result<Expression, ParseError> {
        match self.current_token.kind {
            TokenKind::NotOp => {
                self.next_token();
                let expr = self.parse_exponentiation()?;
                Ok(Expression::Unary(UnaryOp::Not, Box::new(expr)))
            }
            TokenKind::Plus => {
                self.next_token();
                let expr = self.parse_exponentiation()?;
                Ok(Expression::Unary(UnaryOp::Plus, Box::new(expr)))
            }
            TokenKind::Minus => {
                self.next_token();
                let expr = self.parse_exponentiation()?;
                Ok(Expression::Unary(UnaryOp::Minus, Box::new(expr)))
            }
            _ => self.parse_exponentiation(),
        }
    }

    /// 解析指数表达式
    fn parse_exponentiation(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_primary()?;

        // For now, we don't have exponentiation in our grammar
        // This function can be extended later if needed

        Ok(expr)
    }

    /// 解析基本表达式
    fn parse_primary(&mut self) -> Result<Expression, ParseError> {
        let expr = match &self.current_token.kind {
            TokenKind::IntegerLiteral(n) => {
                let value = crate::core::Value::Int(*n);
                self.next_token();
                Expression::Constant(value)
            }
            TokenKind::FloatLiteral(n) => {
                let value = crate::core::Value::Float(*n);
                self.next_token();
                Expression::Constant(value)
            }
            TokenKind::StringLiteral(s) => {
                let value = crate::core::Value::String(s.clone());
                self.next_token();
                Expression::Constant(value)
            }
            TokenKind::BooleanLiteral(b) => {
                let value = crate::core::Value::Bool(*b);
                self.next_token();
                Expression::Constant(value)
            }
            TokenKind::Null => {
                let value = crate::core::Value::Null(crate::core::NullType::Null);
                self.next_token();
                Expression::Constant(value)
            }
            TokenKind::LParen => {
                self.next_token(); // Skip '('
                let expr = self.parse_expression()?;
                self.expect_token(TokenKind::RParen)?;
                expr
            }
            TokenKind::LBracket => {
                self.next_token(); // Skip '['
                let mut elements = Vec::new();

                if self.current_token.kind != TokenKind::RBracket {
                    loop {
                        elements.push(self.parse_expression()?);
                        if self.current_token.kind != TokenKind::Comma {
                            break;
                        }
                        self.next_token(); // Skip comma
                    }
                }

                self.expect_token(TokenKind::RBracket)?;
                Expression::List(elements)
            }
            TokenKind::LBrace => {
                self.next_token(); // Skip '{'
                let mut pairs = Vec::new();

                if self.current_token.kind != TokenKind::RBrace {
                    loop {
                        let key = self.parse_identifier()?;
                        self.expect_token(TokenKind::Colon)?;
                        let value = self.parse_expression()?;
                        pairs.push((key, value));

                        if self.current_token.kind != TokenKind::Comma {
                            break;
                        }
                        self.next_token(); // Skip comma
                    }
                }

                self.expect_token(TokenKind::RBrace)?;
                Expression::Map(pairs)
            }
            TokenKind::Identifier(name) => {
                // Check if it's a function call
                if self.peek_token() == TokenKind::LParen {
                    let func_name = name.clone();
                    self.next_token(); // Skip identifier
                    self.expect_token(TokenKind::LParen)?;

                    let mut args = Vec::new();
                    if self.current_token.kind != TokenKind::RParen {
                        loop {
                            args.push(self.parse_expression()?);
                            if self.current_token.kind != TokenKind::Comma {
                                break;
                            }
                            self.next_token(); // Skip comma
                        }
                    }
                    self.expect_token(TokenKind::RParen)?;

                    Expression::FunctionCall(FunctionCall {
                        name: func_name,
                        args,
                        distinct: false, // For now, no DISTINCT
                    })
                } else {
                    // It's a variable or property access
                    let var_name = name.clone();
                    self.next_token(); // Skip identifier

                    // Check if it's followed by a dot for property access
                    if self.current_token.kind == TokenKind::Dot {
                        self.next_token();
                        let prop_name = self.parse_identifier()?;
                        Expression::PropertyAccess(
                            Box::new(Expression::Variable(var_name)),
                            prop_name,
                        )
                    } else {
                        Expression::Variable(var_name)
                    }
                }
            }
            _ => {
                return Err(ParseError::syntax_error(
                    format!("Unexpected token in expression: {:?}", self.current_token.kind),
                    self.current_token.line,
                    self.current_token.column,
                ));
            }
        };

        Ok(expr)
    }
}