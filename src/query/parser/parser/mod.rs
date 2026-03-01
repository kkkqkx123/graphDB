//! 解析器模块
//!
//! 负责解析查询语句的顶层结构，包括语句、表达式、模式等。

mod expr_parser;
mod stmt_parser;
mod parse_context;
mod parser;

// 子模块解析器
mod clause_parser;
mod ddl_parser;
mod dml_parser;
mod traversal_parser;
mod user_parser;
mod util_stmt_parser;

#[cfg(test)]
mod tests;

pub use expr_parser::ExprParser;
pub use stmt_parser::StmtParser;
pub use parse_context::ParseContext;
pub use parser::{Parser, parse_expression_meta_from_string, parse_expression_meta_from_string_with_cache, ParserResult};

// 导出子模块解析器
pub use clause_parser::ClauseParser;
pub use ddl_parser::DdlParser;
pub use dml_parser::DmlParser;
pub use traversal_parser::TraversalParser;
pub use user_parser::UserParser;
pub use util_stmt_parser::UtilStmtParser;
