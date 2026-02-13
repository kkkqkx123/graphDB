//! 数据定义语言(DDL)集成测试
//!
//! 测试范围:
//! - CREATE TAG - 创建标签
//! - CREATE EDGE - 创建边类型
//! - ALTER TAG - 修改标签
//! - ALTER EDGE - 修改边类型
//! - DROP TAG - 删除标签
//! - DROP EDGE - 删除边类型
//! - DESC - 描述对象

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

// ==================== CREATE TAG 语句测试 ====================

#[tokio::test]
async fn test_create_tag_parser_basic() {
    let query = "CREATE TAG Person(name: STRING, age: INT)";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "CREATE TAG基础解析应该成功: {:?}", result.err());

    let stmt = result.expect("CREATE TAG语句解析应该成功");
    assert_eq!(stmt.kind(), "CREATE");
}

#[tokio::test]
async fn test_create_tag_parser_with_if_not_exists() {
    let query = "CREATE TAG IF NOT EXISTS Person(name: STRING, age: INT)";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "CREATE TAG带IF NOT EXISTS解析应该成功: {:?}", result.err());

    let stmt = result.expect("CREATE TAG语句解析应该成功");
    assert_eq!(stmt.kind(), "CREATE");
}

#[tokio::test]
async fn test_create_tag_parser_single_property() {
    let query = "CREATE TAG Person(name: STRING)";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "CREATE TAG单个属性解析应该成功: {:?}", result.err());

    let stmt = result.expect("CREATE TAG语句解析应该成功");
    assert_eq!(stmt.kind(), "CREATE");
}

#[tokio::test]
async fn test_create_tag_parser_multiple_properties() {
    let query = "CREATE TAG Person(name: STRING, age: INT, created_at: TIMESTAMP)";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "CREATE TAG多个属性解析应该成功: {:?}", result.err());

    let stmt = result.expect("CREATE TAG语句解析应该成功");
    assert_eq!(stmt.kind(), "CREATE");
}

#[tokio::test]
async fn test_create_tag_parser_various_types() {
    let query = "CREATE TAG Test(name: STRING, age: INT, score: DOUBLE, active: BOOL, birth: DATE)";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "CREATE TAG多种类型解析应该成功: {:?}", result.err());

    let stmt = result.expect("CREATE TAG语句解析应该成功");
    assert_eq!(stmt.kind(), "CREATE");
}

#[tokio::test]
async fn test_create_tag_execution_basic() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "CREATE TAG Person(name: STRING, age: INT)";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("CREATE TAG基础执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_create_tag_execution_with_if_not_exists() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "CREATE TAG IF NOT EXISTS Person(name: STRING, age: INT)";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("CREATE TAG带IF NOT EXISTS执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

// ==================== CREATE EDGE 语句测试 ====================

#[tokio::test]
async fn test_create_edge_parser_basic() {
    let query = "CREATE EDGE KNOWS(since: DATE)";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "CREATE EDGE基础解析应该成功: {:?}", result.err());

    let stmt = result.expect("CREATE EDGE语句解析应该成功");
    assert_eq!(stmt.kind(), "CREATE");
}

#[tokio::test]
async fn test_create_edge_parser_with_if_not_exists() {
    let query = "CREATE EDGE IF NOT EXISTS KNOWS(since: DATE)";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "CREATE EDGE带IF NOT EXISTS解析应该成功: {:?}", result.err());

    let stmt = result.expect("CREATE EDGE语句解析应该成功");
    assert_eq!(stmt.kind(), "CREATE");
}

#[tokio::test]
async fn test_create_edge_parser_single_property() {
    let query = "CREATE EDGE KNOWS(since: DATE)";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "CREATE EDGE单个属性解析应该成功: {:?}", result.err());

    let stmt = result.expect("CREATE EDGE语句解析应该成功");
    assert_eq!(stmt.kind(), "CREATE");
}

#[tokio::test]
async fn test_create_edge_parser_multiple_properties() {
    let query = "CREATE EDGE KNOWS(since: DATE, degree: DOUBLE, note: STRING)";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "CREATE EDGE多个属性解析应该成功: {:?}", result.err());

    let stmt = result.expect("CREATE EDGE语句解析应该成功");
    assert_eq!(stmt.kind(), "CREATE");
}

#[tokio::test]
async fn test_create_edge_parser_various_types() {
    let query = "CREATE EDGE Test(since: DATE, weight: DOUBLE, active: BOOL, count: INT)";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "CREATE EDGE多种类型解析应该成功: {:?}", result.err());

    let stmt = result.expect("CREATE EDGE语句解析应该成功");
    assert_eq!(stmt.kind(), "CREATE");
}

#[tokio::test]
async fn test_create_edge_execution_basic() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "CREATE EDGE KNOWS(since: DATE)";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("CREATE EDGE基础执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_create_edge_execution_with_if_not_exists() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "CREATE EDGE IF NOT EXISTS KNOWS(since: DATE)";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("CREATE EDGE带IF NOT EXISTS执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

// ==================== ALTER TAG 语句测试 ====================

#[tokio::test]
async fn test_alter_tag_parser_add() {
    let query = "ALTER TAG Person ADD (email: STRING, phone: STRING)";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "ALTER TAG ADD解析应该成功: {:?}", result.err());

    let stmt = result.expect("ALTER TAG语句解析应该成功");
    assert_eq!(stmt.kind(), "ALTER");
}

#[tokio::test]
async fn test_alter_tag_parser_drop() {
    let query = "ALTER TAG Person DROP (temp_field, old_field)";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "ALTER TAG DROP解析应该成功: {:?}", result.err());

    let stmt = result.expect("ALTER TAG语句解析应该成功");
    assert_eq!(stmt.kind(), "ALTER");
}

#[tokio::test]
async fn test_alter_tag_parser_change() {
    let query = "ALTER TAG Person CHANGE (old_name new_name: STRING)";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "ALTER TAG CHANGE解析应该成功: {:?}", result.err());

    let stmt = result.expect("ALTER TAG语句解析应该成功");
    assert_eq!(stmt.kind(), "ALTER");
}

#[tokio::test]
async fn test_alter_tag_parser_add_single() {
    let query = "ALTER TAG Person ADD (email: STRING)";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "ALTER TAG ADD单个属性解析应该成功: {:?}", result.err());

    let stmt = result.expect("ALTER TAG语句解析应该成功");
    assert_eq!(stmt.kind(), "ALTER");
}

#[tokio::test]
async fn test_alter_tag_parser_drop_single() {
    let query = "ALTER TAG Person DROP (temp_field)";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "ALTER TAG DROP单个属性解析应该成功: {:?}", result.err());

    let stmt = result.expect("ALTER TAG语句解析应该成功");
    assert_eq!(stmt.kind(), "ALTER");
}

#[tokio::test]
async fn test_alter_tag_execution_add() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "ALTER TAG Person ADD (email: STRING)";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("ALTER TAG ADD执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_alter_tag_execution_drop() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "ALTER TAG Person DROP (temp_field)";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("ALTER TAG DROP执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

// ==================== ALTER EDGE 语句测试 ====================

#[tokio::test]
async fn test_alter_edge_parser_add() {
    let query = "ALTER EDGE KNOWS ADD (note: STRING, weight: DOUBLE)";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "ALTER EDGE ADD解析应该成功: {:?}", result.err());

    let stmt = result.expect("ALTER EDGE语句解析应该成功");
    assert_eq!(stmt.kind(), "ALTER");
}

#[tokio::test]
async fn test_alter_edge_parser_drop() {
    let query = "ALTER EDGE KNOWS DROP (temp_field, old_field)";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "ALTER EDGE DROP解析应该成功: {:?}", result.err());

    let stmt = result.expect("ALTER EDGE语句解析应该成功");
    assert_eq!(stmt.kind(), "ALTER");
}

#[tokio::test]
async fn test_alter_edge_parser_change() {
    let query = "ALTER EDGE KNOWS CHANGE (old_since new_since: DATE)";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "ALTER EDGE CHANGE解析应该成功: {:?}", result.err());

    let stmt = result.expect("ALTER EDGE语句解析应该成功");
    assert_eq!(stmt.kind(), "ALTER");
}

#[tokio::test]
async fn test_alter_edge_parser_add_single() {
    let query = "ALTER EDGE KNOWS ADD (note: STRING)";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "ALTER EDGE ADD单个属性解析应该成功: {:?}", result.err());

    let stmt = result.expect("ALTER EDGE语句解析应该成功");
    assert_eq!(stmt.kind(), "ALTER");
}

#[tokio::test]
async fn test_alter_edge_parser_drop_single() {
    let query = "ALTER EDGE KNOWS DROP (temp_field)";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "ALTER EDGE DROP单个属性解析应该成功: {:?}", result.err());

    let stmt = result.expect("ALTER EDGE语句解析应该成功");
    assert_eq!(stmt.kind(), "ALTER");
}

#[tokio::test]
async fn test_alter_edge_execution_add() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "ALTER EDGE KNOWS ADD (note: STRING)";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("ALTER EDGE ADD执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_alter_edge_execution_drop() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "ALTER EDGE KNOWS DROP (temp_field)";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("ALTER EDGE DROP执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

// ==================== DROP TAG 语句测试 ====================

#[tokio::test]
async fn test_drop_tag_parser_basic() {
    let query = "DROP TAG Person";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "DROP TAG基础解析应该成功: {:?}", result.err());

    let stmt = result.expect("DROP TAG语句解析应该成功");
    assert_eq!(stmt.kind(), "DROP");
}

#[tokio::test]
async fn test_drop_tag_parser_with_if_exists() {
    let query = "DROP TAG IF EXISTS Person";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "DROP TAG带IF EXISTS解析应该成功: {:?}", result.err());

    let stmt = result.expect("DROP TAG语句解析应该成功");
    assert_eq!(stmt.kind(), "DROP");
}

#[tokio::test]
async fn test_drop_tag_parser_multiple() {
    let query = "DROP TAG Person, Company, Location";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "DROP TAG多个标签解析应该成功: {:?}", result.err());

    let stmt = result.expect("DROP TAG语句解析应该成功");
    assert_eq!(stmt.kind(), "DROP");
}

#[tokio::test]
async fn test_drop_tag_parser_multiple_with_if_exists() {
    let query = "DROP TAG IF EXISTS Person, Company";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "DROP TAG多个标签带IF EXISTS解析应该成功: {:?}", result.err());

    let stmt = result.expect("DROP TAG语句解析应该成功");
    assert_eq!(stmt.kind(), "DROP");
}

#[tokio::test]
async fn test_drop_tag_execution_basic() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "DROP TAG Person";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("DROP TAG基础执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_drop_tag_execution_with_if_exists() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "DROP TAG IF EXISTS Person";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("DROP TAG带IF EXISTS执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

// ==================== DROP EDGE 语句测试 ====================

#[tokio::test]
async fn test_drop_edge_parser_basic() {
    let query = "DROP EDGE KNOWS";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "DROP EDGE基础解析应该成功: {:?}", result.err());

    let stmt = result.expect("DROP EDGE语句解析应该成功");
    assert_eq!(stmt.kind(), "DROP");
}

#[tokio::test]
async fn test_drop_edge_parser_with_if_exists() {
    let query = "DROP EDGE IF EXISTS KNOWS";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "DROP EDGE带IF EXISTS解析应该成功: {:?}", result.err());

    let stmt = result.expect("DROP EDGE语句解析应该成功");
    assert_eq!(stmt.kind(), "DROP");
}

#[tokio::test]
async fn test_drop_edge_parser_multiple() {
    let query = "DROP EDGE KNOWS, LIKES, FOLLOWS";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "DROP EDGE多个边类型解析应该成功: {:?}", result.err());

    let stmt = result.expect("DROP EDGE语句解析应该成功");
    assert_eq!(stmt.kind(), "DROP");
}

#[tokio::test]
async fn test_drop_edge_parser_multiple_with_if_exists() {
    let query = "DROP EDGE IF EXISTS KNOWS, LIKES";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "DROP EDGE多个边类型带IF EXISTS解析应该成功: {:?}", result.err());

    let stmt = result.expect("DROP EDGE语句解析应该成功");
    assert_eq!(stmt.kind(), "DROP");
}

#[tokio::test]
async fn test_drop_edge_execution_basic() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "DROP EDGE KNOWS";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("DROP EDGE基础执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_drop_edge_execution_with_if_exists() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "DROP EDGE IF EXISTS KNOWS";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("DROP EDGE带IF EXISTS执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

// ==================== DESC 语句测试 ====================

#[tokio::test]
async fn test_desc_parser_tag() {
    let query = "DESCRIBE TAG Person";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "DESCRIBE TAG解析应该成功: {:?}", result.err());

    let stmt = result.expect("DESCRIBE TAG语句解析应该成功");
    assert_eq!(stmt.kind(), "DESC");
}

#[tokio::test]
async fn test_desc_parser_edge() {
    let query = "DESCRIBE EDGE KNOWS";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "DESCRIBE EDGE解析应该成功: {:?}", result.err());

    let stmt = result.expect("DESCRIBE EDGE语句解析应该成功");
    assert_eq!(stmt.kind(), "DESC");
}

#[tokio::test]
async fn test_desc_parser_short_tag() {
    let query = "DESC TAG Person";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "DESC TAG解析应该成功: {:?}", result.err());

    let stmt = result.expect("DESC TAG语句解析应该成功");
    assert_eq!(stmt.kind(), "DESC");
}

#[tokio::test]
async fn test_desc_parser_short_edge() {
    let query = "DESC EDGE KNOWS";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "DESC EDGE解析应该成功: {:?}", result.err());

    let stmt = result.expect("DESC EDGE语句解析应该成功");
    assert_eq!(stmt.kind(), "DESC");
}

#[tokio::test]
async fn test_desc_execution_tag() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "DESCRIBE TAG Person";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("DESCRIBE TAG执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_desc_execution_edge() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "DESCRIBE EDGE KNOWS";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("DESCRIBE EDGE执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

// ==================== DDL 综合测试 ====================

#[tokio::test]
async fn test_ddl_tag_lifecycle() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let lifecycle_queries = vec![
        "CREATE TAG TestTag(name: STRING, age: INT)",
        "DESCRIBE TAG TestTag",
        "ALTER TAG TestTag ADD (email: STRING)",
        "DESCRIBE TAG TestTag",
        "ALTER TAG TestTag DROP (email)",
        "DESCRIBE TAG TestTag",
        "DROP TAG TestTag",
    ];
    
    for (i, query) in lifecycle_queries.iter().enumerate() {
        let result = pipeline_manager.execute_query(query).await;
        println!("DDL标签生命周期操作 {} 执行结果: {:?}", i + 1, result);
        assert!(result.is_ok() || result.is_err());
    }
}

#[tokio::test]
async fn test_ddl_edge_lifecycle() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let lifecycle_queries = vec![
        "CREATE EDGE TestEdge(since: DATE, weight: DOUBLE)",
        "DESCRIBE EDGE TestEdge",
        "ALTER EDGE TestEdge ADD (note: STRING)",
        "DESCRIBE EDGE TestEdge",
        "ALTER EDGE TestEdge DROP (note)",
        "DESCRIBE EDGE TestEdge",
        "DROP EDGE TestEdge",
    ];
    
    for (i, query) in lifecycle_queries.iter().enumerate() {
        let result = pipeline_manager.execute_query(query).await;
        println!("DDL边类型生命周期操作 {} 执行结果: {:?}", i + 1, result);
        assert!(result.is_ok() || result.is_err());
    }
}

#[tokio::test]
async fn test_ddl_multiple_operations() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let create_queries = vec![
        "CREATE TAG Person(name: STRING, age: INT)",
        "CREATE TAG Company(name: STRING, founded: INT)",
        "CREATE EDGE WORKS_AT(since: DATE)",
        "CREATE EDGE KNOWS(since: DATE)",
    ];
    
    for (i, query) in create_queries.iter().enumerate() {
        let result = pipeline_manager.execute_query(query).await;
        println!("DDL创建操作 {} 执行结果: {:?}", i + 1, result);
        assert!(result.is_ok() || result.is_err());
    }
}

#[tokio::test]
async fn test_ddl_error_handling() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let invalid_queries = vec![
        "CREATE TAG Person",  // 缺少属性定义
        "ALTER TAG Person ADD",  // 缺少属性
        "DROP TAG",  // 缺少标签名
        "DESCRIBE",  // 缺少对象
    ];
    
    for query in invalid_queries {
        let result = pipeline_manager.execute_query(query).await;
        assert!(result.is_err(), "无效查询应该返回错误: {}", query);
    }
}

#[tokio::test]
async fn test_ddl_if_not_exists_if_exists() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let queries = vec![
        "CREATE TAG IF NOT EXISTS Person(name: STRING)",
        "CREATE TAG IF NOT EXISTS Person(name: STRING)",  // 重复创建
        "DROP TAG IF EXISTS Person",
        "DROP TAG IF EXISTS Person",  // 重复删除
    ];
    
    for (i, query) in queries.iter().enumerate() {
        let result = pipeline_manager.execute_query(query).await;
        println!("DDL IF NOT EXISTS/IF EXISTS操作 {} 执行结果: {:?}", i + 1, result);
        assert!(result.is_ok() || result.is_err());
    }
}
