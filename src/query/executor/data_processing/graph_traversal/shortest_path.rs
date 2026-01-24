use async_trait::async_trait;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};

use crate::core::error::{DBError, DBResult};
use crate::core::{Edge, Path, Value};
use crate::core::vertex_edge_path::Step;
use crate::query::executor::base::{BaseExecutor, EdgeDirection, InputExecutor};
use crate::query::executor::traits::{ExecutionResult, Executor, HasStorage};
use crate::query::QueryError;
use crate::storage::StorageEngine;
use crate::utils::safe_lock;

/// 最短路径算法枚举
#[derive(Debug, Clone)]
pub enum ShortestPathAlgorithm {
    /// Dijkstra 算法
    Dijkstra,
    /// BFS 广度优先搜索
    BFS,
    /// A* 算法
    AStar,
}

/// ShortestPathExecutor - 最短路径执行器
///
/// 计算两个节点之间的最短路径，支持多种算法
/// 适用于社交网络、路线规划等场景
pub struct ShortestPathExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    start_vertex_ids: Vec<Value>,
    end_vertex_ids: Vec<Value>,
    pub edge_direction: EdgeDirection,
    pub edge_types: Option<Vec<String>>,
    pub max_depth: Option<usize>,     // 最大搜索深度限制
    algorithm: ShortestPathAlgorithm, // 使用的算法
    input_executor: Option<Box<dyn Executor<S>>>,
    // 路径缓存
    shortest_paths: Vec<Path>,
    // 算法状态
    visited_nodes: HashSet<Value>,
    distance_map: HashMap<Value, f64>,
    previous_map: HashMap<Value, (Value, Edge)>, // node -> (previous_node, edge)
    // 统计信息
    pub nodes_visited: usize,
    pub edges_traversed: usize,
    pub execution_time_ms: u64,
    pub max_depth_reached: usize,
}

// Manual Debug implementation for ShortestPathExecutor to avoid requiring Debug trait for Executor trait object
impl<S: StorageEngine> std::fmt::Debug for ShortestPathExecutor<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShortestPathExecutor")
            .field("base", &"BaseExecutor")
            .field("start_vertex_ids", &self.start_vertex_ids)
            .field("end_vertex_ids", &self.end_vertex_ids)
            .field("edge_direction", &self.edge_direction)
            .field("edge_types", &self.edge_types)
            .field("max_depth", &self.max_depth)
            .field("algorithm", &self.algorithm)
            .field("input_executor", &"Option<Box<dyn Executor<S>>>")
            .field("shortest_paths", &self.shortest_paths)
            .field("visited_nodes", &self.visited_nodes)
            .field("distance_map", &"HashMap<Value, f64>")
            .field("previous_map", &"HashMap<Value, (Value, Edge)>")
            .field("nodes_visited", &self.nodes_visited)
            .field("edges_traversed", &self.edges_traversed)
            .field("execution_time_ms", &self.execution_time_ms)
            .field("max_depth_reached", &self.max_depth_reached)
            .finish()
    }
}

impl<S: StorageEngine> ShortestPathExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        start_vertex_ids: Vec<Value>,
        end_vertex_ids: Vec<Value>,
        edge_direction: EdgeDirection,
        edge_types: Option<Vec<String>>,
        max_depth: Option<usize>,
        algorithm: ShortestPathAlgorithm,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "ShortestPathExecutor".to_string(), storage),
            start_vertex_ids,
            end_vertex_ids,
            edge_direction,
            edge_types,
            max_depth,
            algorithm,
            input_executor: None,
            shortest_paths: Vec::new(),
            visited_nodes: HashSet::new(),
            distance_map: HashMap::new(),
            previous_map: HashMap::new(),
            nodes_visited: 0,
            edges_traversed: 0,
            execution_time_ms: 0,
            max_depth_reached: 0,
        }
    }

    /// 获取当前使用的最短路径算法
    pub fn get_algorithm(&self) -> ShortestPathAlgorithm {
        self.algorithm.clone()
    }

    /// 设置最短路径算法
    pub fn set_algorithm(&mut self, algorithm: ShortestPathAlgorithm) {
        self.algorithm = algorithm;
    }

    /// 获取起始节点ID列表
    pub fn get_start_vertex_ids(&self) -> &Vec<Value> {
        &self.start_vertex_ids
    }

    /// 获取结束节点ID列表
    pub fn get_end_vertex_ids(&self) -> &Vec<Value> {
        &self.end_vertex_ids
    }

    /// 设置起始节点ID列表
    pub fn set_start_vertex_ids(&mut self, ids: Vec<Value>) {
        self.start_vertex_ids = ids;
    }

    /// 设置结束节点ID列表
    pub fn set_end_vertex_ids(&mut self, ids: Vec<Value>) {
        self.end_vertex_ids = ids;
    }

    /// 获取节点的邻居节点和对应的边
    async fn get_neighbors_with_edges(
        &self,
        node_id: &Value,
    ) -> Result<Vec<(Value, Edge, f64)>, QueryError> {
        let storage = safe_lock(&*self.get_storage())
            .expect("ShortestPathExecutor storage lock should not be poisoned");

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
            .filter_map(|edge| {
                let (neighbor_id, weight) = match self.edge_direction {
                    EdgeDirection::In => {
                        if *edge.dst == *node_id {
                            // 对于最短路径，我们可以使用边的ranking作为权重
                            ((*edge.src).clone(), edge.ranking as f64)
                        } else {
                            return None;
                        }
                    }
                    EdgeDirection::Out => {
                        if *edge.src == *node_id {
                            ((*edge.dst).clone(), edge.ranking as f64)
                        } else {
                            return None;
                        }
                    }
                    EdgeDirection::Both => {
                        if *edge.src == *node_id {
                            ((*edge.dst).clone(), edge.ranking as f64)
                        } else if *edge.dst == *node_id {
                            ((*edge.src).clone(), edge.ranking as f64)
                        } else {
                            return None;
                        }
                    }
                };
                Some((neighbor_id, edge, weight))
            })
            .collect();

        Ok(neighbors_with_edges)
    }

    /// BFS算法实现
    async fn bfs_shortest_path(&mut self) -> Result<(), QueryError> {
        let mut queue = VecDeque::new();
        let mut path_map: HashMap<Value, Path> = HashMap::new();

        // 初始化队列
        for start_id in &self.start_vertex_ids {
            let storage = safe_lock(&*self.get_storage())
                .expect("ShortestPathExecutor storage lock should not be poisoned");
            if let Ok(Some(start_vertex)) = storage.get_node(start_id) {
                let initial_path = Path {
                    src: Box::new(start_vertex),
                    steps: Vec::new(),
                };
                queue.push_back((start_id.clone(), initial_path.clone()));
                path_map.insert(start_id.clone(), initial_path);
            }
        }

        while let Some((current_id, current_path)) = queue.pop_front() {
            // 检查是否到达目标节点
            if self.end_vertex_ids.contains(&current_id) {
                self.shortest_paths.push(current_path);
                continue;
            }

            // 获取邻居节点
            let neighbors = self.get_neighbors_with_edges(&current_id).await?;

            for (neighbor_id, edge, _weight) in neighbors {
                // 如果已经访问过，跳过
                if path_map.contains_key(&neighbor_id) {
                    continue;
                }

                // 创建新路径
                let storage = safe_lock(&*self.get_storage())
                    .expect("ShortestPathExecutor storage lock should not be poisoned");
                if let Ok(Some(neighbor_vertex)) = storage.get_node(&neighbor_id) {
                    let mut new_path = current_path.clone();
                    new_path.steps.push(Step {
                        dst: Box::new(neighbor_vertex),
                        edge: Box::new(edge),
                    });

                    queue.push_back((neighbor_id.clone(), new_path.clone()));
                    path_map.insert(neighbor_id, new_path);
                }
            }
        }
        Ok(())
    }

    /// Dijkstra算法实现
    async fn dijkstra_shortest_path(&mut self) -> Result<(), QueryError> {
        // 初始化距离表
        for start_id in &self.start_vertex_ids {
            self.distance_map.insert(start_id.clone(), 0.0);
        }

        let mut priority_queue: Vec<(f64, Value)> = self
            .start_vertex_ids
            .iter()
            .map(|id| (0.0, id.clone()))
            .collect();

        while !priority_queue.is_empty() {
            // 找到距离最小的节点
            priority_queue.sort_by(|a, b| {
                a.0.partial_cmp(&b.0)
                    .expect("Distance values should be comparable")
            });
            let (current_distance, current_id) = priority_queue.remove(0);

            // 如果已经访问过，跳过
            if self.visited_nodes.contains(&current_id) {
                continue;
            }
            self.visited_nodes.insert(current_id.clone());

            // 检查是否到达目标节点
            if self.end_vertex_ids.contains(&current_id) {
                // 重建路径
                if let Some(path) = self.reconstruct_path(&current_id)? {
                    self.shortest_paths.push(path);
                }
                continue;
            }

            // 获取邻居节点
            let neighbors = self.get_neighbors_with_edges(&current_id).await?;

            for (neighbor_id, edge, weight) in neighbors {
                if self.visited_nodes.contains(&neighbor_id) {
                    continue;
                }

                let new_distance = current_distance + weight;
                let existing_distance = self
                    .distance_map
                    .get(&neighbor_id)
                    .unwrap_or(&f64::INFINITY);

                if new_distance < *existing_distance {
                    self.distance_map.insert(neighbor_id.clone(), new_distance);
                    self.previous_map
                        .insert(neighbor_id.clone(), (current_id.clone(), edge));
                    priority_queue.push((new_distance, neighbor_id));
                }
            }
        }
        Ok(())
    }

    /// 重建路径
    fn reconstruct_path(&self, end_id: &Value) -> Result<Option<Path>, QueryError> {
        let mut path_steps = Vec::new();
        let mut current_id = end_id.clone();

        // 回溯路径
        while let Some((prev_id, edge)) = self.previous_map.get(&current_id) {
            let storage = safe_lock(&*self.get_storage())
                .expect("ShortestPathExecutor storage lock should not be poisoned");
            if let Ok(Some(current_vertex)) = storage.get_node(&current_id) {
                path_steps.push(Step {
                    dst: Box::new(current_vertex),
                    edge: Box::new(edge.clone()),
                });
            }
            current_id = prev_id.clone();
        }

        // 检查起始节点
        if !self.start_vertex_ids.contains(&current_id) {
            return Ok(None);
        }

        // 获取起始节点
        let storage = safe_lock(&*self.get_storage())
            .expect("ShortestPathExecutor storage lock should not be poisoned");
        if let Ok(Some(start_vertex)) = storage.get_node(&current_id) {
            // 反转路径步骤
            path_steps.reverse();

            Ok(Some(Path {
                src: Box::new(start_vertex),
                steps: path_steps,
            }))
        } else {
            Ok(None)
        }
    }

    /// 执行最短路径计算
    async fn compute_shortest_paths(&mut self) -> Result<(), QueryError> {
        match self.algorithm {
            ShortestPathAlgorithm::BFS => {
                self.bfs_shortest_path().await?;
            }
            ShortestPathAlgorithm::Dijkstra => {
                self.dijkstra_shortest_path().await?;
            }
            ShortestPathAlgorithm::AStar => {
                // A*算法需要启发式函数，这里暂时使用Dijkstra
                self.dijkstra_shortest_path().await?;
            }
        }
        Ok(())
    }

    /// 构建结果
    fn build_result(&self) -> ExecutionResult {
        let mut path_values = Vec::new();

        for path in &self.shortest_paths {
            let mut path_value = Vec::new();

            // 添加起始节点
            path_value.push(Value::Vertex(path.src.clone()));

            // 添加每一步的边和节点
            for step in &path.steps {
                path_value.push(Value::Edge((*step.edge).clone()));
                path_value.push(Value::Vertex(step.dst.clone()));
            }

            path_values.push(Value::Path(path.clone()));
        }

        ExecutionResult::Values(path_values)
    }
}

impl<S: StorageEngine> InputExecutor<S> for ShortestPathExecutor<S> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>) {
        self.input_executor = Some(input);
    }

    fn get_input(&self) -> Option<&Box<dyn Executor<S>>> {
        self.input_executor.as_ref()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for ShortestPathExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        // 首先执行输入执行器（如果存在）
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute().await?
        } else {
            // 如果没有输入执行器，返回空结果
            ExecutionResult::Vertices(Vec::new())
        };

        // 提取起始和结束节点
        let (start_nodes, end_nodes) = match input_result {
            ExecutionResult::Vertices(vertices) => {
                if vertices.len() >= 2 {
                    (
                        vec![(*vertices[0].vid).clone()],
                        vec![(*vertices[1].vid).clone()],
                    )
                } else {
                    (Vec::new(), Vec::new())
                }
            }
            ExecutionResult::Edges(edges) => {
                if !edges.is_empty() {
                    let first_edge = &edges[0];
                    (
                        vec![(*first_edge.src).clone()],
                        vec![(*first_edge.dst).clone()],
                    )
                } else {
                    (Vec::new(), Vec::new())
                }
            }
            ExecutionResult::Values(values) => {
                if values.len() >= 2 {
                    (vec![values[0].clone()], vec![values[1].clone()])
                } else {
                    (Vec::new(), Vec::new())
                }
            }
            _ => (Vec::new(), Vec::new()),
        };

        // 如果没有提供起始和结束节点，使用预设的节点
        let start_nodes = if start_nodes.is_empty() {
            self.start_vertex_ids.clone()
        } else {
            start_nodes
        };

        let end_nodes = if end_nodes.is_empty() {
            self.end_vertex_ids.clone()
        } else {
            end_nodes
        };

        if start_nodes.is_empty() || end_nodes.is_empty() {
            return Ok(ExecutionResult::Values(Vec::new()));
        }

        // 更新起始和结束节点
        self.start_vertex_ids = start_nodes;
        self.end_vertex_ids = end_nodes;

        // 执行最短路径计算
        self.compute_shortest_paths().await.map_err(DBError::from)?;

        // 构建结果
        Ok(self.build_result())
    }

    fn open(&mut self) -> DBResult<()> {
        self.shortest_paths.clear();
        self.visited_nodes.clear();
        self.distance_map.clear();
        self.previous_map.clear();

        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.open()?;
        }
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        self.shortest_paths.clear();
        self.visited_nodes.clear();
        self.distance_map.clear();
        self.previous_map.clear();

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

impl<S: StorageEngine + Send> HasStorage<S> for ShortestPathExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("ShortestPathExecutor storage should be set")
    }
}

/// MultiShortestPathExecutor - 多源最短路径执行器
///
/// 计算多组起始节点到目标节点之间的最短路径
/// 使用双向 BFS 算法，同时从两侧扩展以提高效率
/// 适用于社交网络分析、推荐系统等场景
pub struct MultiShortestPathExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    left_start_vertices: Vec<Value>,
    right_target_vertices: Vec<Value>,
    max_steps: usize,
    edge_types: Option<Vec<String>>,
    single_shortest: bool,
    input_executor: Option<Box<dyn Executor<S>>>,
    left_visited: HashSet<Value>,
    right_visited: HashSet<Value>,
    left_paths: HashMap<Value, Vec<Path>>,
    right_paths: HashMap<Value, Vec<Path>>,
    history_left_paths: HashMap<Value, HashMap<Value, Vec<Path>>>,
    history_right_paths: HashMap<Value, HashMap<Value, Vec<Path>>>,
    current_step: usize,
    result_paths: Vec<Path>,
}

impl<S: StorageEngine> std::fmt::Debug for MultiShortestPathExecutor<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MultiShortestPathExecutor")
            .field("base", &"BaseExecutor")
            .field("left_start_vertices", &self.left_start_vertices)
            .field("right_target_vertices", &self.right_target_vertices)
            .field("max_steps", &self.max_steps)
            .field("single_shortest", &self.single_shortest)
            .field("input_executor", &"Option<Box<dyn Executor<S>>>")
            .field("current_step", &self.current_step)
            .finish()
    }
}

impl<S: StorageEngine> MultiShortestPathExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        left_start_vertices: Vec<Value>,
        right_target_vertices: Vec<Value>,
        max_steps: usize,
        edge_types: Option<Vec<String>>,
        single_shortest: bool,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "MultiShortestPathExecutor".to_string(), storage),
            left_start_vertices,
            right_target_vertices,
            max_steps,
            edge_types,
            single_shortest,
            input_executor: None,
            left_visited: HashSet::new(),
            right_visited: HashSet::new(),
            left_paths: HashMap::new(),
            right_paths: HashMap::new(),
            history_left_paths: HashMap::new(),
            history_right_paths: HashMap::new(),
            current_step: 1,
            result_paths: Vec::new(),
        }
    }

    fn get_neighbors_with_edges(
        &self,
        node_id: &Value,
    ) -> Result<Vec<(Value, Edge)>, QueryError> {
        let storage = safe_lock(&*self.get_storage())
            .expect("MultiShortestPathExecutor storage lock should not be poisoned");

        let edges = storage
            .get_node_edges(node_id, EdgeDirection::Both)
            .map_err(|e| QueryError::StorageError(e.to_string()))?;

        let filtered_edges = if let Some(ref edge_types) = self.edge_types {
            edges
                .into_iter()
                .filter(|edge| edge_types.contains(&edge.edge_type))
                .collect()
        } else {
            edges
        };

        Ok(filtered_edges
            .into_iter()
            .map(|edge| (*edge.dst.clone(), edge))
            .collect())
    }

    fn build_left_paths(&mut self) -> Result<(), QueryError> {
        self.left_paths.clear();

        let mut initial_data: Vec<(Value, Path, Value)> = Vec::new();
        for start_vertex in &self.left_start_vertices {
            let storage = safe_lock(&*self.get_storage())
                .expect("MultiShortestPathExecutor storage lock should not be poisoned");

            if let Ok(Some(vertex)) = storage.get_node(start_vertex) {
                let path = Path {
                    src: Box::new(vertex),
                    steps: Vec::new(),
                };
                initial_data.push((start_vertex.clone(), path, start_vertex.clone()));
            }
        }

        for (vid, path, visited_vid) in initial_data {
            self.left_paths.insert(vid, vec![path]);
            self.left_visited.insert(visited_vid);
        }

        let mut new_discoveries: Vec<(Value, Path)> = Vec::new();
        for (current_id, paths) in &self.left_paths {
            let neighbors = self.get_neighbors_with_edges(current_id)?;
            for (neighbor_id, edge) in neighbors {
                if self.left_visited.contains(&neighbor_id) {
                    continue;
                }

                self.left_visited.insert(neighbor_id.clone());

                let edge_clone = edge.clone();
                for path in paths {
                    let storage = safe_lock(&*self.get_storage())
                        .expect("MultiShortestPathExecutor storage lock should not be poisoned");

                    if let Ok(Some(dst_vertex)) = storage.get_node(&neighbor_id) {
                        let mut new_path = path.clone();
                        new_path.steps.push(Step {
                            dst: Box::new(dst_vertex),
                            edge: Box::new(edge_clone.clone()),
                        });
                        new_discoveries.push((neighbor_id.clone(), new_path));
                    }
                }
            }
        }

        for (vid, path) in new_discoveries {
            self.left_paths
                .entry(vid)
                .or_insert_with(Vec::new)
                .push(path);
        }

        Ok(())
    }

    fn build_right_paths(&mut self) -> Result<(), QueryError> {
        self.right_paths.clear();

        let mut initial_data: Vec<(Value, Path, Value)> = Vec::new();
        for target_vertex in &self.right_target_vertices {
            let storage = safe_lock(&*self.get_storage())
                .expect("MultiShortestPathExecutor storage lock should not be poisoned");

            if let Ok(Some(vertex)) = storage.get_node(target_vertex) {
                let path = Path {
                    src: Box::new(vertex),
                    steps: Vec::new(),
                };
                initial_data.push((target_vertex.clone(), path, target_vertex.clone()));
            }
        }

        for (vid, path, visited_vid) in initial_data {
            self.right_paths.insert(vid, vec![path]);
            self.right_visited.insert(visited_vid);
        }

        let mut new_discoveries: Vec<(Value, Path)> = Vec::new();
        for (current_id, paths) in &self.right_paths {
            let neighbors = self.get_neighbors_with_edges(current_id)?;
            for (neighbor_id, edge) in neighbors {
                if self.right_visited.contains(&neighbor_id) {
                    continue;
                }

                self.right_visited.insert(neighbor_id.clone());

                let edge_clone = edge.clone();
                for path in paths {
                    let storage = safe_lock(&*self.get_storage())
                        .expect("MultiShortestPathExecutor storage lock should not be poisoned");

                    if let Ok(Some(dst_vertex)) = storage.get_node(&neighbor_id) {
                        let mut new_path = path.clone();
                        new_path.steps.push(Step {
                            dst: Box::new(dst_vertex),
                            edge: Box::new(edge_clone.clone()),
                        });
                        new_discoveries.push((neighbor_id.clone(), new_path));
                    }
                }
            }
        }

        for (vid, path) in new_discoveries {
            self.right_paths
                .entry(vid)
                .or_insert_with(Vec::new)
                .push(path);
        }

        Ok(())
    }

    fn conjunct_paths(&mut self) {
        let meet_points: Vec<(Value, Vec<Path>, Vec<Path>)> = self
            .left_paths
            .iter()
            .filter_map(|(left_vid, left_path_list)| {
                self.right_paths
                    .get(left_vid)
                    .map(|right_path_list| {
                        (
                            left_vid.clone(),
                            left_path_list.clone(),
                            right_path_list.clone(),
                        )
                    })
            })
            .collect();

        for (_meet_vid, left_list, right_list) in meet_points {
            for left_path in left_list {
                for right_path in &right_list {
                    let mut combined_path = left_path.clone();
                    for step in right_path.steps.iter().rev() {
                        combined_path.steps.push(step.clone());
                    }
                    self.result_paths.push(combined_path);
                }
            }
        }
    }
}

impl<S: StorageEngine> InputExecutor<S> for MultiShortestPathExecutor<S> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>) {
        self.input_executor = Some(input);
    }

    fn get_input(&self) -> Option<&Box<dyn Executor<S>>> {
        self.input_executor.as_ref()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for MultiShortestPathExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        if self.left_start_vertices.is_empty() || self.right_target_vertices.is_empty() {
            return Ok(ExecutionResult::Paths(vec![]));
        }

        if self.current_step > self.max_steps {
            return Ok(ExecutionResult::Paths(self.result_paths.clone()));
        }

        let _ = self.build_left_paths();
        let _ = self.build_right_paths();

        self.conjunct_paths();

        self.current_step += 1;

        if self.single_shortest && !self.result_paths.is_empty() {
            self.result_paths.sort_by(|a, b| a.steps.len().cmp(&b.steps.len()));
            self.result_paths.truncate(1);
        }

        Ok(ExecutionResult::Paths(self.result_paths.clone()))
    }

    fn open(&mut self) -> DBResult<()> {
        self.base.open()?;
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        self.base.close()?;
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

impl<S: StorageEngine + Send> HasStorage<S> for MultiShortestPathExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("MultiShortestPathExecutor storage should be set")
    }
}
