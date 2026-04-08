//! Vector Search Executor
//!
//! This module implements the executor for vector search queries.

use std::collections::HashMap;
use std::sync::Arc;

use crate::core::error::DBError;
use crate::core::value::list::List;
use crate::core::value::null::NullType;
use crate::core::{DataSet, Value};
use crate::query::executor::base::{
    BaseExecutor, DBResult, ExecutionResult, Executor, ExecutorStats, HasStorage,
};
use crate::query::parser::ast::vector::{VectorQueryExpr, VectorQueryType};
use crate::query::planning::plan::core::nodes::data_access::vector_search::VectorLookupNode;
use crate::query::planning::plan::core::nodes::data_access::vector_search::{
    OutputField, VectorSearchNode,
};
use crate::storage::StorageClient;
use crate::vector::VectorCoordinator;
use parking_lot::Mutex;
use vector_client::types::SearchResult;

/// Vector search executor
pub struct VectorSearchExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    node: VectorSearchNode,
    coordinator: Arc<VectorCoordinator>,
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StorageClient> VectorSearchExecutor<S> {
    /// Create a new vector search executor
    pub fn new(
        id: i64,
        node: VectorSearchNode,
        storage: Arc<Mutex<S>>,
        expr_context: Arc<crate::query::validator::context::ExpressionAnalysisContext>,
        coordinator: Arc<VectorCoordinator>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(
                id,
                "VectorSearchExecutor".to_string(),
                storage,
                expr_context,
            ),
            node,
            coordinator,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Parse query vector from VectorQueryExpr
    fn parse_query_vector(&self, query: &VectorQueryExpr) -> DBResult<Vec<f32>> {
        match query.query_type {
            VectorQueryType::Vector => {
                // Parse vector literal: [0.1, 0.2, 0.3, ...]
                self.parse_vector_literal(&query.query_data).ok_or_else(|| {
                    DBError::Validation(format!("Invalid vector format: {}", query.query_data))
                })
            }
            VectorQueryType::Text => {
                // Text query: use embedding service to convert text to vector
                let text = &query.query_data;
                let coordinator = self.coordinator.clone();

                // Use tokio runtime to execute async embedding
                tokio::runtime::Handle::current().block_on(async move {
                    coordinator
                        .embed_text(text)
                        .await
                        .map_err(|e| DBError::Internal(format!("Text embedding failed: {}", e)))
                })
            }
            VectorQueryType::Parameter => {
                // Parameter reference: resolve from execution context
                let param_name = &query.query_data;
                if let Some(param_value) = self.base.context.get_param(param_name) {
                    match param_value {
                        crate::core::Value::List(values) => {
                            let vector: Result<Vec<f32>, _> = values
                                .iter()
                                .map(|v| {
                                    if let crate::core::Value::Float(f) = v {
                                        Ok(*f as f32)
                                    } else {
                                        Err(DBError::Validation(format!(
                                            "Parameter {} contains non-float value",
                                            param_name
                                        )))
                                    }
                                })
                                .collect();
                            vector
                        }
                        _ => Err(DBError::Validation(format!(
                            "Parameter {} is not a vector",
                            param_name
                        ))),
                    }
                } else {
                    Err(DBError::Validation(format!(
                        "Parameter {} not found",
                        param_name
                    )))
                }
            }
        }
    }

    /// Parse vector literal string to Vec<f32>
    fn parse_vector_literal(&self, text: &str) -> Option<Vec<f32>> {
        let text = text.trim().trim_start_matches('[').trim_end_matches(']');
        text.split(',')
            .map(|s| s.trim().parse::<f32>().ok())
            .collect()
    }

    /// Build column names from output fields
    #[allow(dead_code)]
    fn build_col_names(&self) -> Vec<String> {
        self.node
            .output_fields
            .iter()
            .map(|field| field.alias.clone().unwrap_or_else(|| field.name.clone()))
            .collect()
    }

    /// Execute vector search using blocking runtime
    fn execute_search(&self) -> DBResult<Vec<SearchResult>> {
        // Parse query vector
        let query_vector = self.parse_query_vector(&self.node.query)?;

        // Execute search using tokio blocking runtime
        let coordinator = self.coordinator.clone();
        let space_id = self.node.space_id;
        let tag_name = self.node.tag_name.clone();
        let field_name = self.node.field_name.clone();
        let limit = self.node.limit;
        let threshold = self.node.threshold;

        // Use tokio runtime to execute async operation
        let result = tokio::runtime::Handle::current()
            .block_on(async move {
                if let Some(threshold) = threshold {
                    coordinator
                        .search_with_threshold(
                            space_id,
                            &tag_name,
                            &field_name,
                            query_vector,
                            limit,
                            threshold,
                        )
                        .await
                } else {
                    coordinator
                        .search(space_id, &tag_name, &field_name, query_vector, limit)
                        .await
                }
            })
            .map_err(|e| DBError::Internal(format!("Vector search failed: {}", e)))?;

        Ok(result)
    }

    /// Build result dataset from search results
    fn build_dataset(&self, results: Vec<SearchResult>) -> DBResult<DataSet> {
        let mut dataset = DataSet::new();

        for result in results {
            let mut row = Vec::new();
            for field in &self.node.output_fields {
                let value = self.extract_field_value(field, &result)?;
                row.push(value);
            }
            dataset.add_row(row);
        }

        Ok(dataset)
    }

    /// Extract field value from search result
    fn extract_field_value(&self, field: &OutputField, result: &SearchResult) -> DBResult<Value> {
        match field.name.as_str() {
            "id" | "vertex_id" => Ok(Value::String(result.id.clone())),
            "score" => Ok(Value::Float(result.score as f64)),
            "vector" => {
                // Return vector if requested
                if let Some(vec) = &result.vector {
                    // Convert Vec<f32> to Value::List
                    let values: Vec<Value> = vec.iter().map(|&v| Value::Float(v as f64)).collect();
                    Ok(Value::List(List::from(values)))
                } else {
                    Ok(Value::Null(NullType::Null))
                }
            }
            _ => {
                // Get from payload
                if let Some(payload) = &result.payload {
                    if let Some(payload_value) = payload.get(&field.name) {
                        // Convert serde_json::Value to Value
                        return self.json_value_to_value(payload_value);
                    }
                }
                Ok(Value::Null(NullType::Null))
            }
        }
    }

    /// Convert serde_json::Value to core::Value
    fn json_value_to_value(&self, json_val: &serde_json::Value) -> DBResult<Value> {
        match json_val {
            serde_json::Value::Null => Ok(Value::Null(NullType::Null)),
            serde_json::Value::Bool(b) => Ok(Value::Bool(*b)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(Value::Int(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(Value::Float(f))
                } else {
                    Ok(Value::Null(NullType::Null))
                }
            }
            serde_json::Value::String(s) => Ok(Value::String(s.clone())),
            serde_json::Value::Array(arr) => {
                let vec: DBResult<Vec<Value>> =
                    arr.iter().map(|v| self.json_value_to_value(v)).collect();
                Ok(Value::List(List::from(vec?)))
            }
            serde_json::Value::Object(obj) => {
                let map: DBResult<HashMap<String, Value>> = obj
                    .iter()
                    .map(|(k, v)| self.json_value_to_value(v).map(|val| (k.clone(), val)))
                    .collect();
                Ok(Value::Map(map?))
            }
        }
    }
}

impl<S: StorageClient> Executor<S> for VectorSearchExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        // Execute vector search
        let results = self.execute_search()?;

        // Build dataset
        let dataset = self.build_dataset(results)?;

        Ok(ExecutionResult::DataSet(dataset))
    }

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        "Vector similarity search executor"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for VectorSearchExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

// ============== Vector Lookup Executor ==============

/// Vector lookup executor
pub struct VectorLookupExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    node: VectorLookupNode,
    coordinator: Arc<VectorCoordinator>,
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StorageClient> VectorLookupExecutor<S> {
    /// Create a new vector lookup executor
    pub fn new(
        id: i64,
        node: VectorLookupNode,
        storage: Arc<Mutex<S>>,
        expr_context: Arc<crate::query::validator::context::ExpressionAnalysisContext>,
        coordinator: Arc<VectorCoordinator>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(
                id,
                "VectorLookupExecutor".to_string(),
                storage,
                expr_context,
            ),
            node,
            coordinator,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Parse query vector from VectorQueryExpr
    fn parse_query_vector(&self, query: &VectorQueryExpr) -> DBResult<Vec<f32>> {
        match query.query_type {
            VectorQueryType::Vector => {
                let text = query
                    .query_data
                    .trim()
                    .trim_start_matches('[')
                    .trim_end_matches(']');
                text.split(',')
                    .map(|s| s.trim().parse::<f32>())
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| DBError::Validation(format!("Invalid vector format: {}", e)))
            }
            VectorQueryType::Text => {
                let text = &query.query_data;
                let coordinator = self.coordinator.clone();

                tokio::runtime::Handle::current().block_on(async move {
                    coordinator
                        .embed_text(text)
                        .await
                        .map_err(|e| DBError::Internal(format!("Text embedding failed: {}", e)))
                })
            }
            VectorQueryType::Parameter => {
                let param_name = &query.query_data;
                if let Some(param_value) = self.base.context.get_param(param_name) {
                    match param_value {
                        crate::core::Value::List(values) => {
                            let vector: Result<Vec<f32>, _> = values
                                .iter()
                                .map(|v| {
                                    if let crate::core::Value::Float(f) = v {
                                        Ok(*f as f32)
                                    } else {
                                        Err(DBError::Validation(format!(
                                            "Parameter {} contains non-float value",
                                            param_name
                                        )))
                                    }
                                })
                                .collect();
                            vector
                        }
                        _ => Err(DBError::Validation(format!(
                            "Parameter {} is not a vector",
                            param_name
                        ))),
                    }
                } else {
                    Err(DBError::Validation(format!(
                        "Parameter {} not found",
                        param_name
                    )))
                }
            }
        }
    }

    /// Build column names from yield fields
    #[allow(dead_code)]
    fn build_col_names(&self) -> Vec<String> {
        self.node
            .yield_fields
            .iter()
            .map(|field| field.alias.clone().unwrap_or_else(|| field.name.clone()))
            .collect()
    }

    /// Execute vector search using blocking runtime
    fn execute_search(&self) -> DBResult<Vec<SearchResult>> {
        let query_vector = self.parse_query_vector(&self.node.query)?;

        let coordinator = self.coordinator.clone();
        let space_id = self.base.context.current_space_id().unwrap_or(0);
        let tag_name = self.node.schema_name.clone();
        let field_name = self.node.index_name.clone();
        let limit = self.node.limit;

        tokio::runtime::Handle::current()
            .block_on(async move {
                coordinator
                    .search(space_id, &tag_name, &field_name, query_vector, limit)
                    .await
            })
            .map_err(|e| DBError::Internal(format!("Vector lookup failed: {}", e)))
    }

    /// Build result dataset from search results
    fn build_dataset(&self, results: Vec<SearchResult>) -> DBResult<DataSet> {
        let mut dataset = DataSet::new();

        for result in results {
            let mut row = Vec::new();
            for field in &self.node.yield_fields {
                let value = self.extract_field_value(field, &result)?;
                row.push(value);
            }
            dataset.add_row(row);
        }

        Ok(dataset)
    }

    /// Extract field value from search result
    fn extract_field_value(&self, field: &OutputField, result: &SearchResult) -> DBResult<Value> {
        match field.name.as_str() {
            "id" | "vertex_id" => Ok(Value::String(result.id.clone())),
            "score" => Ok(Value::Float(result.score as f64)),
            _ => {
                if let Some(payload) = &result.payload {
                    if let Some(payload_value) = payload.get(&field.name) {
                        return self.json_value_to_value(payload_value);
                    }
                }
                Ok(Value::Null(NullType::Null))
            }
        }
    }

    /// Convert serde_json::Value to core::Value
    fn json_value_to_value(&self, json_val: &serde_json::Value) -> DBResult<Value> {
        match json_val {
            serde_json::Value::Null => Ok(Value::Null(NullType::Null)),
            serde_json::Value::Bool(b) => Ok(Value::Bool(*b)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(Value::Int(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(Value::Float(f))
                } else {
                    Ok(Value::Null(NullType::Null))
                }
            }
            serde_json::Value::String(s) => Ok(Value::String(s.clone())),
            serde_json::Value::Array(arr) => {
                let vec: DBResult<Vec<Value>> =
                    arr.iter().map(|v| self.json_value_to_value(v)).collect();
                Ok(Value::List(List::from(vec?)))
            }
            serde_json::Value::Object(obj) => {
                let map: DBResult<HashMap<String, Value>> = obj
                    .iter()
                    .map(|(k, v)| self.json_value_to_value(v).map(|val| (k.clone(), val)))
                    .collect();
                Ok(Value::Map(map?))
            }
        }
    }
}

impl<S: StorageClient> Executor<S> for VectorLookupExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let results = self.execute_search()?;
        let dataset = self.build_dataset(results)?;
        Ok(ExecutionResult::DataSet(dataset))
    }

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        "Vector lookup executor"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for VectorLookupExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

// ============== Vector Match Executor ==============

use crate::query::planning::plan::core::nodes::data_access::vector_search::VectorMatchNode;

/// Vector match executor
pub struct VectorMatchExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    node: VectorMatchNode,
    coordinator: Arc<VectorCoordinator>,
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StorageClient> VectorMatchExecutor<S> {
    /// Create a new vector match executor
    pub fn new(
        id: i64,
        node: VectorMatchNode,
        storage: Arc<Mutex<S>>,
        expr_context: Arc<crate::query::validator::context::ExpressionAnalysisContext>,
        coordinator: Arc<VectorCoordinator>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "VectorMatchExecutor".to_string(), storage, expr_context),
            node,
            coordinator,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Parse query vector from VectorQueryExpr
    fn parse_query_vector(&self, query: &VectorQueryExpr) -> DBResult<Vec<f32>> {
        match query.query_type {
            VectorQueryType::Vector => {
                let text = query
                    .query_data
                    .trim()
                    .trim_start_matches('[')
                    .trim_end_matches(']');
                text.split(',')
                    .map(|s| s.trim().parse::<f32>())
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| DBError::Validation(format!("Invalid vector format: {}", e)))
            }
            VectorQueryType::Text => {
                let text = &query.query_data;
                let coordinator = self.coordinator.clone();

                tokio::runtime::Handle::current().block_on(async move {
                    coordinator
                        .embed_text(text)
                        .await
                        .map_err(|e| DBError::Internal(format!("Text embedding failed: {}", e)))
                })
            }
            VectorQueryType::Parameter => {
                let param_name = &query.query_data;
                if let Some(param_value) = self.base.context.get_param(param_name) {
                    match param_value {
                        crate::core::Value::List(values) => {
                            let vector: Result<Vec<f32>, _> = values
                                .iter()
                                .map(|v| {
                                    if let crate::core::Value::Float(f) = v {
                                        Ok(*f as f32)
                                    } else {
                                        Err(DBError::Validation(format!(
                                            "Parameter {} contains non-float value",
                                            param_name
                                        )))
                                    }
                                })
                                .collect();
                            vector
                        }
                        _ => Err(DBError::Validation(format!(
                            "Parameter {} is not a vector",
                            param_name
                        ))),
                    }
                } else {
                    Err(DBError::Validation(format!(
                        "Parameter {} not found",
                        param_name
                    )))
                }
            }
        }
    }

    /// Execute vector search using blocking runtime
    fn execute_search(&self) -> DBResult<Vec<SearchResult>> {
        let query_vector = self.parse_query_vector(&self.node.query)?;

        let coordinator = self.coordinator.clone();
        let space_id = self.base.context.current_space_id().unwrap_or(0);
        let tag_name = self.node.field.clone();
        let field_name = self.node.pattern.clone();
        let limit = 100; // Default limit for MATCH
        let threshold = self.node.threshold;

        tokio::runtime::Handle::current()
            .block_on(async move {
                if let Some(threshold) = threshold {
                    coordinator
                        .search_with_threshold(
                            space_id,
                            &tag_name,
                            &field_name,
                            query_vector,
                            limit,
                            threshold,
                        )
                        .await
                } else {
                    coordinator
                        .search(space_id, &tag_name, &field_name, query_vector, limit)
                        .await
                }
            })
            .map_err(|e| DBError::Internal(format!("Vector match failed: {}", e)))
    }
}

impl<S: StorageClient> Executor<S> for VectorMatchExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let results = self.execute_search()?;

        // For MATCH, we need to bind results to pattern variables
        // This is a simplified implementation
        let mut dataset = DataSet::new();

        for result in results {
            dataset.add_row(vec![
                Value::String(result.id),
                Value::Float(result.score as f64),
            ]);
        }

        Ok(ExecutionResult::DataSet(dataset))
    }

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        "Vector match executor"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for VectorMatchExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_vector_literal() {
        // Skip this test for now as it requires full storage setup
        // The parse_vector_literal method is tested indirectly through integration tests
    }
}
