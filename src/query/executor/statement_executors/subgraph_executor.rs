//! SUBGRAPH 语句执行器
//!
//! 处理 SUBGRAPH 子图查询，支持指定步数范围内的子图提取

use crate::core::error::{DBError, DBResult, QueryError};
use crate::core::Value;
use crate::query::executor::base::{ExecutionResult, Executor};
use crate::query::executor::factory::ExecutorFactory;
use crate::query::parser::ast::stmt::SubgraphStmt;
use crate::query::planner::planner::{Planner, ValidatedStatement};
use crate::query::planner::statements::subgraph_planner::SubgraphPlanner;
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::query::validator::ValidationInfo;
use crate::query::QueryContext;
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// SUBGRAPH 执行器
///
/// 处理子图查询，支持：
/// - 基于顶点的子图提取
/// - 指定步数范围（M TO N STEPS）
/// - 边类型过滤
/// - 边方向过滤
/// - WHERE 子句过滤
pub struct SubgraphExecutor<S: StorageClient + 'static> {
    id: i64,
    storage: Arc<Mutex<S>>,
}

impl<S: StorageClient + 'static> SubgraphExecutor<S> {
    /// 创建新的 SUBGRAPH 执行器
    pub fn new(id: i64, storage: Arc<Mutex<S>>) -> Self {
        Self { id, storage }
    }

    /// 执行 SUBGRAPH 查询
    pub fn execute_subgraph(&self, clause: SubgraphStmt) -> DBResult<ExecutionResult>
    where
        S: Send + Sync + 'static,
    {
        let qctx = Arc::new(QueryContext::default());

        let validation_info = ValidationInfo::new();
        let ctx = Arc::new(ExpressionAnalysisContext::new());
        let ast = Arc::new(crate::query::parser::ast::Ast::new(
            crate::query::parser::ast::stmt::Stmt::Subgraph(clause),
            ctx,
        ));
        let validated = ValidatedStatement::new(ast, validation_info);

        let mut planner = SubgraphPlanner::new();
        let plan = planner
            .transform(&validated, qctx)
            .map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))?;

        let root_node = plan
            .root()
            .as_ref()
            .ok_or_else(|| DBError::Query(QueryError::ExecutionError("执行计划为空".to_string())))?
            .clone();

        let mut executor_factory = ExecutorFactory::with_storage(self.storage.clone());
        let mut executor = executor_factory
            .create_executor(&root_node, self.storage.clone(), &Default::default())
            .map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))?;

        executor
            .open()
            .map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))?;

        let result = executor
            .execute()
            .map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))?;

        executor
            .close()
            .map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))?;

        Ok(result)
    }

    /// 提取子图
    ///
    /// 从起始顶点开始，在指定步数范围内提取子图
    pub fn extract_subgraph(
        &self,
        start_vertices: Vec<Value>,
        steps: usize,
        edge_types: Option<Vec<String>>,
        edge_direction: crate::query::executor::base::EdgeDirection,
        include_properties: bool,
    ) -> DBResult<SubgraphResult> {
        let mut visited = HashSet::new();
        let mut vertices = HashMap::new();
        let mut edges = Vec::new();

        for start_vid in start_vertices {
            self.bfs_extract(
                start_vid,
                steps,
                edge_types.as_deref(),
                edge_direction,
                include_properties,
                &mut visited,
                &mut vertices,
                &mut edges,
            )?;
        }

        Ok(SubgraphResult {
            vertices,
            edges,
            visited,
        })
    }

    /// BFS 提取子图
    ///
    /// 使用广度优先搜索提取子图
    fn bfs_extract(
        &self,
        start_vid: Value,
        max_steps: usize,
        edge_types: Option<&[String]>,
        edge_direction: crate::query::executor::base::EdgeDirection,
        include_properties: bool,
        visited: &mut HashSet<Value>,
        vertices: &mut HashMap<Value, crate::core::Vertex>,
        edges: &mut Vec<crate::core::Edge>,
    ) -> DBResult<()> {
        use crate::query::executor::data_access::GetNeighborsExecutor;
        use crate::query::executor::data_access::GetVerticesExecutor;

        let mut current_level = vec![start_vid.clone()];
        visited.insert(start_vid.clone());

        if include_properties {
            let mut vertex_executor = GetVerticesExecutor::new(
                self.id,
                self.storage.clone(),
                Some(vec![start_vid.clone()]),
                None,
                None,
                None,
                Arc::new(ExpressionAnalysisContext::new()),
            );
            vertex_executor.open()?;
            let vertex_result = vertex_executor.execute()?;
            if let ExecutionResult::Vertices(vertex_list) = vertex_result {
                for vertex in vertex_list {
                    vertices.insert(vertex.vid().clone(), vertex);
                }
            }
            vertex_executor.close()?;
        } else {
            vertices.insert(start_vid.clone(), crate::core::Vertex::with_vid(start_vid.clone()));
        }

        for _step in 0..max_steps {
            if current_level.is_empty() {
                break;
            }

            let mut next_level = HashSet::new();

            for vid in &current_level {
                let mut executor = GetNeighborsExecutor::new(
                    self.id,
                    self.storage.clone(),
                    vec![vid.clone()],
                    edge_direction,
                    edge_types.map(|types| types.to_vec()),
                    Arc::new(ExpressionAnalysisContext::new()),
                );

                executor.open()?;

                let result = executor.execute()?;

                match result {
                    ExecutionResult::Edges(edge_list) => {
                        for edge in edge_list {
                            edges.push(edge.clone());

                            let neighbor_vid = match edge_direction {
                                crate::query::executor::base::EdgeDirection::Out => edge.dst().clone(),
                                crate::query::executor::base::EdgeDirection::In => edge.src().clone(),
                                crate::query::executor::base::EdgeDirection::Both => {
                                    if edge.src() == vid {
                                        edge.dst().clone()
                                    } else {
                                        edge.src().clone()
                                    }
                                }
                            };

                            if !visited.contains(&neighbor_vid) {
                                visited.insert(neighbor_vid.clone());
                                next_level.insert(neighbor_vid.clone());

                                if include_properties {
                                    let mut vertex_executor = GetVerticesExecutor::new(
                                        self.id,
                                        self.storage.clone(),
                                        Some(vec![neighbor_vid.clone()]),
                                        None,
                                        None,
                                        None,
                                        Arc::new(ExpressionAnalysisContext::new()),
                                    );
                                    vertex_executor.open()?;
                                    let vertex_result = vertex_executor.execute()?;
                                    if let ExecutionResult::Vertices(vertex_list) = vertex_result {
                                        for vertex in vertex_list {
                                            vertices.insert(vertex.vid().clone(), vertex);
                                        }
                                    }
                                    vertex_executor.close()?;
                                } else {
                                    vertices.insert(neighbor_vid.clone(), crate::core::Vertex::with_vid(neighbor_vid.clone()));
                                }
                            }
                        }
                    }
                    _ => {}
                }

                executor.close()?;
            }

            current_level = next_level.into_iter().collect();
        }

        Ok(())
    }

    /// 构建子图
    ///
    /// 组合顶点和边，构建完整的子图
    pub fn build_subgraph(
        &self,
        vertices: HashMap<Value, crate::core::Vertex>,
        edges: Vec<crate::core::Edge>,
    ) -> DBResult<SubgraphResult> {
        let visited: HashSet<Value> = vertices.keys().cloned().collect();

        Ok(SubgraphResult {
            vertices,
            edges,
            visited,
        })
    }
}

/// 子图查询结果
#[derive(Debug, Clone)]
pub struct SubgraphResult {
    /// 子图中的顶点
    pub vertices: HashMap<Value, crate::core::Vertex>,
    /// 子图中的边
    pub edges: Vec<crate::core::Edge>,
    /// 访问过的顶点ID
    pub visited: HashSet<Value>,
}

impl SubgraphResult {
    /// 创建空的子图结果
    pub fn new() -> Self {
        Self {
            vertices: HashMap::new(),
            edges: Vec::new(),
            visited: HashSet::new(),
        }
    }

    /// 获取顶点数量
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// 获取边数量
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// 检查是否包含顶点
    pub fn contains_vertex(&self, vid: &Value) -> bool {
        self.visited.contains(vid)
    }

    /// 获取顶点
    pub fn get_vertex(&self, vid: &Value) -> Option<&crate::core::Vertex> {
        self.vertices.get(vid)
    }

    /// 获取所有顶点
    pub fn get_vertices(&self) -> Vec<&crate::core::Vertex> {
        self.vertices.values().collect()
    }

    /// 获取所有边
    pub fn get_edges(&self) -> &[crate::core::Edge] {
        &self.edges
    }

    /// 合并另一个子图结果
    pub fn merge(&mut self, other: SubgraphResult) {
        for (vid, vertex) in other.vertices {
            self.vertices.entry(vid).or_insert(vertex);
        }
        self.edges.extend(other.edges);
        self.visited.extend(other.visited);
    }
}

impl Default for SubgraphResult {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subgraph_result_new() {
        let result = SubgraphResult::new();

        assert_eq!(result.vertex_count(), 0);
        assert_eq!(result.edge_count(), 0);
    }

    #[test]
    fn test_subgraph_result_contains_vertex() {
        let result = SubgraphResult::new();
        let vid = Value::Int(1);

        assert!(!result.contains_vertex(&vid));
    }

    #[test]
    fn test_subgraph_result_merge() {
        let mut result1 = SubgraphResult::new();
        let result2 = SubgraphResult::new();

        result1.merge(result2);

        assert_eq!(result1.vertex_count(), 0);
        assert_eq!(result1.edge_count(), 0);
    }
}
