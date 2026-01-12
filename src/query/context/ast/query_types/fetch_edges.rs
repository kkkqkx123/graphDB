//! Fetch Edges查询上下文

use crate::query::context::ast::{AstContext, ExpressionProps, YieldColumns};

// Fetch Edges查询上下文
#[derive(Debug, Clone)]
pub struct FetchEdgesContext {
    pub base: AstContext,
    pub src: Option<String>,
    pub dst: Option<String>,
    pub rank: Option<String>,
    pub edge_type: Option<String>,
    pub expr_props: ExpressionProps,
    pub yield_expr: Option<YieldColumns>,
    pub edge_name: String,
    pub distinct: bool,
    pub input_var_name: String,
}

impl FetchEdgesContext {
    pub fn new(base: AstContext) -> Self {
        Self {
            base,
            src: None,
            dst: None,
            rank: None,
            edge_type: None,
            expr_props: ExpressionProps::default(),
            yield_expr: None,
            edge_name: String::new(),
            distinct: false,
            input_var_name: String::new(),
        }
    }
}
