//! Subgraph查询上下文

use crate::core::types::expression::Expression;
use crate::query::context::ast::{AstContext, ExpressionProps, FromType, Starts, StepClause};
use std::collections::HashSet;

/// Subgraph查询上下文
///
/// 子图查询的上下文信息，包含：
/// - 起始点信息 (from)
/// - 步数限制 (steps)
/// - 过滤条件 (filter, tag_filter, edge_filter) - 使用 Expression AST
/// - 边集合 (edge_names, edge_types)
/// - 输出配置
#[derive(Debug, Clone)]
pub struct SubgraphContext {
    pub base: AstContext,
    pub from: Starts,
    pub steps: StepClause,
    pub loop_steps: String,
    pub filter: Option<Expression>,
    pub tag_filter: Option<Expression>,
    pub edge_filter: Option<Expression>,
    pub col_names: Vec<String>,
    pub edge_names: HashSet<String>,
    pub edge_types: HashSet<String>,
    pub bi_direct_edge_types: HashSet<String>,
    pub col_type: Vec<String>,
    pub expr_props: ExpressionProps,
    pub with_prop: bool,
    pub get_vertex_prop: bool,
    pub get_edge_prop: bool,
}

impl SubgraphContext {
    pub fn new(base: AstContext) -> Self {
        Self {
            base,
            from: Starts::new(FromType::default()),
            steps: StepClause::new(),
            loop_steps: String::new(),
            filter: None,
            tag_filter: None,
            edge_filter: None,
            col_names: Vec::new(),
            edge_names: HashSet::new(),
            edge_types: HashSet::new(),
            bi_direct_edge_types: HashSet::new(),
            col_type: Vec::new(),
            expr_props: ExpressionProps::default(),
            with_prop: false,
            get_vertex_prop: false,
            get_edge_prop: false,
        }
    }
}
