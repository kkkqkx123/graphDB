//! 测试数据生成模块
//!
//! 提供各种测试数据的生成函数

use graphdb::core::vertex_edge_path::{Tag, Vertex, Edge};
use graphdb::core::Value;
use std::collections::HashMap;

/// 创建人员标签
pub fn person_tag(name: &str, age: i64) -> Tag {
    let mut props = HashMap::new();
    props.insert("name".to_string(), Value::String(name.to_string()));
    props.insert("age".to_string(), Value::Int(age));
    Tag::new("Person".to_string(), props)
}

/// 创建公司标签
pub fn company_tag(name: &str, founded: i64) -> Tag {
    let mut props = HashMap::new();
    props.insert("name".to_string(), Value::String(name.to_string()));
    props.insert("founded".to_string(), Value::Int(founded));
    Tag::new("Company".to_string(), props)
}

/// 创建简单顶点（只有一个标签）
pub fn create_simple_vertex(vid: i64, _tag_name: &str, name: &str, age: i64) -> Vertex {
    let tag = person_tag(name, age);
    create_vertex(Value::Int(vid), vec![tag])
}

/// 创建顶点
pub fn create_vertex(vid: Value, tags: Vec<Tag>) -> Vertex {
    Vertex::new(vid, tags)
}

/// 创建边
pub fn create_edge(src: Value, dst: Value, edge_type: &str) -> Edge {
    Edge::new(src, dst, edge_type.to_string(), 0, HashMap::new())
}

/// 创建带属性的边
pub fn create_edge_with_props(
    src: Value,
    dst: Value,
    edge_type: &str,
    props: HashMap<String, Value>,
) -> Edge {
    Edge::new(src, dst, edge_type.to_string(), 0, props)
}

/// 社交网络测试数据集
/// 返回 (顶点列表, 边列表)
pub fn social_network_dataset() -> (Vec<Vertex>, Vec<Edge>) {
    // 创建4个Person顶点
    let vertices = vec![
        create_simple_vertex(1, "Person", "Alice", 30),
        create_simple_vertex(2, "Person", "Bob", 25),
        create_simple_vertex(3, "Person", "Charlie", 35),
        create_simple_vertex(4, "Person", "David", 28),
    ];

    // 创建KNOWS关系边
    let edges = vec![
        create_edge(Value::Int(1), Value::Int(2), "KNOWS"),
        create_edge(Value::Int(1), Value::Int(3), "KNOWS"),
        create_edge(Value::Int(2), Value::Int(3), "KNOWS"),
        create_edge(Value::Int(3), Value::Int(4), "KNOWS"),
    ];

    (vertices, edges)
}

/// 生成指定数量的测试顶点
pub fn generate_test_vertices(count: usize) -> Vec<Vertex> {
    (1..=count)
        .map(|i| create_simple_vertex(i as i64, "Person", &format!("User{}", i), (20 + i) as i64))
        .collect()
}

/// 生成链式边（1->2->3->...->n）
pub fn generate_chain_edges(count: usize, edge_type: &str) -> Vec<Edge> {
    (1..count)
        .map(|i| create_edge(Value::Int(i as i64), Value::Int((i + 1) as i64), edge_type))
        .collect()
}

/// 生成星形边（中心节点连接到所有其他节点）
pub fn generate_star_edges(center: i64, others: &[i64], edge_type: &str) -> Vec<Edge> {
    others
        .iter()
        .map(|&other| create_edge(Value::Int(center), Value::Int(other), edge_type))
        .collect()
}
