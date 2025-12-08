//! PATH查询规划器
//! 处理Nebula PATH查询的规划

use crate::core::AstContext;
use crate::query::planner::planner::{Planner, PlannerError};
use crate::query::planner::plan::SubPlan;

/// PATH查询规划器
/// 负责将PATH查询转换为执行计划
#[derive(Debug)]
pub struct PathPlanner;

impl PathPlanner {
    /// 创建新的PATH规划器
    pub fn new() -> Self {
        Self
    }

    /// 创建规划器实例的工厂函数
    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    /// 检查AST上下文是否匹配PATH查询
    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "PATH"
    }

    /// 获取匹配和实例化函数
    pub fn get_match_and_instantiate() -> crate::query::planner::planner::MatchAndInstantiate {
        crate::query::planner::planner::MatchAndInstantiate {
            match_func: Self::match_ast_ctx,
            instantiate_func: Self::make,
        }
    }
}

impl Planner for PathPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // TODO: 实现PATH查询的规划逻辑
        Err(PlannerError::UnsupportedOperation(
            "PATH query planning not yet implemented".to_string(),
        ))
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

impl Default for PathPlanner {
    fn default() -> Self {
        Self::new()
    }
}
