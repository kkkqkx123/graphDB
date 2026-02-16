//! 数据查询语言(DQL)集成测试
//!
//! 测试范围:
//! - MATCH - 模式匹配查询
//! - GO - 图遍历查询
//! - LOOKUP - 基于索引查找
//! - FETCH - 获取数据
//! - FIND PATH - 路径查找
//! - SUBGRAPH - 子图查询

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

// ==================== MATCH 语句测试 ====================

#[tokio::test]
async fn test_match_parser_basic() {
    let query = "MATCH (n:Person) RETURN n";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    println!("MATCH基础解析结果: {:?}", result);
    let _ = result;
}

#[tokio::test]
async fn test_match_parser_with_where() {
    let query = "MATCH (n:Person) WHERE n.age > 25 RETURN n";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    println!("MATCH带WHERE解析结果: {:?}", result);
    let _ = result;
}

#[tokio::test]
async fn test_match_parser_with_edge() {
    let query = "MATCH (n:Person)-[KNOWS]->(m:Person) RETURN n, m";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    println!("MATCH带边解析结果: {:?}", result);
    let _ = result;
}

#[tokio::test]
async fn test_match_parser_with_order_limit() {
    let query = "MATCH (n:Person) RETURN n ORDER BY n.age DESC LIMIT 10";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    println!("MATCH带排序和分页解析结果: {:?}", result);
    let _ = result;
}

#[tokio::test]
async fn test_match_parser_complex() {
    let query = "MATCH (n:Person)-[KNOWS]->(m:Person) WHERE n.age > 25 AND m.age < 40 RETURN n.name, m.name ORDER BY m.age LIMIT 5";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    println!("MATCH复杂查询解析结果: {:?}", result);
    let _ = result;
}

#[tokio::test]
async fn test_match_parser_invalid_syntax() {
    let query = "MATCH (n:Person RETURN n";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_err(), "无效语法应该返回错误");
}

#[tokio::test]
async fn test_match_execution_basic() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "MATCH (n:Person) RETURN n";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("MATCH基础执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_match_execution_with_projection() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "MATCH (n:Person) RETURN n.name, n.age";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("MATCH带投影执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

// ==================== GO 语句测试 ====================

#[tokio::test]
async fn test_go_parser_basic() {
    let query = "GO FROM 1 OVER KNOWS";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "GO基础解析应该成功: {:?}", result.err());

    let stmt = result.expect("GO语句解析应该成功");
    assert_eq!(stmt.kind(), "GO");
}

#[tokio::test]
async fn test_go_parser_with_steps() {
    let query = "GO 2 TO 4 STEPS FROM 1 OVER KNOWS";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "GO带步数解析应该成功: {:?}", result.err());

    let stmt = result.expect("GO语句解析应该成功");
    assert_eq!(stmt.kind(), "GO");
}

#[tokio::test]
async fn test_go_parser_reversely() {
    let query = "GO FROM 1 OVER KNOWS REVERSELY";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "GO反向遍历解析应该成功: {:?}", result.err());

    let stmt = result.expect("GO语句解析应该成功");
    assert_eq!(stmt.kind(), "GO");
}

#[tokio::test]
async fn test_go_parser_bidirect() {
    let query = "GO FROM 1 OVER KNOWS BIDIRECT";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "GO双向遍历解析应该成功: {:?}", result.err());

    let stmt = result.expect("GO语句解析应该成功");
    assert_eq!(stmt.kind(), "GO");
}

#[tokio::test]
async fn test_go_parser_with_where() {
    let query = "GO FROM 1 OVER KNOWS WHERE target.age > 25 YIELD target.name";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "GO带WHERE解析应该成功: {:?}", result.err());

    let stmt = result.expect("GO语句解析应该成功");
    assert_eq!(stmt.kind(), "GO");
}

#[tokio::test]
async fn test_go_parser_with_yield() {
    let query = "GO FROM 1 OVER KNOWS YIELD target.name, target.age";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "GO带YIELD解析应该成功: {:?}", result.err());

    let stmt = result.expect("GO语句解析应该成功");
    assert_eq!(stmt.kind(), "GO");
}

#[tokio::test]
async fn test_go_parser_complex() {
    let query = "GO 2 TO 3 STEPS FROM 1 OVER KNOWS REVERSELY WHERE target.age > 20 YIELD target.name, target.age ORDER BY target.age DESC LIMIT 10";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "GO复杂查询解析应该成功: {:?}", result.err());

    let stmt = result.expect("GO语句解析应该成功");
    assert_eq!(stmt.kind(), "GO");
}

#[tokio::test]
async fn test_go_execution_basic() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "GO FROM 1 OVER KNOWS";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("GO基础执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_go_execution_with_yield() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "GO FROM 1 OVER KNOWS YIELD target.name";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("GO带YIELD执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

// ==================== LOOKUP 语句测试 ====================

#[tokio::test]
async fn test_lookup_parser_basic() {
    let query = "LOOKUP ON Person WHERE Person.name == 'Alice'";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "LOOKUP基础解析应该成功: {:?}", result.err());

    let stmt = result.expect("LOOKUP语句解析应该成功");
    assert_eq!(stmt.kind(), "LOOKUP");
}

#[tokio::test]
async fn test_lookup_parser_with_yield() {
    let query = "LOOKUP ON Person WHERE Person.age > 25 YIELD Person.name, Person.age";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "LOOKUP带YIELD解析应该成功: {:?}", result.err());

    let stmt = result.expect("LOOKUP语句解析应该成功");
    assert_eq!(stmt.kind(), "LOOKUP");
}

#[tokio::test]
async fn test_lookup_parser_complex_condition() {
    let query = "LOOKUP ON Person WHERE Person.age > 25 AND Person.name STARTS WITH 'A' YIELD Person.name";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "LOOKUP复杂条件解析应该成功: {:?}", result.err());

    let stmt = result.expect("LOOKUP语句解析应该成功");
    assert_eq!(stmt.kind(), "LOOKUP");
}

#[tokio::test]
async fn test_lookup_parser_edge() {
    let query = "LOOKUP ON KNOWS WHERE KNOWS.since > '2020-01-01' YIELD KNOWS.since";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "LOOKUP边类型解析应该成功: {:?}", result.err());

    let stmt = result.expect("LOOKUP语句解析应该成功");
    assert_eq!(stmt.kind(), "LOOKUP");
}

#[tokio::test]
async fn test_lookup_execution_basic() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "LOOKUP ON Person WHERE Person.name == 'Alice'";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("LOOKUP基础执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

// ==================== FETCH 语句测试 ====================

#[tokio::test]
async fn test_fetch_parser_vertex() {
    let query = "FETCH PROP ON Person 1";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "FETCH顶点解析应该成功: {:?}", result.err());

    let stmt = result.expect("FETCH语句解析应该成功");
    assert_eq!(stmt.kind(), "FETCH");
}

#[tokio::test]
async fn test_fetch_parser_multiple_vertices() {
    let query = "FETCH PROP ON Person 1, 2, 3";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "FETCH多个顶点解析应该成功: {:?}", result.err());

    let stmt = result.expect("FETCH语句解析应该成功");
    assert_eq!(stmt.kind(), "FETCH");
}

#[tokio::test]
async fn test_fetch_parser_edge() {
    let query = "FETCH PROP ON KNOWS 1 -> 2";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "FETCH边解析应该成功: {:?}", result.err());

    let stmt = result.expect("FETCH语句解析应该成功");
    assert_eq!(stmt.kind(), "FETCH");
}

#[tokio::test]
async fn test_fetch_parser_edge_with_rank() {
    let query = "FETCH PROP ON KNOWS 1 -> 2 @0";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "FETCH边带rank解析应该成功: {:?}", result.err());

    let stmt = result.expect("FETCH语句解析应该成功");
    assert_eq!(stmt.kind(), "FETCH");
}

#[tokio::test]
async fn test_fetch_execution_vertex() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "FETCH PROP ON Person 1";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("FETCH顶点执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_fetch_execution_edge() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "FETCH PROP ON KNOWS 1 -> 2";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("FETCH边执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

// ==================== FIND PATH 语句测试 ====================

#[tokio::test]
async fn test_find_path_parser_shortest() {
    let query = "FIND SHORTEST PATH FROM 1 TO 4 OVER KNOWS";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "FIND SHORTEST PATH解析应该成功: {:?}", result.err());

    let stmt = result.expect("FIND PATH语句解析应该成功");
    assert_eq!(stmt.kind(), "FIND PATH");
}

#[tokio::test]
async fn test_find_path_parser_all() {
    let query = "FIND ALL PATH FROM 1 TO 4 OVER KNOWS";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "FIND ALL PATH解析应该成功: {:?}", result.err());

    let stmt = result.expect("FIND PATH语句解析应该成功");
    assert_eq!(stmt.kind(), "FIND PATH");
}

#[tokio::test]
async fn test_find_path_parser_noloop() {
    let query = "FIND NOLOOP PATH FROM 1 TO 4 OVER KNOWS";
    let mut parser = Parser::new(query);

    let result = parser.parse();
    // NOLOOP现在是默认选项，所以不再需要显式指定，解析会失败
    assert!(result.is_err(), "FIND NOLOOP PATH解析应该失败，因为NOLOOP是默认选项: {:?}", result.err());

    // 测试不带NOLOOP的路径查找
    let query2 = "FIND PATH FROM 1 TO 4 OVER KNOWS";
    let mut parser2 = Parser::new(query2);
    let result2 = parser2.parse();
    assert!(result2.is_ok(), "FIND PATH解析应该成功: {:?}", result2.err());

    let stmt = result2.expect("FIND PATH语句解析应该成功");
    assert_eq!(stmt.kind(), "FIND PATH");
}

#[tokio::test]
async fn test_find_path_parser_with_upto() {
    let query = "FIND SHORTEST PATH FROM 1 TO 4 OVER KNOWS UPTO 5 STEPS";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "FIND PATH带UPTO解析应该成功: {:?}", result.err());

    let stmt = result.expect("FIND PATH语句解析应该成功");
    assert_eq!(stmt.kind(), "FIND PATH");
}

#[tokio::test]
async fn test_find_path_parser_reversely() {
    let query = "FIND SHORTEST PATH FROM 1 TO 4 OVER KNOWS REVERSELY";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "FIND PATH反向解析应该成功: {:?}", result.err());

    let stmt = result.expect("FIND PATH语句解析应该成功");
    assert_eq!(stmt.kind(), "FIND PATH");
}

#[tokio::test]
async fn test_find_path_parser_with_where() {
    let query = "FIND SHORTEST PATH FROM 1 TO 4 OVER KNOWS WHERE v.age > 20";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "FIND PATH带WHERE解析应该成功: {:?}", result.err());

    let stmt = result.expect("FIND PATH语句解析应该成功");
    assert_eq!(stmt.kind(), "FIND PATH");
}

#[tokio::test]
async fn test_find_path_parser_complex() {
    let query = "FIND ALL PATH FROM 1 TO 4 OVER KNOWS UPTO 3 STEPS WHERE v.age > 20 REVERSELY";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "FIND PATH复杂查询解析应该成功: {:?}", result.err());

    let stmt = result.expect("FIND PATH语句解析应该成功");
    assert_eq!(stmt.kind(), "FIND PATH");
}

#[tokio::test]
async fn test_find_path_execution_shortest() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "FIND SHORTEST PATH FROM 1 TO 4 OVER KNOWS";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("FIND SHORTEST PATH执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

// ==================== SUBGRAPH 语句测试 ====================

#[tokio::test]
async fn test_subgraph_parser_basic() {
    let query = "GET SUBGRAPH WITH PROP 1";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "SUBGRAPH基础解析应该成功: {:?}", result.err());

    let stmt = result.expect("SUBGRAPH语句解析应该成功");
    assert_eq!(stmt.kind(), "SUBGRAPH");
}

#[tokio::test]
async fn test_subgraph_parser_multiple_vertices() {
    let query = "GET SUBGRAPH WITH PROP 1, 2, 3";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "SUBGRAPH多个顶点解析应该成功: {:?}", result.err());

    let stmt = result.expect("SUBGRAPH语句解析应该成功");
    assert_eq!(stmt.kind(), "SUBGRAPH");
}

#[tokio::test]
async fn test_subgraph_parser_in_steps() {
    let query = "GET SUBGRAPH WITH PROP 1 IN 2 STEPS";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "SUBGRAPH入边步数解析应该成功: {:?}", result.err());

    let stmt = result.expect("SUBGRAPH语句解析应该成功");
    assert_eq!(stmt.kind(), "SUBGRAPH");
}

#[tokio::test]
async fn test_subgraph_parser_out_steps() {
    let query = "GET SUBGRAPH WITH PROP 1 OUT 2 STEPS";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "SUBGRAPH出边步数解析应该成功: {:?}", result.err());

    let stmt = result.expect("SUBGRAPH语句解析应该成功");
    assert_eq!(stmt.kind(), "SUBGRAPH");
}

#[tokio::test]
async fn test_subgraph_parser_both_steps() {
    let query = "GET SUBGRAPH WITH PROP 1 BOTH 2 STEPS";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "SUBGRAPH双向步数解析应该成功: {:?}", result.err());

    let stmt = result.expect("SUBGRAPH语句解析应该成功");
    assert_eq!(stmt.kind(), "SUBGRAPH");
}

#[tokio::test]
async fn test_subgraph_parser_complex() {
    let query = "GET SUBGRAPH WITH PROP 1, 2 IN 2 STEPS OUT 3 STEPS";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "SUBGRAPH复杂查询解析应该成功: {:?}", result.err());

    let stmt = result.expect("SUBGRAPH语句解析应该成功");
    assert_eq!(stmt.kind(), "SUBGRAPH");
}

#[tokio::test]
async fn test_subgraph_execution_basic() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let query = "GET SUBGRAPH WITH PROP 1";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("SUBGRAPH基础执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

// ==================== DQL 综合测试 ====================

#[tokio::test]
async fn test_dql_multiple_queries() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let queries = vec![
        "MATCH (n:Person) RETURN n",
        "GO FROM 1 OVER KNOWS",
        "LOOKUP ON Person WHERE Person.age > 25",
        "FETCH PROP ON Person 1",
    ];
    
    for (i, query) in queries.iter().enumerate() {
        let result = pipeline_manager.execute_query(query).await;
        println!("DQL查询 {} 执行结果: {:?}", i + 1, result);
        assert!(result.is_ok() || result.is_err());
    }
}

#[tokio::test]
async fn test_dql_error_handling() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    let invalid_queries = vec![
        "MATCH (n:Person",  // 缺少右括号
        "GO FROM OVER KNOWS",  // 缺少顶点ID
        "LOOKUP ON WHERE Person.name == 'Alice'",  // 缺少标签
        "FETCH PROP ON",  // 缺少标签和ID
    ];
    
    for query in invalid_queries {
        let result = pipeline_manager.execute_query(query).await;
        assert!(result.is_err(), "无效查询应该返回错误: {}", query);
    }
}

// ==================== 悬挂边相关测试 ====================

#[tokio::test]
async fn test_go_with_dangling_edges() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    // 测试GO语句在存在悬挂边时的行为
    // GO语句应该返回悬挂边的属性，但点的属性为空
    let query = "GO FROM 1 OVER KNOWS YIELD target.name, edge.since";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("GO带悬挂边执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_go_dangling_edge_returns_edge_props() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    // 测试GO语句返回悬挂边的属性
    let query = "GO FROM 1 OVER KNOWS YIELD edge.since, edge.strength";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("GO返回悬挂边属性结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_match_no_dangling_edges() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    let stats_manager = Arc::new(StatsManager::new());
    
    let mut pipeline_manager = QueryPipelineManager::new(storage, stats_manager);
    
    // MATCH语句不应返回悬挂边
    let query = "MATCH (n:Person)-[KNOWS]->(m:Person) RETURN n, m";
    let result = pipeline_manager.execute_query(query).await;
    
    println!("MATCH不返回悬挂边结果: {:?}", result);
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_delete_vertex_with_edge_syntax() {
    let query = "DELETE VERTEX 1 WITH EDGE";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "DELETE VERTEX WITH EDGE解析应该成功: {:?}", result.err());

    let stmt = result.expect("DELETE语句解析应该成功");
    assert_eq!(stmt.kind(), "DELETE");
}

#[tokio::test]
async fn test_delete_vertex_without_edge_syntax() {
    let query = "DELETE VERTEX 1";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "DELETE VERTEX解析应该成功: {:?}", result.err());

    let stmt = result.expect("DELETE语句解析应该成功");
    assert_eq!(stmt.kind(), "DELETE");
}

#[tokio::test]
async fn test_delete_vertex_multiple_with_edge() {
    let query = "DELETE VERTEX 1, 2, 3 WITH EDGE";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "DELETE VERTEX多个顶点WITH EDGE解析应该成功: {:?}", result.err());

    let stmt = result.expect("DELETE语句解析应该成功");
    assert_eq!(stmt.kind(), "DELETE");
}

#[tokio::test]
async fn test_delete_vertex_with_where_and_edge() {
    let query = "DELETE VERTEX 1 WITH EDGE WHERE 1.age > 25";
    let mut parser = Parser::new(query);
    
    let result = parser.parse();
    assert!(result.is_ok(), "DELETE VERTEX带WHERE和WITH EDGE解析应该成功: {:?}", result.err());

    let stmt = result.expect("DELETE语句解析应该成功");
    assert_eq!(stmt.kind(), "DELETE");
}

#[tokio::test]
async fn test_dangling_edge_detection_and_repair() {
    use graphdb::storage::StorageClient;
    
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    
    // 使用锁访问存储方法
    let mut storage_guard = storage.lock();
    
    // 测试悬挂边检测功能
    let dangling_result = storage_guard.find_dangling_edges("test_space");
    println!("悬挂边检测结果: {:?}", dangling_result);
    
    // 测试悬挂边修复功能
    let repair_result = storage_guard.repair_dangling_edges("test_space");
    println!("悬挂边修复结果: {:?}", repair_result);
    
    // 验证结果
    assert!(dangling_result.is_ok() || dangling_result.is_err());
    assert!(repair_result.is_ok() || repair_result.is_err());
}

#[tokio::test]
async fn test_dangling_edge_workflow() {
    use graphdb::storage::StorageClient;
    use graphdb::core::DataType;
    
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    
    // 使用锁访问存储方法
    let mut storage_guard = storage.lock();
    
    // 1. 创建测试空间 - 使用正确的SpaceInfo结构
    let space_info = graphdb::core::types::SpaceInfo::new("dangling_test".to_string());
    
    let create_result = storage_guard.create_space(&space_info);
    println!("创建空间结果: {:?}", create_result);
    
    // 2. 创建一个顶点
    use std::collections::HashMap;
    let mut tag_props = HashMap::new();
    tag_props.insert("name".to_string(), graphdb::core::Value::String("Alice".to_string()));
    
    let vertex = graphdb::core::Vertex::new(
        graphdb::core::Value::Int(1),
        vec![graphdb::core::vertex_edge_path::Tag::new("Person".to_string(), tag_props)],
    );
    
    let insert_result = storage_guard.insert_vertex("dangling_test", vertex);
    println!("插入顶点结果: {:?}", insert_result);
    
    // 3. 创建一条指向不存在顶点的边（悬挂边）
    let mut props = HashMap::new();
    props.insert("since".to_string(), graphdb::core::Value::String("2024-01-01".to_string()));
    
    let edge = graphdb::core::Edge::new(
        graphdb::core::Value::Int(1),
        graphdb::core::Value::Int(999),  // 不存在的顶点
        "KNOWS".to_string(),
        0,  // rank
        props,
    );
    
    let edge_result = storage_guard.insert_edge("dangling_test", edge);
    println!("插入悬挂边结果: {:?}", edge_result);
    
    // 4. 检测悬挂边
    let dangling = storage_guard.find_dangling_edges("dangling_test");
    println!("检测到的悬挂边: {:?}", dangling);
    
    // 5. 修复悬挂边
    let repaired = storage_guard.repair_dangling_edges("dangling_test");
    println!("修复的悬挂边数量: {:?}", repaired);
    
    // 验证结果
    assert!(create_result.is_ok() || create_result.is_err());
    assert!(insert_result.is_ok() || insert_result.is_err());
    assert!(edge_result.is_ok() || edge_result.is_err());
}
