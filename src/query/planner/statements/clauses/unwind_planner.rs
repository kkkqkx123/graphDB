//! UNWIND 子句规划器
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
pub struct UnwindClausePlanner {
    unwind_expr: crate::query::parser::ast::expr::Expr,
    variable: String,
}

impl UnwindClausePlanner {
    pub fn new(unwind_expr: crate::query::parser::ast::expr::Expr) -> Self {
        Self {
            unwind_expr,
            variable: String::new(),
        }
    }
}

impl ClausePlanner for UnwindClausePlanner {
    fn name(&self) -> &'static str {
        "UnwindClausePlanner"
    }

    fn supported_clause_kind(&self) -> CypherClauseKind {
        CypherClauseKind::Unwind
    }
}

impl DataFlowNode for UnwindClausePlanner {
    fn flow_direction(&self) -> crate::query::planner::statements::core::cypher_clause_planner::FlowDirection {
        self.clause_type().flow_direction()
    }
}

impl CypherClausePlanner for UnwindClausePlanner {
    fn clause_type(&self) -> ClauseType {
        ClauseType::Unwind
    }

    fn transform(
        &self,
        _clause_ctx: &CypherClauseContext,
        input_plan: Option<&SubPlan>,
        _context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        self.validate_flow(input_plan)?;
        let input_plan = input_plan.ok_or_else(|| {
            PlannerError::PlanGenerationFailed("UNWIND 子句需要输入计划".to_string())
        })?;
        Ok(input_plan.clone())
    }
}
