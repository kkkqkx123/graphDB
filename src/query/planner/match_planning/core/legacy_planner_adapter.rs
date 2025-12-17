//! 旧规划器适配器
//! 提供旧接口到新接口的适配器，确保向后兼容

use crate::query::planner::match_planning::core::{
    CypherClausePlanner, ClauseType, PlanningContext
};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::CypherClauseContext;

/// 旧规划器适配器
/// 将旧的子句规划器包装成新的接口
pub struct LegacyPlannerAdapter<T> {
    inner: T,
    clause_type: ClauseType,
}

impl<T> LegacyPlannerAdapter<T> {
    pub fn new(inner: T, clause_type: ClauseType) -> Self {
        Self { inner, clause_type }
    }
}

// 为旧的子句规划器实现适配器
impl<T> CypherClausePlanner for LegacyPlannerAdapter<T>
where
    T: crate::query::planner::match_planning::clauses::clause_planner::ClausePlanner + Clone,
{
    fn transform(
        &self,
        clause_ctx: &CypherClauseContext,
        _input_plan: Option<&SubPlan>,
        _context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        // 旧接口只接受clause_ctx，我们需要忽略其他参数
        let mut inner = self.inner.clone();
        inner.transform(clause_ctx)
    }
    
    fn clause_type(&self) -> ClauseType {
        self.clause_type.clone()
    }
    
    fn can_start_flow(&self) -> bool {
        matches!(self.clause_type, ClauseType::Source)
    }
    
    fn requires_input(&self) -> bool {
        !self.can_start_flow()
    }
}

// 为 YieldClausePlanner 实现适配器
#[derive(Clone)]
pub struct YieldPlannerAdapter {
    inner: crate::query::planner::match_planning::clauses::yield_planner::YieldClausePlanner,
}

impl YieldPlannerAdapter {
    pub fn new(inner: crate::query::planner::match_planning::clauses::yield_planner::YieldClausePlanner) -> Self {
        Self { inner }
    }
}

impl CypherClausePlanner for YieldPlannerAdapter {
    fn transform(
        &self,
        clause_ctx: &CypherClauseContext,
        _input_plan: Option<&SubPlan>,
        _context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        // 旧接口只接受clause_ctx，我们需要忽略其他参数
        let mut inner = self.inner.clone();
        inner.transform(clause_ctx)
    }
    
    fn clause_type(&self) -> ClauseType {
        ClauseType::Transform
    }
    
    fn can_start_flow(&self) -> bool {
        false
    }
    
    fn requires_input(&self) -> bool {
        true
    }
}