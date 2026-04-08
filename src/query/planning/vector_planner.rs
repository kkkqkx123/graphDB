//! Vector Search Planner
//!
//! This module contains the planner for vector search operations.

use std::sync::Arc;

use crate::query::parser::ast::vector::{
    CreateVectorIndex, DropVectorIndex, LookupVector, MatchVector, SearchVectorStatement,
};
use crate::query::parser::ast::Stmt;
use crate::query::planning::plan::SubPlan;
use crate::query::planning::plan::core::nodes::base::plan_node_traits::PlanNode;
use crate::query::planning::plan::core::nodes::data_access::vector_search::{
    CreateVectorIndexNode, DropVectorIndexNode, VectorLookupNode, VectorMatchNode, VectorSearchNode,
};
use crate::query::planning::planner::{Planner, PlannerError, ValidatedStatement};
use crate::query::QueryContext;

/// Vector search planner
#[derive(Debug, Clone, Default)]
pub struct VectorSearchPlanner;

impl VectorSearchPlanner {
    pub fn new() -> Self {
        Self
    }
}

impl Planner for VectorSearchPlanner {
    fn transform(
        &mut self,
        validated: &ValidatedStatement,
        qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        let stmt = validated.stmt();
        let space_name = qctx.space_name().unwrap_or_else(|| "default".to_string());
        let space_id = qctx.space_id().unwrap_or(0);

        match stmt {
            Stmt::CreateVectorIndex(create) => {
                self.transform_create_vector_index(create, &space_name)
            }
            Stmt::DropVectorIndex(drop) => {
                self.transform_drop_vector_index(drop, &space_name)
            }
            Stmt::SearchVector(search) => {
                self.transform_search_vector(search, space_id, &space_name, qctx.clone())
            }
            Stmt::LookupVector(lookup) => {
                self.transform_lookup_vector(lookup, space_id, &space_name)
            }
            Stmt::MatchVector(match_stmt) => {
                self.transform_match_vector(match_stmt, space_id)
            }
            _ => Err(PlannerError::PlanGenerationFailed(
                "Not a vector search statement".to_string(),
            )),
        }
    }

    fn match_planner(&self, stmt: &Stmt) -> bool {
        matches!(
            stmt,
            Stmt::CreateVectorIndex(_)
                | Stmt::DropVectorIndex(_)
                | Stmt::SearchVector(_)
                | Stmt::LookupVector(_)
                | Stmt::MatchVector(_)
        )
    }
}

impl VectorSearchPlanner {
    fn transform_create_vector_index(
        &self,
        create: &CreateVectorIndex,
        space_name: &str,
    ) -> Result<SubPlan, PlannerError> {
        let schema_name = if create.schema_name.is_empty() {
            space_name.to_string()
        } else {
            create.schema_name.clone()
        };

        let node = CreateVectorIndexNode::new(
            create.index_name.clone(),
            schema_name,
            create.schema_name.clone(),
            create.field_name.clone(),
            create.config.vector_size,
            create.config.distance,
            create.config.hnsw_m,
            create.config.hnsw_ef_construct,
            create.if_not_exists,
        );

        Ok(SubPlan::new(Some(node.into_enum()), None))
    }

    fn transform_drop_vector_index(
        &self,
        drop: &DropVectorIndex,
        space_name: &str,
    ) -> Result<SubPlan, PlannerError> {
        let node = DropVectorIndexNode::new(
            drop.index_name.clone(),
            space_name.to_string(),
            drop.if_exists,
        );

        Ok(SubPlan::new(Some(node.into_enum()), None))
    }

    fn transform_search_vector(
        &self,
        search: &SearchVectorStatement,
        space_id: u64,
        space_name: &str,
        _qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        // Parse output fields from yield clause
        let output_fields = if let Some(yield_clause) = &search.yield_clause {
            yield_clause
                .items
                .iter()
                .map(|item| crate::query::planning::plan::core::nodes::data_access::vector_search::OutputField {
                    name: item.expr.clone(),
                    alias: item.alias.clone(),
                })
                .collect()
        } else {
            vec![]
        };

        // Parse filter from where clause (simplified - full implementation would need expression conversion)
        let filter = search.where_clause.as_ref().map(|_| {
            // TODO: Convert WHERE clause to ContextualExpression
            // This requires expression transformation logic
            todo!("WHERE clause expression conversion not yet implemented")
        });

        let node = VectorSearchNode::new(
            search.index_name.clone(),
            space_id,
            space_name.to_string(),
            String::new(), // TODO: Extract tag_name from index metadata
            search.query.clone(),
            search.threshold,
            filter,
            search.limit.unwrap_or(10),
            search.offset.unwrap_or(0),
            output_fields,
        );

        Ok(SubPlan::new(Some(node.into_enum()), None))
    }

    fn transform_lookup_vector(
        &self,
        lookup: &LookupVector,
        _space_id: u64,
        space_name: &str,
    ) -> Result<SubPlan, PlannerError> {
        let schema_name = if lookup.schema_name.is_empty() {
            space_name.to_string()
        } else {
            lookup.schema_name.clone()
        };

        let yield_fields = lookup.yield_clause.as_ref().map_or_else(Vec::new, |yield_clause| {
            yield_clause
                .items
                .iter()
                .map(|item| crate::query::planning::plan::core::nodes::data_access::vector_search::OutputField {
                    name: item.expr.clone(),
                    alias: item.alias.clone(),
                })
                .collect()
        });

        let node = VectorLookupNode::new(
            schema_name,
            lookup.index_name.clone(),
            lookup.query.clone(),
            yield_fields,
            lookup.limit.unwrap_or(10),
        );

        Ok(SubPlan::new(Some(node.into_enum()), None))
    }

    fn transform_match_vector(
        &self,
        match_stmt: &MatchVector,
        _space_id: u64,
    ) -> Result<SubPlan, PlannerError> {
        let yield_fields = match_stmt.yield_clause.as_ref().map_or_else(Vec::new, |yield_clause| {
            yield_clause
                .items
                .iter()
                .map(|item| crate::query::planning::plan::core::nodes::data_access::vector_search::OutputField {
                    name: item.expr.clone(),
                    alias: item.alias.clone(),
                })
                .collect()
        });

        let node = VectorMatchNode::new(
            match_stmt.pattern.clone(),
            match_stmt.vector_condition.field.clone(),
            match_stmt.vector_condition.query.clone(),
            match_stmt.vector_condition.threshold,
            yield_fields,
        );

        Ok(SubPlan::new(Some(node.into_enum()), None))
    }
}
