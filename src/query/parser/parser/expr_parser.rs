//! 表达式解析模块
//!
//! 负责解析各种表达式，包括算术表达式、逻辑表达式、函数调用等。

use crate::core::Value;
use crate::query::parser::ast::types::{BinaryOp, UnaryOp};
use crate::query::parser::ast::expression::*;
use crate::query::parser::core::error::{ParseError, ParseErrorKind};
use crate::core::types::Span;
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

    pub fn parse_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<Expression, ParseError> {
        self.parse_or_expression(ctx)
    }

    fn parse_or_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<Expression, ParseError> {
        let mut left = self.parse_and_expression(ctx)?;

        while ctx.match_token(TokenKind::Or) {
            let op = BinaryOp::Or;
            let right = self.parse_and_expression(ctx)?;
            let span = ctx.merge_span(left.span().start, right.span().end);
            left = Expression::Binary(BinaryExpression::new(left, op, right, span));
        }

        Ok(left)
    }

    fn parse_and_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<Expression, ParseError> {
        let mut left = self.parse_not_expression(ctx)?;

        while ctx.match_token(TokenKind::And) {
            let op = BinaryOp::And;
            let right = self.parse_not_expression(ctx)?;
            let span = ctx.merge_span(left.span().start, right.span().end);
            left = Expression::Binary(BinaryExpression::new(left, op, right, span));
        }

        Ok(left)
    }

    fn parse_not_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<Expression, ParseError> {
        if ctx.match_token(TokenKind::Not) {
            let op = UnaryOp::Not;
            let operand = self.parse_not_expression(ctx)?;
            let span = ctx.merge_span(operand.span().start, operand.span().end);
            Ok(Expression::Unary(UnaryExpression::new(op, operand, span)))
        } else {
            self.parse_comparison_expression(ctx)
        }
    }

    fn parse_comparison_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<Expression, ParseError> {
        let mut left = self.parse_additive_expression(ctx)?;

        if let Some(op) = self.parse_comparison_op(ctx) {
            let right = self.parse_additive_expression(ctx)?;
            let span = ctx.merge_span(left.span().start, right.span().end);
            left = Expression::Binary(BinaryExpression::new(left, op, right, span));
        }

        Ok(left)
    }

    fn parse_comparison_op(&mut self, ctx: &mut ParseContext<'a>) -> Option<BinaryOp> {
        match ctx.current_token().kind {
            TokenKind::Eq => {
                ctx.next_token();
                Some(BinaryOp::Equal)
            }
            TokenKind::Ne => {
                ctx.next_token();
                Some(BinaryOp::NotEqual)
            }
            TokenKind::Lt => {
                ctx.next_token();
                Some(BinaryOp::LessThan)
            }
            TokenKind::Le => {
                ctx.next_token();
                Some(BinaryOp::LessThanOrEqual)
            }
            TokenKind::Gt => {
                ctx.next_token();
                Some(BinaryOp::GreaterThan)
            }
            TokenKind::Ge => {
                ctx.next_token();
                Some(BinaryOp::GreaterThanOrEqual)
            }
            TokenKind::Regex => {
                ctx.next_token();
                Some(BinaryOp::Like)
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

    fn parse_additive_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<Expression, ParseError> {
        let mut left = self.parse_multiplicative_expression(ctx)?;

        while let Some(op) = self.parse_additive_op(ctx) {
            let right = self.parse_multiplicative_expression(ctx)?;
            let span = ctx.merge_span(left.span().start, right.span().end);
            left = Expression::Binary(BinaryExpression::new(left, op, right, span));
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
                Some(BinaryOp::Subtract)
            }
            _ => None,
        }
    }

    fn parse_multiplicative_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<Expression, ParseError> {
        let mut left = self.parse_unary_expression(ctx)?;

        while let Some(op) = self.parse_multiplicative_op(ctx) {
            let right = self.parse_unary_expression(ctx)?;
            let span = ctx.merge_span(left.span().start, right.span().end);
            left = Expression::Binary(BinaryExpression::new(left, op, right, span));
        }

        Ok(left)
    }

    fn parse_multiplicative_op(&mut self, ctx: &mut ParseContext<'a>) -> Option<BinaryOp> {
        match ctx.current_token().kind {
            TokenKind::Star => {
                ctx.next_token();
                Some(BinaryOp::Multiply)
            }
            TokenKind::Div => {
                ctx.next_token();
                Some(BinaryOp::Divide)
            }
            TokenKind::Mod => {
                ctx.next_token();
                Some(BinaryOp::Modulo)
            }
            _ => None,
        }
    }

    fn parse_unary_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<Expression, ParseError> {
        if ctx.match_token(TokenKind::Minus) {
            let op = UnaryOp::Minus;
            let operand = self.parse_unary_expression(ctx)?;
            let span = ctx.merge_span(operand.span().start, operand.span().end);
            Ok(Expression::Unary(UnaryExpression::new(op, operand, span)))
        } else if ctx.match_token(TokenKind::Plus) {
            let op = UnaryOp::Plus;
            let operand = self.parse_unary_expression(ctx)?;
            let span = ctx.merge_span(operand.span().start, operand.span().end);
            Ok(Expression::Unary(UnaryExpression::new(op, operand, span)))
        } else if ctx.match_token(TokenKind::NotOp) {
            let op = UnaryOp::Not;
            let operand = self.parse_unary_expression(ctx)?;
            let span = ctx.merge_span(operand.span().start, operand.span().end);
            Ok(Expression::Unary(UnaryExpression::new(op, operand, span)))
        } else {
            self.parse_exponentiation_expression(ctx)
        }
    }

    fn parse_exponentiation_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<Expression, ParseError> {
        let mut expression = self.parse_postfix_expression(ctx)?;

        if ctx.match_token(TokenKind::Exp) {
            let mut right_operands = Vec::new();

            while ctx.match_token(TokenKind::Exp) {
                right_operands.push(self.parse_unary_expression(ctx)?);
            }

            for operand in right_operands.into_iter().rev() {
                let span = ctx.merge_span(expression.span().start, operand.span().end);
                expression = Expression::Binary(BinaryExpression::new(expression, BinaryOp::Exponent, operand, span));
            }
        }

        Ok(expression)
    }

    fn parse_postfix_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<Expression, ParseError> {
        let mut expression = self.parse_primary_expression(ctx)?;

        loop {
            if ctx.match_token(TokenKind::LBracket) {
                let index = self.parse_expression(ctx)?;
                ctx.expect_token(TokenKind::RBracket)?;
                let span = ctx.merge_span(expression.span().start, ctx.current_position());
                expression = Expression::Subscript(SubscriptExpression::new(expression, index, span));
            } else if ctx.match_token(TokenKind::Dot) {
                let property = ctx.expect_identifier()?;
                let span = ctx.merge_span(expression.span().start, ctx.current_position());
                expression = Expression::PropertyAccess(PropertyAccessExpression::new(expression, property, span));
            } else {
                break;
            }
        }

        Ok(expression)
    }

    fn parse_primary_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<Expression, ParseError> {
        let token = ctx.current_token().clone();
        let start_pos = ctx.current_position();

        match token.kind {
            TokenKind::LParen => {
                ctx.next_token();
                let expression = self.parse_expression(ctx)?;
                ctx.expect_token(TokenKind::RParen)?;
                Ok(expression)
            }
            TokenKind::Identifier(name) => {
                ctx.next_token();
                let span = ctx.merge_span(start_pos, ctx.current_position());
                if ctx.match_token(TokenKind::LParen) {
                    self.parse_function_call(name, span, ctx)
                } else {
                    Ok(Expression::Variable(VariableExpression::new(name, span)))
                }
            }
            TokenKind::IntegerLiteral(n) => {
                ctx.next_token();
                let span = ctx.merge_span(start_pos, ctx.current_position());
                Ok(Expression::Constant(ConstantExpression::new(Value::Int(n), span)))
            }
            TokenKind::FloatLiteral(f) => {
                ctx.next_token();
                let span = ctx.merge_span(start_pos, ctx.current_position());
                Ok(Expression::Constant(ConstantExpression::new(Value::Float(f), span)))
            }
            TokenKind::StringLiteral(s) => {
                ctx.next_token();
                let span = ctx.merge_span(start_pos, ctx.current_position());
                Ok(Expression::Constant(ConstantExpression::new(Value::String(s), span)))
            }
            TokenKind::BooleanLiteral(b) => {
                ctx.next_token();
                let span = ctx.merge_span(start_pos, ctx.current_position());
                Ok(Expression::Constant(ConstantExpression::new(Value::Bool(b), span)))
            }
            TokenKind::Null => {
                ctx.next_token();
                let span = ctx.merge_span(start_pos, ctx.current_position());
                Ok(Expression::Constant(ConstantExpression::new(Value::Null(crate::core::NullType::Null), span)))
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
                Ok(Expression::List(ListExpression::new(elements, span)))
            }
            TokenKind::LBracket => {
                ctx.next_token();
                let elements = self.parse_expression_list(ctx)?;
                ctx.expect_token(TokenKind::RBracket)?;
                let span = ctx.merge_span(start_pos, ctx.current_position());
                Ok(Expression::List(ListExpression::new(elements, span)))
            }
            TokenKind::Map => {
                ctx.next_token();
                ctx.expect_token(TokenKind::LBrace)?;
                let properties = self.parse_property_list(ctx)?;
                ctx.expect_token(TokenKind::RBrace)?;
                let span = ctx.merge_span(start_pos, ctx.current_position());
                Ok(Expression::Map(MapExpression::new(properties, span)))
            }
            TokenKind::LBrace => {
                ctx.next_token();
                let properties = self.parse_property_list(ctx)?;
                ctx.expect_token(TokenKind::RBrace)?;
                let span = ctx.merge_span(start_pos, ctx.current_position());
                Ok(Expression::Map(MapExpression::new(properties, span)))
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

    fn parse_function_call(&mut self, name: String, span: Span, ctx: &mut ParseContext<'a>) -> Result<Expression, ParseError> {
        let args = if ctx.match_token(TokenKind::RParen) {
            Vec::new()
        } else {
            let args = self.parse_expression_list(ctx)?;
            ctx.expect_token(TokenKind::RParen)?;
            args
        };
        Ok(Expression::FunctionCall(FunctionCallExpression::new(name, args, false, span)))
    }

    fn parse_expression_list(&mut self, ctx: &mut ParseContext<'a>) -> Result<Vec<Expression>, ParseError> {
        let mut expressions = Vec::new();
        expressions.push(self.parse_expression(ctx)?);
        while ctx.match_token(TokenKind::Comma) {
            expressions.push(self.parse_expression(ctx)?);
        }
        Ok(expressions)
    }

    fn parse_property_list(&mut self, ctx: &mut ParseContext<'a>) -> Result<Vec<(String, Expression)>, ParseError> {
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
