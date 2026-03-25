//! PATH Query Planner
//! Planning for handling Nebula PATH query requests
//!
//! ## Explanation of the improvements
//!
//! Implementing shortest path planning
//! Implement all path planning functions.
//! Support for the shortest path with weights
//! Improve the logic for path filtering.

use crate::query::parser::ast::Stmt;
use crate::query::planning::plan::core::nodes::traversal::{AllPathsNode, ShortestPathNode};
use crate::query::planning::plan::core::PlanNode;
use crate::query::planning::plan::SubPlan;
use crate::query::planning::planner::{Planner, PlannerError, ValidatedStatement};
use crate::query::QueryContext;
use std::sync::Arc;

pub use crate::query::planning::plan::core::nodes::{
    ArgumentNode, DedupNode, ExpandAllNode, FilterNode, GetNeighborsNode, ProjectNode, StartNode,
};
pub use crate::query::planning::plan::core::PlanNodeEnum;

/// PATH Query Planner
/// Responsible for converting PATH queries into execution plans.
#[derive(Debug, Clone)]
pub struct PathPlanner {}

impl PathPlanner {
    /// Create a new PATH planner.
    pub fn new() -> Self {
        Self {}
    }
}

impl Planner for PathPlanner {
    fn transform(
        &mut self,
        validated: &ValidatedStatement,
        _qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        let find_path_stmt = match validated.stmt() {
            Stmt::FindPath(find_path_stmt) => find_path_stmt,
            _ => {
                return Err(PlannerError::InvalidOperation(
                    "PathPlanner 需要 FindPath 语句".to_string(),
                ));
            }
        };

        // Create the starting node.
        let start_node = StartNode::new();
        let start_node_enum = PlanNodeEnum::Start(start_node);

        let edge_types = self.get_edge_types_from_stmt(find_path_stmt);
        let max_steps = self.get_max_steps_from_stmt(find_path_stmt);

        // Select different planning strategies depending on the type of query.
        let root_node = if self.is_shortest_path_stmt(find_path_stmt) {
            // Shortest path query
            self.build_shortest_path_plan(start_node_enum.clone(), edge_types, max_steps)?
        } else {
            // All path queries
            self.build_all_paths_plan(start_node_enum.clone(), edge_types, max_steps)?
        };

        let sub_plan = SubPlan {
            root: Some(root_node),
            tail: Some(start_node_enum),
        };

        Ok(sub_plan)
    }

    fn match_planner(&self, stmt: &Stmt) -> bool {
        matches!(stmt, Stmt::FindPath(_))
    }
}

impl PathPlanner {
    /// Constructing the shortest path plan
    fn build_shortest_path_plan(
        &self,
        left_input: PlanNodeEnum,
        edge_types: Vec<String>,
        max_steps: usize,
    ) -> Result<PlanNodeEnum, PlannerError> {
        // Create the input node (destination) on the right side.
        let right_node = StartNode::new();
        let right_node_enum = PlanNodeEnum::Start(right_node);

        // Create a ShortestPath plan node.
        let shortest_path_node =
            ShortestPathNode::new(left_input, right_node_enum, edge_types, max_steps);

        Ok(shortest_path_node.into_enum())
    }

    /// Construct all path plans.
    fn build_all_paths_plan(
        &self,
        left_input: PlanNodeEnum,
        edge_types: Vec<String>,
        max_steps: usize,
    ) -> Result<PlanNodeEnum, PlannerError> {
        // Create the input node (destination) on the right side.
        let right_node = StartNode::new();
        let right_node_enum = PlanNodeEnum::Start(right_node);

        // Create an AllPaths plan node.
        let all_paths_node = AllPathsNode::new(
            left_input,
            right_node_enum,
            max_steps,
            edge_types,
            1,
            max_steps,
            false,
        );

        Ok(all_paths_node.into_enum())
    }

    /// Determine whether it is a query for the shortest path.
    fn is_shortest_path_stmt(&self, stmt: &crate::query::parser::ast::FindPathStmt) -> bool {
        stmt.shortest
    }

    /// Extract the edge type from the statement.
    fn get_edge_types_from_stmt(
        &self,
        stmt: &crate::query::parser::ast::FindPathStmt,
    ) -> Vec<String> {
        stmt.over
            .as_ref()
            .map(|over| over.edge_types.clone())
            .unwrap_or_default()
    }

    /// Extract the maximum number of steps from the statement.
    fn get_max_steps_from_stmt(&self, stmt: &crate::query::parser::ast::FindPathStmt) -> usize {
        stmt.max_steps.unwrap_or(10)
    }
}

impl Default for PathPlanner {
    fn default() -> Self {
        Self::new()
    }
}
