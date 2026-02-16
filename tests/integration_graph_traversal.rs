//! 图遍历集成测试
//!
//! 测试范围：
//! - 最短路径算法执行器创建和配置
//! - 多源最短路径算法
//! - 子图查询执行器
//! - 算法上下文和配置
//! - 路径数据结构

mod common;

use common::TestStorage;
use graphdb::core::{Value, Vertex, Edge, Path, Step};
use graphdb::core::vertex_edge_path::Tag;
use graphdb::core::DataType;
use graphdb::query::executor::data_processing::graph_traversal::algorithms::{
    MultiShortestPathExecutor, SubgraphExecutor, SubgraphConfig,
    AlgorithmContext, AlgorithmStats,
};
use graphdb::query::executor::base::{Executor, EdgeDirection as ExecEdgeDirection};
use graphdb::storage::RedbStorage;
use std::sync::Arc;
use parking_lot::Mutex;
use std::collections::HashMap;

fn get_storage(storage: &Arc<Mutex<RedbStorage>>) -> parking_lot::MutexGuard<RedbStorage> {
    storage.lock()
}

// ==================== 算法上下文测试 ====================

#[tokio::test]
async fn test_algorithm_context_creation() {
    // 测试算法上下文创建
    let context = AlgorithmContext::new()
        .with_max_depth(Some(10))
        .with_limit(100)
        .with_single_shortest(true)
        .with_no_loop(true);

    assert_eq!(context.max_depth, Some(10));
    assert_eq!(context.limit, 100);
    assert!(context.single_shortest);
    assert!(context.no_loop);
}

#[tokio::test]
async fn test_algorithm_context_default() {
    let context = AlgorithmContext::new();
    
    assert_eq!(context.max_depth, None);
    assert_eq!(context.limit, usize::MAX);
    assert!(!context.single_shortest);
    assert!(context.no_loop);
}

#[tokio::test]
async fn test_algorithm_stats() {
    let mut stats = AlgorithmStats::new();
    
    assert_eq!(stats.nodes_visited, 0);
    assert_eq!(stats.edges_traversed, 0);
    assert_eq!(stats.execution_time_ms, 0);
    
    stats.nodes_visited = 100;
    stats.edges_traversed = 200;
    stats.execution_time_ms = 50;
    
    assert_eq!(stats.nodes_visited, 100);
    assert_eq!(stats.edges_traversed, 200);
    assert_eq!(stats.execution_time_ms, 50);
}

// ==================== 多源最短路径执行器测试 ====================

#[tokio::test]
async fn test_multi_shortest_path_executor_creation() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    let executor = MultiShortestPathExecutor::new(
        1,
        storage.clone(),
        vec![Value::from("alice")],
        vec![Value::from("bob")],
        ExecEdgeDirection::Out,
        None,
        10,
    );

    assert_eq!(executor.id(), 1);
    assert_eq!(executor.name(), "MultiShortestPathExecutor");
    assert!(executor.description().contains("shortest path"));
}

#[tokio::test]
async fn test_multi_shortest_path_with_edge_filter() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    let executor = MultiShortestPathExecutor::new(
        1,
        storage.clone(),
        vec![Value::from("alice")],
        vec![Value::from("bob")],
        ExecEdgeDirection::Out,
        Some(vec!["KNOWS".to_string()]),
        10,
    );

    assert_eq!(executor.id(), 1);
    // 验证执行器创建成功，带边类型过滤
}

#[tokio::test]
async fn test_multi_shortest_path_bidirectional_direction() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    let executor = MultiShortestPathExecutor::new(
        1,
        storage.clone(),
        vec![Value::from("alice")],
        vec![Value::from("bob")],
        ExecEdgeDirection::Both,
        None,
        10,
    );

    assert_eq!(executor.id(), 1);
    // 验证双向边方向设置成功
}

// ==================== 子图查询执行器测试 ====================

#[tokio::test]
async fn test_subgraph_config_default() {
    let config = SubgraphConfig::default();
    
    assert_eq!(config.steps, 1);
    assert_eq!(config.edge_direction, ExecEdgeDirection::Out);
    assert!(config.edge_types.is_none());
    assert!(config.limit.is_none());
    assert!(config.with_properties);
}

#[tokio::test]
async fn test_subgraph_config_builder() {
    let config = SubgraphConfig::new(3)
        .with_direction(ExecEdgeDirection::Both)
        .with_edge_types(vec!["KNOWS".to_string(), "FRIEND".to_string()])
        .with_limit(100);

    assert_eq!(config.steps, 3);
    assert_eq!(config.edge_direction, ExecEdgeDirection::Both);
    assert_eq!(config.edge_types, Some(vec!["KNOWS".to_string(), "FRIEND".to_string()]));
    assert_eq!(config.limit, Some(100));
}

#[tokio::test]
async fn test_subgraph_executor_creation() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    let config = SubgraphConfig::new(2);

    let executor = SubgraphExecutor::new(
        1,
        storage.clone(),
        vec![Value::from("alice")],
        config,
    );

    assert_eq!(executor.id(), 1);
    assert_eq!(executor.name(), "SubgraphExecutor");
    assert!(executor.description().contains("subgraph"));
}

#[tokio::test]
async fn test_subgraph_executor_multiple_start_vids() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    let config = SubgraphConfig::new(2);

    let executor = SubgraphExecutor::new(
        1,
        storage.clone(),
        vec![Value::from("alice"), Value::from("bob"), Value::from("charlie")],
        config,
    );

    assert_eq!(executor.id(), 1);
    // 验证多起点创建成功
}

// ==================== 路径数据结构测试 ====================

#[tokio::test]
async fn test_path_creation() {
    let vertex = Vertex::with_vid(Value::from("A"));
    let path = Path::new(vertex.clone());
    
    assert_eq!(path.src.vid, Box::new(Value::from("A")));
    assert!(path.steps.is_empty());
}

#[tokio::test]
async fn test_path_with_steps() {
    let src = Vertex::with_vid(Value::from("A"));
    let dst = Vertex::with_vid(Value::from("B"));
    
    let mut path = Path::new(src);
    path.steps.push(Step::new(
        dst,
        "KNOWS".to_string(),
        "KNOWS".to_string(),
        0,
    ));
    
    assert_eq!(path.steps.len(), 1);
    assert_eq!(path.steps[0].edge.edge_type, "KNOWS");
}

#[tokio::test]
async fn test_vertex_with_vid() {
    let vertex = Vertex::with_vid(Value::from("test_id"));
    
    assert_eq!(vertex.vid, Box::new(Value::from("test_id")));
    assert!(vertex.tags.is_empty());
    assert!(vertex.properties.is_empty());
}

#[tokio::test]
async fn test_vertex_with_tags() {
    let tag = Tag::new("Person".to_string(), [
        ("name".to_string(), Value::from("Alice")),
    ].iter().cloned().collect());
    
    let vertex = Vertex::new(Value::from("alice"), vec![tag]);
    
    assert_eq!(vertex.vid, Box::new(Value::from("alice")));
    assert_eq!(vertex.tags.len(), 1);
    assert_eq!(vertex.tags[0].name, "Person");
}

// ==================== 边数据结构测试 ====================

#[tokio::test]
async fn test_edge_creation() {
    let edge = Edge::new(
        Value::from("A"),
        Value::from("B"),
        "KNOWS".to_string(),
        0,
        HashMap::new(),
    );
    
    assert_eq!(edge.src, Box::new(Value::from("A")));
    assert_eq!(edge.dst, Box::new(Value::from("B")));
    assert_eq!(edge.edge_type, "KNOWS");
    assert_eq!(edge.ranking, 0);
}

#[tokio::test]
async fn test_edge_with_properties() {
    let mut props = HashMap::new();
    props.insert("since".to_string(), Value::from("2020-01-01"));
    
    let edge = Edge::new(
        Value::from("A"),
        Value::from("B"),
        "KNOWS".to_string(),
        1,
        props,
    );
    
    assert_eq!(edge.ranking, 1);
    assert!(edge.props.contains_key("since"));
}

// ==================== 边界条件测试 ====================

#[tokio::test]
async fn test_multi_shortest_path_empty_start() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    let executor = MultiShortestPathExecutor::new(
        1,
        storage.clone(),
        vec![],  // 空起点
        vec![Value::from("bob")],
        ExecEdgeDirection::Out,
        None,
        10,
    );

    assert_eq!(executor.id(), 1);
    // 验证空起点情况下执行器仍能创建
}

#[tokio::test]
async fn test_multi_shortest_path_empty_end() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    let executor = MultiShortestPathExecutor::new(
        1,
        storage.clone(),
        vec![Value::from("alice")],
        vec![],  // 空终点
        ExecEdgeDirection::Out,
        None,
        10,
    );

    assert_eq!(executor.id(), 1);
    // 验证空终点情况下执行器仍能创建
}

#[tokio::test]
async fn test_subgraph_empty_start() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    let config = SubgraphConfig::new(2);

    let executor = SubgraphExecutor::new(
        1,
        storage.clone(),
        vec![],  // 空起点
        config,
    );

    assert_eq!(executor.id(), 1);
    // 验证空起点情况下执行器仍能创建
}

#[tokio::test]
async fn test_subgraph_zero_steps() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    let config = SubgraphConfig::new(0);  // 0步

    let executor = SubgraphExecutor::new(
        1,
        storage.clone(),
        vec![Value::from("alice")],
        config,
    );

    assert_eq!(executor.id(), 1);
    // 验证0步配置下执行器仍能创建
}

#[tokio::test]
async fn test_algorithm_context_with_zero_limit() {
    let context = AlgorithmContext::new()
        .with_limit(0);
    
    assert_eq!(context.limit, 0);
}

#[tokio::test]
async fn test_algorithm_context_with_max_depth_zero() {
    let context = AlgorithmContext::new()
        .with_max_depth(Some(0));

    assert_eq!(context.max_depth, Some(0));
}

// ==================== 带权最短路径集成测试 ====================

#[tokio::test]
async fn test_weighted_shortest_path_executor_creation() {
    use graphdb::query::executor::data_processing::graph_traversal::ShortestPathExecutor;
    use graphdb::query::executor::data_processing::graph_traversal::algorithms::{
        EdgeWeightConfig, ShortestPathAlgorithmType
    };

    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    // 创建带权最短路径执行器
    let executor = ShortestPathExecutor::new(
        100,
        storage.clone(),
        vec![Value::from("A")],
        vec![Value::from("C")],
        ExecEdgeDirection::Out,
        Some(vec!["connect".to_string()]),
        Some(10),
        ShortestPathAlgorithmType::Dijkstra,
    )
    .with_weight_config(EdgeWeightConfig::Property("weight".to_string()));

    assert_eq!(executor.id(), 100);
    assert_eq!(executor.name(), "ShortestPathExecutor");
}

#[tokio::test]
async fn test_weighted_shortest_path_with_ranking() {
    use graphdb::query::executor::data_processing::graph_traversal::ShortestPathExecutor;
    use graphdb::query::executor::data_processing::graph_traversal::algorithms::{
        EdgeWeightConfig, ShortestPathAlgorithmType
    };

    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    // 使用ranking作为权重
    let executor = ShortestPathExecutor::new(
        101,
        storage.clone(),
        vec![Value::from("A")],
        vec![Value::from("C")],
        ExecEdgeDirection::Out,
        None,
        Some(5),
        ShortestPathAlgorithmType::Dijkstra,
    )
    .with_weight_config(EdgeWeightConfig::Ranking);

    assert_eq!(executor.id(), 101);
}

#[tokio::test]
async fn test_weighted_shortest_path_astar() {
    use graphdb::query::executor::data_processing::graph_traversal::ShortestPathExecutor;
    use graphdb::query::executor::data_processing::graph_traversal::algorithms::{
        EdgeWeightConfig, HeuristicFunction, ShortestPathAlgorithmType
    };

    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    // 使用A*算法，带启发式函数
    let executor = ShortestPathExecutor::new(
        102,
        storage.clone(),
        vec![Value::from("A")],
        vec![Value::from("C")],
        ExecEdgeDirection::Out,
        None,
        Some(10),
        ShortestPathAlgorithmType::AStar,
    )
    .with_weight_config(EdgeWeightConfig::Property("weight".to_string()))
    .with_heuristic_config(HeuristicFunction::Zero);

    assert_eq!(executor.id(), 102);
}

#[tokio::test]
async fn test_weighted_path_query_parser_integration() {
    use graphdb::query::parser::parser::Parser;

    // 测试带权路径查询语句解析
    let query = "FIND SHORTEST PATH FROM 1 TO 2 OVER connect WEIGHT weight";
    let mut parser = Parser::new(query);
    let result = parser.parse();

    assert!(result.is_ok(), "带权路径查询解析应该成功: {:?}", result.err());

    let stmt = result.expect("解析应该成功");
    assert_eq!(stmt.kind(), "FIND PATH");
}

#[tokio::test]
async fn test_weighted_path_query_with_ranking_parser() {
    use graphdb::query::parser::parser::Parser;

    // 测试使用ranking作为权重的查询语句解析
    let query = "FIND SHORTEST PATH FROM 1 TO 2 OVER connect WEIGHT ranking";
    let mut parser = Parser::new(query);
    let result = parser.parse();

    assert!(result.is_ok(), "使用ranking权重的路径查询解析应该成功: {:?}", result.err());
}

#[tokio::test]
async fn test_unweighted_path_query_parser() {
    use graphdb::query::parser::parser::Parser;

    // 测试无权路径查询语句解析
    let query = "FIND SHORTEST PATH FROM 1 TO 2 OVER connect";
    let mut parser = Parser::new(query);
    let result = parser.parse();

    assert!(result.is_ok(), "无权路径查询解析应该成功: {:?}", result.err());
}
