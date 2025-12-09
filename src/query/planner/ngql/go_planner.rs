//! GO语句规划器
//! 处理Nebula GO查询的规划

use crate::query::context::AstContext;
use crate::query::planner::planner::{Planner, PlannerError};
use crate::query::planner::plan::SubPlan;

/// GO查询规划器
/// 负责将GO语句转换为执行计划
#[derive(Debug)]
pub struct GoPlanner;

impl GoPlanner {
    /// 创建新的GO规划器
    pub fn new() -> Self {
        Self
    }

    /// 创建规划器实例的工厂函数
    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    /// 检查AST上下文是否匹配GO查询
    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "GO"
    }

    /// 获取匹配和实例化函数
    pub fn get_match_and_instantiate() -> crate::query::planner::planner::MatchAndInstantiate {
        crate::query::planner::planner::MatchAndInstantiate {
            match_func: Self::match_ast_ctx,
            instantiate_func: Self::make,
        }
    }
}

impl Planner for GoPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // TODO: 实现GO查询的规划逻辑
        Err(PlannerError::UnsupportedOperation(
            "GO query planning not yet implemented".to_string(),
        ))
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

impl Default for GoPlanner {
    fn default() -> Self {
        Self::new()
    }
}
