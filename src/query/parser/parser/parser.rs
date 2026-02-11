use crate::query::parser::parser::parse_context::ParseContext;
use crate::query::parser::parser::expr_parser::ExprParser;
use crate::query::parser::parser::stmt_parser::StmtParser;
use crate::query::parser::ast::stmt::Stmt;
use crate::core::types::expression::{Expression, ExpressionMeta};

pub struct Parser<'a> {
    ctx: ParseContext<'a>,
    _expr_parser: std::marker::PhantomData<ExprParser<'a>>,
    _stmt_parser: std::marker::PhantomData<StmtParser<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        let ctx = ParseContext::new(input);

        Self {
            ctx,
            _expr_parser: std::marker::PhantomData,
            _stmt_parser: std::marker::PhantomData,
        }
    }

    pub fn from_string(input: String) -> Self {
        let ctx = ParseContext::from_string(input);

        Self {
            ctx,
            _expr_parser: std::marker::PhantomData,
            _stmt_parser: std::marker::PhantomData,
        }
    }

    pub fn set_compat_mode(&mut self, enabled: bool) {
        self.ctx.set_compat_mode(enabled);
    }

    pub fn parse(&mut self) -> Result<Stmt, crate::query::parser::core::error::ParseError> {
        self.parse_statement()
    }

    pub fn parse_statement(&mut self) -> Result<Stmt, crate::query::parser::core::error::ParseError> {
        let mut stmt_parser = StmtParser::new();
        stmt_parser.parse_statement(&mut self.ctx)
    }

    pub fn parse_expression(&mut self) -> Result<Expression, crate::query::parser::core::error::ParseError> {
        let mut expr_parser = ExprParser::new(&self.ctx);
        let result = expr_parser.parse_expression(&mut self.ctx)?;
        Ok(result.expr)
    }

    pub fn parse_expression_with_span(&mut self) -> Result<ExpressionMeta, crate::query::parser::core::error::ParseError> {
        let mut expr_parser = ExprParser::new(&self.ctx);
        let result = expr_parser.parse_expression(&mut self.ctx)?;
        Ok(ExpressionMeta::with_span(result.expr, result.span))
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

pub fn parse_expression_meta_from_string(condition: &str) -> Result<ExpressionMeta, String> {
    let mut parser = Parser::new(condition);
    let core_expression = parser
        .parse_expression()
        .map_err(|e| format!("语法分析错误: {:?}", e))?;
    Ok(ExpressionMeta::new(core_expression))
}
