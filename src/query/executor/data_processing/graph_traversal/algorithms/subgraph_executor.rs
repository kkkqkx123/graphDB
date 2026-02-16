//! 子图查询执行器
//!
//! 支持获取指定起点在给定步数内的子图

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use crate::core::{Edge, Path, Value, Vertex};
use crate::core::error::{DBError, DBResult};
use crate::query::executor::base::{BaseExecutor, EdgeDirection, Executor as BaseExecutorTrait, ExecutorStats, HasStorage, InputExecutor, ExecutionResult, DBResult as ExecDBResult};
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::storage::StorageClient;
use parking_lot::Mutex;

use super::types::AlgorithmStats;

/// 子图查询配置
#[derive(Debug, Clone)]
pub struct SubgraphConfig {
    /// 最大步数
    pub steps: usize,
    /// 边方向
    pub edge_direction: EdgeDirection,
    /// 边类型过滤
    pub edge_types: Option<Vec<String>>,
    /// 双向边类型（用于处理双向边）
    pub bidirect_edge_types: Option<HashSet<String>>,
    /// 边过滤条件
    pub edge_filter: Option<String>,
    /// 顶点过滤条件
    pub vertex_filter: Option<String>,
    /// 是否包含属性
    pub with_properties: bool,
    /// 结果限制
    pub limit: Option<usize>,
}

impl Default for SubgraphConfig {
    fn default() -> Self {
        Self {
            steps: 1,
            edge_direction: EdgeDirection::Out,
            edge_types: None,
            bidirect_edge_types: None,
            edge_filter: None,
            vertex_filter: None,
            with_properties: true,
            limit: None,
        }
    }
}

impl SubgraphConfig {
    pub fn new(steps: usize) -> Self {
        Self {
            steps,
            ..Default::default()
        }
    }

    pub fn with_direction(mut self, direction: EdgeDirection) -> Self {
        self.edge_direction = direction;
        self
    }

    pub fn with_edge_types(mut self, edge_types: Vec<String>) -> Self {
        self.edge_types = Some(edge_types);
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// 子图查询结果
#[derive(Debug, Clone)]
pub struct SubgraphResult {
    /// 子图中的顶点
    pub vertices: HashMap<Value, Vertex>,
    /// 子图中的边
    pub edges: Vec<Edge>,
    /// 访问过的顶点ID
    pub visited_vids: HashSet<Value>,
    /// 统计信息
    pub stats: AlgorithmStats,
}

impl SubgraphResult {
    pub fn new() -> Self {
        Self {
            vertices: HashMap::new(),
            edges: Vec::new(),
            visited_vids: HashSet::new(),
            stats: AlgorithmStats::new(),
        }
    }

    /// 转换为路径列表
    pub fn to_paths(&self) -> Vec<Path> {
        let mut paths = Vec::new();
        
        for edge in &self.edges {
            if let Some(src_vertex) = self.vertices.get(&edge.src) {
                let mut path = Path::new(src_vertex.clone());
                let dst_vertex = self.vertices.get(&edge.dst)
                    .cloned()
                    .unwrap_or_else(|| Vertex::with_vid(edge.dst.as_ref().clone()));
                path.steps.push(crate::core::Step::new(
                    dst_vertex,
                    edge.edge_type.clone(),
                    edge.edge_type.clone(),
                    edge.ranking,
                ));
                paths.push(path);
            }
        }
        
        paths
    }
}

impl Default for SubgraphResult {
    fn default() -> Self {
        Self::new()
    }
}

/// 子图查询执行器
///
/// 获取指定起点在给定步数内的所有顶点和边
pub struct SubgraphExecutor<S: StorageClient + Send + 'static> {
    base: BaseExecutor<S>,
    /// 起点ID列表
    start_vids: Vec<Value>,
    /// 配置
    config: SubgraphConfig,
    /// 当前步数
    current_step: usize,
    /// 历史访问的顶点（vid -> step）
    history_vids: HashMap<Value, usize>,
    /// 当前步访问的顶点
    current_vids: HashSet<Value>,
    /// 有效顶点（在步数范围内的顶点）
    valid_vids: HashSet<Value>,
    /// 下一步要访问的顶点
    next_vids: Vec<Value>,
    /// 子图结果
    result: SubgraphResult,
    /// 统计信息
    stats: AlgorithmStats,
}

impl<S: StorageClient> std::fmt::Debug for SubgraphExecutor<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubgraphExecutor")
            .field("base", &"BaseExecutor")
            .field("start_vids", &self.start_vids)
            .field("config", &self.config)
            .field("current_step", &self.current_step)
            .field("history_vids_count", &self.history_vids.len())
            .field("valid_vids_count", &self.valid_vids.len())
            .finish()
    }
}

impl<S: StorageClient> SubgraphExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        start_vids: Vec<Value>,
        config: SubgraphConfig,
    ) -> Self {
        let valid_vids: HashSet<Value> = start_vids.iter().cloned().collect();
        
        Self {
            base: BaseExecutor::new(id, "SubgraphExecutor".to_string(), storage),
            start_vids: start_vids.clone(),
            config,
            current_step: 1,
            history_vids: HashMap::new(),
            current_vids: HashSet::new(),
            valid_vids,
            next_vids: start_vids,
            result: SubgraphResult::new(),
            stats: AlgorithmStats::new(),
        }
    }

    /// 获取邻居节点
    fn get_neighbors(&self, node_id: &Value) -> DBResult<Vec<(Value, Edge)>> {
        let storage = self.base.storage.as_ref()
            .ok_or_else(|| DBError::Storage(
                crate::core::error::StorageError::DbError("Storage not set".to_string())
            ))?;
        let storage = storage.lock();

        let edges = storage
            .get_node_edges("default", node_id, self.config.edge_direction)
            .map_err(|e| DBError::Storage(
                crate::core::error::StorageError::DbError(e.to_string())
            ))?;

        let filtered_edges = if let Some(ref edge_types) = self.config.edge_types {
            edges
                .into_iter()
                .filter(|edge| edge_types.contains(&edge.edge_type))
                .collect()
        } else {
            edges
        };

        let neighbors: Vec<(Value, Edge)> = filtered_edges
            .into_iter()
            .filter_map(|edge| {
                if *edge.src == *node_id {
                    Some(((*edge.dst).clone(), edge))
                } else if *edge.dst == *node_id && self.config.edge_direction == EdgeDirection::Both {
                    Some(((*edge.src).clone(), edge))
                } else {
                    None
                }
            })
            .collect();

        Ok(neighbors)
    }

    /// 处理单步扩展
    fn expand_step(&mut self) -> DBResult<bool> {
        if self.next_vids.is_empty() || self.current_step > self.config.steps {
            return Ok(false);
        }

        self.current_vids.clear();
        let current_step_vids: Vec<Value> = self.next_vids.drain(..).collect();

        for vid in current_step_vids {
            // 跳过已访问的顶点（除非是双向边且需要特殊处理）
            if let Some(&visited_step) = self.history_vids.get(&vid) {
                if self.config.bidirect_edge_types.is_none() {
                    continue;
                }
                // 双向边特殊处理：检查是否是前两步访问的
                if visited_step + 2 != self.current_step {
                    continue;
                }
            }

            let neighbors = self.get_neighbors(&vid)?;

            for (neighbor_id, edge) in neighbors {
                // 添加边到结果
                self.result.edges.push(edge);
                
                // 添加目标顶点到有效顶点集
                self.valid_vids.insert(neighbor_id.clone());

                // 如果不是最后一步，添加到下一步访问列表
                if self.current_step < self.config.steps {
                    if self.current_vids.insert(neighbor_id.clone()) {
                        self.next_vids.push(neighbor_id);
                    }
                }
            }
        }

        // 更新历史记录
        for vid in &self.current_vids {
            self.history_vids.insert(vid.clone(), self.current_step);
        }

        self.current_step += 1;

        // 检查是否需要继续
        Ok(!self.next_vids.is_empty() && self.current_step <= self.config.steps)
    }

    /// 获取顶点详细信息
    fn fetch_vertices(&mut self) -> DBResult<()> {
        let storage = self.base.storage.as_ref()
            .ok_or_else(|| DBError::Storage(
                crate::core::error::StorageError::DbError("Storage not set".to_string())
            ))?;
        let storage = storage.lock();

        for vid in &self.valid_vids {
            match storage.get_vertex("default", vid) {
                Ok(Some(vertex)) => {
                    self.result.vertices.insert(vid.clone(), vertex);
                }
                Ok(None) => {
                    // 顶点不存在，创建一个只有VID的顶点
                    let vertex = Vertex::with_vid(vid.clone());
                    self.result.vertices.insert(vid.clone(), vertex);
                }
                Err(e) => {
                    return Err(DBError::Storage(
                        crate::core::error::StorageError::DbError(e.to_string())
                    ));
                }
            }
        }

        Ok(())
    }

    /// 过滤边（移除指向无效顶点的边）
    fn filter_edges(&mut self) {
        self.result.edges.retain(|edge| {
            self.valid_vids.contains(&edge.src) && self.valid_vids.contains(&edge.dst)
        });
    }

    /// 执行子图查询
    pub fn execute_subgraph(&mut self) -> DBResult<SubgraphResult> {
        let start_time = Instant::now();

        // 执行多步扩展
        while self.expand_step()? {}

        // 获取顶点详细信息
        if self.config.with_properties {
            self.fetch_vertices()?;
        } else {
            // 只添加VID
            for vid in &self.valid_vids {
                let vertex = Vertex::with_vid(vid.clone());
                self.result.vertices.insert(vid.clone(), vertex);
            }
        }

        // 过滤边
        self.filter_edges();

        // 应用限制
        if let Some(limit) = self.config.limit {
            if self.result.edges.len() > limit {
                self.result.edges.truncate(limit);
            }
        }

        self.stats.set_execution_time(start_time.elapsed().as_millis() as u64);
        self.result.stats = self.stats.clone();
        self.result.visited_vids = self.valid_vids.clone();

        Ok(self.result.clone())
    }

    /// 获取结果路径
    pub fn get_result_paths(&self) -> Vec<Path> {
        self.result.to_paths()
    }
}

impl<S: StorageClient + Send + 'static> BaseExecutorTrait<S> for SubgraphExecutor<S> {
    fn execute(&mut self) -> ExecDBResult<ExecutionResult> {
        let result = self.execute_subgraph()
            .map_err(|e| crate::core::error::DBError::Query(
                crate::query::QueryError::ExecutionError(e.to_string())
            ))?;
        
        let paths = result.to_paths();
        Ok(ExecutionResult::Paths(paths))
    }

    fn open(&mut self) -> ExecDBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> ExecDBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        "Subgraph executor - retrieves subgraph within specified steps"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for SubgraphExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient + Send + 'static> InputExecutor<S> for SubgraphExecutor<S> {
    fn set_input(&mut self, _input: ExecutorEnum<S>) {
        // 子图查询不需要输入
    }

    fn get_input(&self) -> Option<&ExecutorEnum<S>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;
    use crate::storage::MockStorage;

    #[test]
    fn test_subgraph_config_default() {
        let config = SubgraphConfig::default();
        assert_eq!(config.steps, 1);
        assert_eq!(config.edge_direction, EdgeDirection::Out);
        assert!(config.edge_types.is_none());
        assert!(config.limit.is_none());
    }

    #[test]
    fn test_subgraph_config_builder() {
        let config = SubgraphConfig::new(3)
            .with_direction(EdgeDirection::Both)
            .with_edge_types(vec!["knows".to_string()])
            .with_limit(100);

        assert_eq!(config.steps, 3);
        assert_eq!(config.edge_direction, EdgeDirection::Both);
        assert_eq!(config.edge_types, Some(vec!["knows".to_string()]));
        assert_eq!(config.limit, Some(100));
    }

    #[test]
    fn test_subgraph_executor_creation() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("Failed to create MockStorage")));
        let config = SubgraphConfig::new(2);
        
        let executor = SubgraphExecutor::new(
            1,
            storage,
            vec![Value::from("a")],
            config,
        );

        assert_eq!(executor.start_vids.len(), 1);
        assert_eq!(executor.config.steps, 2);
        assert_eq!(executor.valid_vids.len(), 1);
    }

    #[test]
    fn test_subgraph_result() {
        let mut result = SubgraphResult::new();
        
        // 添加一些顶点
        result.vertices.insert(
            Value::from("a"),
            Vertex::with_vid(Value::from("a"))
        );
        result.vertices.insert(
            Value::from("b"),
            Vertex::with_vid(Value::from("b"))
        );
        
        // 添加一条边
        let edge = Edge::new(
            Value::from("a"),
            Value::from("b"),
            "knows".to_string(),
            0,
            HashMap::new(),
        );
        result.edges.push(edge);
        
        // 测试转换为路径
        let paths = result.to_paths();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].steps.len(), 1);
    }
}
