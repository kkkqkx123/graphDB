//! Data Validation Helper Module
//!
//! Provides functions to validate data state in storage after operations

use graphdb::core::error::DBResult;
use graphdb::core::Value;
use graphdb::query::executor::base::ExecutionResult;
use graphdb::query::query_pipeline_manager::QueryPipelineManager;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;



/// Data validation helper for verifying storage state
pub struct ValidationHelper<S: graphdb::storage::StorageClient + 'static> {
    pipeline: QueryPipelineManager<S>,
}

impl<S: graphdb::storage::StorageClient + 'static> ValidationHelper<S> {
    /// Create a new validation helper
    pub fn new(storage: Arc<Mutex<S>>) -> Self {
        use graphdb::core::stats::StatsManager;
        use graphdb::query::optimizer::OptimizerEngine;
        use std::sync::Arc;

        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());
        let pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer);

        Self { pipeline }
    }

    // ==================== Vertex Validation ====================

    /// Check if a vertex exists with the given tag
    pub fn vertex_exists(&mut self, vid: i64, tag: &str) -> DBResult<bool> {
        let query = format!("FETCH PROP ON {} {}", tag, vid);
        let result = self.pipeline.execute_query(&query)?;
        Ok(result.count() > 0)
    }

    /// Get vertex properties
    pub fn get_vertex_props(&mut self, vid: i64, tag: &str) -> DBResult<HashMap<String, Value>> {
        let query = format!("FETCH PROP ON {} {}", tag, vid);
        let result = self.pipeline.execute_query(&query)?;

        match result {
            ExecutionResult::Result(r) => {
                if r.row_count() == 0 {
                    return Ok(HashMap::new());
                }
                let col_names = r.col_names().to_vec();
                let row = r.get_row(0).unwrap();

                let mut props = HashMap::new();
                for (i, col_name) in col_names.iter().enumerate() {
                    if let Some(value) = row.get(i) {
                        props.insert(col_name.clone(), value.clone());
                    }
                }
                Ok(props)
            }
            ExecutionResult::DataSet(ds) => {
                if ds.row_count() == 0 {
                    return Ok(HashMap::new());
                }
                let mut props = HashMap::new();
                for (i, col_name) in ds.col_names.iter().enumerate() {
                    if let Some(value) = ds.rows[0].get(i) {
                        props.insert(col_name.clone(), value.clone());
                    }
                }
                Ok(props)
            }
            _ => Ok(HashMap::new()),
        }
    }

    /// Check if vertex has specific property value
    pub fn vertex_has_prop(
        &mut self,
        vid: i64,
        tag: &str,
        prop: &str,
        expected: &Value,
    ) -> DBResult<bool> {
        let props = self.get_vertex_props(vid, tag)?;
        match props.get(prop) {
            Some(value) => Ok(value == expected),
            None => Ok(false),
        }
    }

    /// Count vertices with specific tag
    pub fn count_vertices(&mut self, tag: &str) -> DBResult<usize> {
        let query = format!("LOOKUP ON {}", tag);
        let result = self.pipeline.execute_query(&query)?;
        Ok(result.count())
    }

    // ==================== Edge Validation ====================

    /// Check if an edge exists
    pub fn edge_exists(&mut self, src: i64, dst: i64, edge_type: &str) -> DBResult<bool> {
        let query = format!("FETCH PROP ON {} {} -> {}", edge_type, src, dst);
        let result = self.pipeline.execute_query(&query)?;
        Ok(result.count() > 0)
    }

    /// Get edge properties
    pub fn get_edge_props(
        &mut self,
        src: i64,
        dst: i64,
        edge_type: &str,
    ) -> DBResult<HashMap<String, Value>> {
        let query = format!("FETCH PROP ON {} {} -> {}", edge_type, src, dst);
        let result = self.pipeline.execute_query(&query)?;

        match result {
            ExecutionResult::Result(r) => {
                if r.row_count() == 0 {
                    return Ok(HashMap::new());
                }
                let col_names = r.col_names().to_vec();
                let row = r.get_row(0).unwrap();

                let mut props = HashMap::new();
                for (i, col_name) in col_names.iter().enumerate() {
                    if let Some(value) = row.get(i) {
                        props.insert(col_name.clone(), value.clone());
                    }
                }
                Ok(props)
            }
            ExecutionResult::DataSet(ds) => {
                if ds.row_count() == 0 {
                    return Ok(HashMap::new());
                }
                let mut props = HashMap::new();
                for (i, col_name) in ds.col_names.iter().enumerate() {
                    if let Some(value) = ds.rows[0].get(i) {
                        props.insert(col_name.clone(), value.clone());
                    }
                }
                Ok(props)
            }
            _ => Ok(HashMap::new()),
        }
    }

    /// Check if edge has specific property value
    pub fn edge_has_prop(
        &mut self,
        src: i64,
        dst: i64,
        edge_type: &str,
        prop: &str,
        expected: &Value,
    ) -> DBResult<bool> {
        let props = self.get_edge_props(src, dst, edge_type)?;
        match props.get(prop) {
            Some(value) => Ok(value == expected),
            None => Ok(false),
        }
    }

    /// Count edges of specific type
    pub fn count_edges(&mut self, edge_type: &str) -> DBResult<usize> {
        let query = format!("LOOKUP ON {}", edge_type);
        let result = self.pipeline.execute_query(&query)?;
        Ok(result.count())
    }

    // ==================== Schema Validation ====================

    /// Check if tag exists
    pub fn tag_exists(&mut self, tag: &str) -> DBResult<bool> {
        let query = format!("DESC TAG {}", tag);
        let result = self.pipeline.execute_query(&query)?;
        Ok(result.count() > 0)
    }

    /// Check if edge type exists
    pub fn edge_type_exists(&mut self, edge_type: &str) -> DBResult<bool> {
        let query = format!("DESC EDGE {}", edge_type);
        let result = self.pipeline.execute_query(&query)?;
        Ok(result.count() > 0)
    }

    /// Get tag schema
    pub fn get_tag_schema(&mut self, tag: &str) -> DBResult<Vec<(String, String)>> {
        let query = format!("DESC TAG {}", tag);
        let result = self.pipeline.execute_query(&query)?;

        let mut schema = Vec::new();
        match result {
            ExecutionResult::Result(r) => {
                for row in r.rows() {
                    if row.len() >= 2 {
                        if let (Value::String(field), Value::String(field_type)) =
                            (&row[0], &row[1])
                        {
                            schema.push((field.clone(), field_type.clone()));
                        }
                    }
                }
            }
            ExecutionResult::DataSet(ds) => {
                for row in &ds.rows {
                    if row.len() >= 2 {
                        if let (Value::String(field), Value::String(field_type)) =
                            (&row[0], &row[1])
                        {
                            schema.push((field.clone(), field_type.clone()));
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(schema)
    }

    /// Check if tag has specific field
    pub fn tag_has_field(&mut self, tag: &str, field: &str) -> DBResult<bool> {
        let schema = self.get_tag_schema(tag)?;
        Ok(schema.iter().any(|(f, _)| f == field))
    }

    /// Check if tag field has specific type
    pub fn tag_field_has_type(
        &mut self,
        tag: &str,
        field: &str,
        expected_type: &str,
    ) -> DBResult<bool> {
        let schema = self.get_tag_schema(tag)?;
        Ok(schema.iter().any(|(f, t)| f == field && t == expected_type))
    }
}
