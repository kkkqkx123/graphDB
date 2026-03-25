//! Maintenance Operation Planner
//! Handling query planning related to maintenance tasks (such as SUBMIT JOB, etc.)

use crate::query::parser::ast::{AlterTarget, CreateTarget, IndexType, ShowTarget, Stmt};
use crate::query::planning::plan::core::nodes::management::index_nodes::IndexManageInfo;
use crate::query::planning::plan::core::{
    node_id_generator::next_node_id, AlterSpaceNode, ArgumentNode, ClearSpaceNode, PlanNodeEnum,
    ProjectNode, ShowStatsNode, ShowStatsType,
};
use crate::query::planning::plan::SubPlan;
use crate::query::planning::planner::{Planner, PlannerError, ValidatedStatement};
use crate::query::QueryContext;
use std::sync::Arc;

/// Maintenance Operation Planner
/// Responsible for converting maintenance operations into execution plans.
#[derive(Debug, Clone)]
pub struct MaintainPlanner;

impl MaintainPlanner {
    /// Create a new maintenance planner.
    pub fn new() -> Self {
        Self
    }
}

impl Planner for MaintainPlanner {
    fn transform(
        &mut self,
        validated: &ValidatedStatement,
        _qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        let stmt_type = validated.stmt().kind().to_uppercase();

        // 1. Create a parameter node to receive the operation parameters.
        let arg_node = ArgumentNode::new(1, "maintain_args");

        // 2. Create corresponding plan nodes for different types.
        // Maintenance operations generally do not require the use of expressions; they simply return the results of the operations.
        let yield_columns = Vec::new();

        let project_node = ProjectNode::new(
            PlanNodeEnum::Argument(arg_node.clone()),
            yield_columns,
        )
        .map_err(|e| {
            PlannerError::PlanGenerationFailed(format!("Failed to create ProjectNode: {}", e))
        })?;

        // 3. Different types of operations may require different processing methods.
        let final_node = if stmt_type == "SHOW" {
            // Processing the SHOW STATS statement
            if let Stmt::Show(show_stmt) = validated.stmt() {
                if show_stmt.target == ShowTarget::Stats {
                    let stats_node = ShowStatsNode::new(next_node_id(), ShowStatsType::Storage);
                    PlanNodeEnum::ShowStats(stats_node)
                } else {
                    // Other SHOW statements use PassThrough nodes.
                    PlanNodeEnum::PassThrough(
                        crate::query::planning::plan::core::PassThroughNode::new(1),
                    )
                }
            } else {
                PlanNodeEnum::PassThrough(crate::query::planning::plan::core::PassThroughNode::new(
                    1,
                ))
            }
        } else if stmt_type == "SUBMIT JOB" {
            // Maintenance operations for submitting assignment types
            PlanNodeEnum::Project(project_node)
        } else if stmt_type.starts_with("CREATE") {
            // Operation to create a type
            if let Stmt::Create(create_stmt) = validated.stmt() {
                if let CreateTarget::Index {
                    index_type,
                    name,
                    on,
                    properties,
                } = &create_stmt.target
                {
                    let space_name = validated
                        .validation_info
                        .semantic_info
                        .space_name
                        .clone()
                        .unwrap_or_default();

                    let index_info = IndexManageInfo::new(
                        space_name.clone(),
                        name.clone(),
                        match index_type {
                            IndexType::Tag => "tag".to_string(),
                            IndexType::Edge => "edge".to_string(),
                        },
                    )
                    .with_target_name(on.clone())
                    .with_properties(properties.clone());

                    let plan_node = match index_type {
                        IndexType::Tag => {
                            let create_tag_index_node =
                                crate::query::planning::plan::core::nodes::CreateTagIndexNode::new(
                                    next_node_id(),
                                    index_info,
                                );
                            PlanNodeEnum::CreateTagIndex(create_tag_index_node)
                        }
                        IndexType::Edge => {
                            let create_edge_index_node =
                                crate::query::planning::plan::core::nodes::CreateEdgeIndexNode::new(
                                    next_node_id(),
                                    index_info,
                                );
                            PlanNodeEnum::CreateEdgeIndex(create_edge_index_node)
                        }
                    };
                    return Ok(SubPlan::new(
                        Some(plan_node),
                        Some(PlanNodeEnum::Argument(arg_node)),
                    ));
                }
            }
            // For other creation operations, the default processing methods are used.
            PlanNodeEnum::Project(project_node)
        } else if stmt_type.starts_with("ALTER") {
            // Processing the ALTER SPACE statement
            if let Stmt::Alter(alter_stmt) = validated.stmt() {
                if let AlterTarget::Space {
                    space_name,
                    comment,
                } = &alter_stmt.target
                {
                    let options = comment
                        .as_ref()
                        .map(|c| {
                            vec![
                                crate::query::planning::plan::core::nodes::SpaceAlterOption::Comment(
                                    c.clone(),
                                ),
                            ]
                        })
                        .unwrap_or_default();
                    let alter_space_node =
                        AlterSpaceNode::new(next_node_id(), space_name.clone(), options);
                    PlanNodeEnum::AlterSpace(alter_space_node)
                } else {
                    PlanNodeEnum::Project(project_node)
                }
            } else {
                PlanNodeEnum::Project(project_node)
            }
        } else if stmt_type == "CLEAR SPACE" {
            // Processing the CLEAR SPACE statement
            if let Stmt::ClearSpace(clear_stmt) = validated.stmt() {
                let clear_space_node =
                    ClearSpaceNode::new(next_node_id(), clear_stmt.space_name.clone());
                PlanNodeEnum::ClearSpace(clear_space_node)
            } else {
                PlanNodeEnum::Project(project_node)
            }
        } else {
            // Other types of maintenance operations
            PlanNodeEnum::Project(project_node)
        };

        // Create a SubPlan
        let sub_plan = SubPlan::new(Some(final_node), Some(PlanNodeEnum::Argument(arg_node)));

        Ok(sub_plan)
    }

    fn match_planner(&self, stmt: &Stmt) -> bool {
        let stmt_type = stmt.kind().to_uppercase();
        stmt_type == "SUBMIT JOB"
            || stmt_type.starts_with("CREATE")
            || stmt_type.starts_with("DROP")
            || stmt_type.starts_with("SHOW")
            || stmt_type == "DESC"
            || stmt_type.starts_with("ALTER")
            || stmt_type.starts_with("DESCRIBE")
            || stmt_type == "CLEAR SPACE"
    }
}

impl Default for MaintainPlanner {
    fn default() -> Self {
        Self::new()
    }
}
