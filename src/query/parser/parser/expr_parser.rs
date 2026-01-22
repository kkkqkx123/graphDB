//! 表达式解析模块
//!
//! 负责解析各种表达式，包括算术表达式、逻辑表达式、函数调用等。
//! 提供两种使用方式：
//! 1. 独立的 `ExprParser` 结构体
//! 2. 作为 `Parser` 的方法

use crate::core::Value;
use crate::query::parser::ast::expr::*;
use crate::query::parser::ast::types::*;
use crate::query::parser::core::error::ParseErrorKind;
use crate::query::parser::lexer::{Lexer, TokenKind as LexerToken};

/// 独立的表达式解析器
///
/// 用于独立解析表达式的场景，与完整的 SQL Parser 分离
pub struct ExprParser {
    lexer: Lexer,
}

impl ExprParser {
    /// 创建表达式解析器
    pub fn new(input: &str) -> Self {
        Self {
            lexer: Lexer::new(input),
        }
    }

    /// 解析表达式
    pub fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_or_expression()
    }

    /// 解析 OR 表达式
    fn parse_or_expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and_expression()?;

        while self.match_token(LexerToken::Or) {
            let op = BinaryOp::Or;
            let right = self.parse_and_expression()?;
            let span = Span::new(left.span().start, right.span().end);
            left = Expr::Binary(BinaryExpr::new(left, op, right, span));
        }

        Ok(left)
    }

    /// 解析 AND 表达式
    fn parse_and_expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_not_expression()?;

        while self.match_token(LexerToken::And) {
            let op = BinaryOp::And;
            let right = self.parse_not_expression()?;
            let span = Span::new(left.span().start, right.span().end);
            left = Expr::Binary(BinaryExpr::new(left, op, right, span));
        }

        Ok(left)
    }

    /// 解析 NOT 表达式
    fn parse_not_expression(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(LexerToken::Not) {
            let op = UnaryOp::Not;
            let operand = self.parse_not_expression()?;
            let span = Span::new(operand.span().start, operand.span().end);
            Ok(Expr::Unary(UnaryExpr::new(op, operand, span)))
        } else {
            self.parse_comparison_expression()
        }
    }

    /// 解析比较表达式
    fn parse_comparison_expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_additive_expression()?;

        if let Some(op) = self.parse_comparison_op() {
            let right = self.parse_additive_expression()?;
            let span = Span::new(left.span().start, right.span().end);
            left = Expr::Binary(BinaryExpr::new(left, op, right, span));
        }

        Ok(left)
    }

    /// 解析加法表达式
    fn parse_additive_expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplicative_expression()?;

        while let Some(op) = self.parse_additive_op() {
            let right = self.parse_multiplicative_expression()?;
            let span = Span::new(left.span().start, right.span().end);
            left = Expr::Binary(BinaryExpr::new(left, op, right, span));
        }

        Ok(left)
    }

    /// 解析乘法表达式
    fn parse_multiplicative_expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary_expression()?;

        while let Some(op) = self.parse_multiplicative_op() {
            let right = self.parse_unary_expression()?;
            let span = Span::new(left.span().start, right.span().end);
            left = Expr::Binary(BinaryExpr::new(left, op, right, span));
        }

        Ok(left)
    }

    /// 解析一元表达式
    fn parse_unary_expression(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(LexerToken::Minus) {
            let op = UnaryOp::Minus;
            let operand = self.parse_unary_expression()?;
            let span = Span::new(operand.span().start, operand.span().end);
            Ok(Expr::Unary(UnaryExpr::new(op, operand, span)))
        } else if self.match_token(LexerToken::Plus) {
            let op = UnaryOp::Plus;
            let operand = self.parse_unary_expression()?;
            let span = Span::new(operand.span().start, operand.span().end);
            Ok(Expr::Unary(UnaryExpr::new(op, operand, span)))
        } else {
            self.parse_primary_expression()
        }
    }

    /// 解析基本表达式
    fn parse_primary_expression(&mut self) -> Result<Expr, ParseError> {
        let token = self.lexer.peek().map_err(|e| ParseError::from(e))?;

        match token.kind {
            LexerToken::IntegerLiteral(_) => {
                let value = self.parse_integer()?;
                let span = self.current_span();
                Ok(Expr::Constant(ConstantExpr::new(Value::Int(value), span)))
            }
            LexerToken::FloatLiteral(_) => {
                let value = self.parse_float()?;
                let span = self.current_span();
                Ok(Expr::Constant(ConstantExpr::new(Value::Float(value), span)))
            }
            LexerToken::StringLiteral(_) => {
                let value = self.parse_string()?;
                let span = self.current_span();
                Ok(Expr::Constant(ConstantExpr::new(
                    Value::String(value),
                    span,
                )))
            }
            LexerToken::BooleanLiteral(_) => {
                let value = self.parse_boolean()?;
                let span = self.current_span();
                Ok(Expr::Constant(ConstantExpr::new(Value::Bool(value), span)))
            }
            LexerToken::Identifier(_) => {
                let name = self.expect_identifier()?;
                let span = self.current_span();

                if self.match_token(LexerToken::LParen) {
                    self.parse_function_call(name, span)
                } else {
                    Ok(Expr::Variable(VariableExpr::new(name, span)))
                }
            }
            LexerToken::Count | LexerToken::Sum | LexerToken::Avg | LexerToken::Min | LexerToken::Max => {
                let name = token.lexeme.clone();
                let span = self.current_span();
                self.lexer.advance();

                if self.match_token(LexerToken::LParen) {
                    self.parse_function_call(name, span)
                } else {
                    Ok(Expr::Variable(VariableExpr::new(name, span)))
                }
            }
            LexerToken::LParen => {
                self.lexer.advance();
                let expr = self.parse_expression()?;
                self.expect_token(LexerToken::RParen)?;
                Ok(expr)
            }
            LexerToken::LBracket => self.parse_list_expression(),
            LexerToken::LBrace => self.parse_map_expression(),
            _ => Err(self.parse_error(format!("Unexpected token: {:?}", token.kind))),
        }
    }

    /// 解析函数调用
    fn parse_function_call(&mut self, name: String, span: Span) -> Result<Expr, ParseError> {
        let mut args = Vec::new();
        let mut distinct = false;

        if self.match_token(LexerToken::Distinct) {
            distinct = true;
        }

        if !self.check_token(LexerToken::RParen) {
            loop {
                let arg = self.parse_expression()?;
                args.push(arg);

                if !self.match_token(LexerToken::Comma) {
                    break;
                }
            }
        }

        self.expect_token(LexerToken::RParen)?;
        let end_span = self.current_span();
        let full_span = Span::new(span.start, end_span.end);

        Ok(Expr::FunctionCall(FunctionCallExpr::new(
            name, args, distinct, full_span,
        )))
    }

    /// 解析列表表达式
    fn parse_list_expression(&mut self) -> Result<Expr, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::LBracket)?;

        let mut elements = Vec::new();

        if !self.check_token(LexerToken::RBracket) {
            loop {
                let elem = self.parse_expression()?;
                elements.push(elem);

                if !self.match_token(LexerToken::Comma) {
                    break;
                }
            }
        }

        self.expect_token(LexerToken::RBracket)?;
        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Expr::List(ListExpr::new(elements, span)))
    }

    /// 解析映射表达式
    fn parse_map_expression(&mut self) -> Result<Expr, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::LBrace)?;

        let mut pairs = Vec::new();

        if !self.check_token(LexerToken::RBrace) {
            loop {
                let key = self.expect_identifier()?;
                self.expect_token(LexerToken::Colon)?;
                let value = self.parse_expression()?;
                pairs.push((key, value));

                if !self.match_token(LexerToken::Comma) {
                    break;
                }
            }
        }

        self.expect_token(LexerToken::RBrace)?;
        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Expr::Map(MapExpr::new(pairs, span)))
    }

    /// 解析比较操作符
    fn parse_comparison_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(LexerToken::Eq) {
            Some(BinaryOp::Equal)
        } else if self.match_token(LexerToken::Ne) {
            Some(BinaryOp::NotEqual)
        } else if self.match_token(LexerToken::Lt) {
            Some(BinaryOp::LessThan)
        } else if self.match_token(LexerToken::Le) {
            Some(BinaryOp::LessThanOrEqual)
        } else if self.match_token(LexerToken::Gt) {
            Some(BinaryOp::GreaterThan)
        } else if self.match_token(LexerToken::Ge) {
            Some(BinaryOp::GreaterThanOrEqual)
        } else {
            None
        }
    }

    /// 解析加法操作符
    fn parse_additive_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(LexerToken::Plus) {
            Some(BinaryOp::Add)
        } else if self.match_token(LexerToken::Minus) {
            Some(BinaryOp::Subtract)
        } else {
            None
        }
    }

    /// 解析乘法操作符
    fn parse_multiplicative_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(LexerToken::Star) {
            Some(BinaryOp::Multiply)
        } else if self.match_token(LexerToken::Div) {
            Some(BinaryOp::Divide)
        } else if self.match_token(LexerToken::Mod) {
            Some(BinaryOp::Modulo)
        } else {
            None
        }
    }

    /// 解析整数
    fn parse_integer(&mut self) -> Result<i64, ParseError> {
        let token = self.lexer.peek().map_err(|e| ParseError::from(e))?;
        if let LexerToken::IntegerLiteral(n) = token.kind {
            let text = token.lexeme.clone();
            self.lexer.advance();
            text.parse().map_err(|_| self.parse_error(format!("Invalid integer: {}", text)))
        } else {
            Err(self.parse_error(format!("Expected integer, found {:?}", token.kind)))
        }
    }

    /// 解析浮点数
    fn parse_float(&mut self) -> Result<f64, ParseError> {
        let token = self.lexer.peek().map_err(|e| ParseError::from(e))?;
        if let LexerToken::FloatLiteral(n) = token.kind {
            let text = token.lexeme.clone();
            self.lexer.advance();
            text.parse().map_err(|_| self.parse_error(format!("Invalid float: {}", text)))
        } else {
            Err(self.parse_error(format!("Expected float, found {:?}", token.kind)))
        }
    }

    /// 解析字符串
    fn parse_string(&mut self) -> Result<String, ParseError> {
        let token = self.lexer.peek().map_err(|e| ParseError::from(e))?;
        match &token.kind {
            LexerToken::StringLiteral(s) => {
                self.lexer.advance();
                Ok(s.trim_matches('"').to_string())
            }
            _ => Err(self.parse_error(format!("Expected string, found {:?}", token.kind))),
        }
    }

    /// 解析布尔值
    fn parse_boolean(&mut self) -> Result<bool, ParseError> {
        let token = self.lexer.peek().map_err(|e| ParseError::from(e))?;
        if let LexerToken::BooleanLiteral(b) = token.kind {
            let text = token.lexeme.clone();
            self.lexer.advance();
            text.parse().map_err(|_| self.parse_error(format!("Invalid boolean: {}", text)))
        } else {
            Err(self.parse_error(format!("Expected boolean, found {:?}", token.kind)))
        }
    }

    fn match_token(&mut self, expected: LexerToken) -> bool {
        if self.lexer.check(expected.clone()) {
            let _ = self.lexer.advance();
            true
        } else {
            false
        }
    }

    fn check_token(&mut self, expected: LexerToken) -> bool {
        self.lexer.check(expected.clone())
    }

    fn expect_token(&mut self, expected: LexerToken) -> Result<(), ParseError> {
        let token = self.lexer.peek().map_err(|e| ParseError::from(e))?;
        if token.kind == expected {
            self.lexer.advance();
            Ok(())
        } else {
            Err(self.parse_error(format!("Expected {:?}, found {:?}", expected, token.kind)))
        }
    }

    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        let token = self.lexer.peek().map_err(|e| ParseError::from(e))?;
        if let LexerToken::Identifier(_) = token.kind {
            let text = token.lexeme.clone();
            self.lexer.advance();
            Ok(text)
        } else {
            Err(self.parse_error(format!("Expected identifier, found {:?}", token.kind)))
        }
    }

    fn current_span(&self) -> Span {
        let pos = self.lexer.current_position();
        Span::new(
            Position::new(pos.line, pos.column),
            Position::new(pos.line, pos.column),
        )
    }

    fn parse_error(&self, message: String) -> ParseError {
        let pos = self.lexer.current_position();
        ParseError::new(ParseErrorKind::SyntaxError, message, pos.line, pos.column)
    }
}

impl super::Parser {
    /// 解析表达式
    pub fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_or_expression()
    }

    /// 解析 OR 表达式
    fn parse_or_expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and_expression()?;

        while self.match_token(LexerToken::Or) {
            let op = BinaryOp::Or;
            let right = self.parse_and_expression()?;
            let span = Span::new(left.span().start, right.span().end);
            left = Expr::Binary(BinaryExpr::new(left, op, right, span));
        }

        Ok(left)
    }

    /// 解析 AND 表达式
    fn parse_and_expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_not_expression()?;

        while self.match_token(LexerToken::And) {
            let op = BinaryOp::And;
            let right = self.parse_not_expression()?;
            let span = Span::new(left.span().start, right.span().end);
            left = Expr::Binary(BinaryExpr::new(left, op, right, span));
        }

        Ok(left)
    }

    /// 解析 NOT 表达式
    fn parse_not_expression(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(LexerToken::Not) {
            let op = UnaryOp::Not;
            let operand = self.parse_not_expression()?;
            let span = Span::new(operand.span().start, operand.span().end);
            Ok(Expr::Unary(UnaryExpr::new(op, operand, span)))
        } else {
            self.parse_comparison_expression()
        }
    }

    /// 解析比较表达式
    fn parse_comparison_expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_additive_expression()?;

        // 检查比较操作符
        if let Some(op) = self.parse_comparison_op() {
            let right = self.parse_additive_expression()?;
            let span = Span::new(left.span().start, right.span().end);
            left = Expr::Binary(BinaryExpr::new(left, op, right, span));
        }

        Ok(left)
    }

    /// 解析加法表达式
    fn parse_additive_expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplicative_expression()?;

        while let Some(op) = self.parse_additive_op() {
            let right = self.parse_multiplicative_expression()?;
            let span = Span::new(left.span().start, right.span().end);
            left = Expr::Binary(BinaryExpr::new(left, op, right, span));
        }

        Ok(left)
    }

    /// 解析乘法表达式
    fn parse_multiplicative_expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary_expression()?;

        while let Some(op) = self.parse_multiplicative_op() {
            let right = self.parse_unary_expression()?;
            let span = Span::new(left.span().start, right.span().end);
            left = Expr::Binary(BinaryExpr::new(left, op, right, span));
        }

        Ok(left)
    }

    /// 解析一元表达式
    fn parse_unary_expression(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(LexerToken::Minus) {
            let op = UnaryOp::Minus;
            let operand = self.parse_unary_expression()?;
            let span = Span::new(operand.span().start, operand.span().end);
            Ok(Expr::Unary(UnaryExpr::new(op, operand, span)))
        } else if self.match_token(LexerToken::Plus) {
            let op = UnaryOp::Plus;
            let operand = self.parse_unary_expression()?;
            let span = Span::new(operand.span().start, operand.span().end);
            Ok(Expr::Unary(UnaryExpr::new(op, operand, span)))
        } else {
            self.parse_exponentiation_expression()
        }
    }

    /// 解析指数表达式
    fn parse_exponentiation_expression(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_postfix_expression()?;

        // 指数运算是右结合的，使用迭代方法处理，避免递归导致栈溢出
        if self.match_token(LexerToken::Exp) {
            // 收集所有连续的指数操作
            let mut right_operands = Vec::new();

            while self.match_token(LexerToken::Exp) {
                right_operands.push(self.parse_unary_expression()?);
            }

            // 从最右边开始构建表达式树（右结合性）
            // 例如 a^b^c 应该被解释为 a^(b^c)，而不是 (a^b)^c
            for operand in right_operands.into_iter().rev() {
                let span = Span::new(expr.span().start, operand.span().end);
                expr = Expr::Binary(BinaryExpr::new(expr, BinaryOp::Exponent, operand, span));
            }
        }

        Ok(expr)
    }

    /// 解析后缀表达式
    fn parse_postfix_expression(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary_expression()?;

        loop {
            if self.match_token(LexerToken::LBracket) {
                // 下标访问
                let index = self.parse_expression()?;
                self.expect_token(LexerToken::RBracket)?;
                let span = Span::new(expr.span().start, self.lexer.current_position());
                expr = Expr::Subscript(SubscriptExpr::new(expr, index, span));
            } else if self.match_token(LexerToken::Dot) {
                // 属性访问
                let property = self.expect_identifier()?;
                let span = Span::new(expr.span().start, self.lexer.current_position());
                expr = Expr::PropertyAccess(PropertyAccessExpr::new(expr, property, span));
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// 解析基本表达式
    fn parse_primary_expression(&mut self) -> Result<Expr, ParseError> {
        let token = self.lexer.peek()?;

        match token.kind {
            LexerToken::IntegerLiteral(_) => {
                let value = self.parse_integer()?;
                let span = self.parser_current_span();
                Ok(Expr::Constant(ConstantExpr::new(Value::Int(value), span)))
            }
            LexerToken::FloatLiteral(_) => {
                let value = self.parse_float()?;
                let span = self.parser_current_span();
                Ok(Expr::Constant(ConstantExpr::new(Value::Float(value), span)))
            }
            LexerToken::StringLiteral(_) => {
                let value = self.parse_string()?;
                let span = self.parser_current_span();
                Ok(Expr::Constant(ConstantExpr::new(
                    Value::String(value),
                    span,
                )))
            }
            LexerToken::BooleanLiteral(_) => {
                let value = self.parse_boolean()?;
                let span = self.parser_current_span();
                Ok(Expr::Constant(ConstantExpr::new(Value::Bool(value), span)))
            }
            LexerToken::Identifier(_) => {
                let name = self.expect_identifier()?;
                let span = self.parser_current_span();

                // 检查是否是函数调用
                if self.match_token(LexerToken::LParen) {
                    self.parse_function_call(name, span)
                } else {
                    Ok(Expr::Variable(VariableExpr::new(name, span)))
                }
            }
            LexerToken::LParen => {
                // 括号表达式
                self.lexer.advance();
                let expr = self.parse_expression()?;
                self.expect_token(LexerToken::RParen)?;
                Ok(expr)
            }
            LexerToken::LBracket => {
                // 列表表达式
                self.parse_list_expression()
            }
            LexerToken::LBrace => {
                // 映射表达式
                self.parse_map_expression()
            }
            _ => {
                let span = self.parser_current_span();
                Err(ParseError::new(
                    ParseErrorKind::UnexpectedToken,
                    format!("Unexpected token: {:?}", token.kind),
                    span.start.line,
                    span.start.column,
                ))
            }
        }
    }

    /// 解析函数调用
    fn parse_function_call(&mut self, name: String, span: Span) -> Result<Expr, ParseError> {
        let mut args = Vec::new();
        let mut distinct = false;

        // 检查 DISTINCT 关键字
        if self.match_token(LexerToken::Distinct) {
            distinct = true;
        }

        // 解析参数列表
        if !self.check_token(LexerToken::RParen) {
            loop {
                let arg = self.parse_expression()?;
                args.push(arg);

                if !self.match_token(LexerToken::Comma) {
                    break;
                }
            }
        }

        self.expect_token(LexerToken::RParen)?;
        let end_span = self.parser_current_span();
        let full_span = Span::new(span.start, end_span.end);

        Ok(Expr::FunctionCall(FunctionCallExpr::new(
            name, args, distinct, full_span,
        )))
    }

    /// 解析列表表达式
    fn parse_list_expression(&mut self) -> Result<Expr, ParseError> {
        let start_span = self.parser_current_span();
        self.expect_token(LexerToken::LBracket)?;

        let mut elements = Vec::new();

        if !self.check_token(LexerToken::RBracket) {
            loop {
                let elem = self.parse_expression()?;
                elements.push(elem);

                if !self.match_token(LexerToken::Comma) {
                    break;
                }
            }
        }

        self.expect_token(LexerToken::RBracket)?;
        let end_span = self.parser_current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Expr::List(ListExpr::new(elements, span)))
    }

    /// 解析映射表达式
    fn parse_map_expression(&mut self) -> Result<Expr, ParseError> {
        let start_span = self.parser_current_span();
        self.expect_token(LexerToken::LBrace)?;

        let mut pairs = Vec::new();

        if !self.check_token(LexerToken::RBrace) {
            loop {
                let key = self.expect_identifier()?;
                self.expect_token(LexerToken::Colon)?;
                let value = self.parse_expression()?;
                pairs.push((key, value));

                if !self.match_token(LexerToken::Comma) {
                    break;
                }
            }
        }

        self.expect_token(LexerToken::RBrace)?;
        let end_span = self.parser_current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Expr::Map(MapExpr::new(pairs, span)))
    }

    /// 解析比较操作符
    fn parse_comparison_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(LexerToken::Eq) {
            Some(BinaryOp::Equal)
        } else if self.match_token(LexerToken::Ne) {
            Some(BinaryOp::NotEqual)
        } else if self.match_token(LexerToken::Lt) {
            Some(BinaryOp::LessThan)
        } else if self.match_token(LexerToken::Le) {
            Some(BinaryOp::LessThanOrEqual)
        } else if self.match_token(LexerToken::Gt) {
            Some(BinaryOp::GreaterThan)
        } else if self.match_token(LexerToken::Ge) {
            Some(BinaryOp::GreaterThanOrEqual)
        } else {
            None
        }
    }

    /// 解析加法操作符
    fn parse_additive_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(LexerToken::Plus) {
            Some(BinaryOp::Add)
        } else if self.match_token(LexerToken::Minus) {
            Some(BinaryOp::Subtract)
        } else {
            None
        }
    }

    /// 解析乘法操作符
    fn parse_multiplicative_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(LexerToken::Star) {
            Some(BinaryOp::Multiply)
        } else if self.match_token(LexerToken::Div) {
            Some(BinaryOp::Divide)
        } else if self.match_token(LexerToken::Mod) {
            Some(BinaryOp::Modulo)
        } else {
            None
        }
    }

    /// 解析整数
    fn parse_integer(&mut self) -> Result<i64, ParseError> {
        if let LexerToken::IntegerLiteral(n) = self.current_token.kind {
            let value = n;
            self.next_token();
            Ok(value)
        } else {
            let span = self.parser_current_span();
            Err(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                format!("Expected integer, found {:?}", self.current_token.kind),
                span.start.line,
                span.start.column,
            ))
        }
    }

    /// 解析浮点数
    fn parse_float(&mut self) -> Result<f64, ParseError> {
        if let LexerToken::FloatLiteral(n) = self.current_token.kind {
            let value = n;
            self.next_token();
            Ok(value)
        } else {
            let span = self.parser_current_span();
            Err(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                format!("Expected float, found {:?}", self.current_token.kind),
                span.start.line,
                span.start.column,
            ))
        }
    }

    /// 解析字符串
    fn parse_string(&mut self) -> Result<String, ParseError> {
        if let LexerToken::StringLiteral(s) = &self.current_token.kind {
            let text = s.clone();
            self.next_token();
            Ok(text.trim_matches('"').to_string())
        } else {
            let span = self.parser_current_span();
            Err(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                format!("Expected string, found {:?}", self.current_token.kind),
                span.start.line,
                span.start.column,
            ))
        }
    }

    /// 解析布尔值
    fn parse_boolean(&mut self) -> Result<bool, ParseError> {
        if let LexerToken::BooleanLiteral(b) = self.current_token.kind {
            let value = b;
            self.next_token();
            Ok(value)
        } else {
            let span = self.parser_current_span();
            Err(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                format!("Expected boolean, found {:?}", self.current_token.kind),
                span.start.line,
                span.start.column,
            ))
        }
    }

    pub fn parse_expression_list(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut expressions = Vec::new();
        loop {
            expressions.push(self.parse_expression()?);
            if self.current_token().kind != LexerToken::Comma {
                break;
            }
            self.next_token();
        }
        Ok(expressions)
    }
}
