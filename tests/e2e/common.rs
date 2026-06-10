//! Common test utilities for E2E tests
//!
//! Uses the QueryApi with schema manager for proper initialization.
//! This is the recommended way to create test databases for E2E tests.

use graphdb::api::core::query_api::QueryApi;
use graphdb::api::core::types::QueryResult;
use graphdb::api::core::CoreResult;
use graphdb::core::metadata::SchemaManager;
use graphdb::core::StatsManager;
use graphdb::query::OptimizerEngine;
use graphdb::storage::{GraphStorage, StorageClient, StorageSchemaContextOps};
use parking_lot::RwLock;
use std::sync::Arc;

/// Test database wrapper with proper schema manager initialization
pub struct TestDb {
    storage: Arc<RwLock<GraphStorage>>,
    stats_manager: Arc<StatsManager>,
    schema_manager: Arc<SchemaManager>,
    query_api: QueryApi<GraphStorage>,
}

impl TestDb {
    /// Create a new test database with a temporary file
    pub fn new() -> Self {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let storage = Arc::new(RwLock::new(
            GraphStorage::open(db_path).expect("Failed to create storage"),
        ));
        let stats_manager = Arc::new(StatsManager::new());
        let schema_manager = storage
            .read()
            .get_schema_manager()
            .expect("Storage should provide a schema manager");

        let optimizer = Arc::new(OptimizerEngine::default());
        let query_api = QueryApi::with_schema_manager(
            storage.clone(),
            stats_manager.clone(),
            schema_manager.clone(),
        );

        Self {
            storage,
            stats_manager,
            schema_manager,
            query_api,
        }
    }

    /// Create a new test database in memory
    pub fn new_in_memory() -> Self {
        let storage = Arc::new(RwLock::new(
            GraphStorage::new().expect("Failed to create storage"),
        ));
        let stats_manager = Arc::new(StatsManager::new());
        let schema_manager = storage
            .read()
            .get_schema_manager()
            .expect("Storage should provide a schema manager");

        let optimizer = Arc::new(OptimizerEngine::default());
        let query_api = QueryApi::with_schema_manager(
            storage.clone(),
            stats_manager.clone(),
            schema_manager.clone(),
        );

        Self {
            storage,
            stats_manager,
            schema_manager,
            query_api,
        }
    }

    /// Get a reference to the storage
    pub fn storage(&self) -> Arc<RwLock<GraphStorage>> {
        self.storage.clone()
    }

    /// Get a reference to the stats manager
    pub fn stats_manager(&self) -> Arc<StatsManager> {
        self.stats_manager.clone()
    }

    /// Get a reference to the schema manager
    pub fn schema_manager(&self) -> Arc<SchemaManager> {
        self.schema_manager.clone()
    }

    /// Execute a query using a persistent session context
    pub fn execute_query(&mut self, query: &str) -> CoreResult<QueryResult> {
        let ctx = graphdb::api::core::types::QueryRequest {
            space_id: None,
            space_name: None,
            auto_commit: true,
            transaction_id: None,
            parameters: None,
        };
        self.query_api.execute(query, ctx)
    }
}

/// Create a test database
pub fn create_test_db() -> TestDb {
    TestDb::new()
}

/// Create an in-memory test database
pub fn create_test_db_in_memory() -> TestDb {
    TestDb::new_in_memory()
}

/// Setup a test space with schema
///
/// Creates a space, uses it, and creates the provided tags and edges.
/// Returns the test db for further operations.
pub fn setup_test_space(
    db: &mut TestDb,
    space_name: &str,
    tags: &[&str],
    edges: &[&str],
) -> CoreResult<()> {
    // Drop space if exists (ignore error)
    let _ = db.execute_query(&format!("DROP SPACE IF EXISTS {}", space_name));

    // Create and use space
    db.execute_query(&format!("CREATE SPACE {} (vid_type=STRING)", space_name))?;
    db.execute_query(&format!("USE {}", space_name))?;

    // Create tags
    for tag in tags {
        db.execute_query(tag)?;
    }

    // Create edges
    for edge in edges {
        db.execute_query(edge)?;
    }

    Ok(())
}

/// Assert that a query succeeds
pub fn assert_query_ok<T: std::fmt::Debug>(result: CoreResult<T>, context: &str) {
    assert!(result.is_ok(), "{}: {:?}", context, result.err());
}

/// Assert that a query fails
pub fn assert_query_err<T: std::fmt::Debug>(result: CoreResult<T>, context: &str) {
    assert!(result.is_err(), "{}: expected error but got Ok", context);
}
