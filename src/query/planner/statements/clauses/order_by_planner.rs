//! ORDER BY 子句规划器
use crate::query::planner::statements::clauses::clause_planner::ClausePlanner;
use crate::query::planner::statements::core::cypher_clause_planner::{
    ClauseType, CypherClausePlanner, DataFlowNode, PlanningContext,
};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::common_structs::CypherClauseContext;
use crate::query::validator::structs::CypherClauseKind;

#[derive(Debug)]
pub struct OrderByClausePlanner {}

impl OrderByClausePlanner {
    pub fn new() -> Self {
        Self {}
    }
}

impl ClausePlanner for OrderByClausePlanner {
    fn name(&self) -> &'static str {
        "OrderByClausePlanner"
    }

    fn supported_clause_kind(&self) -> CypherClauseKind {
        CypherClauseKind::OrderBy
    }
}

impl DataFlowNode for OrderByClausePlanner {
    fn flow_direction(&self) -> crate::query::planner::statements::core::cypher_clause_planner::FlowDirection {
        self.clause_type().flow_direction()
    }
}

impl CypherClausePlanner for OrderByClausePlanner {
    fn clause_type(&self) -> ClauseType {
        ClauseType::OrderBy
    }

    fn transform(
        &self,
        _clause_ctx: &CypherClauseContext,
        input_plan: Option<&SubPlan>,
        _context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        self.validate_flow(input_plan)?;
        let input_plan = input_plan.ok_or_else(|| {
            PlannerError::PlanGenerationFailed("ORDER BY 子句需要输入计划".to_string())
        })?;
        Ok(input_plan.clone())
    }
}
