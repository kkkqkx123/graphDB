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

    /// 在 UPDATE token 已被消费后解析 UPDATE 语句
    pub fn parse_update_after_token(&mut self, ctx: &mut ParseContext, start_span: crate::query::parser::ast::types::Span) -> Result<Stmt, ParseError> {
        use crate::query::parser::ast::stmt::{UpdateStmt, UpdateTarget, SetClause};
        use crate::query::parser::parser::clause_parser::ClauseParser;

        let target = if ctx.match_token(TokenKind::Vertex) {
            self.parse_update_vertex(ctx)?
        } else if ctx.match_token(TokenKind::Edge) {
            self.parse_update_edge(ctx)?
        } else {
            // 默认是顶点更新
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

        // 解析可选的 YIELD 子句
        let yield_clause = if ctx.match_token(TokenKind::Yield) {
            Some(self.parse_yield_clause(ctx)?)
        } else {
            None
        };

        Ok(Stmt::Update(UpdateStmt {
            span: start_span,
            target,
            set_clause,
            where_clause,
            is_upsert: false,
            yield_clause,
        }))
    }

    /// 解析 INSERT 语句
    /// 支持语法:
    ///   INSERT VERTEX [IF NOT EXISTS] <tag>(props), <tag>(props)... VALUES vid:(values), ...
    ///   INSERT EDGE [IF NOT EXISTS] <edge_type>(props) VALUES src->dst@rank:(values), ...
    pub fn parse_insert_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Insert)?;

        let target = if ctx.match_token(TokenKind::Vertex) {
            let if_not_exists = self.parse_if_not_exists(ctx)?;
            let target = self.parse_insert_vertices(ctx)?;
            (target, if_not_exists)
        } else if ctx.match_token(TokenKind::Edge) {
            let if_not_exists = self.parse_if_not_exists(ctx)?;
            let target = self.parse_insert_edges(ctx)?;
            (target, if_not_exists)
        } else {
            return Err(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                "Expected VERTEX or EDGE after INSERT".to_string(),
                ctx.current_position(),
            ));
        };

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Stmt::Insert(InsertStmt { span, target: target.0, if_not_exists: target.1 }))
    }

    /// 解析 IF NOT EXISTS 子句
    fn parse_if_not_exists(&self, ctx: &mut ParseContext) -> Result<bool, ParseError> {
        if ctx.match_token(TokenKind::If) {
            ctx.expect_token(TokenKind::Not)?;
            ctx.expect_token(TokenKind::Exists)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// 解析 INSERT VERTICES 语句
    fn parse_insert_vertices(&mut self, ctx: &mut ParseContext) -> Result<InsertTarget, ParseError> {
        // 解析 Tag 列表: tag1(prop1, prop2), tag2(prop1, prop2)...
        let mut tags = Vec::new();
        
        loop {
            let tag_name = ctx.expect_identifier()?;
            
            // 解析属性名列表（可选）
            let mut prop_names = Vec::new();
            let mut is_default_props = false;
            
            if ctx.match_token(TokenKind::LParen) {
                // 检查是否为空括号 ()
                if !ctx.check_token(TokenKind::RParen) {
                    loop {
                        prop_names.push(ctx.expect_identifier()?);
                        if !ctx.match_token(TokenKind::Comma) {
                            break;
                        }
                    }
                } else {
                    is_default_props = true;
                }
                ctx.expect_token(TokenKind::RParen)?;
            } else {
                is_default_props = true;
            }
            
            tags.push(TagInsertSpec {
                tag_name,
                prop_names,
                is_default_props,
            });
            
            if !ctx.match_token(TokenKind::Comma) {
                break;
            }
        }
        
        ctx.expect_token(TokenKind::Values)?;
        
        // 解析值列表 - 格式: vid:(tag1_values):(tag2_values)...
        let mut rows = Vec::new();
        loop {
            let vid = self.parse_expression(ctx)?;
            
            // 每个 Tag 对应一个值列表
            let mut tag_values = Vec::new();
            for _ in 0..tags.len() {
                ctx.expect_token(TokenKind::Colon)?;
                ctx.expect_token(TokenKind::LParen)?;
                
                let mut values = Vec::new();
                if !ctx.check_token(TokenKind::RParen) {
                    loop {
                        values.push(self.parse_expression(ctx)?);
                        if !ctx.match_token(TokenKind::Comma) {
                            break;
                        }
                    }
                }
                ctx.expect_token(TokenKind::RParen)?;
                tag_values.push(values);
            }
            
            rows.push(VertexRow { vid, tag_values });
            
            if !ctx.match_token(TokenKind::Comma) {
                break;
            }
        }
        
        Ok(InsertTarget::Vertices { tags, values: rows })
    }

    /// 解析 INSERT EDGES 语句
    fn parse_insert_edges(&mut self, ctx: &mut ParseContext) -> Result<InsertTarget, ParseError> {
        let edge_name = ctx.expect_identifier()?;
        
        // 解析属性名列表（可选）
        let mut prop_names = Vec::new();
        if ctx.match_token(TokenKind::LParen) {
            if !ctx.check_token(TokenKind::RParen) {
                loop {
                    prop_names.push(ctx.expect_identifier()?);
                    if !ctx.match_token(TokenKind::Comma) {
                        break;
                    }
                }
            }
            ctx.expect_token(TokenKind::RParen)?;
        }
        
        ctx.expect_token(TokenKind::Values)?;

        // 解析边值列表 - 格式: src -> dst @ rank:(prop1, prop2, ...)
        let mut edges = Vec::new();
        loop {
            let src = self.parse_expression(ctx)?;
            ctx.expect_token(TokenKind::Arrow)?;
            let dst = self.parse_expression(ctx)?;
            
            // 可选的 rank: @rank
            let rank = if ctx.match_token(TokenKind::At) {
                Some(self.parse_expression(ctx)?)
            } else {
                None
            };

            ctx.expect_token(TokenKind::Colon)?;
            ctx.expect_token(TokenKind::LParen)?;
            
            let mut props = Vec::new();
            if !ctx.check_token(TokenKind::RParen) {
                loop {
                    props.push(self.parse_expression(ctx)?);
                    if !ctx.match_token(TokenKind::Comma) {
                        break;
                    }
                }
            }
            ctx.expect_token(TokenKind::RParen)?;
            
            edges.push((src, dst, rank, props));

            if !ctx.match_token(TokenKind::Comma) {
                break;
            }
        }
        
        Ok(InsertTarget::Edge {
            edge_name,
            prop_names,
            edges,
        })
    }

    /// 解析 DELETE 语句
    pub fn parse_delete_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Delete)?;

        let target = if ctx.match_token(TokenKind::Vertex) {
            self.parse_delete_vertices(ctx)?
        } else if ctx.match_token(TokenKind::Edge) {
            self.parse_delete_edges(ctx)?
        } else if ctx.match_token(TokenKind::Tag) {
            self.parse_delete_tags(ctx)?
        } else {
            // 默认删除顶点
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

    /// 解析 DELETE VERTICES
    fn parse_delete_vertices(&mut self, ctx: &mut ParseContext) -> Result<DeleteTarget, ParseError> {
        let ids = self.parse_expression_list(ctx)?;
        Ok(DeleteTarget::Vertices(ids))
    }

    /// 解析 DELETE EDGES
    fn parse_delete_edges(&mut self, ctx: &mut ParseContext) -> Result<DeleteTarget, ParseError> {
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
        Ok(DeleteTarget::Edges {
            edge_type: Some(edge_type),
            edges,
        })
    }

    /// 解析 DELETE TAG 语句: DELETE TAG <tag1>, <tag2>... FROM <vid1>, <vid2>...
    /// 支持: DELETE TAG * FROM <vids> 删除所有 Tag
    fn parse_delete_tags(&mut self, ctx: &mut ParseContext) -> Result<DeleteTarget, ParseError> {
        let mut tag_names = Vec::new();
        let mut is_all_tags = false;
        
        // 检查是否是 * (所有 Tag)
        if ctx.match_token(TokenKind::Star) {
            is_all_tags = true;
        } else {
            loop {
                tag_names.push(ctx.expect_identifier()?);
                if !ctx.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }
        
        ctx.expect_token(TokenKind::From)?;
        
        let vertex_ids = self.parse_expression_list(ctx)?;
        
        Ok(DeleteTarget::Tags {
            tag_names,
            vertex_ids,
            is_all_tags,
        })
    }

    /// 解析 UPDATE 语句
    /// 支持语法:
    ///   UPDATE VERTEX <vid> ON <tag> SET ... [WHERE ...] [YIELD ...]
    ///   UPDATE EDGE <src>-><dst>[@rank] OF <edge_type> SET ... [WHERE ...] [YIELD ...]
    ///   UPSERT VERTEX <vid> ON <tag> SET ... [WHERE ...] [YIELD ...]
    ///   UPSERT EDGE <src>-><dst>[@rank] OF <edge_type> SET ... [WHERE ...] [YIELD ...]
    pub fn parse_update_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        
        // 判断是 UPDATE 还是 UPSERT
        let is_upsert = if ctx.match_token(TokenKind::Upsert) {
            true
        } else {
            ctx.expect_token(TokenKind::Update)?;
            false
        };

        let target = if ctx.match_token(TokenKind::Vertex) {
            self.parse_update_vertex(ctx)?
        } else if ctx.match_token(TokenKind::Edge) {
            self.parse_update_edge(ctx)?
        } else {
            // 默认是顶点更新
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

        // 解析可选的 YIELD 子句
        let yield_clause = if ctx.match_token(TokenKind::Yield) {
            Some(self.parse_yield_clause(ctx)?)
        } else {
            None
        };

        Ok(Stmt::Update(UpdateStmt {
            span: start_span,
            target,
            set_clause,
            where_clause,
            is_upsert,
            yield_clause,
        }))
    }

    /// 解析 UPDATE VERTEX
    fn parse_update_vertex(&mut self, ctx: &mut ParseContext) -> Result<UpdateTarget, ParseError> {
        let vid = self.parse_expression(ctx)?;
        
        // 可选的 ON <tag> 子句
        if ctx.match_token(TokenKind::On) {
            let tag_name = ctx.expect_identifier()?;
            Ok(UpdateTarget::TagOnVertex { vid: Box::new(vid), tag_name })
        } else {
            Ok(UpdateTarget::Vertex(vid))
        }
    }

    /// 解析 UPDATE EDGE
    fn parse_update_edge(&mut self, ctx: &mut ParseContext) -> Result<UpdateTarget, ParseError> {
        let src = self.parse_expression(ctx)?;
        ctx.expect_token(TokenKind::Arrow)?;
        let dst = self.parse_expression(ctx)?;
        
        // 可选的 rank: @rank
        let rank = if ctx.match_token(TokenKind::At) {
            Some(self.parse_expression(ctx)?)
        } else {
            None
        };
        
        // 期望 OF <edge_type>
        ctx.expect_token(TokenKind::Of)?;
        let edge_type = ctx.expect_identifier()?;
        
        Ok(UpdateTarget::Edge {
            src,
            dst,
            edge_type: Some(edge_type),
            rank,
        })
    }

    /// 解析 YIELD 子句
    fn parse_yield_clause(&mut self, ctx: &mut ParseContext) -> Result<YieldClause, ParseError> {
        let start_span = ctx.current_span();
        let mut items = Vec::new();
        
        loop {
            let expr = self.parse_expression(ctx)?;
            let alias = if ctx.match_token(TokenKind::As) {
                Some(ctx.expect_identifier()?)
            } else {
                None
            };
            items.push(YieldItem { expression: expr, alias });
            
            if !ctx.match_token(TokenKind::Comma) {
                break;
            }
        }
        
        // 解析 WHERE 子句
        let where_clause = if ctx.match_token(TokenKind::Where) {
            Some(self.parse_expression(ctx)?)
        } else {
            None
        };
        
        let end_span = ctx.current_span();
        Ok(YieldClause {
            span: ctx.merge_span(start_span.start, end_span.end),
            items,
            where_clause,
            limit: None,
            skip: None,
            sample: None,
        })
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
