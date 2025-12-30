//! Fetch Vertices查询上下文

use crate::query::context::ast::{AstContext, ExpressionProps, FromType, Starts, YieldColumns};

// Fetch Vertices查询上下文
#[derive(Debug, Clone)]
pub struct FetchVerticesContext {
    pub base: AstContext,
    pub from: Starts,
    pub distinct: bool,
    pub yield_expr: Option<YieldColumns>,
    pub expr_props: ExpressionProps,
    pub input_var_name: String,
}

impl FetchVerticesContext {
    pub fn new(base: AstContext) -> Self {
        Self {
            base,
            from: Starts::new(FromType::default()),
            distinct: false,
            yield_expr: None,
            expr_props: ExpressionProps::default(),
            input_var_name: String::new(),
        }
    }
}
