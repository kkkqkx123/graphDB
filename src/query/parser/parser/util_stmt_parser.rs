//! 工具语句解析模块
//!
//! 负责解析工具类语句，包括 USE、SHOW、EXPLAIN、FETCH、LOOKUP、UNWIND、RETURN、WITH、YIELD、SET、REMOVE 等。

use crate::core::types::expression::Expression as CoreExpression;
use crate::query::parser::ast::stmt::*;
use crate::query::parser::core::error::{ParseError, ParseErrorKind};
use crate::query::parser::parser::clause_parser::ClauseParser;
use crate::query::parser::parser::ExprParser;
use crate::query::parser::parser::parse_context::ParseContext;
use crate::query::parser::TokenKind;

/// 工具语句解析器
pub struct UtilStmtParser;

impl UtilStmtParser {
    pub fn new() -> Self {
        Self
    }

    /// 解析 USE 语句
    pub fn parse_use_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Use)?;

        let space = ctx.expect_identifier()?;

        Ok(Stmt::Use(UseStmt { span: start_span, space }))
    }

    /// 解析 SHOW 语句
    pub fn parse_show_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Show)?;

        // 检查 SHOW USERS
        if ctx.check_token(TokenKind::Users) {
            return self.parse_show_users_internal(ctx, start_span);
        }

        // 检查 SHOW ROLES
        if ctx.check_token(TokenKind::Roles) {
            return self.parse_show_roles_internal(ctx, start_span);
        }

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

    /// 解析 SHOW USERS 内部方法
    fn parse_show_users_internal(&mut self, ctx: &mut ParseContext, start_span: crate::query::parser::ast::types::Span) -> Result<Stmt, ParseError> {
        ctx.expect_token(TokenKind::Users)?;

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Stmt::ShowUsers(ShowUsersStmt { span }))
    }

    /// 解析 SHOW ROLES 内部方法
    fn parse_show_roles_internal(&mut self, ctx: &mut ParseContext, start_span: crate::query::parser::ast::types::Span) -> Result<Stmt, ParseError> {
        ctx.expect_token(TokenKind::Roles)?;

        // 可选的 IN <space_name> 子句
        let space_name = if ctx.match_token(TokenKind::In) {
            Some(ctx.expect_identifier()?)
        } else {
            None
        };

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Stmt::ShowRoles(ShowRolesStmt { span, space_name }))
    }

    /// 解析 EXPLAIN 语句
    pub fn parse_explain_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let _start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Explain)?;

        // EXPLAIN 后面需要解析一个子语句
        // 这里我们需要调用主解析器，但由于循环依赖问题，我们返回一个占位符
        // 实际解析将在 StmtParser 中处理
        Err(ParseError::new(
            ParseErrorKind::SyntaxError,
            "EXPLAIN should be handled by main parser".to_string(),
            ctx.current_position(),
        ))
    }

    /// 解析 FETCH 语句
    pub fn parse_fetch_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Fetch)?;

        // 支持 FETCH PROP ON <tag> <ids> 语法
        let _with_props = ctx.match_token(TokenKind::Prop);

        let target = if ctx.match_token(TokenKind::On) {
            // FETCH PROP ON <tag> <ids> 语法
            let _tag_name = ctx.expect_identifier()?;
            let ids = self.parse_expression_list(ctx)?;
            FetchTarget::Vertices {
                ids,
                properties: None,
            }
        } else if ctx.match_token(TokenKind::Tag) {
            let _tag_name = ctx.expect_identifier()?;
            let ids = self.parse_expression_list(ctx)?;
            FetchTarget::Vertices {
                ids,
                properties: None,
            }
        } else if ctx.match_token(TokenKind::Edge) {
            let edge_type = ctx.expect_identifier()?;
            let src = self.parse_expression(ctx)?;
            ctx.expect_token(TokenKind::Arrow)?;
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

    /// 解析 LOOKUP 语句
    pub fn parse_lookup_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
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
            Some(ClauseParser::new().parse_yield_clause(ctx)?)
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

    /// 解析 UNWIND 语句
    pub fn parse_unwind_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Unwind)?;

        let expression = self.parse_expression(ctx)?;
        
        ctx.match_token(TokenKind::As);
        
        let variable = ctx.expect_identifier()?;

        Ok(Stmt::Unwind(UnwindStmt {
            span: start_span,
            expression,
            variable,
        }))
    }

    /// 解析 RETURN 语句
    pub fn parse_return_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Return)?;

        let return_clause = ClauseParser::new().parse_return_clause(ctx)?;
        
        Ok(Stmt::Return(ReturnStmt {
            span: start_span,
            items: return_clause.items,
            distinct: return_clause.distinct,
        }))
    }

    /// 解析 WITH 语句
    pub fn parse_with_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::With)?;

        let mut items = Vec::new();
        let distinct = ctx.match_token(TokenKind::Distinct);

        loop {
            let expr = self.parse_expression(ctx)?;
            let alias = if ctx.match_token(TokenKind::As) {
                Some(ctx.expect_identifier()?)
            } else {
                None
            };
            items.push(ReturnItem::Expression {
                expression: expr,
                alias,
            });
            if !ctx.match_token(TokenKind::Comma) {
                break;
            }
        }

        let where_clause = if ctx.match_token(TokenKind::Where) {
            Some(self.parse_expression(ctx)?)
        } else {
            None
        };

        Ok(Stmt::With(WithStmt {
            span: start_span,
            items,
            where_clause,
            distinct,
        }))
    }

    /// 解析 YIELD 语句
    pub fn parse_yield_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Yield)?;

        let yield_clause = ClauseParser::new().parse_yield_clause(ctx)?;
        
        Ok(Stmt::Yield(YieldStmt {
            span: start_span,
            items: yield_clause.items,
            where_clause: None,
            distinct: false,
        }))
    }

    /// 解析 SET 语句
    pub fn parse_set_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Set)?;

        let set_clause = ClauseParser::new().parse_set_clause(ctx)?;
        
        Ok(Stmt::Set(SetStmt {
            span: start_span,
            assignments: set_clause.assignments,
        }))
    }

    /// 解析 REMOVE 语句
    pub fn parse_remove_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Remove)?;

        let mut items = Vec::new();
        loop {
            items.push(self.parse_expression(ctx)?);
            if !ctx.match_token(TokenKind::Comma) {
                break;
            }
        }

        Ok(Stmt::Remove(RemoveStmt {
            span: start_span,
            items,
        }))
    }

    /// 解析表达式列表
    fn parse_expression_list(&mut self, ctx: &mut ParseContext) -> Result<Vec<CoreExpression>, ParseError> {
        let mut expressions = Vec::new();
        
        loop {
            expressions.push(self.parse_expression(ctx)?);
            if !ctx.match_token(TokenKind::Comma) {
                break;
            }
        }
        
        Ok(expressions)
    }

    /// 解析表达式
    fn parse_expression(&mut self, ctx: &mut ParseContext) -> Result<CoreExpression, ParseError> {
        let mut expr_parser = ExprParser::new(ctx);
        let result = expr_parser.parse_expression(ctx)?;
        Ok(result.expr)
    }
}

impl Default for UtilStmtParser {
    fn default() -> Self {
        Self::new()
    }
}
