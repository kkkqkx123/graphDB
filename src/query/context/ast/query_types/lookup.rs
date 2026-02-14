//! Lookup查询上下文

use crate::core::types::expression::Expression;
use crate::query::context::ast::{AstContext, YieldColumns};

/// Lookup查询上下文
///
/// 索引查询的上下文信息，包含：
/// - 公共遍历字段
/// - 模式信息 (is_edge, schema_id)
/// - 过滤条件 (filter) - 使用 Expression AST
/// - 输出配置 (yield_expression, idx_return_cols)
#[derive(Debug, Clone)]
pub struct LookupContext {
    pub base: AstContext,
    pub filter: Option<Expression>,
    pub is_edge: bool,
    pub dedup: bool,
    pub schema_id: i32,
    pub yield_expression: Option<YieldColumns>,
    pub idx_return_cols: Vec<String>,
    pub idx_col_names: Vec<String>,
}

impl LookupContext {
    pub fn new(base: AstContext) -> Self {
        Self {
            base,
            filter: None,
            is_edge: false,
            dedup: false,
            schema_id: -1,
            yield_expression: None,
            idx_return_cols: Vec::new(),
            idx_col_names: Vec::new(),
        }
    }
}
