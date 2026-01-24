//! Fetch Vertices查询上下文

use crate::query::context::ast::query_types::TraverseContext;
use crate::query::context::ast::YieldColumns;

/// Fetch Vertices查询上下文
///
/// 获取点数据的查询上下文
#[derive(Debug, Clone)]
pub struct FetchVerticesContext {
    pub traverse: TraverseContext,
    pub distinct: bool,
    pub yield_expression: Option<YieldColumns>,
}

impl FetchVerticesContext {
    pub fn new(base: crate::query::context::ast::AstContext) -> Self {
        Self {
            traverse: TraverseContext::new(base),
            distinct: false,
            yield_expression: None,
        }
    }
}
