//! Subgraph查询上下文

use crate::query::context::ast::{AstContext, ExpressionProps, FromType, Starts, StepClause};
use std::collections::HashSet;

// Subgraph查询上下文
#[derive(Debug, Clone)]
pub struct SubgraphContext {
    pub base: AstContext,
    pub from: Starts,
    pub steps: StepClause,
    pub loop_steps: String,
    pub filter: Option<String>,
    pub tag_filter: Option<String>,
    pub edge_filter: Option<String>,
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
