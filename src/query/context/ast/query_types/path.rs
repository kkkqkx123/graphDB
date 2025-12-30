//! Path查询上下文

use crate::query::context::ast::{AstContext, ExpressionProps, FromType, Over, Starts, StepClause};

// Path查询上下文
#[derive(Debug, Clone)]
pub struct PathContext {
    pub base: AstContext,
    pub from: Starts,
    pub to: Starts,
    pub steps: StepClause,
    pub over: Over,
    pub limit: i64,
    pub filter: Option<String>,
    pub col_names: Vec<String>,
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
    pub input_var_name: String,
    pub expr_props: ExpressionProps,
}

impl PathContext {
    pub fn new(base: AstContext) -> Self {
        Self {
            base,
            from: Starts::new(FromType::default()),
            to: Starts::new(FromType::default()),
            steps: StepClause::new(),
            over: Over::new(),
            limit: -1,
            filter: None,
            col_names: Vec::new(),
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
            input_var_name: String::new(),
            expr_props: ExpressionProps::default(),
        }
    }
}
