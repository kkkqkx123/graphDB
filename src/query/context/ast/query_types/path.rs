//! Path查询上下文

use crate::query::context::ast::query_types::TraverseContext;

/// Path查询上下文
///
/// 路径查询的上下文信息，包含：
/// - 公共遍历字段（来自 TraverseContext）
/// - 目标点信息 (to)
/// - 路径查找选项（最短路径、权重等）
/// - 运行时计划节点
#[derive(Debug, Clone)]
pub struct PathContext {
    pub traverse: TraverseContext,
    pub to: crate::query::context::ast::Starts,
    pub limit: i64,
    pub from_vids_var: String,
    pub to_vids_var: String,
    pub is_shortest: bool,
    pub single_shortest: bool,
    pub is_weight: bool,
    pub no_loop: bool,
    pub with_prop: bool,
    pub runtime_from_project: Option<String>,
    pub runtime_from_dedup: Option<String>,
    pub runtime_to_project: Option<String>,
    pub runtime_to_dedup: Option<String>,
}

impl PathContext {
    pub fn new(base: crate::query::context::ast::AstContext) -> Self {
        Self {
            traverse: TraverseContext::new(base),
            to: crate::query::context::ast::Starts::new(
                crate::query::context::ast::FromType::default(),
            ),
            limit: -1,
            from_vids_var: String::new(),
            to_vids_var: String::new(),
            is_shortest: false,
            single_shortest: false,
            is_weight: false,
            no_loop: false,
            with_prop: false,
            runtime_from_project: None,
            runtime_from_dedup: None,
            runtime_to_project: None,
            runtime_to_dedup: None,
        }
    }
}
