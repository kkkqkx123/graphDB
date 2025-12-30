//! 结果构建器模块
//!
//! 提供结果集构建和格式化，不同类型结果的转换

use crate::core::error::DBError;
use crate::core::{Edge, Value, Vertex};
use crate::query::executor::cypher::clauses::match_path::path_info::PathInfo;
use crate::query::executor::traits::ExecutionResult;

/// 结果构建器
#[derive(Debug)]
pub struct ResultBuilder {
    /// 最大结果数量限制
    max_result_count: usize,
}

impl ResultBuilder {
    /// 创建新的结果构建器
    pub fn new() -> Self {
        Self {
            max_result_count: 10000,
        }
    }

    /// 设置最大结果数量限制
    pub fn set_max_result_count(&mut self, max_count: usize) {
        self.max_result_count = max_count;
    }

    /// 构建结果集
    pub fn build_result(
        &mut self,
        current_paths: &[PathInfo],
        result_paths: &mut Vec<PathInfo>,
    ) -> Result<ExecutionResult, DBError> {
        // 将当前路径添加到结果路径
        result_paths.extend_from_slice(current_paths);

        // 根据结果类型返回不同的ExecutionResult
        if result_paths.is_empty() {
            return Ok(ExecutionResult::Success);
        }

        // 检查结果数量是否超过限制
        if result_paths.len() > self.max_result_count {
            return Err(DBError::Query(
                crate::core::error::QueryError::ExecutionError("结果集过大，超过限制".to_string()),
            ));
        }

        // 如果只有一个节点模式，返回顶点集合
        if result_paths
            .iter()
            .all(|p| p.vertices.len() == 1 && p.edges.is_empty())
        {
            let vertices: Vec<Vertex> = result_paths
                .iter()
                .flat_map(|p| p.vertices.clone())
                .collect();
            return Ok(ExecutionResult::Vertices(vertices));
        }

        // 如果只有边模式，返回边集合
        if result_paths
            .iter()
            .all(|p| p.vertices.len() == 2 && p.edges.len() == 1)
        {
            let edges: Vec<Edge> = result_paths.iter().flat_map(|p| p.edges.clone()).collect();
            return Ok(ExecutionResult::Edges(edges));
        }

        // 否则返回路径集合
        let paths = self.build_paths(result_paths)?;
        Ok(ExecutionResult::Paths(paths))
    }

    /// 构建路径集合
    fn build_paths(
        &self,
        result_paths: &[PathInfo],
    ) -> Result<Vec<crate::core::vertex_edge_path::Path>, DBError> {
        let paths: Vec<crate::core::vertex_edge_path::Path> = result_paths
            .iter()
            .filter_map(|p| {
                if !p.vertices.is_empty() {
                    let src = p.vertices[0].clone();
                    let mut steps = Vec::new();

                    for i in 0..p.edges.len() {
                        let dst = if i + 1 < p.vertices.len() {
                            p.vertices[i + 1].clone()
                        } else {
                            continue;
                        };

                        let step = crate::core::vertex_edge_path::Step {
                            dst: Box::new(dst),
                            edge: Box::new(p.edges[i].clone()),
                        };
                        steps.push(step);
                    }

                    Some(crate::core::vertex_edge_path::Path {
                        src: Box::new(src),
                        steps,
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(paths)
    }

    /// 构建顶点结果
    pub fn build_vertices_result(&self, paths: &[PathInfo]) -> Result<ExecutionResult, DBError> {
        let vertices: Vec<Vertex> = paths.iter().flat_map(|p| p.vertices.clone()).collect();

        Ok(ExecutionResult::Vertices(vertices))
    }

    /// 构建边结果
    pub fn build_edges_result(&self, paths: &[PathInfo]) -> Result<ExecutionResult, DBError> {
        let edges: Vec<Edge> = paths.iter().flat_map(|p| p.edges.clone()).collect();

        Ok(ExecutionResult::Edges(edges))
    }

    /// 构建值结果
    pub fn build_values_result(&self, values: Vec<Value>) -> Result<ExecutionResult, DBError> {
        if values.len() > self.max_result_count {
            return Err(DBError::Query(
                crate::core::error::QueryError::ExecutionError("结果集过大，超过限制".to_string()),
            ));
        }

        Ok(ExecutionResult::Values(values))
    }

    /// 构建计数结果
    pub fn build_count_result(&self, count: usize) -> Result<ExecutionResult, DBError> {
        Ok(ExecutionResult::Count(count))
    }

    /// 构建成功结果
    pub fn build_success_result(&self) -> Result<ExecutionResult, DBError> {
        Ok(ExecutionResult::Success)
    }

    /// 构建错误结果
    pub fn build_error_result(&self, error: String) -> Result<ExecutionResult, DBError> {
        Ok(ExecutionResult::Error(error))
    }

    /// 从路径中提取所有唯一的顶点
    pub fn extract_unique_vertices(&self, paths: &[PathInfo]) -> Vec<Vertex> {
        let mut unique_vertices = std::collections::HashMap::new();

        for path in paths {
            for vertex in &path.vertices {
                unique_vertices.insert(vertex.id().clone(), vertex.clone());
            }
        }

        unique_vertices.into_values().collect()
    }

    /// 从路径中提取所有唯一的边
    pub fn extract_unique_edges(&self, paths: &[PathInfo]) -> Vec<Edge> {
        let mut unique_edges = std::collections::HashMap::new();

        for path in paths {
            for edge in &path.edges {
                let edge_key = (
                    edge.src().clone(),
                    edge.dst().clone(),
                    edge.edge_type().to_string(),
                );
                unique_edges.insert(edge_key, edge.clone());
            }
        }

        unique_edges.into_values().collect()
    }

    /// 统计路径信息
    pub fn analyze_paths(&self, paths: &[PathInfo]) -> PathAnalysis {
        let mut analysis = PathAnalysis::new();

        analysis.total_paths = paths.len();
        analysis.empty_paths = paths.iter().filter(|p| p.is_empty()).count();

        if !paths.is_empty() {
            let path_lengths: Vec<usize> = paths.iter().map(|p| p.length).collect();
            analysis.min_path_length = *path_lengths
                .iter()
                .min()
                .expect("Path lengths should not be empty when paths is not empty");
            analysis.max_path_length = *path_lengths
                .iter()
                .max()
                .expect("Path lengths should not be empty when paths is not empty");
            analysis.avg_path_length =
                path_lengths.iter().sum::<usize>() as f64 / paths.len() as f64;
        }

        let vertex_counts: Vec<usize> = paths.iter().map(|p| p.vertex_count()).collect();
        if !vertex_counts.is_empty() {
            analysis.min_vertices = *vertex_counts
                .iter()
                .min()
                .expect("Vertex counts should not be empty when paths is not empty");
            analysis.max_vertices = *vertex_counts
                .iter()
                .max()
                .expect("Vertex counts should not be empty when paths is not empty");
            analysis.avg_vertices = vertex_counts.iter().sum::<usize>() as f64 / paths.len() as f64;
        }

        let edge_counts: Vec<usize> = paths.iter().map(|p| p.edge_count()).collect();
        if !edge_counts.is_empty() {
            analysis.min_edges = *edge_counts
                .iter()
                .min()
                .expect("Edge counts should not be empty when paths is not empty");
            analysis.max_edges = *edge_counts
                .iter()
                .max()
                .expect("Edge counts should not be empty when paths is not empty");
            analysis.avg_edges = edge_counts.iter().sum::<usize>() as f64 / paths.len() as f64;
        }

        analysis
    }
}

/// 路径分析结果
#[derive(Debug, Clone)]
pub struct PathAnalysis {
    /// 总路径数
    pub total_paths: usize,
    /// 空路径数
    pub empty_paths: usize,
    /// 最小路径长度
    pub min_path_length: usize,
    /// 最大路径长度
    pub max_path_length: usize,
    /// 平均路径长度
    pub avg_path_length: f64,
    /// 最小顶点数
    pub min_vertices: usize,
    /// 最大顶点数
    pub max_vertices: usize,
    /// 平均顶点数
    pub avg_vertices: f64,
    /// 最小边数
    pub min_edges: usize,
    /// 最大边数
    pub max_edges: usize,
    /// 平均边数
    pub avg_edges: f64,
}

impl PathAnalysis {
    /// 创建新的路径分析
    pub fn new() -> Self {
        Self {
            total_paths: 0,
            empty_paths: 0,
            min_path_length: 0,
            max_path_length: 0,
            avg_path_length: 0.0,
            min_vertices: 0,
            max_vertices: 0,
            avg_vertices: 0.0,
            min_edges: 0,
            max_edges: 0,
            avg_edges: 0.0,
        }
    }
}

impl Default for ResultBuilder {
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
    fn test_result_builder_creation() {
        let builder = ResultBuilder::new();
        assert_eq!(builder.max_result_count, 10000);
    }

    #[test]
    fn test_set_max_result_count() {
        let mut builder = ResultBuilder::new();
        builder.set_max_result_count(5000);
        assert_eq!(builder.max_result_count, 5000);
    }

    #[test]
    fn test_build_success_result() {
        let builder = ResultBuilder::new();
        let result = builder
            .build_success_result()
            .expect("Failed to build success result");
        assert!(matches!(result, ExecutionResult::Success));
    }

    #[test]
    fn test_build_count_result() {
        let builder = ResultBuilder::new();
        let result = builder
            .build_count_result(42)
            .expect("Failed to build count result");
        assert!(matches!(result, ExecutionResult::Count(42)));
    }

    #[test]
    fn test_build_error_result() {
        let builder = ResultBuilder::new();
        let error_msg = "Test error".to_string();
        let result = builder
            .build_error_result(error_msg.clone())
            .expect("Failed to build error result");

        if let ExecutionResult::Error(msg) = result {
            assert_eq!(msg, error_msg);
        } else {
            panic!("Expected error result");
        }
    }

    #[test]
    fn test_build_values_result() {
        let builder = ResultBuilder::new();
        let values = vec![
            Value::String("test1".to_string()),
            Value::String("test2".to_string()),
            Value::Int(42),
        ];

        let result = builder
            .build_values_result(values.clone())
            .expect("Failed to build values result");

        if let ExecutionResult::Values(result_values) = result {
            assert_eq!(result_values, values);
        } else {
            panic!("Expected values result");
        }
    }

    #[test]
    fn test_build_values_result_too_large() {
        let mut builder = ResultBuilder::new();
        builder.set_max_result_count(2);

        let values = vec![
            Value::String("test1".to_string()),
            Value::String("test2".to_string()),
            Value::Int(42),
        ];

        let result = builder.build_values_result(values);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_unique_vertices() {
        let builder = ResultBuilder::new();

        let tag = Tag::new("Person".to_string(), HashMap::new());
        let vertex1 = Vertex::new(Value::String("v1".to_string()), vec![tag.clone()]);
        let vertex2 = Vertex::new(Value::String("v2".to_string()), vec![tag.clone()]);
        let vertex3 = Vertex::new(Value::String("v3".to_string()), vec![tag]);

        // 创建包含重复顶点的路径
        let mut path1 = PathInfo::new();
        path1.add_vertex(vertex1.clone());
        path1.add_vertex(vertex2.clone());

        let mut path2 = PathInfo::new();
        path2.add_vertex(vertex2.clone()); // 重复
        path2.add_vertex(vertex3.clone());

        let paths = vec![path1, path2];
        let unique_vertices = builder.extract_unique_vertices(&paths);

        assert_eq!(unique_vertices.len(), 3);
        assert!(unique_vertices
            .iter()
            .any(|v| v.vid() == &Value::String("v1".to_string())));
        assert!(unique_vertices
            .iter()
            .any(|v| v.vid() == &Value::String("v2".to_string())));
        assert!(unique_vertices
            .iter()
            .any(|v| v.vid() == &Value::String("v3".to_string())));
    }

    #[test]
    fn test_extract_unique_edges() {
        let builder = ResultBuilder::new();

        let edge1 = Edge::new(
            Value::String("v1".to_string()),
            Value::String("v2".to_string()),
            "KNOWS".to_string(),
            0,
            HashMap::new(),
        );
        let edge2 = Edge::new(
            Value::String("v2".to_string()),
            Value::String("v3".to_string()),
            "FOLLOWS".to_string(),
            0,
            HashMap::new(),
        );
        let edge3 = Edge::new(
            Value::String("v1".to_string()),
            Value::String("v2".to_string()),
            "KNOWS".to_string(),
            0,
            HashMap::new(),
        );

        // 创建包含重复边的路径
        let mut path1 = PathInfo::new();
        path1.add_edge(edge1.clone());
        path1.add_edge(edge2.clone());

        let mut path2 = PathInfo::new();
        path2.add_edge(edge3.clone()); // 与edge1相同
        path2.add_edge(edge2.clone()); // 与path1中的edge2相同

        let paths = vec![path1, path2];
        let unique_edges = builder.extract_unique_edges(&paths);

        assert_eq!(unique_edges.len(), 2);
        assert!(unique_edges.iter().any(|e| e.edge_type() == "KNOWS"));
        assert!(unique_edges.iter().any(|e| e.edge_type() == "FOLLOWS"));
    }

    #[test]
    fn test_analyze_paths() {
        let builder = ResultBuilder::new();

        let tag = Tag::new("Person".to_string(), HashMap::new());
        let vertex1 = Vertex::new(Value::String("v1".to_string()), vec![tag.clone()]);
        let vertex2 = Vertex::new(Value::String("v2".to_string()), vec![tag.clone()]);
        let vertex3 = Vertex::new(Value::String("v3".to_string()), vec![tag]);

        let edge1 = Edge::new(
            Value::String("v1".to_string()),
            Value::String("v2".to_string()),
            "KNOWS".to_string(),
            0,
            HashMap::new(),
        );
        let edge2 = Edge::new(
            Value::String("v2".to_string()),
            Value::String("v3".to_string()),
            "FOLLOWS".to_string(),
            0,
            HashMap::new(),
        );

        // 创建不同长度的路径
        let mut path1 = PathInfo::new();
        path1.add_vertex(vertex1.clone());

        let mut path2 = PathInfo::new();
        path2.add_vertex(vertex1.clone());
        path2.add_vertex(vertex2.clone());
        path2.add_edge(edge1.clone());

        let mut path3 = PathInfo::new();
        path3.add_vertex(vertex1.clone());
        path3.add_vertex(vertex2.clone());
        path3.add_vertex(vertex3.clone());
        path3.add_edge(edge1.clone());
        path3.add_edge(edge2.clone());

        let paths = vec![path1, path2, path3];
        let analysis = builder.analyze_paths(&paths);

        assert_eq!(analysis.total_paths, 3);
        assert_eq!(analysis.empty_paths, 0);
        assert_eq!(analysis.min_path_length, 0);
        assert_eq!(analysis.max_path_length, 2);
        assert_eq!(analysis.avg_path_length, 1.0);
        assert_eq!(analysis.min_vertices, 1);
        assert_eq!(analysis.max_vertices, 3);
        assert_eq!(analysis.avg_vertices, 2.0);
        assert_eq!(analysis.min_edges, 0);
        assert_eq!(analysis.max_edges, 2);
        assert_eq!(analysis.avg_edges, 1.0);
    }

    #[test]
    fn test_path_analysis_default() {
        let analysis = PathAnalysis::new();
        assert_eq!(analysis.total_paths, 0);
        assert_eq!(analysis.empty_paths, 0);
        assert_eq!(analysis.min_path_length, 0);
        assert_eq!(analysis.max_path_length, 0);
        assert_eq!(analysis.avg_path_length, 0.0);
    }
}
