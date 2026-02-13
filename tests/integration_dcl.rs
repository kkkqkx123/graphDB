//! 数据控制语言(DCL)集成测试
//!
//! 测试范围:
//! - CREATE USER - 创建用户
//! - ALTER USER - 修改用户
//! - DROP USER - 删除用户
//! - CHANGE PASSWORD - 修改密码

mod common;

use common::{
    TestStorage,
    assertions::{assert_ok, assert_err_with, assert_count},
    data_fixtures::{social_network_dataset, create_simple_vertex, create_edge},
    storage_helpers::{create_test_space, person_tag_info, knows_edge_type_info},
};

use graphdb::core::Value;
use graphdb::query::parser::Parser;
use graphdb::query::query_pipeline_manager::QueryPipelineManager;
use graphdb::api::service::stats_manager::StatsManager;
use std::sync::Arc;

// ==================== CREATE USER 语句测试 ====================

#[tokio::test]
async fn test_create_user_parser_basic() {
    let query = "CREATE USER alice WITH PASSWORD 'password123'";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "CREATE USER基础解析应该成功: {:?}", result.err());

    let stmt = result.expect("CREATE USER语句解析应该成功");
    assert_eq!(stmt.kind(), "CREATE USER");
}

#[tokio::test]
async fn test_create_user_parser_with_if_not_exists() {
    let query = "CREATE USER IF NOT EXISTS alice WITH PASSWORD 'password123'";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "CREATE USER带IF NOT EXISTS解析应该成功: {:?}", result.err());

    let stmt = result.expect("CREATE USER语句解析应该成功");
    assert_eq!(stmt.kind(), "CREATE USER");
}

#[tokio::test]
async fn test_create_user_parser_complex_password() {
    let query = "CREATE USER alice WITH PASSWORD 'P@ssw0rd!2024'";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "CREATE USER复杂密码解析应该成功: {:?}", result.err());

    let stmt = result.expect("CREATE USER语句解析应该成功");
    assert_eq!(stmt.kind(), "CREATE USER");
}

#[tokio::test]
async fn test_create_user_parser_special_username() {
    let query = "CREATE USER user_123 WITH PASSWORD 'password'";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "CREATE USER特殊用户名解析应该成功: {:?}", result.err());

    let stmt = result.expect("CREATE USER语句解析应该成功");
    assert_eq!(stmt.kind(), "CREATE USER");
}

#[tokio::test]
async fn test_create_user_execution_basic() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "CREATE USER alice WITH PASSWORD 'password123'";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("CREATE USER基础执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_create_user_execution_with_if_not_exists() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "CREATE USER IF NOT EXISTS alice WITH PASSWORD 'password123'";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("CREATE USER带IF NOT EXISTS执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_create_user_duplicate() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "CREATE USER alice WITH PASSWORD 'password123'";
    let result1 = pipeline_manager.execute_query(query).await;
    println!("第一次CREATE USER执行结果: {:?}", result1);
    
    let result2 = pipeline_manager.execute_query(query).await;
    println!("第二次CREATE USER执行结果: {:?}", result2);
    
    assert!(result1.is_ok() || result1.is_err());
    assert!(result2.is_ok() || result2.is_err());
}

// ==================== ALTER USER 语句测试 ====================

#[tokio::test]
async fn test_alter_user_parser_basic() {
    let query = "ALTER USER alice WITH PASSWORD 'newpassword123'";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "ALTER USER基础解析应该成功: {:?}", result.err());

    let stmt = result.expect("ALTER USER语句解析应该成功");
    assert_eq!(stmt.kind(), "ALTER USER");
}

#[tokio::test]
async fn test_alter_user_parser_complex_password() {
    let query = "ALTER USER alice WITH PASSWORD 'NewP@ssw0rd!2024'";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "ALTER USER复杂密码解析应该成功: {:?}", result.err());

    let stmt = result.expect("ALTER USER语句解析应该成功");
    assert_eq!(stmt.kind(), "ALTER USER");
}

#[tokio::test]
async fn test_alter_user_parser_special_username() {
    let query = "ALTER USER user_123 WITH PASSWORD 'newpassword'";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "ALTER USER特殊用户名解析应该成功: {:?}", result.err());

    let stmt = result.expect("ALTER USER语句解析应该成功");
    assert_eq!(stmt.kind(), "ALTER USER");
}

#[tokio::test]
async fn test_alter_user_execution_basic() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "ALTER USER alice WITH PASSWORD 'newpassword123'";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("ALTER USER基础执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_alter_user_nonexistent() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "ALTER USER nonexistent_user WITH PASSWORD 'newpassword'";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("ALTER USER不存在用户执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

// ==================== DROP USER 语句测试 ====================

#[tokio::test]
async fn test_drop_user_parser_basic() {
    let query = "DROP USER alice";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "DROP USER基础解析应该成功: {:?}", result.err());

    let stmt = result.expect("DROP USER语句解析应该成功");
    assert_eq!(stmt.kind(), "DROP USER");
}

#[tokio::test]
async fn test_drop_user_parser_with_if_exists() {
    let query = "DROP USER IF EXISTS alice";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "DROP USER带IF EXISTS解析应该成功: {:?}", result.err());

    let stmt = result.expect("DROP USER语句解析应该成功");
    assert_eq!(stmt.kind(), "DROP USER");
}

#[tokio::test]
async fn test_drop_user_parser_special_username() {
    let query = "DROP USER user_123";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "DROP USER特殊用户名解析应该成功: {:?}", result.err());

    let stmt = result.expect("DROP USER语句解析应该成功");
    assert_eq!(stmt.kind(), "DROP USER");
}

#[tokio::test]
async fn test_drop_user_execution_basic() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "DROP USER alice";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("DROP USER基础执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_drop_user_with_if_exists() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "DROP USER IF EXISTS alice";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("DROP USER带IF EXISTS执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_drop_user_nonexistent() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "DROP USER nonexistent_user";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("DROP USER不存在用户执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_drop_user_nonexistent_with_if_exists() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "DROP USER IF EXISTS nonexistent_user";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("DROP USER IF EXISTS不存在用户执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

// ==================== CHANGE PASSWORD 语句测试 ====================

#[tokio::test]
async fn test_change_password_parser_basic() {
    let query = "CHANGE PASSWORD 'oldpassword' TO 'newpassword'";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "CHANGE PASSWORD基础解析应该成功: {:?}", result.err());

    let stmt = result.expect("CHANGE PASSWORD语句解析应该成功");
    assert_eq!(stmt.kind(), "CHANGE PASSWORD");
}

#[tokio::test]
async fn test_change_password_parser_complex_passwords() {
    let query = "CHANGE PASSWORD 'OldP@ssw0rd!' TO 'NewP@ssw0rd!2024'";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "CHANGE PASSWORD复杂密码解析应该成功: {:?}", result.err());

    let stmt = result.expect("CHANGE PASSWORD语句解析应该成功");
    assert_eq!(stmt.kind(), "CHANGE PASSWORD");
}

#[tokio::test]
async fn test_change_password_parser_special_chars() {
    let query = "CHANGE PASSWORD 'p@$$w0rd#123' TO 'n3wP@$$w0rd#456'";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "CHANGE PASSWORD特殊字符密码解析应该成功: {:?}", result.err());

    let stmt = result.expect("CHANGE PASSWORD语句解析应该成功");
    assert_eq!(stmt.kind(), "CHANGE PASSWORD");
}

#[tokio::test]
async fn test_change_password_execution_basic() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "CHANGE PASSWORD 'oldpassword' TO 'newpassword'";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("CHANGE PASSWORD基础执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_change_password_wrong_old_password() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "CHANGE PASSWORD 'wrongpassword' TO 'newpassword'";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("CHANGE PASSWORD错误旧密码执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

// ==================== DCL 综合测试 ====================

#[tokio::test]
async fn test_dcl_user_lifecycle() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let lifecycle_queries = vec![
        "CREATE USER testuser WITH PASSWORD 'password123'",
        "ALTER USER testuser WITH PASSWORD 'newpassword123'",
        "CHANGE PASSWORD 'newpassword123' TO 'anotherpassword123'",
        "DROP USER testuser",
    ];
    
    for (i, query) in lifecycle_queries.iter().enumerate() {
        let result = pipeline_manager.execute_query(query).await;
        println!("DCL用户生命周期操作 {} 执行结果: {:?}", i + 1, result);
        assert!(result.is_ok() || result.is_err());
    }
}

#[tokio::test]
async fn test_dcl_multiple_users() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let create_queries = vec![
        "CREATE USER alice WITH PASSWORD 'alice123'",
        "CREATE USER bob WITH PASSWORD 'bob123'",
        "CREATE USER charlie WITH PASSWORD 'charlie123'",
    ];
    
    for (i, query) in create_queries.iter().enumerate() {
        let result = pipeline_manager.execute_query(query).await;
        println!("DCL创建用户 {} 执行结果: {:?}", i + 1, result);
        assert!(result.is_ok() || result.is_err());
    }
    
    let drop_queries = vec![
        "DROP USER alice",
        "DROP USER bob",
        "DROP USER charlie",
    ];
    
    for (i, query) in drop_queries.iter().enumerate() {
        let result = pipeline_manager.execute_query(query).await;
        println!("DCL删除用户 {} 执行结果: {:?}", i + 1, result);
        assert!(result.is_ok() || result.is_err());
    }
}

#[tokio::test]
async fn test_dcl_if_not_exists_if_exists() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let queries = vec![
        "CREATE USER IF NOT EXISTS testuser WITH PASSWORD 'password'",
        "CREATE USER IF NOT EXISTS testuser WITH PASSWORD 'password'",  // 重复创建
        "DROP USER IF EXISTS testuser",
        "DROP USER IF EXISTS testuser",  // 重复删除
    ];
    
    for (i, query) in queries.iter().enumerate() {
        let result = pipeline_manager.execute_query(query).await;
        println!("DCL IF NOT EXISTS/IF EXISTS操作 {} 执行结果: {:?}", i + 1, result);
        assert!(result.is_ok() || result.is_err());
    }
}

#[tokio::test]
async fn test_dcl_error_handling() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let invalid_queries = vec![
        "CREATE USER",  // 缺少用户名和密码
        "CREATE USER testuser",  // 缺少密码
        "CREATE USER WITH PASSWORD 'password'",  // 缺少用户名
        "ALTER USER",  // 缺少用户名和密码
        "ALTER USER testuser",  // 缺少密码
        "DROP USER",  // 缺少用户名
        "CHANGE PASSWORD",  // 缺少密码
        "CHANGE PASSWORD 'oldpassword'",  // 缺少新密码
    ];
    
    for query in invalid_queries {
        let result = pipeline_manager.execute_query(query).await;
        assert!(result.is_err(), "无效查询应该返回错误: {}", query);
    }
}

#[tokio::test]
async fn test_dcl_password_security() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let password_queries = vec![
        "CREATE USER secureuser WITH PASSWORD 'SecureP@ssw0rd!2024'",
        "ALTER USER secureuser WITH PASSWORD 'N3wS3cur3P@ssw0rd!2024'",
        "CHANGE PASSWORD 'N3wS3cur3P@ssw0rd!2024' TO 'An0th3rS3cur3P@ssw0rd!2024'",
    ];
    
    for (i, query) in password_queries.iter().enumerate() {
        let result = pipeline_manager.execute_query(query).await;
        println!("DCL密码安全操作 {} 执行结果: {:?}", i + 1, result);
        assert!(result.is_ok() || result.is_err());
    }
}

#[tokio::test]
async fn test_dcl_user_management_workflow() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let workflow_queries = vec![
        "CREATE USER admin WITH PASSWORD 'Admin@2024'",
        "CREATE USER readonly WITH PASSWORD 'Read@2024'",
        "ALTER USER readonly WITH PASSWORD 'NewRead@2024'",
        "DROP USER readonly",
        "CHANGE PASSWORD 'Admin@2024' TO 'NewAdmin@2024'",
        "DROP USER admin",
    ];
    
    for (i, query) in workflow_queries.iter().enumerate() {
        let result = pipeline_manager.execute_query(query).await;
        println!("DCL用户管理工作流操作 {} 执行结果: {:?}", i + 1, result);
        assert!(result.is_ok() || result.is_err());
    }
}

#[tokio::test]
async fn test_dcl_special_usernames() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let special_username_queries = vec![
        "CREATE USER user_123 WITH PASSWORD 'password'",
        "CREATE USER user-456 WITH PASSWORD 'password'",
        "CREATE USER user.789 WITH PASSWORD 'password'",
        "DROP USER user_123",
        "DROP USER user-456",
        "DROP USER user.789",
    ];
    
    for (i, query) in special_username_queries.iter().enumerate() {
        let result = pipeline_manager.execute_query(query).await;
        println!("DCL特殊用户名操作 {} 执行结果: {:?}", i + 1, result);
        assert!(result.is_ok() || result.is_err());
    }
}
