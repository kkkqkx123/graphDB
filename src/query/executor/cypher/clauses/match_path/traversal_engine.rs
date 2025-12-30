//! 图遍历引擎模块
//!
//! 提供路径遍历算法，邻居查找和扩展，循环检测和限制

use crate::core::error::DBError;
use crate::core::{Direction, Edge, Value, Vertex};
use crate::query::executor::cypher::clauses::match_path::path_info::PathInfo;
use crate::query::executor::cypher::clauses::match_path::pattern_matcher::PatternMatcher;
use crate::query::executor::cypher::context::CypherExecutionContext;
use crate::query::parser::cypher::ast::patterns::RelationshipPattern;
use crate::storage::StorageEngine;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

/// 图遍历引擎
#[derive(Debug)]
pub struct TraversalEngine<S: StorageEngine> {
    storage: Arc<Mutex<S>>,
    pattern_matcher: PatternMatcher<S>,
    /// 已访问的节点集合，避免循环
    visited_vertices: HashSet<Value>,
    /// 最大路径长度限制
    max_path_length: usize,
}

impl<S: StorageEngine> TraversalEngine<S> {
    /// 创建新的图遍历引擎
    pub fn new(storage: Arc<Mutex<S>>) -> Self {
        Self {
            pattern_matcher: PatternMatcher::new(storage.clone()),
            storage,
            visited_vertices: HashSet::new(),
            max_path_length: 1000,
        }
    }

    /// 设置最大路径长度限制
    pub fn set_max_path_length(&mut self, max_length: usize) {
        self.max_path_length = max_length;
    }

    /// 重置遍历状态
    pub fn reset(&mut self) {
        self.visited_vertices.clear();
    }

    /// 使用关系模式扩展路径
    pub async fn expand_with_relationship(
        &mut self,
        current_paths: &[PathInfo],
        rel_pattern: &RelationshipPattern,
        context: &mut CypherExecutionContext,
    ) -> Result<Vec<PathInfo>, DBError> {
        let mut new_paths = Vec::new();

        for path in current_paths {
            if let Some(last_vertex) = path.last_vertex() {
                let expanded_paths = self
                    .expand_path_from_vertex(path, last_vertex, rel_pattern, context)
                    .await?;

                new_paths.extend(expanded_paths);
            }
        }

        Ok(new_paths)
    }

    /// 从指定节点扩展路径
    async fn expand_path_from_vertex(
        &mut self,
        current_path: &PathInfo,
        vertex: &Vertex,
        rel_pattern: &RelationshipPattern,
        context: &mut CypherExecutionContext,
    ) -> Result<Vec<PathInfo>, DBError> {
        // 检查路径长度是否超过限制（防止无限循环）
        if current_path.length > self.max_path_length {
            return Err(DBError::Query(
                crate::core::error::QueryError::ExecutionError(
                    "路径长度超过限制，可能存在循环".to_string(),
                ),
            ));
        }

        let storage = self.storage.lock().map_err(|e| {
            DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                "存储引擎锁定失败: {}",
                e
            )))
        })?;

        // 获取节点的邻居边
        let direction = self.pattern_direction_to_storage_direction(&rel_pattern.direction);
        let edges = storage
            .get_node_edges(&vertex.vid, direction)
            .map_err(|e| {
                DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                    "获取邻居边失败: {}",
                    e
                )))
            })?;

        // 如果没有边，直接返回空结果
        if edges.is_empty() {
            return Ok(Vec::new());
        }

        // 按关系类型过滤
        let filtered_edges = if !rel_pattern.types.is_empty() {
            self.pattern_matcher
                .filter_edges_by_types(edges, &rel_pattern.types)
        } else {
            edges
        };

        // 按属性过滤
        let filtered_edges = if let Some(properties) = &rel_pattern.properties {
            self.pattern_matcher
                .filter_edges_by_properties(filtered_edges, properties, context)?
        } else {
            filtered_edges
        };

        // 如果过滤后没有边，直接返回空结果
        if filtered_edges.is_empty() {
            return Ok(Vec::new());
        }

        // 构建新路径
        let mut new_paths = Vec::new();
        for edge in filtered_edges {
            // 检查是否会产生重复边
            if current_path.has_duplicate_edge(&edge) {
                continue;
            }

            // 获取目标节点
            let target_vertex_id = self.get_target_vertex_id(vertex, &edge, &rel_pattern.direction);

            // 检查是否已经访问过该节点（防止循环）
            if self.visited_vertices.contains(&target_vertex_id) {
                continue;
            }

            let target_vertex = storage.get_node(&target_vertex_id).map_err(|e| {
                DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                    "获取目标节点失败: {}",
                    e
                )))
            })?;

            if let Some(target_vertex) = target_vertex {
                // 创建新路径
                let mut new_path = current_path.clone();
                new_path.add_edge(edge.clone());
                new_path.add_vertex(target_vertex.clone());

                // 如果关系有变量名，存储到上下文
                if let Some(var_name) = &rel_pattern.variable {
                    context.set_variable_value(var_name, Value::Edge(edge.clone()));
                }

                new_paths.push(new_path);

                // 标记节点为已访问
                self.visited_vertices.insert(target_vertex_id);
            }
        }

        Ok(new_paths)
    }

    /// 将模式方向转换为存储方向
    fn pattern_direction_to_storage_direction(
        &self,
        pattern_dir: &crate::query::parser::cypher::ast::patterns::Direction,
    ) -> Direction {
        match pattern_dir {
            crate::query::parser::cypher::ast::patterns::Direction::Left => Direction::In,
            crate::query::parser::cypher::ast::patterns::Direction::Right => Direction::Out,
            crate::query::parser::cypher::ast::patterns::Direction::Both => Direction::Both,
        }
    }

    /// 获取目标节点ID
    fn get_target_vertex_id(
        &self,
        source_vertex: &Vertex,
        edge: &Edge,
        direction: &crate::query::parser::cypher::ast::patterns::Direction,
    ) -> Value {
        use crate::query::parser::cypher::ast::patterns::Direction as PatternDirection;
        match direction {
            PatternDirection::Right => {
                // 出边，目标是边的dst
                edge.dst().clone()
            }
            PatternDirection::Left => {
                // 入边，目标是边的src
                edge.src().clone()
            }
            PatternDirection::Both => {
                // 双向，选择不是源节点的另一端
                if edge.src() == source_vertex.vid() {
                    edge.dst().clone()
                } else {
                    edge.src().clone()
                }
            }
        }
    }

    /// 检查节点是否已访问
    pub fn is_vertex_visited(&self, vertex_id: &Value) -> bool {
        self.visited_vertices.contains(vertex_id)
    }

    /// 标记节点为已访问
    pub fn mark_vertex_visited(&mut self, vertex_id: Value) {
        self.visited_vertices.insert(vertex_id);
    }

    /// 取消标记节点为已访问
    pub fn unmark_vertex_visited(&mut self, vertex_id: &Value) {
        self.visited_vertices.remove(vertex_id);
    }

    /// 获取已访问节点的数量
    pub fn visited_count(&self) -> usize {
        self.visited_vertices.len()
    }

    /// 清空已访问节点集合
    pub fn clear_visited(&mut self) {
        self.visited_vertices.clear();
    }

    /// 检查路径是否包含循环
    pub fn has_cycle(&self, path: &PathInfo) -> bool {
        let mut seen = HashSet::new();
        for vertex in &path.vertices {
            if seen.contains(vertex.vid()) {
                return true;
            }
            seen.insert(vertex.vid().clone());
        }
        false
    }

    /// 获取路径中的唯一节点数量
    pub fn unique_vertex_count(&self, path: &PathInfo) -> usize {
        let mut unique_ids = HashSet::new();
        for vertex in &path.vertices {
            unique_ids.insert(vertex.vid());
        }
        unique_ids.len()
    }

    /// 获取路径中的唯一边数量
    pub fn unique_edge_count(&self, path: &PathInfo) -> usize {
        let mut unique_edges = HashSet::new();
        for edge in &path.edges {
            let edge_key = (
                edge.src().clone(),
                edge.dst().clone(),
                edge.edge_type().to_string(),
            );
            unique_edges.insert(edge_key);
        }
        unique_edges.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::vertex_edge_path::Tag;
    use crate::query::parser::cypher::ast::patterns::Direction as PatternDirection;

    #[test]
    fn test_pattern_direction_to_storage_direction() {
        let storage = Arc::new(Mutex::new(
            crate::storage::native_storage::NativeStorage::new("data/tests/test_db")
                .expect("Failed to create test storage"),
        ));
        let engine = TraversalEngine::new(storage);

        assert!(matches!(
            engine.pattern_direction_to_storage_direction(&PatternDirection::Right),
            crate::core::Direction::Out
        ));
        assert!(matches!(
            engine.pattern_direction_to_storage_direction(&PatternDirection::Left),
            crate::core::Direction::In
        ));
        assert!(matches!(
            engine.pattern_direction_to_storage_direction(&PatternDirection::Both),
            crate::core::Direction::Both
        ));
    }

    #[test]
    fn test_get_target_vertex_id() {
        let storage = Arc::new(Mutex::new(
            crate::storage::native_storage::NativeStorage::new("data/tests/test_db")
                .expect("Failed to create test storage"),
        ));
        let engine = TraversalEngine::new(storage);

        let vertex1 = Vertex::new(Value::String("v1".to_string()), vec![]);
        let vertex2 = Vertex::new(Value::String("v2".to_string()), vec![]);

        let edge = Edge::new(
            Value::String("v1".to_string()),
            Value::String("v2".to_string()),
            "KNOWS".to_string(),
            0,
            std::collections::HashMap::new(),
        );

        // 右向边，目标是dst
        let target = engine.get_target_vertex_id(&vertex1, &edge, &PatternDirection::Right);
        assert_eq!(target, Value::String("v2".to_string()));

        // 左向边，目标是src
        let target = engine.get_target_vertex_id(&vertex2, &edge, &PatternDirection::Left);
        assert_eq!(target, Value::String("v1".to_string()));

        // 双向边，选择不是源节点的另一端
        let target = engine.get_target_vertex_id(&vertex1, &edge, &PatternDirection::Both);
        assert_eq!(target, Value::String("v2".to_string()));
    }

    #[test]
    fn test_visited_vertices_management() {
        let storage = Arc::new(Mutex::new(
            crate::storage::native_storage::NativeStorage::new("data/tests/test_db")
                .expect("Failed to create test storage"),
        ));
        let mut engine = TraversalEngine::new(storage);

        let vertex_id = Value::String("v1".to_string());

        // 初始状态
        assert!(!engine.is_vertex_visited(&vertex_id));
        assert_eq!(engine.visited_count(), 0);

        // 标记为已访问
        engine.mark_vertex_visited(vertex_id.clone());
        assert!(engine.is_vertex_visited(&vertex_id));
        assert_eq!(engine.visited_count(), 1);

        // 取消标记
        engine.unmark_vertex_visited(&vertex_id);
        assert!(!engine.is_vertex_visited(&vertex_id));
        assert_eq!(engine.visited_count(), 0);

        // 清空所有访问标记
        engine.mark_vertex_visited(Value::String("v1".to_string()));
        engine.mark_vertex_visited(Value::String("v2".to_string()));
        assert_eq!(engine.visited_count(), 2);

        engine.clear_visited();
        assert_eq!(engine.visited_count(), 0);
    }

    #[test]
    fn test_cycle_detection() {
        let storage = Arc::new(Mutex::new(
            crate::storage::native_storage::NativeStorage::new("data/tests/test_db")
                .expect("Failed to create test storage"),
        ));
        let engine = TraversalEngine::new(storage);

        let tag = Tag::new("Person".to_string(), std::collections::HashMap::new());
        let vertex1 = Vertex::new(Value::String("v1".to_string()), vec![tag.clone()]);
        let vertex2 = Vertex::new(Value::String("v2".to_string()), vec![tag.clone()]);
        let vertex3 = Vertex::new(Value::String("v3".to_string()), vec![tag]);

        // 创建无循环的路径
        let mut path_no_cycle = PathInfo::new();
        path_no_cycle.add_vertex(vertex1.clone());
        path_no_cycle.add_vertex(vertex2.clone());
        path_no_cycle.add_vertex(vertex3.clone());

        assert!(!engine.has_cycle(&path_no_cycle));

        // 创建有循环的路径
        let mut path_with_cycle = PathInfo::new();
        path_with_cycle.add_vertex(vertex1.clone());
        path_with_cycle.add_vertex(vertex2.clone());
        path_with_cycle.add_vertex(vertex3.clone());
        path_with_cycle.add_vertex(vertex1.clone()); // 回到起点，形成循环

        assert!(engine.has_cycle(&path_with_cycle));
    }

    #[test]
    fn test_unique_count() {
        let storage = Arc::new(Mutex::new(
            crate::storage::native_storage::NativeStorage::new("data/tests/test_db")
                .expect("Failed to create test storage"),
        ));
        let engine = TraversalEngine::new(storage);

        let tag = Tag::new("Person".to_string(), std::collections::HashMap::new());
        let vertex1 = Vertex::new(Value::String("v1".to_string()), vec![tag.clone()]);
        let vertex2 = Vertex::new(Value::String("v2".to_string()), vec![tag.clone()]);
        let vertex3 = Vertex::new(Value::String("v3".to_string()), vec![tag]);

        let edge1 = Edge::new(
            Value::String("v1".to_string()),
            Value::String("v2".to_string()),
            "KNOWS".to_string(),
            0,
            std::collections::HashMap::new(),
        );
        let edge2 = Edge::new(
            Value::String("v2".to_string()),
            Value::String("v3".to_string()),
            "KNOWS".to_string(),
            0,
            std::collections::HashMap::new(),
        );
        let edge3 = Edge::new(
            Value::String("v1".to_string()),
            Value::String("v2".to_string()),
            "KNOWS".to_string(),
            0,
            std::collections::HashMap::new(),
        );

        // 创建路径
        let mut path = PathInfo::new();
        path.add_vertex(vertex1.clone());
        path.add_vertex(vertex2.clone());
        path.add_vertex(vertex3.clone());
        path.add_edge(edge1.clone());
        path.add_edge(edge2.clone());
        path.add_edge(edge3.clone()); // 重复的边

        assert_eq!(engine.unique_vertex_count(&path), 3);
        assert_eq!(engine.unique_edge_count(&path), 2); // edge1 和 edge3 是相同的
    }

    #[test]
    fn test_max_path_length() {
        let storage = Arc::new(Mutex::new(
            crate::storage::native_storage::NativeStorage::new("data/tests/test_db")
                .expect("Failed to create test storage"),
        ));
        let mut engine = TraversalEngine::new(storage);

        assert_eq!(engine.max_path_length, 1000);

        engine.set_max_path_length(500);
        assert_eq!(engine.max_path_length, 500);
    }

    #[test]
    fn test_reset() {
        let storage = Arc::new(Mutex::new(
            crate::storage::native_storage::NativeStorage::new("data/tests/test_db")
                .expect("Failed to create test storage"),
        ));
        let mut engine = TraversalEngine::new(storage);

        // 添加一些访问标记
        engine.mark_vertex_visited(Value::String("v1".to_string()));
        engine.mark_vertex_visited(Value::String("v2".to_string()));
        assert_eq!(engine.visited_count(), 2);

        // 重置
        engine.reset();
        assert_eq!(engine.visited_count(), 0);
    }
}
