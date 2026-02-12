//! 语句解析模块
//!
//! 负责解析各种语句，包括 MATCH、GO、CREATE、DELETE、UPDATE 等。

use crate::core::types::graph_schema::EdgeDirection;
use crate::core::types::PropertyDef;
use crate::core::types::expression::Expression as CoreExpression;
use crate::query::parser::ast::*;
use crate::query::parser::ast::pattern::{EdgePattern, NodePattern, PathElement, PathPattern};
use crate::query::parser::core::error::{ParseError, ParseErrorKind};
use crate::query::parser::parser::ExprParser;
use crate::query::parser::parser::parse_context::ParseContext;
use crate::query::parser::TokenKind;

pub struct StmtParser<'a> {
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> StmtParser<'a> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    /// 解析表达式并返回 Core Expression
    fn parse_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<CoreExpression, ParseError> {
        let mut expr_parser = ExprParser::new(ctx);
        let result = expr_parser.parse_expression(ctx)?;
        Ok(result.expr)
    }

    pub fn parse_statement(&mut self, ctx: &mut ParseContext<'a>) -> Result<Stmt, ParseError> {
        let token = ctx.current_token().clone();
        match token.kind {
            TokenKind::Match => self.parse_match_statement(ctx),
            TokenKind::Go => self.parse_go_statement(ctx),
            TokenKind::Create => self.parse_create_statement(ctx),
            TokenKind::Delete => self.parse_delete_statement(ctx),
            TokenKind::Update => self.parse_update_statement(ctx),
            TokenKind::Use => self.parse_use_statement(ctx),
            TokenKind::Show => self.parse_show_statement(ctx),
            TokenKind::Explain => self.parse_explain_statement(ctx),
            TokenKind::Lookup => self.parse_lookup_statement(ctx),
            TokenKind::Fetch => self.parse_fetch_statement(ctx),
            TokenKind::Unwind => self.parse_unwind_statement(ctx),
            TokenKind::Merge => self.parse_merge_statement(ctx),
            TokenKind::Insert => self.parse_insert_statement(ctx),
            TokenKind::Return => self.parse_return_statement(ctx),
            TokenKind::With => self.parse_with_statement(ctx),
            TokenKind::Set => self.parse_set_statement(ctx),
            TokenKind::Remove => self.parse_remove_statement(ctx),
            TokenKind::Pipe => self.parse_pipe_statement(ctx),
            TokenKind::Drop => self.parse_drop_statement(ctx),
            TokenKind::Desc => self.parse_desc_statement(ctx),
            TokenKind::Alter => self.parse_alter_statement(ctx),
            TokenKind::CreateUser => self.parse_create_user_statement(ctx),
            TokenKind::AlterUser => self.parse_alter_user_statement(ctx),
            TokenKind::DropUser => self.parse_drop_user_statement(ctx),
            TokenKind::ChangePassword => self.parse_change_password_statement(ctx),
            _ => Err(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                format!("Unexpected token: {:?}", token.kind),
                ctx.current_position(),
            )),
        }
    }

    fn parse_match_statement(&mut self, ctx: &mut ParseContext<'a>) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Match)?;

        let mut patterns = Vec::new();
        patterns.push(self.parse_pattern(ctx)?);

        let where_clause = if ctx.match_token(TokenKind::Where) {
            Some(self.parse_expression(ctx)?)
        } else {
            None
        };

        let return_clause = if ctx.match_token(TokenKind::Return) {
            Some(self.parse_return_clause(ctx)?)
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
        }))
    }

    fn parse_go_statement(&mut self, ctx: &mut ParseContext<'a>) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Go)?;

        let steps = self.parse_steps(ctx)?;

        ctx.expect_token(TokenKind::From)?;
        let from_span = ctx.current_span();
        let vertices = self.parse_expression_list(ctx)?;
        let from_clause = FromClause {
            span: from_span,
            vertices,
        };

        let over = if ctx.match_token(TokenKind::Over) {
            Some(self.parse_over_clause(ctx)?)
        } else {
            None
        };

        let where_clause = if ctx.match_token(TokenKind::Where) {
            Some(self.parse_expression(ctx)?)
        } else {
            None
        };

        let yield_clause = if ctx.match_token(TokenKind::Yield) {
            Some(self.parse_yield_clause(ctx)?)
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

    fn parse_create_statement(&mut self, ctx: &mut ParseContext<'a>) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Create)?;

        if ctx.match_token(TokenKind::Tag) {
            let name = ctx.expect_identifier()?;
            let properties = self.parse_property_defs(ctx)?;
            Ok(Stmt::Create(CreateStmt {
                span: start_span,
                target: CreateTarget::Tag { name, properties },
            }))
        } else if ctx.match_token(TokenKind::Edge) {
            let name = ctx.expect_identifier()?;
            let properties = self.parse_property_defs(ctx)?;
            Ok(Stmt::Create(CreateStmt {
                span: start_span,
                target: CreateTarget::EdgeType { name, properties },
            }))
        } else {
            Err(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                "Expected TAG or EDGE after CREATE".to_string(),
                ctx.current_position(),
            ))
        }
    }

    fn parse_delete_statement(&mut self, ctx: &mut ParseContext<'a>) -> Result<Stmt, ParseError> {
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

        Ok(Stmt::Delete(DeleteStmt {
            span: start_span,
            target,
            where_clause,
        }))
    }

    fn parse_update_statement(&mut self, ctx: &mut ParseContext<'a>) -> Result<Stmt, ParseError> {
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
            self.parse_set_clause(ctx)?
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

    fn parse_use_statement(&mut self, ctx: &mut ParseContext<'a>) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Use)?;

        let space = ctx.expect_identifier()?;

        Ok(Stmt::Use(UseStmt { span: start_span, space }))
    }

    fn parse_show_statement(&mut self, ctx: &mut ParseContext<'a>) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Show)?;

        let target = if ctx.match_token(TokenKind::Spaces) {
            ShowTarget::Spaces
        } else if ctx.match_token(TokenKind::Tags) {
            ShowTarget::Tags
        } else if ctx.match_token(TokenKind::Edges) {
            ShowTarget::Edges
        } else {
            ShowTarget::Spaces
        };

        Ok(Stmt::Show(ShowStmt { span: start_span, target }))
    }

    fn parse_explain_statement(&mut self, ctx: &mut ParseContext<'a>) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Explain)?;

        let statement = Box::new(self.parse_statement(ctx)?);

        Ok(Stmt::Explain(ExplainStmt { span: start_span, statement }))
    }

    fn parse_lookup_statement(&mut self, ctx: &mut ParseContext<'a>) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Lookup)?;

        let target = if ctx.match_token(TokenKind::On) {
            let name = ctx.expect_identifier()?;
            if ctx.match_token(TokenKind::Tag) {
                LookupTarget::Tag(name)
            } else {
                LookupTarget::Tag(name)
            }
        } else {
            LookupTarget::Tag(String::new())
        };

        let where_clause = if ctx.match_token(TokenKind::Where) {
            Some(self.parse_expression(ctx)?)
        } else {
            None
        };

        let yield_clause = if ctx.match_token(TokenKind::Yield) {
            Some(self.parse_yield_clause(ctx)?)
        } else {
            None
        };

        Ok(Stmt::Lookup(LookupStmt {
            span: start_span,
            target,
            where_clause,
            yield_clause,
        }))
    }

    fn parse_fetch_statement(&mut self, ctx: &mut ParseContext<'a>) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Fetch)?;

        let target = if ctx.match_token(TokenKind::Tag) {
            let _tag_name = ctx.expect_identifier()?;
            let ids = self.parse_expression_list(ctx)?;
            FetchTarget::Vertices {
                ids,
                properties: None,
            }
        } else if ctx.match_token(TokenKind::Edge) {
            let edge_type = ctx.expect_identifier()?;
            let src = self.parse_expression(ctx)?;
            ctx.expect_token(TokenKind::Minus)?;
            ctx.expect_token(TokenKind::Gt)?;
            let dst = self.parse_expression(ctx)?;
            let rank = if ctx.match_token(TokenKind::At) {
                Some(self.parse_expression(ctx)?)
            } else {
                None
            };
            FetchTarget::Edges {
                src,
                dst,
                edge_type,
                rank,
                properties: None,
            }
        } else {
            let ids = self.parse_expression_list(ctx)?;
            FetchTarget::Vertices {
                ids,
                properties: None,
            }
        };

        Ok(Stmt::Fetch(FetchStmt {
            span: start_span,
            target,
        }))
    }

    fn parse_unwind_statement(&mut self, ctx: &mut ParseContext<'a>) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Unwind)?;

        let expression = self.parse_expression(ctx)?;
        let variable = ctx.expect_identifier()?;

        Ok(Stmt::Unwind(UnwindStmt {
            span: start_span,
            expression,
            variable,
        }))
    }

    fn parse_merge_statement(&mut self, ctx: &mut ParseContext<'a>) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Merge)?;

        let pattern = self.parse_pattern(ctx)?;

        let on_create = if ctx.match_token(TokenKind::On)
            && ctx.match_token(TokenKind::Create)
        {
            Some(self.parse_set_clause(ctx)?)
        } else {
            None
        };

        let on_match = if ctx.match_token(TokenKind::On) && ctx.match_token(TokenKind::Match) {
            Some(self.parse_set_clause(ctx)?)
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

    fn parse_insert_statement(&mut self, ctx: &mut ParseContext<'a>) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Insert)?;

        let target = if ctx.match_token(TokenKind::Vertex) {
            let tag_name = ctx.expect_identifier()?;
            let prop_names = if ctx.match_token(TokenKind::LParen) {
                self.parse_property_names(ctx)?
            } else {
                Vec::new()
            };
            ctx.expect_token(TokenKind::Values)?;
            let values = self.parse_insert_vertex_values(ctx)?;
            InsertTarget::Vertices {
                tag_name,
                prop_names,
                values,
            }
        } else if ctx.match_token(TokenKind::Edge) {
            let edge_name = ctx.expect_identifier()?;
            let prop_names = if ctx.match_token(TokenKind::LParen) {
                self.parse_property_names(ctx)?
            } else {
                Vec::new()
            };
            ctx.expect_token(TokenKind::Values)?;
            let edges = self.parse_insert_edge_values(ctx)?;
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

        Ok(Stmt::Insert(InsertStmt { span: start_span, target }))
    }

    fn parse_property_names(&mut self, ctx: &mut ParseContext<'a>) -> Result<Vec<String>, ParseError> {
        let mut names = Vec::new();
        // 检查是否是空列表 ()
        if ctx.match_token(TokenKind::RParen) {
            return Ok(names);
        }
        loop {
            names.push(ctx.expect_identifier()?);
            if ctx.match_token(TokenKind::RParen) {
                break;
            }
            ctx.expect_token(TokenKind::Comma)?;
        }
        Ok(names)
    }

    fn parse_insert_vertex_values(
        &mut self,
        ctx: &mut ParseContext<'a>,
    ) -> Result<Vec<(CoreExpression, Vec<CoreExpression>)>, ParseError> {
        let mut values = Vec::new();
        loop {
            let vid = self.parse_expression(ctx)?;
            ctx.expect_token(TokenKind::Colon)?;
            ctx.expect_token(TokenKind::LParen)?;
            let mut prop_values = Vec::new();
            // 如果下一个 token 不是 RParen，则解析属性值
            if ctx.current_token().kind != TokenKind::RParen {
                loop {
                    prop_values.push(self.parse_expression(ctx)?);
                    if !ctx.match_token(TokenKind::Comma) {
                        break;
                    }
                }
            }
            ctx.expect_token(TokenKind::RParen)?;
            values.push((vid, prop_values));
            if !ctx.match_token(TokenKind::Comma) {
                break;
            }
        }
        Ok(values)
    }

    fn parse_insert_edge_values(
        &mut self,
        ctx: &mut ParseContext<'a>,
    ) -> Result<Vec<(CoreExpression, CoreExpression, Option<CoreExpression>, Vec<CoreExpression>)>, ParseError> {
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
            ctx.expect_token(TokenKind::Colon)?;
            ctx.expect_token(TokenKind::LParen)?;
            let mut values = Vec::new();
            // 如果下一个 token 不是 RParen，则解析属性值
            if ctx.current_token().kind != TokenKind::RParen {
                loop {
                    values.push(self.parse_expression(ctx)?);
                    if !ctx.match_token(TokenKind::Comma) {
                        break;
                    }
                }
            }
            ctx.expect_token(TokenKind::RParen)?;
            edges.push((src, dst, rank, values));
            if !ctx.match_token(TokenKind::Comma) {
                break;
            }
        }
        Ok(edges)
    }

    fn parse_return_statement(&mut self, ctx: &mut ParseContext<'a>) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Return)?;

        let items = self.parse_return_items(ctx)?;

        Ok(Stmt::Return(ReturnStmt {
            span: start_span,
            items,
            distinct: false,
        }))
    }

    fn parse_with_statement(&mut self, ctx: &mut ParseContext<'a>) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::With)?;

        let items = self.parse_return_items(ctx)?;

        let where_clause = if ctx.match_token(TokenKind::Where) {
            Some(self.parse_expression(ctx)?)
        } else {
            None
        };

        Ok(Stmt::With(WithStmt {
            span: start_span,
            items,
            where_clause,
        }))
    }

    fn parse_set_statement(&mut self, ctx: &mut ParseContext<'a>) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Set)?;

        let assignments = self.parse_set_assignments(ctx)?;

        Ok(Stmt::Set(SetStmt {
            span: start_span,
            assignments,
        }))
    }

    fn parse_remove_statement(&mut self, ctx: &mut ParseContext<'a>) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Remove)?;

        let mut items = Vec::new();
        if ctx.match_token(TokenKind::Tag) {
            items.push(self.parse_expression(ctx)?);
        }

        Ok(Stmt::Remove(RemoveStmt {
            span: start_span,
            items,
        }))
    }

    fn parse_pipe_statement(&mut self, ctx: &mut ParseContext<'a>) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Pipe)?;

        let expression = self.parse_expression(ctx)?;

        Ok(Stmt::Pipe(PipeStmt {
            span: start_span,
            expression,
        }))
    }

    fn parse_expression_list(&mut self, ctx: &mut ParseContext<'a>) -> Result<Vec<CoreExpression>, ParseError> {
        let mut expressions = Vec::new();
        expressions.push(self.parse_expression(ctx)?);
        while ctx.match_token(TokenKind::Comma) {
            expressions.push(self.parse_expression(ctx)?);
        }
        Ok(expressions)
    }

    fn parse_return_items(&mut self, ctx: &mut ParseContext<'a>) -> Result<Vec<ReturnItem>, ParseError> {
        let mut items = Vec::new();
        loop {
            if ctx.match_token(TokenKind::Star) {
                items.push(ReturnItem::All);
            } else {
                let expression = self.parse_expression(ctx)?;
                let alias = if ctx.match_token(TokenKind::As) {
                    Some(ctx.expect_identifier()?)
                } else {
                    None
                };
                items.push(ReturnItem::Expression { expression, alias });
            }
            if !ctx.match_token(TokenKind::Comma) {
                break;
            }
        }
        Ok(items)
    }

    fn parse_return_clause(&mut self, ctx: &mut ParseContext<'a>) -> Result<ReturnClause, ParseError> {
        ctx.expect_token(TokenKind::Return)?;
        let items = self.parse_return_items(ctx)?;
        Ok(ReturnClause {
            span: ctx.current_span(),
            items,
            distinct: false,
            limit: None,
            skip: None,
            sample: None,
        })
    }

    fn parse_yield_clause(&mut self, ctx: &mut ParseContext<'a>) -> Result<YieldClause, ParseError> {
        ctx.expect_token(TokenKind::Yield)?;
        let items = self.parse_yield_items(ctx)?;
        Ok(YieldClause {
            span: ctx.current_span(),
            items,
            limit: None,
            skip: None,
            sample: None,
        })
    }

    fn parse_yield_items(&mut self, ctx: &mut ParseContext<'a>) -> Result<Vec<YieldItem>, ParseError> {
        let mut items = Vec::new();
        loop {
            let expression = self.parse_expression(ctx)?;
            let alias = if ctx.match_token(TokenKind::As) {
                Some(ctx.expect_identifier()?)
            } else {
                None
            };
            items.push(YieldItem { expression, alias });
            if !ctx.match_token(TokenKind::Comma) {
                break;
            }
        }
        Ok(items)
    }

    fn parse_pattern(&mut self, ctx: &mut ParseContext<'a>) -> Result<Pattern, ParseError> {
        let start_span = ctx.current_span();

        let left = self.parse_node_pattern(ctx)?;

        let mut elements = Vec::new();
        elements.push(PathElement::Node(left));

        while ctx.match_token(TokenKind::Minus) {
            let edge = self.parse_edge_pattern(ctx)?;
            elements.push(PathElement::Edge(edge));

            if ctx.match_token(TokenKind::Gt) {
                let right = self.parse_node_pattern(ctx)?;
                elements.push(PathElement::Node(right));
            }
        }

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        let path_pattern = PathPattern::new(elements, span);
        Ok(Pattern::Path(path_pattern))
    }

    fn parse_node_pattern(&mut self, ctx: &mut ParseContext<'a>) -> Result<NodePattern, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::LParen)?;

        let variable = if !ctx.match_token(TokenKind::Colon) {
            Some(ctx.expect_identifier()?)
        } else {
            None
        };

        let mut labels = Vec::new();
        while ctx.match_token(TokenKind::Colon) {
            labels.push(ctx.expect_identifier()?);
        }

        let properties = if ctx.match_token(TokenKind::LBrace) {
            Some(self.parse_expression(ctx)?)
        } else {
            None
        };

        let predicates = Vec::new();

        ctx.expect_token(TokenKind::RParen)?;

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(NodePattern::new(variable, labels, properties, predicates, span))
    }

    fn parse_edge_pattern(&mut self, ctx: &mut ParseContext<'a>) -> Result<EdgePattern, ParseError> {
        let start_span = ctx.current_span();

        ctx.expect_token(TokenKind::Minus)?;

        let variable = None;
        let edge_types = if ctx.match_token(TokenKind::LBracket) {
            let types = self.parse_edge_type_list(ctx)?;
            ctx.expect_token(TokenKind::RBracket)?;
            types
        } else {
            Vec::new()
        };

        let properties = if ctx.match_token(TokenKind::LBrace) {
            Some(self.parse_expression(ctx)?)
        } else {
            None
        };

        let predicates = Vec::new();
        let direction = if ctx.match_token(TokenKind::Out) {
            EdgeDirection::Out
        } else if ctx.match_token(TokenKind::In) {
            EdgeDirection::In
        } else {
            EdgeDirection::Out
        };
        let range = None;

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(EdgePattern::new(
            variable,
            edge_types,
            properties,
            predicates,
            direction,
            range,
            span,
        ))
    }

    fn parse_steps(&mut self, ctx: &mut ParseContext<'a>) -> Result<Steps, ParseError> {
        if ctx.match_token(TokenKind::Step) {
            if let TokenKind::IntegerLiteral(n) = ctx.current_token().kind {
                ctx.next_token();
                Ok(Steps::Fixed(n as usize))
            } else {
                Ok(Steps::Fixed(1))
            }
        } else {
            Ok(Steps::Fixed(1))
        }
    }

    fn parse_over_clause(&mut self, ctx: &mut ParseContext<'a>) -> Result<OverClause, ParseError> {
        let span = ctx.current_span();
        let edge_types = self.parse_edge_type_list(ctx)?;
        let direction = if ctx.match_token(TokenKind::Out) {
            EdgeDirection::Out
        } else if ctx.match_token(TokenKind::In) {
            EdgeDirection::In
        } else if ctx.match_token(TokenKind::Both) {
            EdgeDirection::Both
        } else {
            EdgeDirection::Out
        };
        Ok(OverClause { span, edge_types, direction })
    }

    fn parse_edge_types(&mut self, ctx: &mut ParseContext<'a>) -> Result<Vec<String>, ParseError> {
        let mut types = Vec::new();
        types.push(ctx.expect_identifier()?);
        while ctx.match_token(TokenKind::Comma) {
            types.push(ctx.expect_identifier()?);
        }
        Ok(types)
    }

    fn parse_edge_type_list(&mut self, ctx: &mut ParseContext<'a>) -> Result<Vec<String>, ParseError> {
        self.parse_edge_types(ctx)
    }

    fn parse_property_defs(&mut self, ctx: &mut ParseContext<'a>) -> Result<Vec<PropertyDef>, ParseError> {
        let mut defs = Vec::new();
        if ctx.match_token(TokenKind::LParen) {
            while !ctx.match_token(TokenKind::RParen) {
                let name = ctx.expect_identifier()?;
                ctx.expect_token(TokenKind::Colon)?;
                
                // 解析数据类型，支持关键字或标识符
                let dtype = self.parse_data_type(ctx)?;
                
                defs.push(PropertyDef {
                    name,
                    data_type: dtype,
                    nullable: true,
                    default: None,
                    comment: None,
                });
                if !ctx.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }
        Ok(defs)
    }
    
    /// 解析数据类型，支持关键字（如 STRING, INT）或标识符
    fn parse_data_type(&mut self, ctx: &mut ParseContext<'a>) -> Result<DataType, ParseError> {
        let token = ctx.current_token();
        match token.kind {
            // 支持数据类型关键字
            TokenKind::Int | TokenKind::Int8 | TokenKind::Int16 | TokenKind::Int32 | TokenKind::Int64 => {
                ctx.next_token();
                Ok(DataType::Int)
            }
            TokenKind::Float | TokenKind::Double => {
                ctx.next_token();
                Ok(DataType::Float)
            }
            TokenKind::String | TokenKind::FixedString => {
                ctx.next_token();
                Ok(DataType::String)
            }
            TokenKind::Bool => {
                ctx.next_token();
                Ok(DataType::Bool)
            }
            TokenKind::Date => {
                ctx.next_token();
                Ok(DataType::Date)
            }
            TokenKind::Timestamp => {
                ctx.next_token();
                Ok(DataType::Timestamp)
            }
            TokenKind::Datetime => {
                ctx.next_token();
                Ok(DataType::DateTime)
            }
            // 支持标识符形式的数据类型（如 "INT", "string" 等）
            TokenKind::Identifier(ref s) => {
                let type_name = s.clone();
                ctx.next_token();
                match type_name.to_uppercase().as_str() {
                    "INT" | "INTEGER" | "INT8" | "INT16" | "INT32" | "INT64" => Ok(DataType::Int),
                    "FLOAT" | "DOUBLE" => Ok(DataType::Float),
                    "STRING" | "VARCHAR" | "TEXT" => Ok(DataType::String),
                    "BOOL" | "BOOLEAN" => Ok(DataType::Bool),
                    "DATE" => Ok(DataType::Date),
                    "TIMESTAMP" => Ok(DataType::Timestamp),
                    "DATETIME" => Ok(DataType::DateTime),
                    _ => Err(ParseError::new(
                        ParseErrorKind::SyntaxError,
                        format!("未知数据类型: {}", type_name),
                        ctx.current_position(),
                    )),
                }
            }
            _ => Err(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                format!("期望数据类型，发现 {:?}", token.kind),
                ctx.current_position(),
            )),
        }
    }

    fn parse_set_clause(&mut self, ctx: &mut ParseContext<'a>) -> Result<SetClause, ParseError> {
        let span = ctx.current_span();
        let assignments = self.parse_set_assignments(ctx)?;
        Ok(SetClause { span, assignments })
    }

    fn parse_set_assignments(&mut self, ctx: &mut ParseContext<'a>) -> Result<Vec<Assignment>, ParseError> {
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

    fn parse_drop_statement(&mut self, ctx: &mut ParseContext<'a>) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Drop)?;

        let target = if ctx.match_token(TokenKind::Space) {
            DropTarget::Space(ctx.expect_identifier()?)
        } else if ctx.match_token(TokenKind::Tag) {
            let tag_name = ctx.expect_identifier()?;
            let space_name = if ctx.match_token(TokenKind::In) {
                Some(ctx.expect_identifier()?)
            } else {
                None
            };
            DropTarget::Tag {
                space_name: space_name.unwrap_or_default(),
                tag_name,
            }
        } else if ctx.match_token(TokenKind::Edge) && !ctx.match_token(TokenKind::Index) {
            let edge_name = ctx.expect_identifier()?;
            let space_name = if ctx.match_token(TokenKind::In) {
                Some(ctx.expect_identifier()?)
            } else {
                None
            };
            DropTarget::Edge {
                space_name: space_name.unwrap_or_default(),
                edge_name,
            }
        } else if ctx.match_token(TokenKind::Index) {
            let index_name = ctx.expect_identifier()?;
            let space_name = if ctx.match_token(TokenKind::On) {
                Some(ctx.expect_identifier()?)
            } else {
                None
            };
            DropTarget::TagIndex {
                space_name: space_name.unwrap_or_default(),
                index_name,
            }
        } else if ctx.match_token(TokenKind::Edge) && ctx.match_token(TokenKind::Index) {
            let index_name = ctx.expect_identifier()?;
            let space_name = if ctx.match_token(TokenKind::On) {
                Some(ctx.expect_identifier()?)
            } else {
                None
            };
            DropTarget::EdgeIndex {
                space_name: space_name.unwrap_or_default(),
                index_name,
            }
        } else {
            return Err(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                "Expected SPACE, TAG, EDGE, or INDEX".to_string(),
                ctx.current_position(),
            ));
        };

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Stmt::Drop(DropStmt { span, target }))
    }

    fn parse_desc_statement(&mut self, ctx: &mut ParseContext<'a>) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Desc)?;

        let target = if ctx.match_token(TokenKind::Space) {
            DescTarget::Space(ctx.expect_identifier()?)
        } else if ctx.match_token(TokenKind::Tag) {
            let tag_name = ctx.expect_identifier()?;
            let space_name = if ctx.match_token(TokenKind::In) {
                Some(ctx.expect_identifier()?)
            } else {
                None
            };
            DescTarget::Tag {
                space_name: space_name.unwrap_or_default(),
                tag_name,
            }
        } else if ctx.match_token(TokenKind::Edge) {
            let edge_name = ctx.expect_identifier()?;
            let space_name = if ctx.match_token(TokenKind::In) {
                Some(ctx.expect_identifier()?)
            } else {
                None
            };
            DescTarget::Edge {
                space_name: space_name.unwrap_or_default(),
                edge_name,
            }
        } else {
            return Err(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                "Expected SPACE, TAG, or EDGE".to_string(),
                ctx.current_position(),
            ));
        };

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Stmt::Desc(DescStmt { span, target }))
    }

    fn parse_alter_statement(&mut self, ctx: &mut ParseContext<'a>) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Alter)?;

        let (is_tag, space_name, name, additions, deletions) = if ctx.match_token(TokenKind::Tag) {
            let tag_name = ctx.expect_identifier()?;
            ctx.expect_token(TokenKind::Space)?;
            let space_name = ctx.expect_identifier()?;
            let additions = self.parse_alter_additions(ctx)?;
            let deletions = self.parse_alter_deletions(ctx)?;
            (true, space_name, tag_name, additions, deletions)
        } else if ctx.match_token(TokenKind::Edge) {
            let edge_name = ctx.expect_identifier()?;
            ctx.expect_token(TokenKind::Space)?;
            let space_name = ctx.expect_identifier()?;
            let additions = self.parse_alter_additions(ctx)?;
            let deletions = self.parse_alter_deletions(ctx)?;
            (false, space_name, edge_name, additions, deletions)
        } else {
            return Err(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                "Expected TAG or EDGE".to_string(),
                ctx.current_position(),
            ));
        };

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        if is_tag {
            Ok(Stmt::Alter(AlterStmt {
                span,
                target: AlterTarget::Tag {
                    space_name,
                    tag_name: name,
                    additions,
                    deletions,
                },
            }))
        } else {
            Ok(Stmt::Alter(AlterStmt {
                span,
                target: AlterTarget::Edge {
                    space_name,
                    edge_name: name,
                    additions,
                    deletions,
                },
            }))
        }
    }

    fn parse_alter_additions(&mut self, ctx: &mut ParseContext<'a>) -> Result<Vec<PropertyDef>, ParseError> {
        let mut additions = Vec::new();
        if ctx.match_token(TokenKind::Add) {
            additions = self.parse_property_defs(ctx)?;
        }
        Ok(additions)
    }

    fn parse_alter_deletions(&mut self, ctx: &mut ParseContext<'a>) -> Result<Vec<String>, ParseError> {
        let mut deletions = Vec::new();
        if ctx.match_token(TokenKind::Drop) {
            ctx.expect_token(TokenKind::LParen)?;
            while !ctx.match_token(TokenKind::RParen) {
                deletions.push(ctx.expect_identifier()?);
                if !ctx.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }
        Ok(deletions)
    }

    fn parse_create_user_statement(&mut self, ctx: &mut ParseContext<'a>) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::CreateUser)?;

        let mut if_not_exists = false;
        if ctx.match_token(TokenKind::If) {
            ctx.expect_token(TokenKind::Not)?;
            ctx.expect_token(TokenKind::Exists)?;
            if_not_exists = true;
        }

        let username = ctx.expect_identifier()?;
        ctx.expect_token(TokenKind::Password)?;
        let password = ctx.expect_string_literal()?;

        let mut role = None;
        if ctx.match_token(TokenKind::With) {
            ctx.expect_token(TokenKind::Role)?;
            role = Some(ctx.expect_identifier()?);
        }

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Stmt::CreateUser(CreateUserStmt {
            span,
            username,
            password,
            role,
            if_not_exists,
        }))
    }

    fn parse_alter_user_statement(&mut self, ctx: &mut ParseContext<'a>) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::AlterUser)?;

        let username = ctx.expect_identifier()?;

        let mut new_role = None;
        let mut is_locked = None;

        while ctx.match_token(TokenKind::Set) {
            if ctx.match_token(TokenKind::Role) {
                ctx.expect_token(TokenKind::Eq)?;
                new_role = Some(ctx.expect_identifier()?);
            } else if ctx.match_token(TokenKind::Locked) {
                ctx.expect_token(TokenKind::Eq)?;
                let value = ctx.expect_identifier()?;
                is_locked = Some(value.to_lowercase() == "true");
            }
        }

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Stmt::AlterUser(AlterUserStmt {
            span,
            username,
            new_role,
            is_locked,
        }))
    }

    fn parse_drop_user_statement(&mut self, ctx: &mut ParseContext<'a>) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::DropUser)?;

        let mut if_exists = false;
        if ctx.match_token(TokenKind::If) {
            ctx.expect_token(TokenKind::Exists)?;
            if_exists = true;
        }

        let username = ctx.expect_identifier()?;

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Stmt::DropUser(DropUserStmt {
            span,
            username,
            if_exists,
        }))
    }

    fn parse_change_password_statement(&mut self, ctx: &mut ParseContext<'a>) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::ChangePassword)?;

        let username = ctx.expect_identifier()?;
        ctx.expect_token(TokenKind::Password)?;
        let old_password = ctx.expect_string_literal()?;
        let new_password = ctx.expect_string_literal()?;

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Stmt::ChangePassword(ChangePasswordStmt {
            span,
            username,
            old_password,
            new_password,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::parser::parser::ParseContext;

    fn create_parser_context(input: &str) -> ParseContext {
        ParseContext::new(input)
    }

    #[test]
    fn test_parse_data_type_keywords() {
        let mut parser = StmtParser::new();
        
        // 测试 STRING 关键字
        let mut ctx = create_parser_context("STRING");
        let result = parser.parse_data_type(&mut ctx);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DataType::String);
        
        // 测试 INT 关键字
        let mut ctx = create_parser_context("INT");
        let result = parser.parse_data_type(&mut ctx);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DataType::Int);
        
        // 测试 FLOAT 关键字
        let mut ctx = create_parser_context("FLOAT");
        let result = parser.parse_data_type(&mut ctx);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DataType::Float);
        
        // 测试 BOOL 关键字
        let mut ctx = create_parser_context("BOOL");
        let result = parser.parse_data_type(&mut ctx);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DataType::Bool);
        
        // 测试 DATE 关键字
        let mut ctx = create_parser_context("DATE");
        let result = parser.parse_data_type(&mut ctx);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DataType::Date);
    }

    #[test]
    fn test_parse_data_type_identifiers() {
        let mut parser = StmtParser::new();
        
        // 测试小写标识符形式
        let mut ctx = create_parser_context("string");
        let result = parser.parse_data_type(&mut ctx);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DataType::String);
        
        // 测试混合大小写
        let mut ctx = create_parser_context("Int");
        let result = parser.parse_data_type(&mut ctx);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), DataType::Int);
    }

    #[test]
    fn test_create_tag_statement_parses() {
        let mut parser = StmtParser::new();
        
        // 测试 CREATE TAG 语句能够解析成功
        let mut ctx = create_parser_context("CREATE TAG Person(name: STRING, age: INT)");
        let result = parser.parse_statement(&mut ctx);
        
        // 验证解析成功
        assert!(result.is_ok(), "CREATE TAG 解析失败: {:?}", result.err());
        
        // 验证是 Create 语句
        if let Ok(Stmt::Create(stmt)) = result {
            // 验证是 Tag 创建目标
            match stmt.target {
                CreateTarget::Tag { name, .. } => {
                    assert_eq!(name, "Person");
                }
                _ => panic!("期望 Tag 创建目标，实际得到 {:?}", stmt.target),
            }
        } else {
            panic!("期望 Create 语句");
        }
    }

    #[test]
    fn test_create_edge_type_statement_parses() {
        let mut parser = StmtParser::new();
        
        // 测试 CREATE EDGE 语句能够解析成功
        let mut ctx = create_parser_context("CREATE EDGE KNOWS(since: DATE)");
        let result = parser.parse_statement(&mut ctx);
        
        // 验证解析成功
        assert!(result.is_ok(), "CREATE EDGE 解析失败: {:?}", result.err());
        
        // 验证是 Create 语句
        if let Ok(Stmt::Create(stmt)) = result {
            // 验证是 EdgeType 创建目标
            match stmt.target {
                CreateTarget::EdgeType { name, .. } => {
                    assert_eq!(name, "KNOWS");
                }
                _ => panic!("期望 EdgeType 创建目标，实际得到 {:?}", stmt.target),
            }
        } else {
            panic!("期望 Create 语句");
        }
    }

    #[test]
    fn test_insert_vertex_statement_parses() {
        let mut parser = StmtParser::new();
        
        // 测试 INSERT VERTEX 语句解析
        let mut ctx = create_parser_context("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30)");
        let result = parser.parse_statement(&mut ctx);
        
        // 验证解析成功
        assert!(result.is_ok(), "INSERT VERTEX 解析失败: {:?}", result.err());
        
        // 验证是 Insert 语句
        if let Ok(Stmt::Insert(stmt)) = result {
            match stmt.target {
                InsertTarget::Vertices { tag_name, .. } => {
                    assert_eq!(tag_name, "Person");
                }
                _ => panic!("期望 Vertices 插入目标，实际得到 {:?}", stmt.target),
            }
        } else {
            panic!("期望 Insert 语句");
        }
    }
    
    #[test]
    fn test_insert_vertex_simple() {
        // 测试简化版 INSERT VERTEX
        let mut parser = StmtParser::new();
        let mut ctx = create_parser_context("INSERT VERTEX Person() VALUES 1:()");
        
        // 调试：打印所有 token
        println!("调试 INSERT VERTEX 解析:");
        let mut debug_ctx = create_parser_context("INSERT VERTEX Person() VALUES 1:()");
        loop {
            let token = debug_ctx.current_token();
            println!("Token: {:?}, Lexeme: '{}'", token.kind, token.lexeme);
            if token.kind == TokenKind::Eof {
                break;
            }
            debug_ctx.next_token();
        }
        
        let result = parser.parse_statement(&mut ctx);
        
        assert!(result.is_ok(), "简化版 INSERT VERTEX 解析失败: {:?}", result.err());
        
        if let Ok(Stmt::Insert(stmt)) = result {
            match stmt.target {
                InsertTarget::Vertices { tag_name, .. } => {
                    assert_eq!(tag_name, "Person");
                }
                _ => panic!("期望 Vertices 插入目标"),
            }
        }
    }
    
    #[test]
    fn test_tokenize_parentheses() {
        // 测试括号是否正确识别 - 使用 ParseContext
        use crate::query::parser::parser::ParseContext;
        use crate::query::parser::lexer::TokenKind as Tk;
        
        let mut ctx = ParseContext::new("()");
        println!("Token 1: {:?}", ctx.current_token().kind);
        ctx.next_token();
        println!("Token 2: {:?}", ctx.current_token().kind);
        ctx.next_token();
        println!("Token 3: {:?}", ctx.current_token().kind);
        
        let mut ctx2 = ParseContext::new("()");
        assert_eq!(ctx2.current_token().kind, Tk::LParen);
        ctx2.next_token();
        assert_eq!(ctx2.current_token().kind, Tk::RParen);
    }
}
