//! Vector Search Planner
//!
//! This module contains the planner for vector search operations.

use std::sync::Arc;

use crate::query::metadata::MetadataContext;
use crate::query::parser::ast::vector::{
    ComparisonOp, CreateVectorIndex, DropVectorIndex, LookupVector, MatchVector,
    SearchVectorStatement, WhereClause, WhereCondition,
};
use crate::query::parser::ast::Stmt;
use crate::query::planning::plan::core::nodes::base::plan_node_traits::PlanNode;
use crate::query::planning::plan::core::nodes::search::vector::data_access::{
    OutputField, VectorLookupNode, VectorMatchNode, VectorSearchNode,
};
use crate::query::planning::plan::core::nodes::search::vector::management::{
    CreateVectorIndexNode, CreateVectorIndexParams, DropVectorIndexNode,
};
use crate::query::planning::plan::core::nodes::search::vector::VectorSearchParams;
use crate::query::planning::plan::SubPlan;
use crate::query::planning::planner::{Planner, PlannerError, ValidatedStatement};
use crate::query::QueryContext;
use vector_client::types::{ConditionType, FilterCondition, RangeCondition, VectorFilter};

/// Vector search planner
#[derive(Debug, Clone, Default)]
pub struct VectorSearchPlanner {
    /// Metadata context for pre-resolved metadata (optional for backward compatibility)
    metadata_context: Option<Arc<MetadataContext>>,
}

impl VectorSearchPlanner {
    pub fn new() -> Self {
        Self {
            metadata_context: None,
        }
    }

    /// Create a new vector search planner with metadata context
    pub fn with_metadata_context(metadata_context: Arc<MetadataContext>) -> Self {
        Self {
            metadata_context: Some(metadata_context),
        }
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
            Stmt::DropVectorIndex(drop) => self.transform_drop_vector_index(drop, &space_name),
            Stmt::SearchVector(search) => self.transform_search_vector(search, space_id),
            Stmt::LookupVector(lookup) => {
                self.transform_lookup_vector(lookup, space_id, &space_name)
            }
            Stmt::MatchVector(match_stmt) => self.transform_match_vector(match_stmt, space_id),
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

    fn transform_with_metadata(
        &mut self,
        validated: &ValidatedStatement,
        qctx: Arc<QueryContext>,
        metadata_context: &MetadataContext,
    ) -> Result<SubPlan, PlannerError> {
        let stmt = validated.stmt();
        let space_name = qctx.space_name().unwrap_or_else(|| "default".to_string());
        let space_id = qctx.space_id().unwrap_or(0);

        match stmt {
            Stmt::CreateVectorIndex(create) => {
                self.transform_create_vector_index(create, &space_name)
            }
            Stmt::DropVectorIndex(drop) => self.transform_drop_vector_index(drop, &space_name),
            Stmt::SearchVector(search) => {
                self.transform_search_vector_with_metadata(search, space_id, metadata_context)
            }
            Stmt::LookupVector(lookup) => self.transform_lookup_vector_with_metadata(
                lookup,
                space_id,
                &space_name,
                metadata_context,
            ),
            Stmt::MatchVector(match_stmt) => {
                self.transform_match_vector_with_metadata(match_stmt, space_id, metadata_context)
            }
            _ => Err(PlannerError::PlanGenerationFailed(
                "Not a vector search statement".to_string(),
            )),
        }
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
            CreateVectorIndexParams::new(
                create.index_name.clone(),
                schema_name,
                create.schema_name.clone(),
                create.field_name.clone(),
                create.config.vector_size,
                create.config.distance,
            )
            .with_hnsw_m(create.config.hnsw_m)
            .with_hnsw_ef_construct(create.config.hnsw_ef_construct)
            .with_if_not_exists(),
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

    /// Transform SEARCH VECTOR statement into execution plan
    ///
    /// # Architecture Note
    /// This method now pre-resolves index metadata during the planning phase.
    /// The metadata_context is used to look up tag_name and field_name from the index_name.
    /// This allows for early error detection and better query optimization.
    fn transform_search_vector(
        &self,
        search: &SearchVectorStatement,
        space_id: u64,
    ) -> Result<SubPlan, PlannerError> {
        // Parse output fields from yield clause
        let output_fields = if let Some(yield_clause) = &search.yield_clause {
            yield_clause
                .items
                .iter()
                .map(|item| OutputField {
                    name: item.expr.clone(),
                    alias: item.alias.clone(),
                })
                .collect()
        } else {
            vec![]
        };

        // Convert WHERE clause to VectorFilter
        let filter = search
            .where_clause
            .as_ref()
            .and_then(|where_clause| self.convert_where_clause_to_filter(where_clause));

        // Pre-resolve tag_name and field_name from metadata context if available
        let (tag_name, field_name) = if let Some(ref metadata_context) = self.metadata_context {
            // Try to get index metadata from context
            if let Some(index_metadata) = metadata_context.get_index_metadata(&search.index_name) {
                (
                    index_metadata.tag_name.clone(),
                    index_metadata.field_name.clone(),
                )
            } else {
                // Metadata not pre-resolved, use empty strings (executor will resolve)
                (String::new(), String::new())
            }
        } else {
            // No metadata context, use empty strings (backward compatibility)
            (String::new(), String::new())
        };

        let node = VectorSearchNode::new(
            VectorSearchParams::new(
                search.index_name.clone(),
                space_id,
                tag_name,
                field_name,
                search.query.clone(),
            )
            .with_threshold(search.threshold.unwrap_or(0.0))
            .with_filter(filter)
            .with_limit(search.limit.unwrap_or(10))
            .with_offset(search.offset.unwrap_or(0))
            .with_output_fields(output_fields),
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

        let yield_fields = lookup
            .yield_clause
            .as_ref()
            .map_or_else(Vec::new, |yield_clause| {
                yield_clause
                    .items
                    .iter()
                    .map(|item| OutputField {
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
        let yield_fields = match_stmt
            .yield_clause
            .as_ref()
            .map_or_else(Vec::new, |yield_clause| {
                yield_clause
                    .items
                    .iter()
                    .map(|item| OutputField {
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

    /// Convert WhereClause to VectorFilter
    ///
    /// This method transforms the AST WhereClause into a VectorFilter that can be
    /// used by the vector search engine (e.g., Qdrant).
    fn convert_where_clause_to_filter(&self, where_clause: &WhereClause) -> Option<VectorFilter> {
        self.convert_where_condition_to_filter(&where_clause.condition)
    }

    /// Recursively convert WhereCondition to VectorFilter
    fn convert_where_condition_to_filter(
        &self,
        condition: &WhereCondition,
    ) -> Option<VectorFilter> {
        match condition {
            WhereCondition::Comparison(field, op, value) => {
                self.convert_comparison_to_filter(field, *op, value)
            }
            WhereCondition::And(left, right) => {
                let left_filter = self.convert_where_condition_to_filter(left)?;
                let right_filter = self.convert_where_condition_to_filter(right)?;

                // Merge filters: AND means both conditions must be met
                Some(self.merge_filters_must(left_filter, right_filter))
            }
            WhereCondition::Or(left, right) => {
                let left_filter = self.convert_where_condition_to_filter(left)?;
                let right_filter = self.convert_where_condition_to_filter(right)?;

                // Merge filters: OR means either condition can be met
                Some(self.merge_filters_should(left_filter, right_filter))
            }
            WhereCondition::Not(inner) => {
                let inner_filter = self.convert_where_condition_to_filter(inner)?;
                // Negate the filter: must_not
                Some(self.negate_filter(inner_filter))
            }
        }
    }

    /// Convert a comparison condition to VectorFilter
    fn convert_comparison_to_filter(
        &self,
        field: &str,
        op: ComparisonOp,
        value: &crate::core::Value,
    ) -> Option<VectorFilter> {
        let value_str = self.value_to_string(value)?;

        let condition = match op {
            ComparisonOp::Eq => {
                FilterCondition::new(field, ConditionType::Match { value: value_str })
            }
            ComparisonOp::Ne => {
                // For Not Equal, we use must_not with Match
                let filter = VectorFilter::new().must_not(FilterCondition::new(
                    field,
                    ConditionType::Match { value: value_str },
                ));
                return Some(filter);
            }
            ComparisonOp::Lt | ComparisonOp::Le | ComparisonOp::Gt | ComparisonOp::Ge => {
                // Range condition
                let range = self.create_range_condition(op, &value_str)?;
                FilterCondition::new(field, ConditionType::Range(range))
            }
        };

        Some(VectorFilter::new().must(condition))
    }

    /// Create RangeCondition from comparison operator and value
    fn create_range_condition(&self, op: ComparisonOp, value_str: &str) -> Option<RangeCondition> {
        let mut range = RangeCondition::new();

        match op {
            ComparisonOp::Lt => {
                range.lt = Some(value_str.parse().ok()?);
            }
            ComparisonOp::Le => {
                range.lte = Some(value_str.parse().ok()?);
            }
            ComparisonOp::Gt => {
                range.gt = Some(value_str.parse().ok()?);
            }
            ComparisonOp::Ge => {
                range.gte = Some(value_str.parse().ok()?);
            }
            _ => return None,
        }

        Some(range)
    }

    /// Convert core::Value to String for filter conditions
    fn value_to_string(&self, value: &crate::core::Value) -> Option<String> {
        match value {
            crate::core::Value::String(s) => Some(s.clone()),
            crate::core::Value::Int(i) => Some(i.to_string()),
            crate::core::Value::Float(f) => Some(f.to_string()),
            crate::core::Value::Bool(b) => Some(b.to_string()),
            _ => None,
        }
    }

    /// Merge two filters with AND logic (must)
    fn merge_filters_must(&self, left: VectorFilter, right: VectorFilter) -> VectorFilter {
        let mut result = VectorFilter::new();

        // Add all must conditions from left
        if let Some(must) = left.must {
            for condition in must {
                result = result.must(condition);
            }
        }

        // Add all must conditions from right
        if let Some(must) = right.must {
            for condition in must {
                result = result.must(condition);
            }
        }

        // Add all must_not conditions from left
        if let Some(must_not) = left.must_not {
            for condition in must_not {
                result = result.must_not(condition);
            }
        }

        // Add all must_not conditions from right
        if let Some(must_not) = right.must_not {
            for condition in must_not {
                result = result.must_not(condition);
            }
        }

        result
    }

    /// Merge two filters with OR logic (should)
    fn merge_filters_should(&self, left: VectorFilter, right: VectorFilter) -> VectorFilter {
        let mut result = VectorFilter::new();

        // Add all must conditions from left as should
        if let Some(must) = left.must {
            for condition in must {
                result = result.should(condition);
            }
        }

        // Add all must conditions from right as should
        if let Some(must) = right.must {
            for condition in must {
                result = result.should(condition);
            }
        }

        result
    }

    /// Negate a filter (convert to must_not)
    fn negate_filter(&self, filter: VectorFilter) -> VectorFilter {
        let mut result = VectorFilter::new();

        // Convert must to must_not
        if let Some(must) = filter.must {
            for condition in must {
                result = result.must_not(condition);
            }
        }

        // Convert must_not to must
        if let Some(must_not) = filter.must_not {
            for condition in must_not {
                result = result.must(condition);
            }
        }

        result
    }

    /// Transform SEARCH VECTOR with pre-resolved metadata
    fn transform_search_vector_with_metadata(
        &self,
        search: &SearchVectorStatement,
        space_id: u64,
        metadata_context: &MetadataContext,
    ) -> Result<SubPlan, PlannerError> {
        // Parse output fields from yield clause
        let output_fields = if let Some(yield_clause) = &search.yield_clause {
            yield_clause
                .items
                .iter()
                .map(|item| OutputField {
                    name: item.expr.clone(),
                    alias: item.alias.clone(),
                })
                .collect()
        } else {
            vec![]
        };

        // Convert WHERE clause to VectorFilter
        let filter = search
            .where_clause
            .as_ref()
            .and_then(|where_clause| self.convert_where_clause_to_filter(where_clause));

        // Pre-resolve tag_name and field_name from metadata context
        let (tag_name, field_name) = match metadata_context.get_index_metadata(&search.index_name) {
            Some(index_metadata) => (
                index_metadata.tag_name.clone(),
                index_metadata.field_name.clone(),
            ),
            None => {
                return Err(PlannerError::IndexNotFound(search.index_name.clone()));
            }
        };

        let node = VectorSearchNode::new(
            VectorSearchParams::new(
                search.index_name.clone(),
                space_id,
                tag_name,
                field_name,
                search.query.clone(),
            )
            .with_threshold(search.threshold.unwrap_or(0.0))
            .with_filter(filter)
            .with_limit(search.limit.unwrap_or(10))
            .with_offset(search.offset.unwrap_or(0))
            .with_output_fields(output_fields),
        );

        Ok(SubPlan::new(Some(node.into_enum()), None))
    }

    /// Transform LOOKUP VECTOR with pre-resolved metadata
    fn transform_lookup_vector_with_metadata(
        &self,
        lookup: &LookupVector,
        _space_id: u64,
        space_name: &str,
        _metadata_context: &MetadataContext,
    ) -> Result<SubPlan, PlannerError> {
        let schema_name = if lookup.schema_name.is_empty() {
            space_name.to_string()
        } else {
            lookup.schema_name.clone()
        };

        let yield_fields = lookup
            .yield_clause
            .as_ref()
            .map_or_else(Vec::new, |yield_clause| {
                yield_clause
                    .items
                    .iter()
                    .map(|item| OutputField {
                        name: item.expr.clone(),
                        alias: item.alias.clone(),
                    })
                    .collect()
            });

        let limit = lookup.limit.unwrap_or(10);

        let node = VectorLookupNode::new(
            schema_name,
            lookup.index_name.clone(),
            lookup.query.clone(),
            yield_fields,
            limit,
        );

        Ok(SubPlan::new(Some(node.into_enum()), None))
    }

    /// Transform MATCH VECTOR with pre-resolved metadata
    fn transform_match_vector_with_metadata(
        &self,
        match_stmt: &MatchVector,
        _space_id: u64,
        _metadata_context: &MetadataContext,
    ) -> Result<SubPlan, PlannerError> {
        // Parse yield fields
        let yield_fields = match_stmt
            .yield_clause
            .as_ref()
            .map_or_else(Vec::new, |yield_clause| {
                yield_clause
                    .items
                    .iter()
                    .map(|item| OutputField {
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
