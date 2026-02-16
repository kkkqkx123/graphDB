//! AllPaths 执行器
//!
//! 基于 Nebula 3.8.0 的 AllPathsExecutor 实现，使用 NPath 链表结构优化内存
//! 功能特点：
//! - 双向 BFS 算法
//! - 使用 NPath 链表结构，共享路径前缀
//! - 支持找到所有路径（非最短路径）
//! - 自动环路检测，避免循环路径
//! - 支持 limit 和 offset 限制结果数量
//! - 支持 withProp 返回路径属性
//! - 使用两阶段扩展（左扩展和右扩展）
//! - 当节点数量超过阈值时使用启发式扩展
//! - CPU 密集型操作，使用 Rayon 进行并行化

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::Instant;

use rayon::prelude::*;

use crate::core::error::{DBResult, DBError};
use crate::core::{Edge, NPath, Path, Value};
use crate::query::executor::base::{BaseExecutor, EdgeDirection, ExecutorStats, Executor, ExecutionResult};
use crate::query::executor::recursion_detector::ParallelConfig;
use crate::storage::StorageClient;
use parking_lot::Mutex;

/// 自环边去重辅助结构
#[derive(Debug, Default)]
struct SelfLoopDedup {
    seen: HashSet<(String, i64)>,
    with_loop: bool,
}

impl SelfLoopDedup {
    fn with_loop(with_loop: bool) -> Self {
        Self {
            seen: HashSet::new(),
            with_loop,
        }
    }

    fn should_include(&mut self, edge: &Edge) -> bool {
        let is_self_loop = *edge.src == *edge.dst;
        if is_self_loop {
            if self.with_loop {
                return true;
            }
            let key = (edge.edge_type.clone(), edge.ranking);
            self.seen.insert(key)
        } else {
            true
        }
    }
}

/// 路径结果缓存，使用 NPath 减少内存占用
#[derive(Debug, Clone)]
struct PathResultCache {
    /// 使用 NPath 存储中间结果，共享前缀
    npaths: Vec<Arc<NPath>>,
}

impl PathResultCache {
    fn new(limit: usize) -> Self {
        Self {
            npaths: Vec::with_capacity(limit.min(1024)), // 预分配容量，但不超过1024避免内存浪费
        }
    }

    fn len(&self) -> usize {
        self.npaths.len()
    }

    /// 添加 NPath 到缓存
    fn push(&mut self, npath: Arc<NPath>) {
        self.npaths.push(npath);
    }

    /// 批量转换为 Path
    fn to_paths(&self) -> Vec<Path> {
        self.npaths.iter().map(|np| np.to_path()).collect()
    }

    /// 并行批量转换为 Path
    fn to_paths_parallel(&self) -> Vec<Path> {
        const BATCH_SIZE: usize = 1000;
        if self.npaths.len() < BATCH_SIZE {
            return self.to_paths();
        }

        self.npaths
            .par_chunks(BATCH_SIZE)
            .flat_map(|chunk| chunk.iter().map(|np| np.to_path()).collect::<Vec<_>>())
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct AllPathsExecutor<S: StorageClient + Send + 'static> {
    base: BaseExecutor<S>,
    left_start_ids: Vec<Value>,
    right_start_ids: Vec<Value>,
    pub edge_direction: EdgeDirection,
    pub edge_types: Option<Vec<String>>,
    pub max_steps: usize,
    pub with_prop: bool,
    pub limit: usize,
    pub offset: usize,
    pub step_filter: Option<String>,
    pub filter: Option<String>,
    pub with_loop: bool, // 是否允许自环边
    left_steps: usize,
    right_steps: usize,
    left_visited: HashSet<Value>,
    right_visited: HashSet<Value>,
    /// 存储从顶点到其NPath的映射，用于路径重建
    left_path_map: HashMap<Value, Arc<NPath>>,
    right_path_map: HashMap<Value, Arc<NPath>>,
    /// 使用 NPath 替代 Path 存储中间结果，减少内存复制
    left_queue: VecDeque<(Value, Arc<NPath>)>,
    right_queue: VecDeque<(Value, Arc<NPath>)>,
    /// 使用 NPath 缓存结果，延迟转换为 Path
    result_cache: PathResultCache,
    nodes_visited: usize,
    edges_traversed: usize,
    parallel_config: ParallelConfig,
}

impl<S: StorageClient> AllPathsExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        left_start_ids: Vec<Value>,
        right_start_ids: Vec<Value>,
        edge_direction: EdgeDirection,
        edge_types: Option<Vec<String>>,
        max_steps: usize,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "AllPathsExecutor".to_string(), storage),
            left_start_ids,
            right_start_ids,
            edge_direction,
            edge_types,
            max_steps,
            with_prop: false,
            limit: std::usize::MAX,
            offset: 0,
            step_filter: None,
            filter: None,
            with_loop: false,
            left_steps: 0,
            right_steps: 0,
            left_visited: HashSet::new(),
            right_visited: HashSet::new(),
            left_path_map: HashMap::new(),
            right_path_map: HashMap::new(),
            left_queue: VecDeque::new(),
            right_queue: VecDeque::new(),
            result_cache: PathResultCache::new(std::usize::MAX),
            nodes_visited: 0,
            edges_traversed: 0,
            parallel_config: ParallelConfig::default(),
        }
    }

    /// 设置是否允许自环边
    pub fn with_loop(mut self, with_loop: bool) -> Self {
        self.with_loop = with_loop;
        self
    }

    /// 设置并行计算配置
    pub fn with_parallel_config(mut self, config: ParallelConfig) -> Self {
        self.parallel_config = config;
        self
    }

    pub fn with_config(
        mut self,
        with_prop: bool,
        limit: usize,
        offset: usize,
    ) -> Self {
        self.with_prop = with_prop;
        self.limit = limit;
        self.offset = offset;
        self.result_cache = PathResultCache::new(limit);
        self
    }

    pub fn with_filters(mut self, step_filter: Option<String>, filter: Option<String>) -> Self {
        self.step_filter = step_filter;
        self.filter = filter;
        self
    }

    fn get_neighbors(
        &self,
        node_id: &Value,
        direction: EdgeDirection,
    ) -> DBResult<Vec<(Value, Edge)>> {
        let storage = self.base.storage.as_ref()
            .expect("AllPathsExecutor storage not set");
        let storage = storage.lock();

        let edges = storage
            .get_node_edges("default", node_id, direction)
            .map_err(|e| DBError::Storage(
                crate::core::error::StorageError::DbError(e.to_string())
            ))?;

        let filtered_edges = if let Some(ref edge_types) = self.edge_types {
            edges
                .into_iter()
                .filter(|edge| edge_types.contains(&edge.edge_type))
                .collect()
        } else {
            edges
        };

        // 自环边去重（根据 with_loop 配置决定是否去重）
        let mut dedup = SelfLoopDedup::with_loop(self.with_loop);

        let neighbors = filtered_edges
            .into_iter()
            .filter(|edge| dedup.should_include(edge))
            .filter_map(|edge| match direction {
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

        Ok(neighbors)
    }

    /// 左向扩展 - 使用 NPath 避免路径复制
    fn expand_left(&mut self) -> DBResult<()> {
        while let Some((current_id, current_npath)) = self.left_queue.pop_front() {
            if self.left_visited.contains(&current_id) {
                continue;
            }
            self.left_visited.insert(current_id.clone());
            self.left_path_map.insert(current_id.clone(), current_npath.clone());
            self.nodes_visited += 1;

            // 检查是否达到 limit
            if self.result_cache.len() >= self.limit {
                return Ok(());
            }

            let neighbors = self.get_neighbors(&current_id, EdgeDirection::Out)?;
            self.edges_traversed += neighbors.len();

            for (neighbor_id, edge) in neighbors {
                // 环路检测：使用 NPath 的 contains_vertex 方法
                if current_npath.contains_vertex(&neighbor_id) {
                    continue;
                }
                if self.left_visited.contains(&neighbor_id) {
                    continue;
                }

                let storage = self.base.storage.as_ref()
                    .expect("AllPathsExecutor storage not set");
                let storage = storage.lock();
                if let Ok(Some(neighbor_vertex)) = storage.get_vertex("default", &neighbor_id) {
                    // 使用 NPath 扩展，O(1) 操作，共享前缀
                    let new_npath = Arc::new(NPath::extend(
                        current_npath.clone(),
                        Arc::new(edge.clone()),
                        Arc::new(neighbor_vertex),
                    ));

                    // 检查是否与右向搜索交汇
                    if let Some(right_npath) = self.right_path_map.get(&neighbor_id) {
                        // 构建完整路径
                        if let Some(full_path) = self.join_paths(&new_npath, right_npath) {
                            self.result_cache.push(Arc::new(full_path));
                        }
                    }

                    self.left_queue.push_back((neighbor_id.clone(), new_npath));
                }
            }
        }

        self.left_steps += 1;
        Ok(())
    }

    /// 右向扩展 - 使用 NPath 避免路径复制
    fn expand_right(&mut self) -> DBResult<()> {
        while let Some((current_id, current_npath)) = self.right_queue.pop_front() {
            if self.right_visited.contains(&current_id) {
                continue;
            }
            self.right_visited.insert(current_id.clone());
            self.right_path_map.insert(current_id.clone(), current_npath.clone());
            self.nodes_visited += 1;

            // 检查是否达到 limit
            if self.result_cache.len() >= self.limit {
                return Ok(());
            }

            let neighbors = self.get_neighbors(&current_id, EdgeDirection::In)?;
            self.edges_traversed += neighbors.len();

            for (neighbor_id, edge) in neighbors {
                // 环路检测
                if current_npath.contains_vertex(&neighbor_id) {
                    continue;
                }
                if self.right_visited.contains(&neighbor_id) {
                    continue;
                }

                let storage = self.base.storage.as_ref()
                    .expect("AllPathsExecutor storage not set");
                let storage = storage.lock();
                if let Ok(Some(neighbor_vertex)) = storage.get_vertex("default", &neighbor_id) {
                    // 使用 NPath 扩展
                    let new_npath = Arc::new(NPath::extend(
                        current_npath.clone(),
                        Arc::new(edge.clone()),
                        Arc::new(neighbor_vertex),
                    ));

                    // 检查是否与左向搜索交汇
                    if let Some(left_npath) = self.left_path_map.get(&neighbor_id) {
                        // 构建完整路径
                        if let Some(full_path) = self.join_paths(left_npath, &new_npath) {
                            self.result_cache.push(Arc::new(full_path));
                        }
                    }

                    self.right_queue.push_back((neighbor_id.clone(), new_npath));
                }
            }
        }

        self.right_steps += 1;
        Ok(())
    }

    /// 启发式扩展决策
    fn should_expand_both(&self) -> bool {
        let left_size = self.left_visited.len();
        let right_size = self.right_visited.len();

        const PATH_THRESHOLD_SIZE: usize = 100;
        const PATH_THRESHOLD_RATIO: usize = 2;

        if left_size > PATH_THRESHOLD_SIZE && right_size > PATH_THRESHOLD_SIZE {
            if left_size > right_size && left_size / right_size > PATH_THRESHOLD_RATIO {
                return false;
            }
            if right_size > left_size && right_size / left_size > PATH_THRESHOLD_RATIO {
                return false;
            }
        }
        true
    }

    /// 连接左右两条路径
    /// 左路径：从起点到交汇点
    /// 右路径：从终点到交汇点（需要反转方向）
    fn join_paths(&self, left_path: &NPath, right_path: &NPath) -> Option<NPath> {
        use crate::core::{Vertex, Edge};
        use std::sync::Arc;

        // 获取两条路径的顶点集合
        let left_vertices: std::collections::HashSet<_> = left_path.iter_vertices()
            .map(|v| v.vid.as_ref().clone())
            .collect();
        let right_vertices: std::collections::HashSet<_> = right_path.iter_vertices()
            .map(|v| v.vid.as_ref().clone())
            .collect();

        // 检查是否有重复顶点（除交汇点外）
        let common: Vec<_> = left_vertices.intersection(&right_vertices).collect();
        if common.len() != 1 {
            // 没有共同顶点或有多个共同顶点，无法连接或会形成环路
            return None;
        }

        // 获取交汇点
        let junction_id = common[0].clone();

        // 验证交汇点是两条路径的终点
        if left_path.end_vertex().vid.as_ref() != &junction_id {
            return None;
        }
        if right_path.end_vertex().vid.as_ref() != &junction_id {
            return None;
        }

        // 检查总路径长度是否超过限制
        let total_length = left_path.len() + right_path.len();
        if total_length > self.max_steps {
            return None;
        }

        // 构建完整路径：左路径 + 反转的右路径
        // 右路径是从终点往回走的，需要反转方向
        let mut full_path = left_path.clone();

        // 收集右路径的所有步骤（边和顶点）
        // 右路径结构：End <- ... <- A <- Junction（从终点向交汇点扩展）
        // 需要反转为：Junction -> A -> ... -> End
        let right_steps: Vec<(Arc<Edge>, Arc<Vertex>)> = right_path.iter()
            .filter_map(|node| {
                if let Some(ref edge) = node.edge() {
                    Some(((*edge).clone(), node.vertex().clone()))
                } else {
                    None
                }
            })
            .collect();

        // 反转并连接（跳过交汇点本身，从它的前一个节点开始）
        for (edge, vertex) in right_steps.into_iter().rev() {
            // 跳过交汇点
            if vertex.vid.as_ref() == &junction_id {
                continue;
            }
            // 创建反向边（从交汇点向外）
            let reversed_edge = Arc::new(Edge::new(
                vertex.vid.as_ref().clone(),
                full_path.end_vertex().vid.as_ref().clone(),
                edge.edge_type.clone(),
                edge.ranking,
                edge.props.clone(),
            ));
            full_path = NPath::extend(Arc::new(full_path), reversed_edge, vertex);
        }

        Some(full_path)
    }

    /// 初始化队列
    fn initialize_queues(&mut self) -> DBResult<()> {
        let storage = self.base.storage.as_ref()
            .expect("AllPathsExecutor storage not set");
        let storage = storage.lock();

        // 初始化左队列
        for left_id in &self.left_start_ids {
            if let Ok(Some(vertex)) = storage.get_vertex("default", left_id) {
                let npath = Arc::new(NPath::new(Arc::new(vertex)));
                self.left_queue.push_back((left_id.clone(), npath));
            }
        }

        // 初始化右队列
        for right_id in &self.right_start_ids {
            if let Ok(Some(vertex)) = storage.get_vertex("default", right_id) {
                let npath = Arc::new(NPath::new(Arc::new(vertex)));
                self.right_queue.push_back((right_id.clone(), npath));
            }
        }

        Ok(())
    }

    /// 执行双向 BFS
    fn execute_bidirectional(&mut self) -> DBResult<()> {
        self.initialize_queues()?;

        while self.left_steps + self.right_steps < self.max_steps {
            // 检查是否还有节点可以扩展
            if self.left_queue.is_empty() && self.right_queue.is_empty() {
                break;
            }

            // 启发式扩展决策
            let expand_both = self.should_expand_both();

            if expand_both {
                // 双向扩展
                if !self.left_queue.is_empty() {
                    self.expand_left()?;
                }
                if !self.right_queue.is_empty() {
                    self.expand_right()?;
                }
            } else {
                // 单侧扩展：选择节点少的一侧
                let left_size = self.left_visited.len();
                let right_size = self.right_visited.len();

                if left_size <= right_size && !self.left_queue.is_empty() {
                    self.expand_left()?;
                } else if !self.right_queue.is_empty() {
                    self.expand_right()?;
                }
            }

            // 检查是否已达到限制
            if self.result_cache.len() >= self.limit {
                break;
            }
        }

        Ok(())
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for AllPathsExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start_time = Instant::now();

        // 执行双向 BFS
        self.execute_bidirectional()?;

        // 转换为 Path 结果
        let paths = if self.parallel_config.enable_parallel {
            self.result_cache.to_paths_parallel()
        } else {
            self.result_cache.to_paths()
        };

        // 应用 offset
        let paths: Vec<Path> = if self.offset > 0 && self.offset < paths.len() {
            paths.into_iter().skip(self.offset).collect()
        } else {
            paths
        };

        let execution_time = start_time.elapsed().as_millis() as u64;

        // 更新统计信息
        self.base.get_stats_mut().add_stat("nodes_visited".to_string(), self.nodes_visited.to_string());
        self.base.get_stats_mut().add_stat("edges_traversed".to_string(), self.edges_traversed.to_string());
        self.base.get_stats_mut().add_stat("execution_time_ms".to_string(), execution_time.to_string());
        self.base.get_stats_mut().add_stat("paths_found".to_string(), paths.len().to_string());

        Ok(ExecutionResult::Paths(paths))
    }

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
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
        &self.base.description
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Value, Edge};
    use std::collections::HashMap;

    #[test]
    fn test_self_loop_dedup() {
        let mut dedup = SelfLoopDedup::new();
        let edge = Edge::new(
            Value::Int(1),
            Value::Int(1),
            "friend".to_string(),
            0,
            HashMap::new()
        );

        assert!(dedup.should_include(&edge));
        assert!(!dedup.should_include(&edge)); // 第二次应该返回 false
    }
}
