//! Match planner implementation for handling MATCH queries
use super::plan::{ExecutionPlan, PlanNodeKind, SingleInputNode, SubPlan};
use super::planner::{Planner, PlannerError};
use crate::query::context::AstContext;

#[derive(Debug)]
pub struct MatchPlanner {
    // Whether the tail is connected in the plan
    tail_connected: bool,
}

impl MatchPlanner {
    pub fn new() -> Self {
        Self {
            tail_connected: false,
        }
    }

    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        // Check if the AST context represents a match statement
        // In a real implementation, this would check specific properties of the AST
        matches!(ast_ctx.statement_type(), "MATCH")
    }
}

impl Planner for MatchPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // Generate the execution plan for the match statement
        // This would call various helper methods to build the plan

        // First, verify this is a match statement
        if !Self::match_ast_ctx(ast_ctx) {
            return Err(PlannerError::InvalidAstContext(
                "AST context is not a match statement".to_string(),
            ));
        }

        // Create a plan node for the match operation
        let match_node = Box::new(SingleInputNode::new(
            PlanNodeKind::GetNeighbors, // Using GetNeighbors as a sample plan node for match
            Box::new(SingleInputNode::new(
                PlanNodeKind::Start,
                create_empty_node()?,
            )),
        ));

        // Create the execution plan
        let execution_plan = ExecutionPlan::new(Some(match_node));

        // For now, just return a subplan with the execution plan
        Ok(SubPlan::new(Some(execution_plan.root.unwrap()), None))
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

// Helper function to create an empty start node
fn create_empty_node() -> Result<Box<dyn super::plan::PlanNode>, PlannerError> {
    use super::plan::{PlanNodeKind, SingleDependencyNode};

    Ok(Box::new(SingleDependencyNode {
        id: -1,
        kind: PlanNodeKind::Start,
        dependencies: vec![],
        output_var: None,
        col_names: vec![],
        cost: 0.0,
    }))
}
