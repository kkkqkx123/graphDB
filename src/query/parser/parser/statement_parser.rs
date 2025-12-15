//! 语句解析模块
//!
//! 负责解析各种语句，包括查询语句、创建语句、删除语句等。

use crate::core::Value;
use crate::query::parser::ast::expr::*;
use crate::query::parser::ast::stmt::*;
use crate::query::parser::ast::types::*;
use crate::query::parser::lexer::TokenKind as LexerToken;

impl super::Parser {
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
                    .map_err(|e| ParseError::new(e.message, self.current_span()))
                    .and_then(|opt| {
                        opt.ok_or_else(|| {
                            ParseError::new(
                                "Failed to parse MATCH statement".to_string(),
                                self.current_span(),
                            )
                        })
                    })
            }
            LexerToken::Create => {
                self.next_token();
                self.parse_create_statement()
                    .map_err(|e| ParseError::new(e.message, self.current_span()))
                    .and_then(|opt| {
                        opt.ok_or_else(|| {
                            ParseError::new(
                                "Failed to parse CREATE statement".to_string(),
                                self.current_span(),
                            )
                        })
                    })
            }
            LexerToken::Delete => {
                self.next_token();
                self.parse_delete_statement()
                    .map_err(|e| ParseError::new(e.message, self.current_span()))
                    .and_then(|opt| {
                        opt.ok_or_else(|| {
                            ParseError::new(
                                "Failed to parse DELETE statement".to_string(),
                                self.current_span(),
                            )
                        })
                    })
            }
            LexerToken::Update => {
                self.next_token();
                self.parse_update_statement()
                    .map_err(|e| ParseError::new(e.message, self.current_span()))
                    .and_then(|opt| {
                        opt.ok_or_else(|| {
                            ParseError::new(
                                "Failed to parse UPDATE statement".to_string(),
                                self.current_span(),
                            )
                        })
                    })
            }
            LexerToken::Use => {
                self.next_token();
                self.parse_use_statement()
                    .map_err(|e| ParseError::new(e.message, self.current_span()))
                    .and_then(|opt| {
                        opt.ok_or_else(|| {
                            ParseError::new(
                                "Failed to parse USE statement".to_string(),
                                self.current_span(),
                            )
                        })
                    })
            }
            LexerToken::Show => {
                self.next_token();
                self.parse_show_statement()
                    .map_err(|e| ParseError::new(e.message, self.current_span()))
                    .and_then(|opt| {
                        opt.ok_or_else(|| {
                            ParseError::new(
                                "Failed to parse SHOW statement".to_string(),
                                self.current_span(),
                            )
                        })
                    })
            }
            LexerToken::Explain => {
                self.next_token();
                self.parse_explain_statement()
                    .map_err(|e| ParseError::new(e.message, self.current_span()))
                    .and_then(|opt| {
                        opt.ok_or_else(|| {
                            ParseError::new(
                                "Failed to parse EXPLAIN statement".to_string(),
                                self.current_span(),
                            )
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
                    .map_err(|e| ParseError::new(e.message, self.current_span()))
                    .and_then(|opt| {
                        opt.ok_or_else(|| {
                            ParseError::new(
                                "Failed to parse MATCH statement".to_string(),
                                self.current_span(),
                            )
                        })
                    })
            }
            LexerToken::Create => {
                self.next_token();
                self.parse_create_statement()
                    .map_err(|e| ParseError::new(e.message, self.current_span()))
                    .and_then(|opt| {
                        opt.ok_or_else(|| {
                            ParseError::new(
                                "Failed to parse CREATE statement".to_string(),
                                self.current_span(),
                            )
                        })
                    })
            }
            LexerToken::Delete => {
                self.next_token();
                self.parse_delete_statement()
                    .map_err(|e| ParseError::new(e.message, self.current_span()))
                    .and_then(|opt| {
                        opt.ok_or_else(|| {
                            ParseError::new(
                                "Failed to parse DELETE statement".to_string(),
                                self.current_span(),
                            )
                        })
                    })
            }
            LexerToken::Update => {
                self.next_token();
                self.parse_update_statement()
                    .map_err(|e| ParseError::new(e.message, self.current_span()))
                    .and_then(|opt| {
                        opt.ok_or_else(|| {
                            ParseError::new(
                                "Failed to parse UPDATE statement".to_string(),
                                self.current_span(),
                            )
                        })
                    })
            }
            LexerToken::Use => {
                self.next_token();
                self.parse_use_statement()
                    .map_err(|e| ParseError::new(e.message, self.current_span()))
                    .and_then(|opt| {
                        opt.ok_or_else(|| {
                            ParseError::new(
                                "Failed to parse USE statement".to_string(),
                                self.current_span(),
                            )
                        })
                    })
            }
            LexerToken::Show => {
                self.next_token();
                self.parse_show_statement()
                    .map_err(|e| ParseError::new(e.message, self.current_span()))
                    .and_then(|opt| {
                        opt.ok_or_else(|| {
                            ParseError::new(
                                "Failed to parse SHOW statement".to_string(),
                                self.current_span(),
                            )
                        })
                    })
            }
            LexerToken::Explain => {
                self.next_token();
                self.parse_explain_statement()
                    .map_err(|e| ParseError::new(e.message, self.current_span()))
                    .and_then(|opt| {
                        opt.ok_or_else(|| {
                            ParseError::new(
                                "Failed to parse EXPLAIN statement".to_string(),
                                self.current_span(),
                            )
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
                let error = ParseError::new(
                    format!(
                        "Expected VERTEX or EDGE after CREATE, got {:?}",
                        self.current_token.kind
                    ),
                    self.current_span(),
                );
                Err(error)
            }
        }
    }

    /// 解析 CREATE NODE 语句
    fn parse_create_node_statement(&mut self) -> Result<Option<Stmt>, ParseError> {
        // 简化实现，直接解析 TAG 创建
        let tag_name = self.parse_identifier()?;

        Ok(Some(Stmt::Create(CreateStmt {
            span: self.current_span(),
            target: CreateTarget::Tag {
                name: tag_name,
                properties: vec![],
            },
        })))
    }

    /// 解析 CREATE EDGE 语句
    fn parse_create_edge_statement(&mut self) -> Result<Option<Stmt>, ParseError> {
        // 简化实现，直接解析 EDGE 创建
        let edge_type = self.parse_identifier()?;

        Ok(Some(Stmt::Create(CreateStmt {
            span: self.current_span(),
            target: CreateTarget::Edge {
                variable: None,
                edge_type,
                src: Expr::Constant(ConstantExpr::new(
                    Value::Null(crate::core::NullType::Null),
                    Span::default(),
                )),
                dst: Expr::Constant(ConstantExpr::new(
                    Value::Null(crate::core::NullType::Null),
                    Span::default(),
                )),
                properties: None,
                direction: EdgeDirection::Out,
            },
        })))
    }

    /// 解析 MATCH 语句
    pub fn parse_match_statement(&mut self) -> Result<Option<Stmt>, ParseError> {
        // 简化实现，创建一个空的 MATCH 语句
        let match_stmt = MatchStmt {
            span: self.current_span(),
            patterns: vec![],
            where_clause: None,
            return_clause: None,
            order_by: None,
            limit: None,
            skip: None,
        };

        Ok(Some(Stmt::Match(match_stmt)))
    }

    /// 解析 DELETE 语句
    pub fn parse_delete_statement(&mut self) -> Result<Option<Stmt>, ParseError> {
        // 简化实现，创建一个空的 DELETE 语句
        let delete_stmt = DeleteStmt {
            span: self.current_span(),
            target: DeleteTarget::Vertices(vec![]),
            where_clause: None,
        };

        Ok(Some(Stmt::Delete(delete_stmt)))
    }

    /// 解析 UPDATE 语句
    pub fn parse_update_statement(&mut self) -> Result<Option<Stmt>, ParseError> {
        // 简化实现，创建一个空的 UPDATE 语句
        let update_stmt = UpdateStmt {
            span: self.current_span(),
            target: UpdateTarget::Vertex(Expr::Constant(ConstantExpr::new(
                Value::Null(crate::core::NullType::Null),
                Span::default(),
            ))),
            set_clause: SetClause {
                span: self.current_span(),
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
            span: self.current_span(),
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
                return Err(ParseError::new(
                    format!(
                        "Unexpected token in SHOW statement: {:?}",
                        self.current_token.kind
                    ),
                    self.current_span(),
                ));
            }
        };

        Ok(Some(Stmt::Show(ShowStmt {
            span: self.current_span(),
            target: show_stmt,
        })))
    }

    /// 解析 EXPLAIN 语句
    pub fn parse_explain_statement(&mut self) -> Result<Option<Stmt>, ParseError> {
        self.next_token(); // Skip EXPLAIN

        // Parse the statement to explain
        let stmt = self.parse_statement()?;
        Ok(Some(Stmt::Explain(ExplainStmt {
            span: self.current_span(),
            statement: Box::new(stmt),
        })))
    }

    /// 跳过可选的分号
    fn skip_optional_semicolon(&mut self) {
        self.match_token(LexerToken::Semicolon);
    }
}