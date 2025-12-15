//! Expression parsing module for the query parser
//!
//! This module implements parsing for expressions in the query language.

use crate::query::parser::TokenKind;
use crate::query::parser::ast::*;
use crate::query::parser::ParseError;

impl super::Parser {
    /// 解析表达式
    pub fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.enter_recursion()?;
        let result = self.parse_logical_or();
        self.exit_recursion();
        result
    }

    /// 解析逻辑或表达式
    fn parse_logical_or(&mut self) -> Result<Expr, ParseError> {
        self.enter_recursion()?;
        let mut expr = self.parse_logical_and()?;
        while self.current_token.kind == TokenKind::Or {
            self.next_token();
            let right = self.parse_logical_and()?;
            expr = Box::new(node::BinaryExpr::new(
                expr,
                node::BinaryOp::Or,
                right,
                Span::single(Position::new(self.current_token.line as u32, self.current_token.column as u32, 0))
            ));
        }
        self.exit_recursion();
        Ok(expr)
    }

    /// 解析逻辑与表达式
    fn parse_logical_and(&mut self) -> Result<Expr, ParseError> {
        self.enter_recursion()?;
        let mut expr = self.parse_equality()?;
        while self.current_token.kind == TokenKind::And {
            self.next_token();
            let right = self.parse_equality()?;
            expr = Box::new(node::BinaryExpr::new(
                expr,
                node::BinaryOp::And,
                right,
                Span::single(Position::new(self.current_token.line as u32, self.current_token.column as u32, 0))
            ));
        }
        self.exit_recursion();
        Ok(expr)
    }

    /// 解析相等性表达式
    fn parse_equality(&mut self) -> Result<Expr, ParseError> {
        self.enter_recursion()?;
        let mut expr = self.parse_comparison()?;
        loop {
            let op = match self.current_token.kind {
                TokenKind::Eq => { self.next_token(); node::BinaryOp::Eq },
                TokenKind::Ne => { self.next_token(); node::BinaryOp::Ne },
                _ => break,
            };

            let right = self.parse_comparison()?;
            expr = Box::new(node::BinaryExpr::new(
                expr,
                op,
                right,
                Span::single(Position::new(self.current_token.line as u32, self.current_token.column as u32, 0))
            ));
        }
        self.exit_recursion();
        Ok(expr)
    }

    /// 解析比较表达式
    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        self.enter_recursion()?;
        let mut expr = self.parse_addition()?;

        loop {
            let op = match self.current_token.kind {
                TokenKind::Lt => { self.next_token(); node::BinaryOp::Lt },
                TokenKind::Le => { self.next_token(); node::BinaryOp::Le },
                TokenKind::Gt => { self.next_token(); node::BinaryOp::Gt },
                TokenKind::Ge => { self.next_token(); node::BinaryOp::Ge },
                TokenKind::Regex => { self.next_token(); node::BinaryOp::Regex },
                _ => break,
            };

            let right = self.parse_addition()?;
            expr = Box::new(node::BinaryExpr::new(
                expr,
                op,
                right,
                Span::single(Position::new(self.current_token.line as u32, self.current_token.column as u32, 0))
            ));
        }

        self.exit_recursion();
        Ok(expr)
    }

    /// 解析加法表达式
    fn parse_addition(&mut self) -> Result<Expr, ParseError> {
        self.enter_recursion()?;
        let mut expr = self.parse_multiplication()?;

        loop {
            let op = match self.current_token.kind {
                TokenKind::Plus => { self.next_token(); node::BinaryOp::Add },
                TokenKind::Minus => { self.next_token(); node::BinaryOp::Sub },
                _ => break,
            };

            let right = self.parse_multiplication()?;
            expr = Box::new(node::BinaryExpr::new(
                expr,
                op,
                right,
                Span::single(Position::new(self.current_token.line as u32, self.current_token.column as u32, 0))
            ));
        }

        self.exit_recursion();
        Ok(expr)
    }

    /// 解析乘法表达式
    fn parse_multiplication(&mut self) -> Result<Expr, ParseError> {
        self.enter_recursion()?;
        let mut expr = self.parse_unary()?;

        loop {
            let op = match self.current_token.kind {
                TokenKind::Star => { self.next_token(); node::BinaryOp::Mul },
                TokenKind::Div => { self.next_token(); node::BinaryOp::Div },
                TokenKind::Mod => { self.next_token(); node::BinaryOp::Mod },
                _ => break,
            };

            let right = self.parse_unary()?;
            expr = Box::new(node::BinaryExpr::new(
                expr,
                op,
                right,
                Span::single(Position::new(self.current_token.line as u32, self.current_token.column as u32, 0))
            ));
        }

        self.exit_recursion();
        Ok(expr)
    }

    /// 解析一元表达式
    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        self.enter_recursion()?;
        let result = match self.current_token.kind {
            TokenKind::NotOp => {
                self.next_token();
                let expr = self.parse_unary()?;
                Box::new(node::UnaryExpr::new(node::UnaryOp::Not, expr, Span::single(Position::new(self.current_token.line as u32, self.current_token.column as u32, 0))))
            }
            TokenKind::Plus => {
                self.next_token();
                // 对于一元加号，需要获取它作用的表达式，这应该是一元表达式（处理多重符号，如 ++a, +-b等）
                let expr = self.parse_unary()?;
                Box::new(node::UnaryExpr::new(node::UnaryOp::Plus, expr, Span::single(Position::new(self.current_token.line as u32, self.current_token.column as u32, 0))))
            }
            TokenKind::Minus => {
                self.next_token();
                // 对于一元减号，需要获取它作用的表达式，这应该是一元表达式（处理多重符号，如 --a, -+b等）
                let expr = self.parse_unary()?;
                Box::new(node::UnaryExpr::new(node::UnaryOp::Minus, expr, Span::single(Position::new(self.current_token.line as u32, self.current_token.column as u32, 0))))
            }
            _ => self.parse_primary()?,  // 当前token不是一元操作符，解析基本表达式
        };
        self.exit_recursion();
        Ok(result)
    }

    /// 解析指数表达式
    fn parse_exponentiation(&mut self) -> Result<Expr, ParseError> {
        self.enter_recursion()?;
        let mut expr = self.parse_unary()?;

        // 指数运算是右结合的，使用迭代方法处理，避免递归导致栈溢出
        if self.current_token.kind == TokenKind::Exp {
            // 收集所有连续的指数操作
            let mut right_operands = Vec::new();

            while self.current_token.kind == TokenKind::Exp {
                self.next_token(); // consume '^'
                right_operands.push(self.parse_unary()?);
            }

            // 从最右边开始构建表达式树（右结合性）
            // 例如 a^b^c 应该被解释为 a^(b^c)，而不是 (a^b)^c
            for operand in right_operands.into_iter().rev() {
                expr = Box::new(node::BinaryExpr::new(
                    expr,
                    node::BinaryOp::Exp,
                    operand,
                    Span::single(Position::new(self.current_token.line as u32, self.current_token.column as u32, 0))
                ));
            }
        }

        self.exit_recursion();
        Ok(expr)
    }

    /// 解析基本表达式
    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        self.enter_recursion()?;
        // 检查当前token类型并进行相应处理
        let expr = match self.current_token.kind.clone() { // 这里clone token kind避免引用
            TokenKind::IntegerLiteral(n) => {
                let value = crate::core::Value::Int(n);
                self.next_token();
                Box::new(node::ConstantExpr::new(value, Span::single(Position::new(self.current_token.line as u32, self.current_token.column as u32, 0)))) as Expr
            }
            TokenKind::FloatLiteral(n) => {
                let value = crate::core::Value::Float(n);
                self.next_token();
                Box::new(node::ConstantExpr::new(value, Span::single(Position::new(self.current_token.line as u32, self.current_token.column as u32, 0)))) as Expr
            }
            TokenKind::StringLiteral(s) => {
                let value = crate::core::Value::String(s);
                self.next_token();
                Box::new(node::ConstantExpr::new(value, Span::single(Position::new(self.current_token.line as u32, self.current_token.column as u32, 0)))) as Expr
            }
            TokenKind::BooleanLiteral(b) => {
                let value = crate::core::Value::Bool(b);
                self.next_token();
                Box::new(node::ConstantExpr::new(value, Span::single(Position::new(self.current_token.line as u32, self.current_token.column as u32, 0)))) as Expr
            }
            TokenKind::Null => {
                let value = crate::core::Value::Null(crate::core::NullType::Null);
                self.next_token();
                Box::new(node::ConstantExpr::new(value, Span::single(Position::new(self.current_token.line as u32, self.current_token.column as u32, 0)))) as Expr
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
                Box::new(node::ListExpr::new(elements, Span::single(Position::new(self.current_token.line as u32, self.current_token.column as u32, 0)))) as Expr
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
                Box::new(node::MapExpr::new(pairs, Span::single(Position::new(self.current_token.line as u32, self.current_token.column as u32, 0)))) as Expr
            }
            TokenKind::Identifier(name) => {
                // 首先获取下一个token的类型，以判断是函数调用还是变量/属性访问
                let next_token_kind = self.peek_token();
                self.next_token(); // 消费当前的identifier token

                if next_token_kind == TokenKind::LParen {
                    // 这是一个函数调用
                    let func_name = name;
                    self.expect_token(TokenKind::LParen)?; // 现在消费 '('

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

                    Box::new(FunctionCallExpr::new(func_name, args, false, Span::single(Position::new(self.current_token.line as u32, self.current_token.column as u32, 0)))) as Expr
                } else {
                    // 这是一个变量或属性访问
                    let var_name = name;

                    // 检查是否跟着点号进行属性访问
                    if self.current_token.kind == TokenKind::Dot {
                        self.next_token();
                        let prop_name = self.parse_identifier()?;
                        Box::new(node::PropertyAccessExpr::new(
                            Box::new(node::VariableExpr::new(var_name, Span::single(Position::new(self.current_token.line as u32, self.current_token.column as u32, 0)))) as Expr,
                            prop_name,
                            Span::single(Position::new(self.current_token.line as u32, self.current_token.column as u32, 0))
                        ))
                    } else {
                        Box::new(node::VariableExpr::new(var_name, Span::single(Position::new(self.current_token.line as u32, self.current_token.column as u32, 0)))) as Expr
                    }
                }
            }
            _ => {
                self.exit_recursion();
                return Err(ParseError::syntax_error(
                    format!("Unexpected token in expression: {:?}", self.current_token.kind),
                    self.current_token.line,
                    self.current_token.column,
                ));
            }
        };
        self.exit_recursion();

        Ok(expr)
    }
}