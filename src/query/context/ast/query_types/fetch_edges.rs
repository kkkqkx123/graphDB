//! Fetch Edges查询上下文

use crate::query::context::ast::query_types::TraverseContext;
use crate::query::context::ast::YieldColumns;

/// Fetch Edges查询上下文
///
/// 获取边数据的查询上下文
#[derive(Debug, Clone)]
pub struct FetchEdgesContext {
    pub traverse: TraverseContext,
    pub src: Option<String>,
    pub dst: Option<String>,
    pub rank: Option<String>,
    pub edge_type: Option<String>,
    pub yield_expr: Option<YieldColumns>,
    pub edge_name: String,
    pub distinct: bool,
}

impl FetchEdgesContext {
    pub fn new(base: crate::query::context::ast::AstContext) -> Self {
        Self {
            traverse: TraverseContext::new(base),
            src: None,
            dst: None,
            rank: None,
            edge_type: None,
            yield_expr: None,
            edge_name: String::new(),
            distinct: false,
        }
    }
}
