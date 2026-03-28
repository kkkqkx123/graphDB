//! The FETCH VERTICES query planner
//! Planning for the FETCH VERTICES query

use crate::query::parser::ast::{FetchTarget, Stmt};
use crate::query::planning::plan::core::common::TagProp;
use crate::query::planning::plan::core::nodes::{
    ArgumentNode, GetVerticesNode, PlanNodeEnum, ProjectNode,
};
use crate::query::planning::plan::SubPlan;
use crate::query::planning::planner::{Planner, PlannerError, ValidatedStatement};
use crate::query::QueryContext;
use std::sync::Arc;

/// The FETCH VERTICES query planner
/// Responsible for converting the FETCH VERTICES query into an execution plan.
#[derive(Debug, Clone)]
pub struct FetchVerticesPlanner;

impl FetchVerticesPlanner {
    /// Create a new FETCH VERTICES planner.
    pub fn new() -> Self {
        Self
    }
}

impl Planner for FetchVerticesPlanner {
    fn transform(
        &mut self,
        validated: &ValidatedStatement,
        _qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        let fetch_stmt = match validated.stmt() {
            Stmt::Fetch(fetch_stmt) => fetch_stmt,
            _ => {
                return Err(PlannerError::InvalidOperation(
                    "FetchVerticesPlanner 需要 Fetch 语句".to_string(),
                ));
            }
        };

        // Check whether it is a FETCH VERTICES operation.
        let (_ids, properties) = match &fetch_stmt.target {
            FetchTarget::Vertices {
                ids, properties, ..
            } => (ids, properties),
            _ => {
                return Err(PlannerError::InvalidOperation(
                    "FetchVerticesPlanner 需要 FETCH VERTICES 语句".to_string(),
                ));
            }
        };

        let var_name = "v";

        // 1. Create a parameter node to obtain the vertex ID.
        let mut arg_node = ArgumentNode::new(1, var_name);
        arg_node.set_col_names(vec!["vid".to_string()]);
        arg_node.set_output_var("vertex_ids".to_string());

        let arg_node_enum = PlanNodeEnum::Argument(arg_node.clone());

        let mut get_vertices_node = GetVerticesNode::new(2, var_name);
        get_vertices_node.add_dependency(arg_node_enum.clone());
        get_vertices_node.set_output_var("fetched_vertices".to_string());

        // Set the tag attributes (obtained from the properties field)
        let tag_props = if let Some(props) = properties {
            vec![TagProp::new("default", props.clone())]
        } else {
            vec![]
        };
        get_vertices_node.set_tag_props(tag_props);

        let get_vertices_node_enum = PlanNodeEnum::GetVertices(get_vertices_node);

        // 3. Create a projection node.
        let project_node = ProjectNode::new(get_vertices_node_enum.clone(), vec![])?;

        let project_node_enum = PlanNodeEnum::Project(project_node);

        // 4. Create a SubPlan
        let sub_plan = SubPlan::new(Some(project_node_enum), Some(arg_node_enum));

        Ok(sub_plan)
    }

    fn match_planner(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Fetch(fetch_stmt) => {
                matches!(&fetch_stmt.target, FetchTarget::Vertices { .. })
            }
            _ => false,
        }
    }
}

impl Default for FetchVerticesPlanner {
    fn default() -> Self {
        Self::new()
    }
}
