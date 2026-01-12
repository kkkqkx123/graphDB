use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::core::error::{DBError, DBResult};
use crate::core::Value;
use crate::query::executor::base::{BaseExecutor, EdgeDirection, InputExecutor};
use crate::query::executor::traits::{ExecutionResult, Executor, HasStorage};
use crate::query::QueryError;
use crate::storage::StorageEngine;
use crate::utils::safe_lock;

/// ExpandExecutor - 单步路径扩展执行器
///
/// 从当前节点按照指定的边类型和方向扩展一步，获取相邻节点
/// 通常用于图遍历和路径查询
pub struct ExpandExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    pub edge_direction: EdgeDirection,
    pub edge_types: Option<Vec<String>>,
    pub max_depth: Option<usize>, // 最大扩展深度
    input_executor: Option<Box<dyn Executor<S>>>,
    // 缓存已访问的节点，用于避免循环
    visited_nodes: HashSet<Value>,
    // 邻接关系缓存
    adjacency_cache: HashMap<Value, Vec<Value>>,
}

// Manual Debug implementation for ExpandExecutor to avoid requiring Debug trait for Executor trait object
impl<S: StorageEngine> std::fmt::Debug for ExpandExecutor<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExpandExecutor")
            .field("base", &"BaseExecutor")
            .field("edge_direction", &self.edge_direction)
            .field("edge_types", &self.edge_types)
            .field("max_depth", &self.max_depth)
            .field("input_executor", &"Option<Box<dyn Executor<S>>>")
            .field("visited_nodes", &self.visited_nodes)
            .field("adjacency_cache", &"HashMap<Value, Vec<Value>>")
            .finish()
    }
}

impl<S: StorageEngine> ExpandExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        edge_direction: EdgeDirection,
        edge_types: Option<Vec<String>>,
        max_depth: Option<usize>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "ExpandExecutor".to_string(), storage),
            edge_direction,
            edge_types,
            max_depth,
            input_executor: None,
            visited_nodes: HashSet::new(),
            adjacency_cache: HashMap::new(),
        }
    }

    /// 获取节点的邻居节点
    async fn get_neighbors(&self, node_id: &Value) -> Result<Vec<Value>, QueryError> {
        let storage = safe_lock(self.get_storage())
            .expect("ExpandExecutor storage lock should not be poisoned");

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

        // 根据方向过滤边并提取邻居节点ID
        let neighbors = filtered_edges
            .into_iter()
            .filter_map(|edge| match self.edge_direction {
                EdgeDirection::Incoming => {
                    if *edge.dst == *node_id {
                        Some((*edge.src).clone())
                    } else {
                        None
                    }
                }
                EdgeDirection::Outgoing => {
                    if *edge.src == *node_id {
                        Some((*edge.dst).clone())
                    } else {
                        None
                    }
                }
                EdgeDirection::Both => {
                    if *edge.src == *node_id {
                        Some((*edge.dst).clone())
                    } else if *edge.dst == *node_id {
                        Some((*edge.src).clone())
                    } else {
                        None
                    }
                }
            })
            .collect();

        Ok(neighbors)
    }

    /// 执行单步扩展
    async fn expand_step(&mut self, input_nodes: Vec<Value>) -> Result<Vec<Value>, QueryError> {
        let mut expanded_nodes = Vec::new();

        for node_id in input_nodes {
            // 检查是否已访问过该节点
            if self.visited_nodes.contains(&node_id) {
                continue;
            }

            // 标记为已访问
            self.visited_nodes.insert(node_id.clone());

            // 获取邻居节点
            let neighbors = self.get_neighbors(&node_id).await?;

            // 缓存邻接关系
            self.adjacency_cache
                .insert(node_id.clone(), neighbors.clone());

            // 添加未访问的邻居节点
            for neighbor in neighbors {
                if !self.visited_nodes.contains(&neighbor) {
                    expanded_nodes.push(neighbor);
                }
            }
        }

        Ok(expanded_nodes)
    }

    /// 构建扩展结果
    fn build_expansion_result(&self, expanded_nodes: Vec<Value>) -> ExecutionResult {
        // 将节点ID转换为顶点对象
        let mut vertices = Vec::new();
        let storage = safe_lock(&*self.get_storage())
            .expect("ExpandExecutor storage lock should not be poisoned");

        for node_id in expanded_nodes {
            if let Ok(Some(vertex)) = storage.get_node(&node_id) {
                vertices.push(vertex);
            }
        }

        ExecutionResult::Vertices(vertices)
    }
}

impl<S: StorageEngine> InputExecutor<S> for ExpandExecutor<S> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>) {
        self.input_executor = Some(input);
    }

    fn get_input(&self) -> Option<&Box<dyn Executor<S>>> {
        self.input_executor.as_ref()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for ExpandExecutor<S> {
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
            ExecutionResult::Vertices(vertices) => vertices.into_iter().map(|v| *v.vid).collect(),
            ExecutionResult::Edges(edges) => {
                let mut nodes = Vec::new();
                let mut visited = HashSet::new();
                for edge in edges {
                    if visited.insert(edge.src.clone()) {
                        nodes.push(*edge.src);
                    }
                    if visited.insert(edge.dst.clone()) {
                        nodes.push(*edge.dst);
                    }
                }
                nodes
            }
            ExecutionResult::Values(values) => values
                .into_iter()
                .filter_map(|v| match v {
                    Value::Vertex(vertex) => Some(*vertex.vid),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        };

        // 执行扩展操作
        let expanded_nodes = self.expand_step(input_nodes).await.map_err(DBError::from)?;

        // 构建结果
        Ok(self.build_expansion_result(expanded_nodes))
    }

    fn open(&mut self) -> DBResult<()> {
        // 初始化扩展所需的任何资源
        self.visited_nodes.clear();
        self.adjacency_cache.clear();

        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.open()?;
        }
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        // 清理资源
        self.visited_nodes.clear();
        self.adjacency_cache.clear();

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
}

impl<S: StorageEngine + Send> HasStorage<S> for ExpandExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("ExpandExecutor storage should be set")
    }
}
