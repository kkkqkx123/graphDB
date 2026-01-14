//! 管理语句模块
//!
//! 包含所有管理相关的语句：用户、权限、进程控制

use crate::query::parser::ast::*;

/// CREATE USER 语句解析器
pub trait CreateUserParser {
    fn parse_create_user_statement(&mut self) -> Result<Stmt, ParseError>;
}

/// DROP USER 语句解析器
pub trait DropUserParser {
    fn parse_drop_user_statement(&mut self) -> Result<Stmt, ParseError>;
}

/// ALTER USER 语句解析器
pub trait AlterUserParser {
    fn parse_alter_user_statement(&mut self) -> Result<Stmt, ParseError>;
}

/// GRANT 语句解析器
pub trait GrantParser {
    fn parse_grant_statement(&mut self) -> Result<Stmt, ParseError>;
}

/// REVOKE 语句解析器
pub trait RevokeParser {
    fn parse_revoke_statement(&mut self) -> Result<Stmt, ParseError>;
}

/// SHOW 语句解析器
pub trait ShowParser {
    fn parse_show_statement(&mut self) -> Result<Stmt, ParseError>;
}

/// USE 语句解析器
pub trait UseParser {
    fn parse_use_statement(&mut self) -> Result<Stmt, ParseError>;
}

/// EXPLAIN 语句解析器
pub trait ExplainParser {
    fn parse_explain_statement(&mut self) -> Result<Stmt, ParseError>;
}

/// PROFILE 语句解析器
pub trait ProfileParser {
    fn parse_profile_statement(&mut self) -> Result<Stmt, ParseError>;
}
