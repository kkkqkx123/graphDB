//! 最短路径规划器
//!
//! 负责规划最短路径查询，支持 BFS 等算法

use crate::core::types::graph_schema::EdgeDirection;
use crate::core::{Edge, StorageError, Value, Vertex};
use crate::query::planner::statements::seeks::seek_strategy_base::{NodePattern, SeekStrategyContext, SeekStrategySelector, SeekStrategyType};
use crate::storage::StorageClient;
use crate::storage::transaction::TransactionId;
use std::collections::{HashMap, HashSet, VecDeque};

pub type PlannerError = StorageError;

#[derive(Debug)]
pub struct ShortestPathPlanner;

impl ShortestPathPlanner {
    pub fn new() -> Self {
        Self
    }

    pub fn plan_shortest_path<S: StorageClient>(
        &self,
        _storage: &S,
        start: &NodePattern,
        end: &NodePattern,
        edge_pattern: &EdgePattern,
        space_id: i32,
    ) -> Result<ShortestPathPlan, PlannerError> {
        let bfs_config = BfsConfig {
            max_iterations: 10000,
            max_path_length: 100,
            direction: self.extract_direction(edge_pattern)?,
            edge_types: edge_pattern.types.clone().unwrap_or_default(),
        };

        let start_context = SeekStrategyContext::new(space_id, start.clone(), vec![]);
        let selector = SeekStrategySelector::new();
        let start_strategy = selector.select_strategy(&DummyStorage, &start_context);

        let start_finder = match start_strategy {
            SeekStrategyType::VertexSeek => StartVidSource::VertexSeek(start.clone()),
            SeekStrategyType::IndexSeek => StartVidSource::IndexScan(start.clone()),
            SeekStrategyType::ScanSeek => StartVidSource::FullScan(start.clone()),
        };

        let end_condition = EndCondition {
            pattern: end.clone(),
        };

        Ok(ShortestPathPlan {
            start: start_finder,
            end: end_condition,
            bfs_config,
        })
    }

    fn extract_direction(&self, edge: &EdgePattern) -> Result<EdgeDirection, PlannerError> {
        Ok(match edge.direction {
            Some(ref dir) => dir.clone(),
            None => EdgeDirection::Both,
        })
    }
}

#[derive(Debug)]
struct DummyStorage;

impl StorageClient for DummyStorage {
    fn insert_vertex(&mut self, _space: &str, _vertex: crate::core::Vertex) -> Result<Value, crate::core::StorageError> {
        Ok(Value::Int(0))
    }
    fn get_vertex(&self, _space: &str, _id: &Value) -> Result<Option<crate::core::Vertex>, crate::core::StorageError> {
        Ok(None)
    }
    fn update_vertex(&mut self, _space: &str, _vertex: crate::core::Vertex) -> Result<(), crate::core::StorageError> {
        Ok(())
    }
    fn delete_vertex(&mut self, _space: &str, _id: &Value) -> Result<(), crate::core::StorageError> {
        Ok(())
    }
    fn scan_vertices(&self, _space: &str) -> Result<Vec<crate::core::Vertex>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn scan_vertices_by_tag(&self, _space: &str, _tag: &str) -> Result<Vec<crate::core::Vertex>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn scan_vertices_by_prop(&self, _space: &str, _tag: &str, _prop: &str, _value: &Value) -> Result<Vec<crate::core::Vertex>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn insert_edge(&mut self, _space: &str, _edge: crate::core::Edge) -> Result<(), crate::core::StorageError> {
        Ok(())
    }
    fn get_edge(&self, _space: &str, _src: &Value, _dst: &Value, _edge_type: &str) -> Result<Option<crate::core::Edge>, crate::core::StorageError> {
        Ok(None)
    }
    fn get_node_edges(&self, _space: &str, _node_id: &Value, _direction: EdgeDirection) -> Result<Vec<crate::core::Edge>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn get_node_edges_filtered(&self, _space: &str, _node_id: &Value, _direction: EdgeDirection, _filter: Option<Box<dyn Fn(&crate::core::Edge) -> bool + Send + Sync>>) -> Result<Vec<crate::core::Edge>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn delete_edge(&mut self, _space: &str, _src: &Value, _dst: &Value, _edge_type: &str) -> Result<(), crate::core::StorageError> {
        Ok(())
    }
    fn scan_edges_by_type(&self, _space: &str, _edge_type: &str) -> Result<Vec<crate::core::Edge>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn scan_all_edges(&self, _space: &str) -> Result<Vec<crate::core::Edge>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn batch_insert_vertices(&mut self, _space: &str, _vertices: Vec<crate::core::Vertex>) -> Result<Vec<Value>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn batch_insert_edges(&mut self, _space: &str, _edges: Vec<crate::core::Edge>) -> Result<(), crate::core::StorageError> {
        Ok(())
    }
    fn begin_transaction(&mut self, _space: &str) -> Result<TransactionId, crate::core::StorageError> {
        Ok(TransactionId::new(0))
    }
    fn commit_transaction(&mut self, _space: &str, _tx_id: TransactionId) -> Result<(), crate::core::StorageError> {
        Ok(())
    }
    fn rollback_transaction(&mut self, _space: &str, _tx_id: TransactionId) -> Result<(), crate::core::StorageError> {
        Ok(())
    }
    fn create_space(&mut self, _space: &crate::core::types::SpaceInfo) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn drop_space(&mut self, _space: &str) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn get_space(&self, _space: &str) -> Result<Option<crate::core::types::SpaceInfo>, crate::core::StorageError> {
        Ok(None)
    }
    fn list_spaces(&self) -> Result<Vec<crate::core::types::SpaceInfo>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn create_tag(&mut self, _space: &str, _info: &crate::core::types::TagInfo) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn alter_tag(&mut self, _space: &str, _tag_name: &str, _additions: Vec<crate::core::types::PropertyDef>, _deletions: Vec<String>) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn get_tag(&self, _space: &str, _tag_name: &str) -> Result<Option<crate::core::types::TagInfo>, crate::core::StorageError> {
        Ok(None)
    }
    fn drop_tag(&mut self, _space: &str, _tag_name: &str) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn list_tags(&self, _space: &str) -> Result<Vec<crate::core::types::TagInfo>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn create_edge_type(&mut self, _space: &str, _info: &crate::core::types::EdgeTypeSchema) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn alter_edge_type(&mut self, _space: &str, _edge_type_name: &str, _additions: Vec<crate::core::types::PropertyDef>, _deletions: Vec<String>) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn get_edge_type(&self, _space: &str, _edge_type_name: &str) -> Result<Option<crate::core::types::EdgeTypeSchema>, crate::core::StorageError> {
        Ok(None)
    }
    fn drop_edge_type(&mut self, _space: &str, _edge_type_name: &str) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn list_edge_types(&self, _space: &str) -> Result<Vec<crate::core::types::EdgeTypeSchema>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn create_tag_index(&mut self, _space: &str, _info: &crate::index::Index) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn drop_tag_index(&mut self, _space: &str, _index_name: &str) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn get_tag_index(&self, _space: &str, _index_name: &str) -> Result<Option<crate::index::Index>, crate::core::StorageError> {
        Ok(None)
    }
    fn list_tag_indexes(&self, _space: &str) -> Result<Vec<crate::index::Index>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn rebuild_tag_index(&mut self, _space: &str, _index_name: &str) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn create_edge_index(&mut self, _space: &str, _info: &crate::index::Index) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn drop_edge_index(&mut self, _space: &str, _index_name: &str) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn get_edge_index(&self, _space: &str, _index_name: &str) -> Result<Option<crate::index::Index>, crate::core::StorageError> {
        Ok(None)
    }
    fn list_edge_indexes(&self, _space: &str) -> Result<Vec<crate::index::Index>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn rebuild_edge_index(&mut self, _space: &str, _index_name: &str) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn change_password(&mut self, _info: &crate::core::types::PasswordInfo) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }

    fn create_user(&mut self, _info: &crate::core::types::metadata::UserInfo) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }

    fn alter_user(&mut self, _info: &crate::core::types::metadata::UserAlterInfo) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }

    fn drop_user(&mut self, _username: &str) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }

    fn insert_vertex_data(&mut self, _space: &str, _info: &crate::core::types::InsertVertexInfo) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn insert_edge_data(&mut self, _space: &str, _info: &crate::core::types::InsertEdgeInfo) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn delete_vertex_data(&mut self, _space: &str, _vertex_id: &str) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn delete_edge_data(&mut self, _space: &str, _src: &str, _dst: &str, _rank: i64) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn update_data(&mut self, _space: &str, _info: &crate::core::types::UpdateInfo) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }

    fn get_vertex_with_schema(&self, _space: &str, _tag_name: &str, _id: &crate::core::Value) -> Result<Option<(crate::storage::Schema, Vec<u8>)>, crate::core::StorageError> {
        Ok(None)
    }

    fn get_edge_with_schema(&self, _space: &str, _edge_type_name: &str, _src: &crate::core::Value, _dst: &crate::core::Value) -> Result<Option<(crate::storage::Schema, Vec<u8>)>, crate::core::StorageError> {
        Ok(None)
    }

    fn scan_vertices_with_schema(&self, _space: &str, _tag_name: &str) -> Result<Vec<(crate::storage::Schema, Vec<u8>)>, crate::core::StorageError> {
        Ok(Vec::new())
    }

    fn scan_edges_with_schema(&self, _space: &str, _edge_type_name: &str) -> Result<Vec<(crate::storage::Schema, Vec<u8>)>, crate::core::StorageError> {
        Ok(Vec::new())
    }

    fn lookup_index(&self, _space: &str, _index: &str, _value: &crate::core::Value) -> Result<Vec<crate::core::Value>, crate::core::StorageError> {
        Ok(Vec::new())
    }

    fn load_from_disk(&mut self) -> Result<(), crate::core::StorageError> {
        Ok(())
    }

    fn save_to_disk(&self) -> Result<(), crate::core::StorageError> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct EdgePattern {
    pub types: Option<Vec<String>>,
    pub direction: Option<EdgeDirection>,
    pub properties: Vec<(String, Value)>,
}

#[derive(Debug)]
pub enum StartVidSource {
    VertexSeek(NodePattern),
    IndexScan(NodePattern),
    FullScan(NodePattern),
}

#[derive(Debug, Clone)]
pub struct EndCondition {
    pub pattern: NodePattern,
}

#[derive(Debug)]
pub struct BfsConfig {
    pub max_iterations: usize,
    pub max_path_length: usize,
    pub direction: EdgeDirection,
    pub edge_types: Vec<String>,
}

#[derive(Debug)]
pub struct ShortestPathPlan {
    pub start: StartVidSource,
    pub end: EndCondition,
    pub bfs_config: BfsConfig,
}

#[derive(Debug)]
pub struct ShortestPathResult {
    pub paths: Vec<ShortestPath>,
    pub nodes_visited: usize,
    pub edges_explored: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShortestPath {
    pub vertices: Vec<Value>,
    pub edges: Vec<Edge>,
}

impl ShortestPathPlanner {
    pub fn find_shortest_path<S: StorageClient>(
        &self,
        storage: &S,
        plan: &ShortestPathPlan,
    ) -> Result<ShortestPathResult, PlannerError> {
        let start_vids = self.resolve_start_vids(storage, &plan.start)?;
        let end_pattern = &plan.end.pattern;

        let mut all_paths = Vec::new();
        let mut total_nodes_visited = 0;
        let mut total_edges_explored = 0;

        for start_vid in start_vids {
            if let Some(end_vid) = self.resolve_end_vid(storage, end_pattern)? {
                match self.bfs_search(storage, &start_vid, &end_vid, &plan.bfs_config) {
                    Ok(Some(path)) => {
                        total_nodes_visited += path.vertices.len();
                        total_edges_explored += path.edges.len();
                        all_paths.push(path);
                    }
                    Ok(None) => continue,
                    Err(e) => return Err(e),
                }
            }
        }

        all_paths.sort_by(|a, b| a.vertices.len().cmp(&b.vertices.len()));

        Ok(ShortestPathResult {
            paths: all_paths,
            nodes_visited: total_nodes_visited,
            edges_explored: total_edges_explored,
        })
    }

    fn resolve_start_vids<S: StorageClient>(
        &self,
        storage: &S,
        start: &StartVidSource,
    ) -> Result<Vec<Value>, PlannerError> {
        match start {
            StartVidSource::VertexSeek(pattern) => {
                match &pattern.vid {
                    Some(vid) => {
                        let vid_clone: Value = vid.clone();
                        Ok(vec![vid_clone])
                    }
                    None => self.scan_matching_vertices(storage, pattern),
                }
            }
            StartVidSource::IndexScan(pattern) => self.scan_matching_vertices(storage, pattern),
            StartVidSource::FullScan(pattern) => self.scan_matching_vertices(storage, pattern),
        }
    }

    fn resolve_end_vid<S: StorageClient>(
        &self,
        storage: &S,
        pattern: &NodePattern,
    ) -> Result<Option<Value>, PlannerError> {
        match &pattern.vid {
            Some(vid) => {
                let vid_clone: Value = vid.clone();
                Ok(Some(vid_clone))
            }
            None => {
                let vertices = self.scan_matching_vertices(storage, pattern)?;
                Ok(vertices.first().cloned())
            }
        }
    }

    fn scan_matching_vertices<S: StorageClient>(
        &self,
        storage: &S,
        pattern: &NodePattern,
    ) -> Result<Vec<Value>, PlannerError> {
        let vertices = storage.scan_vertices("default")?;
        let mut matching: Vec<Value> = Vec::new();

        for vertex in vertices {
            if self.vertex_matches_pattern(&vertex, pattern) {
                matching.push(vertex.vid().clone());
            }
        }

        Ok(matching)
    }

    fn vertex_matches_pattern(&self, vertex: &Vertex, pattern: &NodePattern) -> bool {
        if !pattern.labels.is_empty() {
            let has_all_labels = pattern.labels.iter().all(|label| {
                vertex.tags.iter().any(|tag| tag.name == *label)
            });
            if !has_all_labels {
                return false;
            }
        }

        for (prop_name, prop_value) in &pattern.properties {
            let found = vertex.get_all_properties().iter().any(|(name, value)| {
                name == prop_name && **value == *prop_value
            });
            if !found {
                return false;
            }
        }

        true
    }

    fn bfs_search<S: StorageClient>(
        &self,
        storage: &S,
        start: &Value,
        end: &Value,
        config: &BfsConfig,
    ) -> Result<Option<ShortestPath>, StorageError> {
        let mut queue = VecDeque::new();
        let mut visited: HashSet<Value> = HashSet::new();
        let mut parent_map: HashMap<Value, (Value, Edge)> = HashMap::new();

        queue.push_back(start.clone());
        visited.insert(start.clone());

        while let Some(current) = queue.pop_front() {
            if current == *end {
                return Ok(self.reconstruct_path(start, end, &parent_map));
            }

            if parent_map.len() >= config.max_iterations {
                continue;
            }

            let edges = storage.get_node_edges("default", &current, config.direction.clone())?;

            for edge in edges {
                let neighbor = if *edge.src == current {
                    edge.dst.as_ref().clone()
                } else {
                    edge.src.as_ref().clone()
                };

                if !config.edge_types.is_empty()
                    && !config.edge_types.contains(&edge.edge_type)
                {
                    continue;
                }

                if visited.insert(neighbor.clone()) {
                    parent_map.insert(neighbor.clone(), (current.clone(), edge.clone()));
                    queue.push_back(neighbor);
                }
            }
        }

        Ok(None)
    }

    fn reconstruct_path(
        &self,
        start: &Value,
        end: &Value,
        parent_map: &HashMap<Value, (Value, Edge)>,
    ) -> Option<ShortestPath> {
        let mut vertices = Vec::new();
        let mut edges = Vec::new();
        let mut current = end.clone();

        let mut path = Vec::new();
        path.push(current.clone());

        while let Some((parent, edge)) = parent_map.get(&current) {
            vertices.push(current.clone());
            edges.push(edge.clone());
            current = parent.clone();
            path.push(current.clone());
        }

        vertices.push(start.clone());
        vertices.reverse();
        edges.reverse();

        Some(ShortestPath { vertices, edges })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortest_path_planner_new() {
        let _planner = ShortestPathPlanner::new();
        assert!(true);
    }

    #[test]
    fn test_edge_pattern() {
        let _pattern = EdgePattern {
            types: Some(vec!["follows".to_string()]),
            direction: Some(EdgeDirection::Out),
            properties: vec![],
        };
        assert!(true);
    }
}
