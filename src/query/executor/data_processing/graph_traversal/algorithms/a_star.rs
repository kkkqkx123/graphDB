//! A*最短路径算法
//!
//! 使用启发式函数的A*搜索算法，支持带权图和多终点查询

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::sync::Arc;

use crate::core::{Edge, Path, Step, Value, Vertex};
use crate::query::QueryError;
use crate::storage::StorageClient;
use parking_lot::Mutex;

use super::types::{AlgorithmStats, EdgeWeightConfig, HeuristicFunction, SelfLoopDedup, has_duplicate_edges};
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
    /// 权重配置
    weight_config: EdgeWeightConfig,
    /// 启发式函数配置
    heuristic_config: HeuristicFunction,
}

impl<S: StorageClient> AStar<S> {
    pub fn new(storage: Arc<Mutex<S>>) -> Self {
        Self {
            storage,
            stats: AlgorithmStats::new(),
            edge_direction: crate::core::types::EdgeDirection::Both,
            weight_config: EdgeWeightConfig::Unweighted,
            heuristic_config: HeuristicFunction::Zero,
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

    pub fn with_heuristic(mut self, heuristic: HeuristicFunction) -> Self {
        self.heuristic_config = heuristic;
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

    /// 计算启发式值
    /// 对于多终点，使用到最近终点的启发式值
    fn calculate_heuristic(
        &self,
        current_id: &Value,
        end_ids: &[Value],
    ) -> Result<f64, QueryError> {
        if self.heuristic_config.is_zero() {
            return Ok(0.0);
        }

        // 获取当前节点属性
        let current_props = self.get_vertex_props(current_id)?;

        // 计算到所有终点的启发式值，取最小值
        let min_h = end_ids
            .iter()
            .filter_map(|end_id| {
                let end_props = self.get_vertex_props(end_id).ok()?;
                Some(self.heuristic_config.evaluate(
                    current_id,
                    end_id,
                    current_props.as_ref(),
                    end_props.as_ref(),
                ))
            })
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);

        Ok(min_h)
    }

    /// 获取顶点属性
    fn get_vertex_props(
        &self,
        vid: &Value,
    ) -> Result<Option<std::collections::HashMap<String, crate::core::Value>>, QueryError> {
        let storage = self.storage.lock();
        storage
            .get_vertex("default", vid)
            .map(|v| v.map(|vertex| vertex.properties))
            .map_err(|e| QueryError::StorageError(e.to_string()))
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
        let mut g_cost_map: HashMap<Value, f64> = HashMap::new();
        let mut previous_map: HashMap<Value, (Value, Edge)> = HashMap::new();
        let mut closed_set: HashSet<Value> = HashSet::new();
        let mut open_set: HashSet<Value> = HashSet::new();
        let mut priority_queue: BinaryHeap<Reverse<AStarNode>> = BinaryHeap::new();

        // 初始化起点
        for start_id in start_ids {
            let h_cost = self.calculate_heuristic(start_id, end_ids)?;

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

            // 检查是否到达终点
            if end_ids.contains(&current.vertex_id) {
                if let Some(path) = self.reconstruct_path(&current.vertex_id, &previous_map, start_ids)? {
                    if !has_duplicate_edges(&path) {
                        result_paths.push(path);
                    }
                }
                continue;
            }

            // 检查深度限制
            if let Some(max_d) = max_depth {
                if current.g_cost as usize >= max_d {
                    continue;
                }
            }

            // 扩展邻居
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

                    let h_cost = self.calculate_heuristic(&neighbor_id, end_ids)?;
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
