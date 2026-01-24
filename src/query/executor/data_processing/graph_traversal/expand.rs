use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::core::error::{DBError, DBResult};
use crate::core::Value;
use crate::query::executor::base::{BaseExecutor, EdgeDirection, InputExecutor};
use crate::query::executor::traits::{ExecutionResult, Executor, HasStorage};
use crate::query::QueryError;
use crate::storage::StorageEngine;
use crate::utils::safe_lock;

/// ExpandExecutor - 路径扩展执行器
///
/// 从当前节点按照指定的边类型和方向扩展，获取相邻节点
/// 支持多步扩展和采样，通常用于图遍历和路径查询
pub struct ExpandExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    pub edge_direction: EdgeDirection,
    pub edge_types: Option<Vec<String>>,
    pub max_depth: Option<usize>, // 最大扩展深度
    pub step_limits: Option<Vec<usize>>, // 每步的扩展限制
    pub sample: bool, // 是否启用采样
    pub sample_limit: Option<usize>, // 采样限制
    input_executor: Option<Box<dyn Executor<S>>>,
    // 缓存已访问的节点，用于避免循环
    pub visited_nodes: HashSet<Value>,
    // 邻接关系缓存
    adjacency_cache: HashMap<Value, Vec<Value>>,
    // 当前扩展步数
    current_step: usize,
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
            step_limits: None,
            sample: false,
            sample_limit: None,
            input_executor: None,
            visited_nodes: HashSet::new(),
            adjacency_cache: HashMap::new(),
            current_step: 0,
        }
    }

    /// 设置每步的扩展限制
    pub fn with_step_limits(mut self, step_limits: Vec<usize>) -> Self {
        self.step_limits = Some(step_limits);
        self
    }

    /// 启用采样
    pub fn with_sampling(mut self, sample_limit: usize) -> Self {
        self.sample = true;
        self.sample_limit = Some(sample_limit);
        self
    }

    /// 执行多步扩展
    async fn expand_multi_step(&mut self, input_nodes: Vec<Value>) -> Result<Vec<Value>, QueryError> {
        let max_steps = self.max_depth.unwrap_or(1);
        let mut current_nodes = input_nodes;
        let mut all_expanded = HashSet::new();

        for step in 0..max_steps {
            self.current_step = step;

            // 检查每步的限制
            if let Some(ref step_limits) = self.step_limits {
                if step < step_limits.len() && current_nodes.len() > step_limits[step] {
                    // 应用采样
                    current_nodes = self.apply_sampling(&current_nodes, step_limits[step])?;
                }
            }

            // 执行单步扩展
            current_nodes = self.expand_step(current_nodes).await?;

            // 检查是否还有节点可以扩展
            if current_nodes.is_empty() {
                break;
            }

            // 记录扩展的节点
            for node in &current_nodes {
                all_expanded.insert(node.clone());
            }

            // 更新统计信息
            self.base
                .get_stats_mut()
                .add_stat(format!("step_{}_count", step), current_nodes.len().to_string());
        }

        Ok(all_expanded.into_iter().collect())
    }

    /// 应用水库采样算法
    fn apply_sampling(&self, nodes: &[Value], limit: usize) -> Result<Vec<Value>, QueryError> {
        if nodes.len() <= limit {
            return Ok(nodes.to_vec());
        }

        // 使用水库采样算法
        let mut sampled = Vec::with_capacity(limit);
        for (i, node) in nodes.iter().enumerate() {
            if i < limit {
                sampled.push(node.clone());
            } else {
                let j = rand::random::<usize>() % (i + 1);
                if j < limit {
                    sampled[j] = node.clone();
                }
            }
        }

        Ok(sampled)
    }

    /// 获取节点的邻居节点
    async fn get_neighbors(&self, node_id: &Value) -> Result<Vec<Value>, QueryError> {
        let storage = self.base.get_storage().clone();
        super::traversal_utils::get_neighbors(
            &storage,
            node_id,
            self.edge_direction,
            &self.edge_types,
        )
        .await
        .map_err(|e| QueryError::StorageError(e.to_string()))
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
        let start = Instant::now();

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
        let expanded_nodes = if self.max_depth.unwrap_or(1) > 1 {
            // 多步扩展
            self.expand_multi_step(input_nodes).await.map_err(DBError::from)?
        } else {
            // 单步扩展
            self.expand_step(input_nodes).await.map_err(DBError::from)?
        };

        // 构建结果
        let result = self.build_expansion_result(expanded_nodes);

        // 更新统计信息
        self.base.get_stats_mut().add_row(result.count());
        self.base.get_stats_mut().add_exec_time(start.elapsed());
        self.base.get_stats_mut().add_total_time(start.elapsed());

        Ok(result)
    }

    fn open(&mut self) -> DBResult<()> {
        // 初始化扩展所需的任何资源
        self.visited_nodes.clear();
        self.adjacency_cache.clear();
        self.current_step = 0;

        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.open()?;
        }
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        // 清理资源
        self.visited_nodes.clear();
        self.adjacency_cache.clear();
        self.current_step = 0;

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

impl<S: StorageEngine + Send> HasStorage<S> for ExpandExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("ExpandExecutor storage should be set")
    }
}
