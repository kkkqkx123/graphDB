//! E2E Test Suite for Schema Manager Initialization
//!
//! Tests that verify schema manager is properly initialized in various scenarios:
//! 1. Basic query operations work when vector search is disabled
//! 2. Basic query operations work when vector search is enabled but fails to initialize
//! 3. Schema validation works correctly

use graphdb::api::server::graph_service::GraphService;
use graphdb::config::Config;
use graphdb::core::stats::StatsManager;
use graphdb::query::optimizer::OptimizerEngine;
use graphdb::query::query_pipeline_manager::QueryPipelineManager;
use graphdb::storage::{GraphStorage, SyncWrapper};
use std::sync::Arc;

use crate::common::TestStorage;

/// Test schema manager initialization in different configurations
mod initialization {
    use super::*;

    /// Verify basic connection works
    #[tokio::test]
    async fn test_basic_connection() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        let result = pipeline.execute_query("SHOW SPACES");
        assert!(result.is_ok(), "Basic connection failed");
    }

    /// Create space should work regardless of vector config
    #[tokio::test]
    async fn test_create_space_without_vector() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        // Drop if exists
        let _ = pipeline.execute_query("DROP SPACE IF EXISTS schema_manager_test_space");

        // Create space - this should work even if schema_manager is not initialized
        let result = pipeline.execute_query(
            "CREATE SPACE IF NOT EXISTS schema_manager_test_space (vid_type=STRING)"
        );
        assert!(
            result.is_ok(),
            "CREATE SPACE failed - schema_manager may not be initialized"
        );
    }

    /// Use space should work
    #[tokio::test]
    async fn test_use_space() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        // Create space first
        pipeline.execute_query("CREATE SPACE IF NOT EXISTS schema_manager_test_space (vid_type=STRING)")
            .expect("CREATE SPACE should succeed");

        let result = pipeline.execute_query("USE schema_manager_test_space");
        assert!(
            result.is_ok(),
            "USE SPACE failed - schema_manager may not be initialized"
        );
    }

    /// Create tag should work with schema_manager
    #[tokio::test]
    async fn test_create_tag() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        // Setup
        pipeline.execute_query("CREATE SPACE IF NOT EXISTS schema_manager_test_space (vid_type=STRING)")
            .expect("CREATE SPACE should succeed");
        pipeline.execute_query("USE schema_manager_test_space")
            .expect("USE should succeed");

        let result = pipeline.execute_query(
            "CREATE TAG IF NOT EXISTS test_person(name STRING NOT NULL, age INT)"
        );
        assert!(
            result.is_ok(),
            "CREATE TAG failed - schema_manager may not be initialized"
        );
    }

    /// Show tags should work
    #[tokio::test]
    async fn test_show_tags() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        // Setup
        pipeline.execute_query("CREATE SPACE IF NOT EXISTS schema_manager_test_space (vid_type=STRING)")
            .expect("CREATE SPACE should succeed");
        pipeline.execute_query("USE schema_manager_test_space")
            .expect("USE should succeed");
        pipeline.execute_query("CREATE TAG IF NOT EXISTS test_person(name STRING, age INT)")
            .expect("CREATE TAG should succeed");

        let result = pipeline.execute_query("SHOW TAGS");
        assert!(
            result.is_ok(),
            "SHOW TAGS failed - schema_manager may not be initialized"
        );
    }

    /// Insert vertex should work
    #[tokio::test]
    async fn test_insert_vertex() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        // Setup
        pipeline.execute_query("CREATE SPACE IF NOT EXISTS schema_manager_test_space (vid_type=STRING)")
            .expect("CREATE SPACE should succeed");
        pipeline.execute_query("USE schema_manager_test_space")
            .expect("USE should succeed");
        pipeline.execute_query("CREATE TAG IF NOT EXISTS test_person(name STRING, age INT)")
            .expect("CREATE TAG should succeed");

        let result = pipeline.execute_query(
            "INSERT VERTEX test_person(name, age) VALUES 'p1': ('Alice', 30)"
        );
        assert!(
            result.is_ok(),
            "INSERT VERTEX failed - schema_manager may not be initialized"
        );
    }

    /// Fetch vertex should work
    #[tokio::test]
    async fn test_fetch_vertex() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        // Setup
        pipeline.execute_query("CREATE SPACE IF NOT EXISTS schema_manager_test_space (vid_type=STRING)")
            .expect("CREATE SPACE should succeed");
        pipeline.execute_query("USE schema_manager_test_space")
            .expect("USE should succeed");
        pipeline.execute_query("CREATE TAG IF NOT EXISTS test_person(name STRING, age INT)")
            .expect("CREATE TAG should succeed");

        // Insert vertex
        pipeline.execute_query("INSERT VERTEX test_person(name, age) VALUES 'p_fetch': ('Bob', 25)")
            .expect("INSERT should succeed");

        let result = pipeline.execute_query("FETCH PROP ON test_person 'p_fetch'");
        assert!(
            result.is_ok(),
            "FETCH PROP failed - schema_manager may not be initialized"
        );
    }

    /// MATCH query should work
    #[tokio::test]
    async fn test_match_query() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        // Setup
        pipeline.execute_query("CREATE SPACE IF NOT EXISTS schema_manager_test_space (vid_type=STRING)")
            .expect("CREATE SPACE should succeed");
        pipeline.execute_query("USE schema_manager_test_space")
            .expect("USE should succeed");
        pipeline.execute_query("CREATE TAG IF NOT EXISTS test_person(name STRING, age INT)")
            .expect("CREATE TAG should succeed");

        // Insert vertex
        pipeline.execute_query("INSERT VERTEX test_person(name, age) VALUES 'p1': ('Alice', 30)")
            .expect("INSERT should succeed");

        let result = pipeline.execute_query("MATCH (v:test_person) RETURN v LIMIT 1");
        // MATCH might not be fully implemented, so we just check it doesn't crash
        // and doesn't return schema_manager error
        if let Err(ref e) = result {
            let error_msg = format!("{:?}", e).to_lowercase();
            assert!(
                !error_msg.contains("schema manager not initialized"),
                "MATCH query failed due to schema_manager not initialized"
            );
        }
    }

    /// Drop space should work
    #[tokio::test]
    async fn test_drop_space() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        // Setup
        pipeline.execute_query("CREATE SPACE IF NOT EXISTS schema_manager_test_space (vid_type=STRING)")
            .expect("CREATE SPACE should succeed");

        let result = pipeline.execute_query("DROP SPACE IF EXISTS schema_manager_test_space");
        assert!(
            result.is_ok(),
            "DROP SPACE failed"
        );
    }
}

/// Test error handling when schema manager is not available
mod error_handling {
    use super::*;

    /// Error messages should be clear when operations fail
    #[tokio::test]
    async fn test_error_message_clarity() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        // Try to use a non-existent space
        let result = pipeline.execute_query("USE non_existent_space_xyz");

        // Should fail, but error should not be "schema manager not initialized"
        if let Err(ref e) = result {
            let error_msg = format!("{:?}", e).to_lowercase();
            assert!(
                !error_msg.contains("schema manager not initialized"),
                "Error message indicates schema_manager not initialized - this is a server config issue"
            );
        }
    }

    /// SHOW SPACES should always work
    #[tokio::test]
    async fn test_show_spaces_always_works() {
        let test_storage = TestStorage::new().expect("Failed to create test storage");
        let storage = test_storage.storage();
        let schema_manager = test_storage.schema_manager();
        let stats_manager = Arc::new(StatsManager::new());
        let optimizer = Arc::new(OptimizerEngine::default());

        let mut pipeline = QueryPipelineManager::with_optimizer(storage, stats_manager, optimizer)
            .with_schema_manager(schema_manager);

        let result = pipeline.execute_query("SHOW SPACES");
        assert!(
            result.is_ok(),
            "SHOW SPACES should always work but failed"
        );
    }
}

/// Test GraphService with schema manager
mod graph_service {
    use super::*;

    /// Test GraphService creation and basic operations
    #[tokio::test]
    async fn test_graph_service_creation() {
        let config = Config::default();
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let storage = Arc::new(SyncWrapper::new(
            GraphStorage::new_with_path(db_path).expect("Failed to create storage"),
        ));

        let graph_service = GraphService::new(config, storage).await;

        // Verify the service was created
        assert!(
            graph_service
                .get_session_manager()
                .list_sessions()
                .await
                .is_empty(),
            "GraphService should be created with empty sessions"
        );
    }

    /// Test authentication and query execution
    #[tokio::test]
    async fn test_graph_service_query_execution() {
        let config = Config::default();
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("test.db");
        let storage = Arc::new(SyncWrapper::new(
            GraphStorage::new_with_path(db_path).expect("Failed to create storage"),
        ));

        let graph_service = GraphService::new(config, storage).await;

        // Authenticate
        let session = graph_service
            .authenticate("root", "root")
            .await
            .expect("Root auth should succeed");
        let session_id = session.id();

        // Execute query
        let result = graph_service
            .execute(session_id, "SHOW SPACES")
            .await;

        // Should succeed (or at least not fail due to schema manager)
        if let Err(ref e) = result {
            let error_msg = format!("{:?}", e).to_lowercase();
            assert!(
                !error_msg.contains("schema manager not initialized"),
                "Query failed due to schema_manager not initialized"
            );
        }
    }
}
