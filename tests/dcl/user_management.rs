//! DCL User Management Tests
//!
//! Test coverage:
//! - CREATE USER - Create a user
//! - ALTER USER - Modifies a user account
//! - DROP USER - Deletes a user
//! - CHANGE PASSWORD - Change your password

use super::common;

use common::TestStorage;

use graphdb::core::stats::StatsManager;
use graphdb::query::optimizer::OptimizerEngine;
use graphdb::query::parser::Parser;
use graphdb::query::query_pipeline_manager::QueryPipelineManager;
use std::sync::Arc;

// ==================== CREATE USER Parser Tests ====================

#[test]
fn test_create_user_parser_basic() {
    let query = "CREATE USER alice WITH PASSWORD 'password123'";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "CREATE USER basic parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("CREATE USER statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "CREATE USER");
}

#[test]
fn test_create_user_parser_with_if_not_exists() {
    let query = "CREATE USER IF NOT EXISTS alice WITH PASSWORD 'password123'";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "CREATE USER with IF NOT EXISTS parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("CREATE USER statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "CREATE USER");
}

#[test]
fn test_create_user_parser_complex_password() {
    let query = "CREATE USER alice WITH PASSWORD 'P@ssw0rd!2024'";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "CREATE USER complex password parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("CREATE USER statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "CREATE USER");
}

#[test]
fn test_create_user_parser_special_username() {
    let query = "CREATE USER user_123 WITH PASSWORD 'password'";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "CREATE USER special username parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("CREATE USER statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "CREATE USER");
}

// ==================== CREATE USER Execution Tests ====================

#[test]
fn test_create_user_execution_basic() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());

    let mut pipeline_manager = QueryPipelineManager::with_optimizer(
        storage,
        stats_manager,
        Arc::new(OptimizerEngine::default()),
    );

    let query = "CREATE USER alice WITH PASSWORD 'password123'";
    let result = pipeline_manager.execute_query(query);

    assert!(
        result.is_ok(),
        "CREATE USER should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_create_user_execution_with_if_not_exists() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());

    let mut pipeline_manager = QueryPipelineManager::with_optimizer(
        storage,
        stats_manager,
        Arc::new(OptimizerEngine::default()),
    );

    let query = "CREATE USER IF NOT EXISTS alice WITH PASSWORD 'password123'";
    let result = pipeline_manager.execute_query(query);

    assert!(
        result.is_ok(),
        "CREATE USER IF NOT EXISTS should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_create_user_duplicate() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());

    let mut pipeline_manager = QueryPipelineManager::with_optimizer(
        storage,
        stats_manager,
        Arc::new(OptimizerEngine::default()),
    );

    let query = "CREATE USER alice WITH PASSWORD 'password123'";
    let result1 = pipeline_manager.execute_query(query);
    assert!(
        result1.is_ok(),
        "First CREATE USER should succeed: {:?}",
        result1.err()
    );

    let result2 = pipeline_manager.execute_query(query);
    assert!(result2.is_err(), "Duplicate CREATE USER should fail");
}

// ==================== ALTER USER Parser Tests ====================

#[test]
fn test_alter_user_parser_basic() {
    let query = "ALTER USER alice WITH PASSWORD 'newpassword123'";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "ALTER USER basic parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("ALTER USER statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "ALTER USER");
}

#[test]
fn test_alter_user_parser_complex_password() {
    let query = "ALTER USER alice WITH PASSWORD 'NewP@ssw0rd!2024'";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "ALTER USER complex password parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("ALTER USER statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "ALTER USER");
}

#[test]
fn test_alter_user_parser_special_username() {
    let query = "ALTER USER user_123 WITH PASSWORD 'newpassword'";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "ALTER USER special username parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("ALTER USER statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "ALTER USER");
}

// ==================== ALTER USER Execution Tests ====================

#[test]
fn test_alter_user_execution_basic() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());

    let mut pipeline_manager = QueryPipelineManager::with_optimizer(
        storage,
        stats_manager,
        Arc::new(OptimizerEngine::default()),
    );

    pipeline_manager
        .execute_query("CREATE USER alice WITH PASSWORD 'password123'")
        .expect("CREATE USER should succeed before ALTER");

    let query = "ALTER USER alice WITH PASSWORD 'newpassword123'";
    let result = pipeline_manager.execute_query(query);

    assert!(
        result.is_ok(),
        "ALTER USER on existing user should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_alter_user_nonexistent() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());

    let mut pipeline_manager = QueryPipelineManager::with_optimizer(
        storage,
        stats_manager,
        Arc::new(OptimizerEngine::default()),
    );

    let query = "ALTER USER nonexistent_user WITH PASSWORD 'newpassword'";
    let result = pipeline_manager.execute_query(query);

    assert!(
        result.is_err(),
        "ALTER USER on nonexistent user should fail"
    );
}

// ==================== DROP USER Parser Tests ====================

#[test]
fn test_drop_user_parser_basic() {
    let query = "DROP USER alice";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "DROP USER basic parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("DROP USER statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "DROP USER");
}

#[test]
fn test_drop_user_parser_with_if_exists() {
    let query = "DROP USER IF EXISTS alice";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "DROP USER with IF EXISTS parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("DROP USER statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "DROP USER");
}

#[test]
fn test_drop_user_parser_special_username() {
    let query = "DROP USER user_123";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "DROP USER special username parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("DROP USER statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "DROP USER");
}

// ==================== DROP USER Execution Tests ====================

#[test]
fn test_drop_user_execution_basic() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());

    let mut pipeline_manager = QueryPipelineManager::with_optimizer(
        storage,
        stats_manager,
        Arc::new(OptimizerEngine::default()),
    );

    pipeline_manager
        .execute_query("CREATE USER alice WITH PASSWORD 'password123'")
        .expect("CREATE USER should succeed before DROP");

    let query = "DROP USER alice";
    let result = pipeline_manager.execute_query(query);

    assert!(
        result.is_ok(),
        "DROP USER on existing user should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_drop_user_with_if_exists() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());

    let mut pipeline_manager = QueryPipelineManager::with_optimizer(
        storage,
        stats_manager,
        Arc::new(OptimizerEngine::default()),
    );

    pipeline_manager
        .execute_query("CREATE USER alice WITH PASSWORD 'password123'")
        .expect("CREATE USER should succeed before DROP IF EXISTS");

    let query = "DROP USER IF EXISTS alice";
    let result = pipeline_manager.execute_query(query);

    assert!(
        result.is_ok(),
        "DROP USER IF EXISTS on existing user should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_drop_user_nonexistent() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());

    let mut pipeline_manager = QueryPipelineManager::with_optimizer(
        storage,
        stats_manager,
        Arc::new(OptimizerEngine::default()),
    );

    let query = "DROP USER nonexistent_user";
    let result = pipeline_manager.execute_query(query);

    assert!(result.is_err(), "DROP USER on nonexistent user should fail");
}

#[test]
fn test_drop_user_nonexistent_with_if_exists() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());

    let mut pipeline_manager = QueryPipelineManager::with_optimizer(
        storage,
        stats_manager,
        Arc::new(OptimizerEngine::default()),
    );

    let query = "DROP USER IF EXISTS nonexistent_user";
    let result = pipeline_manager.execute_query(query);

    assert!(
        result.is_ok(),
        "DROP USER IF EXISTS on nonexistent user should succeed: {:?}",
        result.err()
    );
}

// ==================== CHANGE PASSWORD Tests ====================

#[test]
fn test_change_password_parser_basic() {
    let query = "CHANGE PASSWORD 'oldpassword' TO 'newpassword'";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "CHANGE PASSWORD basic parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("CHANGE PASSWORD statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "CHANGE PASSWORD");
}

#[test]
fn test_change_password_parser_complex_passwords() {
    let query = "CHANGE PASSWORD 'OldP@ssw0rd!' TO 'NewP@ssw0rd!2024'";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "CHANGE PASSWORD complex password parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("CHANGE PASSWORD statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "CHANGE PASSWORD");
}

#[test]
fn test_change_password_parser_special_chars() {
    let query = "CHANGE PASSWORD 'p@$$w0rd#123' TO 'n3wP@$$w0rd#456'";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    assert!(
        result.is_ok(),
        "CHANGE PASSWORD special char password parsing should succeed: {:?}",
        result.err()
    );

    let stmt = result.expect("CHANGE PASSWORD statement parsing should succeed");
    assert_eq!(stmt.ast.stmt.kind(), "CHANGE PASSWORD");
}

#[test]
fn test_change_password_execution_basic() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());

    let mut pipeline_manager = QueryPipelineManager::with_optimizer(
        storage,
        stats_manager,
        Arc::new(OptimizerEngine::default()),
    );

    pipeline_manager
        .execute_query("CREATE USER alice WITH PASSWORD 'oldpassword'")
        .expect("CREATE USER should succeed before CHANGE PASSWORD");

    let query = "CHANGE PASSWORD alice 'oldpassword' TO 'newpassword'";
    let result = pipeline_manager.execute_query(query);

    assert!(
        result.is_ok(),
        "CHANGE PASSWORD should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_change_password_wrong_old_password() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());

    let mut pipeline_manager = QueryPipelineManager::with_optimizer(
        storage,
        stats_manager,
        Arc::new(OptimizerEngine::default()),
    );

    pipeline_manager
        .execute_query("CREATE USER alice WITH PASSWORD 'oldpassword'")
        .expect("CREATE USER should succeed before CHANGE PASSWORD");

    let query = "CHANGE PASSWORD 'wrongpassword' TO 'newpassword'";
    let result = pipeline_manager.execute_query(query);

    assert!(
        result.is_err(),
        "CHANGE PASSWORD with wrong old password should fail"
    );
}

// ==================== User Lifecycle Tests ====================

#[test]
fn test_dcl_user_lifecycle() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());

    let mut pipeline_manager = QueryPipelineManager::with_optimizer(
        storage,
        stats_manager,
        Arc::new(OptimizerEngine::default()),
    );

    let lifecycle_queries = [
        "CREATE USER testuser WITH PASSWORD 'password123'",
        "CHANGE PASSWORD testuser 'password123' TO 'newpassword123'",
        "CHANGE PASSWORD testuser 'newpassword123' TO 'anotherpassword123'",
        "DROP USER testuser",
    ];

    for query in lifecycle_queries.iter() {
        let result = pipeline_manager.execute_query(query);
        assert!(
            result.is_ok(),
            "Lifecycle query '{}' should succeed: {:?}",
            query,
            result.err()
        );
    }
}

#[test]
fn test_dcl_multiple_users() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());

    let mut pipeline_manager = QueryPipelineManager::with_optimizer(
        storage,
        stats_manager,
        Arc::new(OptimizerEngine::default()),
    );

    let create_queries = [
        "CREATE USER alice WITH PASSWORD 'alice123'",
        "CREATE USER bob WITH PASSWORD 'bob123'",
        "CREATE USER charlie WITH PASSWORD 'charlie123'",
    ];

    for query in create_queries.iter() {
        let result = pipeline_manager.execute_query(query);
        assert!(
            result.is_ok(),
            "CREATE USER '{}' should succeed: {:?}",
            query,
            result.err()
        );
    }

    let drop_queries = ["DROP USER alice", "DROP USER bob", "DROP USER charlie"];

    for query in drop_queries.iter() {
        let result = pipeline_manager.execute_query(query);
        assert!(
            result.is_ok(),
            "DROP USER '{}' should succeed: {:?}",
            query,
            result.err()
        );
    }
}

#[test]
fn test_dcl_if_not_exists_if_exists() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());

    let mut pipeline_manager = QueryPipelineManager::with_optimizer(
        storage,
        stats_manager,
        Arc::new(OptimizerEngine::default()),
    );

    let queries = [
        "CREATE USER IF NOT EXISTS testuser WITH PASSWORD 'password'",
        "CREATE USER IF NOT EXISTS testuser WITH PASSWORD 'password'",
        "DROP USER IF EXISTS testuser",
        "DROP USER IF EXISTS testuser",
    ];

    for query in queries.iter() {
        let result = pipeline_manager.execute_query(query);
        assert!(
            result.is_ok(),
            "IF NOT EXISTS/IF EXISTS query '{}' should succeed: {:?}",
            query,
            result.err()
        );
    }
}

#[test]
fn test_dcl_error_handling() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());

    let mut pipeline_manager = QueryPipelineManager::with_optimizer(
        storage,
        stats_manager,
        Arc::new(OptimizerEngine::default()),
    );

    let invalid_queries = vec![
        "CREATE USER",
        "CREATE USER testuser",
        "CREATE USER WITH PASSWORD 'password'",
        "ALTER USER",
        "ALTER USER testuser",
        "DROP USER",
        "CHANGE PASSWORD",
        "CHANGE PASSWORD 'oldpassword'",
    ];

    for query in invalid_queries {
        let result = pipeline_manager.execute_query(query);
        assert!(
            result.is_err(),
            "Invalid query should return error: {}",
            query
        );
    }
}

#[test]
fn test_dcl_password_security() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());

    let mut pipeline_manager = QueryPipelineManager::with_optimizer(
        storage,
        stats_manager,
        Arc::new(OptimizerEngine::default()),
    );

    let password_queries = [
        "CREATE USER secureuser WITH PASSWORD 'SecureP@ssw0rd!2024'",
        "CHANGE PASSWORD secureuser 'SecureP@ssw0rd!2024' TO 'N3wS3cur3P@ssw0rd!2024'",
        "CHANGE PASSWORD secureuser 'N3wS3cur3P@ssw0rd!2024' TO 'An0th3rS3cur3P@ssw0rd!2024'",
    ];

    // First create the user
    let create_result = pipeline_manager.execute_query(password_queries[0]);
    assert!(
        create_result.is_ok(),
        "CREATE USER should succeed: {:?}",
        create_result.err()
    );

    // Then change password (first time)
    let change_result = pipeline_manager.execute_query(password_queries[1]);
    assert!(
        change_result.is_ok(),
        "CHANGE PASSWORD should succeed: {:?}",
        change_result.err()
    );

    // Then change password again
    let change_result = pipeline_manager.execute_query(password_queries[2]);
    assert!(
        change_result.is_ok(),
        "CHANGE PASSWORD should succeed: {:?}",
        change_result.err()
    );
}

#[test]
fn test_dcl_user_management_workflow() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());

    let mut pipeline_manager = QueryPipelineManager::with_optimizer(
        storage,
        stats_manager,
        Arc::new(OptimizerEngine::default()),
    );

    let workflow_queries = [
        "CREATE USER admin_user WITH PASSWORD 'Admin@2024'",
        "CREATE USER readonly_user WITH PASSWORD 'Read@2024'",
        "CHANGE PASSWORD readonly_user 'Read@2024' TO 'NewRead@2024'",
        "DROP USER readonly_user",
        "CHANGE PASSWORD admin_user 'Admin@2024' TO 'NewAdmin@2024'",
        "DROP USER admin_user",
    ];

    for query in workflow_queries.iter() {
        let result = pipeline_manager.execute_query(query);
        assert!(
            result.is_ok(),
            "Workflow query '{}' should succeed: {:?}",
            query,
            result.err()
        );
    }
}

#[test]
fn test_dcl_special_usernames() {
    let test_storage = TestStorage::new().expect("Failed to create test storage");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());

    let mut pipeline_manager = QueryPipelineManager::with_optimizer(
        storage,
        stats_manager,
        Arc::new(OptimizerEngine::default()),
    );

let special_username_queries = [
            "CREATE USER user_123 WITH PASSWORD 'password'",
            "CREATE USER user_456 WITH PASSWORD 'password'",
            "CREATE USER user_789 WITH PASSWORD 'password'",
            "DROP USER user_123",
            "DROP USER user_456",
            "DROP USER user_789",
        ];

    for query in special_username_queries.iter() {
        let result = pipeline_manager.execute_query(query);
        assert!(
            result.is_ok(),
            "Query with special username '{}' should succeed: {:?}",
            query,
            result.err()
        );
    }
}
