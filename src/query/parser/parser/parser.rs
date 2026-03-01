use std::sync::Arc;

use crate::query::parser::parser::parse_context::ParseContext;
use crate::query::parser::parser::expr_parser::ExprParser;
use crate::query::parser::parser::stmt_parser::StmtParser;
use crate::query::parser::ast::stmt::Stmt;
use crate::core::types::expression::{Expression, ExpressionMeta, ExpressionContext, ContextualExpression};

/// Parser 解析结果，包含 AST 和表达式上下文
#[derive(Debug, Clone)]
pub struct ParserResult {
    /// 解析后的 AST
    pub stmt: Stmt,
    /// 表达式上下文，包含所有注册的表达式
    pub expr_context: Arc<ExpressionContext>,
}

pub struct Parser<'a> {
    ctx: ParseContext<'a>,
    expr_context: Arc<ExpressionContext>,
    _expr_parser: std::marker::PhantomData<ExprParser<'a>>,
    _stmt_parser: std::marker::PhantomData<StmtParser>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        let expr_context = Arc::new(ExpressionContext::new());
        let mut ctx = ParseContext::new(input);
        ctx.set_expression_context(expr_context.clone());

        Self {
            ctx,
            expr_context,
            _expr_parser: std::marker::PhantomData,
            _stmt_parser: std::marker::PhantomData,
        }
    }

    pub fn from_string(input: String) -> Self {
        let expr_context = Arc::new(ExpressionContext::new());
        let mut ctx = ParseContext::from_string(input);
        ctx.set_expression_context(expr_context.clone());

        Self {
            ctx,
            expr_context,
            _expr_parser: std::marker::PhantomData,
            _stmt_parser: std::marker::PhantomData,
        }
    }

    pub fn set_compat_mode(&mut self, enabled: bool) {
        self.ctx.set_compat_mode(enabled);
    }

    pub fn parse(&mut self) -> Result<ParserResult, crate::query::parser::core::error::ParseError> {
        let stmt = self.parse_statement()?;
        Ok(ParserResult {
            stmt,
            expr_context: self.expr_context.clone(),
        })
    }

    pub fn parse_statement(&mut self) -> Result<Stmt, crate::query::parser::core::error::ParseError> {
        let mut stmt_parser = StmtParser::new();
        stmt_parser.parse_statement(&mut self.ctx)
    }

    /// 解析表达式并返回 ContextualExpression
    pub fn parse_expression_contextual(&mut self) -> Result<ContextualExpression, crate::query::parser::core::error::ParseError> {
        let mut expr_parser = ExprParser::new(&self.ctx);
        expr_parser.parse_expression_with_context(&mut self.ctx, self.expr_context.clone())
    }

    /// 获取表达式上下文
    pub fn expression_context(&self) -> &Arc<ExpressionContext> {
        &self.expr_context
    }

    /// 获取表达式上下文的克隆
    pub fn expression_context_clone(&self) -> Arc<ExpressionContext> {
        self.expr_context.clone()
    }

    pub fn has_errors(&self) -> bool {
        self.ctx.has_errors()
    }

    pub fn errors(&self) -> &crate::query::parser::ParseErrors {
        self.ctx.errors()
    }

    pub fn take_errors(&mut self) -> crate::query::parser::ParseErrors {
        self.ctx.take_errors()
    }
}

/// 从字符串解析表达式元数据
///
/// # 参数
/// - `input`: 表达式字符串
///
/// # 返回
/// 解析成功返回 `Arc<ExpressionMeta>`，解析失败返回错误
pub fn parse_expression_meta_from_string(input: &str) -> Result<Arc<ExpressionMeta>, crate::query::parser::core::error::ParseError> {
    let mut parser = Parser::new(input);
    let expr = parser.parse_expression_contextual()?;
    expr.expression().ok_or_else(|| {
        crate::query::parser::core::error::ParseError::new(
            crate::query::parser::core::error::ParseErrorKind::InvalidExpression,
            crate::core::types::Position::default(),
        )
    })
}

/// 从字符串解析表达式元数据（带缓存）
///
/// # 参数
/// - `input`: 表达式字符串
/// - `cache`: 缓存 ExpressionContext
///
/// # 返回
/// 解析成功返回 `Arc<ExpressionMeta>`，解析失败返回错误
pub fn parse_expression_meta_from_string_with_cache(
    input: &str,
    cache: Arc<ExpressionContext>,
) -> Result<Arc<ExpressionMeta>, crate::query::parser::core::error::ParseError> {
    let mut parser = Parser::new(input);
    let expr = parser.parse_expression_contextual()?;
    expr.expression().ok_or_else(|| {
        crate::query::parser::core::error::ParseError::new(
            crate::query::parser::core::error::ParseErrorKind::InvalidExpression,
            crate::core::types::Position::default(),
        )
    })
}
