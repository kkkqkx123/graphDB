//! Merge Operation Planner
//!
//! Query planning for handling MERGE statements
//!
//! Note: This is a simplified implementation that converts MERGE to INSERT IF NOT EXISTS.
//! Full MERGE implementation with ON MATCH and ON CREATE clauses requires additional work.

use crate::core::types::expr::contextual::ContextualExpression;
use crate::core::types::expr::ExpressionMeta;
use crate::core::{Expression, Value};
use crate::query::parser::ast::{MergeStmt, Pattern, Stmt};
use crate::query::planning::plan::core::{
    node_id_generator::next_node_id,
    nodes::{ArgumentNode, InsertVerticesNode, TagInsertSpec, VertexInsertInfo},
};
use crate::query::planning::plan::{PlanNodeEnum, SubPlan};
use crate::query::planning::planner::{Planner, PlannerError, ValidatedStatement};
use crate::query::QueryContext;
use std::sync::Arc;

/// Merge Operation Planner
/// Responsible for converting MERGE statements into execution plans.
#[derive(Debug, Clone)]
pub struct MergePlanner;

impl MergePlanner {
    /// Create a new merge planner.
    pub fn new() -> Self {
        Self
    }

    /// Check whether the statements match the merge operations.
    pub fn match_stmt(stmt: &Stmt) -> bool {
        matches!(stmt, Stmt::Merge(_))
    }

    /// Extract the MergeStmt from the Stmt.
    fn extract_merge_stmt(&self, stmt: &Stmt) -> Result<MergeStmt, PlannerError> {
        match stmt {
            Stmt::Merge(merge_stmt) => Ok(merge_stmt.clone()),
            _ => Err(PlannerError::PlanGenerationFailed(
                "statement does not contain the MERGE".to_string(),
            )),
        }
    }

    /// Convert a Pattern to VertexInsertInfo
    /// Currently only supports simple node patterns like (p:Person {name: 'Alice'})
    fn pattern_to_vertex_info(
        &self,
        pattern: &Pattern,
        space_name: String,
        expr_context: &Arc<crate::query::validator::context::ExpressionAnalysisContext>,
    ) -> Result<VertexInsertInfo, PlannerError> {
        match pattern {
            Pattern::Node(node_pattern) => {
                // Get tag name from labels
                let tag_name = node_pattern
                    .labels
                    .first()
                    .ok_or_else(|| {
                        PlannerError::PlanGenerationFailed(
                            "MERGE node pattern must have a label".to_string(),
                        )
                    })?
                    .clone();

                // Handle properties
                let (prop_names, prop_values, vid_expr) =
                    if let Some(props_expr) = &node_pattern.properties {
                        self.extract_properties_and_vid(props_expr, expr_context)?
                    } else {
                        // No properties - generate a random ID
                        let vid_expr = self.create_vid_expression(expr_context)?;
                        (vec![], vec![], vid_expr)
                    };

                let tag_spec = TagInsertSpec {
                    tag_name,
                    prop_names,
                };

                Ok(VertexInsertInfo {
                    space_name,
                    tags: vec![tag_spec],
                    values: vec![(vid_expr, vec![prop_values])],
                    if_not_exists: true, // MERGE uses IF NOT EXISTS semantics
                })
            }
            _ => Err(PlannerError::PlanGenerationFailed(
                "MERGE currently only supports node patterns".to_string(),
            )),
        }
    }

    /// Extract property names and values from a properties expression
    /// Also generates a vertex ID
    fn extract_properties_and_vid(
        &self,
        props_expr: &ContextualExpression,
        expr_context: &Arc<crate::query::validator::context::ExpressionAnalysisContext>,
    ) -> Result<(Vec<String>, Vec<ContextualExpression>, ContextualExpression), PlannerError> {
        if let Some(Expression::Map(entries)) = props_expr.get_expression() {
            let mut prop_names = Vec::new();
            let mut prop_values = Vec::new();

            for (key, value) in entries {
                prop_names.push(key.clone());

                let value_meta = ExpressionMeta::new(value.clone());
                let value_id = expr_context.register_expression(value_meta);
                let ctx_value = ContextualExpression::new(value_id, expr_context.clone());
                prop_values.push(ctx_value);
            }

            let vid_expr = if let Some(Expression::Literal(Value::Int(i))) =
                prop_values.first().and_then(|v| v.get_expression())
            {
                let vid_meta = ExpressionMeta::new(Expression::Literal(Value::Int(i)));
                let vid_id = expr_context.register_expression(vid_meta);
                ContextualExpression::new(vid_id, expr_context.clone())
            } else {
                self.create_vid_expression(expr_context)?
            };

            Ok((prop_names, prop_values, vid_expr))
        } else {
            let vid_expr = self.create_vid_expression(expr_context)?;
            Ok((vec![], vec![], vid_expr))
        }
    }

    /// Create a vertex ID expression
    fn create_vid_expression(
        &self,
        expr_context: &Arc<crate::query::validator::context::ExpressionAnalysisContext>,
    ) -> Result<ContextualExpression, PlannerError> {
        // Generate a random ID
        let random_id = rand::random::<i64>().abs();
        let vid_meta = ExpressionMeta::new(Expression::Literal(Value::BigInt(random_id)));
        let vid_id = expr_context.register_expression(vid_meta);
        Ok(ContextualExpression::new(vid_id, expr_context.clone()))
    }
}

impl Planner for MergePlanner {
    fn transform(
        &mut self,
        validated: &ValidatedStatement,
        qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        // Obtain the space name
        let space_name = qctx.space_name().unwrap_or_else(|| "default".to_string());

        // Use the verification information to optimize the planning process.
        let validation_info = &validated.validation_info;

        // Check the semantic information.
        let referenced_tags = &validation_info.semantic_info.referenced_tags;
        if !referenced_tags.is_empty() {
            log::debug!("MERGE quoted tags: {:?}", referenced_tags);
        }

        let referenced_edges = &validation_info.semantic_info.referenced_edges;
        if !referenced_edges.is_empty() {
            log::debug!("MERGE references edge type: {:?}", referenced_edges);
        }

        let referenced_properties = &validation_info.semantic_info.referenced_properties;
        if !referenced_properties.is_empty() {
            log::debug!("MERGE Referenced Properties: {:?}", referenced_properties);
        }

        let merge_stmt = self.extract_merge_stmt(validated.stmt())?;

        // Convert the pattern to vertex insert info
        let vertex_info =
            self.pattern_to_vertex_info(&merge_stmt.pattern, space_name, validated.expr_context())?;

        // Create an Argument Node
        let arg_node = ArgumentNode::new(next_node_id(), "merge_args");
        let arg_node_enum = PlanNodeEnum::Argument(arg_node.clone());

        // Create InsertVertices node with IF NOT EXISTS
        let insert_node = InsertVerticesNode::new(next_node_id(), vertex_info);
        let insert_node_enum = PlanNodeEnum::InsertVertices(insert_node);

        // Create a SubPlan
        let sub_plan = SubPlan::new(Some(insert_node_enum), Some(arg_node_enum));

        Ok(sub_plan)
    }

    fn match_planner(&self, stmt: &Stmt) -> bool {
        Self::match_stmt(stmt)
    }
}

impl Default for MergePlanner {
    fn default() -> Self {
        Self::new()
    }
}
