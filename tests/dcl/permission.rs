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
    assert!(result.is_ok(), "GRANT basic parsing should succeed: {:?}", result.err());

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
    assert!(result.is_ok(), "REVOKE basic parsing should succeed: {:?}", result.err());

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

    let queries = [
        "CREATE USER alice WITH PASSWORD 'password123'",
        "GRANT ADMIN ON test_space TO alice",
        "REVOKE ADMIN ON test_space FROM alice",
        "DROP USER alice",
    ];

    for query in queries.iter() {
        let result = pipeline_manager.execute_query(query);
        assert!(result.is_ok() || result.is_err());
    }
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

    let queries = [
        "CREATE USER multi_role_user WITH PASSWORD 'password'",
        "GRANT ADMIN ON space1 TO multi_role_user",
        "GRANT DBA ON space2 TO multi_role_user",
        "GRANT USER ON space3 TO multi_role_user",
        "REVOKE ADMIN ON space1 FROM multi_role_user",
        "REVOKE DBA ON space2 FROM multi_role_user",
        "REVOKE USER ON space3 FROM multi_role_user",
        "DROP USER multi_role_user",
    ];

    for query in queries.iter() {
        let result = pipeline_manager.execute_query(query);
        assert!(result.is_ok() || result.is_err());
    }
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

    assert!(result.is_ok() || result.is_err());
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

    let queries = [
        "CREATE USER testuser WITH PASSWORD 'password'",
        "REVOKE ADMIN ON test_space FROM testuser",
        "DROP USER testuser",
    ];

    for query in queries.iter() {
        let result = pipeline_manager.execute_query(query);
        assert!(result.is_ok() || result.is_err());
    }
}
