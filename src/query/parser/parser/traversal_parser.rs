//! 图遍历语句解析模块
//!
//! 负责解析图遍历相关语句，包括 MATCH、GO、FIND PATH、GET SUBGRAPH 等。

use crate::core::types::graph_schema::EdgeDirection;
use crate::core::types::expression::Expression as CoreExpression;
use crate::query::parser::ast::stmt::*;
use crate::query::parser::ast::pattern::{EdgePattern, NodePattern, PathElement, PathPattern, Pattern, VariablePattern};
use crate::query::parser::core::error::{ParseError, ParseErrorKind};
use crate::query::parser::parser::clause_parser::ClauseParser;
use crate::query::parser::parser::ExprParser;
use crate::query::parser::parser::parse_context::ParseContext;
use crate::query::parser::TokenKind;

/// 图遍历解析器
pub struct TraversalParser;

impl TraversalParser {
    pub fn new() -> Self {
        Self
    }

    /// 解析 MATCH 语句
    pub fn parse_match_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();

        // 检查是否是 OPTIONAL MATCH
        let optional = ctx.match_token(TokenKind::Optional);

        ctx.expect_token(TokenKind::Match)?;

        let mut patterns = Vec::new();
        patterns.push(self.parse_pattern(ctx)?);

        let where_clause = if ctx.match_token(TokenKind::Where) {
            Some(self.parse_expression(ctx)?)
        } else {
            None
        };

        let return_clause = if ctx.match_token(TokenKind::Return) {
            Some(ClauseParser::new().parse_return_clause(ctx)?)
        } else {
            None
        };

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Stmt::Match(MatchStmt {
            span,
            patterns,
            where_clause,
            return_clause,
            order_by: None,
            limit: None,
            skip: None,
            optional,
        }))
    }

    /// 解析 GO 语句
    pub fn parse_go_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Go)?;

        let steps = self.parse_steps(ctx)?;
        
        // 消费可选的 STEP/STEP 关键字
        ctx.match_token(TokenKind::Step);

        ctx.expect_token(TokenKind::From)?;
        let from_span = ctx.current_span();
        let vertices = self.parse_expression_list(ctx)?;
        let from_clause = FromClause {
            span: from_span,
            vertices,
        };

        let over = if ctx.match_token(TokenKind::Over) {
            Some(ClauseParser::new().parse_over_clause(ctx)?)
        } else {
            None
        };

        let where_clause = if ctx.match_token(TokenKind::Where) {
            Some(self.parse_expression(ctx)?)
        } else {
            None
        };

        let yield_clause = if ctx.match_token(TokenKind::Yield) {
            Some(ClauseParser::new().parse_yield_clause(ctx)?)
        } else {
            None
        };

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Stmt::Go(GoStmt {
            span,
            steps,
            from: from_clause,
            over,
            where_clause,
            yield_clause,
        }))
    }

    /// 解析 FIND PATH 语句
    pub fn parse_find_path_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Find)?;

        // 解析路径类型: SHORTEST, ALL
        let shortest = if ctx.match_token(TokenKind::Shortest) {
            true
        } else if ctx.match_token(TokenKind::All) {
            false
        } else {
            true
        };

        ctx.expect_token(TokenKind::Path)?;

        // 可选的 WITH LOOP / WITH CYCLE
        let mut with_loop = false;
        let mut with_cycle = false;
        while ctx.match_token(TokenKind::With) {
            if ctx.match_token(TokenKind::Loop) {
                with_loop = true;
            } else if ctx.match_token(TokenKind::Cycle) {
                with_cycle = true;
            }
        }

        ctx.expect_token(TokenKind::From)?;
        let from_span = ctx.current_span();
        let from_vertices = self.parse_expression_list(ctx)?;
        let from_clause = FromClause {
            span: from_span,
            vertices: from_vertices,
        };

        ctx.expect_token(TokenKind::To)?;
        let to_vertex = self.parse_expression(ctx)?;

        ctx.expect_token(TokenKind::Over)?;
        let over = ClauseParser::new().parse_over_clause(ctx)?;

        // 可选的 UPTO N STEPS
        let mut max_steps = None;
        if ctx.match_token(TokenKind::Upto) {
            max_steps = Some(ctx.expect_integer_literal()? as usize);
            ctx.expect_token(TokenKind::Step)?;
        }

        // 可选的 WEIGHT 子句
        let weight_expression = if ctx.match_token(TokenKind::Weight) {
            Some(ctx.expect_identifier()?)
        } else {
            None
        };

        // 可选的 WHERE 子句
        let where_clause = if ctx.match_token(TokenKind::Where) {
            Some(self.parse_expression(ctx)?)
        } else {
            None
        };

        // 可选的 YIELD 子句
        let yield_clause = if ctx.match_token(TokenKind::Yield) {
            Some(ClauseParser::new().parse_yield_clause(ctx)?)
        } else {
            None
        };

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Stmt::FindPath(FindPathStmt {
            span,
            from: from_clause,
            to: to_vertex,
            over: Some(over),
            where_clause,
            shortest,
            max_steps,
            limit: None,
            offset: None,
            yield_clause,
            weight_expression,
            heuristic_expression: None,
            with_loop,
            with_cycle,
        }))
    }

    /// 解析 GET SUBGRAPH 语句
    pub fn parse_subgraph_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Get)?;

        // 可选的 WITH EDGE 子句
        let _with_edge = ctx.match_token(TokenKind::With) && ctx.match_token(TokenKind::Edge);

        ctx.expect_token(TokenKind::Subgraph)?;

        // 解析步数
        let steps = if ctx.match_token(TokenKind::Step) {
            self.parse_steps(ctx)?
        } else {
            Steps::Fixed(1)
        };

        ctx.expect_token(TokenKind::From)?;
        let from_span = ctx.current_span();
        let vertices = self.parse_expression_list(ctx)?;
        let from_clause = FromClause {
            span: from_span,
            vertices,
        };

        let over = if ctx.match_token(TokenKind::Over) {
            Some(ClauseParser::new().parse_over_clause(ctx)?)
        } else {
            None
        };

        let where_clause = if ctx.match_token(TokenKind::Where) {
            Some(self.parse_expression(ctx)?)
        } else {
            None
        };

        let yield_clause = if ctx.match_token(TokenKind::Yield) {
            Some(ClauseParser::new().parse_yield_clause(ctx)?)
        } else {
            None
        };

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Stmt::Subgraph(SubgraphStmt {
            span,
            steps,
            from: from_clause,
            over,
            where_clause,
            yield_clause,
        }))
    }

    /// 解析模式
    pub fn parse_pattern(&mut self, ctx: &mut ParseContext) -> Result<Pattern, ParseError> {
        let start_span = ctx.current_span();

        // 检查是否是节点模式（以 ( 开头）
        if ctx.match_token(TokenKind::LParen) {
            let node = self.parse_node_pattern(ctx, start_span)?;
            
            // 检查是否有链式边模式
            if ctx.check_token(TokenKind::LeftArrow) || ctx.check_token(TokenKind::RightArrow) || ctx.check_token(TokenKind::Minus) {
                return self.parse_path_pattern(ctx, node);
            }
            
            return Ok(Pattern::Node(node));
        }

        // 检查是否是变量模式
        if let TokenKind::Identifier(ref name) = ctx.current_token().kind.clone() {
            let name = name.clone();
            let span = ctx.current_span();
            ctx.next_token();
            return Ok(Pattern::Variable(VariablePattern {
                span,
                name,
            }));
        }

        Err(ParseError::new(
            ParseErrorKind::SyntaxError,
            "Expected pattern (node or path)".to_string(),
            ctx.current_position(),
        ))
    }

    /// 解析节点模式
    fn parse_node_pattern(&mut self, ctx: &mut ParseContext, start_span: crate::query::parser::ast::types::Span) -> Result<NodePattern, ParseError> {
        let mut variable = None;
        let mut labels = Vec::new();
        let mut properties = None;

        // 解析变量名（可选）
        if let TokenKind::Identifier(ref name) = ctx.current_token().kind.clone() {
            let name = name.clone();
            ctx.next_token();
            
            // 检查后面是否是标签（:label）
            if ctx.check_token(TokenKind::Colon) {
                variable = Some(name);
            } else {
                // 没有冒号，这个标识符就是标签名
                labels.push(name);
            }
        }

        // 解析标签
        if ctx.match_token(TokenKind::Colon) {
            // 解析标签列表（支持多标签，如 :Person:Actor）
            loop {
                let label = ctx.expect_identifier()?;
                labels.push(label);
                if !ctx.check_token(TokenKind::Colon) {
                    break;
                }
                ctx.next_token(); // 消费下一个冒号
            }
        }

        // 解析属性（可选）
        if ctx.match_token(TokenKind::LBrace) {
            properties = Some(self.parse_properties_expr(ctx)?);
            ctx.expect_token(TokenKind::RBrace)?;
        }

        // 期望右括号
        ctx.expect_token(TokenKind::RParen)?;

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(NodePattern {
            span,
            variable,
            labels,
            properties,
            predicates: Vec::new(),
        })
    }

    /// 解析路径模式
    fn parse_path_pattern(&mut self, ctx: &mut ParseContext, start_node: NodePattern) -> Result<Pattern, ParseError> {
        let start_span = start_node.span;
        let mut elements = vec![PathElement::Node(start_node)];

        // 解析边和节点的链式结构
        while ctx.check_token(TokenKind::LeftArrow) || ctx.check_token(TokenKind::RightArrow) || ctx.check_token(TokenKind::Minus) {
            let edge = self.parse_edge_pattern(ctx)?;
            elements.push(PathElement::Edge(edge));

            // 期望后面跟着一个节点
            if ctx.match_token(TokenKind::LParen) {
                let node_span = ctx.current_span();
                let node = self.parse_node_pattern(ctx, node_span)?;
                elements.push(PathElement::Node(node));
            } else {
                break;
            }
        }

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Pattern::Path(PathPattern {
            span,
            elements,
        }))
    }

    /// 解析边模式
    fn parse_edge_pattern(&mut self, ctx: &mut ParseContext) -> Result<EdgePattern, ParseError> {
        let start_span = ctx.current_span();
        let mut direction = EdgeDirection::Out;

        // 解析方向
        if ctx.match_token(TokenKind::LeftArrow) {
            direction = EdgeDirection::In;
        }

        // 期望 -[ 或 -
        ctx.expect_token(TokenKind::Minus)?;

        let mut variable = None;
        let mut edge_types = Vec::new();
        let mut properties = None;
        let mut range = None;

        // 解析详细的边模式 [variable:Type|Type {props}]
        if ctx.match_token(TokenKind::LBracket) {
            // 解析变量名（可选）
            if let TokenKind::Identifier(ref name) = ctx.current_token().kind.clone() {
                let name = name.clone();
                ctx.next_token();
                
                if ctx.check_token(TokenKind::Colon) {
                    variable = Some(name);
                } else {
                    edge_types.push(name);
                }
            }

            // 解析边类型
            if ctx.match_token(TokenKind::Colon) {
                loop {
                    let edge_type = ctx.expect_identifier()?;
                    edge_types.push(edge_type);
                    if !ctx.match_token(TokenKind::Pipe) {
                        break;
                    }
                }
            }

            // 解析属性（可选）
            if ctx.match_token(TokenKind::LBrace) {
                properties = Some(self.parse_properties_expr(ctx)?);
                ctx.expect_token(TokenKind::RBrace)?;
            }

            // 解析范围（可选）如 *[1..3]
            if ctx.match_token(TokenKind::Star) {
                // 范围解析暂时简化处理
                range = Some(crate::query::parser::ast::pattern::EdgeRange { min: None, max: None });
            }

            ctx.expect_token(TokenKind::RBracket)?;
        }

        // 期望 -
        ctx.expect_token(TokenKind::Minus)?;

        // 解析右侧箭头
        if ctx.match_token(TokenKind::RightArrow) {
            if direction == EdgeDirection::In {
                direction = EdgeDirection::Both;
            } else {
                direction = EdgeDirection::Out;
            }
        }

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(EdgePattern {
            span,
            variable,
            edge_types,
            properties,
            predicates: Vec::new(),
            direction,
            range,
        })
    }

    /// 解析步数
    fn parse_steps(&mut self, ctx: &mut ParseContext) -> Result<Steps, ParseError> {
        // 尝试解析数字或范围
        let token = ctx.current_token();
        match token.kind {
            TokenKind::IntegerLiteral(n) => {
                ctx.next_token();
                Ok(Steps::Fixed(n as usize))
            }
            _ => {
                // 默认 1 步
                Ok(Steps::Fixed(1))
            }
        }
    }

    /// 解析表达式列表
    fn parse_expression_list(&mut self, ctx: &mut ParseContext) -> Result<Vec<CoreExpression>, ParseError> {
        let mut expressions = Vec::new();
        
        loop {
            expressions.push(self.parse_expression(ctx)?);
            if !ctx.match_token(TokenKind::Comma) {
                break;
            }
        }
        
        Ok(expressions)
    }

    /// 解析属性表达式
    fn parse_properties_expr(&mut self, ctx: &mut ParseContext) -> Result<CoreExpression, ParseError> {
        // 简化实现：解析为表达式
        let mut expr_parser = ExprParser::new(ctx);
        let result = expr_parser.parse_expression(ctx)?;
        Ok(result.expr)
    }

    /// 解析表达式
    fn parse_expression(&mut self, ctx: &mut ParseContext) -> Result<CoreExpression, ParseError> {
        let mut expr_parser = ExprParser::new(ctx);
        let result = expr_parser.parse_expression(ctx)?;
        Ok(result.expr)
    }
}

impl Default for TraversalParser {
    fn default() -> Self {
        Self::new()
    }
}
