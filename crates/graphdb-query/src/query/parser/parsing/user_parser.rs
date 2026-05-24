//! User Management Statement Parsing Module
//!
//! Responsible for parsing statements related to user management, including CREATE USER, ALTER USER, DROP USER, CHANGE PASSWORD, etc.

use crate::query::parser::ast::stmt::*;
use crate::query::parser::ast::types::Span;
use crate::query::parser::core::error::{ParseError, ParseErrorKind};
use crate::query::parser::core::token::TokenKindExt;
use crate::query::parser::parsing::parse_context::ParseContext;
use crate::query::parser::TokenKind;

/// User Management Parser
pub struct UserParser;

impl UserParser {
    pub fn new() -> Self {
        Self
    }

    /// Analysis of the CREATE USER statement
    pub fn parse_create_user_statement(
        &mut self,
        ctx: &mut ParseContext,
    ) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::CreateUser)?;
        self.parse_create_user_internal(ctx, start_span)
    }

    /// Analysis of the CREATE USER statement (the CREATE token has already been consumed)
    pub fn parse_create_user_statement_after_create(
        &mut self,
        ctx: &mut ParseContext,
        start_span: Span,
    ) -> Result<Stmt, ParseError> {
        ctx.expect_token(TokenKind::User)?;
        self.parse_create_user_internal(ctx, start_span)
    }

    /// Analyzing the internal implementation of the CREATE USER statement
    fn parse_create_user_internal(
        &mut self,
        ctx: &mut ParseContext,
        start_span: Span,
    ) -> Result<Stmt, ParseError> {
        let mut if_not_exists = false;
        if ctx.match_token(TokenKind::If) {
            ctx.expect_token(TokenKind::Not)?;
            ctx.expect_token(TokenKind::Exists)?;
            if_not_exists = true;
        }

        let username = ctx.expect_identifier()?;

        // Support for the WITH PASSWORD syntax
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

    /// Analysis of the ALTER USER statement
    pub fn parse_alter_user_statement(
        &mut self,
        ctx: &mut ParseContext,
    ) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::AlterUser)?;
        self.parse_alter_user_internal(ctx, start_span)
    }

    /// Analysis of the internal implementation of the ALTER USER command
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

        // Analyzing the WITH PASSWORD or SET clause
        if ctx.match_token(TokenKind::With) {
            if ctx.match_token(TokenKind::Password) {
                password = Some(ctx.expect_string_literal()?);
            } else if ctx.match_token(TokenKind::Role) {
                new_role = Some(ctx.expect_identifier()?);
            }
        }

        // The SET ROLE = ... and SET LOCKED = ... syntax are also supported.
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

    /// Analysis of the DROP USER statement
    pub fn parse_drop_user_statement(
        &mut self,
        ctx: &mut ParseContext,
    ) -> Result<Stmt, ParseError> {
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

    /// Analysis of the CHANGE PASSWORD statement
    pub fn parse_change_password_statement(
        &mut self,
        ctx: &mut ParseContext,
    ) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::ChangePassword)?;

        self.parse_change_password_internal(ctx, start_span)
    }

    /// Analysis of the internal implementation of the “CHANGE PASSWORD” command
    pub fn parse_change_password_internal(
        &mut self,
        ctx: &mut ParseContext,
        start_span: Span,
    ) -> Result<Stmt, ParseError> {
        // Parse the optional username (if the next token is an identifier).
        // At this point, the PASSWORD keyword has already been used (i.e., it has been “consumed” in the context of the program or code).
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

    /// Analysis of the CHANGE statement (currently only CHANGE PASSWORD is supported)
    pub fn parse_change_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Change)?;

        // Check whether it is “CHANGE PASSWORD”.
        if ctx.match_token(TokenKind::Password) {
            return self.parse_change_password_internal(ctx, start_span);
        }

        Err(ParseError::new(
            ParseErrorKind::UnexpectedToken,
            "Expected PASSWORD after CHANGE".to_string(),
            ctx.current_position(),
        ))
    }

    /// Analyzing character types (supporting keyword-based searches)
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

        role_str
            .parse::<RoleType>()
            .map_err(|e| ParseError::new(ParseErrorKind::SyntaxError, e, ctx.current_position()))
    }

    /// Analysis of the GRANT statement
    /// Syntax:  `GRANT ROLE <role_type> ON <space_name> TO <username>`
    pub fn parse_grant_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Grant)?;

        // Analyzing the ROLE keyword (optional)
        let _ = ctx.match_token(TokenKind::Role);

        // Analyzing character types
        let role = self.parse_role_type(ctx)?;

        // Analysis of the ON keyword
        ctx.expect_token(TokenKind::On)?;

        // Analysis of the Space name
        let space_name = ctx.expect_identifier()?;

        // Analysis of the TO keyword
        ctx.expect_token(TokenKind::To)?;

        // Analyzing the username
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

    /// Analysis of the REVOKE statement
    /// Syntax: `REVOKE ROLE <role_type> ON <space_name> FROM <username>`
    pub fn parse_revoke_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Revoke)?;

        // Analysis of the ROLE keyword (optional)
        let _ = ctx.match_token(TokenKind::Role);

        // Analyzing character types
        let role = self.parse_role_type(ctx)?;

        // Analysis of the ON keyword
        ctx.expect_token(TokenKind::On)?;

        // Analyzing the name “Space”
        let space_name = ctx.expect_identifier()?;

        // Analysis of the FROM keyword
        ctx.expect_token(TokenKind::From)?;

        // Analyzing the username
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

    /// Analysis of the DESCRIBE USER statement
    /// Grammar: DESCRIBE USER <username>
    pub fn parse_describe_user_statement(
        &mut self,
        ctx: &mut ParseContext,
    ) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Desc)?;
        ctx.expect_token(TokenKind::User)?;

        let username = ctx.expect_identifier()?;

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Stmt::DescribeUser(DescribeUserStmt { span, username }))
    }

    /// Analysis of the SHOW USERS statement
    /// Syntax: SHOW USERS
    pub fn parse_show_users_statement(
        &mut self,
        ctx: &mut ParseContext,
    ) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Show)?;
        ctx.expect_token(TokenKind::Users)?;

        let end_span = ctx.current_span();
        let span = ctx.merge_span(start_span.start, end_span.end);

        Ok(Stmt::ShowUsers(ShowUsersStmt { span }))
    }

    /// Analysis of the SHOW ROLES statement
    /// Syntax: SHOW ROLES [IN <space_name>]
    pub fn parse_show_roles_statement(
        &mut self,
        ctx: &mut ParseContext,
    ) -> Result<Stmt, ParseError> {
        let start_span = ctx.current_span();
        ctx.expect_token(TokenKind::Show)?;
        ctx.expect_token(TokenKind::Roles)?;

        // Optional IN <space_name> clause
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
