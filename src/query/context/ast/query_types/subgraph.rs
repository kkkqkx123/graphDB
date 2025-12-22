//! Subgraph查询上下文

use crate::query::context::ast::{AstContext, ExpressionProps, Starts, StepClause};

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
    pub edge_names: Vec<String>,
    pub edge_types: Vec<String>,
    pub bi_direct_edge_types: Vec<String>,
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
            from: Starts {
                from_type: "instant_expr".to_string(),
                src: None,
                original_src: None,
                user_defined_var_name: String::new(),
                runtime_vid_name: String::new(),
                vids: Vec::new(),
            },
            steps: StepClause {
                m_steps: 1,
                n_steps: 1,
                is_m_to_n: false,
            },
            loop_steps: String::new(),
            filter: None,
            tag_filter: None,
            edge_filter: None,
            col_names: Vec::new(),
            edge_names: Vec::new(),
            edge_types: Vec::new(),
            bi_direct_edge_types: Vec::new(),
            col_type: Vec::new(),
            expr_props: ExpressionProps::default(),
            with_prop: false,
            get_vertex_prop: false,
            get_edge_prop: false,
        }
    }
}

// 维护操作查询上下文
#[derive(Debug, Clone)]
pub struct MaintainContext {
    pub base: AstContext,
    // 可以根据具体维护操作添加更多字段
}
