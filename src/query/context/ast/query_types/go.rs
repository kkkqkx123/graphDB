//! GO查询上下文

use crate::query::context::ast::{
    AstContext, ExpressionProps, FromType, Over, Starts, StepClause, YieldColumns,
};

// GO查询上下文
#[derive(Debug, Clone)]
pub struct GoContext {
    pub base: AstContext,
    pub from: Starts,
    pub steps: StepClause,
    pub over: Over,
    pub filter: Option<String>,
    pub yield_expr: Option<YieldColumns>,
    pub distinct: bool,
    pub random: bool,
    pub limits: Vec<i64>,
    pub col_names: Vec<String>,
    pub vids_var: String,
    pub join_input: bool,
    pub join_dst: bool,
    pub is_simple: bool,
    pub expr_props: ExpressionProps,
    pub dst_props_expr: Option<YieldColumns>,
    pub src_props_expr: Option<YieldColumns>,
    pub edge_props_expr: Option<YieldColumns>,
    pub src_vid_col_name: String,
    pub dst_vid_col_name: String,
    pub input_var_name: String,
}

impl GoContext {
    pub fn new(base: AstContext) -> Self {
        Self {
            base,
            from: Starts::new(FromType::default()),
            steps: StepClause::new(),
            over: Over::new(),
            filter: None,
            yield_expr: None,
            distinct: false,
            random: false,
            limits: Vec::new(),
            col_names: Vec::new(),
            vids_var: String::new(),
            join_input: false,
            join_dst: false,
            is_simple: false,
            expr_props: ExpressionProps::default(),
            dst_props_expr: None,
            src_props_expr: None,
            edge_props_expr: None,
            src_vid_col_name: String::new(),
            dst_vid_col_name: String::new(),
            input_var_name: String::new(),
        }
    }
}
