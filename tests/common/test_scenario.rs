//! Test Scenario Module
//!
//! Provides a high-level API for writing integration tests with fluent interface

use graphdb::core::error::DBResult;
use graphdb::core::Value;
use graphdb::query::executor::base::ExecutionResult;
use graphdb::query::query_pipeline_manager::QueryPipelineManager;
use graphdb::storage::redb_storage::RedbStorage;
use graphdb::storage::StorageClient;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

use crate::common::TestStorage;

/// Test scenario builder for fluent test writing
pub struct TestScenario {
    storage: Arc<Mutex<RedbStorage>>,
    pipeline: QueryPipelineManager<RedbStorage>,
    last_result: Option<ExecutionResult>,
    last_error: Option<String>,
    current_space: Option<graphdb::core::types::SpaceInfo>,
}

impl TestScenario {
    /// Create a new test scenario
    pub fn new() -> DBResult<Self> {
        let test_storage = TestStorage::new()?;
        let storage = test_storage.storage();

        use graphdb::core::stats::StatsManager;
        use graphdb::query::optimizer::OptimizerEngine;
        use std::sync::Arc;

        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());
        let pipeline = QueryPipelineManager::with_optimizer(storage.clone(), stats_manager, optimizer);

        Ok(Self {
            storage,
            pipeline,
            last_result: None,
            last_error: None,
            current_space: None,
        })
    }

    // ==================== Execution Methods ====================

    /// Execute a DDL statement
    pub fn exec_ddl(mut self, query: &str) -> Self {
        match self
            .pipeline
            .execute_query_with_space(query, self.current_space.clone())
        {
            Ok(result) => {
                self.last_result = Some(result);
                self.last_error = None;
            }
            Err(e) => {
                self.last_error = Some(format!("{:?}", e));
                self.last_result = None;
            }
        }
        self
    }

    /// Execute a DML statement
    pub fn exec_dml(mut self, query: &str) -> Self {
        match self
            .pipeline
            .execute_query_with_space(query, self.current_space.clone())
        {
            Ok(result) => {
                self.last_result = Some(result);
                self.last_error = None;
            }
            Err(e) => {
                self.last_error = Some(format!("{:?}", e));
                self.last_result = None;
            }
        }
        self
    }

    /// Execute a query
    pub fn query(mut self, query: &str) -> Self {
        match self
            .pipeline
            .execute_query_with_space(query, self.current_space.clone())
        {
            Ok(result) => {
                self.last_result = Some(result);
                self.last_error = None;
            }
            Err(e) => {
                self.last_error = Some(format!("{:?}", e));
                self.last_result = None;
            }
        }
        self
    }

    // ==================== Setup Methods ====================

    /// Setup graph space
    pub fn setup_space(mut self, space_name: &str) -> Self {
        let query = format!("CREATE SPACE IF NOT EXISTS {}", space_name);

        match self.pipeline.execute_query(&query) {
            Ok(result) => {
                match &result {
                    ExecutionResult::Success | ExecutionResult::Empty => {
                        self.last_result = Some(result);
                        self.last_error = None;
                    }
                    ExecutionResult::Error(e) => {
                        self.last_error = Some(format!("CREATE SPACE failed: {}", e));
                        self.last_result = Some(result);
                        return self;
                    }
                    _ => {
                        self.last_result = Some(result);
                        self.last_error = None;
                    }
                }
            }
            Err(e) => {
                self.last_error = Some(format!("{:?}", e));
                self.last_result = None;
                return self;
            }
        }

        let space_result = {
            let storage_guard = self.storage.lock();
            storage_guard.get_space(space_name)
        };

        match space_result {
            Ok(Some(space)) => {
                self.current_space = Some(space);
            }
            Ok(None) => {
                self.last_error = Some(format!(
                    "Space '{}' not found in storage after creation",
                    space_name
                ));
                return self;
            }
            Err(e) => {
                self.last_error = Some(format!("Failed to get space from storage: {}", e));
                return self;
            }
        }

        self
    }

    /// Setup schema with tags and edges
    pub fn setup_schema(mut self, ddls: Vec<&str>) -> Self {
        for ddl in ddls {
            match self.pipeline.execute_query(ddl) {
                Ok(result) => {
                    self.last_result = Some(result);
                    self.last_error = None;
                }
                Err(e) => {
                    self.last_error = Some(format!("{:?}", e));
                    self.last_result = None;
                }
            }
        }
        self
    }

    /// Load test data
    pub fn load_data(mut self, dmls: Vec<&str>) -> Self {
        for dml in dmls {
            match self.pipeline.execute_query(dml) {
                Ok(result) => {
                    self.last_result = Some(result);
                    self.last_error = None;
                }
                Err(e) => {
                    self.last_error = Some(format!("{:?}", e));
                    self.last_result = None;
                }
            }
        }
        self
    }

    // ==================== Assertion Methods ====================

    /// Assert that the last operation succeeded
    pub fn assert_success(self) -> Self {
        assert!(
            self.last_error.is_none(),
            "Expected success but got error: {:?}",
            self.last_error
        );
        self
    }

    /// Assert that the last operation failed
    pub fn assert_error(self) -> Self {
        assert!(
            self.last_error.is_some(),
            "Expected error but operation succeeded"
        );
        self
    }

    /// Assert result count
    pub fn assert_result_count(self, expected: usize) -> Self {
        let actual = self
            .last_result
            .as_ref()
            .map(|r| r.count())
            .unwrap_or(0);
        assert_eq!(
            actual, expected,
            "Expected {} results but got {}",
            expected, actual
        );
        self
    }

    /// Assert result is empty
    pub fn assert_result_empty(self) -> Self {
        self.assert_result_count(0)
    }

    /// Assert result columns
    pub fn assert_result_columns(self, expected: &[&str]) -> Self {
        if let Some(ref result) = self.last_result {
            let col_names: Vec<String> = match result {
                ExecutionResult::Result(r) => r.col_names().to_vec(),
                ExecutionResult::DataSet(ds) => ds.col_names.clone(),
                _ => vec![],
            };

            let expected: Vec<String> = expected.iter().map(|s| s.to_string()).collect();
            assert_eq!(
                col_names, expected,
                "Column names don't match. Expected {:?}, got {:?}",
                expected, col_names
            );
        } else {
            panic!("No result to check columns");
        }
        self
    }

    /// Assert result contains specific values
    pub fn assert_result_contains(self, expected: Vec<Value>) -> Self {
        if let Some(ref result) = self.last_result {
            let rows: Vec<Vec<Value>> = match result {
                ExecutionResult::Result(r) => r.rows().to_vec(),
                ExecutionResult::DataSet(ds) => ds.rows.clone(),
                _ => vec![],
            };

            let found = rows.iter().any(|row| row == &expected);
            assert!(
                found,
                "Expected to find row {:?} in results",
                expected
            );
        } else {
            panic!("No result to check");
        }
        self
    }

    // ==================== Data Validation Methods ====================

    /// Assert vertex exists
    pub fn assert_vertex_exists(mut self, vid: i64, tag: &str) -> Self {
        let query = format!("FETCH PROP ON {} {}", tag, vid);
        match self.pipeline.execute_query(&query) {
            Ok(result) => {
                assert!(
                    result.count() > 0,
                    "Expected vertex {} with tag {} to exist",
                    vid,
                    tag
                );
            }
            Err(e) => {
                panic!("Failed to check vertex existence: {:?}", e);
            }
        }
        self
    }

    /// Assert vertex does not exist
    pub fn assert_vertex_not_exists(mut self, vid: i64, tag: &str) -> Self {
        let query = format!("FETCH PROP ON {} {}", tag, vid);
        match self.pipeline.execute_query(&query) {
            Ok(result) => {
                assert!(
                    result.count() == 0,
                    "Expected vertex {} with tag {} to not exist",
                    vid,
                    tag
                );
            }
            Err(_e) => {
                // Error might mean vertex doesn't exist, which is what we want
            }
        }
        self
    }

    /// Assert vertex has specific properties
    pub fn assert_vertex_props(
        mut self,
        vid: i64,
        tag: &str,
        expected: HashMap<&str, Value>,
    ) -> Self {
        let query = format!("FETCH PROP ON {} {}", tag, vid);
        match self.pipeline.execute_query(&query) {
            Ok(result) => {
                let props = self.extract_props(&result);
                for (key, value) in expected {
                    assert_eq!(
                        props.get(key),
                        Some(&value),
                        "Property {} mismatch for vertex {}. Expected {:?}, got {:?}",
                        key,
                        vid,
                        value,
                        props.get(key)
                    );
                }
            }
            Err(e) => {
                panic!("Failed to get vertex properties: {:?}", e);
            }
        }
        self
    }

    /// Assert edge exists
    pub fn assert_edge_exists(mut self, src: i64, dst: i64, edge_type: &str) -> Self {
        let query = format!("FETCH PROP ON {} {} -> {}", edge_type, src, dst);
        match self.pipeline.execute_query(&query) {
            Ok(result) => {
                assert!(
                    result.count() > 0,
                    "Expected edge {} -> {} with type {} to exist",
                    src,
                    dst,
                    edge_type
                );
            }
            Err(e) => {
                panic!("Failed to check edge existence: {:?}", e);
            }
        }
        self
    }

    /// Assert edge does not exist
    pub fn assert_edge_not_exists(mut self, src: i64, dst: i64, edge_type: &str) -> Self {
        let query = format!("FETCH PROP ON {} {} -> {}", edge_type, src, dst);
        match self.pipeline.execute_query(&query) {
            Ok(result) => {
                assert!(
                    result.count() == 0,
                    "Expected edge {} -> {} with type {} to not exist",
                    src,
                    dst,
                    edge_type
                );
            }
            Err(_e) => {
                // Error might mean edge doesn't exist, which is what we want
            }
        }
        self
    }

    /// Assert tag exists
    pub fn assert_tag_exists(mut self, tag: &str) -> Self {
        let query = format!("DESC TAG {}", tag);
        match self.pipeline.execute_query(&query) {
            Ok(result) => {
                assert!(result.count() > 0, "Expected tag {} to exist", tag);
            }
            Err(e) => {
                panic!("Failed to check tag existence: {:?}", e);
            }
        }
        self
    }

    /// Assert tag does not exist
    pub fn assert_tag_not_exists(mut self, tag: &str) -> Self {
        let query = format!("DESC TAG {}", tag);
        match self.pipeline.execute_query(&query) {
            Ok(result) => {
                assert!(result.count() == 0, "Expected tag {} to not exist", tag);
            }
            Err(_e) => {
                // Error might mean tag doesn't exist, which is what we want
            }
        }
        self
    }

    /// Assert vertex count
    pub fn assert_vertex_count(mut self, tag: &str, expected: usize) -> Self {
        let query = format!("LOOKUP ON {}", tag);
        match self.pipeline.execute_query(&query) {
            Ok(result) => {
                let actual = result.count();
                assert_eq!(
                    actual, expected,
                    "Expected {} vertices with tag {}, got {}",
                    expected, tag, actual
                );
            }
            Err(e) => {
                panic!("Failed to count vertices: {:?}", e);
            }
        }
        self
    }

    /// Assert edge count
    pub fn assert_edge_count(mut self, edge_type: &str, expected: usize) -> Self {
        let query = format!("LOOKUP ON {}", edge_type);
        match self.pipeline.execute_query(&query) {
            Ok(result) => {
                let actual = result.count();
                assert_eq!(
                    actual, expected,
                    "Expected {} edges with type {}, got {}",
                    expected, edge_type, actual
                );
            }
            Err(e) => {
                panic!("Failed to count edges: {:?}", e);
            }
        }
        self
    }

    // ==================== Helper Methods ====================

    fn extract_props(&self, result: &ExecutionResult) -> HashMap<String, Value> {
        let mut props = HashMap::new();

        match result {
            ExecutionResult::Result(r) => {
                if let Some(row) = r.get_row(0) {
                    for (i, col_name) in r.col_names().iter().enumerate() {
                        if let Some(value) = row.get(i) {
                            props.insert(col_name.clone(), value.clone());
                        }
                    }
                }
            }
            ExecutionResult::DataSet(ds) => {
                if let Some(row) = ds.rows.get(0) {
                    for (i, col_name) in ds.col_names.iter().enumerate() {
                        if let Some(value) = row.get(i) {
                            props.insert(col_name.clone(), value.clone());
                        }
                    }
                }
            }
            _ => {}
        }

        props
    }
}

impl Default for TestScenario {
    fn default() -> Self {
        Self::new().expect("Failed to create default TestScenario")
    }
}
