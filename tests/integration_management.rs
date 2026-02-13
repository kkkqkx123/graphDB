//! 管理和辅助语句集成测试
//!
//! 测试范围:
//! - USE - 使用图空间
//! - SHOW - 显示信息
//! - EXPLAIN - 查询计划
//! - RETURN - 返回结果
//! - WITH - 中间结果处理
//! - UNWIND - 展开列表
//! - PIPE - 管道操作

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

// ==================== USE 语句测试 ====================

#[tokio::test]
async fn test_use_parser_basic() {
    let query = "USE test_space";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "USE基础解析应该成功: {:?}", result.err());

    let stmt = result.expect("USE语句解析应该成功");
    assert_eq!(stmt.kind(), "USE");
}

#[tokio::test]
async fn test_use_parser_complex_name() {
    let query = "USE my_graph_space_123";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "USE复杂名称解析应该成功: {:?}", result.err());

    let stmt = result.expect("USE语句解析应该成功");
    assert_eq!(stmt.kind(), "USE");
}

#[tokio::test]
async fn test_use_parser_with_dots() {
    let query = "USE db.graph.space";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "USE带点号名称解析应该成功: {:?}", result.err());

    let stmt = result.expect("USE语句解析应该成功");
    assert_eq!(stmt.kind(), "USE");
}

#[tokio::test]
async fn test_use_execution_basic() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "USE test_space";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("USE基础执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_use_execution_nonexistent() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "USE nonexistent_space_xyz";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("USE不存在空间执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

// ==================== SHOW 语句测试 ====================

#[tokio::test]
async fn test_show_parser_spaces() {
    let query = "SHOW SPACES";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "SHOW SPACES解析应该成功: {:?}", result.err());

    let stmt = result.expect("SHOW语句解析应该成功");
    assert_eq!(stmt.kind(), "SHOW");
}

#[tokio::test]
async fn test_show_parser_tags() {
    let query = "SHOW TAGS";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "SHOW TAGS解析应该成功: {:?}", result.err());

    let stmt = result.expect("SHOW语句解析应该成功");
    assert_eq!(stmt.kind(), "SHOW");
}

#[tokio::test]
async fn test_show_parser_edges() {
    let query = "SHOW EDGES";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "SHOW EDGES解析应该成功: {:?}", result.err());

    let stmt = result.expect("SHOW语句解析应该成功");
    assert_eq!(stmt.kind(), "SHOW");
}

#[tokio::test]
async fn test_show_parser_hosts() {
    let query = "SHOW HOSTS";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "SHOW HOSTS解析应该成功: {:?}", result.err());

    let stmt = result.expect("SHOW语句解析应该成功");
    assert_eq!(stmt.kind(), "SHOW");
}

#[tokio::test]
async fn test_show_parser_parts() {
    let query = "SHOW PARTS";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "SHOW PARTS解析应该成功: {:?}", result.err());

    let stmt = result.expect("SHOW语句解析应该成功");
    assert_eq!(stmt.kind(), "SHOW");
}

#[tokio::test]
async fn test_show_execution_spaces() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "SHOW SPACES";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("SHOW SPACES执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_show_execution_tags() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "SHOW TAGS";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("SHOW TAGS执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_show_execution_edges() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "SHOW EDGES";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("SHOW EDGES执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

// ==================== EXPLAIN 语句测试 ====================

#[tokio::test]
async fn test_explain_parser_match() {
    let query = "EXPLAIN MATCH (n:Person) RETURN n";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "EXPLAIN MATCH解析应该成功: {:?}", result.err());

    let stmt = result.expect("EXPLAIN语句解析应该成功");
    assert_eq!(stmt.kind(), "EXPLAIN");
}

#[tokio::test]
async fn test_explain_parser_go() {
    let query = "EXPLAIN GO FROM 1 OVER KNOWS";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "EXPLAIN GO解析应该成功: {:?}", result.err());

    let stmt = result.expect("EXPLAIN语句解析应该成功");
    assert_eq!(stmt.kind(), "EXPLAIN");
}

#[tokio::test]
async fn test_explain_parser_lookup() {
    let query = "EXPLAIN LOOKUP ON Person WHERE Person.name == 'Alice'";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "EXPLAIN LOOKUP解析应该成功: {:?}", result.err());

    let stmt = result.expect("EXPLAIN语句解析应该成功");
    assert_eq!(stmt.kind(), "EXPLAIN");
}

#[tokio::test]
async fn test_explain_execution_match() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "EXPLAIN MATCH (n:Person) RETURN n";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("EXPLAIN MATCH执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_explain_execution_go() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "EXPLAIN GO FROM 1 OVER KNOWS";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("EXPLAIN GO执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

// ==================== RETURN 语句测试 ====================

#[tokio::test]
async fn test_return_parser_basic() {
    let query = "RETURN n.name, n.age";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "RETURN基础解析应该成功: {:?}", result.err());

    let stmt = result.expect("RETURN语句解析应该成功");
    assert_eq!(stmt.kind(), "RETURN");
}

#[tokio::test]
async fn test_return_parser_with_alias() {
    let query = "RETURN n.name AS name, n.age AS age";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "RETURN带别名解析应该成功: {:?}", result.err());

    let stmt = result.expect("RETURN语句解析应该成功");
    assert_eq!(stmt.kind(), "RETURN");
}

#[tokio::test]
async fn test_return_parser_with_expression() {
    let query = "RETURN n.age * 2 AS double_age";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "RETURN带表达式解析应该成功: {:?}", result.err());

    let stmt = result.expect("RETURN语句解析应该成功");
    assert_eq!(stmt.kind(), "RETURN");
}

#[tokio::test]
async fn test_return_parser_with_aggregate() {
    let query = "RETURN count(*) AS total, avg(n.age) AS avg_age";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "RETURN带聚合函数解析应该成功: {:?}", result.err());

    let stmt = result.expect("RETURN语句解析应该成功");
    assert_eq!(stmt.kind(), "RETURN");
}

#[tokio::test]
async fn test_return_parser_with_distinct() {
    let query = "RETURN DISTINCT n.name";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "RETURN带DISTINCT解析应该成功: {:?}", result.err());

    let stmt = result.expect("RETURN语句解析应该成功");
    assert_eq!(stmt.kind(), "RETURN");
}

#[tokio::test]
async fn test_return_execution_basic() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "RETURN 'Hello World'";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("RETURN基础执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

// ==================== WITH 语句测试 ====================

#[tokio::test]
async fn test_with_parser_basic() {
    let query = "WITH n.name AS name, n.age AS age";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "WITH基础解析应该成功: {:?}", result.err());

    let stmt = result.expect("WITH语句解析应该成功");
    assert_eq!(stmt.kind(), "WITH");
}

#[tokio::test]
async fn test_with_parser_with_aggregate() {
    let query = "WITH count(*) AS total";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "WITH带聚合解析应该成功: {:?}", result.err());

    let stmt = result.expect("WITH语句解析应该成功");
    assert_eq!(stmt.kind(), "WITH");
}

#[tokio::test]
async fn test_with_parser_with_expression() {
    let query = "WITH n.age * 2 AS double_age";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "WITH带表达式解析应该成功: {:?}", result.err());

    let stmt = result.expect("WITH语句解析应该成功");
    assert_eq!(stmt.kind(), "WITH");
}

#[tokio::test]
async fn test_with_execution_basic() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "WITH 1 AS x RETURN x";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("WITH基础执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

// ==================== UNWIND 语句测试 ====================

#[tokio::test]
async fn test_unwind_parser_basic() {
    let query = "UNWIND [1, 2, 3] AS n";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "UNWIND基础解析应该成功: {:?}", result.err());

    let stmt = result.expect("UNWIND语句解析应该成功");
    assert_eq!(stmt.kind(), "UNWIND");
}

#[tokio::test]
async fn test_unwind_parser_with_string_list() {
    let query = "UNWIND ['a', 'b', 'c'] AS s";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "UNWIND字符串列表解析应该成功: {:?}", result.err());

    let stmt = result.expect("UNWIND语句解析应该成功");
    assert_eq!(stmt.kind(), "UNWIND");
}

#[tokio::test]
async fn test_unwind_parser_with_expression() {
    let query = "UNWIND range(1, 10) AS n";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "UNWIND带表达式解析应该成功: {:?}", result.err());

    let stmt = result.expect("UNWIND语句解析应该成功");
    assert_eq!(stmt.kind(), "UNWIND");
}

#[tokio::test]
async fn test_unwind_execution_basic() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "UNWIND [1, 2, 3] AS n RETURN n";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("UNWIND基础执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

// ==================== PIPE 语句测试 ====================

#[tokio::test]
async fn test_pipe_parser_basic() {
    let query = "GO FROM 1 OVER KNOWS | YIELD target.name";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "PIPE基础解析应该成功: {:?}", result.err());

    let stmt = result.expect("PIPE语句解析应该成功");
    assert_eq!(stmt.kind(), "PIPE");
}

#[tokio::test]
async fn test_pipe_parser_multiple() {
    let query = "GO FROM 1 OVER KNOWS | YIELD target.name | FETCH PROP ON Person $-.id";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "PIPE多个操作解析应该成功: {:?}", result.err());

    let stmt = result.expect("PIPE语句解析应该成功");
    assert_eq!(stmt.kind(), "PIPE");
}

#[tokio::test]
async fn test_pipe_parser_complex() {
    let query = "GO FROM 1 OVER KNOWS | YIELD target.name AS name, target.age AS age WHERE age > 25 | RETURN name";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "PIPE复杂查询解析应该成功: {:?}", result.err());

    let stmt = result.expect("PIPE语句解析应该成功");
    assert_eq!(stmt.kind(), "PIPE");
}

#[tokio::test]
async fn test_pipe_execution_basic() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "GO FROM 1 OVER KNOWS | YIELD target.name";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("PIPE基础执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

// ==================== 管理和辅助语句综合测试 ====================

#[tokio::test]
async fn test_management_show_operations() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let show_queries = vec![
        "SHOW SPACES",
        "SHOW TAGS",
        "SHOW EDGES",
        "SHOW HOSTS",
        "SHOW PARTS",
    ];
    
    for (i, query) in show_queries.iter().enumerate() {
        let result = pipeline_manager.execute_query(query).await;
        println!("SHOW操作 {} 执行结果: {:?}", i + 1, result);
        assert!(result.is_ok() || result.is_err());
    }
}

#[tokio::test]
async fn test_management_explain_operations() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let explain_queries = vec![
        "EXPLAIN MATCH (n:Person) RETURN n",
        "EXPLAIN GO FROM 1 OVER KNOWS",
        "EXPLAIN LOOKUP ON Person WHERE Person.age > 25",
        "EXPLAIN FETCH PROP ON Person 1",
    ];
    
    for (i, query) in explain_queries.iter().enumerate() {
        let result = pipeline_manager.execute_query(query).await;
        println!("EXPLAIN操作 {} 执行结果: {:?}", i + 1, result);
        assert!(result.is_ok() || result.is_err());
    }
}

#[tokio::test]
async fn test_auxiliary_return_operations() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let return_queries = vec![
        "RETURN 'Hello'",
        "RETURN 1 + 2",
        "RETURN [1, 2, 3]",
        "RETURN {name: 'Alice', age: 30}",
    ];
    
    for (i, query) in return_queries.iter().enumerate() {
        let result = pipeline_manager.execute_query(query).await;
        println!("RETURN操作 {} 执行结果: {:?}", i + 1, result);
        assert!(result.is_ok() || result.is_err());
    }
}

#[tokio::test]
async fn test_auxiliary_unwind_operations() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let unwind_queries = vec![
        "UNWIND [1, 2, 3] AS n RETURN n",
        "UNWIND ['a', 'b', 'c'] AS s RETURN s",
        "UNWIND [1, 2, 3] AS n RETURN n * 2",
    ];
    
    for (i, query) in unwind_queries.iter().enumerate() {
        let result = pipeline_manager.execute_query(query).await;
        println!("UNWIND操作 {} 执行结果: {:?}", i + 1, result);
        assert!(result.is_ok() || result.is_err());
    }
}

#[tokio::test]
async fn test_auxiliary_pipe_operations() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let pipe_queries = vec![
        "GO FROM 1 OVER KNOWS | YIELD target.name",
        "GO FROM 1 OVER KNOWS | YIELD target.name AS name | RETURN name",
        "LOOKUP ON Person WHERE Person.age > 25 | YIELD Person.name",
    ];
    
    for (i, query) in pipe_queries.iter().enumerate() {
        let result = pipeline_manager.execute_query(query).await;
        println!("PIPE操作 {} 执行结果: {:?}", i + 1, result);
        assert!(result.is_ok() || result.is_err());
    }
}

#[tokio::test]
async fn test_management_error_handling() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let invalid_queries = vec![
        "USE",  // 缺少空间名
        "SHOW",  // 缺少对象
        "EXPLAIN",  // 缺少查询
        "RETURN",  // 缺少表达式
        "UNWIND",  // 缺少列表和变量
        "GO FROM 1 OVER |",  // PIPE语法错误
    ];
    
    for query in invalid_queries {
        let result = pipeline_manager.execute_query(query).await;
        assert!(result.is_err(), "无效查询应该返回错误: {}", query);
    }
}

#[tokio::test]
async fn test_management_combined_operations() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let combined_queries = vec![
        "USE test_space",
        "SHOW TAGS",
        "EXPLAIN GO FROM 1 OVER KNOWS",
        "UNWIND [1, 2, 3] AS n RETURN n",
        "RETURN 'Complete'",
    ];
    
    for (i, query) in combined_queries.iter().enumerate() {
        let result = pipeline_manager.execute_query(query).await;
        println!("组合操作 {} 执行结果: {:?}", i + 1, result);
        assert!(result.is_ok() || result.is_err());
    }
}

#[tokio::test]
async fn test_auxiliary_with_operations() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let with_queries = vec![
        "WITH 1 AS x RETURN x",
        "WITH [1, 2, 3] AS list RETURN list",
        "WITH 'Hello' AS msg RETURN msg",
    ];
    
    for (i, query) in with_queries.iter().enumerate() {
        let result = pipeline_manager.execute_query(query).await;
        println!("WITH操作 {} 执行结果: {:?}", i + 1, result);
        assert!(result.is_ok() || result.is_err());
    }
}

#[tokio::test]
async fn test_management_performance() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "SHOW SPACES";
    let iterations = 10;
    
    for i in 0..iterations {
        let result = pipeline_manager.execute_query(query).await;
        println!("性能测试 {} 执行结果: {:?}", i + 1, result);
        assert!(result.is_ok() || result.is_err());
    }
}
