//! YIELD 子句规划器
use crate::query::planner::statements::clauses::clause_planner::ClausePlanner;
use crate::query::planner::statements::core::cypher_clause_planner::{
    ClauseType, CypherClausePlanner, DataFlowNode, PlanningContext,
};
use crate::query::planner::plan::core::nodes::join_node::JoinConnector;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::common_structs::CypherClauseContext;
use crate::query::validator::structs::CypherClauseKind;

#[derive(Debug)]
pub struct YieldClausePlanner {
    yield_items: Vec<crate::query::parser::ast::expr::Expr>,
}

impl YieldClausePlanner {
    pub fn new() -> Self {
        Self { yield_items: vec![] }
    }
}

impl ClausePlanner for YieldClausePlanner {
    fn name(&self) -> &'static str {
        "YieldClausePlanner"
    }

    fn supported_clause_kind(&self) -> CypherClauseKind {
        CypherClauseKind::Yield
    }
}

impl DataFlowNode for YieldClausePlanner {
    fn flow_direction(&self) -> crate::query::planner::statements::core::cypher_clause_planner::FlowDirection {
        self.clause_type().flow_direction()
    }
}

impl CypherClausePlanner for YieldClausePlanner {
    fn clause_type(&self) -> ClauseType {
        ClauseType::Yield
    }

    fn transform(
        &self,
        _clause_ctx: &CypherClauseContext,
        input_plan: Option<&SubPlan>,
        _context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        self.validate_flow(input_plan)?;
        let input_plan = input_plan.ok_or_else(|| {
            PlannerError::PlanGenerationFailed("YIELD 子句需要输入计划".to_string())
        })?;
        Ok(input_plan.clone())
    }
}
