//! RETURN 子句规划器
use crate::query::planner::statements::clauses::clause_planner::ClausePlanner;
use crate::query::planner::statements::core::cypher_clause_planner::{
    ClauseType, CypherClausePlanner, DataFlowNode, PlanningContext,
};
use crate::query::planner::statements::utils::connection_strategy::UnifiedConnector;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::common_structs::CypherClauseContext;
use crate::query::validator::structs::CypherClauseKind;

#[derive(Debug)]
pub struct ReturnClausePlanner {
    return_items: Vec<crate::query::parser::ast::expr::Expr>,
    distinct: bool,
}

impl ReturnClausePlanner {
    pub fn new() -> Self {
        Self {
            return_items: vec![],
            distinct: false,
        }
    }
}

impl ClausePlanner for ReturnClausePlanner {
    fn name(&self) -> &'static str {
        "ReturnClausePlanner"
    }

    fn supported_clause_kind(&self) -> CypherClauseKind {
        CypherClauseKind::Return
    }
}

impl DataFlowNode for ReturnClausePlanner {
    fn flow_direction(&self) -> crate::query::planner::statements::core::cypher_clause_planner::FlowDirection {
        self.clause_type().flow_direction()
    }
}

impl CypherClausePlanner for ReturnClausePlanner {
    fn clause_type(&self) -> ClauseType {
        ClauseType::Return
    }

    fn transform(
        &self,
        _clause_ctx: &CypherClauseContext,
        input_plan: Option<&SubPlan>,
        _context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        self.validate_flow(input_plan)?;
        let input_plan = input_plan.ok_or_else(|| {
            PlannerError::PlanGenerationFailed("RETURN 子句需要输入计划".to_string())
        })?;
        Ok(input_plan.clone())
    }
}
