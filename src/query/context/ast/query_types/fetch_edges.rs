//! Fetch Edges查询上下文

use crate::query::context::ast::{AstContext, ExpressionProps, YieldColumns};

/// Fetch Edges查询上下文
///
/// 获取边数据的查询上下文
#[derive(Debug, Clone)]
pub struct FetchEdgesContext {
    pub base: AstContext,
    pub src: Option<String>,
    pub dst: Option<String>,
    pub rank: Option<String>,
    pub edge_type: Option<String>,
    pub yield_expression: Option<YieldColumns>,
    pub edge_name: String,
    pub distinct: bool,
    pub expr_props: ExpressionProps,
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
            yield_expression: None,
            edge_name: String::new(),
            distinct: false,
            expr_props: ExpressionProps::default(),
            input_var_name: String::new(),
        }
    }
}
