//! 用户管理语句解析模块
//!
//! 负责解析用户管理相关语句，包括 CREATE USER、ALTER USER、DROP USER、CHANGE PASSWORD 等。

use crate::query::parser::ast::stmt::*;
use crate::query::parser::ast::types::Span;
use crate::query::parser::core::error::{ParseError, ParseErrorKind};
use crate::query::parser::core::token::TokenKindExt;
use crate::query::parser::parser::parse_context::ParseContext;
use crate::query::parser::TokenKind;

/// 用户管理解析器
pub struct UserParser;

impl UserParser {
    pub fn new() -> Self {
        Self
    }

    /// 解析 CREATE USER 语句
    pub fn parse_create_user_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::CreateUser)?;
        self.parse_create_user_internal(ctx, start_span)
    }

    /// 解析 CREATE USER 语句（CREATE token 已被消费）
    pub fn parse_create_user_statement_after_create(&mut self, ctx: &mut ParseContext, start_span: Span) -> Result<Stmt, ParseError> {
        ctx.expect_token(TokenKind::User)?;
        self.parse_create_user_internal(ctx, start_span)
    }

    /// 解析 CREATE USER 语句的内部实现
    fn parse_create_user_internal(&mut self, ctx: &mut ParseContext, start_span: Span) -> Result<Stmt, ParseError> {
        let mut if_not_exists = false;
        if ctx.match_token(TokenKind::If) {
            ctx.expect_token(TokenKind::Not)?;
            ctx.expect_token(TokenKind::Exists)?;
            if_not_exists = true;
        }

        let username = ctx.expect_identifier()?;
        
        // 支持 WITH PASSWORD 语法
        ctx.match_token(TokenKind::With);
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

    /// 解析 ALTER USER 语句
    pub fn parse_alter_user_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::AlterUser)?;
        self.parse_alter_user_internal(ctx, start_span)
    }

    /// 解析 ALTER USER 内部实现
    pub fn parse_alter_user_internal(
        &mut self,
        ctx: &mut ParseContext,
        start_span: Span,
    ) -> Result<Stmt, ParseError> {
        ctx.expect_token(TokenKind::User)?;

        let username = ctx.expect_identifier()?;

        let mut password = None;
        let mut new_role = None;
        let mut is_locked = None;

        // 解析 WITH PASSWORD 或 SET 子句
        if ctx.match_token(TokenKind::With) {
            if ctx.match_token(TokenKind::Password) {
                password = Some(ctx.expect_string_literal()?);
            } else if ctx.match_token(TokenKind::Role) {
                new_role = Some(ctx.expect_identifier()?);
            }
        }

        // 也支持 SET ROLE = ... 和 SET LOCKED = ... 语法
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
            password,
            new_role,
            is_locked,
        }))
    }

    /// 解析 DROP USER 语句
    pub fn parse_drop_user_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
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

    /// 解析 CHANGE PASSWORD 语句
    pub fn parse_change_password_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::ChangePassword)?;

        self.parse_change_password_internal(ctx, start_span)
    }

    /// 解析 CHANGE PASSWORD 内部实现
    pub fn parse_change_password_internal(
        &mut self,
        ctx: &mut ParseContext,
        start_span: Span,
    ) -> Result<Stmt, ParseError> {
        // 解析可选的用户名（如果下一个 token 是标识符）
        // 注意：此时 PASSWORD 关键字已经被消费
        let username = if ctx.current_token().kind.is_identifier() {
            Some(ctx.expect_identifier()?)
        } else {
            None
        };

        let old_password = ctx.expect_string_literal()?;
        ctx.expect_token(TokenKind::To)?;
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

    /// 解析 CHANGE 语句（目前只支持 CHANGE PASSWORD）
    pub fn parse_change_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Change)?;

        // 检查是否是 CHANGE PASSWORD
        if ctx.match_token(TokenKind::Password) {
            return self.parse_change_password_internal(ctx, start_span);
        }

        Err(ParseError::new(
            ParseErrorKind::UnexpectedToken,
            "Expected PASSWORD after CHANGE".to_string(),
            ctx.current_position(),
        ))
    }

    /// 解析角色类型（支持关键字形式）
    fn parse_role_type(&mut self, ctx: &mut ParseContext) -> Result<RoleType, ParseError> {
        let token = ctx.current_token();
        let role_str = match token.kind {
            TokenKind::God => {
                ctx.next_token();
                "GOD".to_string()
            }
            TokenKind::Admin | TokenKind::AdminRole => {
                ctx.next_token();
                "ADMIN".to_string()
            }
            TokenKind::Dba => {
                ctx.next_token();
                "DBA".to_string()
            }
            TokenKind::Guest => {
                ctx.next_token();
                "GUEST".to_string()
            }
            TokenKind::User => {
                ctx.next_token();
                "USER".to_string()
            }
            _ => ctx.expect_identifier()?,
        };

        role_str.parse::<RoleType>().map_err(|e| {
            ParseError::new(
                ParseErrorKind::SyntaxError,
                e,
                ctx.current_position(),
            )
        })
    }

    /// 解析 GRANT 语句
    /// 语法: GRANT ROLE <role_type> ON <space_name> TO <username>
    pub fn parse_grant_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Grant)?;

        // 解析 ROLE 关键字（可选）
        let _ = ctx.match_token(TokenKind::Role);

        // 解析角色类型
        let role = self.parse_role_type(ctx)?;

        // 解析 ON 关键字
        ctx.expect_token(TokenKind::On)?;

        // 解析 Space 名称
        let space_name = ctx.expect_identifier()?;

        // 解析 TO 关键字
        ctx.expect_token(TokenKind::To)?;

        // 解析用户名
        let username = ctx.expect_identifier()?;

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Stmt::Grant(GrantStmt {
            span,
            role,
            space_name,
            username,
        }))
    }

    /// 解析 REVOKE 语句
    /// 语法: REVOKE ROLE <role_type> ON <space_name> FROM <username>
    pub fn parse_revoke_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Revoke)?;

        // 解析 ROLE 关键字（可选）
        let _ = ctx.match_token(TokenKind::Role);

        // 解析角色类型
        let role = self.parse_role_type(ctx)?;

        // 解析 ON 关键字
        ctx.expect_token(TokenKind::On)?;

        // 解析 Space 名称
        let space_name = ctx.expect_identifier()?;

        // 解析 FROM 关键字
        ctx.expect_token(TokenKind::From)?;

        // 解析用户名
        let username = ctx.expect_identifier()?;

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Stmt::Revoke(RevokeStmt {
            span,
            role,
            space_name,
            username,
        }))
    }

    /// 解析 DESCRIBE USER 语句
    /// 语法: DESCRIBE USER <username>
    pub fn parse_describe_user_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Desc)?;
        ctx.expect_token(TokenKind::User)?;

        let username = ctx.expect_identifier()?;

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Stmt::DescribeUser(DescribeUserStmt {
            span,
            username,
        }))
    }

    /// 解析 SHOW USERS 语句
    /// 语法: SHOW USERS
    pub fn parse_show_users_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Show)?;
        ctx.expect_token(TokenKind::Users)?;

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Stmt::ShowUsers(ShowUsersStmt { span }))
    }

    /// 解析 SHOW ROLES 语句
    /// 语法: SHOW ROLES [IN <space_name>]
    pub fn parse_show_roles_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Show)?;
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
}

impl Default for UserParser {
    fn default() -> Self {
        Self::new()
    }
}
