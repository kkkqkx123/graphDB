//! 测试数据生成模块
//!
//! 提供各种测试数据的生成函数

#![allow(dead_code)]

use graphdb::core::vertex_edge_path::{Edge, Tag, Vertex};
use graphdb::core::Value;
use std::collections::HashMap;

/// 创建简单顶点（只有一个标签）
pub fn create_simple_vertex(vid: i64, _tag_name: &str, name: &str, age: i64) -> Vertex {
    let mut props = HashMap::new();
    props.insert("name".to_string(), Value::String(name.to_string()));
    props.insert("age".to_string(), Value::Int(age));
    let tag = Tag::new("Person".to_string(), props);
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
