//! Lookup查询上下文

use crate::core::types::expression::Expr;
use crate::query::context::ast::{AstContext, YieldColumns};

/// Lookup查询上下文
///
/// 索引查询的上下文信息，包含：
/// - 模式信息 (is_edge, schema_id)
/// - 过滤条件 (filter) - 使用 Expression AST
/// - 全文索引 (is_fulltext_index, fulltext_expr)
/// - 输出配置 (yield_expr, idx_return_cols)
#[derive(Debug, Clone)]
pub struct LookupContext {
    pub base: AstContext,
    pub is_edge: bool,
    pub dedup: bool,
    pub schema_id: i32,
    pub filter: Option<Expression>,
    pub yield_expr: Option<YieldColumns>,
    pub idx_return_cols: Vec<String>,
    pub idx_col_names: Vec<String>,
    pub is_fulltext_index: bool,
    pub has_score: bool,
    pub fulltext_expr: Option<Expression>,
}

impl LookupContext {
    pub fn new(base: AstContext) -> Self {
        Self {
            base,
            is_edge: false,
            dedup: false,
            schema_id: -1,
            filter: None,
            yield_expr: None,
            idx_return_cols: Vec::new(),
            idx_col_names: Vec::new(),
            is_fulltext_index: false,
            has_score: false,
            fulltext_expr: None,
        }
    }
}
