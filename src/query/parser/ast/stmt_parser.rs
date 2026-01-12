//! 语句解析器 (v2)

use super::*;
use crate::core::Value;
use crate::query::parser::lexer::{Lexer, TokenKind as LexerToken};

/// 语句解析器
pub struct StmtParser {
    lexer: Lexer,
}

impl StmtParser {
    /// 创建语句解析器
    pub fn new(input: &str) -> Self {
        Self {
            lexer: Lexer::new(input),
        }
    }
    /// 解析 MATCH 语句
    pub fn parse_match_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::Match)?;

        // 解析模式
        let patterns = self.parse_patterns()?;

        // 解析可选的 WHERE 子句
        let where_clause = if self.match_token(LexerToken::Where) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        // 解析 RETURN 子句
        let return_clause = if self.match_token(LexerToken::Return) {
            Some(self.parse_return_clause()?)
        } else {
            None
        };

        // 解析可选的 ORDER BY 子句
        let order_by = if self.match_token(LexerToken::Order) && self.match_token(LexerToken::By) {
            Some(self.parse_order_by_clause()?)
        } else {
            None
        };

        // 解析可选的 LIMIT 子句
        let limit = if self.match_token(LexerToken::Limit) {
            Some(self.parse_integer()? as usize)
        } else {
            None
        };

        // 解析可选的 SKIP 子句
        let skip = if self.match_token(LexerToken::Skip) {
            Some(self.parse_integer()? as usize)
        } else {
            None
        };

        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Stmt::Match(MatchStmt {
            span,
            patterns,
            where_clause,
            return_clause,
            order_by,
            limit,
            skip,
        }))
    }

    /// 解析 CREATE 语句
    pub fn parse_create_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::Create)?;

        // 检查是否是 TAG 或 INDEX
        if self.match_token(LexerToken::Tag) {
            self.parse_create_tag_statement(start_span)
        } else if self.match_token(LexerToken::Index) {
            self.parse_create_index_statement(start_span)
        } else {
            // 默认创建节点或边
            self.parse_create_node_or_edge_statement(start_span)
        }
    }

    /// 解析 DELETE 语句
    pub fn parse_delete_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::Delete)?;

        // 检查删除目标类型
        let target =
            if self.match_token(LexerToken::Vertex) || self.match_token(LexerToken::Vertices) {
                // 删除顶点
                let vertices = self.parse_vertex_list()?;
                DeleteTarget::Vertices(vertices)
            } else if self.match_token(LexerToken::Edge) || self.match_token(LexerToken::Edges) {
                // 删除边
                self.parse_delete_edge_target()?
            } else if self.match_token(LexerToken::Tag) {
                // 删除标签
                let tag_name = self.expect_identifier()?;
                DeleteTarget::Tag(tag_name)
            } else {
                // 默认删除顶点
                let vertices = self.parse_vertex_list()?;
                DeleteTarget::Vertices(vertices)
            };

        // 解析可选的 WHERE 子句
        let where_clause = if self.match_token(LexerToken::Where) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Stmt::Delete(DeleteStmt {
            span,
            target,
            where_clause,
        }))
    }

    /// 解析 UPDATE 语句
    pub fn parse_update_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::Update)?;

        // 解析更新目标
        let target =
            if self.match_token(LexerToken::Vertex) || self.match_token(LexerToken::Vertices) {
                // 更新顶点
                let vertex = self.parse_expression()?;
                UpdateTarget::Vertex(vertex)
            } else if self.match_token(LexerToken::Edge) || self.match_token(LexerToken::Edges) {
                // 更新边
                self.parse_update_edge_target()?
            } else if self.match_token(LexerToken::Tag) {
                // 更新标签
                let tag_name = self.expect_identifier()?;
                UpdateTarget::Tag(tag_name)
            } else {
                // 默认更新顶点
                let vertex = self.parse_expression()?;
                UpdateTarget::Vertex(vertex)
            };

        // 解析 SET 子句
        let set_clause = self.parse_set_clause()?;

        // 解析可选的 WHERE 子句
        let where_clause = if self.match_token(LexerToken::Where) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Stmt::Update(UpdateStmt {
            span,
            target,
            set_clause,
            where_clause,
        }))
    }

    /// 解析 GO 语句
    pub fn parse_go_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::Go)?;

        // 解析步数
        let steps = self.parse_steps()?;

        // 解析 FROM 子句
        let from = self.parse_from_clause()?;

        // 解析可选的 OVER 子句
        let over = if self.match_token(LexerToken::Over) {
            Some(self.parse_over_clause()?)
        } else {
            None
        };

        // 解析可选的 WHERE 子句
        let where_clause = if self.match_token(LexerToken::Where) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        // 解析可选的 YIELD 子句
        let yield_clause = if self.match_token(LexerToken::Yield) {
            Some(self.parse_yield_clause()?)
        } else {
            None
        };

        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Stmt::Go(GoStmt {
            span,
            steps,
            from,
            over,
            where_clause,
            yield_clause,
        }))
    }

    /// 解析 FETCH 语句
    pub fn parse_fetch_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::Fetch)?;

        let target =
            if self.match_token(LexerToken::Vertex) || self.match_token(LexerToken::Vertices) {
                // 获取顶点
                self.parse_fetch_vertices_target()?
            } else if self.match_token(LexerToken::Edge) || self.match_token(LexerToken::Edges) {
                // 获取边
                self.parse_fetch_edges_target()?
            } else {
                // 默认获取顶点
                self.parse_fetch_vertices_target()?
            };

        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Stmt::Fetch(FetchStmt { span, target }))
    }

    /// 解析 USE 语句
    pub fn parse_use_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::Use)?;

        let space = self.expect_identifier()?;

        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Stmt::Use(UseStmt { span, space }))
    }

    /// 解析 SHOW 语句
    pub fn parse_show_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::Show)?;

        let target = if self.match_token(LexerToken::Spaces) {
            ShowTarget::Spaces
        } else if self.match_token(LexerToken::Tags) {
            ShowTarget::Tags
        } else if self.match_token(LexerToken::Edges) {
            ShowTarget::Edges
        } else if self.match_token(LexerToken::Tag) {
            let tag_name = self.expect_identifier()?;
            ShowTarget::Tag(tag_name)
        } else if self.match_token(LexerToken::Edge) {
            let edge_name = self.expect_identifier()?;
            ShowTarget::Edge(edge_name)
        } else if self.match_token(LexerToken::Indexes) {
            ShowTarget::Indexes
        } else if self.match_token(LexerToken::Index) {
            let index_name = self.expect_identifier()?;
            ShowTarget::Index(index_name)
        } else if self.match_token(LexerToken::Users) {
            ShowTarget::Users
        } else if self.match_token(LexerToken::Roles) {
            ShowTarget::Roles
        } else {
            let span = self.current_span();
            return Err(ParseError::new(
                "Expected SHOW target".to_string(),
                span.start.line,
                span.start.column,
            ));
        };

        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Stmt::Show(ShowStmt { span, target }))
    }

    /// 解析 EXPLAIN 语句
    pub fn parse_explain_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::Explain)?;

        let statement = Box::new(self.parse_go_statement()?);

        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Stmt::Explain(ExplainStmt { span, statement }))
    }

    /// 解析 LOOKUP 语句
    pub fn parse_lookup_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::Lookup)?;

        let target = if self.match_token(LexerToken::On) {
            let name = self.expect_identifier()?;
            if self.match_token(LexerToken::Tag) {
                LookupTarget::Tag(name)
            } else if self.match_token(LexerToken::Edge) {
                LookupTarget::Edge(name)
            } else {
                LookupTarget::Tag(name)
            }
        } else {
            let span = self.current_span();
            return Err(ParseError::new(
                "Expected LOOKUP target".to_string(),
                span.start.line,
                span.start.column,
            ));
        };

        // 解析可选的 WHERE 子句
        let where_clause = if self.match_token(LexerToken::Where) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        // 解析可选的 YIELD 子句
        let yield_clause = if self.match_token(LexerToken::Yield) {
            Some(self.parse_yield_clause()?)
        } else {
            None
        };

        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Stmt::Lookup(LookupStmt {
            span,
            target,
            where_clause,
            yield_clause,
        }))
    }

    /// 解析 SUBGRAPH 语句
    pub fn parse_subgraph_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::Subgraph)?;

        // 解析步数
        let steps = self.parse_steps()?;

        // 解析 FROM 子句
        let from = self.parse_from_clause()?;

        // 解析可选的 OVER 子句
        let over = if self.match_token(LexerToken::Over) {
            Some(self.parse_over_clause()?)
        } else {
            None
        };

        // 解析可选的 WHERE 子句
        let where_clause = if self.match_token(LexerToken::Where) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        // 解析可选的 YIELD 子句
        let yield_clause = if self.match_token(LexerToken::Yield) {
            Some(self.parse_yield_clause()?)
        } else {
            None
        };

        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Stmt::Subgraph(SubgraphStmt {
            span,
            steps,
            from,
            over,
            where_clause,
            yield_clause,
        }))
    }

    /// 解析 FIND PATH 语句
    pub fn parse_find_path_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::FindPath)?;

        // 解析 FROM 子句
        let from = self.parse_from_clause()?;

        // 解析 TO 表达式
        let to = self.parse_expression()?;

        // 解析可选的 OVER 子句
        let over = if self.match_token(LexerToken::Over) {
            Some(self.parse_over_clause()?)
        } else {
            None
        };

        // 解析可选的 WHERE 子句
        let where_clause = if self.match_token(LexerToken::Where) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        // 检查是否是最短路径
        let shortest = self.match_token(LexerToken::Shortest);

        // 解析可选的 YIELD 子句
        let yield_clause = if self.match_token(LexerToken::Yield) {
            Some(self.parse_yield_clause()?)
        } else {
            None
        };

        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Stmt::FindPath(FindPathStmt {
            span,
            from,
            to,
            over,
            where_clause,
            shortest,
            yield_clause,
        }))
    }

    /// 辅助解析方法

    fn parse_patterns(&mut self) -> Result<Vec<Pattern>, ParseError> {
        let mut patterns = Vec::new();

        loop {
            let pattern = self.parse_pattern()?;
            patterns.push(pattern);

            if !self.match_token(LexerToken::Comma) {
                break;
            }
        }

        Ok(patterns)
    }

    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        // 简化的模式解析 - 这里应该调用模式解析器
        // 暂时返回一个简单的节点模式
        let span = self.current_span();
        Ok(PatternFactory::simple_node(None, vec![], span))
    }

    fn parse_return_clause(&mut self) -> Result<ReturnClause, ParseError> {
        let span = self.current_span();
        let mut items = Vec::new();
        let mut distinct = false;

        // 检查 DISTINCT
        if self.match_token(LexerToken::Distinct) {
            distinct = true;
        }

        // 解析返回项
        loop {
            let expr = self.parse_expression()?;
            let alias = if self.match_token(LexerToken::As) {
                Some(self.expect_identifier()?)
            } else {
                None
            };

            items.push(ReturnItem::Expression { expr, alias });

            if !self.match_token(LexerToken::Comma) {
                break;
            }
        }

        Ok(ReturnClause {
            span,
            items,
            distinct,
        })
    }

    fn parse_order_by_clause(&mut self) -> Result<OrderByClause, ParseError> {
        let span = self.current_span();
        let mut items = Vec::new();

        loop {
            let expr = self.parse_expression()?;
            let direction = if self.match_token(LexerToken::Asc) {
                OrderDirection::Asc
            } else if self.match_token(LexerToken::Desc) {
                OrderDirection::Desc
            } else {
                OrderDirection::Asc // 默认升序
            };

            items.push(OrderByItem { expr, direction });

            if !self.match_token(LexerToken::Comma) {
                break;
            }
        }

        Ok(OrderByClause { span, items })
    }

    fn parse_steps(&mut self) -> Result<Steps, ParseError> {
        let token = self.lexer.peek()?;
        if matches!(token.kind, LexerToken::IntegerLiteral(_)) {
            let steps = self.parse_integer()? as usize;
            Ok(Steps::Fixed(steps))
        } else if matches!(token.kind, LexerToken::Identifier(_)) {
            let var_name = self.expect_identifier()?;
            Ok(Steps::Variable(var_name))
        } else {
            // 默认步数为 1
            Ok(Steps::Fixed(1))
        }
    }

    fn parse_from_clause(&mut self) -> Result<FromClause, ParseError> {
        let span = self.current_span();
        self.expect_token(LexerToken::From)?;

        let mut vertices = Vec::new();

        loop {
            let vertex = self.parse_expression()?;
            vertices.push(vertex);

            if !self.match_token(LexerToken::Comma) {
                break;
            }
        }

        Ok(FromClause { span, vertices })
    }

    fn parse_over_clause(&mut self) -> Result<OverClause, ParseError> {
        let span = self.current_span();

        let mut edge_types = Vec::new();
        let mut direction = EdgeDirection::Outgoing;

        // 解析边类型
        loop {
            let edge_type = self.expect_identifier()?;
            edge_types.push(edge_type);

            if !self.match_token(LexerToken::Comma) {
                break;
            }
        }

        // 解析方向
        if self.match_token(LexerToken::Out) {
            direction = EdgeDirection::Outgoing;
        } else if self.match_token(LexerToken::In) {
            direction = EdgeDirection::Incoming;
        } else if self.match_token(LexerToken::Both) {
            direction = EdgeDirection::Both;
        }

        Ok(OverClause {
            span,
            edge_types,
            direction,
        })
    }

    fn parse_yield_clause(&mut self) -> Result<YieldClause, ParseError> {
        let span = self.current_span();
        let mut items = Vec::new();

        loop {
            let expr = self.parse_expression()?;
            let alias = if self.match_token(LexerToken::As) {
                Some(self.expect_identifier()?)
            } else {
                None
            };

            items.push(YieldItem { expr, alias });

            if !self.match_token(LexerToken::Comma) {
                break;
            }
        }

        Ok(YieldClause { span, items })
    }

    fn parse_set_clause(&mut self) -> Result<SetClause, ParseError> {
        let span = self.current_span();
        self.expect_token(LexerToken::Set)?;

        let mut assignments = Vec::new();

        loop {
            let property = self.expect_identifier()?;
            self.expect_token(LexerToken::Assign)?;
            let value = self.parse_expression()?;

            assignments.push(Assignment { property, value });

            if !self.match_token(LexerToken::Comma) {
                break;
            }
        }

        Ok(SetClause { span, assignments })
    }

    fn parse_vertex_list(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut vertices = Vec::new();

        // 解析顶点列表
        self.expect_token(LexerToken::LParen)?;

        if !self.check_token(LexerToken::RParen) {
            loop {
                let vertex = self.parse_expression()?;
                vertices.push(vertex);

                if !self.match_token(LexerToken::Comma) {
                    break;
                }
            }
        }

        self.expect_token(LexerToken::RParen)?;

        Ok(vertices)
    }

    fn parse_delete_edge_target(&mut self) -> Result<DeleteTarget, ParseError> {
        // 解析源顶点
        let src = self.parse_expression()?;

        // 解析目标顶点
        let dst = self.parse_expression()?;

        // 解析可选的边类型
        let edge_type = if self.match_token(LexerToken::Edge) {
            Some(self.expect_identifier()?)
        } else {
            None
        };

        // 解析可选的排名
        let rank = if self.match_token(LexerToken::Rank) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        Ok(DeleteTarget::Edges {
            src,
            dst,
            edge_type,
            rank,
        })
    }

    fn parse_update_edge_target(&mut self) -> Result<UpdateTarget, ParseError> {
        // 解析源顶点
        let src = self.parse_expression()?;

        // 解析目标顶点
        let dst = self.parse_expression()?;

        // 解析可选的边类型
        let edge_type = if self.match_token(LexerToken::Edge) {
            Some(self.expect_identifier()?)
        } else {
            None
        };

        // 解析可选的排名
        let rank = if self.match_token(LexerToken::Rank) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        Ok(UpdateTarget::Edge {
            src,
            dst,
            edge_type,
            rank,
        })
    }

    fn parse_fetch_vertices_target(&mut self) -> Result<FetchTarget, ParseError> {
        // 解析顶点 ID 列表
        let ids = self.parse_vertex_list()?;

        // 解析可选的属性列表
        let properties = if self.match_token(LexerToken::Prop) {
            Some(self.parse_property_list()?)
        } else {
            None
        };

        Ok(FetchTarget::Vertices { ids, properties })
    }

    fn parse_fetch_edges_target(&mut self) -> Result<FetchTarget, ParseError> {
        // 解析源顶点
        let src = self.parse_expression()?;

        // 解析目标顶点
        let dst = self.parse_expression()?;

        // 解析边类型
        let edge_type = self.expect_identifier()?;

        // 解析可选的排名
        let rank = if self.match_token(LexerToken::Rank) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        // 解析可选的属性列表
        let properties = if self.match_token(LexerToken::Prop) {
            Some(self.parse_property_list()?)
        } else {
            None
        };

        Ok(FetchTarget::Edges {
            src,
            dst,
            edge_type,
            rank,
            properties,
        })
    }

    fn parse_property_list(&mut self) -> Result<Vec<String>, ParseError> {
        let mut properties = Vec::new();

        self.expect_token(LexerToken::LParen)?;

        if !self.check_token(LexerToken::RParen) {
            loop {
                let property = self.expect_identifier()?;
                properties.push(property);

                if !self.match_token(LexerToken::Comma) {
                    break;
                }
            }
        }

        self.expect_token(LexerToken::RParen)?;

        Ok(properties)
    }

    fn parse_create_tag_statement(&mut self, start_span: Span) -> Result<Stmt, ParseError> {
        let tag_name = self.expect_identifier()?;

        // 解析属性定义
        let properties = self.parse_property_definitions()?;

        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Stmt::Create(CreateStmt {
            span,
            target: CreateTarget::Tag {
                name: tag_name,
                properties,
            },
        }))
    }

    fn parse_create_index_statement(&mut self, start_span: Span) -> Result<Stmt, ParseError> {
        let index_name = self.expect_identifier()?;

        self.expect_token(LexerToken::On)?;
        let on = self.expect_identifier()?;

        // 解析属性列表
        let properties = self.parse_property_list()?;

        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Stmt::Create(CreateStmt {
            span,
            target: CreateTarget::Index {
                name: index_name,
                on,
                properties,
            },
        }))
    }

    fn parse_create_node_or_edge_statement(
        &mut self,
        _start_span: Span,
    ) -> Result<Stmt, ParseError> {
        // 简化的节点/边创建解析
        // 这里应该解析完整的模式语法
        let span = self.current_span();

        // 临时实现：创建简单的节点
        Ok(Stmt::Create(CreateStmt {
            span,
            target: CreateTarget::Node {
                variable: None,
                labels: vec![],
                properties: None,
            },
        }))
    }

    fn parse_property_definitions(&mut self) -> Result<Vec<PropertyDef>, ParseError> {
        let mut properties = Vec::new();

        self.expect_token(LexerToken::LParen)?;

        if !self.check_token(LexerToken::RParen) {
            loop {
                let name = self.expect_identifier()?;
                let data_type = self.parse_data_type()?;
                let nullable = !self.match_token(LexerToken::Not);
                if !nullable {
                    self.expect_token(LexerToken::Null)?;
                }

                let default = if self.match_token(LexerToken::Default) {
                    Some(self.parse_expression()?)
                } else {
                    None
                };

                properties.push(PropertyDef {
                    name,
                    data_type,
                    nullable,
                    default: default.map(|_expr| {
                        // 这里应该将表达式转换为值
                        // 临时使用空值
                        Value::Null(crate::core::NullType::Null)
                    }),
                });

                if !self.match_token(LexerToken::Comma) {
                    break;
                }
            }
        }

        self.expect_token(LexerToken::RParen)?;

        Ok(properties)
    }

    /// 辅助方法

    fn match_token(&mut self, expected: LexerToken) -> bool {
        if self.lexer.check(expected) {
            let _ = self.lexer.advance();
            true
        } else {
            false
        }
    }

    fn check_token(&mut self, expected: LexerToken) -> bool {
        self.lexer.check(expected)
    }

    fn expect_token(&mut self, expected: LexerToken) -> Result<(), ParseError> {
        let token = self.lexer.peek()?;
        if token.kind == expected {
            self.lexer.advance();
            Ok(())
        } else {
            let span = self.current_span();
            Err(ParseError::new(
                format!("Expected {:?}, found {:?}", expected, token.kind),
                span.start.line,
                span.start.column,
            ))
        }
    }

    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        let token = self.lexer.peek()?;
        if let LexerToken::Identifier(name) = &token.kind {
            let text = name.clone();
            let _ = self.lexer.advance();
            Ok(text)
        } else {
            let span = self.current_span();
            Err(ParseError::new(
                format!("Expected identifier, found {:?}", token.kind),
                span.start.line,
                span.start.column,
            ))
        }
    }

    fn parse_integer(&mut self) -> Result<i64, ParseError> {
        let token = self.lexer.peek()?;
        if let LexerToken::IntegerLiteral(value) = token.kind {
            let _ = self.lexer.advance();
            Ok(value)
        } else {
            let span = self.current_span();
            Err(ParseError::new(
                format!("Expected integer, found {:?}", token.kind),
                span.start.line,
                span.start.column,
            ))
        }
    }

    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        // 使用表达式解析器
        // 由于lexer.input是私有的，这里需要重新设计
        // 暂时返回一个简单的变量表达式
        let name = self.expect_identifier()?;
        let span = self.current_span();
        Ok(Expr::Variable(VariableExpr::new(name, span)))
    }

    fn current_span(&self) -> Span {
        let pos = self.lexer.current_position();
        Span::new(
            Position::new(pos.line, pos.column),
            Position::new(pos.line, pos.column),
        )
    }

    fn parse_data_type(&mut self) -> Result<DataType, ParseError> {
        let token = self.lexer.peek()?;
        let data_type = match token.kind {
            LexerToken::Int => DataType::Int,
            LexerToken::Float => DataType::Float,
            LexerToken::String => DataType::String,
            LexerToken::Bool => DataType::Bool,
            LexerToken::Date => DataType::Date,
            LexerToken::Datetime => DataType::DateTime,
            LexerToken::Time => DataType::Time,
            LexerToken::Duration => DataType::Duration,
            LexerToken::List => DataType::List,
            LexerToken::Map => DataType::Map,
            _ => {
                let span = self.current_span();
                return Err(ParseError::new(
                    format!("Expected data type, found {:?}", token.kind),
                    span.start.line,
                    span.start.column,
                ));
            }
        };

        let _ = self.lexer.advance();
        Ok(data_type)
    }
}
