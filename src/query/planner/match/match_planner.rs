//! MATCH查询主规划器
//! 负责将MATCH查询转换为执行计划

use crate::query::context::AstContext;
use crate::query::planner::planner::{Planner, PlannerError};
use crate::query::planner::plan::SubPlan;

/// MATCH查询规划器
/// 处理Cypher MATCH语句的转换为执行计划
#[derive(Debug)]
pub struct MatchPlanner {
    tail_connected: bool,
}

impl MatchPlanner {
    /// 创建新的MATCH规划器
    pub fn new() -> Self {
        Self {
            tail_connected: false,
        }
    }

    /// 创建规划器实例的工厂函数
    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    /// 检查AST上下文是否匹配MATCH查询
    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "MATCH"
    }

    /// 获取匹配和实例化函数
    pub fn get_match_and_instantiate() -> crate::query::planner::planner::MatchAndInstantiate {
        crate::query::planner::planner::MatchAndInstantiate {
            match_func: Self::match_ast_ctx,
            instantiate_func: Self::make,
        }
    }
}

impl Planner for MatchPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // TODO: 实现MATCH查询的复杂规划逻辑
        // 使用MatchClausePlanner、WhereClausePlanner、ReturnClausePlanner等
        Err(PlannerError::UnsupportedOperation(
            "MATCH query planning not yet implemented".to_string(),
        ))
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

impl Default for MatchPlanner {
    fn default() -> Self {
        Self::new()
    }
}
