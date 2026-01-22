//! 语句解析模块
//!
//! 负责解析各种语句，包括 MATCH、CREATE、DELETE、UPDATE 等。

use crate::core::types::graph::EdgeDirection;
use crate::query::parser::ast::types::{BinaryOp, UnaryOp};
use crate::query::parser::ast::*;
use crate::query::parser::ast::expr::*;
use crate::query::parser::ast::pattern::*;
use crate::query::parser::ast::stmt::*;
use crate::query::parser::core::error::{ParseError, ParseErrorKind};
use crate::query::parser::core::position::Position;
use crate::query::parser::core::span::Span;
use crate::query::parser::lexer::TokenKind as LexerToken;
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

        let patterns = self.parse_patterns(ctx)?;

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

        let order_by = if ctx.match_token(TokenKind::Order) && ctx.match_token(TokenKind::By) {
            Some(self.parse_order_by_clause(ctx)?)
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
            order_by,
            limit: None,
            skip: None,
        }))
    }

    fn parse_create_statement(&mut self, ctx: &mut ParseContext<'a>) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Create)?;

        if ctx.match_token(TokenKind::Tag) {
            let name = ctx.expect_identifier()?;
            let properties = self.parse_properties(ctx)?;
            Ok(Stmt::Create(CreateStmt {
                span: start_span,
                kind: CreateStmtKind::Tag(name),
                properties,
            }))
        } else if ctx.match_token(TokenKind::Edge) {
            let name = ctx.expect_identifier()?;
            let properties = self.parse_properties(ctx)?;
            Ok(Stmt::Create(CreateStmt {
                span: start_span,
                kind: CreateStmtKind::Edge(name),
                properties,
            }))
        } else {
            Err(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                "Expected TAG or EDGE after CREATE".to_string(),
                ctx.current_position(),
            ))
        }
    }

    pub fn parse_delete_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::Delete)?;

        let target = if self.match_token(LexerToken::Vertex) || self.match_token(LexerToken::Vertices) {
            let vertices = self.parse_expression_list()?;
            DeleteTarget::Vertices(vertices)
        } else if self.match_token(LexerToken::Edge) || self.match_token(LexerToken::Edges) {
            DeleteTarget::Edges {
                src: self.parse_expression()?,
                dst: self.parse_expression()?,
                edge_type: None,
                rank: None,
            }
        } else {
            DeleteTarget::Vertices(vec![self.parse_expression()?])
        };

        let where_clause = if self.match_token(LexerToken::Where) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Stmt::Delete(DeleteStmt { span, target, where_clause }))
    }

    pub fn parse_update_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::Update)?;

        let target = UpdateTarget::Vertex(self.parse_expression()?);
        let set_clause = self.parse_set_clause()?;
        let where_clause = if self.match_token(LexerToken::Where) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Stmt::Update(UpdateStmt { span, target, set_clause, where_clause }))
    }

    pub fn parse_go_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::Go)?;

        let steps = self.parse_steps()?;
        let from = self.parse_from_clause()?;
        let over = if self.match_token(LexerToken::Over) {
            Some(self.parse_over_clause()?)
        } else {
            None
        };
        let where_clause = if self.match_token(LexerToken::Where) {
            Some(self.parse_expression()?)
        } else {
            None
        };
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

    pub fn parse_fetch_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::Fetch)?;

        let ids = self.parse_expression_list()?;

        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Stmt::Fetch(FetchStmt {
            span,
            target: FetchTarget::Vertices { ids, properties: None },
        }))
    }

    pub fn parse_use_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::Use)?;

        let space = self.expect_identifier()?;
        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Stmt::Use(UseStmt { span, space }))
    }

    pub fn parse_show_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::Show)?;

        let target = if self.match_token(LexerToken::Spaces) {
            ShowTarget::Spaces
        } else if self.match_token(LexerToken::Tags) {
            ShowTarget::Tags
        } else if self.match_token(LexerToken::Edges) {
            ShowTarget::Edges
        } else if self.match_token(LexerToken::Indexes) {
            ShowTarget::Indexes
        } else {
            let name = self.expect_identifier()?;
            ShowTarget::Tag(name)
        };

        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Stmt::Show(ShowStmt { span, target }))
    }

    pub fn parse_explain_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::Explain)?;

        let statement = Box::new(self.parse_statement()?);
        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Stmt::Explain(ExplainStmt { span, statement }))
    }

    pub fn parse_lookup_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::Lookup)?;
        self.expect_token(LexerToken::On)?;

        let name = self.expect_identifier()?;
        let target = if self.match_token(LexerToken::Tag) {
            LookupTarget::Tag(name)
        } else if self.match_token(LexerToken::Edge) {
            LookupTarget::Edge(name)
        } else {
            LookupTarget::Tag(name)
        };

        let where_clause = if self.match_token(LexerToken::Where) {
            Some(self.parse_expression()?)
        } else {
            None
        };

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

    pub fn parse_unwind_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::Unwind)?;

        let expression = self.parse_expression()?;
        self.expect_token(LexerToken::As)?;
        let variable = self.expect_identifier()?;

        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Stmt::Unwind(UnwindStmt { span, expression, variable }))
    }

    pub fn parse_merge_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::Merge)?;

        let pattern = self.parse_pattern()?;
        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Stmt::Merge(MergeStmt { span, pattern }))
    }

    pub fn parse_insert_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::Insert)?;

        let target = if self.match_token(LexerToken::Vertex) {
            let ids = self.parse_expression_list()?;
            InsertTarget::Vertices { ids }
        } else if self.match_token(LexerToken::Edge) {
            let src = self.parse_expression()?;
            let dst = self.parse_expression()?;
            InsertTarget::Edge { src, dst }
        } else {
            return Err(self.parse_error("Expected VERTEX or EDGE".to_string()));
        };

        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Stmt::Insert(InsertStmt { span, target }))
    }

    pub fn parse_return_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::Return)?;

        let distinct = self.match_token(LexerToken::Distinct);
        let items = self.parse_return_items()?;

        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Stmt::Return(ReturnStmt { span, items, distinct }))
    }

    pub fn parse_with_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::With)?;

        let items = self.parse_return_items()?;
        let where_clause = if self.match_token(LexerToken::Where) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Stmt::With(WithStmt { span, items, where_clause }))
    }

    pub fn parse_set_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
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

        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Stmt::Set(SetStmt { span, assignments }))
    }

    pub fn parse_remove_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::Remove)?;

        let items = self.parse_expression_list()?;
        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Stmt::Remove(RemoveStmt { span, items }))
    }

    pub fn parse_pipe_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::Pipe)?;

        let expression = self.parse_expression()?;
        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Stmt::Pipe(PipeStmt { span, expression }))
    }

    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.expr_parser.parse_expression()
    }

    fn parse_patterns(&mut self) -> Result<Vec<Pattern>, ParseError> {
        let mut patterns = Vec::new();
        loop {
            patterns.push(self.parse_pattern()?);
            if !self.match_token(LexerToken::Comma) {
                break;
            }
        }
        Ok(patterns)
    }

    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        let span = self.current_span();
        if self.match_token(LexerToken::LParen) {
            let variable = if let LexerToken::Identifier(_) = self.lexer.peek()?.kind {
                Some(self.expect_identifier()?)
            } else {
                None
            };
            self.expect_token(LexerToken::RParen)?;
            Ok(Pattern::Node(NodePattern::new(variable, vec![], None, vec![], span)))
        } else {
            Err(self.parse_error("Expected pattern".to_string()))
        }
    }

    fn parse_return_items(&mut self) -> Result<Vec<ReturnItem>, ParseError> {
        let mut items = Vec::new();
        loop {
            if self.check_token(LexerToken::Eof)
                || matches!(self.lexer.peek()?.kind, LexerToken::Semicolon | LexerToken::Where)
            {
                break;
            }
            if self.match_token(LexerToken::Star) {
                items.push(ReturnItem::All);
            } else {
                let expr = self.parse_expression()?;
                let alias = if self.match_token(LexerToken::As) {
                    Some(self.expect_identifier()?)
                } else {
                    None
                };
                items.push(ReturnItem::Expression { expr, alias });
            }
            if !self.match_token(LexerToken::Comma) {
                break;
            }
        }
        Ok(items)
    }

    fn parse_return_clause(&mut self) -> Result<ReturnClause, ParseError> {
        let span = self.current_span();
        let items = self.parse_return_items()?;
        Ok(ReturnClause {
            span,
            items,
            distinct: false,
            limit: None,
            skip: None,
            sample: None,
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
                OrderDirection::Asc
            };
            items.push(OrderByItem { expr, direction });
            if !self.match_token(LexerToken::Comma) {
                break;
            }
        }
        Ok(OrderByClause { span, items })
    }

    fn parse_steps(&mut self) -> Result<Steps, ParseError> {
        if let LexerToken::IntegerLiteral(_) = self.lexer.peek()?.kind {
            let steps = self.parse_integer()? as usize;
            Ok(Steps::Fixed(steps))
        } else {
            Ok(Steps::Fixed(1))
        }
    }

    fn parse_from_clause(&mut self) -> Result<FromClause, ParseError> {
        let span = self.current_span();
        self.expect_token(LexerToken::From)?;
        let vertices = self.parse_expression_list()?;
        Ok(FromClause { span, vertices })
    }

    fn parse_over_clause(&mut self) -> Result<OverClause, ParseError> {
        let span = self.current_span();
        self.expect_token(LexerToken::Over)?;

        let mut edge_types = Vec::new();
        let mut direction = EdgeDirection::Outgoing;

        loop {
            let edge_type = self.expect_identifier()?;
            edge_types.push(edge_type);
            if !self.match_token(LexerToken::Comma) {
                break;
            }
        }

        if self.match_token(LexerToken::Out) {
            direction = EdgeDirection::Outgoing;
        } else if self.match_token(LexerToken::In) {
            direction = EdgeDirection::Incoming;
        } else if self.match_token(LexerToken::Both) {
            direction = EdgeDirection::Both;
        }

        Ok(OverClause { span, edge_types, direction })
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
        Ok(YieldClause {
            span,
            items,
            limit: None,
            skip: None,
            sample: None,
        })
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

    fn parse_expression_list(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut expressions = Vec::new();
        loop {
            expressions.push(self.parse_expression()?);
            if !self.match_token(LexerToken::Comma) {
                break;
            }
        }
        Ok(expressions)
    }

    fn parse_properties(&mut self) -> Result<Vec<PropertyDef>, ParseError> {
        let mut properties = Vec::new();
        if self.match_token(LexerToken::LParen) {
            loop {
                let name = self.expect_identifier()?;
                self.expect_token(LexerToken::Colon)?;
                let _data_type = self.expect_identifier()?;
                properties.push(PropertyDef {
                    name,
                    data_type: DataType::String,
                    nullable: true,
                    default: None,
                });
                if !self.match_token(LexerToken::Comma) {
                    break;
                }
            }
            self.expect_token(LexerToken::RParen)?;
        }
        Ok(properties)
    }

    fn parse_index_properties(&mut self) -> Result<Vec<String>, ParseError> {
        let mut properties = Vec::new();
        if self.match_token(LexerToken::LParen) {
            loop {
                let prop = self.expect_identifier()?;
                properties.push(prop);
                if !self.match_token(LexerToken::Comma) {
                    break;
                }
            }
            self.expect_token(LexerToken::RParen)?;
        }
        Ok(properties)
    }

    fn match_token(&mut self, expected: LexerToken) -> bool {
        if self.lexer.check(expected.clone()) {
            let _ = self.lexer.advance();
            true
        } else {
            false
        }
    }

    fn check_token(&mut self, expected: LexerToken) -> bool {
        self.lexer.check(expected.clone())
    }

    fn expect_token(&mut self, expected: LexerToken) -> Result<(), ParseError> {
        let token = self.lexer.peek().map_err(|e| ParseError::from(e))?;
        if token.kind == expected {
            self.lexer.advance();
            Ok(())
        } else {
            Err(self.parse_error(format!("Expected {:?}, found {:?}", expected, token.kind)))
        }
    }

    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        let token = self.lexer.peek().map_err(|e| ParseError::from(e))?;
        if let LexerToken::Identifier(_) = token.kind {
            let text = token.lexeme.clone();
            self.lexer.advance();
            Ok(text)
        } else {
            Err(self.parse_error(format!("Expected identifier, found {:?}", token.kind)))
        }
    }

    fn current_span(&self) -> Span {
        let pos = self.lexer.current_position();
        Span::new(
            Position::new(pos.line, pos.column),
            Position::new(pos.line, pos.column),
        )
    }

    fn parse_integer(&mut self) -> Result<i64, ParseError> {
        let token = self.lexer.peek().map_err(|e| ParseError::from(e))?;
        if let LexerToken::IntegerLiteral(n) = token.kind {
            let text = token.lexeme.clone();
            self.lexer.advance();
            text.parse().map_err(|_| self.parse_error(format!("Invalid integer: {}", text)))
        } else {
            Err(self.parse_error(format!("Expected integer, found {:?}", token.kind)))
        }
    }

    fn parse_error(&self, message: String) -> ParseError {
        let pos = self.lexer.current_position();
        ParseError::new(ParseErrorKind::SyntaxError, message, pos.line, pos.column)
    }
}
