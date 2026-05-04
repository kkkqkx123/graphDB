//! Server Complete Workflow Integration Tests
//!
//! Test coverage:
//! - GraphService initialization with schema_manager
//! - Basic query execution (CREATE SPACE, USE, CREATE TAG, etc.)
//! - Vector search configuration handling
//! - Error handling when schema_manager is not available

mod common;

use common::TestStorage;
use graphdb::api::core::QueryApi;
use graphdb::api::server::graph_service::GraphService;
use graphdb::config::Config;
use graphdb::core::stats::StatsManager;
use graphdb::query::optimizer::OptimizerEngine;
use graphdb::query::query_pipeline_manager::QueryPipelineManager;
use graphdb::storage::GraphStorage;
use graphdb::storage::entity::event_storage::SyncStorage;
use graphdb::storage::api::StorageClient;
use std::sync::Arc;
use vector_client::VectorClientConfig;

/// Test that GraphService can be created with SyncStorage<GraphStorage>
#[tokio::test]
async fn test_graph_service_creation_with_sync_storage() {
    let config = Config::default();

    // Create SyncStorage<GraphStorage>
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let storage = Arc::new(SyncStorage::new(
        GraphStorage::new_with_path(db_path).expect("Failed to create storage"),
    ));

    // Create GraphService - this should work with our fix
    let graph_service = GraphService::new(config, storage).await;

    // Verify the service was created
    assert!(
        graph_service.get_session_manager().list_sessions().await.is_empty(),
        "GraphService should be created with empty sessions"
    );
}

/// Test QueryApi creation with schema_manager from GraphStorage
#[test]
fn test_query_api_with_graph_storage_schema_manager() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let schema_manager = test_storage.schema_manager();

    // Create QueryApi with schema_manager
    let query_api = QueryApi::with_schema_manager(storage, schema_manager);

    // QueryApi should be created successfully
    // We cannot easily test execution here without full setup,
    // but we verify the API was created without panicking
    let _ = query_api;
}

/// Test QueryPipelineManager behavior with and without schema_manager
#[test]
fn test_pipeline_manager_schema_manager_behavior() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    let optimizer_engine = Arc::new(OptimizerEngine::default());

    // Test 1: Without schema_manager, CREATE SPACE should fail
    let mut pipeline_manager_without =
        QueryPipelineManager::with_optimizer(storage.clone(), stats_manager.clone(), optimizer_engine.clone());

    let result = pipeline_manager_without.execute_query("CREATE SPACE test (vid_type=STRING)");
    assert!(
        result.is_err(),
        "CREATE SPACE should fail without schema_manager"
    );
    let error_msg = format!("{:?}", result.err()).to_lowercase();
    assert!(
        error_msg.contains("schema") || error_msg.contains("not initialized"),
        "Error should mention schema manager: {}",
        error_msg
    );

    // Test 2: With schema_manager, CREATE SPACE should succeed
    let schema_manager = test_storage.schema_manager();
    let mut pipeline_manager_with =
        QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer_engine)
            .with_schema_manager(schema_manager);

    let result = pipeline_manager_with.execute_query("CREATE SPACE test2 (vid_type=STRING)");
    assert!(
        result.is_ok(),
        "CREATE SPACE should succeed with schema_manager: {:?}",
        result.err()
    );
}

/// Test that VectorClientConfig::default() returns disabled config
#[test]
fn test_vector_config_default_is_disabled() {
    let config = VectorClientConfig::default();

    assert!(
        !config.enabled,
        "VectorClientConfig::default() should return disabled config"
    );
}

/// Test SyncStorage can provide access to inner storage
#[test]
fn test_sync_storage_inner_access() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let storage = GraphStorage::new_with_path(db_path).expect("Failed to create storage");

    // Create SyncStorage
    let sync_storage = SyncStorage::new(storage);

    // Access inner storage
    let inner = sync_storage.inner();

    // Verify we can access schema_manager from inner storage
    let _schema_manager = inner.state().schema_manager.clone();
    // If we get here without panic, the access works
}

/// Test StorageClient trait method get_schema_manager
#[test]
fn test_storage_client_get_schema_manager() {
    // Test with GraphStorage
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let storage = GraphStorage::new_with_path(db_path).expect("Failed to create storage");

    // get_schema_manager should return Some
    let schema_manager = storage.get_schema_manager();
    assert!(
        schema_manager.is_some(),
        "GraphStorage should return Some schema_manager"
    );

    // Test with SyncStorage<GraphStorage>
    let sync_storage = SyncStorage::new(storage);
    let schema_manager = sync_storage.get_schema_manager();
    assert!(
        schema_manager.is_some(),
        "SyncStorage<GraphStorage> should return Some schema_manager"
    );
}

/// Test error handling when schema_manager is not available
#[test]
fn test_schema_manager_error_handling() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    let optimizer_engine = Arc::new(OptimizerEngine::default());

    let mut pipeline_manager =
        QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer_engine);

    // CREATE SPACE requires schema_manager and should fail without it
    let result = pipeline_manager.execute_query("CREATE SPACE test (vid_type=STRING)");
    assert!(
        result.is_err(),
        "CREATE SPACE should fail without schema_manager"
    );

    let error_msg = format!("{:?}", result.err()).to_lowercase();
    assert!(
        error_msg.contains("schema") || error_msg.contains("not initialized"),
        "Error should mention schema manager: {}",
        error_msg
    );
}

/// Integration test: Complete workflow from storage to query execution
#[test]
fn test_complete_storage_to_query_workflow() {
    // Step 1: Create storage
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let schema_manager = test_storage.schema_manager();

    // Step 2: Create QueryApi with schema_manager
    let mut query_api = QueryApi::with_schema_manager(storage, schema_manager);

    // Step 3: Execute a query request
    let request = graphdb::api::core::types::QueryRequest {
        space_id: None,
        space_name: None,
        auto_commit: true,
        transaction_id: None,
        parameters: None,
    };

    let result = query_api.execute("CREATE SPACE workflow_test (vid_type=STRING)", request);

    // The query should succeed
    assert!(
        result.is_ok(),
        "Complete workflow should succeed: {:?}",
        result.err()
    );
}
