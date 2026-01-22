//! 表达式解析模块
//!
//! 负责解析各种表达式，包括算术表达式、逻辑表达式、函数调用等。

use crate::query::parser::ast::types::{BinaryOp, UnaryOp};
use crate::query::parser::ast::expr::*;
use crate::query::parser::core::error::{ParseError, ParseErrorKind};
use crate::query::parser::core::position::Position;
use crate::query::parser::core::span::Span;
use crate::query::parser::parser::ParseContext;
use crate::query::parser::TokenKind;

pub struct ExprParser<'a> {
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> ExprParser<'a> {
    pub fn new(_ctx: &ParseContext<'a>) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn parse_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<Expr, ParseError> {
        self.parse_or_expression(ctx)
    }

    fn parse_or_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<Expr, ParseError> {
        let mut left = self.parse_and_expression(ctx)?;

        while ctx.match_token(TokenKind::Or) {
            let op = BinaryOp::Or;
            let right = self.parse_and_expression(ctx)?;
            let span = ctx.merge_span(left.span().start, right.span().end);
            left = Expr::Binary(BinaryExpr::new(left, op, right, span));
        }

        Ok(left)
    }

    fn parse_and_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<Expr, ParseError> {
        let mut left = self.parse_not_expression(ctx)?;

        while ctx.match_token(TokenKind::And) {
            let op = BinaryOp::And;
            let right = self.parse_not_expression(ctx)?;
            let span = ctx.merge_span(left.span().start, right.span().end);
            left = Expr::Binary(BinaryExpr::new(left, op, right, span));
        }

        Ok(left)
    }

    fn parse_not_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<Expr, ParseError> {
        if ctx.match_token(TokenKind::Not) {
            let op = UnaryOp::Not;
            let operand = self.parse_not_expression(ctx)?;
            let span = ctx.merge_span(operand.span().start, operand.span().end);
            Ok(Expr::Unary(UnaryExpr::new(op, operand, span)))
        } else {
            self.parse_comparison_expression(ctx)
        }
    }

    fn parse_comparison_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<Expr, ParseError> {
        let mut left = self.parse_additive_expression(ctx)?;

        if let Some(op) = self.parse_comparison_op(ctx) {
            let right = self.parse_additive_expression(ctx)?;
            let span = ctx.merge_span(left.span().start, right.span().end);
            left = Expr::Binary(BinaryExpr::new(left, op, right, span));
        }

        Ok(left)
    }

    fn parse_comparison_op(&mut self, ctx: &mut ParseContext<'a>) -> Option<BinaryOp> {
        match ctx.current_token().kind {
            TokenKind::Eq => {
                ctx.next_token();
                Some(BinaryOp::Eq)
            }
            TokenKind::Ne => {
                ctx.next_token();
                Some(BinaryOp::Ne)
            }
            TokenKind::Lt => {
                ctx.next_token();
                Some(BinaryOp::Lt)
            }
            TokenKind::Le => {
                ctx.next_token();
                Some(BinaryOp::Le)
            }
            TokenKind::Gt => {
                ctx.next_token();
                Some(BinaryOp::Gt)
            }
            TokenKind::Ge => {
                ctx.next_token();
                Some(BinaryOp::Ge)
            }
            TokenKind::Regex => {
                ctx.next_token();
                Some(BinaryOp::Regex)
            }
            TokenKind::Contains => {
                ctx.next_token();
                Some(BinaryOp::Contains)
            }
            TokenKind::StartsWith => {
                ctx.next_token();
                Some(BinaryOp::StartsWith)
            }
            TokenKind::EndsWith => {
                ctx.next_token();
                Some(BinaryOp::EndsWith)
            }
            _ => None,
        }
    }

    fn parse_additive_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplicative_expression(ctx)?;

        while let Some(op) = self.parse_additive_op(ctx) {
            let right = self.parse_multiplicative_expression(ctx)?;
            let span = ctx.merge_span(left.span().start, right.span().end);
            left = Expr::Binary(BinaryExpr::new(left, op, right, span));
        }

        Ok(left)
    }

    fn parse_additive_op(&mut self, ctx: &mut ParseContext<'a>) -> Option<BinaryOp> {
        match ctx.current_token().kind {
            TokenKind::Plus => {
                ctx.next_token();
                Some(BinaryOp::Add)
            }
            TokenKind::Minus => {
                ctx.next_token();
                Some(BinaryOp::Sub)
            }
            _ => None,
        }
    }

    fn parse_multiplicative_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary_expression(ctx)?;

        while let Some(op) = self.parse_multiplicative_op(ctx) {
            let right = self.parse_unary_expression(ctx)?;
            let span = ctx.merge_span(left.span().start, right.span().end);
            left = Expr::Binary(BinaryExpr::new(left, op, right, span));
        }

        Ok(left)
    }

    fn parse_multiplicative_op(&mut self, ctx: &mut ParseContext<'a>) -> Option<BinaryOp> {
        match ctx.current_token().kind {
            TokenKind::Star => {
                ctx.next_token();
                Some(BinaryOp::Mul)
            }
            TokenKind::Div => {
                ctx.next_token();
                Some(BinaryOp::Div)
            }
            TokenKind::Mod => {
                ctx.next_token();
                Some(BinaryOp::Mod)
            }
            _ => None,
        }
    }

    fn parse_unary_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<Expr, ParseError> {
        if ctx.match_token(TokenKind::Minus) {
            let op = UnaryOp::Minus;
            let operand = self.parse_unary_expression(ctx)?;
            let span = ctx.merge_span(operand.span().start, operand.span().end);
            Ok(Expr::Unary(UnaryExpr::new(op, operand, span)))
        } else if ctx.match_token(TokenKind::Plus) {
            let op = UnaryOp::Plus;
            let operand = self.parse_unary_expression(ctx)?;
            let span = ctx.merge_span(operand.span().start, operand.span().end);
            Ok(Expr::Unary(UnaryExpr::new(op, operand, span)))
        } else if ctx.match_token(TokenKind::NotOp) {
            let op = UnaryOp::Not;
            let operand = self.parse_unary_expression(ctx)?;
            let span = ctx.merge_span(operand.span().start, operand.span().end);
            Ok(Expr::Unary(UnaryExpr::new(op, operand, span)))
        } else {
            self.parse_exponentiation_expression(ctx)
        }
    }

    fn parse_exponentiation_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<Expr, ParseError> {
        let mut expr = self.parse_postfix_expression(ctx)?;

        if ctx.match_token(TokenKind::Exp) {
            let mut right_operands = Vec::new();

            while ctx.match_token(TokenKind::Exp) {
                right_operands.push(self.parse_unary_expression(ctx)?);
            }

            for operand in right_operands.into_iter().rev() {
                let span = ctx.merge_span(expr.span().start, operand.span().end);
                expr = Expr::Binary(BinaryExpr::new(expr, BinaryOp::Exponent, operand, span));
            }
        }

        Ok(expr)
    }

    fn parse_postfix_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary_expression(ctx)?;

        loop {
            if ctx.match_token(TokenKind::LBracket) {
                let index = self.parse_expression(ctx)?;
                ctx.expect_token(TokenKind::RBracket)?;
                let span = ctx.merge_span(expr.span().start, ctx.current_position());
                expr = Expr::Subscript(SubscriptExpr::new(expr, index, span));
            } else if ctx.match_token(TokenKind::Dot) {
                let property = ctx.expect_identifier()?;
                let span = ctx.merge_span(expr.span().start, ctx.current_position());
                expr = Expr::PropertyAccess(PropertyAccessExpr::new(expr, property, span));
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_primary_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<Expr, ParseError> {
        let token = ctx.current_token().clone();
        let start_pos = ctx.current_position();

        match token.kind {
            TokenKind::LParen => {
                ctx.next_token();
                let expr = self.parse_expression(ctx)?;
                ctx.expect_token(TokenKind::RParen)?;
                let span = ctx.merge_span(start_pos, ctx.current_position());
                Ok(Expr::Grouped(Box::new(expr), span))
            }
            TokenKind::Identifier(name) => {
                ctx.next_token();
                let span = ctx.merge_span(start_pos, ctx.current_position());
                if ctx.match_token(TokenKind::LParen) {
                    self.parse_function_call(name, span, ctx)
                } else {
                    Ok(Expr::Variable(name, span))
                }
            }
            TokenKind::IntegerLiteral(n) => {
                ctx.next_token();
                let span = ctx.merge_span(start_pos, ctx.current_position());
                Ok(Expr::Constant(Constant::Int(n), span))
            }
            TokenKind::FloatLiteral(f) => {
                ctx.next_token();
                let span = ctx.merge_span(start_pos, ctx.current_position());
                Ok(Expr::Constant(Constant::Float(f), span))
            }
            TokenKind::StringLiteral(s) => {
                ctx.next_token();
                let span = ctx.merge_span(start_pos, ctx.current_position());
                Ok(Expr::Constant(Constant::String(s), span))
            }
            TokenKind::BooleanLiteral(b) => {
                ctx.next_token();
                let span = ctx.merge_span(start_pos, ctx.current_position());
                Ok(Expr::Constant(Constant::Bool(b), span))
            }
            TokenKind::Null => {
                ctx.next_token();
                let span = ctx.merge_span(start_pos, ctx.current_position());
                Ok(Expr::Constant(Constant::Null, span))
            }
            TokenKind::Count | TokenKind::Sum | TokenKind::Avg | TokenKind::Min | TokenKind::Max => {
                let func_name = token.lexeme.clone();
                ctx.next_token();
                let span = ctx.merge_span(start_pos, ctx.current_position());
                self.parse_function_call(func_name, span, ctx)
            }
            TokenKind::List => {
                ctx.next_token();
                let elements = self.parse_expression_list(ctx)?;
                ctx.expect_token(TokenKind::RBracket)?;
                let span = ctx.merge_span(start_pos, ctx.current_position());
                Ok(Expr::List(elements, span))
            }
            TokenKind::LBracket => {
                ctx.next_token();
                let elements = self.parse_expression_list(ctx)?;
                ctx.expect_token(TokenKind::RBracket)?;
                let span = ctx.merge_span(start_pos, ctx.current_position());
                Ok(Expr::List(elements, span))
            }
            TokenKind::Map => {
                ctx.next_token();
                ctx.expect_token(TokenKind::LBrace)?;
                let properties = self.parse_property_list(ctx)?;
                ctx.expect_token(TokenKind::RBrace)?;
                let span = ctx.merge_span(start_pos, ctx.current_position());
                Ok(Expr::Map(properties, span))
            }
            TokenKind::LBrace => {
                ctx.next_token();
                let properties = self.parse_property_list(ctx)?;
                ctx.expect_token(TokenKind::RBrace)?;
                let span = ctx.merge_span(start_pos, ctx.current_position());
                Ok(Expr::Map(properties, span))
            }
            _ => {
                Err(ParseError::new(
                    ParseErrorKind::UnexpectedToken,
                    format!("Unexpected token in expression: {:?}", token.kind),
                    start_pos,
                ))
            }
        }
    }

    fn parse_function_call(&mut self, name: String, span: Span, ctx: &mut ParseContext<'a>) -> Result<Expr, ParseError> {
        let args = if ctx.match_token(TokenKind::RParen) {
            Vec::new()
        } else {
            let args = self.parse_expression_list(ctx)?;
            ctx.expect_token(TokenKind::RParen)?;
            args
        };
        Ok(Expr::FunctionCall(FunctionCallExpr::new(name, args, span)))
    }

    fn parse_expression_list(&mut self, ctx: &mut ParseContext<'a>) -> Result<Vec<Expr>, ParseError> {
        let mut expressions = Vec::new();
        expressions.push(self.parse_expression(ctx)?);
        while ctx.match_token(TokenKind::Comma) {
            expressions.push(self.parse_expression(ctx)?);
        }
        Ok(expressions)
    }

    fn parse_property_list(&mut self, ctx: &mut ParseContext<'a>) -> Result<Vec<(String, Expr)>, ParseError> {
        let mut properties = Vec::new();
        while !ctx.match_token(TokenKind::RBrace) {
            let key = ctx.expect_identifier()?;
            ctx.expect_token(TokenKind::Colon)?;
            let value = self.parse_expression(ctx)?;
            properties.push((key, value));
            if !ctx.match_token(TokenKind::Comma) {
                break;
            }
        }
        Ok(properties)
    }

    pub fn set_compat_mode(&mut self, _enabled: bool) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_expression() {
        let input = "1 + 2 * 3";
        let ctx = &mut ParseContext::new(input);
        let mut parser = ExprParser::new(ctx);
        let result = parser.parse_expression(ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_parenthesized_expression() {
        let input = "(1 + 2) * 3";
        let ctx = &mut ParseContext::new(input);
        let mut parser = ExprParser::new(ctx);
        let result = parser.parse_expression(ctx);
        assert!(result.is_ok());
    }
}
