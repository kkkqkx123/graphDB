//! 子句解析模块
//!
//! 负责解析各种共享子句，包括 RETURN、YIELD、SET、OVER、WHERE 等。

use crate::core::types::graph_schema::EdgeDirection;
use crate::core::types::expression::Expression as CoreExpression;
use crate::query::parser::ast::stmt::*;
use crate::query::parser::ast::types::{OrderDirection, LimitClause, SkipClause};
use crate::query::parser::core::error::{ParseError, ParseErrorKind};
use crate::query::parser::parser::ExprParser;
use crate::query::parser::parser::parse_context::ParseContext;
use crate::query::parser::TokenKind;

/// 子句解析器
pub struct ClauseParser;

impl ClauseParser {
    pub fn new() -> Self {
        Self
    }

    /// 解析 RETURN 子句
    pub fn parse_return_clause(&mut self, ctx: &mut ParseContext) -> Result<ReturnClause, ParseError> {
        let span = ctx.current_span();
        
        let distinct = ctx.match_token(TokenKind::Distinct);
        
        let mut items = Vec::new();
        
        // 检查是否是 *
        if ctx.match_token(TokenKind::Star) {
            items.push(ReturnItem::All);
        } else {
            loop {
                let expr = self.parse_expression(ctx)?;
                let alias = if ctx.match_token(TokenKind::As) {
                    Some(ctx.expect_identifier()?)
                } else {
                    None
                };
                items.push(ReturnItem::Expression {
                    expression: expr,
                    alias,
                });
                if !ctx.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }
        
        // 解析 ORDER BY
        let _order_by = if ctx.match_token(TokenKind::Order) {
            ctx.expect_token(TokenKind::By)?;
            Some(self.parse_order_by_clause(ctx)?)
        } else {
            None
        };
        
        // 解析 LIMIT
        let limit = if ctx.match_token(TokenKind::Limit) {
            let count = ctx.expect_integer_literal()? as usize;
            Some(LimitClause {
                span: ctx.current_span(),
                count,
            })
        } else {
            None
        };
        
        // 解析 SKIP
        let skip = if ctx.match_token(TokenKind::Skip) {
            let count = ctx.expect_integer_literal()? as usize;
            Some(SkipClause {
                span: ctx.current_span(),
                count,
            })
        } else {
            None
        };
        
        Ok(ReturnClause {
            span,
            items,
            distinct,
            limit,
            skip,
            sample: None,
        })
    }

    /// 解析 YIELD 子句
    pub fn parse_yield_clause(&mut self, ctx: &mut ParseContext) -> Result<YieldClause, ParseError> {
        let span = ctx.current_span();
        
        // 消费 YIELD token
        ctx.expect_token(TokenKind::Yield)?;
        
        let mut items = Vec::new();
        
        // 检查是否是 *
        if ctx.match_token(TokenKind::Star) {
            // YIELD * 表示返回所有列
        } else {
            loop {
                let expr = self.parse_expression(ctx)?;
                let alias = if ctx.match_token(TokenKind::As) {
                    Some(ctx.expect_identifier()?)
                } else {
                    None
                };
                items.push(YieldItem {
                    expression: expr,
                    alias,
                });
                if !ctx.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }
        
        // 解析 LIMIT
        let limit = if ctx.match_token(TokenKind::Limit) {
            let count = ctx.expect_integer_literal()? as usize;
            Some(LimitClause {
                span: ctx.current_span(),
                count,
            })
        } else {
            None
        };
        
        // 解析 SKIP
        let skip = if ctx.match_token(TokenKind::Skip) {
            let count = ctx.expect_integer_literal()? as usize;
            Some(SkipClause {
                span: ctx.current_span(),
                count,
            })
        } else {
            None
        };
        
        Ok(YieldClause {
            span,
            items,
            limit,
            skip,
            sample: None,
        })
    }

    /// 解析 SET 子句
    pub fn parse_set_clause(&mut self, ctx: &mut ParseContext) -> Result<SetClause, ParseError> {
        let span = ctx.current_span();
        let assignments = self.parse_set_assignments(ctx)?;
        Ok(SetClause { span, assignments })
    }

    /// 解析 SET 赋值列表
    pub fn parse_set_assignments(&mut self, ctx: &mut ParseContext) -> Result<Vec<Assignment>, ParseError> {
        let mut assignments = Vec::new();
        loop {
            let property_expr = self.parse_expression(ctx)?;
            ctx.expect_token(TokenKind::Assign)?;
            let value = self.parse_expression(ctx)?;
            
            let property = match &property_expr {
                CoreExpression::Property { property, .. } => property.clone(),
                CoreExpression::Variable(name) => name.clone(),
                _ => {
                    return Err(ParseError::new(
                        ParseErrorKind::SyntaxError,
                        "SET assignment requires a property path (e.g., p.age)".to_string(),
                        ctx.current_position(),
                    ));
                }
            };
            
            assignments.push(Assignment { property, value });
            if !ctx.match_token(TokenKind::Comma) {
                break;
            }
        }
        Ok(assignments)
    }

    /// 解析 OVER 子句
    pub fn parse_over_clause(&mut self, ctx: &mut ParseContext) -> Result<OverClause, ParseError> {
        let span = ctx.current_span();
        
        let edge_types = self.parse_edge_types(ctx)?;
        
        // 解析方向（可选）
        let direction = if ctx.match_token(TokenKind::In) {
            EdgeDirection::In
        } else if ctx.match_token(TokenKind::Bidirect) {
            EdgeDirection::Both
        } else {
            EdgeDirection::Out
        };
        
        Ok(OverClause { span, edge_types, direction })
    }

    /// 解析边类型列表
    fn parse_edge_types(&mut self, ctx: &mut ParseContext) -> Result<Vec<String>, ParseError> {
        let mut types = Vec::new();
        types.push(ctx.expect_identifier()?);
        while ctx.match_token(TokenKind::Comma) {
            types.push(ctx.expect_identifier()?);
        }
        Ok(types)
    }

    /// 解析 ORDER BY 子句
    fn parse_order_by_clause(&mut self, ctx: &mut ParseContext) -> Result<OrderByClause, ParseError> {
        let span = ctx.current_span();
        let mut items = Vec::new();
        
        loop {
            let expr = self.parse_expression(ctx)?;
            let direction = if ctx.match_token(TokenKind::Asc) {
                OrderDirection::Asc
            } else if ctx.match_token(TokenKind::Desc) {
                OrderDirection::Desc
            } else {
                OrderDirection::Asc
            };
            items.push(OrderByItem { expression: expr, direction });
            if !ctx.match_token(TokenKind::Comma) {
                break;
            }
        }
        
        Ok(OrderByClause { span, items })
    }

    /// 解析表达式
    fn parse_expression(&mut self, ctx: &mut ParseContext) -> Result<CoreExpression, ParseError> {
        let mut expr_parser = ExprParser::new(ctx);
        let result = expr_parser.parse_expression(ctx)?;
        Ok(result.expr)
    }
}

impl Default for ClauseParser {
    fn default() -> Self {
        Self::new()
    }
}
