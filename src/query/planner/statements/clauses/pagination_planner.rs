//! 分页子句规划器
use crate::query::planner::statements::clauses::clause_planner::ClausePlanner;
use crate::query::planner::statements::core::cypher_clause_planner::{
    ClauseType, CypherClausePlanner, DataFlowNode, PlanningContext,
};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::common_structs::CypherClauseContext;
use crate::query::validator::structs::CypherClauseKind;

#[derive(Debug)]
pub struct PaginationPlanner {}

impl PaginationPlanner {
    pub fn new() -> Self {
        Self {}
    }
}

impl ClausePlanner for PaginationPlanner {
    fn name(&self) -> &'static str {
        "PaginationPlanner"
    }

    fn supported_clause_kind(&self) -> CypherClauseKind {
        CypherClauseKind::Pagination
    }
}

impl DataFlowNode for PaginationPlanner {
    fn flow_direction(&self) -> crate::query::planner::statements::core::cypher_clause_planner::FlowDirection {
        self.clause_type().flow_direction()
    }
}

impl CypherClausePlanner for PaginationPlanner {
    fn clause_type(&self) -> ClauseType {
        ClauseType::Limit
    }

    fn transform(
        &self,
        _clause_ctx: &CypherClauseContext,
        input_plan: Option<&SubPlan>,
        _context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        self.validate_flow(input_plan)?;
        let input_plan = input_plan.ok_or_else(|| {
            PlannerError::PlanGenerationFailed("分页子句需要输入计划".to_string())
        })?;
        Ok(input_plan.clone())
    }
}
