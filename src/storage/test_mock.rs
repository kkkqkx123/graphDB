use crate::core::error::StorageError;
#[cfg(test)]
use crate::core::types::{
    EdgeTypeInfo, EdgeTypeSchema, Index, InsertEdgeInfo, InsertVertexInfo, PasswordInfo,
    PropertyDef, SpaceInfo, TagInfo, UpdateInfo, UserAlterInfo, UserInfo, VertexId,
};
#[cfg(test)]
use crate::core::{Edge, EdgeDirection, RoleType, Value, Vertex};
#[cfg(test)]
use crate::storage::engine::PropertyGraph;
#[cfg(test)]
#[cfg(test)]
use crate::storage::{
    StorageAdmin, StorageAuthOps, StorageReader, StorageSchemaOps, StorageStats, StorageWriter,
};
#[cfg(test)]
use parking_lot::RwLock;
#[cfg(test)]
use std::sync::Arc;

macro_rules! mock_stub {
    (&self, $fn:ident($($arg:ident: $ty:ty),*) -> $ret:ty, $val:expr) => {
        fn $fn(&self, $($arg: $ty),*) -> $ret { $val }
    };
    (&mut self, $fn:ident($($arg:ident: $ty:ty),*) -> $ret:ty, $val:expr) => {
        fn $fn(&mut self, $($arg: $ty),*) -> $ret { $val }
    };
}

#[cfg(test)]
#[derive(Debug, Clone)]
pub struct MockStorage {
    graph: Arc<RwLock<PropertyGraph>>,
}

#[cfg(test)]
impl MockStorage {
    pub fn new() -> Result<Self, StorageError> {
        let graph = PropertyGraph::new();
        Ok(Self { graph: Arc::new(RwLock::new(graph)) })
    }

    pub fn get_graph(&self) -> &Arc<RwLock<PropertyGraph>> { &self.graph }
}

#[cfg(test)]
impl Default for MockStorage {
    fn default() -> Self { Self::new().expect("Failed to create MockStorage") }
}

#[cfg(test)]
impl StorageReader for MockStorage {
    mock_stub!(&self, get_vertex(_space: &str, _id: &VertexId) -> Result<Option<Vertex>, StorageError>, Ok(None));
    mock_stub!(&self, scan_vertices(_space: &str) -> Result<Vec<Vertex>, StorageError>, Ok(Vec::new()));
    mock_stub!(&self, scan_vertices_by_tag(_space: &str, _tag: &str) -> Result<Vec<Vertex>, StorageError>, Ok(Vec::new()));
    mock_stub!(&self, scan_vertices_by_prop(_space: &str, _tag: &str, _prop: &str, _value: &Value) -> Result<Vec<Vertex>, StorageError>, Ok(Vec::new()));
    mock_stub!(&self, get_edge(_space: &str, _src: &VertexId, _dst: &VertexId, _edge_type: &str, _rank: i64) -> Result<Option<Edge>, StorageError>, Ok(None));
    mock_stub!(&self, get_node_edges(_space: &str, _node_id: &VertexId, _direction: EdgeDirection) -> Result<Vec<Edge>, StorageError>, Ok(Vec::new()));
    mock_stub!(&self, scan_edges_by_type(_space: &str, _edge_type: &str) -> Result<Vec<Edge>, StorageError>, Ok(Vec::new()));
    mock_stub!(&self, scan_all_edges(_space: &str) -> Result<Vec<Edge>, StorageError>, Ok(Vec::new()));
    mock_stub!(&self, lookup_index(_space: &str, _index: &str, _value: &Value) -> Result<Vec<Value>, StorageError>, Ok(Vec::new()));
    mock_stub!(&self, lookup_index_with_score(_space: &str, _index: &str, _value: &Value) -> Result<Vec<(Value, f32)>, StorageError>, Ok(Vec::new()));
    mock_stub!(&self, get_vertex_with_schema(_space: &str, _tag: &str, _id: &Value) -> Result<Option<(TagInfo, Vec<u8>)>, StorageError>, Ok(None));
    mock_stub!(&self, get_edge_with_schema(_space: &str, _edge_type: &str, _src: &Value, _dst: &Value) -> Result<Option<(EdgeTypeInfo, Vec<u8>)>, StorageError>, Ok(None));
    mock_stub!(&self, scan_vertices_with_schema(_space: &str, _tag: &str) -> Result<Vec<(TagInfo, Vec<u8>)>, StorageError>, Ok(Vec::new()));
    mock_stub!(&self, scan_edges_with_schema(_space: &str, _edge_type: &str) -> Result<Vec<(EdgeTypeInfo, Vec<u8>)>, StorageError>, Ok(Vec::new()));
    mock_stub!(&self, get_space(_space: &str) -> Result<Option<SpaceInfo>, StorageError>, Ok(None));
    mock_stub!(&self, get_space_by_id(_space_id: u64) -> Result<Option<SpaceInfo>, StorageError>, Ok(None));
    mock_stub!(&self, list_spaces() -> Result<Vec<SpaceInfo>, StorageError>, Ok(Vec::new()));
    mock_stub!(&self, get_space_id(_space: &str) -> Result<u64, StorageError>, Ok(1));
    mock_stub!(&self, space_exists(_space: &str) -> bool, false);
    mock_stub!(&self, get_tag(_space: &str, _tag: &str) -> Result<Option<TagInfo>, StorageError>, Ok(None));
    mock_stub!(&self, list_tags(_space: &str) -> Result<Vec<TagInfo>, StorageError>, Ok(Vec::new()));
    mock_stub!(&self, get_edge_type(_space: &str, _edge_type: &str) -> Result<Option<EdgeTypeSchema>, StorageError>, Ok(None));
    mock_stub!(&self, list_edge_types(_space: &str) -> Result<Vec<EdgeTypeSchema>, StorageError>, Ok(Vec::new()));
    mock_stub!(&self, get_tag_index(_space: &str, _index: &str) -> Result<Option<Index>, StorageError>, Ok(None));
    mock_stub!(&self, list_tag_indexes(_space: &str) -> Result<Vec<Index>, StorageError>, Ok(Vec::new()));
    mock_stub!(&self, get_edge_index(_space: &str, _index: &str) -> Result<Option<Index>, StorageError>, Ok(None));
    mock_stub!(&self, list_edge_indexes(_space: &str) -> Result<Vec<Index>, StorageError>, Ok(Vec::new()));
}

#[cfg(test)]
impl StorageWriter for MockStorage {
    mock_stub!(&mut self, insert_vertex(_space: &str, _vertex: Vertex) -> Result<VertexId, StorageError>, Ok(VertexId::new()));
    mock_stub!(&mut self, update_vertex(_space: &str, _vertex: Vertex) -> Result<(), StorageError>, Ok(()));
    mock_stub!(&mut self, delete_vertex(_space: &str, _id: &VertexId) -> Result<(), StorageError>, Ok(()));
    mock_stub!(&mut self, delete_vertex_with_edges(_space: &str, _id: &VertexId) -> Result<(), StorageError>, Ok(()));
    mock_stub!(&mut self, batch_insert_vertices(_space: &str, _vertices: Vec<Vertex>) -> Result<Vec<VertexId>, StorageError>, Ok(Vec::new()));
    mock_stub!(&mut self, delete_tags(_space: &str, _vertex_id: &VertexId, _tag_names: &[String]) -> Result<usize, StorageError>, Ok(0));
    mock_stub!(&mut self, insert_edge(_space: &str, _edge: Edge) -> Result<(), StorageError>, Ok(()));
    mock_stub!(&mut self, delete_edge(_space: &str, _src: &VertexId, _dst: &VertexId, _edge_type: &str, _rank: i64) -> Result<(), StorageError>, Ok(()));
    mock_stub!(&mut self, batch_insert_edges(_space: &str, _edges: Vec<Edge>) -> Result<(), StorageError>, Ok(()));
    mock_stub!(&mut self, insert_vertex_data(_space: &str, _info: &InsertVertexInfo) -> Result<bool, StorageError>, Ok(true));
    mock_stub!(&mut self, insert_edge_data(_space: &str, _info: &InsertEdgeInfo) -> Result<bool, StorageError>, Ok(true));
    mock_stub!(&mut self, delete_vertex_data(_space: &str, _vertex_id: &str) -> Result<bool, StorageError>, Ok(true));
    mock_stub!(&mut self, delete_edge_data(_space: &str, _src: &str, _dst: &str, _rank: i64) -> Result<bool, StorageError>, Ok(true));
    mock_stub!(&mut self, update_data(_space: &str, _space_id: u64, _info: &UpdateInfo) -> Result<bool, StorageError>, Ok(true));
}

#[cfg(test)]
impl StorageSchemaOps for MockStorage {
    mock_stub!(&mut self, create_space(_space: &mut SpaceInfo) -> Result<bool, StorageError>, Ok(true));
    mock_stub!(&mut self, drop_space(_space: &str) -> Result<bool, StorageError>, Ok(true));
    mock_stub!(&mut self, clear_space(_space: &str) -> Result<bool, StorageError>, Ok(true));
    mock_stub!(&mut self, alter_space_comment(_space_id: u64, _comment: String) -> Result<bool, StorageError>, Ok(true));
    mock_stub!(&mut self, create_tag(_space: &str, _info: &TagInfo) -> Result<u32, StorageError>, Ok(1));
    mock_stub!(&mut self, alter_tag(_space: &str, _tag: &str, _additions: Vec<PropertyDef>, _deletions: Vec<String>) -> Result<bool, StorageError>, Ok(true));
    mock_stub!(&mut self, drop_tag(_space: &str, _tag: &str) -> Result<bool, StorageError>, Ok(true));
    mock_stub!(&mut self, create_edge_type(_space: &str, _info: &EdgeTypeSchema) -> Result<u32, StorageError>, Ok(1));
    mock_stub!(&mut self, alter_edge_type(_space: &str, _edge_type: &str, _additions: Vec<PropertyDef>, _deletions: Vec<String>) -> Result<bool, StorageError>, Ok(true));
    mock_stub!(&mut self, drop_edge_type(_space: &str, _edge_type: &str) -> Result<bool, StorageError>, Ok(true));
    mock_stub!(&mut self, create_tag_index(_space: &str, _info: &Index) -> Result<bool, StorageError>, Ok(true));
    mock_stub!(&mut self, drop_tag_index(_space: &str, _index: &str) -> Result<bool, StorageError>, Ok(true));
    mock_stub!(&mut self, rebuild_tag_index(_space: &str, _index: &str) -> Result<bool, StorageError>, Ok(true));
    mock_stub!(&mut self, create_edge_index(_space: &str, _info: &Index) -> Result<bool, StorageError>, Ok(true));
    mock_stub!(&mut self, drop_edge_index(_space: &str, _index: &str) -> Result<bool, StorageError>, Ok(true));
    mock_stub!(&mut self, rebuild_edge_index(_space: &str, _index: &str) -> Result<bool, StorageError>, Ok(true));
}

#[cfg(test)]
impl StorageAuthOps for MockStorage {
    mock_stub!(&mut self, change_password(_info: &PasswordInfo) -> Result<bool, StorageError>, Ok(true));
    mock_stub!(&mut self, create_user(_info: &UserInfo) -> Result<bool, StorageError>, Ok(true));
    mock_stub!(&mut self, alter_user(_info: &UserAlterInfo) -> Result<bool, StorageError>, Ok(true));
    mock_stub!(&mut self, drop_user(_username: &str) -> Result<bool, StorageError>, Ok(true));
    mock_stub!(&mut self, grant_role(_username: &str, _space_id: u64, _role: RoleType) -> Result<bool, StorageError>, Ok(true));
    mock_stub!(&mut self, revoke_role(_username: &str, _space_id: u64) -> Result<bool, StorageError>, Ok(true));
}

#[cfg(test)]
impl StorageAdmin for MockStorage {
    mock_stub!(&mut self, load_from_disk() -> Result<(), StorageError>, Ok(()));
    mock_stub!(&self, save_to_disk() -> Result<(), StorageError>, Ok(()));

    fn get_storage_stats(&self) -> StorageStats {
        StorageStats { total_vertices: 0, total_edges: 0, total_spaces: 0, total_tags: 0, total_edge_types: 0 }
    }

    mock_stub!(&self, find_dangling_edges(_space: &str) -> Result<Vec<Edge>, StorageError>, Ok(Vec::new()));
    mock_stub!(&mut self, repair_dangling_edges(_space: &str) -> Result<usize, StorageError>, Ok(0));
    mock_stub!(&self, get_db_path() -> &str, "");
}
