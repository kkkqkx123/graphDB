//! 数据修改语句解析模块
//!
//! 负责解析数据修改相关语句，包括 INSERT、DELETE、UPDATE、MERGE 等。

use crate::core::types::expression::Expression as CoreExpression;
use crate::query::parser::ast::stmt::*;
use crate::query::parser::core::error::{ParseError, ParseErrorKind};
use crate::query::parser::parser::clause_parser::ClauseParser;
use crate::query::parser::parser::ExprParser;
use crate::query::parser::parser::parse_context::ParseContext;
use crate::query::parser::parser::traversal_parser::TraversalParser;
use crate::query::parser::TokenKind;

/// 数据修改解析器
pub struct DmlParser;

impl DmlParser {
    pub fn new() -> Self {
        Self
    }

    /// 解析 INSERT 语句
    pub fn parse_insert_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Insert)?;

        let target = if ctx.match_token(TokenKind::Vertex) {
            let tag_name = ctx.expect_identifier()?;
            
            // 解析属性名列表（可选）
            let mut prop_names = Vec::new();
            if ctx.match_token(TokenKind::LParen) {
                loop {
                    prop_names.push(ctx.expect_identifier()?);
                    if !ctx.match_token(TokenKind::Comma) {
                        break;
                    }
                }
                ctx.expect_token(TokenKind::RParen)?;
            }
            
            ctx.expect_token(TokenKind::Values)?;
            
            // 解析值列表 - 支持 NebulaGraph 格式: vid:(prop1, prop2, ...)
            let mut values = Vec::new();
            loop {
                // 解析 VID
                let vid = self.parse_expression(ctx)?;
                
                // 期望 :
                ctx.expect_token(TokenKind::Colon)?;
                
                // 解析属性值列表
                ctx.expect_token(TokenKind::LParen)?;
                let mut props = Vec::new();
                
                // 检查是否为空括号
                if !ctx.check_token(TokenKind::RParen) {
                    loop {
                        props.push(self.parse_expression(ctx)?);
                        if !ctx.match_token(TokenKind::Comma) {
                            break;
                        }
                    }
                }
                
                ctx.expect_token(TokenKind::RParen)?;
                values.push((vid, props));
                
                if !ctx.match_token(TokenKind::Comma) {
                    break;
                }
            }
            
            InsertTarget::Vertices {
                tag_name,
                prop_names,
                values,
            }
        } else if ctx.match_token(TokenKind::Edge) {
            let edge_name = ctx.expect_identifier()?;
            
            // 解析属性名列表（可选）
            let mut prop_names = Vec::new();
            if ctx.match_token(TokenKind::LParen) {
                loop {
                    prop_names.push(ctx.expect_identifier()?);
                    if !ctx.match_token(TokenKind::Comma) {
                        break;
                    }
                }
                ctx.expect_token(TokenKind::RParen)?;
            }
            
            ctx.expect_token(TokenKind::Values)?;
            
            // 解析边值列表
            let mut edges = Vec::new();
            loop {
                ctx.expect_token(TokenKind::LParen)?;
                let src = self.parse_expression(ctx)?;
                ctx.expect_token(TokenKind::Comma)?;
                let dst = self.parse_expression(ctx)?;
                
                // 可选的 rank
                let rank = if ctx.match_token(TokenKind::Comma) {
                    Some(self.parse_expression(ctx)?)
                } else {
                    None
                };
                
                // 解析属性值
                let mut props = Vec::new();
                while ctx.match_token(TokenKind::Comma) {
                    props.push(self.parse_expression(ctx)?);
                }
                
                ctx.expect_token(TokenKind::RParen)?;
                edges.push((src, dst, rank, props));
                
                if !ctx.match_token(TokenKind::Comma) {
                    break;
                }
            }
            
            InsertTarget::Edge {
                edge_name,
                prop_names,
                edges,
            }
        } else {
            return Err(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                "Expected VERTEX or EDGE after INSERT".to_string(),
                ctx.current_position(),
            ));
        };

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Stmt::Insert(InsertStmt { span, target }))
    }

    /// 解析 DELETE 语句
    pub fn parse_delete_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Delete)?;

        let target = if ctx.match_token(TokenKind::Vertex) {
            let ids = self.parse_expression_list(ctx)?;
            DeleteTarget::Vertices(ids)
        } else if ctx.match_token(TokenKind::Edge) {
            let edge_type = ctx.expect_identifier()?;
            let mut edges = Vec::new();
            loop {
                let src = self.parse_expression(ctx)?;
                ctx.expect_token(TokenKind::Arrow)?;
                let dst = self.parse_expression(ctx)?;
                let rank = if ctx.match_token(TokenKind::At) {
                    Some(self.parse_expression(ctx)?)
                } else {
                    None
                };
                edges.push((src, dst, rank));
                if !ctx.match_token(TokenKind::Comma) {
                    break;
                }
            }
            DeleteTarget::Edges {
                edge_type: Some(edge_type),
                edges,
            }
        } else {
            DeleteTarget::Vertices(self.parse_expression_list(ctx)?)
        };

        let where_clause = if ctx.match_token(TokenKind::Where) {
            Some(self.parse_expression(ctx)?)
        } else {
            None
        };

        // 解析 WITH EDGE 选项（仅对删除顶点有效）
        let with_edge = if matches!(target, DeleteTarget::Vertices(_)) {
            ctx.match_token(TokenKind::With) && ctx.match_token(TokenKind::Edge)
        } else {
            false
        };

        Ok(Stmt::Delete(DeleteStmt {
            span: start_span,
            target,
            where_clause,
            with_edge,
        }))
    }

    /// 解析 UPDATE 语句
    pub fn parse_update_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Update)?;

        let target = if ctx.match_token(TokenKind::Vertex) {
            UpdateTarget::Vertex(self.parse_expression(ctx)?)
        } else if ctx.match_token(TokenKind::Tag) {
            UpdateTarget::Tag(ctx.expect_identifier()?)
        } else {
            UpdateTarget::Vertex(self.parse_expression(ctx)?)
        };

        let set_clause = if ctx.match_token(TokenKind::Set) {
            ClauseParser::new().parse_set_clause(ctx)?
        } else {
            SetClause {
                span: ctx.current_span(),
                assignments: Vec::new(),
            }
        };

        let where_clause = if ctx.match_token(TokenKind::Where) {
            Some(self.parse_expression(ctx)?)
        } else {
            None
        };

        Ok(Stmt::Update(UpdateStmt {
            span: start_span,
            target,
            set_clause,
            where_clause,
        }))
    }

    /// 解析 MERGE 语句
    pub fn parse_merge_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Merge)?;

        let pattern = TraversalParser::new().parse_pattern(ctx)?;

        let on_create = if ctx.match_token(TokenKind::On)
            && ctx.match_token(TokenKind::Create)
        {
            Some(ClauseParser::new().parse_set_clause(ctx)?)
        } else {
            None
        };

        let on_match = if ctx.match_token(TokenKind::On) && ctx.match_token(TokenKind::Match) {
            Some(ClauseParser::new().parse_set_clause(ctx)?)
        } else {
            None
        };

        Ok(Stmt::Merge(MergeStmt {
            span: start_span,
            pattern,
            on_create,
            on_match,
        }))
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

    /// 解析表达式
    fn parse_expression(&mut self, ctx: &mut ParseContext) -> Result<CoreExpression, ParseError> {
        let mut expr_parser = ExprParser::new(ctx);
        let result = expr_parser.parse_expression(ctx)?;
        Ok(result.expr)
    }
}

impl Default for DmlParser {
    fn default() -> Self {
        Self::new()
    }
}
