//! Path查询上下文

use crate::core::types::expression::Expression;
use crate::query::context::ast::{AstContext, ExpressionProps, FromType, Over, Starts, StepClause};

/// Path查询上下文
///
/// 路径查询的上下文信息，包含：
/// - 公共遍历字段
/// - 目标点信息 (to)
/// - 路径查找选项（最短路径、权重等）
/// - 运行时计划节点
#[derive(Debug, Clone)]
pub struct PathContext {
    pub base: AstContext,
    pub from: Starts,
    pub to: Starts,
    pub steps: StepClause,
    pub over: Over,
    pub filter: Option<Expression>,
    pub col_names: Vec<String>,
    pub expr_props: ExpressionProps,
    pub input_var_name: String,
    pub limit: i64,
    pub from_vids_var: String,
    pub to_vids_var: String,
    pub is_shortest: bool,
    pub single_shortest: bool,
    pub is_weight: bool,
    pub weight_expression: Option<String>,
    pub heuristic_expression: Option<String>,
    pub no_loop: bool,
    pub with_prop: bool,
    pub runtime_from_project: Option<String>,
    pub runtime_from_dedup: Option<String>,
    pub runtime_to_project: Option<String>,
    pub runtime_to_dedup: Option<String>,
}

impl PathContext {
    pub fn new(base: AstContext) -> Self {
        Self {
            base,
            from: Starts::new(FromType::default()),
            to: Starts::new(FromType::default()),
            steps: StepClause::new(),
            over: Over::new(),
            filter: None,
            col_names: Vec::new(),
            expr_props: ExpressionProps::default(),
            input_var_name: String::new(),
            limit: -1,
            from_vids_var: String::new(),
            to_vids_var: String::new(),
            is_shortest: false,
            single_shortest: false,
            is_weight: false,
            weight_expression: None,
            heuristic_expression: None,
            no_loop: false,
            with_prop: false,
            runtime_from_project: None,
            runtime_from_dedup: None,
            runtime_to_project: None,
            runtime_to_dedup: None,
        }
    }
}
