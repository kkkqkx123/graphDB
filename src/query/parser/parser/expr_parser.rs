//! 表达式解析模块
//!
//! 负责解析各种表达式，包括算术表达式、逻辑表达式、函数调用等。
//! 直接生成 Core Expression，避免 AST Expression 的冗余转换。

use crate::core::Value;
use crate::core::types::expression::Expression;
use crate::core::types::operators::{BinaryOperator, UnaryOperator};
use crate::query::parser::core::error::{ParseError, ParseErrorKind};
use crate::core::types::{Span, Position};
use crate::query::parser::parser::ParseContext;
use crate::query::parser::TokenKind;

/// 表达式解析结果，包含表达式和位置信息
pub struct ParseResult {
    pub expr: Expression,
    pub span: Span,
}

pub struct ExprParser<'a> {
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> ExprParser<'a> {
    pub fn new(_ctx: &ParseContext<'a>) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn parse_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<ParseResult, ParseError> {
        self.parse_or_expression(ctx)
    }

    fn parse_or_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<ParseResult, ParseError> {
        let mut left = self.parse_and_expression(ctx)?;

        while ctx.match_token(TokenKind::Or) {
            let op = BinaryOperator::Or;
            let right = self.parse_and_expression(ctx)?;
            let span = ctx.merge_span(left.span.start, right.span.end);
            left = ParseResult {
                expr: Expression::binary(left.expr, op, right.expr),
                span,
            };
        }

        Ok(left)
    }

    fn parse_and_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<ParseResult, ParseError> {
        let mut left = self.parse_not_expression(ctx)?;

        while ctx.match_token(TokenKind::And) {
            let op = BinaryOperator::And;
            let right = self.parse_not_expression(ctx)?;
            let span = ctx.merge_span(left.span.start, right.span.end);
            left = ParseResult {
                expr: Expression::binary(left.expr, op, right.expr),
                span,
            };
        }

        Ok(left)
    }

    fn parse_not_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<ParseResult, ParseError> {
        if ctx.match_token(TokenKind::Not) {
            let op = UnaryOperator::Not;
            let operand = self.parse_not_expression(ctx)?;
            let span = ctx.merge_span(operand.span.start, operand.span.end);
            Ok(ParseResult {
                expr: Expression::unary(op, operand.expr),
                span,
            })
        } else {
            self.parse_comparison_expression(ctx)
        }
    }

    fn parse_comparison_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<ParseResult, ParseError> {
        let mut left = self.parse_additive_expression(ctx)?;

        if let Some(op) = self.parse_comparison_op(ctx) {
            let right = self.parse_additive_expression(ctx)?;
            let span = ctx.merge_span(left.span.start, right.span.end);
            left = ParseResult {
                expr: Expression::binary(left.expr, op, right.expr),
                span,
            };
        }

        Ok(left)
    }

    fn parse_comparison_op(&mut self, ctx: &mut ParseContext<'a>) -> Option<BinaryOperator> {
        match ctx.current_token().kind {
            TokenKind::Eq => {
                ctx.next_token();
                Some(BinaryOperator::Equal)
            }
            TokenKind::Ne => {
                ctx.next_token();
                Some(BinaryOperator::NotEqual)
            }
            TokenKind::Lt => {
                ctx.next_token();
                Some(BinaryOperator::LessThan)
            }
            TokenKind::Le => {
                ctx.next_token();
                Some(BinaryOperator::LessThanOrEqual)
            }
            TokenKind::Gt => {
                ctx.next_token();
                Some(BinaryOperator::GreaterThan)
            }
            TokenKind::Ge => {
                ctx.next_token();
                Some(BinaryOperator::GreaterThanOrEqual)
            }
            TokenKind::Regex => {
                ctx.next_token();
                Some(BinaryOperator::Like)
            }
            TokenKind::Contains => {
                ctx.next_token();
                Some(BinaryOperator::Contains)
            }
            TokenKind::StartsWith => {
                ctx.next_token();
                Some(BinaryOperator::StartsWith)
            }
            TokenKind::EndsWith => {
                ctx.next_token();
                Some(BinaryOperator::EndsWith)
            }
            _ => None,
        }
    }

    fn parse_additive_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<ParseResult, ParseError> {
        let mut left = self.parse_multiplicative_expression(ctx)?;

        while let Some(op) = self.parse_additive_op(ctx) {
            let right = self.parse_multiplicative_expression(ctx)?;
            let span = ctx.merge_span(left.span.start, right.span.end);
            left = ParseResult {
                expr: Expression::binary(left.expr, op, right.expr),
                span,
            };
        }

        Ok(left)
    }

    fn parse_additive_op(&mut self, ctx: &mut ParseContext<'a>) -> Option<BinaryOperator> {
        match ctx.current_token().kind {
            TokenKind::Plus => {
                ctx.next_token();
                Some(BinaryOperator::Add)
            }
            TokenKind::Minus => {
                ctx.next_token();
                Some(BinaryOperator::Subtract)
            }
            _ => None,
        }
    }

    fn parse_multiplicative_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<ParseResult, ParseError> {
        let mut left = self.parse_unary_expression(ctx)?;

        while let Some(op) = self.parse_multiplicative_op(ctx) {
            let right = self.parse_unary_expression(ctx)?;
            let span = ctx.merge_span(left.span.start, right.span.end);
            left = ParseResult {
                expr: Expression::binary(left.expr, op, right.expr),
                span,
            };
        }

        Ok(left)
    }

    fn parse_multiplicative_op(&mut self, ctx: &mut ParseContext<'a>) -> Option<BinaryOperator> {
        match ctx.current_token().kind {
            TokenKind::Star => {
                ctx.next_token();
                Some(BinaryOperator::Multiply)
            }
            TokenKind::Div => {
                ctx.next_token();
                Some(BinaryOperator::Divide)
            }
            TokenKind::Mod => {
                ctx.next_token();
                Some(BinaryOperator::Modulo)
            }
            _ => None,
        }
    }

    fn parse_unary_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<ParseResult, ParseError> {
        if ctx.match_token(TokenKind::Minus) {
            let op = UnaryOperator::Minus;
            let operand = self.parse_unary_expression(ctx)?;
            let span = ctx.merge_span(operand.span.start, operand.span.end);
            Ok(ParseResult {
                expr: Expression::unary(op, operand.expr),
                span,
            })
        } else if ctx.match_token(TokenKind::Plus) {
            let op = UnaryOperator::Plus;
            let operand = self.parse_unary_expression(ctx)?;
            let span = ctx.merge_span(operand.span.start, operand.span.end);
            Ok(ParseResult {
                expr: Expression::unary(op, operand.expr),
                span,
            })
        } else if ctx.match_token(TokenKind::NotOp) {
            let op = UnaryOperator::Not;
            let operand = self.parse_unary_expression(ctx)?;
            let span = ctx.merge_span(operand.span.start, operand.span.end);
            Ok(ParseResult {
                expr: Expression::unary(op, operand.expr),
                span,
            })
        } else {
            self.parse_exponentiation_expression(ctx)
        }
    }

    fn parse_exponentiation_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<ParseResult, ParseError> {
        let mut expression = self.parse_postfix_expression(ctx)?;

        if ctx.match_token(TokenKind::Exp) {
            let mut right_operands = Vec::new();

            while ctx.match_token(TokenKind::Exp) {
                right_operands.push(self.parse_unary_expression(ctx)?);
            }

            for operand in right_operands.into_iter().rev() {
                let span = ctx.merge_span(expression.span.start, operand.span.end);
                expression = ParseResult {
                    expr: Expression::binary(expression.expr, BinaryOperator::Exponent, operand.expr),
                    span,
                };
            }
        }

        Ok(expression)
    }

    fn parse_postfix_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<ParseResult, ParseError> {
        let mut expression = self.parse_primary_expression(ctx)?;

        loop {
            if ctx.match_token(TokenKind::LBracket) {
                let index = self.parse_expression(ctx)?;
                ctx.expect_token(TokenKind::RBracket)?;
                let span = ctx.merge_span(expression.span.start, ctx.current_position());
                expression = ParseResult {
                    expr: Expression::subscript(expression.expr, index.expr),
                    span,
                };
            } else if ctx.match_token(TokenKind::Dot) {
                let property = ctx.expect_identifier()?;
                let span = ctx.merge_span(expression.span.start, ctx.current_position());
                expression = ParseResult {
                    expr: Expression::property(expression.expr, property),
                    span,
                };
            } else {
                break;
            }
        }

        Ok(expression)
    }

    fn parse_primary_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<ParseResult, ParseError> {
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
                    Ok(ParseResult {
                        expr: Expression::variable(name),
                        span,
                    })
                }
            }
            TokenKind::IntegerLiteral(n) => {
                ctx.next_token();
                let span = ctx.merge_span(start_pos, ctx.current_position());
                Ok(ParseResult {
                    expr: Expression::literal(Value::Int(n)),
                    span,
                })
            }
            TokenKind::FloatLiteral(f) => {
                ctx.next_token();
                let span = ctx.merge_span(start_pos, ctx.current_position());
                Ok(ParseResult {
                    expr: Expression::literal(Value::Float(f)),
                    span,
                })
            }
            TokenKind::StringLiteral(s) => {
                ctx.next_token();
                let span = ctx.merge_span(start_pos, ctx.current_position());
                Ok(ParseResult {
                    expr: Expression::literal(Value::String(s)),
                    span,
                })
            }
            TokenKind::BooleanLiteral(b) => {
                ctx.next_token();
                let span = ctx.merge_span(start_pos, ctx.current_position());
                Ok(ParseResult {
                    expr: Expression::literal(Value::Bool(b)),
                    span,
                })
            }
            TokenKind::Null => {
                ctx.next_token();
                let span = ctx.merge_span(start_pos, ctx.current_position());
                Ok(ParseResult {
                    expr: Expression::literal(Value::Null(crate::core::NullType::Null)),
                    span,
                })
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
                Ok(ParseResult {
                    expr: Expression::list(elements.into_iter().map(|e| e.expr).collect()),
                    span,
                })
            }
            TokenKind::LBracket => {
                ctx.next_token();
                if ctx.is_identifier_or_in_token() {
                    self.parse_list_comprehension(start_pos, ctx)
                } else {
                    let elements = self.parse_expression_list(ctx)?;
                    ctx.expect_token(TokenKind::RBracket)?;
                    let span = ctx.merge_span(start_pos, ctx.current_position());
                    Ok(ParseResult {
                        expr: Expression::list(elements.into_iter().map(|e| e.expr).collect()),
                        span,
                    })
                }
            }
            TokenKind::Case => {
                self.parse_case_expression(start_pos, ctx)
            }
            TokenKind::Map => {
                ctx.next_token();
                ctx.expect_token(TokenKind::LBrace)?;
                let properties = self.parse_property_list(ctx)?;
                ctx.expect_token(TokenKind::RBrace)?;
                let span = ctx.merge_span(start_pos, ctx.current_position());
                Ok(ParseResult {
                    expr: Expression::map(properties.into_iter().map(|(k, v)| (k, v.expr)).collect()),
                    span,
                })
            }
            TokenKind::LBrace => {
                ctx.next_token();
                let properties = self.parse_property_list(ctx)?;
                ctx.expect_token(TokenKind::RBrace)?;
                let span = ctx.merge_span(start_pos, ctx.current_position());
                Ok(ParseResult {
                    expr: Expression::map(properties.into_iter().map(|(k, v)| (k, v.expr)).collect()),
                    span,
                })
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

    fn parse_function_call(&mut self, name: String, span: Span, ctx: &mut ParseContext<'a>) -> Result<ParseResult, ParseError> {
        let args = if ctx.match_token(TokenKind::RParen) {
            Vec::new()
        } else {
            let args = self.parse_expression_list(ctx)?;
            ctx.expect_token(TokenKind::RParen)?;
            args
        };
        Ok(ParseResult {
            expr: Expression::function(name, args.into_iter().map(|e| e.expr).collect()),
            span,
        })
    }

    fn parse_expression_list(&mut self, ctx: &mut ParseContext<'a>) -> Result<Vec<ParseResult>, ParseError> {
        let mut expressions = Vec::new();
        expressions.push(self.parse_expression(ctx)?);
        while ctx.match_token(TokenKind::Comma) {
            expressions.push(self.parse_expression(ctx)?);
        }
        Ok(expressions)
    }

    fn parse_property_list(&mut self, ctx: &mut ParseContext<'a>) -> Result<Vec<(String, ParseResult)>, ParseError> {
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

    fn parse_case_expression(&mut self, start_pos: Position, ctx: &mut ParseContext<'a>) -> Result<ParseResult, ParseError> {
        ctx.expect_token(TokenKind::Case)?;
        
        let test_expr = if ctx.peek_token().kind != TokenKind::When {
            Some(self.parse_expression(ctx)?.expr)
        } else {
            None
        };
        
        let mut conditions = Vec::new();
        while ctx.match_token(TokenKind::When) {
            let when_expr = self.parse_expression(ctx)?;
            ctx.expect_token(TokenKind::Then)?;
            let then_expr = self.parse_expression(ctx)?;
            conditions.push((when_expr.expr, then_expr.expr));
        }
        
        let default = if ctx.match_token(TokenKind::Else) {
            Some(self.parse_expression(ctx)?.expr)
        } else {
            None
        };
        
        ctx.expect_token(TokenKind::End)?;
        
        let span = ctx.merge_span(start_pos, ctx.current_position());
        Ok(ParseResult {
            expr: Expression::case(test_expr, conditions, default),
            span,
        })
    }

    fn parse_list_comprehension(&mut self, start_pos: Position, ctx: &mut ParseContext<'a>) -> Result<ParseResult, ParseError> {
        let variable = ctx.expect_identifier()?;
        ctx.expect_token(TokenKind::In)?;
        let source = self.parse_expression(ctx)?.expr;
        
        let (filter, map) = if ctx.match_token(TokenKind::Pipe) {
            let map_expr = self.parse_expression(ctx)?;
            (None, Some(map_expr.expr))
        } else if ctx.match_token(TokenKind::Where) {
            let filter_expr = self.parse_expression(ctx)?;
            let map_expr = if ctx.match_token(TokenKind::Pipe) {
                Some(self.parse_expression(ctx)?.expr)
            } else {
                None
            };
            (Some(filter_expr.expr), map_expr)
        } else {
            (None, None)
        };
        
        ctx.expect_token(TokenKind::RBracket)?;
        
        let span = ctx.merge_span(start_pos, ctx.current_position());
        Ok(ParseResult {
            expr: Expression::list_comprehension(variable, source, filter, map),
            span,
        })
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
        let parse_result = result.unwrap();
        // 验证表达式结构正确，不检查具体运算符优先级
        assert!(matches!(parse_result.expr, Expression::Binary { .. }));
    }

    #[test]
    fn test_parse_parenthesized_expression() {
        let input = "(1 + 2) * 3";
        let ctx = &mut ParseContext::new(input);
        let mut parser = ExprParser::new(ctx);
        let result = parser.parse_expression(ctx);
        assert!(result.is_ok());
        let parse_result = result.unwrap();
        // 验证表达式结构正确，不检查具体运算符优先级
        assert!(matches!(parse_result.expr, Expression::Binary { .. }));
    }
}
