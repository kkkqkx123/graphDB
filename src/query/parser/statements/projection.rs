//! 投影语句模块
//!
//! 包含所有投影相关的语句：RETURN, WITH, YIELD

use crate::query::parser::ast::*;

/// RETURN 语句解析器
pub trait ReturnStmtParser {
    fn parse_return_statement(&mut self) -> Result<Stmt, ParseError>;
}

/// WITH 语句解析器
pub trait WithStmtParser {
    fn parse_with_statement(&mut self) -> Result<Stmt, ParseError>;
}

/// YIELD 语句解析器
pub trait YieldStmtParser {
    fn parse_yield_statement(&mut self) -> Result<Stmt, ParseError>;
}
