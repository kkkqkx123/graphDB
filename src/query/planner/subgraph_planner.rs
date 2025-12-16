//! Subgraph planner implementation for handling SUBGRAPH queries in NebulaGraph

use super::plan::{ExecutionPlan, PlanNodeKind, SingleInputNode, SubPlan};
use super::planner::{Planner, PlannerError};
use crate::query::context::ast::AstContext;
use std::sync::Arc;

#[derive(Debug)]
pub struct SubgraphPlanner {
    // Planner-specific fields would go here
}

impl SubgraphPlanner {
    pub fn new() -> Self {
        Self {}
    }

    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        // Check if the AST context represents a subgraph statement
        // In a real implementation, this would check specific properties of the AST
        matches!(ast_ctx.statement_type(), "SUBGRAPH")
    }
}

impl Planner for SubgraphPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // Generate the execution plan for the subgraph statement
        if !Self::match_ast_ctx(ast_ctx) {
            return Err(PlannerError::InvalidAstContext(
                "AST context is not a subgraph statement".to_string(),
            ));
        }

        // Create a plan node for the subgraph operation
        let subgraph_node = Arc::new(SingleInputNode::new(
            PlanNodeKind::Subgraph, // Using Subgraph as the base node for subgraph queries
            create_empty_node()?,
        ));

        // Create the execution plan
        let execution_plan = ExecutionPlan::new(Some(subgraph_node));

        // For now, just return a subplan with the execution plan
        Ok(SubPlan::new(Some(execution_plan.root.unwrap()), None))
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

// Helper function to create an empty start node
fn create_empty_node() -> Result<Arc<dyn super::plan::PlanNode>, PlannerError> {
    use super::plan::{PlanNodeKind, SingleDependencyNode};

    Ok(Arc::new(SingleDependencyNode {
        id: -1,
        kind: PlanNodeKind::Start,
        dependencies: vec![],
        output_var: None,
        col_names: vec![],
        cost: 0.0,
    }))
}
