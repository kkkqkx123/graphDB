//! GO查询上下文

use crate::core::types::expression::Expression;
use crate::query::context::ast::{AstContext, ExpressionProps, FromType, Over, Starts, StepClause, YieldColumns};

/// GO查询上下文
///
/// GO遍历查询的上下文信息，包含：
/// - 公共遍历字段
/// - Yield表达式
/// - 查询选项（distinct, random, limits等）
/// - 属性表达式（src, dst, edge）
/// - VID列名
#[derive(Debug, Clone)]
pub struct GoContext {
    pub base: AstContext,
    pub from: Starts,
    pub steps: StepClause,
    pub over: Over,
    pub filter: Option<Expression>,
    pub col_names: Vec<String>,
    pub expr_props: ExpressionProps,
    pub input_var_name: String,
    pub yield_expression: Option<YieldColumns>,
    pub distinct: bool,
    pub random: bool,
    pub limits: Vec<i64>,
    pub vids_var: String,
    pub join_input: bool,
    pub join_dst: bool,
    pub is_simple: bool,
    pub dst_props_expression: Option<YieldColumns>,
    pub src_props_expression: Option<YieldColumns>,
    pub edge_props_expression: Option<YieldColumns>,
    pub src_vid_col_name: String,
    pub dst_vid_col_name: String,
}

impl GoContext {
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
            yield_expression: None,
            distinct: false,
            random: false,
            limits: Vec::new(),
            vids_var: String::new(),
            join_input: false,
            join_dst: false,
            is_simple: false,
            dst_props_expression: None,
            src_props_expression: None,
            edge_props_expression: None,
            src_vid_col_name: String::new(),
            dst_vid_col_name: String::new(),
        }
    }
}
