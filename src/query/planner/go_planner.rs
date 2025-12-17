use crate::query::planner::plan::core::nodes::PlanNodeFactory;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::plan::PlanNodeKind;
//! Go planner implementation for handling GO queries
use super::planner::{Planner, PlannerError};
use crate::query::context::ast::AstContext;
use std::sync::Arc;

#[derive(Debug)]
pub struct GoPlanner {
    // Planner-specific fields would go here
}

impl GoPlanner {
    pub fn new() -> Self {
        Self {}
    }

    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        // Check if the AST context represents a go statement
        // In a real implementation, this would check specific properties of the AST
        matches!(ast_ctx.statement_type(), "GO")
    }
}

impl Planner for GoPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // Generate the execution plan for the go statement
        // This would call various helper methods to build the plan

        // First, verify this is a go statement
        if !Self::match_ast_ctx(ast_ctx) {
            return Err(PlannerError::InvalidAstContext(
                "AST context is not a go statement".to_string(),
            ));
        }

        // Create a plan node for the go operation
        let go_node = Arc::new(PlanNodeFactory::create_placeholder_node()??,
        ));

        // Create the execution plan
        let execution_plan = ExecutionPlan::new(Some(go_node));

        // For now, just return a subplan with the execution plan
        Ok(SubPlan::new(Some(execution_plan.root.unwrap()), None))
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

// Helper function to create an empty start node
fn create_empty_node() -> Result<Arc<dyn super::plan::PlanNode>, PlannerError> {

    Ok(PlanNodeFactory::create_start_node()?)
}
