//! Query parser module for graph database
//!
//! This module provides functionality to parse query strings into abstract syntax trees (AST)
//! that can be processed by query execution pipeline.

pub mod core;
pub mod lexer;
pub mod ast;
pub mod parser;

// 重新导出 core 模块的常用类型
pub use core::{ParseError, ParseErrors, Token, TokenKind};
pub use crate::core::types::{Position, Span};

// 重新导出 AST 中的类型
pub use ast::{
    LimitClause, OrderDirection, OrderByClause, OrderByItem, SampleClause,
    SetClause, SkipClause, Steps, YieldClause, YieldItem,
};

// 重新导出解析器
pub use parser::Parser;
pub use parser::ExprParser;
pub use parser::StmtParser;
pub use parser::ParserResult;
