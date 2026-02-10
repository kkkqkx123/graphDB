//! AllPaths 执行器
//!
//! 基于 Nebula 3.8.0 的 AllPathsExecutor 实现
//! 功能特点：
//! - 双向 BFS 算法
//! - 支持找到所有路径（非最短路径）
//! - 支持 noLoop 避免循环
//! - 支持 limit 和 offset 限制结果数量
//! - 支持 withProp 返回路径属性
//! - 使用两阶段扩展（左扩展和右扩展）
//! - 当节点数量超过阈值时使用启发式扩展
//! - CPU 密集型操作，使用 Rayon 进行并行化

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use rayon::prelude::*;

use crate::core::error::DBResult;
use crate::core::{Edge, Path, Value, Vertex};
use crate::core::vertex_edge_path::Step;
use crate::query::executor::base::{BaseExecutor, EdgeDirection, ExecutorStats};
use crate::query::executor::recursion_detector::ParallelConfig;
use crate::query::executor::traits::{ExecutionResult, Executor, HasStorage};
use crate::storage::StorageClient;
use crate::utils::safe_lock;

#[derive(Debug, Clone)]
pub struct AllPathsExecutor<S: StorageClient + Send + 'static> {
    base: BaseExecutor<S>,
    left_start_ids: Vec<Value>,
    right_start_ids: Vec<Value>,
    pub edge_direction: EdgeDirection,
    pub edge_types: Option<Vec<String>>,
    pub max_steps: usize,
    pub no_loop: bool,
    pub with_prop: bool,
    pub limit: usize,
    pub offset: usize,
    pub step_filter: Option<String>,
    pub filter: Option<String>,
    left_steps: usize,
    right_steps: usize,
    left_visited: HashSet<Value>,
    right_visited: HashSet<Value>,
    left_adj_list: HashMap<Value, Vec<(Edge, Value)>>,
    right_adj_list: HashMap<Value, Vec<(Edge, Value)>>,
    left_queue: VecDeque<(Value, Path)>,
    right_queue: VecDeque<(Value, Path)>,
    result_paths: Vec<Path>,
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
            no_loop: false,
            with_prop: false,
            limit: std::usize::MAX,
            offset: 0,
            step_filter: None,
            filter: None,
            left_steps: 0,
            right_steps: 0,
            left_visited: HashSet::new(),
            right_visited: HashSet::new(),
            left_adj_list: HashMap::new(),
            right_adj_list: HashMap::new(),
            left_queue: VecDeque::new(),
            right_queue: VecDeque::new(),
            result_paths: Vec::new(),
            nodes_visited: 0,
            edges_traversed: 0,
            parallel_config: ParallelConfig::default(),
        }
    }

    /// 设置并行计算配置
    pub fn with_parallel_config(mut self, config: ParallelConfig) -> Self {
        self.parallel_config = config;
        self
    }

    pub fn with_config(
        mut self,
        no_loop: bool,
        with_prop: bool,
        limit: usize,
        offset: usize,
    ) -> Self {
        self.no_loop = no_loop;
        self.with_prop = with_prop;
        self.limit = limit;
        self.offset = offset;
        self
    }

    pub fn with_filters(mut self, step_filter: Option<String>, filter: Option<String>) -> Self {
        self.step_filter = step_filter;
        self.filter = filter;
        self
    }

    fn get_edge_direction(&self) -> EdgeDirection {
        self.edge_direction.clone()
    }

    fn get_edge_types(&self) -> Option<Vec<String>> {
        self.edge_types.clone()
    }

    fn get_max_steps(&self) -> usize {
        self.max_steps
    }

    fn get_neighbors(
        &self,
        node_id: &Value,
        direction: EdgeDirection,
    ) -> DBResult<Vec<(Value, Edge)>> {
        let storage = safe_lock(&*self.base.get_storage())
            .expect("AllPathsExecutor storage lock should not be poisoned");

        let edges = storage
            .get_node_edges("default", node_id, direction)
            .map_err(|e| crate::core::error::DBError::Storage(
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

        let neighbors = filtered_edges
            .into_iter()
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
            .map(|(id, edge)| (id, edge.clone()))
            .collect();

        Ok(neighbors)
    }

    fn expand_left(
        &mut self,
    ) -> DBResult<Vec<(Value, Vec<(Edge, Value)>)>> {
        let mut expansions = Vec::new();

        while let Some((current_id, current_path)) = self.left_queue.pop_front() {
            if self.left_visited.contains(&current_id) {
                continue;
            }
            self.left_visited.insert(current_id.clone());
            self.nodes_visited += 1;

            let neighbors = self.get_neighbors(&current_id, EdgeDirection::Out)?;
            self.edges_traversed += neighbors.len();

            let mut valid_neighbors = Vec::new();
            for (neighbor_id, edge) in neighbors {
                if self.no_loop && self.left_visited.contains(&neighbor_id) {
                    continue;
                }
                if self.left_visited.contains(&neighbor_id) {
                    continue;
                }

                let storage = safe_lock(&*self.base.get_storage())
                    .expect("AllPathsExecutor storage lock should not be poisoned");
                if let Ok(Some(neighbor_vertex)) = storage.get_vertex("default", &neighbor_id) {
                    let mut new_path = current_path.clone();
                    new_path.steps.push(Step {
                        dst: Box::new(neighbor_vertex),
                        edge: Box::new(edge.clone()),
                    });

                    self.left_queue.push_back((neighbor_id.clone(), new_path));
                    valid_neighbors.push((edge.clone(), neighbor_id));
                }
            }

            if !valid_neighbors.is_empty() {
                expansions.push((current_id, valid_neighbors));
            }
        }

        self.left_steps += 1;
        Ok(expansions)
    }

    fn expand_right(
        &mut self,
    ) -> DBResult<Vec<(Value, Vec<(Edge, Value)>)>> {
        let mut expansions = Vec::new();

        while let Some((current_id, current_path)) = self.right_queue.pop_front() {
            if self.right_visited.contains(&current_id) {
                continue;
            }
            self.right_visited.insert(current_id.clone());
            self.nodes_visited += 1;

            let neighbors = self.get_neighbors(&current_id, EdgeDirection::In)?;
            self.edges_traversed += neighbors.len();

            let mut valid_neighbors = Vec::new();
            for (neighbor_id, edge) in neighbors {
                if self.no_loop && self.right_visited.contains(&neighbor_id) {
                    continue;
                }
                if self.right_visited.contains(&neighbor_id) {
                    continue;
                }

                let storage = safe_lock(&*self.base.get_storage())
                    .expect("AllPathsExecutor storage lock should not be poisoned");
                if let Ok(Some(neighbor_vertex)) = storage.get_vertex("default", &neighbor_id) {
                    let mut new_path = current_path.clone();
                    new_path.steps.push(Step {
                        dst: Box::new(neighbor_vertex),
                        edge: Box::new(edge.clone()),
                    });

                    self.right_queue.push_back((neighbor_id.clone(), new_path));
                    valid_neighbors.push((edge.clone(), neighbor_id));
                }
            }

            if !valid_neighbors.is_empty() {
                expansions.push((current_id, valid_neighbors));
            }
        }

        self.right_steps += 1;
        Ok(expansions)
    }

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

    fn has_same_vertices(&self, path: &Path, edge: &Edge) -> bool {
        let mut vertices: HashSet<Box<Value>> = HashSet::new();
        vertices.insert(path.src.vid.clone());
        for step in &path.steps {
            vertices.insert(step.dst.vid.clone());
        }
        vertices.contains(&edge.dst)
    }

    fn has_same_edge(&self, path: &Path, edge: &Edge) -> bool {
        for step in &path.steps {
            if step.edge.src == edge.src
                && step.edge.dst == edge.dst
                && step.edge.ranking == edge.ranking
            {
                return true;
            }
        }
        false
    }

    fn build_conjunct_paths(
        &self,
    ) -> DBResult<Vec<Path>> {
        let mut result_paths = Vec::new();

        for (left_vertex, left_edges) in &self.left_adj_list {
            for (left_edge, left_intermediate) in left_edges {
                if let Some(right_paths) = self.right_adj_list.get(left_intermediate) {
                    for (right_edge, right_vertex) in right_paths {
                        if self.no_loop {
                            if self.has_same_vertices(
                                &Path {
                                    src: Box::new(Vertex::new(
                                        left_vertex.clone(),
                                        Vec::new(),
                                    )),
                                    steps: Vec::new(),
                                },
                                left_edge,
                            ) {
                                continue;
                            }
                        }

                        let src_vertex = Vertex::new(
                            left_vertex.clone(),
                            Vec::new(),
                        );
                        let mid_vertex = Vertex::new(
                            left_intermediate.clone(),
                            Vec::new(),
                        );
                        let dst_vertex = Vertex::new(
                            right_vertex.clone(),
                            Vec::new(),
                        );

                        let mut path = Path {
                            src: Box::new(src_vertex),
                            steps: Vec::new(),
                        };

                        path.steps.push(Step {
                            dst: Box::new(mid_vertex),
                            edge: Box::new(left_edge.clone()),
                        });

                        path.steps.push(Step {
                            dst: Box::new(dst_vertex),
                            edge: Box::new(right_edge.clone()),
                        });

                        result_paths.push(path);
                    }
                }
            }
        }

        result_paths.sort_by(|a, b| a.steps.len().cmp(&b.steps.len()));

        if result_paths.len() > self.limit {
            result_paths.truncate(self.limit);
        }

        Ok(result_paths)
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for AllPathsExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start = Instant::now();

        let result = self.do_execute()?;

        self.base.get_stats_mut().add_total_time(start.elapsed());

        Ok(ExecutionResult::Paths(result))
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
        "AllPathsExecutor"
    }

    fn description(&self) -> &str {
        "All paths executor - finds all paths between vertices"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient + Send + Sync + 'static> AllPathsExecutor<S> {
    fn do_execute(&mut self) -> DBResult<Vec<Path>> {
        if self.left_start_ids.is_empty() || self.right_start_ids.is_empty() {
            return Ok(Vec::new());
        }

        if self.max_steps == 0 {
            return Ok(Vec::new());
        }

        {
            let storage = safe_lock(self.base.get_storage())?;

            for left_id in &self.left_start_ids {
                if let Ok(Some(vertex)) = storage.get_vertex("default", left_id) {
                    self.left_queue.push_back((
                        left_id.clone(),
                        Path {
                            src: Box::new(vertex),
                            steps: Vec::new(),
                        },
                    ));
                    self.left_visited.insert(left_id.clone());
                }
            }

            for right_id in &self.right_start_ids {
                if let Ok(Some(vertex)) = storage.get_vertex("default", right_id) {
                    self.right_queue.push_back((
                        right_id.clone(),
                        Path {
                            src: Box::new(vertex),
                            steps: Vec::new(),
                        },
                    ));
                    self.right_visited.insert(right_id.clone());
                }
            }
        }

        while self.left_steps + self.right_steps < self.max_steps
            && !self.left_queue.is_empty()
            && !self.right_queue.is_empty()
        {
            if self.result_paths.len() >= self.limit {
                break;
            }

            let expand_both = self.should_expand_both();

            if expand_both {
                let left_expansions = self.expand_left()?;
                for (vertex, edges) in left_expansions {
                    self.left_adj_list.insert(vertex, edges);
                }

                let right_expansions = self.expand_right()?;
                for (vertex, edges) in right_expansions {
                    self.right_adj_list.insert(vertex, edges);
                }
            } else {
                let left_size = self.left_visited.len();
                let right_size = self.right_visited.len();

                if left_size > right_size {
                    let right_expansions = self.expand_right()?;
                    for (vertex, edges) in right_expansions {
                        self.right_adj_list.insert(vertex, edges);
                    }
                } else {
                    let left_expansions = self.expand_left()?;
                    for (vertex, edges) in left_expansions {
                        self.left_adj_list.insert(vertex, edges);
                    }
                }
            }

            if self.left_steps + self.right_steps >= self.max_steps {
                break;
            }
        }

        let conjunct_paths = self.build_conjunct_paths()?;

        if self.left_steps == 0 {
            for path in conjunct_paths {
                self.result_paths.push(path);
            }
        }

        if self.right_steps == 0 {
            if self.parallel_config.should_use_parallel(self.left_adj_list.len()) {
                self.build_right_paths_parallel()?;
            } else {
                self.build_right_paths_sequential()?;
            }
        }

        if self.result_paths.len() > self.limit {
            self.result_paths.truncate(self.limit);
        }

        if self.offset > 0 && self.result_paths.len() > self.offset {
            self.result_paths = self.result_paths[self.offset..].to_vec();
        }

        Ok(self.result_paths.clone())
    }

    fn build_right_paths_sequential(&mut self) -> DBResult<()> {
        for (left_vertex, left_edges) in &self.left_adj_list {
            for (left_edge, left_intermediate) in left_edges {
                let src_vertex = Vertex::new(
                    left_vertex.clone(),
                    Vec::new(),
                );
                let dst_vertex = Vertex::new(
                    left_intermediate.clone(),
                    Vec::new(),
                );

                let mut path = Path {
                    src: Box::new(src_vertex),
                    steps: Vec::new(),
                };

                path.steps.push(Step {
                    dst: Box::new(dst_vertex),
                    edge: Box::new(left_edge.clone()),
                });

                self.result_paths.push(path);
            }
        }
        Ok(())
    }

    fn build_right_paths_parallel(&mut self) -> DBResult<()> {
        let left_adj_list = std::mem::take(&mut self.left_adj_list);

        let parallel_paths: Vec<Path> = left_adj_list
            .par_iter()
            .flat_map(|(left_vertex, left_edges)| {
                let mut paths = Vec::new();
                for (left_edge, left_intermediate) in left_edges {
                    let src_vertex = Vertex::new(
                        left_vertex.clone(),
                        Vec::new(),
                    );
                    let dst_vertex = Vertex::new(
                        left_intermediate.clone(),
                        Vec::new(),
                    );

                    let mut path = Path {
                        src: Box::new(src_vertex),
                        steps: Vec::new(),
                    };

                    path.steps.push(Step {
                        dst: Box::new(dst_vertex),
                        edge: Box::new(left_edge.clone()),
                    });

                    paths.push(path);
                }
                paths
            })
            .collect();

        self.result_paths.extend(parallel_paths);
        Ok(())
    }
}
