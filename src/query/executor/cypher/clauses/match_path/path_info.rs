//! 路径信息管理模块
//!
//! 提供路径信息的结构体和相关操作方法

use crate::core::{Edge, Vertex};

/// 路径信息
#[derive(Debug, Clone)]
pub struct PathInfo {
    /// 路径中的节点序列
    pub vertices: Vec<Vertex>,
    /// 路径中的边序列
    pub edges: Vec<Edge>,
    /// 当前路径长度
    pub length: usize,
}

impl PathInfo {
    /// 创建新的路径
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            edges: Vec::new(),
            length: 0,
        }
    }

    /// 添加节点到路径
    pub fn add_vertex(&mut self, vertex: Vertex) {
        self.vertices.push(vertex);
        if self.vertices.len() > 1 {
            self.length = self.vertices.len() - 1;
        }
    }

    /// 添加边到路径
    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
        self.length = self.edges.len();
    }

    /// 获取最后一个节点
    pub fn last_vertex(&self) -> Option<&Vertex> {
        self.vertices.last()
    }

    /// 检查路径是否包含重复边
    pub fn has_duplicate_edge(&self, edge: &Edge) -> bool {
        self.edges.iter().any(|e| {
            e.src() == edge.src() && e.dst() == edge.dst() && e.edge_type() == edge.edge_type()
        })
    }

    /// 检查路径是否为空
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    /// 获取路径中的节点数量
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// 获取路径中的边数量
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// 清空路径
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.edges.clear();
        self.length = 0;
    }

    /// 检查路径是否包含指定节点
    pub fn contains_vertex(&self, vertex_id: &crate::core::Value) -> bool {
        self.vertices.iter().any(|v| v.vid() == vertex_id)
    }
}

impl Default for PathInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::vertex_edge_path::Tag;
    use std::collections::HashMap;

    #[test]
    fn test_path_info_creation() {
        let path = PathInfo::new();
        assert!(path.is_empty());
        assert_eq!(path.length, 0);
        assert_eq!(path.vertex_count(), 0);
        assert_eq!(path.edge_count(), 0);
    }

    #[test]
    fn test_add_vertex() {
        let mut path = PathInfo::new();

        let tag = Tag::new("Person".to_string(), HashMap::new());
        let vertex = Vertex::new(crate::core::Value::String("v1".to_string()), vec![tag]);

        path.add_vertex(vertex.clone());
        assert!(!path.is_empty());
        assert_eq!(path.length, 0);
        assert_eq!(path.vertex_count(), 1);
        assert_eq!(path.edge_count(), 0);
        assert_eq!(path.last_vertex(), Some(&vertex));
    }

    #[test]
    fn test_add_multiple_vertices() {
        let mut path = PathInfo::new();

        let tag1 = Tag::new("Person".to_string(), HashMap::new());
        let vertex1 = Vertex::new(crate::core::Value::String("v1".to_string()), vec![tag1]);

        let tag2 = Tag::new("Person".to_string(), HashMap::new());
        let vertex2 = Vertex::new(crate::core::Value::String("v2".to_string()), vec![tag2]);

        path.add_vertex(vertex1);
        path.add_vertex(vertex2);

        assert_eq!(path.length, 1);
        assert_eq!(path.vertex_count(), 2);
        assert_eq!(path.edge_count(), 0);
    }

    #[test]
    fn test_add_edge() {
        let mut path = PathInfo::new();

        let tag1 = Tag::new("Person".to_string(), HashMap::new());
        let vertex1 = Vertex::new(crate::core::Value::String("v1".to_string()), vec![tag1]);

        let tag2 = Tag::new("Person".to_string(), HashMap::new());
        let vertex2 = Vertex::new(crate::core::Value::String("v2".to_string()), vec![tag2]);

        let edge = Edge::new(
            crate::core::Value::String("v1".to_string()),
            crate::core::Value::String("v2".to_string()),
            "KNOWS".to_string(),
            0,
            HashMap::new(),
        );

        path.add_vertex(vertex1);
        path.add_vertex(vertex2);
        path.add_edge(edge.clone());

        assert_eq!(path.length, 1);
        assert_eq!(path.vertex_count(), 2);
        assert_eq!(path.edge_count(), 1);
    }

    #[test]
    fn test_has_duplicate_edge() {
        let mut path = PathInfo::new();

        let tag1 = Tag::new("Person".to_string(), HashMap::new());
        let vertex1 = Vertex::new(crate::core::Value::String("v1".to_string()), vec![tag1]);

        let tag2 = Tag::new("Person".to_string(), HashMap::new());
        let vertex2 = Vertex::new(crate::core::Value::String("v2".to_string()), vec![tag2]);

        let edge = Edge::new(
            crate::core::Value::String("v1".to_string()),
            crate::core::Value::String("v2".to_string()),
            "KNOWS".to_string(),
            0,
            HashMap::new(),
        );

        path.add_vertex(vertex1);
        path.add_vertex(vertex2);
        path.add_edge(edge.clone());

        assert!(!path.has_duplicate_edge(&edge));

        // 添加相同的边
        path.add_edge(edge.clone());
        assert!(path.has_duplicate_edge(&edge));
    }

    #[test]
    fn test_contains_vertex() {
        let mut path = PathInfo::new();

        let tag = Tag::new("Person".to_string(), HashMap::new());
        let vertex = Vertex::new(crate::core::Value::String("v1".to_string()), vec![tag]);

        path.add_vertex(vertex.clone());

        assert!(path.contains_vertex(&crate::core::Value::String("v1".to_string())));
        assert!(!path.contains_vertex(&crate::core::Value::String("v2".to_string())));
    }

    #[test]
    fn test_clear() {
        let mut path = PathInfo::new();

        let tag = Tag::new("Person".to_string(), HashMap::new());
        let vertex = Vertex::new(crate::core::Value::String("v1".to_string()), vec![tag]);

        path.add_vertex(vertex);
        assert!(!path.is_empty());

        path.clear();
        assert!(path.is_empty());
        assert_eq!(path.length, 0);
    }
}
