//! DCL Permission Tests
//!
//! Test coverage:
//! - GRANT - Grant privileges to users
//! - REVOKE - Revoke privileges from users

use super::common;

use common::TestStorage;

use graphdb::core::stats::StatsManager;
use graphdb::query::optimizer::OptimizerEngine;
use graphdb::query::parser::Parser;
use graphdb::query::query_pipeline_manager::QueryPipelineManager;
use std::sync::Arc;

// ==================== GRANT Parser Tests ====================

#[test]
fn test_grant_parser_basic() {
    let query = "GRANT ROLE ADMIN ON test_space TO alice";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "GRANT basic parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("GRANT statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "GRANT");
}

#[test]
fn test_grant_parser_without_role_keyword() {
    let query = "GRANT ADMIN ON test_space TO alice";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "GRANT without ROLE keyword parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("GRANT statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "GRANT");
}

#[test]
fn test_grant_parser_all_roles() {
    let queries = vec![
        "GRANT GOD ON test_space TO user1",
        "GRANT ADMIN ON test_space TO user2",
        "GRANT DBA ON test_space TO user3",
        "GRANT USER ON test_space TO user4",
        "GRANT GUEST ON test_space TO user5",
    ];

    for query in queries {
        let mut parser = Parser::new(query);
        let result = parser.parse();
        assert!(
            result.is_ok(),
            "GRANT role {} parsing should succeed: {:?}",
            query,
            result.err()
        );
    }
}

// ==================== REVOKE Parser Tests ====================

#[test]
fn test_revoke_parser_basic() {
    let query = "REVOKE ROLE ADMIN ON test_space FROM alice";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "REVOKE basic parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("REVOKE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "REVOKE");
}

#[test]
fn test_revoke_parser_without_role_keyword() {
    let query = "REVOKE ADMIN ON test_space FROM alice";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "REVOKE without ROLE keyword parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("REVOKE statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "REVOKE");
}

// ==================== GRANT/REVOKE Execution Tests ====================

#[test]
fn test_grant_revoke_execution() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());

    let mut pipeline_manager = QueryPipelineManager::with_optimizer(
        storage,
        stats_manager,
        Arc::new(OptimizerEngine::default()),
    );

    // Create user first
    let create_result =
        pipeline_manager.execute_query("CREATE USER alice WITH PASSWORD 'password123'");
    assert!(
        create_result.is_ok(),
        "CREATE USER should succeed: {:?}",
        create_result.err()
    );

    let create_space =
        pipeline_manager.execute_query("CREATE SPACE test_space WITH DIMENSION=128");
    assert!(
        create_space.is_ok(),
        "CREATE SPACE should succeed: {:?}",
        create_space.err()
    );

    // Grant role
    let grant_result = pipeline_manager.execute_query("GRANT ADMIN ON test_space TO alice");
    assert!(
        grant_result.is_ok(),
        "GRANT should succeed: {:?}",
        grant_result.err()
    );

    // Revoke role
    let revoke_result = pipeline_manager.execute_query("REVOKE ADMIN ON test_space FROM alice");
    assert!(
        revoke_result.is_ok(),
        "REVOKE should succeed: {:?}",
        revoke_result.err()
    );

    // Drop user
    let drop_result = pipeline_manager.execute_query("DROP USER alice");
    assert!(
        drop_result.is_ok(),
        "DROP USER should succeed: {:?}",
        drop_result.err()
    );
}

#[test]
fn test_grant_multiple_roles() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());

    let mut pipeline_manager = QueryPipelineManager::with_optimizer(
        storage,
        stats_manager,
        Arc::new(OptimizerEngine::default()),
    );

    let create_result =
        pipeline_manager.execute_query("CREATE USER multi_role_user WITH PASSWORD 'password'");
    assert!(
        create_result.is_ok(),
        "CREATE USER should succeed: {:?}",
        create_result.err()
    );

    for space_name in ["space1", "space2", "space3"] {
        let create_space = pipeline_manager.execute_query(&format!(
            "CREATE SPACE {} WITH DIMENSION=128",
            space_name
        ));
        assert!(
            create_space.is_ok(),
            "CREATE SPACE should succeed: {:?}",
            create_space.err()
        );
    }

    let grant_queries = [
        "GRANT ADMIN ON space1 TO multi_role_user",
        "GRANT DBA ON space2 TO multi_role_user",
        "GRANT USER ON space3 TO multi_role_user",
    ];
    for (i, q) in grant_queries.iter().enumerate() {
        let result = pipeline_manager.execute_query(q);
        assert!(
            result.is_ok(),
            "GRANT {} should succeed: {:?}",
            i,
            result.err()
        );
    }

    let revoke_queries = [
        "REVOKE ADMIN ON space1 FROM multi_role_user",
        "REVOKE DBA ON space2 FROM multi_role_user",
        "REVOKE USER ON space3 FROM multi_role_user",
    ];
    for (i, q) in revoke_queries.iter().enumerate() {
        let result = pipeline_manager.execute_query(q);
        assert!(
            result.is_ok(),
            "REVOKE {} should succeed: {:?}",
            i,
            result.err()
        );
    }

    let drop_result = pipeline_manager.execute_query("DROP USER multi_role_user");
    assert!(
        drop_result.is_ok(),
        "DROP USER should succeed: {:?}",
        drop_result.err()
    );
}

#[test]
fn test_grant_nonexistent_user() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());

    let mut pipeline_manager = QueryPipelineManager::with_optimizer(
        storage,
        stats_manager,
        Arc::new(OptimizerEngine::default()),
    );

    let query = "GRANT ADMIN ON test_space TO nonexistent_user";
    let result = pipeline_manager.execute_query(query);

    assert!(result.is_err(), "GRANT to nonexistent user should fail");
}

#[test]
fn test_revoke_nonexistent_permission() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());

    let mut pipeline_manager = QueryPipelineManager::with_optimizer(
        storage,
        stats_manager,
        Arc::new(OptimizerEngine::default()),
    );

    let create_result =
        pipeline_manager.execute_query("CREATE USER testuser WITH PASSWORD 'password'");
    assert!(
        create_result.is_ok(),
        "CREATE USER should succeed: {:?}",
        create_result.err()
    );

    let create_space =
        pipeline_manager.execute_query("CREATE SPACE test_space WITH DIMENSION=128");
    assert!(
        create_space.is_ok(),
        "CREATE SPACE should succeed: {:?}",
        create_space.err()
    );

    // Revoking a permission that doesn't exist may succeed (no-op) or fail depending on implementation
    let revoke_result = pipeline_manager.execute_query("REVOKE ADMIN ON test_space FROM testuser");
    assert!(
        revoke_result.is_ok(),
        "REVOKE should handle nonexistent permission gracefully: {:?}",
        revoke_result.err()
    );

    let drop_result = pipeline_manager.execute_query("DROP USER testuser");
    assert!(
        drop_result.is_ok(),
        "DROP USER should succeed: {:?}",
        drop_result.err()
    );
}
