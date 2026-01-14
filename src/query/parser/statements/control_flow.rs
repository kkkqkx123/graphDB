//! 控制流语句模块
//!
//! 包含所有控制流相关的语句：UNWIND, PIPE

use crate::query::parser::ast::*;

/// UNWIND 语句解析器
pub trait UnwindParser {
    fn parse_unwind_statement(&mut self) -> Result<Stmt, ParseError>;
}

/// PIPE 语句解析器
pub trait PipeParser {
    fn parse_pipe_statement(&mut self) -> Result<Stmt, ParseError>;
}
