//! WITH 子句规划器
use crate::query::planner::statements::core::{
    ClauseType, CypherClausePlanner, DataFlowNode, PlanningContext,
};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::common_structs::CypherClauseContext;

#[derive(Debug)]
pub struct WithClausePlanner {}

impl WithClausePlanner {
    pub fn new() -> Self {
        Self {}
    }
}

impl DataFlowNode for WithClausePlanner {
    fn flow_direction(&self) -> crate::query::planner::statements::core::cypher_clause_planner::FlowDirection {
        self.clause_type().flow_direction()
    }
}

impl CypherClausePlanner for WithClausePlanner {
    fn clause_type(&self) -> ClauseType {
        ClauseType::With
    }

    fn transform(
        &self,
        _clause_ctx: &CypherClauseContext,
        input_plan: Option<&SubPlan>,
        context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        self.validate_flow(input_plan)?;
        let input_plan = input_plan.ok_or_else(|| {
            PlannerError::PlanGenerationFailed("WITH 子句需要输入计划".to_string())
        })?;
        context.reset_variable_scope();
        Ok(input_plan.clone())
    }
}
