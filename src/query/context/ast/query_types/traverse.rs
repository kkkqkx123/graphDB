//! 遍历查询上下文基类
//!
//! 提供 PathContext 和 GoContext 的公共字段，减少代码重复

use crate::query::context::ast::{AstContext, ExpressionProps, FromType, Over, Starts, StepClause};

/// 遍历查询上下文基类
///
/// 包含 PathContext 和 GoContext 的公共字段：
/// - 起始点信息 (from)
/// - 步数限制 (steps)
/// - 边遍历规则 (over)
/// - 过滤条件 (filter)
/// - 输出列名 (col_names)
/// - 表达式属性 (expr_props)
/// - 输入变量名 (input_var_name)
#[derive(Debug, Clone)]
pub struct TraverseContext {
    pub base: AstContext,
    pub from: Starts,
    pub steps: StepClause,
    pub over: Over,
    pub filter: Option<String>,
    pub col_names: Vec<String>,
    pub expr_props: ExpressionProps,
    pub input_var_name: String,
}

impl TraverseContext {
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
        }
    }
}
