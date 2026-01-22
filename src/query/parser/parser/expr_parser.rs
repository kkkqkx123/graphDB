//! 表达式解析模块
//!
//! 负责解析各种表达式，包括算术表达式、逻辑表达式、函数调用等。

use crate::core::Value;
use crate::query::parser::ast::expr::*;
use crate::query::parser::ast::types::*;
use crate::query::parser::core::error::ParseErrorKind;
use crate::query::parser::lexer::{Lexer, TokenKind as LexerToken};

pub struct ExprParser {
    lexer: Lexer,
}

impl ExprParser {
    pub fn new(input: &str) -> Self {
        Self {
            lexer: Lexer::new(input),
        }
    }

    pub fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_or_expression()
    }

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

    fn parse_comparison_expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_additive_expression()?;

        if let Some(op) = self.parse_comparison_op() {
            let right = self.parse_additive_expression()?;
            let span = Span::new(left.span().start, right.span().end);
            left = Expr::Binary(BinaryExpr::new(left, op, right, span));
        }

        Ok(left)
    }

    fn parse_additive_expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplicative_expression()?;

        while let Some(op) = self.parse_additive_op() {
            let right = self.parse_multiplicative_expression()?;
            let span = Span::new(left.span().start, right.span().end);
            left = Expr::Binary(BinaryExpr::new(left, op, right, span));
        }

        Ok(left)
    }

    fn parse_multiplicative_expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary_expression()?;

        while let Some(op) = self.parse_multiplicative_op() {
            let right = self.parse_unary_expression()?;
            let span = Span::new(left.span().start, right.span().end);
            left = Expr::Binary(BinaryExpr::new(left, op, right, span));
        }

        Ok(left)
    }

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

    fn parse_exponentiation_expression(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_postfix_expression()?;

        if self.match_token(LexerToken::Exp) {
            let mut right_operands = Vec::new();

            while self.match_token(LexerToken::Exp) {
                right_operands.push(self.parse_unary_expression()?);
            }

            for operand in right_operands.into_iter().rev() {
                let span = Span::new(expr.span().start, operand.span().end);
                expr = Expr::Binary(BinaryExpr::new(expr, BinaryOp::Exponent, operand, span));
            }
        }

        Ok(expr)
    }

    fn parse_postfix_expression(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary_expression()?;

        loop {
            if self.match_token(LexerToken::LBracket) {
                let index = self.parse_expression()?;
                self.expect_token(LexerToken::RBracket)?;
                let span = Span::new(expr.span().start, self.lexer.current_position());
                expr = Expr::Subscript(SubscriptExpr::new(expr, index, span));
            } else if self.match_token(LexerToken::Dot) {
                let property = self.expect_identifier()?;
                let span = Span::new(expr.span().start, self.lexer.current_position());
                expr = Expr::PropertyAccess(PropertyAccessExpr::new(expr, property, span));
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_primary_expression(&mut self) -> Result<Expr, ParseError> {
        let token = self.lexer.peek()?;

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
                Ok(Expr::Constant(ConstantExpr::new(Value::String(value), span)))
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

    fn parse_additive_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(LexerToken::Plus) {
            Some(BinaryOp::Add)
        } else if self.match_token(LexerToken::Minus) {
            Some(BinaryOp::Subtract)
        } else {
            None
        }
    }

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
