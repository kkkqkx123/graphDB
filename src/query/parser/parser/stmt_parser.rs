//! 语句解析模块
//!
//! 负责解析各种语句，包括 MATCH、GO、CREATE、DELETE、UPDATE 等。

use crate::core::types::graph_schema::EdgeDirection;
use crate::core::types::PropertyDef;
use crate::query::parser::ast::*;
use crate::query::parser::ast::pattern::{EdgePattern, NodePattern, PathElement, PathPattern};
use crate::query::parser::core::error::{ParseError, ParseErrorKind};
use crate::query::parser::parser::ExprParser;
use crate::query::parser::parser::ParseContext;
use crate::query::parser::TokenKind;

pub struct StmtParser<'a> {
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> StmtParser<'a> {
    pub fn new(_ctx: &ParseContext<'a>) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
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

        let target = FetchTarget::Vertices {
            ids: self.parse_expression_list(ctx)?,
            properties: None,
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

        Ok(Stmt::Merge(MergeStmt {
            span: start_span,
            pattern,
        }))
    }

    fn parse_insert_statement(&mut self, ctx: &mut ParseContext<'a>) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Insert)?;

        let target = InsertTarget::Vertices {
            ids: Vec::new(),
        };

        Ok(Stmt::Insert(InsertStmt { span: start_span, target }))
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

    fn parse_expression(&mut self, ctx: &mut ParseContext<'a>) -> Result<Expression, ParseError> {
        let mut expr_parser = ExprParser::new(ctx);
        expr_parser.parse_expression(ctx)
    }

    fn parse_expression_list(&mut self, ctx: &mut ParseContext<'a>) -> Result<Vec<Expression>, ParseError> {
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
                let data_type = ctx.expect_identifier()?;
                let dtype = match data_type.to_uppercase().as_str() {
                    "INT" => DataType::Int,
                    "FLOAT" => DataType::Float,
                    "STRING" => DataType::String,
                    _ => DataType::String,
                };
                defs.push(PropertyDef {
                    name,
                    data_type: dtype,
                    nullable: true,
                    default: None,
                });
                if !ctx.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }
        Ok(defs)
    }

    fn parse_set_clause(&mut self, ctx: &mut ParseContext<'a>) -> Result<SetClause, ParseError> {
        let span = ctx.current_span();
        let assignments = self.parse_set_assignments(ctx)?;
        Ok(SetClause { span, assignments })
    }

    fn parse_set_assignments(&mut self, ctx: &mut ParseContext<'a>) -> Result<Vec<Assignment>, ParseError> {
        let mut assignments = Vec::new();
        loop {
            let property = ctx.expect_identifier()?;
            ctx.expect_token(TokenKind::Assign)?;
            let value = self.parse_expression(ctx)?;
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
            DropTarget::Tag { space_name: String::new(), tag_name }
        } else if ctx.match_token(TokenKind::Edge) {
            let edge_name = ctx.expect_identifier()?;
            DropTarget::Edge { space_name: String::new(), edge_name }
        } else if ctx.match_token(TokenKind::Index) {
            let index_name = ctx.expect_identifier()?;
            DropTarget::TagIndex { space_name: String::new(), index_name }
        } else if ctx.match_token(TokenKind::Edge) && ctx.match_token(TokenKind::Index) {
            let index_name = ctx.expect_identifier()?;
            DropTarget::EdgeIndex { space_name: String::new(), index_name }
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
            DescTarget::Tag { space_name: String::new(), tag_name }
        } else if ctx.match_token(TokenKind::Edge) {
            let edge_name = ctx.expect_identifier()?;
            DescTarget::Edge { space_name: String::new(), edge_name }
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
