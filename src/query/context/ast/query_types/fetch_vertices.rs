//! Fetch Vertices查询上下文

use crate::query::context::ast::{AstContext, ExpressionProps, Starts};

// Fetch Vertices查询上下文
#[derive(Debug, Clone)]
pub struct FetchVerticesContext {
    pub base: AstContext,
    pub from: Starts,
    pub distinct: bool,
    pub yield_expr: Option<String>,
    pub expr_props: ExpressionProps,
    pub input_var_name: String,
}

impl FetchVerticesContext {
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
            distinct: false,
            yield_expr: None,
            expr_props: ExpressionProps::default(),
            input_var_name: String::new(),
        }
    }
}
