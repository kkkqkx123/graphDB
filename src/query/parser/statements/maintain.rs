//! 维护语句模块
//!
//! 包含所有维护相关的语句：CREATE/ALTER/DROP SPACE/TAG/EDGE/INDEX

use crate::query::parser::ast::*;

/// CREATE SPACE 语句解析器
pub trait CreateSpaceParser {
    fn parse_create_space_statement(&mut self) -> Result<Stmt, ParseError>;
}

/// DROP SPACE 语句解析器
pub trait DropSpaceParser {
    fn parse_drop_space_statement(&mut self) -> Result<Stmt, ParseError>;
}

/// ALTER SPACE 语句解析器
pub trait AlterSpaceParser {
    fn parse_alter_space_statement(&mut self) -> Result<Stmt, ParseError>;
}

/// CREATE TAG 语句解析器
pub trait CreateTagParser {
    fn parse_create_tag_statement(&mut self) -> Result<Stmt, ParseError>;
}

/// DROP TAG 语句解析器
pub trait DropTagParser {
    fn parse_drop_tag_statement(&mut self) -> Result<Stmt, ParseError>;
}

/// ALTER TAG 语句解析器
pub trait AlterTagParser {
    fn parse_alter_tag_statement(&mut self) -> Result<Stmt, ParseError>;
}

/// CREATE EDGE 语句解析器
pub trait CreateEdgeParser {
    fn parse_create_edge_statement(&mut self) -> Result<Stmt, ParseError>;
}

/// DROP EDGE 语句解析器
pub trait DropEdgeParser {
    fn parse_drop_edge_statement(&mut self) -> Result<Stmt, ParseError>;
}

/// ALTER EDGE 语句解析器
pub trait AlterEdgeParser {
    fn parse_alter_edge_statement(&mut self) -> Result<Stmt, ParseError>;
}

/// CREATE INDEX 语句解析器
pub trait CreateIndexParser {
    fn parse_create_index_statement(&mut self) -> Result<Stmt, ParseError>;
}

/// DROP INDEX 语句解析器
pub trait DropIndexParser {
    fn parse_drop_index_statement(&mut self) -> Result<Stmt, ParseError>;
}
