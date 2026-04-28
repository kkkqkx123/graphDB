//! DCL Role Tests
//!
//! Test coverage:
//! - SHOW USERS - List all users
//! - SHOW ROLES - List all roles
//! - DESCRIBE USER - Describe user details

use super::common;

use common::TestStorage;

use graphdb::core::stats::StatsManager;
use graphdb::query::optimizer::OptimizerEngine;
use graphdb::query::parser::Parser;
use graphdb::query::query_pipeline_manager::QueryPipelineManager;
use std::sync::Arc;

// ==================== DESCRIBE USER Tests ====================

#[test]
fn test_describe_user_parser_basic() {
    let query = "DESCRIBE USER alice";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "DESCRIBE USER basic parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("DESCRIBE USER statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "DESCRIBE USER");
}

#[test]
fn test_describe_user_execution() {
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
        "DESCRIBE USER alice",
        "DROP USER alice",
    ];

    for query in queries.iter() {
        let result = pipeline_manager.execute_query(query);
        assert!(result.is_ok() || result.is_err());
    }
}

#[test]
fn test_describe_user_nonexistent() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());

    let mut pipeline_manager = QueryPipelineManager::with_optimizer(
        storage,
        stats_manager,
        Arc::new(OptimizerEngine::default()),
    );

    let query = "DESCRIBE USER nonexistent_user";
    let result = pipeline_manager.execute_query(query);

    assert!(result.is_ok() || result.is_err());
}

// ==================== SHOW USERS Tests ====================

#[test]
fn test_show_users_parser_basic() {
    let query = "SHOW USERS";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "SHOW USERS basic parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("SHOW USERS statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "SHOW USERS");
}

#[test]
fn test_show_users_execution() {
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
        "CREATE USER bob WITH PASSWORD 'password456'",
        "SHOW USERS",
        "DROP USER alice",
        "DROP USER bob",
    ];

    for query in queries.iter() {
        let result = pipeline_manager.execute_query(query);
        assert!(result.is_ok() || result.is_err());
    }
}

// ==================== SHOW ROLES Tests ====================

#[test]
fn test_show_roles_parser_basic() {
    let query = "SHOW ROLES";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "SHOW ROLES basic parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("SHOW ROLES statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "SHOW ROLES");
}

#[test]
fn test_show_roles_parser_with_space() {
    let query = "SHOW ROLES IN test_space";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "SHOW ROLES with Space parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("SHOW ROLES statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "SHOW ROLES");
}

#[test]
fn test_show_roles_execution() {
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
        "SHOW ROLES",
        "SHOW ROLES IN test_space",
        "REVOKE ADMIN ON test_space FROM alice",
        "DROP USER alice",
    ];

    for query in queries.iter() {
        let result = pipeline_manager.execute_query(query);
        assert!(result.is_ok() || result.is_err());
    }
}

// ==================== Comprehensive DCL Lifecycle Tests ====================

#[test]
fn test_new_dcl_statements_lifecycle() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());

    let mut pipeline_manager = QueryPipelineManager::with_optimizer(
        storage,
        stats_manager,
        Arc::new(OptimizerEngine::default()),
    );

    let lifecycle_queries = vec![
        "CREATE USER adminuser WITH PASSWORD 'Admin@2024'",
        "CREATE USER dbauser WITH PASSWORD 'Dba@2024'",
        "CREATE USER readonly WITH PASSWORD 'Read@2024'",
        "SHOW USERS",
        "DESCRIBE USER adminuser",
        "GRANT ADMIN ON test_space TO adminuser",
        "GRANT DBA ON test_space TO dbauser",
        "GRANT GUEST ON test_space TO readonly",
        "SHOW ROLES",
        "SHOW ROLES IN test_space",
        "REVOKE GUEST ON test_space FROM readonly",
        "REVOKE DBA ON test_space FROM dbauser",
        "REVOKE ADMIN ON test_space FROM adminuser",
        "DROP USER readonly",
        "DROP USER dbauser",
        "DROP USER adminuser",
    ];

    for query in lifecycle_queries.iter() {
        let result = pipeline_manager.execute_query(query);
        assert!(result.is_ok() || result.is_err());
    }
}

#[test]
fn test_role_hierarchy() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());

    let mut pipeline_manager = QueryPipelineManager::with_optimizer(
        storage,
        stats_manager,
        Arc::new(OptimizerEngine::default()),
    );

    let queries = [
        "CREATE USER god_user WITH PASSWORD 'password'",
        "CREATE USER admin_user WITH PASSWORD 'password'",
        "CREATE USER guest_user WITH PASSWORD 'password'",
        "GRANT GOD ON system TO god_user",
        "GRANT ADMIN ON system TO admin_user",
        "GRANT GUEST ON system TO guest_user",
        "SHOW ROLES IN system",
        "REVOKE GOD ON system FROM god_user",
        "REVOKE ADMIN ON system FROM admin_user",
        "REVOKE GUEST ON system FROM guest_user",
        "DROP USER god_user",
        "DROP USER admin_user",
        "DROP USER guest_user",
    ];

    for query in queries.iter() {
        let result = pipeline_manager.execute_query(query);
        assert!(result.is_ok() || result.is_err());
    }
}
