//! 变异语句模块
//!
//! 包含所有数据变异相关的语句：CREATE, DELETE, SET, INSERT, UPDATE, MERGE, REMOVE

use crate::query::parser::ast::*;

/// CREATE 语句解析器
pub trait CreateParser {
    fn parse_create_statement(&mut self) -> Result<Stmt, ParseError>;
}

/// DELETE 语句解析器
pub trait DeleteParser {
    fn parse_delete_statement(&mut self) -> Result<Stmt, ParseError>;
}

/// SET 语句解析器
pub trait SetParser {
    fn parse_set_statement(&mut self) -> Result<Stmt, ParseError>;
}

/// INSERT 语句解析器
pub trait InsertParser {
    fn parse_insert_statement(&mut self) -> Result<Stmt, ParseError>;
}

/// UPDATE 语句解析器
pub trait UpdateParser {
    fn parse_update_statement(&mut self) -> Result<Stmt, ParseError>;
}

/// MERGE 语句解析器
pub trait MergeParser {
    fn parse_merge_statement(&mut self) -> Result<Stmt, ParseError>;
}

/// REMOVE 语句解析器
pub trait RemoveParser {
    fn parse_remove_statement(&mut self) -> Result<Stmt, ParseError>;
}
