//! Dijkstra最短路径算法
//!
//! 使用二叉堆优化的Dijkstra算法查找带权最短路径

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::sync::Arc;

use crate::core::{Edge, Path, Step, Value, Vertex};
use crate::query::QueryError;
use crate::storage::StorageClient;
use parking_lot::Mutex;

use super::types::{AlgorithmStats, DistanceNode, EdgeWeightConfig, SelfLoopDedup, has_duplicate_edges};
use super::traits::ShortestPathAlgorithm;

/// Dijkstra最短路径算法
pub struct Dijkstra<S: StorageClient> {
    storage: Arc<Mutex<S>>,
    stats: AlgorithmStats,
    edge_direction: crate::core::types::EdgeDirection,
    weight_config: EdgeWeightConfig,
}

impl<S: StorageClient> Dijkstra<S> {
    pub fn new(storage: Arc<Mutex<S>>) -> Self {
        Self {
            storage,
            stats: AlgorithmStats::new(),
            edge_direction: crate::core::types::EdgeDirection::Both,
            weight_config: EdgeWeightConfig::Unweighted,
        }
    }

    pub fn with_edge_direction(mut self, direction: crate::core::types::EdgeDirection) -> Self {
        self.edge_direction = direction;
        self
    }

    pub fn with_weight_config(mut self, config: EdgeWeightConfig) -> Self {
        self.weight_config = config;
        self
    }

    /// 获取边的权重
    fn get_edge_weight(&self, edge: &Edge) -> f64 {
        match &self.weight_config {
            EdgeWeightConfig::Unweighted => 1.0,
            EdgeWeightConfig::Ranking => edge.ranking as f64,
            EdgeWeightConfig::Property(prop_name) => {
                edge.get_property(prop_name)
                    .map(|v| match v {
                        crate::core::Value::Int(i) => *i as f64,
                        crate::core::Value::Float(f) => *f,
                        _ => 1.0,
                    })
                    .unwrap_or(1.0)
            }
        }
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
                let neighbor_id = match self.edge_direction {
                    crate::core::types::EdgeDirection::In => {
                        if *edge.dst == *node_id {
                            (*edge.src).clone()
                        } else {
                            return None;
                        }
                    }
                    crate::core::types::EdgeDirection::Out => {
                        if *edge.src == *node_id {
                            (*edge.dst).clone()
                        } else {
                            return None;
                        }
                    }
                    crate::core::types::EdgeDirection::Both => {
                        if *edge.src == *node_id {
                            (*edge.dst).clone()
                        } else if *edge.dst == *node_id {
                            (*edge.src).clone()
                        } else {
                            return None;
                        }
                    }
                };
                let weight = self.get_edge_weight(&edge);
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

impl<S: StorageClient> ShortestPathAlgorithm for Dijkstra<S> {
    fn find_paths(
        &mut self,
        start_ids: &[Value],
        end_ids: &[Value],
        edge_types: Option<&[String]>,
        max_depth: Option<usize>,
        single_shortest: bool,
        limit: usize,
    ) -> Result<Vec<Path>, QueryError> {
        let mut distance_map: HashMap<Value, f64> = HashMap::new();
        let mut previous_map: HashMap<Value, (Value, Edge)> = HashMap::new();
        let mut visited_nodes: HashSet<Value> = HashSet::new();
        let mut priority_queue: BinaryHeap<Reverse<DistanceNode>> = BinaryHeap::new();

        for start_id in start_ids {
            distance_map.insert(start_id.clone(), 0.0);
            priority_queue.push(Reverse(DistanceNode {
                distance: 0.0,
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

            if visited_nodes.contains(&current.vertex_id) {
                continue;
            }
            visited_nodes.insert(current.vertex_id.clone());
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
                if current.distance as usize >= max_d {
                    continue;
                }
            }

            let neighbors = self.get_neighbors_with_edges(&current.vertex_id, edge_types)?;
            self.stats.increment_edges_traversed(neighbors.len());

            for (neighbor_id, edge, weight) in neighbors {
                if visited_nodes.contains(&neighbor_id) {
                    continue;
                }

                let new_distance = current.distance + weight;
                let existing_distance = distance_map.get(&neighbor_id).unwrap_or(&f64::INFINITY);

                if new_distance < *existing_distance {
                    distance_map.insert(neighbor_id.clone(), new_distance);
                    previous_map.insert(neighbor_id.clone(), (current.vertex_id.clone(), edge.clone()));
                    priority_queue.push(Reverse(DistanceNode {
                        distance: new_distance,
                        vertex_id: neighbor_id,
                    }));
                }
            }
        }

        if single_shortest && !result_paths.is_empty() {
            result_paths.sort_by(|a, b| {
                let weight_a: f64 = a.steps.iter().map(|s| self.get_edge_weight(&s.edge)).sum();
                let weight_b: f64 = b.steps.iter().map(|s| self.get_edge_weight(&s.edge)).sum();
                weight_a.partial_cmp(&weight_b).unwrap_or(std::cmp::Ordering::Equal)
            });
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
