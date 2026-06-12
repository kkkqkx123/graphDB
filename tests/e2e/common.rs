//! Common test utilities for E2E tests
//!
//! Uses the QueryApi with schema manager for proper initialization.
//! This is the recommended way to create test databases for E2E tests.

use graphdb::api::core::query_api::QueryApi;
use graphdb::api::core::types::QueryResult;
use graphdb::api::core::CoreResult;
use graphdb::core::metadata::SchemaManager;
use graphdb::core::Value;
use graphdb::core::StatsManager;
use graphdb::query::OptimizerEngine;
use graphdb::search::{FulltextConfig, FulltextIndexManager, SyncFailurePolicy};
use graphdb::storage::{GraphStorage, StorageClient, StorageSchemaContextOps};
use graphdb::sync::{SyncConfig, SyncManager};
use parking_lot::RwLock;
use std::sync::Arc;
use tempfile::TempDir;

#[cfg(feature = "qdrant")]
use vector_client::{VectorClientConfig, VectorManager};

/// Test database wrapper with proper schema manager initialization
pub struct TestDb {
    temp_dir: Option<TempDir>,
    storage: Arc<RwLock<GraphStorage>>,
    stats_manager: Arc<StatsManager>,
    schema_manager: Arc<SchemaManager>,
    query_api: QueryApi<GraphStorage>,
    current_space_id: Option<u64>,
    current_space_name: Option<String>,
}

fn create_sync_manager() -> Arc<SyncManager> {
    let fulltext_config = FulltextConfig::default();
    let manager = Arc::new(
        FulltextIndexManager::new(fulltext_config).expect("Failed to create fulltext manager"),
    );
    let sync_config = SyncConfig::default();
    let batch_config = graphdb::sync::batch::BatchConfig::from(sync_config.clone());
    let sync_coordinator = Arc::new(graphdb::sync::coordinator::SyncCoordinator::new(
        manager,
        batch_config,
    ));

    let mut sync_manager = SyncManager::with_sync_config(sync_coordinator, sync_config);

    #[cfg(feature = "qdrant")]
    {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        let vector_manager = rt
            .block_on(VectorManager::new(VectorClientConfig::disabled()))
            .expect("Failed to create disabled vector manager");
        let vector_coordinator = Arc::new(graphdb::sync::vector_sync::VectorSyncCoordinator::new(
            Arc::new(vector_manager),
            None,
        ));
        sync_manager = sync_manager.with_vector_coordinator(vector_coordinator);
    }

    Arc::new(sync_manager)
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

        let sync_manager = create_sync_manager();
        let query_api = QueryApi::with_schema_and_sync_manager(
            storage.clone(),
            stats_manager.clone(),
            schema_manager.clone(),
            sync_manager,
        );

        Self {
            temp_dir: Some(temp_dir),
            storage,
            stats_manager,
            schema_manager,
            query_api,
            current_space_id: None,
            current_space_name: None,
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

        let sync_manager = create_sync_manager();
        let query_api = QueryApi::with_schema_and_sync_manager(
            storage.clone(),
            stats_manager.clone(),
            schema_manager.clone(),
            sync_manager,
        );

        Self {
            temp_dir: None,
            storage,
            stats_manager,
            schema_manager,
            query_api,
            current_space_id: None,
            current_space_name: None,
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
            space_id: self.current_space_id,
            space_name: self.current_space_name.clone(),
            auto_commit: true,
            transaction_id: None,
            parameters: None,
        };
        let result = self.query_api.execute(query, ctx)?;

        // Track space switching from USE statements
        if result.columns.iter().any(|c| c == "space_name") {
            if let Some(row) = result.rows.first() {
                if let Some(Value::String(name)) = row.values.get("space_name") {
                    self.current_space_name = Some(name.clone());
                }
                if let Some(Value::BigInt(id)) = row.values.get("space_id") {
                    self.current_space_id = Some(*id as u64);
                }
            }
        }

        Ok(result)
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

/// Load and execute a GQL data file
///
/// Reads the file line-by-line.  Blank lines and comment lines (`--`)
/// are statement separators.  Continuation lines (indented, or starting
/// with `)`) are appended to the current statement.
pub fn load_gql_file(db: &mut TestDb, path: &str) -> CoreResult<()> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| graphdb::api::core::CoreError::Internal(format!("Failed to read {}: {}", path, e)))?;

    let mut buffer = String::new();
    let mut stmt_num: usize = 0;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("--") {
            if !buffer.is_empty() {
                stmt_num += 1;
                if let Err(e) = db.execute_query(&buffer) {
                    return Err(e);
                }
                buffer.clear();
            }
            continue;
        }
        if line.starts_with(' ') || line.starts_with('\t') || trimmed.starts_with(')') {
            buffer.push(' ');
            buffer.push_str(trimmed);
        } else {
            if !buffer.is_empty() {
                stmt_num += 1;
                if let Err(e) = db.execute_query(&buffer) {
                    return Err(e);
                }
            }
            buffer = trimmed.to_string();
        }
    }
    if !buffer.is_empty() {
        stmt_num += 1;
                if let Err(e) = db.execute_query(&buffer) {
                    return Err(e);
                }
    }

    Ok(())
}

/// Assert that `result` is Ok and that the QueryResult contains exactly `expected` rows
pub fn assert_row_count(result: CoreResult<QueryResult>, expected: usize, context: &str) {
    match result {
        Ok(ref qr) => assert_eq!(
            qr.rows.len(),
            expected,
            "{}: expected {} rows, got {}",
            context,
            expected,
            qr.rows.len()
        ),
        Err(e) => panic!("{}: query failed: {:?}", context, e),
    }
}

/// Assert that a single-column count query returns the expected value
///
/// Executes `query` and reads the first row's first value as i64.
pub fn assert_count_eq(db: &mut TestDb, query: &str, expected: i64, context: &str) {
    match db.execute_query(query) {
        Ok(qr) => {
            let first = qr
                .rows
                .first()
                .expect(&format!("{}: result set is empty", context));
            let val = first
                .values
                .values()
                .next()
                .expect(&format!("{}: no column", context));
            let actual = match val {
                Value::BigInt(v) => *v as i64,
                Value::Int(v) => *v as i64,
                Value::SmallInt(v) => *v as i64,
                other => panic!("{}: expected numeric value, got {:?}", context, other),
            };
            assert_eq!(
                actual, expected,
                "{}: expected count {}, got {}",
                context, expected, actual
            );
        }
        Err(e) => panic!("{}: query failed: {:?}", context, e),
    }
}

/// Assert that a query succeeds and returns exactly `expected` rows
pub fn assert_query_row_count(
    db: &mut TestDb,
    query: &str,
    expected: usize,
    context: &str,
) {
    match db.execute_query(query) {
        Ok(qr) => {
            let actual = qr.rows.len();
            assert_eq!(
                actual, expected,
                "{}: expected {} rows, got {}",
                context, expected, actual
            );
        }
        Err(e) => panic!("{}: query failed: {:?}", context, e),
    }
}

/// Assert that a single-value query returns the expected f64 value (within epsilon)
pub fn assert_float_eq(db: &mut TestDb, query: &str, expected: f64, context: &str) {
    match db.execute_query(query) {
        Ok(qr) => {
            let first = qr
                .rows
                .first()
                .expect(&format!("{}: result set is empty", context));
            let val = first
                .values
                .values()
                .next()
                .expect(&format!("{}: no column", context));
            let actual = match val {
                Value::Double(v) => *v,
                Value::Float(v) => *v as f64,
                other => panic!("{}: expected float, got {:?}", context, other),
            };
            assert!(
                (actual - expected).abs() < 1e-6,
                "{}: expected {}, got {}",
                context,
                expected,
                actual
            );
        }
        Err(e) => panic!("{}: query failed: {:?}", context, e),
    }
}
