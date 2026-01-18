//! 投影子句规划器
use crate::query::planner::statements::core::cypher_clause_planner::{CypherClausePlanner, DataFlowNode};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::common_structs::CypherClauseContext;

#[derive(Debug)]
pub struct ProjectionPlanner {
    projection_items: Vec<crate::query::parser::ast::expr::Expr>,
}

impl ProjectionPlanner {
    pub fn new(projection_items: Vec<crate::query::parser::ast::expr::Expr>) -> Self {
        Self { projection_items }
    }
}

impl DataFlowNode for ProjectionPlanner {
    fn flow_direction(&self) -> crate::query::planner::statements::core::cypher_clause_planner::FlowDirection {
        crate::query::planner::statements::core::cypher_clause_planner::ClauseType::With.flow_direction()
    }
}

impl CypherClausePlanner for ProjectionPlanner {
    fn clause_type(&self) -> crate::query::planner::statements::core::cypher_clause_planner::ClauseType {
        crate::query::planner::statements::core::cypher_clause_planner::ClauseType::With
    }

    fn transform(
        &self,
        _clause_ctx: &CypherClauseContext,
        input_plan: Option<&SubPlan>,
        _context: &mut crate::query::planner::statements::core::cypher_clause_planner::PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        let input_plan = input_plan.ok_or_else(|| {
            PlannerError::PlanGenerationFailed("投影子句需要输入计划".to_string())
        })?;
        Ok(input_plan.clone())
    }
}
