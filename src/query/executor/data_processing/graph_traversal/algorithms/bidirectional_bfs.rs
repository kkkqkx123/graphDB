//! 双向BFS最短路径算法
//!
//! 使用双向广度优先搜索查找最短路径

use std::collections::HashMap;
use std::sync::Arc;

use crate::core::{Edge, NPath, Path, Value, Vertex};
use crate::query::QueryError;
use crate::storage::StorageClient;
use parking_lot::Mutex;

use super::types::{AlgorithmStats, BidirectionalBFSState, SelfLoopDedup, combine_npaths, has_duplicate_edges};
use super::traits::ShortestPathAlgorithm;

/// 双向BFS最短路径算法
pub struct BidirectionalBFS<S: StorageClient> {
    storage: Arc<Mutex<S>>,
    stats: AlgorithmStats,
    edge_direction: crate::core::types::EdgeDirection,
}

impl<S: StorageClient> BidirectionalBFS<S> {
    pub fn new(storage: Arc<Mutex<S>>) -> Self {
        Self {
            storage,
            stats: AlgorithmStats::new(),
            edge_direction: crate::core::types::EdgeDirection::Both,
        }
    }

    pub fn with_edge_direction(mut self, direction: crate::core::types::EdgeDirection) -> Self {
        self.edge_direction = direction;
        self
    }

    /// 获取邻居节点和边
    fn get_neighbors_with_edges(
        &self,
        node_id: &Value,
        edge_types: Option<&[String]>,
    ) -> Result<Vec<(Value, Edge, f64)>, QueryError> {
        let storage = self.storage.lock();

        let edges = storage
            .get_node_edges("default", node_id, self.edge_direction)
            .map_err(|e| QueryError::StorageError(e.to_string()))?;

        let filtered_edges = if let Some(types) = edge_types {
            edges
                .into_iter()
                .filter(|edge| types.contains(&edge.edge_type))
                .collect()
        } else {
            edges
        };

        // 自环边去重
        let mut dedup = SelfLoopDedup::new();

        let neighbors_with_edges: Vec<(Value, Edge, f64)> = filtered_edges
            .into_iter()
            .filter(|edge| dedup.should_include(edge))
            .filter_map(|edge| {
                let (neighbor_id, weight) = match self.edge_direction {
                    crate::core::types::EdgeDirection::In => {
                        if *edge.dst == *node_id {
                            ((*edge.src).clone(), edge.ranking as f64)
                        } else {
                            return None;
                        }
                    }
                    crate::core::types::EdgeDirection::Out => {
                        if *edge.src == *node_id {
                            ((*edge.dst).clone(), edge.ranking as f64)
                        } else {
                            return None;
                        }
                    }
                    crate::core::types::EdgeDirection::Both => {
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

    /// 获取顶点
    fn get_vertex(&self, vid: &Value) -> Result<Option<Vertex>, QueryError> {
        let storage = self.storage.lock();
        storage
            .get_vertex("default", vid)
            .map_err(|e| QueryError::StorageError(e.to_string()))
    }
}

impl<S: StorageClient> ShortestPathAlgorithm for BidirectionalBFS<S> {
    fn find_paths(
        &mut self,
        start_ids: &[Value],
        end_ids: &[Value],
        edge_types: Option<&[String]>,
        max_depth: Option<usize>,
        single_shortest: bool,
        limit: usize,
    ) -> Result<Vec<Path>, QueryError> {
        let mut state = BidirectionalBFSState::new();
        let mut result_paths = Vec::new();
        let mut visited_left: HashMap<Value, Arc<NPath>> = HashMap::new();
        let mut visited_right: HashMap<Value, Arc<NPath>> = HashMap::new();
        let mut left_edges: Vec<HashMap<Value, Vec<(Edge, Value)>>> = Vec::new();
        let mut right_edges: Vec<HashMap<Value, Vec<(Edge, Value)>>> = Vec::new();

        // 初始化左向队列（从起点开始）
        for start_id in start_ids {
            if let Ok(Some(start_vertex)) = self.get_vertex(start_id) {
                let initial_npath = Arc::new(NPath::new(Arc::new(start_vertex)));
                state.left_queue.push_back((start_id.clone(), initial_npath.clone()));
                visited_left.insert(start_id.clone(), initial_npath);
            }
        }

        // 初始化右向队列（从终点开始）
        for end_id in end_ids {
            if let Ok(Some(end_vertex)) = self.get_vertex(end_id) {
                let initial_npath = Arc::new(NPath::new(Arc::new(end_vertex)));
                state.right_queue.push_back((end_id.clone(), initial_npath.clone()));
                visited_right.insert(end_id.clone(), initial_npath);
            }
        }

        while !state.left_queue.is_empty() && !state.right_queue.is_empty() {
            if single_shortest && !result_paths.is_empty() {
                break;
            }

            if result_paths.len() >= limit {
                break;
            }

            left_edges.push(HashMap::new());
            let left_step_edges = left_edges.last_mut().expect("left_edges不应为空");

            // 左向扩展
            while let Some((current_id, current_npath)) = state.left_queue.pop_front() {
                self.stats.increment_nodes_visited();

                // 检查是否与右向路径交汇
                if let Some(right_npath) = visited_right.get(&current_id) {
                    // 拼接路径：左路径 + 反转的右路径
                    if let Some(combined_path) = combine_npaths(&current_npath, right_npath) {
                        if !has_duplicate_edges(&combined_path) {
                            result_paths.push(combined_path);

                            if single_shortest {
                                break;
                            }
                        }
                    }
                    continue;
                }

                // 检查深度限制
                if let Some(max_d) = max_depth {
                    if current_npath.len() >= max_d {
                        continue;
                    }
                }

                let neighbors = self.get_neighbors_with_edges(&current_id, edge_types)?;
                self.stats.increment_edges_traversed(neighbors.len());

                for (neighbor_id, edge, _weight) in neighbors {
                    if visited_left.contains_key(&neighbor_id) {
                        continue;
                    }

                    if let Ok(Some(neighbor_vertex)) = self.get_vertex(&neighbor_id) {
                        // 使用 NPath 扩展，O(1) 操作
                        let new_npath = Arc::new(NPath::extend(
                            current_npath.clone(),
                            Arc::new(edge.clone()),
                            Arc::new(neighbor_vertex),
                        ));

                        state.left_queue.push_back((neighbor_id.clone(), new_npath.clone()));
                        visited_left.insert(neighbor_id.clone(), new_npath);
                        left_step_edges.insert(neighbor_id.clone(), vec![(edge, current_id.clone())]);
                    }
                }
            }

            if single_shortest && !result_paths.is_empty() {
                break;
            }

            right_edges.push(HashMap::new());
            let right_step_edges = right_edges.last_mut().expect("right_edges不应为空");

            // 右向扩展
            while let Some((current_id, current_npath)) = state.right_queue.pop_front() {
                self.stats.increment_nodes_visited();

                if visited_left.contains_key(&current_id) {
                    continue;
                }

                if let Some(max_d) = max_depth {
                    if current_npath.len() >= max_d {
                        continue;
                    }
                }

                let neighbors = self.get_neighbors_with_edges(&current_id, edge_types)?;
                self.stats.increment_edges_traversed(neighbors.len());

                for (neighbor_id, edge, _weight) in neighbors {
                    if visited_right.contains_key(&neighbor_id) {
                        continue;
                    }

                    if let Ok(Some(neighbor_vertex)) = self.get_vertex(&neighbor_id) {
                        // 使用 NPath 扩展，O(1) 操作
                        let new_npath = Arc::new(NPath::extend(
                            current_npath.clone(),
                            Arc::new(edge.clone()),
                            Arc::new(neighbor_vertex),
                        ));

                        state.right_queue.push_back((neighbor_id.clone(), new_npath.clone()));
                        visited_right.insert(neighbor_id.clone(), new_npath);
                        right_step_edges.insert(neighbor_id.clone(), vec![(edge, current_id.clone())]);
                    }
                }
            }

            if state.left_queue.is_empty() && state.right_queue.is_empty() {
                break;
            }
        }

        if single_shortest && !result_paths.is_empty() {
            result_paths.sort_by(|a, b| a.steps.len().cmp(&b.steps.len()));
            result_paths.truncate(1);
        }

        if result_paths.len() > limit {
            result_paths.truncate(limit);
        }

        Ok(result_paths)
    }

    fn stats(&self) -> &AlgorithmStats {
        &self.stats
    }

    fn stats_mut(&mut self) -> &mut AlgorithmStats {
        &mut self.stats
    }
}
