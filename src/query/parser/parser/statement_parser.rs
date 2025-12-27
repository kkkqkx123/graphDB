//! 语句解析模块
//!
//! 负责解析各种语句，包括查询语句、创建语句、删除语句等。

use crate::core::Value;
use crate::query::parser::ast::expr::*;
use crate::query::parser::ast::pattern::{NodePattern, PathElement, PathPattern, Pattern};
use crate::query::parser::ast::stmt::PropertyDef;
use crate::query::parser::ast::stmt::*;
use crate::query::parser::ast::types::*;
use crate::query::parser::lexer::TokenKind as LexerToken;

impl super::Parser {
    /// 获取当前位置的行和列
    fn current_position(&self) -> (usize, usize) {
        let pos = self.lexer.current_position();
        (pos.line, pos.column)
    }

    /// 创建ParseError
    fn parse_error(&self, message: String) -> ParseError {
        let (line, column) = self.current_position();
        ParseError::new(message, line, column)
    }
    /// 解析完整的查询
    pub fn parse_query(&mut self) -> Result<QueryStmt, ParseError> {
        let start_pos = self.lexer.current_position();
        let mut statements = Vec::new();

        // 解析多个语句
        while !self.lexer.is_at_end() {
            let stmt = self.parse_statement_impl()?;
            statements.push(stmt);

            // 跳过可选的分号
            self.skip_optional_semicolon();
        }

        let end_pos = self.lexer.current_position();
        let span = Span::new(
            Position::new(start_pos.line, start_pos.column),
            Position::new(end_pos.line, end_pos.column),
        );

        Ok(QueryStmt::new(statements, span))
    }

    /// 解析单个语句（内部实现）
    fn parse_statement_impl(&mut self) -> Result<Stmt, ParseError> {
        let token = self.lexer.peek()?;

        match token.kind {
            LexerToken::Match => {
                self.next_token();
                self.parse_match_statement()
                    .map_err(|e| self.parse_error(e.message))
                    .and_then(|opt| {
                        opt.ok_or_else(|| {
                            self.parse_error("Failed to parse MATCH statement".to_string())
                        })
                    })
            }
            LexerToken::Create => {
                self.next_token();
                self.parse_create_statement()
                    .map_err(|e| self.parse_error(e.message))
                    .and_then(|opt| {
                        opt.ok_or_else(|| {
                            self.parse_error("Failed to parse CREATE statement".to_string())
                        })
                    })
            }
            LexerToken::Delete => {
                self.next_token();
                self.parse_delete_statement()
                    .map_err(|e| self.parse_error(e.message))
                    .and_then(|opt| {
                        opt.ok_or_else(|| {
                            self.parse_error("Failed to parse DELETE statement".to_string())
                        })
                    })
            }
            LexerToken::Update => {
                self.next_token();
                self.parse_update_statement()
                    .map_err(|e| self.parse_error(e.message))
                    .and_then(|opt| {
                        opt.ok_or_else(|| {
                            self.parse_error("Failed to parse UPDATE statement".to_string())
                        })
                    })
            }
            LexerToken::Use => {
                self.next_token();
                self.parse_use_statement()
                    .map_err(|e| self.parse_error(e.message))
                    .and_then(|opt| {
                        opt.ok_or_else(|| {
                            self.parse_error("Failed to parse USE statement".to_string())
                        })
                    })
            }
            LexerToken::Show => {
                self.next_token();
                self.parse_show_statement()
                    .map_err(|e| self.parse_error(e.message))
                    .and_then(|opt| {
                        opt.ok_or_else(|| {
                            self.parse_error("Failed to parse SHOW statement".to_string())
                        })
                    })
            }
            LexerToken::Explain => {
                self.next_token();
                self.parse_explain_statement()
                    .map_err(|e| self.parse_error(e.message))
                    .and_then(|opt| {
                        opt.ok_or_else(|| {
                            self.parse_error("Failed to parse EXPLAIN statement".to_string())
                        })
                    })
            }
            _ => {
                // 如果不是关键字，尝试解析为表达式语句
                let expr = self.parse_expression()?;
                let span = expr.span();
                Ok(Stmt::Query(QueryStmt::new(vec![], span)))
            }
        }
    }

    /// 解析单个语句
    pub fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        let token = self.lexer.peek()?;

        match token.kind {
            LexerToken::Match => {
                self.next_token();
                self.parse_match_statement()
                    .map_err(|e| self.parse_error(e.message))
                    .and_then(|opt| {
                        opt.ok_or_else(|| {
                            self.parse_error("Failed to parse MATCH statement".to_string())
                        })
                    })
            }
            LexerToken::Create => {
                self.next_token();
                self.parse_create_statement()
                    .map_err(|e| self.parse_error(e.message))
                    .and_then(|opt| {
                        opt.ok_or_else(|| {
                            self.parse_error("Failed to parse CREATE statement".to_string())
                        })
                    })
            }
            LexerToken::Delete => {
                self.next_token();
                self.parse_delete_statement()
                    .map_err(|e| self.parse_error(e.message))
                    .and_then(|opt| {
                        opt.ok_or_else(|| {
                            self.parse_error("Failed to parse DELETE statement".to_string())
                        })
                    })
            }
            LexerToken::Update => {
                self.next_token();
                self.parse_update_statement()
                    .map_err(|e| self.parse_error(e.message))
                    .and_then(|opt| {
                        opt.ok_or_else(|| {
                            self.parse_error("Failed to parse UPDATE statement".to_string())
                        })
                    })
            }
            LexerToken::Use => {
                self.next_token();
                self.parse_use_statement()
                    .map_err(|e| self.parse_error(e.message))
                    .and_then(|opt| {
                        opt.ok_or_else(|| {
                            self.parse_error("Failed to parse USE statement".to_string())
                        })
                    })
            }
            LexerToken::Show => {
                self.next_token();
                self.parse_show_statement()
                    .map_err(|e| self.parse_error(e.message))
                    .and_then(|opt| {
                        opt.ok_or_else(|| {
                            self.parse_error("Failed to parse SHOW statement".to_string())
                        })
                    })
            }
            LexerToken::Explain => {
                self.next_token();
                self.parse_explain_statement()
                    .map_err(|e| self.parse_error(e.message))
                    .and_then(|opt| {
                        opt.ok_or_else(|| {
                            self.parse_error("Failed to parse EXPLAIN statement".to_string())
                        })
                    })
            }
            _ => {
                // 如果不是关键字，尝试解析为表达式语句
                let expr = self.parse_expression()?;
                let span = expr.span();
                Ok(Stmt::Query(QueryStmt::new(vec![], span)))
            }
        }
    }

    /// 解析 CREATE 语句
    pub fn parse_create_statement(&mut self) -> Result<Option<Stmt>, ParseError> {
        match self.current_token.kind {
            LexerToken::Vertex | LexerToken::Vertices => {
                self.next_token();
                self.parse_create_node_statement()
            }
            LexerToken::Edge | LexerToken::Edges => {
                self.next_token();
                self.parse_create_edge_statement()
            }
            _ => {
                let span = self.parser_current_span();
                let error = ParseError::new(
                    format!(
                        "Expected VERTEX or EDGE after CREATE, got {:?}",
                        self.current_token.kind
                    ),
                    span.start.line,
                    span.start.column,
                );
                Err(error)
            }
        }
    }

    /// 解析 CREATE NODE 语句
    fn parse_create_node_statement(&mut self) -> Result<Option<Stmt>, ParseError> {
        let _if_not_exists = self.check_and_skip_keyword(LexerToken::If);

        // Skip 'EXISTS' if we found 'IF'
        if _if_not_exists {
            self.expect_token(LexerToken::Exists)?;
        }

        // Parse tag list
        let tags = self.parse_tag_list()?;

        // Parse SET clause
        self.expect_token(LexerToken::Set)?;

        // Properties can be in two forms: SET prop = value or SET {prop: value}
        let properties = if self.current_token.kind == LexerToken::LBrace {
            // Handle SET {prop: value} form
            let map = self.parse_property_map()?;
            // Convert HashMap to Vec<Property>
            let props: Vec<PropertyDef> = map
                .into_iter()
                .map(|(name, _value)| PropertyDef {
                    name,
                    data_type: DataType::String,
                    nullable: false,
                    default: None,
                })
                .collect();
            props
        } else {
            // Handle SET prop = value form
            self.parse_property_list()?
        };

        // Optionally parse YIELD clause
        let _yield_clause = if self.current_token.kind == LexerToken::Yield {
            Some(self.parse_yield_clause()?)
        } else {
            None
        };

        Ok(Some(Stmt::Create(CreateStmt {
            span: self.parser_current_span(),
            target: CreateTarget::Tag {
                name: tags.join(":"),
                properties,
            },
        })))
    }

    /// 解析 CREATE EDGE 语句
    fn parse_create_edge_statement(&mut self) -> Result<Option<Stmt>, ParseError> {
        let _if_not_exists = self.check_and_skip_keyword(LexerToken::If);

        // Skip 'EXISTS' if we found 'IF'
        if _if_not_exists {
            self.expect_token(LexerToken::Exists)?;
        }

        // Parse edge type
        let edge_type = self.parse_identifier()?;

        // Parse source and destination
        self.expect_token(LexerToken::LParen)?;
        let src = self.parse_expression()?;
        self.expect_token(LexerToken::RParen)?;

        // Parse edge pattern -> or <-
        let direction = if self.current_token.kind == LexerToken::Arrow {
            self.next_token();
            EdgeDirection::Outgoing
        } else if self.current_token.kind == LexerToken::BackArrow {
            self.next_token();
            EdgeDirection::Incoming
        } else {
            let span = self.parser_current_span();
            return Err(ParseError::new(
                format!("Expected -> or <-, got {:?}", self.current_token.kind),
                span.start.line,
                span.start.column,
            ));
        };

        self.expect_token(LexerToken::LParen)?;
        let dst = self.parse_expression()?;
        self.expect_token(LexerToken::RParen)?;

        // Parse SET clause
        self.expect_token(LexerToken::Set)?;
        let properties = self.parse_property_list()?;

        // Optionally parse YIELD clause
        let _yield_clause = if self.current_token.kind == LexerToken::Yield {
            Some(self.parse_yield_clause()?)
        } else {
            None
        };

        // Convert Vec<PropertyDef> to Expr if not empty
        let properties_expr = if properties.is_empty() {
            None
        } else {
            // For now, create a simple expression representation
            // In a real implementation, you might create a map/object expression
            Some(Expr::Constant(ConstantExpr::new(
                Value::String(format!("{} properties", properties.len())),
                Span::default(),
            )))
        };

        Ok(Some(Stmt::Create(CreateStmt {
            span: self.parser_current_span(),
            target: CreateTarget::Edge {
                variable: None,
                edge_type,
                src,
                dst,
                properties: properties_expr,
                direction,
            },
        })))
    }

    /// 解析 MATCH 语句
    pub fn parse_match_statement(&mut self) -> Result<Option<Stmt>, ParseError> {
        // Parse match patterns - convert PathElement to Pattern
        let path_elements = self.parse_match_patterns()?;
        let patterns: Vec<Pattern> = path_elements
            .into_iter()
            .map(|elem| {
                // Convert PathElement to Pattern
                match elem {
                    PathElement::Node(node) => Ok(Pattern::Node(node)),
                    PathElement::Edge(edge) => Ok(Pattern::Edge(edge)),
                    PathElement::Alternative(alts) => {
                        // For alternatives, create a PathPattern containing the alternatives
                        let path_pattern = PathPattern::new(
                            alts.into_iter()
                                .map(|pattern| {
                                    // Convert each Pattern back to PathElement
                                    match pattern {
                                        Pattern::Node(node) => PathElement::Node(node),
                                        Pattern::Edge(edge) => PathElement::Edge(edge),
                                        Pattern::Path(path) => {
                                            // Extract first element from path if it's a single element path
                                            if path.elements.len() == 1 {
                                                path.elements.into_iter().next()
                                                    .expect("Path should have at least one element when length is 1")
                                            } else {
                                                // For complex paths, create a simple node pattern as fallback
                                                PathElement::Node(NodePattern::new(
                                                    None,
                                                    vec![],
                                                    None,
                                                    vec![],
                                                    path.span,
                                                ))
                                            }
                                        }
                                        Pattern::Variable(var) => {
                                            // Convert variable pattern to node pattern
                                            PathElement::Node(NodePattern::new(
                                                Some(var.name),
                                                vec![],
                                                None,
                                                vec![],
                                                var.span,
                                            ))
                                        }
                                    }
                                })
                                .collect(),
                            Span::default(),
                        );
                        Ok(Pattern::Path(path_pattern))
                    }
                    PathElement::Optional(opt) => {
                        // Convert Optional PathElement to PathPattern with optional element
                        let path_pattern =
                            PathPattern::new(vec![PathElement::Optional(opt)], Span::default());
                        Ok(Pattern::Path(path_pattern))
                    }
                    PathElement::Repeated(elem, rep_type) => {
                        // Convert Repeated PathElement to PathPattern with repeated element
                        let path_pattern = PathPattern::new(
                            vec![PathElement::Repeated(elem, rep_type)],
                            Span::default(),
                        );
                        Ok(Pattern::Path(path_pattern))
                    }
                }
            })
            .collect::<Result<Vec<Pattern>, ParseError>>()?;

        // Parse optional WHERE clause
        let where_clause = if self.current_token.kind == LexerToken::Where {
            Some(self.parse_where_clause()?)
        } else {
            None
        };

        // Parse optional RETURN clause
        let return_clause = if self.current_token.kind == LexerToken::Return {
            Some(self.parse_return_clause()?)
        } else {
            None
        };

        // Parse optional ORDER BY clause
        let order_by = if self.current_token.kind == LexerToken::Order {
            Some(self.parse_order_by_clause()?)
        } else {
            None
        };

        // Parse optional LIMIT clause - extract numeric value from expression
        let limit = if self.current_token.kind == LexerToken::Limit {
            let expr = self.parse_limit_clause()?;
            // Try to extract a numeric constant from the expression
            if let Expr::Constant(ConstantExpr { value, .. }) = &expr {
                if let Value::Int(n) = value {
                    Some(*n as usize)
                } else {
                    Some(10) // Default limit
                }
            } else {
                Some(10) // Default limit
            }
        } else {
            None
        };

        // Parse optional SKIP clause - extract numeric value from expression
        let skip = if self.current_token.kind == LexerToken::Skip {
            let expr = self.parse_skip_clause()?;
            // Try to extract a numeric constant from the expression
            if let Expr::Constant(ConstantExpr { value, .. }) = &expr {
                if let Value::Int(n) = value {
                    Some(*n as usize)
                } else {
                    Some(0) // Default skip
                }
            } else {
                Some(0) // Default skip
            }
        } else {
            None
        };

        let match_stmt = MatchStmt {
            span: self.parser_current_span(),
            patterns,
            where_clause,
            return_clause,
            order_by,
            limit,
            skip,
        };

        Ok(Some(Stmt::Match(match_stmt)))
    }

    /// 解析 DELETE 语句
    pub fn parse_delete_statement(&mut self) -> Result<Option<Stmt>, ParseError> {
        // 简化实现，创建一个空的 DELETE 语句
        let delete_stmt = DeleteStmt {
            span: self.parser_current_span(),
            target: DeleteTarget::Vertices(vec![]),
            where_clause: None,
        };

        Ok(Some(Stmt::Delete(delete_stmt)))
    }

    /// 解析 UPDATE 语句
    pub fn parse_update_statement(&mut self) -> Result<Option<Stmt>, ParseError> {
        // 简化实现，创建一个空的 UPDATE 语句
        let update_stmt = UpdateStmt {
            span: self.parser_current_span(),
            target: UpdateTarget::Vertex(Expr::Constant(ConstantExpr::new(
                Value::Null(crate::core::NullType::Null),
                Span::default(),
            ))),
            set_clause: SetClause {
                span: self.parser_current_span(),
                assignments: vec![],
            },
            where_clause: None,
        };

        Ok(Some(Stmt::Update(update_stmt)))
    }

    /// 解析 USE 语句
    pub fn parse_use_statement(&mut self) -> Result<Option<Stmt>, ParseError> {
        self.next_token(); // Skip USE
        let space = self.parse_identifier()?;
        Ok(Some(Stmt::Use(UseStmt {
            span: self.parser_current_span(),
            space,
        })))
    }

    /// 解析 SHOW 语句
    pub fn parse_show_statement(&mut self) -> Result<Option<Stmt>, ParseError> {
        self.next_token(); // Skip SHOW

        let show_stmt = match self.current_token.kind {
            LexerToken::Spaces => {
                self.next_token();
                ShowTarget::Spaces
            }
            LexerToken::Tags => {
                self.next_token();
                ShowTarget::Tags
            }
            LexerToken::Edges => {
                self.next_token();
                ShowTarget::Edges
            }
            LexerToken::Tag => {
                self.next_token();
                ShowTarget::Tag("".to_string())
            }
            LexerToken::Edge => {
                self.next_token();
                ShowTarget::Edge("".to_string())
            }
            LexerToken::Users => {
                self.next_token();
                ShowTarget::Users
            }
            LexerToken::Roles => {
                self.next_token();
                ShowTarget::Roles
            }
            _ => {
                let span = self.parser_current_span();
                return Err(ParseError::new(
                    format!(
                        "Unexpected token in SHOW statement: {:?}",
                        self.current_token.kind
                    ),
                    span.start.line,
                    span.start.column,
                ));
            }
        };

        Ok(Some(Stmt::Show(ShowStmt {
            span: self.parser_current_span(),
            target: show_stmt,
        })))
    }

    /// 解析 EXPLAIN 语句
    pub fn parse_explain_statement(&mut self) -> Result<Option<Stmt>, ParseError> {
        self.next_token(); // Skip EXPLAIN

        // Parse the statement to explain
        let stmt = self.parse_statement()?;
        Ok(Some(Stmt::Explain(ExplainStmt {
            span: self.parser_current_span(),
            statement: Box::new(stmt),
        })))
    }

    /// 跳过可选的分号
    fn skip_optional_semicolon(&mut self) {
        self.match_token(LexerToken::Semicolon);
    }
}
