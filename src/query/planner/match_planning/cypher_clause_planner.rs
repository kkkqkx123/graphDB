//! Cypher子句规划器基类
//! 定义所有Cypher子句规划器的通用接口

use crate::query::planner::plan::SubPlan;
use crate::query::validator::structs::common_structs::CypherClauseContext;

/// Cypher子句规划器trait
/// 所有Cypher子句规划器都应实现该trait
pub trait CypherClausePlanner: std::fmt::Debug {
    /// 将Cypher子句上下文转换为执行计划
    fn transform(&mut self, clause_ctx: &CypherClauseContext) -> Result<SubPlan, crate::query::planner::planner::PlannerError>;
}