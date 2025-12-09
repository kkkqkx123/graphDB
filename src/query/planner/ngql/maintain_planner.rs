//! 维护操作规划器
//! 处理维护相关的查询规划（如SUBMIT JOB等）

use crate::query::context::AstContext;
use crate::query::planner::planner::{Planner, PlannerError};
use crate::query::planner::plan::SubPlan;

/// 维护操作规划器
/// 负责将维护操作转换为执行计划
#[derive(Debug)]
pub struct MaintainPlanner;

impl MaintainPlanner {
    /// 创建新的维护规划器
    pub fn new() -> Self {
        Self
    }

    /// 创建规划器实例的工厂函数
    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    /// 检查AST上下文是否匹配维护操作
    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        let stmt_type = ast_ctx.statement_type().to_uppercase();
        stmt_type == "SUBMIT JOB" || stmt_type.starts_with("CREATE") || stmt_type.starts_with("DROP")
    }

    /// 获取匹配和实例化函数
    pub fn get_match_and_instantiate() -> crate::query::planner::planner::MatchAndInstantiate {
        crate::query::planner::planner::MatchAndInstantiate {
            match_func: Self::match_ast_ctx,
            instantiate_func: Self::make,
        }
    }
}

impl Planner for MaintainPlanner {
    fn transform(&mut self, _ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // TODO: 实现维护操作的规划逻辑
        Err(PlannerError::UnsupportedOperation(
            "Maintenance query planning not yet implemented".to_string(),
        ))
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

impl Default for MaintainPlanner {
    fn default() -> Self {
        Self::new()
    }
}
