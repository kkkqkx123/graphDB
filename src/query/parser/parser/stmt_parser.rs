//! 语句解析模块
//!
//! 负责解析各种语句，包括 MATCH、CREATE、DELETE、UPDATE 等。
//! 提供两种使用方式：
//! 1. 独立的 `StmtParser` 结构体
//! 2. 作为 `Parser` 的方法

use crate::core::types::graph::EdgeDirection;
use crate::core::Value;
use crate::core::value::types::NullType;
use crate::query::parser::ast::*;
use crate::query::parser::ast::expr::*;
use crate::query::parser::ast::pattern::*;
use crate::query::parser::ast::stmt::*;
use crate::query::parser::ast::types::*;
use crate::query::parser::core::error::ParseErrorKind;
use crate::query::parser::lexer::{Lexer, TokenKind as LexerToken};

/// 独立的语句解析器
///
/// 用于独立解析语句的场景，与完整的 SQL Parser 分离
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

    /// 解析语句
    pub fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        let token = self.lexer.peek().map_err(|e| ParseError::from(e))?;
        match token.kind {
            LexerToken::Match => self.parse_match_statement(),
            LexerToken::Go => self.parse_go_statement(),
            LexerToken::Create => self.parse_create_statement(),
            LexerToken::Delete => self.parse_delete_statement(),
            LexerToken::Update => self.parse_update_statement(),
            LexerToken::Use => self.parse_use_statement(),
            LexerToken::Show => self.parse_show_statement(),
            LexerToken::Explain => self.parse_explain_statement(),
            LexerToken::Lookup => self.parse_lookup_statement(),
            LexerToken::Fetch => self.parse_fetch_statement(),
            LexerToken::Unwind => self.parse_unwind_statement(),
            LexerToken::Merge => self.parse_merge_statement(),
            LexerToken::Insert => self.parse_insert_statement(),
            LexerToken::Return => self.parse_return_statement(),
            LexerToken::With => self.parse_with_statement(),
            LexerToken::Set => self.parse_set_statement(),
            LexerToken::Remove => self.parse_remove_statement(),
            LexerToken::Pipe => self.parse_pipe_statement(),
            _ => Err(self.parse_error(format!("Unexpected token: {:?}", token.kind))),
        }
    }

    /// 解析 MATCH 语句
    pub fn parse_match_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::Match)?;

        let patterns = self.parse_patterns()?;

        let where_clause = if self.match_token(LexerToken::Where) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        let return_clause = if self.match_token(LexerToken::Return) {
            Some(self.parse_return_clause()?)
        } else {
            None
        };

        let order_by = if self.match_token(LexerToken::Order) && self.match_token(LexerToken::By) {
            Some(self.parse_order_by_clause()?)
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
            limit: None,
            skip: None,
        }))
    }

    /// 解析 CREATE 语句
    pub fn parse_create_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::Create)?;

        if self.match_token(LexerToken::Tag) {
            let name = self.expect_identifier()?;
            let properties = self.parse_properties()?;
            Ok(Stmt::Create(CreateStmt {
                span: start_span,
                target: CreateTarget::Tag { name, properties },
            }))
        } else if self.match_token(LexerToken::Index) {
            let name = self.expect_identifier()?;
            self.expect_token(LexerToken::On)?;
            let on = self.expect_identifier()?;
            let properties = self.parse_index_properties()?;
            Ok(Stmt::Create(CreateStmt {
                span: start_span,
                target: CreateTarget::Index { name, on, properties },
            }))
        } else {
            let pattern = self.parse_pattern()?;
            Ok(Stmt::Create(CreateStmt {
                span: start_span,
                target: CreateTarget::Node {
                    variable: None,
                    labels: vec![],
                    properties: None,
                },
            }))
        }
    }

    /// 解析 DELETE 语句
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

    /// 解析 UPDATE 语句
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

    /// 解析 GO 语句
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

    /// 解析 FETCH 语句
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

    /// 解析 EXPLAIN 语句
    pub fn parse_explain_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::Explain)?;

        let statement = Box::new(self.parse_statement()?);
        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Stmt::Explain(ExplainStmt { span, statement }))
    }

    /// 解析 LOOKUP 语句
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

    /// 解析 UNWIND 语句
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

    /// 解析 MERGE 语句
    pub fn parse_merge_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::Merge)?;

        let pattern = self.parse_pattern()?;
        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Stmt::Merge(MergeStmt { span, pattern }))
    }

    /// 解析 INSERT 语句
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

    /// 解析 RETURN 语句
    pub fn parse_return_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::Return)?;

        let distinct = self.match_token(LexerToken::Distinct);
        let items = self.parse_return_items()?;

        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Stmt::Return(ReturnStmt { span, items, distinct }))
    }

    /// 解析 WITH 语句
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

    /// 解析 SET 语句
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

    /// 解析 REMOVE 语句
    pub fn parse_remove_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::Remove)?;

        let items = self.parse_expression_list()?;
        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Stmt::Remove(RemoveStmt { span, items }))
    }

    /// 解析 PIPE 语句
    pub fn parse_pipe_statement(&mut self) -> Result<Stmt, ParseError> {
        let start_span = self.current_span();
        self.expect_token(LexerToken::Pipe)?;

        let expression = self.parse_expression()?;
        let end_span = self.current_span();
        let span = Span::new(start_span.start, end_span.end);

        Ok(Stmt::Pipe(PipeStmt { span, expression }))
    }

    /// 表达式解析方法

    pub fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_or_expression()
    }

    fn parse_or_expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and_expression()?;
        while self.match_token(LexerToken::Or) {
            let op = BinaryOp::Or;
            let right = self.parse_and_expression()?;
            let span = Span::new(left.span().start, right.span().end);
            left = Expr::Binary(BinaryExpr::new(left, op, right, span));
        }
        Ok(left)
    }

    fn parse_and_expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_not_expression()?;
        while self.match_token(LexerToken::And) {
            let op = BinaryOp::And;
            let right = self.parse_not_expression()?;
            let span = Span::new(left.span().start, right.span().end);
            left = Expr::Binary(BinaryExpr::new(left, op, right, span));
        }
        Ok(left)
    }

    fn parse_not_expression(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(LexerToken::NotOp) {
            let op = UnaryOp::Not;
            let operand = self.parse_not_expression()?;
            let span = Span::new(operand.span().start, operand.span().end);
            Ok(Expr::Unary(UnaryExpr::new(op, operand, span)))
        } else {
            self.parse_comparison_expression()
        }
    }

    fn parse_comparison_expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_additive_expression()?;
        if let Some(op) = self.parse_comparison_op() {
            let right = self.parse_additive_expression()?;
            let span = Span::new(left.span().start, right.span().end);
            left = Expr::Binary(BinaryExpr::new(left, op, right, span));
        }
        Ok(left)
    }

    fn parse_comparison_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(LexerToken::Eq) {
            Some(BinaryOp::Equal)
        } else if self.match_token(LexerToken::Ne) {
            Some(BinaryOp::NotEqual)
        } else if self.match_token(LexerToken::Lt) {
            Some(BinaryOp::LessThan)
        } else if self.match_token(LexerToken::Le) {
            Some(BinaryOp::LessThanOrEqual)
        } else if self.match_token(LexerToken::Gt) {
            Some(BinaryOp::GreaterThan)
        } else if self.match_token(LexerToken::Ge) {
            Some(BinaryOp::GreaterThanOrEqual)
        } else {
            None
        }
    }

    fn parse_additive_expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplicative_expression()?;
        while let Some(op) = self.parse_additive_op() {
            let right = self.parse_multiplicative_expression()?;
            let span = Span::new(left.span().start, right.span().end);
            left = Expr::Binary(BinaryExpr::new(left, op, right, span));
        }
        Ok(left)
    }

    fn parse_additive_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(LexerToken::Plus) {
            Some(BinaryOp::Add)
        } else if self.match_token(LexerToken::Minus) {
            Some(BinaryOp::Subtract)
        } else {
            None
        }
    }

    fn parse_multiplicative_expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary_expression()?;
        while let Some(op) = self.parse_multiplicative_op() {
            let right = self.parse_unary_expression()?;
            let span = Span::new(left.span().start, right.span().end);
            left = Expr::Binary(BinaryExpr::new(left, op, right, span));
        }
        Ok(left)
    }

    fn parse_multiplicative_op(&mut self) -> Option<BinaryOp> {
        if self.match_token(LexerToken::Star) {
            Some(BinaryOp::Multiply)
        } else if self.match_token(LexerToken::Div) {
            Some(BinaryOp::Divide)
        } else if self.match_token(LexerToken::Mod) {
            Some(BinaryOp::Modulo)
        } else {
            None
        }
    }

    fn parse_unary_expression(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(LexerToken::Minus) {
            let op = UnaryOp::Minus;
            let operand = self.parse_unary_expression()?;
            let span = Span::new(operand.span().start, operand.span().end);
            Ok(Expr::Unary(UnaryExpr::new(op, operand, span)))
        } else if self.match_token(LexerToken::Plus) {
            let op = UnaryOp::Plus;
            let operand = self.parse_unary_expression()?;
            let span = Span::new(operand.span().start, operand.span().end);
            Ok(Expr::Unary(UnaryExpr::new(op, operand, span)))
        } else {
            self.parse_primary_expression()
        }
    }

    fn parse_primary_expression(&mut self) -> Result<Expr, ParseError> {
        let token = self.lexer.peek().map_err(|e| ParseError::from(e))?;
        match token.kind {
            LexerToken::IntegerLiteral(n) => {
                let value = self.parse_integer()?;
                let span = self.current_span();
                Ok(Expr::Constant(ConstantExpr::new(Value::Int(value), span)))
            }
            LexerToken::FloatLiteral(f) => {
                let value = self.parse_float()?;
                let span = self.current_span();
                Ok(Expr::Constant(ConstantExpr::new(Value::Float(value), span)))
            }
            LexerToken::StringLiteral(_) => {
                let value = self.parse_string()?;
                let span = self.current_span();
                Ok(Expr::Constant(ConstantExpr::new(Value::String(value), span)))
            }
            LexerToken::BooleanLiteral(b) => {
                let value = Value::Bool(b);
                self.lexer.advance();
                let span = self.current_span();
                Ok(Expr::Constant(ConstantExpr::new(value, span)))
            }
            LexerToken::Null => {
                self.lexer.advance();
                let span = self.current_span();
                Ok(Expr::Constant(ConstantExpr::new(Value::Null(NullType::Null), span)))
            }
            LexerToken::Identifier(_) => {
                let name = self.expect_identifier()?;
                let span = self.current_span();
                if self.match_token(LexerToken::LParen) {
                    self.parse_function_call(name, span)
                } else {
                    Ok(Expr::Variable(VariableExpr::new(name, span)))
                }
            }
            LexerToken::LParen => self.parse_subquery_expression(),
            _ => Err(self.parse_error(format!("Unexpected token in expression: {:?}", token.kind))),
        }
    }

    fn parse_function_call(&mut self, name: String, span: Span) -> Result<Expr, ParseError> {
        let mut args = Vec::new();
        if !self.check_token(LexerToken::RParen) {
            loop {
                args.push(self.parse_expression()?);
                if !self.match_token(LexerToken::Comma) {
                    break;
                }
            }
        }
        self.expect_token(LexerToken::RParen)?;
        let call_span = Span::new(span.start, self.current_span().end);
        Ok(Expr::FunctionCall(FunctionCallExpr::new(name, args, false, call_span)))
    }

    fn parse_subquery_expression(&mut self) -> Result<Expr, ParseError> {
        self.expect_token(LexerToken::LParen)?;
        let expr = self.parse_expression()?;
        self.expect_token(LexerToken::RParen)?;
        Ok(expr)
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

    fn parse_float(&mut self) -> Result<f64, ParseError> {
        let token = self.lexer.peek().map_err(|e| ParseError::from(e))?;
        if let LexerToken::FloatLiteral(f) = token.kind {
            let text = token.lexeme.clone();
            self.lexer.advance();
            text.parse().map_err(|_| self.parse_error(format!("Invalid float: {}", text)))
        } else {
            Err(self.parse_error(format!("Expected float, found {:?}", token.kind)))
        }
    }

    fn parse_string(&mut self) -> Result<String, ParseError> {
        let token = self.lexer.peek().map_err(|e| ParseError::from(e))?;
        if let LexerToken::StringLiteral(_) = token.kind {
            let text = token.lexeme.clone();
            self.lexer.advance();
            Ok(text)
        } else {
            Err(self.parse_error(format!("Expected string, found {:?}", token.kind)))
        }
    }

    /// 辅助方法

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

    fn parse_error(&self, message: String) -> ParseError {
        let pos = self.lexer.current_position();
        ParseError::new(ParseErrorKind::SyntaxError, message, pos.line, pos.column)
    }
}
