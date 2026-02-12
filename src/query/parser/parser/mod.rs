//! 解析器模块
//!
//! 负责解析查询语句的顶层结构，包括语句、表达式、模式等。

mod expr_parser;
mod stmt_parser;
mod parse_context;
mod parser;

#[cfg(test)]
mod tests;

pub use expr_parser::ExprParser;
pub use stmt_parser::StmtParser;
pub use parse_context::ParseContext;
pub use parser::{Parser, parse_expression_meta_from_string};
