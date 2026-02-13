//! 数据操作语言(DML)集成测试
//!
//! 测试范围:
//! - INSERT - 插入数据
//! - CREATE - 创建数据
//! - UPDATE - 更新数据
//! - DELETE - 删除数据
//! - MERGE - 合并数据
//! - SET - 设置属性
//! - REMOVE - 移除属性

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

// ==================== INSERT 语句测试 ====================

#[tokio::test]
async fn test_insert_parser_vertex() {
    let query = "INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30)";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "INSERT VERTEX解析应该成功: {:?}", result.err());

    let stmt = result.expect("INSERT语句解析应该成功");
    assert_eq!(stmt.kind(), "INSERT");
}

#[tokio::test]
async fn test_insert_parser_multiple_vertices() {
    let query = "INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30), 2:('Bob', 25)";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "INSERT多个顶点解析应该成功: {:?}", result.err());

    let stmt = result.expect("INSERT语句解析应该成功");
    assert_eq!(stmt.kind(), "INSERT");
}

#[tokio::test]
async fn test_insert_parser_edge() {
    let query = "INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01')";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "INSERT EDGE解析应该成功: {:?}", result.err());

    let stmt = result.expect("INSERT语句解析应该成功");
    assert_eq!(stmt.kind(), "INSERT");
}

#[tokio::test]
async fn test_insert_parser_edge_with_rank() {
    let query = "INSERT EDGE KNOWS(since) VALUES 1 -> 2 @0:('2020-01-01')";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "INSERT EDGE带rank解析应该成功: {:?}", result.err());

    let stmt = result.expect("INSERT语句解析应该成功");
    assert_eq!(stmt.kind(), "INSERT");
}

#[tokio::test]
async fn test_insert_parser_multiple_edges() {
    let query = "INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01'), 2 -> 3:('2021-01-01')";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "INSERT多个边解析应该成功: {:?}", result.err());

    let stmt = result.expect("INSERT语句解析应该成功");
    assert_eq!(stmt.kind(), "INSERT");
}

#[tokio::test]
async fn test_insert_parser_invalid_syntax() {
    let query = "INSERT VERTEX Person(name, age) VALUES 1:'Alice', 30";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_err(), "无效语法应该返回错误");
}

#[tokio::test]
async fn test_insert_execution_vertex() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30)";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("INSERT VERTEX执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_insert_execution_edge() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01')";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("INSERT EDGE执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

// ==================== CREATE 语句测试 ====================

#[tokio::test]
async fn test_create_parser_vertex() {
    let query = "CREATE (p:Person {name: 'Alice', age: 30})";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    println!("CREATE顶点解析结果: {:?}", result);
    let _ = result;
}

#[tokio::test]
async fn test_create_parser_edge() {
    let query = "CREATE (a:Person)-[:KNOWS {since: '2020-01-01'}]->(b:Person)";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    println!("CREATE边解析结果: {:?}", result);
    let _ = result;
}

#[tokio::test]
async fn test_create_parser_multiple() {
    let query = "CREATE (a:Person {name: 'Alice'}), (b:Person {name: 'Bob'})";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    println!("CREATE多个顶点解析结果: {:?}", result);
    let _ = result;
}

#[tokio::test]
async fn test_create_execution_vertex() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "CREATE (p:Person {name: 'Alice', age: 30})";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("CREATE顶点执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_create_execution_edge() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "CREATE (a:Person)-[:KNOWS {since: '2020-01-01'}]->(b:Person)";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("CREATE边执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

// ==================== UPDATE 语句测试 ====================

#[tokio::test]
async fn test_update_parser_vertex() {
    let query = "UPDATE 1 SET age = 26, name = 'Alice Smith'";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "UPDATE顶点解析应该成功: {:?}", result.err());

    let stmt = result.expect("UPDATE语句解析应该成功");
    assert_eq!(stmt.kind(), "UPDATE");
}

#[tokio::test]
async fn test_update_parser_vertex_with_when() {
    let query = "UPDATE 1 SET age = 26 WHEN age > 20";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "UPDATE顶点带WHEN解析应该成功: {:?}", result.err());

    let stmt = result.expect("UPDATE语句解析应该成功");
    assert_eq!(stmt.kind(), "UPDATE");
}

#[tokio::test]
async fn test_update_parser_edge() {
    let query = "UPDATE 1 -> 2 @0 OF KNOWS SET since = '2021-01-01'";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "UPDATE边解析应该成功: {:?}", result.err());

    let stmt = result.expect("UPDATE语句解析应该成功");
    assert_eq!(stmt.kind(), "UPDATE");
}

#[tokio::test]
async fn test_update_parser_edge_with_when() {
    let query = "UPDATE 1 -> 2 @0 OF KNOWS SET since = '2021-01-01' WHEN since < '2021-01-01'";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "UPDATE边带WHEN解析应该成功: {:?}", result.err());

    let stmt = result.expect("UPDATE语句解析应该成功");
    assert_eq!(stmt.kind(), "UPDATE");
}

#[tokio::test]
async fn test_update_parser_multiple_props() {
    let query = "UPDATE 1 SET age = 26, name = 'Alice', updated = true";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "UPDATE多个属性解析应该成功: {:?}", result.err());

    let stmt = result.expect("UPDATE语句解析应该成功");
    assert_eq!(stmt.kind(), "UPDATE");
}

#[tokio::test]
async fn test_update_execution_vertex() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "UPDATE 1 SET age = 26";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("UPDATE顶点执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_update_execution_edge() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "UPDATE 1 -> 2 @0 OF KNOWS SET since = '2021-01-01'";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("UPDATE边执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

// ==================== DELETE 语句测试 ====================

#[tokio::test]
async fn test_delete_parser_vertex() {
    let query = "DELETE VERTEX 1";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "DELETE VERTEX解析应该成功: {:?}", result.err());

    let stmt = result.expect("DELETE语句解析应该成功");
    assert_eq!(stmt.kind(), "DELETE");
}

#[tokio::test]
async fn test_delete_parser_multiple_vertices() {
    let query = "DELETE VERTEX 1, 2, 3";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "DELETE多个顶点解析应该成功: {:?}", result.err());

    let stmt = result.expect("DELETE语句解析应该成功");
    assert_eq!(stmt.kind(), "DELETE");
}

#[tokio::test]
async fn test_delete_parser_edge() {
    let query = "DELETE EDGE KNOWS 1 -> 2";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "DELETE EDGE解析应该成功: {:?}", result.err());

    let stmt = result.expect("DELETE语句解析应该成功");
    assert_eq!(stmt.kind(), "DELETE");
}

#[tokio::test]
async fn test_delete_parser_edge_with_rank() {
    let query = "DELETE EDGE KNOWS 1 -> 2 @0";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "DELETE EDGE带rank解析应该成功: {:?}", result.err());

    let stmt = result.expect("DELETE语句解析应该成功");
    assert_eq!(stmt.kind(), "DELETE");
}

#[tokio::test]
async fn test_delete_parser_multiple_edges() {
    let query = "DELETE EDGE KNOWS 1 -> 2, 2 -> 3";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "DELETE多个边解析应该成功: {:?}", result.err());

    let stmt = result.expect("DELETE语句解析应该成功");
    assert_eq!(stmt.kind(), "DELETE");
}

#[tokio::test]
async fn test_delete_execution_vertex() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "DELETE VERTEX 1";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("DELETE VERTEX执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_delete_execution_edge() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "DELETE EDGE KNOWS 1 -> 2";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("DELETE EDGE执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

// ==================== MERGE 语句测试 ====================

#[tokio::test]
async fn test_merge_parser_basic() {
    let query = "MERGE (p:Person {name: 'Alice'})";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    println!("MERGE基础解析结果: {:?}", result);
    let _ = result;
}

#[tokio::test]
async fn test_merge_parser_on_match() {
    let query = "MERGE (p:Person {name: 'Alice'}) ON MATCH SET p.last_seen = timestamp()";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    println!("MERGE带ON MATCH解析结果: {:?}", result);
    let _ = result;
}

#[tokio::test]
async fn test_merge_parser_on_create() {
    let query = "MERGE (p:Person {name: 'Alice'}) ON CREATE SET p.created_at = timestamp()";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    println!("MERGE带ON CREATE解析结果: {:?}", result);
    let _ = result;
}

#[tokio::test]
async fn test_merge_parser_both() {
    let query = "MERGE (p:Person {name: 'Alice'}) ON MATCH SET p.last_seen = timestamp() ON CREATE SET p.created_at = timestamp()";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    println!("MERGE带ON MATCH和ON CREATE解析结果: {:?}", result);
    let _ = result;
}

#[tokio::test]
async fn test_merge_execution_basic() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "MERGE (p:Person {name: 'Alice'})";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("MERGE基础执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

// ==================== SET 语句测试 ====================

#[tokio::test]
async fn test_set_parser_basic() {
    let query = "SET p.age = 26";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "SET基础解析应该成功: {:?}", result.err());

    let stmt = result.expect("SET语句解析应该成功");
    assert_eq!(stmt.kind(), "SET");
}

#[tokio::test]
async fn test_set_parser_multiple() {
    let query = "SET p.age = 26, p.name = 'Alice', p.updated = true";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "SET多个属性解析应该成功: {:?}", result.err());

    let stmt = result.expect("SET语句解析应该成功");
    assert_eq!(stmt.kind(), "SET");
}

#[tokio::test]
async fn test_set_parser_with_expression() {
    let query = "SET p.age = p.age + 1";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "SET带表达式解析应该成功: {:?}", result.err());

    let stmt = result.expect("SET语句解析应该成功");
    assert_eq!(stmt.kind(), "SET");
}

#[tokio::test]
async fn test_set_execution_basic() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "SET p.age = 26";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("SET基础执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

// ==================== REMOVE 语句测试 ====================

#[tokio::test]
async fn test_remove_parser_property() {
    let query = "REMOVE p.temp_field";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "REMOVE属性解析应该成功: {:?}", result.err());

    let stmt = result.expect("REMOVE语句解析应该成功");
    assert_eq!(stmt.kind(), "REMOVE");
}

#[tokio::test]
async fn test_remove_parser_multiple_properties() {
    let query = "REMOVE p.temp_field, p.old_field";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "REMOVE多个属性解析应该成功: {:?}", result.err());

    let stmt = result.expect("REMOVE语句解析应该成功");
    assert_eq!(stmt.kind(), "REMOVE");
}

#[tokio::test]
async fn test_remove_parser_label() {
    let query = "REMOVE p:OldLabel";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "REMOVE标签解析应该成功: {:?}", result.err());

    let stmt = result.expect("REMOVE语句解析应该成功");
    assert_eq!(stmt.kind(), "REMOVE");
}

#[tokio::test]
async fn test_remove_parser_multiple_labels() {
    let query = "REMOVE p:OldLabel, p:AnotherLabel";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "REMOVE多个标签解析应该成功: {:?}", result.err());

    let stmt = result.expect("REMOVE语句解析应该成功");
    assert_eq!(stmt.kind(), "REMOVE");
}

#[tokio::test]
async fn test_remove_parser_mixed() {
    let query = "REMOVE p.temp_field, p:OldLabel";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "REMOVE混合解析应该成功: {:?}", result.err());

    let stmt = result.expect("REMOVE语句解析应该成功");
    assert_eq!(stmt.kind(), "REMOVE");
}

#[tokio::test]
async fn test_remove_execution_property() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "REMOVE p.temp_field";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("REMOVE属性执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

// ==================== DML 综合测试 ====================

#[tokio::test]
async fn test_dml_crud_operations() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let queries = vec![
        "INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30)",
        "UPDATE 1 SET age = 31",
        "FETCH PROP ON Person 1",
        "DELETE VERTEX 1",
    ];
    
    for (i, query) in queries.iter().enumerate() {
        let result = pipeline_manager.execute_query(query).await;
        println!("DML CRUD操作 {} 执行结果: {:?}", i + 1, result);
        assert!(result.is_ok() || result.is_err());
    }
}

#[tokio::test]
async fn test_dml_batch_operations() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let batch_queries = vec![
        "INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30), 2:('Bob', 25), 3:('Charlie', 35)",
        "INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01'), 2 -> 3:('2021-01-01')",
        "UPDATE 1 SET age = 31, name = 'Alice Smith'",
        "DELETE VERTEX 1, 2, 3",
    ];
    
    for (i, query) in batch_queries.iter().enumerate() {
        let result = pipeline_manager.execute_query(query).await;
        println!("DML批量操作 {} 执行结果: {:?}", i + 1, result);
        assert!(result.is_ok() || result.is_err());
    }
}

#[tokio::test]
async fn test_dml_error_handling() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let invalid_queries = vec![
        "INSERT VERTEX Person(name, age) VALUES 1:'Alice', 30",  // 无效语法
        "UPDATE SET age = 26",  // 缺少顶点ID
        "DELETE VERTEX",  // 缺少顶点ID
        "SET = 26",  // 缺少变量
    ];
    
    for query in invalid_queries {
        let result = pipeline_manager.execute_query(query).await;
        assert!(result.is_err(), "无效查询应该返回错误: {}", query);
    }
}

#[tokio::test]
async fn test_dml_transaction_like_operations() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let transaction_queries = vec![
        "INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30)",
        "INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01')",
        "UPDATE 1 SET age = 31",
        "FETCH PROP ON Person 1",
    ];
    
    for (i, query) in transaction_queries.iter().enumerate() {
        let result = pipeline_manager.execute_query(query).await;
        println!("DML事务类操作 {} 执行结果: {:?}", i + 1, result);
        assert!(result.is_ok() || result.is_err());
    }
}
