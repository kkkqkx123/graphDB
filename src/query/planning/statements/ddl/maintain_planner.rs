//! Maintenance Operation Planner
//! Handling query planning related to maintenance tasks (such as SUBMIT JOB, etc.)

use crate::query::parser::ast::{AlterTarget, CreateTarget, IndexType, ShowTarget, Stmt};
use crate::query::planning::plan::core::nodes::management::index_nodes::IndexManageInfo;
use crate::query::planning::plan::core::nodes::management::space_nodes::{CreateSpaceNode, SpaceManageInfo};
use crate::query::planning::plan::core::{
    node_id_generator::next_node_id, AlterSpaceNode, ArgumentNode, ClearSpaceNode, PlanNodeEnum,
    ProjectNode, ShowStatsNode, ShowStatsType,
};
use crate::query::planning::plan::core::nodes::{
    CreateTagNode, CreateEdgeNode, TagManageInfo, EdgeManageInfo, ShowCreateTagNode,
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
        } else if stmt_type == "SHOW CREATE" {
            // Processing SHOW CREATE statements
            if let Stmt::ShowCreate(show_create_stmt) = validated.stmt() {
                let current_space = validated
                    .validation_info
                    .semantic_info
                    .space_name
                    .clone()
                    .unwrap_or_default();

                match &show_create_stmt.target {
                    crate::query::parser::ast::stmt::ShowCreateTarget::Tag(tag_name) => {
                        let show_create_tag_node = ShowCreateTagNode::new(
                            next_node_id(),
                            current_space,
                            tag_name.clone(),
                        );
                        PlanNodeEnum::ShowCreateTag(show_create_tag_node)
                    }
                    _ => {
                        // Other SHOW CREATE targets use PassThrough nodes
                        PlanNodeEnum::PassThrough(crate::query::planning::plan::core::PassThroughNode::new(1))
                    }
                }
            } else {
                PlanNodeEnum::PassThrough(crate::query::planning::plan::core::PassThroughNode::new(1))
            }
        } else if stmt_type == "SUBMIT JOB" {
            // Maintenance operations for submitting assignment types
            // Create a parameter node to receive the operation parameters.
            let arg_node = ArgumentNode::new(1, "maintain_args");
            let yield_columns = Vec::new();
            let project_node = ProjectNode::new(
                PlanNodeEnum::Argument(arg_node.clone()),
                yield_columns,
            )
            .map_err(|e| {
                PlannerError::PlanGenerationFailed(format!("Failed to create ProjectNode: {}", e))
            })?;
            PlanNodeEnum::Project(project_node)
        } else if stmt_type.starts_with("CREATE") {
            // Operation to create a type
            println!("[MaintainPlanner] Processing CREATE statement");
            if let Stmt::Create(create_stmt) = validated.stmt() {
                println!("[MaintainPlanner] Create target: {:?}", create_stmt.target);
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
                    return Ok(SubPlan::from_single_node(plan_node));
                } else if let CreateTarget::Space {
                    name,
                    vid_type,
                    ..
                } = &create_stmt.target
                {
                    let space_info = SpaceManageInfo::new(name.clone())
                        .with_vid_type(vid_type.clone());

                    let create_space_node = CreateSpaceNode::new(next_node_id(), space_info);
                    return Ok(SubPlan::from_single_node(PlanNodeEnum::CreateSpace(create_space_node)));
                } else if let CreateTarget::Tag {
                    name,
                    properties,
                    ..
                } = &create_stmt.target
                {
                    let space_name = validated
                        .validation_info
                        .semantic_info
                        .space_name
                        .clone()
                        .unwrap_or_default();

                    let tag_info = TagManageInfo::new(space_name.clone(), name.clone())
                        .with_properties(properties.clone());

                    let create_tag_node = CreateTagNode::new(next_node_id(), tag_info);
                    return Ok(SubPlan::from_single_node(PlanNodeEnum::CreateTag(create_tag_node)));
                } else if let CreateTarget::EdgeType {
                    name,
                    properties,
                    ..
                } = &create_stmt.target
                {
                    let space_name = validated
                        .validation_info
                        .semantic_info
                        .space_name
                        .clone()
                        .unwrap_or_default();

                    let edge_info = EdgeManageInfo::new(space_name.clone(), name.clone())
                        .with_properties(properties.clone());

                    let create_edge_node = CreateEdgeNode::new(next_node_id(), edge_info);
                    return Ok(SubPlan::from_single_node(PlanNodeEnum::CreateEdge(create_edge_node)));
                }
            }
            // For other creation operations, use PassThrough nodes
            PlanNodeEnum::PassThrough(crate::query::planning::plan::core::PassThroughNode::new(1))
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
                    // For other ALTER operations, use PassThrough nodes
                    PlanNodeEnum::PassThrough(crate::query::planning::plan::core::PassThroughNode::new(1))
                }
            } else {
                PlanNodeEnum::PassThrough(crate::query::planning::plan::core::PassThroughNode::new(1))
            }
        } else if stmt_type == "CLEAR SPACE" {
            // Processing the CLEAR SPACE statement
            if let Stmt::ClearSpace(clear_stmt) = validated.stmt() {
                let clear_space_node =
                    ClearSpaceNode::new(next_node_id(), clear_stmt.space_name.clone());
                PlanNodeEnum::ClearSpace(clear_space_node)
            } else {
                PlanNodeEnum::PassThrough(crate::query::planning::plan::core::PassThroughNode::new(1))
            }
        } else if stmt_type == "DESC" || stmt_type.starts_with("DESCRIBE") {
            // Processing DESC/DESCRIBE statements
            if let Stmt::Desc(desc_stmt) = validated.stmt() {
                // Get space_name from validation_info if available, otherwise use current context
                let current_space = validated
                    .validation_info
                    .semantic_info
                    .space_name
                    .clone()
                    .unwrap_or_default();

                match &desc_stmt.target {
                    crate::query::parser::ast::stmt::DescTarget::Tag { space_name, tag_name } => {
                        // Use space_name from DESC statement if provided, otherwise use current space
                        let effective_space = if space_name.is_empty() {
                            current_space
                        } else {
                            space_name.clone()
                        };
                        let desc_tag_node = crate::query::planning::plan::core::nodes::DescTagNode::new(
                            next_node_id(),
                            effective_space,
                            tag_name.clone(),
                        );
                        PlanNodeEnum::DescTag(desc_tag_node)
                    }
                    crate::query::parser::ast::stmt::DescTarget::Edge { space_name, edge_name } => {
                        // Use space_name from DESC statement if provided, otherwise use current space
                        let effective_space = if space_name.is_empty() {
                            current_space
                        } else {
                            space_name.clone()
                        };
                        let desc_edge_node = crate::query::planning::plan::core::nodes::DescEdgeNode::new(
                            next_node_id(),
                            effective_space,
                            edge_name.clone(),
                        );
                        PlanNodeEnum::DescEdge(desc_edge_node)
                    }
                    crate::query::parser::ast::stmt::DescTarget::Space(space_name) => {
                        let desc_space_node = crate::query::planning::plan::core::nodes::DescSpaceNode::new(
                            next_node_id(),
                            space_name.clone(),
                        );
                        PlanNodeEnum::DescSpace(desc_space_node)
                    }
                }
            } else {
                PlanNodeEnum::PassThrough(crate::query::planning::plan::core::PassThroughNode::new(1))
            }
        } else if stmt_type.starts_with("DROP") {
            // Processing DROP statements
            if let Stmt::Drop(drop_stmt) = validated.stmt() {
                use crate::query::parser::ast::stmt::DropTarget;
                match &drop_stmt.target {
                    DropTarget::Tags(tag_names) if !tag_names.is_empty() => {
                        // Get space_name from validation_info if available, otherwise use current context
                        let current_space = validated
                            .validation_info
                            .semantic_info
                            .space_name
                            .clone()
                            .unwrap_or_default();
                        let drop_tag_node = crate::query::planning::plan::core::nodes::DropTagNode::new(
                            next_node_id(),
                            current_space,
                            tag_names[0].clone(),
                        );
                        PlanNodeEnum::DropTag(drop_tag_node)
                    }
                    DropTarget::Edges(edge_names) if !edge_names.is_empty() => {
                        // Get space_name from validation_info if available, otherwise use current context
                        let current_space = validated
                            .validation_info
                            .semantic_info
                            .space_name
                            .clone()
                            .unwrap_or_default();
                        let drop_edge_node = crate::query::planning::plan::core::nodes::DropEdgeNode::new(
                            next_node_id(),
                            current_space,
                            edge_names[0].clone(),
                        );
                        PlanNodeEnum::DropEdge(drop_edge_node)
                    }
                    DropTarget::Space(space_name) => {
                        let drop_space_node = crate::query::planning::plan::core::nodes::DropSpaceNode::new(
                            next_node_id(),
                            space_name.clone(),
                        );
                        PlanNodeEnum::DropSpace(drop_space_node)
                    }
                    _ => PlanNodeEnum::PassThrough(crate::query::planning::plan::core::PassThroughNode::new(1)),
                }
            } else {
                PlanNodeEnum::PassThrough(crate::query::planning::plan::core::PassThroughNode::new(1))
            }
        } else {
            // Other types of maintenance operations use PassThrough nodes
            PlanNodeEnum::PassThrough(crate::query::planning::plan::core::PassThroughNode::new(1))
        };

        // Create a SubPlan without ArgumentNode for DDL operations
        let sub_plan = SubPlan::from_single_node(final_node);

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
