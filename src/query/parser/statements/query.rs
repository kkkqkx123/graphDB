//! 查询语句模块
//!
//! 包含所有查询相关的语句：MATCH, GO, LOOKUP, FETCH

use crate::query::parser::ast::*;

/// MATCH 语句解析器
pub trait MatchParser {
    fn parse_match_statement(&mut self) -> Result<Stmt, ParseError>;
}

/// GO 语句解析器
pub trait GoParser {
    fn parse_go_statement(&mut self) -> Result<Stmt, ParseError>;
}

/// LOOKUP 语句解析器
pub trait LookupParser {
    fn parse_lookup_statement(&mut self) -> Result<Stmt, ParseError>;
}

/// FETCH 语句解析器
pub trait FetchParser {
    fn parse_fetch_statement(&mut self) -> Result<Stmt, ParseError>;
}

/// FIND PATH 语句解析器
pub trait FindPathParser {
    fn parse_find_path_statement(&mut self) -> Result<Stmt, ParseError>;
}
