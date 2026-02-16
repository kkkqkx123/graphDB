//! A*最短路径算法
//!
//! 使用启发式函数的A*搜索算法

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::sync::Arc;

use crate::core::{Edge, Path, Step, Value, Vertex};
use crate::query::QueryError;
use crate::storage::StorageClient;
use parking_lot::Mutex;

use super::types::{AlgorithmStats, has_duplicate_edges};
use super::traits::ShortestPathAlgorithm;

/// A*算法节点
#[derive(Debug, Clone)]
pub struct AStarNode {
    /// 从起点到当前节点的实际代价
    pub g_cost: f64,
    /// 启发式估计代价（到终点的估计）
    pub h_cost: f64,
    /// 总代价 = g_cost + h_cost
    pub f_cost: f64,
    pub vertex_id: Value,
}

impl Eq for AStarNode {}

impl PartialEq for AStarNode {
    fn eq(&self, other: &Self) -> bool {
        self.f_cost == other.f_cost && self.vertex_id == other.vertex_id
    }
}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.f_cost.partial_cmp(&self.f_cost).unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// A*最短路径算法
pub struct AStar<S: StorageClient> {
    storage: Arc<Mutex<S>>,
    stats: AlgorithmStats,
    edge_direction: crate::core::types::EdgeDirection,
    /// 启发式函数
    heuristic: Box<dyn Fn(&Value, &Value) -> f64 + Send>,
}

impl<S: StorageClient> AStar<S> {
    pub fn new(storage: Arc<Mutex<S>>) -> Self {
        Self {
            storage,
            stats: AlgorithmStats::new(),
            edge_direction: crate::core::types::EdgeDirection::Both,
            heuristic: Box::new(|_current: &Value, _end: &Value| -> f64 {
                // 默认启发式函数返回0，退化为Dijkstra
                0.0
            }),
        }
    }

    pub fn with_edge_direction(mut self, direction: crate::core::types::EdgeDirection) -> Self {
        self.edge_direction = direction;
        self
    }

    pub fn with_heuristic<F>(mut self, heuristic: F) -> Self
    where
        F: Fn(&Value, &Value) -> f64 + Send + 'static,
    {
        self.heuristic = Box::new(heuristic);
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

        let neighbors_with_edges: Vec<(Value, Edge, f64)> = filtered_edges
            .into_iter()
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

    /// 根据前驱映射重建路径
    fn reconstruct_path(
        &self,
        end_id: &Value,
        previous_map: &HashMap<Value, (Value, Edge)>,
        start_ids: &[Value],
    ) -> Result<Option<Path>, QueryError> {
        let mut path_edges: Vec<(Value, Edge)> = Vec::new();
        let mut current = end_id.clone();

        while let Some((prev_id, edge)) = previous_map.get(&current) {
            path_edges.push((current.clone(), edge.clone()));
            current = prev_id.clone();

            if start_ids.contains(&current) {
                // 找到起点，构建路径
                let start_vertex = match self.get_vertex(&current)? {
                    Some(v) => v,
                    None => return Ok(None),
                };

                let mut path = Path {
                    src: Box::new(start_vertex),
                    steps: Vec::new(),
                };

                // 反向遍历构建路径
                path_edges.reverse();
                for (dst_id, edge) in path_edges {
                    let dst_vertex = match self.get_vertex(&dst_id)? {
                        Some(v) => v,
                        None => return Ok(None),
                    };

                    path.steps.push(Step {
                        dst: Box::new(dst_vertex),
                        edge: Box::new(edge),
                    });
                }

                return Ok(Some(path));
            }
        }

        Ok(None)
    }
}

impl<S: StorageClient> ShortestPathAlgorithm for AStar<S> {
    fn find_paths(
        &mut self,
        start_ids: &[Value],
        end_ids: &[Value],
        edge_types: Option<&[String]>,
        max_depth: Option<usize>,
        single_shortest: bool,
        limit: usize,
    ) -> Result<Vec<Path>, QueryError> {
        // 使用第一个终点作为启发式函数的目标
        let target = end_ids.first().cloned();

        let mut g_cost_map: HashMap<Value, f64> = HashMap::new();
        let mut previous_map: HashMap<Value, (Value, Edge)> = HashMap::new();
        let mut closed_set: HashSet<Value> = HashSet::new();
        let mut open_set: HashSet<Value> = HashSet::new();
        let mut priority_queue: BinaryHeap<Reverse<AStarNode>> = BinaryHeap::new();

        for start_id in start_ids {
            let h_cost = target.as_ref()
                .map(|t| (self.heuristic)(start_id, t))
                .unwrap_or(0.0);

            g_cost_map.insert(start_id.clone(), 0.0);
            open_set.insert(start_id.clone());
            priority_queue.push(Reverse(AStarNode {
                g_cost: 0.0,
                h_cost,
                f_cost: h_cost,
                vertex_id: start_id.clone(),
            }));
        }

        let mut result_paths = Vec::new();

        while let Some(Reverse(current)) = priority_queue.pop() {
            if single_shortest && !result_paths.is_empty() {
                break;
            }

            if result_paths.len() >= limit {
                break;
            }

            if closed_set.contains(&current.vertex_id) {
                continue;
            }

            closed_set.insert(current.vertex_id.clone());
            open_set.remove(&current.vertex_id);
            self.stats.increment_nodes_visited();

            if end_ids.contains(&current.vertex_id) {
                if let Some(path) = self.reconstruct_path(&current.vertex_id, &previous_map, start_ids)? {
                    if !has_duplicate_edges(&path) {
                        result_paths.push(path);
                    }
                }
                continue;
            }

            if let Some(max_d) = max_depth {
                if current.g_cost as usize >= max_d {
                    continue;
                }
            }

            let neighbors = self.get_neighbors_with_edges(&current.vertex_id, edge_types)?;
            self.stats.increment_edges_traversed(neighbors.len());

            for (neighbor_id, edge, weight) in neighbors {
                if closed_set.contains(&neighbor_id) {
                    continue;
                }

                let tentative_g_cost = current.g_cost + weight;
                let existing_g_cost = g_cost_map.get(&neighbor_id).unwrap_or(&f64::INFINITY);

                if tentative_g_cost < *existing_g_cost {
                    g_cost_map.insert(neighbor_id.clone(), tentative_g_cost);
                    previous_map.insert(neighbor_id.clone(), (current.vertex_id.clone(), edge.clone()));

                    let h_cost = target.as_ref()
                        .map(|t| (self.heuristic)(&neighbor_id, t))
                        .unwrap_or(0.0);

                    let f_cost = tentative_g_cost + h_cost;

                    priority_queue.push(Reverse(AStarNode {
                        g_cost: tentative_g_cost,
                        h_cost,
                        f_cost,
                        vertex_id: neighbor_id,
                    }));
                    open_set.insert(current.vertex_id.clone());
                }
            }
        }

        if single_shortest && !result_paths.is_empty() {
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
