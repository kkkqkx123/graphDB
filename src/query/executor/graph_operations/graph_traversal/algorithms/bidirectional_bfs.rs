//! Bidirectional BFS (Broad-Search First) shortest path algorithm
//!
//! Use a bidirectional breadth-first search to find the shortest path.

use std::collections::HashMap;
use std::sync::Arc;

use crate::core::{Edge, NPath, Path, Vertex};
use crate::core::types::VertexId;
use crate::query::QueryError;
use crate::storage::StorageClient;
use parking_lot::RwLock;

use super::traits::ShortestPathAlgorithm;
use super::types::{
    combine_npaths, has_duplicate_edges, AlgorithmStats, BidirectionalBFSState, SelfLoopDedup,
};

/// Bidirectional BFS (Broad-Search First) shortest path algorithm
pub struct BidirectionalBFS<S: StorageClient> {
    storage: Arc<RwLock<S>>,
    stats: AlgorithmStats,
    edge_direction: crate::core::types::EdgeDirection,
}

impl<S: StorageClient> BidirectionalBFS<S> {
    pub fn new(storage: Arc<RwLock<S>>) -> Self {
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

    /// Obtaining neighbor nodes and edges
    fn get_neighbors_with_edges(
        &self,
        node_id: &VertexId,
        edge_types: Option<&[String]>,
    ) -> Result<Vec<(VertexId, Edge, f64)>, QueryError> {
        let storage = self.storage.read();

        let edges = storage
            .get_node_edges("default", node_id, self.edge_direction)
            .map_err(|e| QueryError::storage(e.to_string()))?;

        let filtered_edges = if let Some(types) = edge_types {
            edges
                .into_iter()
                .filter(|edge| types.contains(&edge.edge_type))
                .collect()
        } else {
            edges
        };

        // Remove duplicates from the self-loop edges.
        let mut dedup = SelfLoopDedup::new();

        let neighbors_with_edges: Vec<(VertexId, Edge, f64)> = filtered_edges
            .into_iter()
            .filter(|edge| dedup.should_include(edge))
            .filter_map(|edge| {
                let (neighbor_id, weight) = match self.edge_direction {
                    crate::core::types::EdgeDirection::In => {
                        if edge.dst == *node_id {
                            (edge.src, edge.ranking as f64)
                        } else {
                            return None;
                        }
                    }
                    crate::core::types::EdgeDirection::Out => {
                        if edge.src == *node_id {
                            (edge.dst, edge.ranking as f64)
                        } else {
                            return None;
                        }
                    }
                    crate::core::types::EdgeDirection::Both => {
                        if edge.src == *node_id {
                            (edge.dst, edge.ranking as f64)
                        } else if edge.dst == *node_id {
                            (edge.src, edge.ranking as f64)
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

    /// Obtain the vertices
    fn get_vertex(&self, vid: &VertexId) -> Result<Option<Vertex>, QueryError> {
        let storage = self.storage.read();
        storage
            .get_vertex("default", vid)
            .map_err(|e| QueryError::storage(e.to_string()))
    }
}

impl<S: StorageClient> ShortestPathAlgorithm for BidirectionalBFS<S> {
    fn find_paths(
        &mut self,
        start_ids: &[VertexId],
        end_ids: &[VertexId],
        edge_types: Option<&[String]>,
        max_depth: Option<usize>,
        single_shortest: bool,
        limit: usize,
    ) -> Result<Vec<Path>, QueryError> {
        let mut state = BidirectionalBFSState::new();
        let mut result_paths = Vec::new();
        let mut visited_left: HashMap<VertexId, Arc<NPath>> = HashMap::new();
        let mut visited_right: HashMap<VertexId, Arc<NPath>> = HashMap::new();
        let mut left_edges: Vec<HashMap<VertexId, Vec<(Edge, VertexId)>>> = Vec::new();
        let mut right_edges: Vec<HashMap<VertexId, Vec<(Edge, VertexId)>>> = Vec::new();

        for start_id in start_ids {
            if let Ok(Some(start_vertex)) = self.get_vertex(start_id) {
                let initial_npath = Arc::new(NPath::new(Arc::new(start_vertex)));
                state
                    .left_queue
                    .push_back((*start_id, initial_npath.clone()));
                visited_left.insert(*start_id, initial_npath);
            }
        }

        for end_id in end_ids {
            if let Ok(Some(end_vertex)) = self.get_vertex(end_id) {
                let initial_npath = Arc::new(NPath::new(Arc::new(end_vertex)));
                state
                    .right_queue
                    .push_back((*end_id, initial_npath.clone()));
                visited_right.insert(*end_id, initial_npath);
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
            let left_step_edges = left_edges
                .last_mut()
                .expect("left_edges should not be empty");

            while let Some((current_id, current_npath)) = state.left_queue.pop_front() {
                self.stats.increment_nodes_visited();

                if let Some(right_npath) = visited_right.get(&current_id) {
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
                        let new_npath = Arc::new(NPath::extend(
                            current_npath.clone(),
                            Arc::new(edge.clone()),
                            Arc::new(neighbor_vertex),
                        ));

                        state
                            .left_queue
                            .push_back((neighbor_id, new_npath.clone()));
                        visited_left.insert(neighbor_id, new_npath);
                        left_step_edges
                            .insert(neighbor_id, vec![(edge, current_id)]);
                    }
                }
            }

            if single_shortest && !result_paths.is_empty() {
                break;
            }

            right_edges.push(HashMap::new());
            let right_step_edges = right_edges
                .last_mut()
                .expect("right_edges should not be empty");

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
                        let new_npath = Arc::new(NPath::extend(
                            current_npath.clone(),
                            Arc::new(edge.clone()),
                            Arc::new(neighbor_vertex),
                        ));

                        state
                            .right_queue
                            .push_back((neighbor_id, new_npath.clone()));
                        visited_right.insert(neighbor_id, new_npath);
                        right_step_edges
                            .insert(neighbor_id, vec![(edge, current_id)]);
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
