//! 数据修改语句解析模块
//!
//! 负责解析数据修改相关语句，包括 INSERT、DELETE、UPDATE、MERGE 等。

use std::sync::Arc;

use crate::core::types::expression::ContextualExpression;
use crate::core::types::expression::Expression as CoreExpression;
use crate::core::types::EdgeDirection;
use crate::query::parser::ast::stmt::*;
use crate::query::parser::core::error::ParseError;
use crate::query::parser::core::token::TokenKindExt;
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

    /// 解析 UPDATE 语句（完整的，包括 UPDATE token）
    pub fn parse_update_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Update)?;
        self.parse_update_after_token(ctx, start_span)
    }

    /// 解析 UPSERT 语句（完整的，包括 UPSERT token）
    pub fn parse_upsert_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Upsert)?;
        ctx.set_upsert_mode(true);
        let result = self.parse_update_after_token(ctx, start_span);
        ctx.set_upsert_mode(false);
        result
    }

    /// 在 UPDATE token 已被消费后解析 UPDATE 语句
    pub fn parse_update_after_token(&mut self, ctx: &mut ParseContext, start_span: crate::query::parser::ast::types::Span) -> Result<Stmt, ParseError> {
        use crate::query::parser::ast::stmt::{UpdateStmt, UpdateTarget, SetClause};
        use crate::query::parser::parser::clause_parser::ClauseParser;

        // 检查是否是 UPSERT 语法
        let is_upsert = ctx.is_upsert_mode();
        
        let target = if ctx.match_token(TokenKind::Vertex) {
            self.parse_update_vertex(ctx)?
        } else if ctx.match_token(TokenKind::Edge) {
            self.parse_update_edge(ctx)?
        } else {
            // 检查是否是 UPSERT VERTEX vid ON tag_name 语法
            if is_upsert {
                let vid = self.parse_expression(ctx)?;
                if ctx.match_token(TokenKind::On) {
                    let tag_name = ctx.expect_identifier()?;
                    UpdateTarget::TagOnVertex {
                        vid: Box::new(vid),
                        tag_name,
                    }
                } else {
                    UpdateTarget::Vertex(vid)
                }
            } else {
                // 默认是顶点更新
                UpdateTarget::Vertex(self.parse_expression(ctx)?)
            }
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

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Stmt::Update(UpdateStmt {
            span,
            target,
            set_clause,
            where_clause,
            is_upsert,
            yield_clause: None,
        }))
    }

    fn parse_update_vertex(&mut self, ctx: &mut ParseContext) -> Result<UpdateTarget, ParseError> {
        let vid = self.parse_expression(ctx)?;
        Ok(UpdateTarget::Vertex(vid))
    }

    fn parse_update_edge(&mut self, ctx: &mut ParseContext) -> Result<UpdateTarget, ParseError> {
        // 检查是否是 UPSERT EDGE 语法：src -> dst @rank OF edge_type
        // 还是 UPDATE EDGE 语法：OF edge_type FROM src TO dst [@rank]
        let is_upsert = ctx.is_upsert_mode();
        
        if is_upsert {
            // UPSERT EDGE 语法：src -> dst [@rank] OF edge_type
            // 注意：EDGE token 已经在 parse_update_after_token 中被消费了
            let src = self.parse_expression(ctx)?;
            ctx.expect_token(TokenKind::Arrow)?;
            let dst = self.parse_expression(ctx)?;

            let rank = if ctx.match_token(TokenKind::At) {
                Some(self.parse_expression(ctx)?)
            } else {
                None
            };

            ctx.expect_token(TokenKind::Of)?;
            let edge_type = Some(ctx.expect_identifier()?);

            Ok(UpdateTarget::Edge {
                edge_type,
                src,
                dst,
                rank,
            })
        } else {
            // UPDATE EDGE 语法：OF edge_type FROM src TO dst [@rank]
            ctx.expect_token(TokenKind::Of)?;

            // 解析边类型
            let edge_type = ctx.expect_identifier()?;

            // 解析 src 和 dst
            ctx.expect_token(TokenKind::From)?;
            let src = self.parse_expression(ctx)?;

            ctx.expect_token(TokenKind::To)?;
            let dst = self.parse_expression(ctx)?;

            // 解析 @rank（可选）
            let rank = if ctx.match_token(TokenKind::At) {
                Some(self.parse_expression(ctx)?)
            } else {
                None
            };

            Ok(UpdateTarget::Edge {
                edge_type: Some(edge_type),
                src,
                dst,
                rank,
            })
        }
    }

    /// 解析 DELETE 语句
    pub fn parse_delete_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        use crate::query::parser::ast::stmt::{DeleteStmt, DeleteTarget};

        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Delete)?;

        // 检查是否有 VERTEX、EDGE 或 TAG 关键字
        let target = if ctx.match_token(TokenKind::Vertex) {
            // DELETE VERTEX vid [, vid ...]
            let mut vids = vec![];
            loop {
                vids.push(self.parse_expression(ctx)?);
                if !ctx.match_token(TokenKind::Comma) {
                    break;
                }
            }
            DeleteTarget::Vertices(vids)
        } else if ctx.match_token(TokenKind::Edge) {
            // DELETE EDGE edge_type src -> dst [@rank]
            let edge_type = Some(ctx.expect_identifier()?);

            let mut edges = vec![];
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
                edge_type,
                edges,
            }
        } else if ctx.match_token(TokenKind::Tag) {
            // DELETE TAG tag_name [, tag_name ...] FROM vid [, vid ...]
            let mut tags = vec![];
            
            // 检查是否是通配符 *
            if ctx.match_token(TokenKind::Star) {
                tags.push("*".to_string());
            } else {
                // 解析标签列表
                loop {
                    let tag_name = ctx.expect_identifier()?;
                    tags.push(tag_name);
                    if !ctx.match_token(TokenKind::Comma) {
                        break;
                    }
                }
            }
            
            // 期望 FROM 关键字
            ctx.expect_token(TokenKind::From)?;
            
            // 解析顶点 ID 列表
            let mut vids = vec![];
            loop {
                vids.push(self.parse_expression(ctx)?);
                if !ctx.match_token(TokenKind::Comma) {
                    break;
                }
            }
            
            let is_all_tags = tags.iter().any(|t| t == "*");
            
            DeleteTarget::Tags { 
                tag_names: tags, 
                vertex_ids: vids,
                is_all_tags,
            }
        } else {
            // 默认解析为顶点删除
            let mut vids = vec![];
            loop {
                vids.push(self.parse_expression(ctx)?);
                if !ctx.match_token(TokenKind::Comma) {
                    break;
                }
            }
            DeleteTarget::Vertices(vids)
        };

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Stmt::Delete(DeleteStmt { 
            span, 
            target,
            where_clause: None,
            with_edge: false,
        }))
    }

    /// 解析 INSERT 语句
    pub fn parse_insert_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {

        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Insert)?;

        // 检查是 VERTEX 还是 EDGE
        let target = if ctx.match_token(TokenKind::Vertex) {
            self.parse_insert_vertex(ctx, start_span)?
        } else if ctx.match_token(TokenKind::Edge) {
            self.parse_insert_edge(ctx, start_span)?
        } else {
            return Err(ParseError::new(
                crate::query::parser::core::error::ParseErrorKind::UnexpectedToken,
                "Expected VERTEX or EDGE after INSERT".to_string(),
                ctx.current_position(),
            ));
        };

        Ok(target)
    }

    /// 解析 INSERT VERTEX
    fn parse_insert_vertex(&mut self, ctx: &mut ParseContext, start_span: crate::query::parser::ast::types::Span) -> Result<Stmt, ParseError> {
        use crate::query::parser::ast::stmt::{InsertStmt, InsertTarget, TagInsertSpec, VertexRow};

        // 解析 IF NOT EXISTS（可选）
        let mut if_not_exists = false;
        if ctx.match_token(TokenKind::If) {
            ctx.expect_token(TokenKind::Not)?;
            ctx.expect_token(TokenKind::Exists)?;
            if_not_exists = true;
        }

        // 解析 TAG 列表
        // 支持两种语法：
        // 1. ON tag1, tag2（可选）
        // 2. tag_name(prop1, prop2), tag2_name(prop3, prop4)（NebulaGraph 标准语法）
        let mut tags = vec![];
        if ctx.match_token(TokenKind::On) {
            // 语法：ON tag1, tag2
            loop {
                let tag_name = ctx.expect_identifier()?;
                tags.push(TagInsertSpec {
                    tag_name,
                    prop_names: vec![],
                    is_default_props: false,
                });
                if !ctx.match_token(TokenKind::Comma) {
                    break;
                }
            }
        } else {
            // 检查是否是 NebulaGraph 标准语法：tag_name(prop1, prop2), tag2_name(prop3, prop4)
            if ctx.is_identifier_token() {
                loop {
                    let tag_name = ctx.expect_identifier()?;
                    let mut prop_names = vec![];
                    
                    // 检查是否有属性名列表
                    if ctx.match_token(TokenKind::LParen) {
                        loop {
                            let prop_name = ctx.expect_identifier()?;
                            prop_names.push(prop_name);
                            if !ctx.match_token(TokenKind::Comma) {
                                break;
                            }
                        }
                        ctx.expect_token(TokenKind::RParen)?;
                    }
                    
                    tags.push(TagInsertSpec {
                        tag_name,
                        prop_names,
                        is_default_props: false,
                    });
                    
                    // 检查是否有更多标签
                    if !ctx.match_token(TokenKind::Comma) {
                        break;
                    }
                }
            }
        }

        // 解析 VALUES 关键字
        if ctx.check_token(TokenKind::Values) {
            ctx.next_token(); // 消费 VALUES
        }

        // 解析插入值列表
        let mut values = vec![];
        loop {
            // 解析 vid
            let vid = self.parse_expression(ctx)?;

            // 解析属性列表
            let tag_values = if ctx.match_token(TokenKind::Colon) {
                ctx.expect_token(TokenKind::LParen)?;
                let mut props = vec![];
                loop {
                    let value = self.parse_expression(ctx)?;
                    props.push(value);
                    if !ctx.match_token(TokenKind::Comma) {
                        break;
                    }
                }
                ctx.expect_token(TokenKind::RParen)?;
                vec![props]
            } else {
                vec![]
            };

            values.push(VertexRow { vid, tag_values });

            if !ctx.match_token(TokenKind::Comma) {
                break;
            }
        }

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Stmt::Insert(InsertStmt {
            span,
            target: InsertTarget::Vertices { tags, values },
            if_not_exists,
        }))
    }

    /// 解析 INSERT EDGE
    fn parse_insert_edge(&mut self, ctx: &mut ParseContext, start_span: crate::query::parser::ast::types::Span) -> Result<Stmt, ParseError> {
        use crate::query::parser::ast::stmt::{InsertStmt, InsertTarget};

        // 解析边类型和属性名列表
        let edge_name = ctx.expect_identifier()?;
        let mut prop_names = vec![];

        if ctx.match_token(TokenKind::LParen) {
            loop {
                let prop_name = ctx.expect_identifier()?;
                prop_names.push(prop_name);
                if !ctx.match_token(TokenKind::Comma) {
                    break;
                }
            }
            ctx.expect_token(TokenKind::RParen)?;
        }

        // 解析 IF NOT EXISTS（可选）
        let mut if_not_exists = false;
        if ctx.match_token(TokenKind::If) {
            ctx.expect_token(TokenKind::Not)?;
            ctx.expect_token(TokenKind::Exists)?;
            if_not_exists = true;
        }

        // 解析 VALUES 关键字
        if ctx.check_token(TokenKind::Values) {
            ctx.next_token(); // 消费 VALUES
        }

        // 解析边值列表
        let mut edges = vec![];
        loop {
            // 解析 src -> dst
            let src = self.parse_expression(ctx)?;
            ctx.expect_token(TokenKind::Arrow)?;
            let dst = self.parse_expression(ctx)?;

            // 解析可选的 rank
            let rank = if ctx.match_token(TokenKind::At) {
                Some(self.parse_expression(ctx)?)
            } else {
                None
            };

            // 解析属性值列表
            let mut values = vec![];
            if ctx.match_token(TokenKind::Colon) {
                ctx.expect_token(TokenKind::LParen)?;
                loop {
                    let value = self.parse_expression(ctx)?;
                    values.push(value);
                    if !ctx.match_token(TokenKind::Comma) {
                        break;
                    }
                }
                ctx.expect_token(TokenKind::RParen)?;
            }

            edges.push((src, dst, rank, values));

            if !ctx.match_token(TokenKind::Comma) {
                break;
            }
        }

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Stmt::Insert(InsertStmt {
            span,
            target: InsertTarget::Edge {
                edge_name,
                prop_names,
                edges,
            },
            if_not_exists,
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

    /// 解析表达式
    fn parse_expression(&mut self, ctx: &mut ParseContext) -> Result<ContextualExpression, ParseError> {
        let mut expr_parser = ExprParser::new(ctx);
        expr_parser.parse_expression_with_context(ctx, ctx.expression_context_clone())
    }

    /// 解析 Cypher 风格的 CREATE 数据语句（CREATE token 已被消费）
    /// 支持语法:
    ///   CREATE (n:Label {prop: value})
    ///   CREATE (a)-[:Type {prop: value}]->(b)
    ///   CREATE (a:Label1)-[:Type]->(b:Label2)
    pub fn parse_create_data_after_token(&mut self, ctx: &mut ParseContext, start_span: crate::query::parser::ast::types::Span) -> Result<Stmt, ParseError> {
        // 解析模式列表（支持多个模式用逗号分隔）
        let mut patterns = Vec::new();
        
        loop {
            let pattern = self.parse_create_pattern(ctx)?;
            patterns.push(pattern);
            
            if !ctx.match_token(TokenKind::Comma) {
                break;
            }
        }

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Stmt::Create(CreateStmt {
            span,
            target: CreateTarget::Path { patterns },
            if_not_exists: false,
        }))
    }

    /// 解析 CREATE 语句中的模式
    fn parse_create_pattern(&mut self, ctx: &mut ParseContext) -> Result<crate::query::parser::ast::pattern::Pattern, ParseError> {
        use crate::query::parser::ast::pattern::*;

        // 解析起始节点
        let start_node = self.parse_node_pattern(ctx)?;
        
        // 检查是否有边模式（使用 Arrow 或 LeftArrow）
        if ctx.check_token(TokenKind::Arrow) || ctx.check_token(TokenKind::LeftArrow) {
            let edge = self.parse_edge_pattern(ctx)?;
            let end_node = self.parse_node_pattern(ctx)?;
            
            // 构建路径模式
            let span = ctx.merge_span(start_node.span.start, end_node.span.end);
            let elements = vec![
                PathElement::Node(start_node),
                PathElement::Edge(edge),
                PathElement::Node(end_node),
            ];
            Ok(Pattern::Path(PathPattern {
                span,
                elements,
            }))
        } else {
            // 只有节点模式
            Ok(Pattern::Node(start_node))
        }
    }

    /// 解析节点模式: (var:Label {prop: value})
    fn parse_node_pattern(&mut self, ctx: &mut ParseContext) -> Result<crate::query::parser::ast::pattern::NodePattern, ParseError> {
        use crate::query::parser::ast::pattern::*;

        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::LParen)?;
        
        // 可选的变量名
        let variable = if ctx.current_token().kind.is_identifier() {
            Some(ctx.expect_identifier()?)
        } else {
            None
        };
        
        // 可选的标签列表
        let mut labels = Vec::new();
        if ctx.match_token(TokenKind::Colon) {
            loop {
                labels.push(ctx.expect_identifier()?);
                if !ctx.match_token(TokenKind::Colon) {
                    break;
                }
            }
        }
        
        // 可选的属性映射
        let properties = if ctx.match_token(TokenKind::LBrace) {
            let props = self.parse_property_map(ctx)?;
            ctx.expect_token(TokenKind::RBrace)?;
            Some(props)
        } else {
            None
        };
        
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

    /// 解析边模式: -[:Type {prop: value}]-> 或 <-[:Type {prop: value}]-
    fn parse_edge_pattern(&mut self, ctx: &mut ParseContext) -> Result<crate::query::parser::ast::pattern::EdgePattern, ParseError> {
        use crate::query::parser::ast::pattern::*;

        let start_span = ctx.current_span();
        
        // 确定方向（使用 Arrow、LeftArrow、RightArrow）
        let direction = if ctx.match_token(TokenKind::LeftArrow) {
            // <- 开始，表示入边
            EdgeDirection::In
        } else if ctx.match_token(TokenKind::Arrow) {
            // -> 出边
            EdgeDirection::Out
        } else if ctx.match_token(TokenKind::RightArrow) {
            // => 或其他箭头
            EdgeDirection::Out
        } else {
            // 默认双向
            EdgeDirection::Both
        };
        
        // 解析边类型和属性 [:Type {prop: value}]
        ctx.expect_token(TokenKind::LBracket)?;
        
        // 可选的变量名
        let variable = if ctx.current_token().kind.is_identifier() {
            Some(ctx.expect_identifier()?)
        } else {
            None
        };
        
        // 可选的边类型
        let mut edge_types = Vec::new();
        if ctx.match_token(TokenKind::Colon) {
            edge_types.push(ctx.expect_identifier()?);
        }
        
        // 可选的属性映射
        let properties = if ctx.match_token(TokenKind::LBrace) {
            let props = self.parse_property_map(ctx)?;
            ctx.expect_token(TokenKind::RBrace)?;
            Some(props)
        } else {
            None
        };
        
        ctx.expect_token(TokenKind::RBracket)?;
        
        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);
        
        Ok(EdgePattern {
            span,
            variable,
            edge_types,
            properties,
            predicates: Vec::new(),
            direction,
            range: None,
        })
    }

    /// 解析属性映射: {prop1: value1, prop2: value2}
    fn parse_property_map(&mut self, ctx: &mut ParseContext) -> Result<ContextualExpression, ParseError> {
        let _start_span = ctx.current_span();
        let mut properties = Vec::new();
        
        if !ctx.check_token(TokenKind::RBrace) {
            loop {
                let key = ctx.expect_identifier()?;
                ctx.expect_token(TokenKind::Colon)?;
                let value = self.parse_expression(ctx)?;
                let value_expr = value.expression()
                    .ok_or_else(|| ParseError::new_simple(
                        "Expression not registered in context".to_string(),
                        ctx.current_position()
                    ))?
                    .inner()
                    .clone();
                properties.push((key, value_expr));
                
                if !ctx.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }
        
        // 创建 Map 表达式并注册到上下文
        let expr = CoreExpression::Map(properties);
        let expr_meta = crate::core::types::expression::ExpressionMeta::new(expr);
        let id = ctx.expression_context().register_expression(expr_meta);
        Ok(ContextualExpression::new(id, ctx.expression_context_clone()))
    }
}

impl Default for DmlParser {
    fn default() -> Self {
        Self::new()
    }
}
