//! MATCH子句执行器模块
//!
//! 提供完整的MATCH子句执行功能，包括模式匹配、路径遍历和结果构建

pub mod expression_evaluator;
pub mod path_info;
pub mod pattern_matcher;
pub mod result_builder;
pub mod traversal_engine;

// 重新导出主要类型
pub use expression_evaluator::ExpressionEvaluator;
pub use path_info::PathInfo;
pub use pattern_matcher::PatternMatcher;
pub use result_builder::{PathAnalysis, ResultBuilder};
pub use traversal_engine::TraversalEngine;
