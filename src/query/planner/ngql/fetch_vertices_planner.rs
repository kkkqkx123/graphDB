//! FETCH VERTICES查询规划器
//! 处理FETCH VERTICES查询的规划

use crate::query::context::AstContext;
use crate::query::planner::planner::{Planner, PlannerError};
use crate::query::planner::plan::SubPlan;

/// FETCH VERTICES查询规划器
/// 负责将FETCH VERTICES查询转换为执行计划
#[derive(Debug)]
pub struct FetchVerticesPlanner;

impl FetchVerticesPlanner {
    /// 创建新的FETCH VERTICES规划器
    pub fn new() -> Self {
        Self
    }

    /// 创建规划器实例的工厂函数
    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    /// 检查AST上下文是否匹配FETCH VERTICES查询
    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "FETCH VERTICES"
    }

    /// 获取匹配和实例化函数
    pub fn get_match_and_instantiate() -> crate::query::planner::planner::MatchAndInstantiate {
        crate::query::planner::planner::MatchAndInstantiate {
            match_func: Self::match_ast_ctx,
            instantiate_func: Self::make,
        }
    }
}

impl Planner for FetchVerticesPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // TODO: 实现FETCH VERTICES查询的规划逻辑
        Err(PlannerError::UnsupportedOperation(
            "FETCH VERTICES query planning not yet implemented".to_string(),
        ))
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

impl Default for FetchVerticesPlanner {
    fn default() -> Self {
        Self::new()
    }
}
