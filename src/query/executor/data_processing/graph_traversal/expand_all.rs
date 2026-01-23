use async_trait::async_trait;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::core::error::{DBError, DBResult};
use crate::core::{Edge, Path, Step, Value, Vertex};
use crate::query::executor::base::{BaseExecutor, EdgeDirection, InputExecutor};
use crate::query::executor::traits::{ExecutionResult, Executor, HasStorage};
use crate::query::QueryError;
use crate::storage::StorageEngine;
use crate::utils::safe_lock;

/// ExpandAllExecutor - 全路径扩展执行器
///
/// 返回从当前节点出发的所有可能的路径，而不仅仅是下一跳节点
/// 通常用于路径探索查询
pub struct ExpandAllExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    pub edge_direction: EdgeDirection,
    pub edge_types: Option<Vec<String>>,
    pub max_depth: Option<usize>, // 最大扩展深度
    input_executor: Option<Box<dyn Executor<S>>>,
    // 路径缓存
    path_cache: Vec<Path>,
    // 已访问节点集合，用于避免循环
    visited_nodes: HashSet<Value>,
}

// Manual Debug implementation for ExpandAllExecutor to avoid requiring Debug trait for Executor trait object
impl<S: StorageEngine> std::fmt::Debug for ExpandAllExecutor<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExpandAllExecutor")
            .field("base", &"BaseExecutor")
            .field("edge_direction", &self.edge_direction)
            .field("edge_types", &self.edge_types)
            .field("max_depth", &self.max_depth)
            .field("input_executor", &"Option<Box<dyn Executor<S>>>")
            .field("path_cache", &self.path_cache)
            .field("visited_nodes", &self.visited_nodes)
            .finish()
    }
}

impl<S: StorageEngine + Send> ExpandAllExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        edge_direction: EdgeDirection,
        edge_types: Option<Vec<String>>,
        max_depth: Option<usize>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "ExpandAllExecutor".to_string(), storage),
            edge_direction,
            edge_types,
            max_depth,
            input_executor: None,
            path_cache: Vec::new(),
            visited_nodes: HashSet::new(),
        }
    }

    /// 获取节点的邻居节点和对应的边
    async fn get_neighbors_with_edges(
        &self,
        node_id: &Value,
    ) -> Result<Vec<(Value, Edge)>, QueryError> {
        let storage = safe_lock(&*self.get_storage())
            .expect("ExpandAllExecutor storage lock should not be poisoned");

        // 获取节点的所有边
        let edges = storage
            .get_node_edges(node_id, EdgeDirection::Both)
            .map_err(|e| QueryError::StorageError(e.to_string()))?;

        // 过滤边类型
        let filtered_edges = if let Some(ref edge_types) = self.edge_types {
            edges
                .into_iter()
                .filter(|edge| edge_types.contains(&edge.edge_type))
                .collect()
        } else {
            edges
        };

        // 根据方向过滤边并提取邻居节点ID和边
        let neighbors_with_edges = filtered_edges
            .into_iter()
            .filter_map(|edge| match self.edge_direction {
                EdgeDirection::In => {
                    if *edge.dst == *node_id {
                        Some(((*edge.src).clone(), edge))
                    } else {
                        None
                    }
                }
                EdgeDirection::Out => {
                    if *edge.src == *node_id {
                        Some(((*edge.dst).clone(), edge))
                    } else {
                        None
                    }
                }
                EdgeDirection::Both => {
                    if *edge.src == *node_id {
                        Some(((*edge.dst).clone(), edge))
                    } else if *edge.dst == *node_id {
                        Some(((*edge.src).clone(), edge))
                    } else {
                        None
                    }
                }
            })
            .collect();

        Ok(neighbors_with_edges)
    }

    /// 递归扩展路径
    fn expand_paths_recursive<'a>(
        &'a mut self,
        current_path: &'a mut Path,
        current_depth: usize,
        max_depth: usize,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<Path>, QueryError>> + Send + 'a>,
    > {
        Box::pin(async move {
            // 获取当前路径的最后一个节点
            let current_node = if current_path.steps.is_empty() {
                &current_path.src.vid
            } else {
                &current_path
                    .steps
                    .last()
                    .expect("Path should have at least one step if steps is not empty")
                    .dst
                    .vid
            };

            // 检查是否达到最大深度
            if current_depth >= max_depth {
                // 返回当前路径
                return Ok(vec![current_path.clone()]);
            }

            // 获取邻居节点和边
            let neighbors_with_edges = self.get_neighbors_with_edges(current_node).await?;

            if neighbors_with_edges.is_empty() {
                // 没有更多邻居，返回当前路径
                return Ok(vec![current_path.clone()]);
            }

            let mut all_paths = Vec::new();

            // 为每个邻居创建新路径
            for (neighbor_id, edge) in neighbors_with_edges {
                // 检查是否已访问过该节点（避免循环）
                if self.visited_nodes.contains(&neighbor_id) {
                    // 创建包含循环的路径
                    let mut path_with_cycle = current_path.clone();
                    path_with_cycle.steps.push(Step {
                        dst: Box::new(Vertex::new(neighbor_id.clone(), Vec::new())),
                        edge: Box::new(edge),
                    });
                    all_paths.push(path_with_cycle);
                    continue;
                }

                // 获取邻居节点的完整信息
                let neighbor_vertex = {
                    let storage = safe_lock(&*self.get_storage())
                        .expect("ExpandAllExecutor storage lock should not be poisoned");
                    storage
                        .get_node(&neighbor_id)
                        .map_err(|e| QueryError::StorageError(e.to_string()))?
                };

                if let Some(vertex) = neighbor_vertex {
                    // 创建新路径
                    let mut new_path = current_path.clone();
                    new_path.steps.push(Step {
                        dst: Box::new(vertex),
                        edge: Box::new(edge),
                    });

                    // 标记为已访问
                    self.visited_nodes.insert(neighbor_id.clone());

                    // 递归扩展
                    let mut expanded_paths = self
                        .expand_paths_recursive(&mut new_path, current_depth + 1, max_depth)
                        .await?;
                    all_paths.append(&mut expanded_paths);

                    // 取消标记（允许在其他路径中访问）
                    self.visited_nodes.remove(&neighbor_id);
                }
            }

            // 添加当前路径
            all_paths.push(current_path.clone());

            Ok(all_paths)
        })
    }

    /// 构建扩展结果
    fn build_expansion_result(&self) -> ExecutionResult {
        // 将路径转换为值列表
        let mut path_values = Vec::new();

        for path in &self.path_cache {
            let mut path_value = Vec::new();

            // 添加起始节点
            path_value.push(Value::Vertex(path.src.clone()));

            // 添加每一步的边和节点
            for step in &path.steps {
                path_value.push(Value::Edge((*step.edge).clone()));
                path_value.push(Value::Vertex(Box::new((*step.dst).clone())));
            }

            path_values.push(Value::List(path_value));
        }

        ExecutionResult::Values(path_values)
    }
}

impl<S: StorageEngine> InputExecutor<S> for ExpandAllExecutor<S> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>) {
        self.input_executor = Some(input);
    }

    fn get_input(&self) -> Option<&Box<dyn Executor<S>>> {
        self.input_executor.as_ref()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for ExpandAllExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        // 首先执行输入执行器（如果存在）
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute().await?
        } else {
            // 如果没有输入执行器，返回空结果
            ExecutionResult::Vertices(Vec::new())
        };

        // 提取输入节点
        let input_nodes = match input_result {
            ExecutionResult::Vertices(vertices) => vertices,
            ExecutionResult::Edges(edges) => {
                // 从边中提取节点
                let mut nodes = Vec::new();
                let storage = safe_lock(&*self.get_storage())
                    .expect("ExpandAllExecutor storage lock should not be poisoned");
                let mut visited = HashSet::new();
                for edge in edges {
                    if let Ok(Some(src_vertex)) = storage.get_node(&edge.src) {
                        if visited.insert(src_vertex.vid.clone()) {
                            nodes.push(src_vertex);
                        }
                    }
                    if let Ok(Some(dst_vertex)) = storage.get_node(&edge.dst) {
                        if visited.insert(dst_vertex.vid.clone()) {
                            nodes.push(dst_vertex);
                        }
                    }
                }
                nodes.into_iter().map(|v| v).collect()
            }
            ExecutionResult::Values(values) => {
                // 从值中提取节点
                let mut vertices = Vec::new();
                let storage = safe_lock(&*self.get_storage())
                    .expect("ExpandAllExecutor storage lock should not be poisoned");
                for value in values {
                    match value {
                        Value::Vertex(vertex) => vertices.push(*vertex),
                        Value::String(id_str) => {
                            // 尝试将字符串作为节点ID获取节点
                            let node_id = Value::String(id_str);
                            if let Ok(Some(vertex)) = storage.get_node(&node_id) {
                                vertices.push(vertex);
                            }
                        }
                        _ => continue,
                    }
                }
                vertices
            }
            _ => Vec::new(),
        };

        // 确定最大深度
        let max_depth = self.max_depth.unwrap_or(3); // 默认深度为3

        // 为每个输入节点生成路径
        for vertex in input_nodes {
            // 重置访问状态
            self.visited_nodes.clear();
            self.visited_nodes.insert((*vertex.vid).clone());

            // 创建初始路径
            let mut initial_path = Path {
                src: Box::new(vertex),
                steps: Vec::new(),
            };

            // 递归扩展路径
            let mut expanded_paths = self
                .expand_paths_recursive(&mut initial_path, 0, max_depth)
                .await
                .map_err(DBError::from)?;
            self.path_cache.append(&mut expanded_paths);
        }

        // 构建结果
        Ok(self.build_expansion_result())
    }

    fn open(&mut self) -> DBResult<()> {
        self.path_cache.clear();
        self.visited_nodes.clear();

        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.open()?;
        }
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        self.path_cache.clear();
        self.visited_nodes.clear();

        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.close()?;
        }
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.base.is_open()
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        &self.base.description
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageEngine + Send> HasStorage<S> for ExpandAllExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("ExpandAllExecutor storage should be set")
    }
}
