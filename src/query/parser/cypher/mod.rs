//! Cypher查询解析器模块
//!
//! 支持Cypher查询语言的解析和执行

pub mod ast;
pub mod lexer;
pub mod parser;

// 新增的解析器模块
pub mod clause_parser;
pub mod cypher_processor;
pub mod expression_converter;
pub mod expression_evaluator;
pub mod expression_optimizer;
pub mod expression_parser;
pub mod parser_core;
pub mod pattern_parser;
pub mod statement_parser;

// 重新导出主要类型
pub use ast::CypherStatement;
pub use lexer::CypherLexer;
pub use parser::{CypherParser, ParseResult, ParserInfo};

// 重新导出ParseError
pub use crate::query::parser::ParseError;

// 重新导出Cypher表达式相关类型
pub use cypher_processor::CypherProcessor;
pub use expression_converter::ExpressionConverter;
pub use expression_evaluator::CypherEvaluator;
pub use expression_optimizer::CypherExpressionOptimizer;

// 重新导出新的 Cypher 执行器
pub use crate::query::executor::cypher::CypherExecutor;
