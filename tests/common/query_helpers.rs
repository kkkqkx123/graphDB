//! Query Execution Helper Module
//!
//! Provides convenient query execution and result extraction functions for tests

use graphdb::core::error::DBResult;
use graphdb::core::Value;
use graphdb::query::executor::base::ExecutionResult;
use graphdb::query::query_pipeline_manager::QueryPipelineManager;
use parking_lot::Mutex;
use std::sync::Arc;

use crate::common::TestStorage;

/// Query execution helper
pub struct QueryHelper<S: graphdb::storage::StorageClient + 'static> {
    pipeline: QueryPipelineManager<S>,
}

impl<S: graphdb::storage::StorageClient + 'static> QueryHelper<S> {
    /// Create a new query helper
    pub fn new(storage: Arc<Mutex<S>>) -> Self {
        use graphdb::core::stats::StatsManager;
        use graphdb::query::optimizer::OptimizerEngine;
        use std::sync::Arc;

        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());
        let pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer);

        Self { pipeline }
    }

    /// Execute a query and return the result
    pub fn execute(&mut self, query: &str) -> DBResult<ExecutionResult> {
        self.pipeline.execute_query(query)
    }

    /// Execute a DDL statement (CREATE, ALTER, DROP)
    pub fn exec_ddl(&mut self, query: &str) -> DBResult<()> {
        let result = self.execute(query)?;
        match result {
            ExecutionResult::Success | ExecutionResult::Empty => Ok(()),
            ExecutionResult::Error(msg) => Err(graphdb::core::error::DBError::Query(
                graphdb::core::error::QueryError::ExecutionError(msg),
            )),
            _ => Ok(()),
        }
    }

    /// Execute a DML statement (INSERT, UPDATE, DELETE)
    /// Returns the number of affected rows
    pub fn exec_dml(&mut self, query: &str) -> DBResult<usize> {
        let result = self.execute(query)?;
        match result {
            ExecutionResult::Count(n) => Ok(n),
            ExecutionResult::Success => Ok(1),
            ExecutionResult::Empty => Ok(0),
            ExecutionResult::Error(msg) => Err(graphdb::core::error::DBError::Query(
                graphdb::core::error::QueryError::ExecutionError(msg),
            )),
            _ => Ok(0),
        }
    }

    /// Execute a query and return the result as a Vec of rows
    pub fn query_rows(&mut self, query: &str) -> DBResult<Vec<Vec<Value>>> {
        let result = self.execute(query)?;
        match result {
            ExecutionResult::Result(r) => Ok(r.rows().to_vec()),
            ExecutionResult::DataSet(ds) => Ok(ds.rows),
            ExecutionResult::Values(v) => Ok(vec![v]),
            ExecutionResult::Vertices(v) => {
                Ok(v.into_iter().map(|vertex| vec![Value::Vertex(Box::new(vertex))]).collect())
            }
            ExecutionResult::Edges(e) => {
                Ok(e.into_iter().map(|edge| vec![Value::Edge(edge)]).collect())
            }
            ExecutionResult::Empty => Ok(vec![]),
            ExecutionResult::Error(msg) => Err(graphdb::core::error::DBError::Query(
                graphdb::core::error::QueryError::ExecutionError(msg),
            )),
            _ => Ok(vec![]),
        }
    }

    /// Execute a query and return a single scalar value
    pub fn query_scalar<T: FromValue>(&mut self, query: &str) -> DBResult<Option<T>> {
        let rows = self.query_rows(query)?;
        if rows.is_empty() || rows[0].is_empty() {
            return Ok(None);
        }
        T::from_value(&rows[0][0]).map(Some)
    }

    /// Execute a query and return the first row
    pub fn query_first(&mut self, query: &str) -> DBResult<Option<Vec<Value>>> {
        let rows = self.query_rows(query)?;
        Ok(rows.into_iter().next())
    }

    /// Execute a query and return the count
    pub fn query_count(&mut self, query: &str) -> DBResult<usize> {
        let result = self.execute(query)?;
        Ok(result.count())
    }
}

/// Trait for converting Value to specific types
trait FromValue: Sized {
    fn from_value(value: &Value) -> DBResult<Self>;
}

impl FromValue for i64 {
    fn from_value(value: &Value) -> DBResult<Self> {
        match value {
            Value::Int(i) => Ok(*i),
            Value::Int64(i) => Ok(*i),
            Value::Int32(i) => Ok(*i as i64),
            Value::Int16(i) => Ok(*i as i64),
            Value::Int8(i) => Ok(*i as i64),
            _ => Err(graphdb::core::error::DBError::Validation(format!(
                "Expected Int, got {:?}",
                value
            ))),
        }
    }
}

impl FromValue for String {
    fn from_value(value: &Value) -> DBResult<Self> {
        match value {
            Value::String(s) => Ok(s.clone()),
            _ => Err(graphdb::core::error::DBError::Validation(format!(
                "Expected String, got {:?}",
                value
            ))),
        }
    }
}

impl FromValue for f64 {
    fn from_value(value: &Value) -> DBResult<Self> {
        match value {
            Value::Float(f) => Ok(*f),
            _ => Err(graphdb::core::error::DBError::Validation(format!(
                "Expected Float, got {:?}",
                value
            ))),
        }
    }
}

impl FromValue for bool {
    fn from_value(value: &Value) -> DBResult<Self> {
        match value {
            Value::Bool(b) => Ok(*b),
            _ => Err(graphdb::core::error::DBError::Validation(format!(
                "Expected Bool, got {:?}",
                value
            ))),
        }
    }
}
