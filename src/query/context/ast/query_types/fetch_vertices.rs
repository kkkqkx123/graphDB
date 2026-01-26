//! Fetch Vertices查询上下文

use crate::core::types::expression::Expression;
use crate::query::context::ast::{AstContext, ExpressionProps, FromType, Over, Starts, StepClause, YieldColumns};

/// Fetch Vertices查询上下文
///
/// 获取点数据的查询上下文
#[derive(Debug, Clone)]
pub struct FetchVerticesContext {
    pub base: AstContext,
    pub from: Starts,
    pub steps: StepClause,
    pub over: Over,
    pub filter: Option<Expression>,
    pub col_names: Vec<String>,
    pub expr_props: ExpressionProps,
    pub input_var_name: String,
    pub distinct: bool,
    pub yield_expression: Option<YieldColumns>,
}

impl FetchVerticesContext {
    pub fn new(base: AstContext) -> Self {
        Self {
            base,
            from: Starts::new(FromType::default()),
            steps: StepClause::new(),
            over: Over::new(),
            filter: None,
            col_names: Vec::new(),
            expr_props: ExpressionProps::default(),
            input_var_name: String::new(),
            distinct: false,
            yield_expression: None,
        }
    }
}
