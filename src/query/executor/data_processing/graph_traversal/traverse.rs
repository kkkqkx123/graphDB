use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::core::{Edge, Path, Step, Value, Vertex};
use crate::query::executor::base::{
    BaseExecutor, EdgeDirection, ExecutionResult, Executor, InputExecutor,
};
use crate::query::QueryError;
use crate::storage::StorageEngine;

/// TraverseExecutor - 完整图遍历执行器
///
/// 执行完整的图遍历操作，支持多跳和条件过滤
/// 结合了 ExpandExecutor 的功能，支持更复杂的遍历需求
pub struct TraverseExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    edge_direction: EdgeDirection,
    edge_types: Option<Vec<String>>,
    max_depth: Option<usize>,
    conditions: Option<String>, // 遍历条件
    input_executor: Option<Box<dyn Executor<S>>>,
    // 遍历状态
    current_paths: Vec<Path>,
    completed_paths: Vec<Path>,
    visited_nodes: HashSet<Value>,
    // 遍历配置
    track_prev_path: bool,
    generate_path: bool,
}

impl<S: StorageEngine> TraverseExecutor<S> {
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
        edge_direction: EdgeDirection,
        edge_types: Option<Vec<String>>,
        max_depth: Option<usize>,
        conditions: Option<String>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "TraverseExecutor".to_string(), storage),
            edge_direction,
            edge_types,
            max_depth,
            conditions,
            input_executor: None,
            current_paths: Vec::new(),
            completed_paths: Vec::new(),
            visited_nodes: HashSet::new(),
            track_prev_path: true,
            generate_path: true,
        }
    }

    /// 设置是否跟踪前一个路径
    pub fn with_track_prev_path(mut self, track_prev_path: bool) -> Self {
        self.track_prev_path = track_prev_path;
        self
    }

    /// 设置是否生成路径
    pub fn with_generate_path(mut self, generate_path: bool) -> Self {
        self.generate_path = generate_path;
        self
    }

    /// 获取节点的邻居节点和对应的边
    async fn get_neighbors_with_edges(
        &self,
        node_id: &Value,
    ) -> Result<Vec<(Value, Edge)>, QueryError> {
        let storage = self.base.storage.lock().unwrap();

        // 获取节点的所有边
        let edges = storage
            .get_node_edges(node_id, crate::core::Direction::Both)
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
                        Some((edge.src.clone(), edge))
                    } else {
                        None
                    }
                }
                EdgeDirection::Out => {
                    if *edge.src == *node_id {
                        Some((edge.dst.clone(), edge))
                    } else {
                        None
                    }
                }
                EdgeDirection::Both => {
                    if *edge.src == *node_id {
                        Some((edge.dst.clone(), edge))
                    } else if *edge.dst == *node_id {
                        Some((edge.src.clone(), edge))
                    } else {
                        None
                    }
                }
            })
            .collect();

        Ok(neighbors_with_edges)
    }

    /// 检查条件是否满足
    fn check_conditions(&self, _path: &Path, _edge: &Edge, _vertex: &Vertex) -> bool {
        // TODO: 实现条件检查逻辑
        // 目前总是返回true
        true
    }

    /// 执行单步遍历
    async fn traverse_step(
        &mut self,
        current_depth: usize,
        max_depth: usize,
    ) -> Result<(), QueryError> {
        if current_depth >= max_depth {
            // 达到最大深度，将当前路径标记为完成
            self.completed_paths.extend(self.current_paths.clone());
            self.current_paths.clear();
            return Ok(());
        }

        let mut next_paths = Vec::new();

        for path in &self.current_paths {
            // 获取当前路径的最后一个节点
            let current_node = if path.steps.is_empty() {
                &path.src.vid
            } else {
                &path.steps.last().unwrap().dst.vid
            };

            // 获取邻居节点和边
            let neighbors_with_edges = self.get_neighbors_with_edges(current_node).await?;

            for (neighbor_id, edge) in neighbors_with_edges {
                // 获取邻居节点的完整信息
                let storage = self.base.storage.lock().unwrap();
                let neighbor_vertex = storage
                    .get_node(&neighbor_id)
                    .map_err(|e| QueryError::StorageError(e.to_string()))?;

                if let Some(vertex) = neighbor_vertex {
                    // 检查条件
                    if !self.check_conditions(path, &edge, &vertex) {
                        continue;
                    }

                    // 创建新路径
                    let mut new_path = path.clone();
                    new_path.steps.push(Step {
                        dst: Box::new(vertex),
                        edge: Box::new(edge),
                    });

                    // 检查是否达到最大深度
                    if current_depth + 1 >= max_depth {
                        self.completed_paths.push(new_path);
                    } else {
                        next_paths.push(new_path);
                    }
                }
            }
        }

        self.current_paths = next_paths;
        Ok(())
    }

    /// 初始化遍历
    async fn initialize_traversal(&mut self, input_nodes: Vec<Vertex>) -> Result<(), QueryError> {
        self.current_paths.clear();
        self.completed_paths.clear();
        self.visited_nodes.clear();

        // 为每个输入节点创建初始路径
        for vertex in input_nodes {
            let initial_path = Path {
                src: Box::new(vertex),
                steps: Vec::new(),
            };
            self.current_paths.push(initial_path);
            self.visited_nodes.insert(vertex.vid.clone());
        }

        Ok(())
    }

    /// 构建遍历结果
    fn build_traversal_result(&self) -> ExecutionResult {
        if self.generate_path {
            // 返回路径结果
            let mut path_values = Vec::new();

            for path in &self.completed_paths {
                let mut path_value = Vec::new();

                // 添加起始节点
                path_value.push(Value::Vertex(path.src.clone()));

                // 添加每一步的边和节点
                for step in &path.steps {
                    path_value.push(Value::Edge(step.edge.clone()));
                    path_value.push(Value::Vertex(step.dst.clone()));
                }

                path_values.push(Value::List(path_value));
            }

            ExecutionResult::Values(path_values)
        } else {
            // 返回顶点结果
            let mut vertices = Vec::new();
            let mut visited_vertices = HashSet::new();

            for path in &self.completed_paths {
                // 添加起始节点
                if !visited_vertices.contains(&path.src.vid) {
                    vertices.push(path.src.as_ref().clone());
                    visited_vertices.insert(path.src.vid.clone());
                }

                // 添加路径中的所有节点
                for step in &path.steps {
                    if !visited_vertices.contains(&step.dst.vid) {
                        vertices.push(step.dst.as_ref().clone());
                        visited_vertices.insert(step.dst.vid.clone());
                    }
                }
            }

            ExecutionResult::Vertices(vertices)
        }
    }
}

impl<S: StorageEngine> InputExecutor<S> for TraverseExecutor<S> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>) {
        self.input_executor = Some(input);
    }

    fn get_input(&self) -> Option<&Box<dyn Executor<S>>> {
        self.input_executor.as_ref()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for TraverseExecutor<S> {
    async fn execute(&mut self) -> Result<ExecutionResult, QueryError> {
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
                let mut nodes = HashSet::new();
                for edge in edges {
                    let storage = self.base.storage.lock().unwrap();
                    if let Ok(Some(src_vertex)) = storage.get_node(&edge.src) {
                        nodes.insert(src_vertex);
                    }
                    if let Ok(Some(dst_vertex)) = storage.get_node(&edge.dst) {
                        nodes.insert(dst_vertex);
                    }
                }
                nodes.into_iter().collect()
            }
            ExecutionResult::Values(values) => {
                // 从值中提取节点
                let mut vertices = Vec::new();
                let storage = self.base.storage.lock().unwrap();
                for value in values {
                    match value {
                        Value::Vertex(vertex) => vertices.push(vertex),
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

        if input_nodes.is_empty() {
            return Ok(ExecutionResult::Vertices(Vec::new()));
        }

        // 初始化遍历
        self.initialize_traversal(input_nodes).await?;

        // 确定最大深度
        let max_depth = self.max_depth.unwrap_or(3); // 默认深度为3

        // 执行遍历
        for current_depth in 0..max_depth {
            self.traverse_step(current_depth, max_depth).await?;

            // 如果没有更多路径可以扩展，提前结束
            if self.current_paths.is_empty() {
                break;
            }
        }

        // 将剩余的当前路径添加到完成路径中
        self.completed_paths.extend(self.current_paths.clone());

        // 构建结果
        Ok(self.build_traversal_result())
    }

    fn open(&mut self) -> Result<(), QueryError> {
        // 初始化遍历所需的任何资源
        self.current_paths.clear();
        self.completed_paths.clear();
        self.visited_nodes.clear();

        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.open()?;
        }
        Ok(())
    }

    fn close(&mut self) -> Result<(), QueryError> {
        // 清理资源
        self.current_paths.clear();
        self.completed_paths.clear();
        self.visited_nodes.clear();

        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.close()?;
        }
        Ok(())
    }

    fn id(&self) -> usize {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }
}
