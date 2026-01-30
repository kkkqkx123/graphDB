use crate::core::{Edge, EdgeDirection, StorageError, Value, Vertex};
use crate::core::types::{
    EdgeTypeSchema, IndexInfo, InsertEdgeInfo, InsertVertexInfo, PasswordInfo,
    PropertyDef, SpaceInfo, TagInfo, UpdateInfo,
};
use crate::expression::storage::Schema;
use crate::storage::transaction::TransactionId;

pub trait StorageClient: Send + Sync {
    fn get_vertex(&self, space: &str, id: &Value) -> Result<Option<Vertex>, StorageError>;
    fn scan_vertices(&self, space: &str) -> Result<Vec<Vertex>, StorageError>;
    fn scan_vertices_by_tag(&self, space: &str, tag: &str) -> Result<Vec<Vertex>, StorageError>;
    fn scan_vertices_by_prop(
        &self,
        space: &str,
        tag: &str,
        prop: &str,
        value: &Value,
    ) -> Result<Vec<Vertex>, StorageError>;

    fn get_edge(
        &self,
        space: &str,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<Option<Edge>, StorageError>;
    fn get_node_edges(
        &self,
        space: &str,
        node_id: &Value,
        direction: EdgeDirection,
    ) -> Result<Vec<Edge>, StorageError>;
    fn get_node_edges_filtered(
        &self,
        space: &str,
        node_id: &Value,
        direction: EdgeDirection,
        filter: Option<Box<dyn Fn(&Edge) -> bool + Send + Sync + 'static>>,
    ) -> Result<Vec<Edge>, StorageError>;
    fn scan_edges_by_type(&self, space: &str, edge_type: &str) -> Result<Vec<Edge>, StorageError>;
    fn scan_all_edges(&self, space: &str) -> Result<Vec<Edge>, StorageError>;

    fn insert_vertex(&mut self, space: &str, vertex: Vertex) -> Result<Value, StorageError>;
    fn update_vertex(&mut self, space: &str, vertex: Vertex) -> Result<(), StorageError>;
    fn delete_vertex(&mut self, space: &str, id: &Value) -> Result<(), StorageError>;
    fn batch_insert_vertices(&mut self, space: &str, vertices: Vec<Vertex>) -> Result<Vec<Value>, StorageError>;

    fn insert_edge(&mut self, space: &str, edge: Edge) -> Result<(), StorageError>;
    fn delete_edge(&mut self, space: &str, src: &Value, dst: &Value, edge_type: &str) -> Result<(), StorageError>;
    fn batch_insert_edges(&mut self, space: &str, edges: Vec<Edge>) -> Result<(), StorageError>;

    fn begin_transaction(&mut self, space: &str) -> Result<TransactionId, StorageError>;
    fn commit_transaction(&mut self, space: &str, tx_id: TransactionId) -> Result<(), StorageError>;
    fn rollback_transaction(&mut self, space: &str, tx_id: TransactionId) -> Result<(), StorageError>;

    fn create_space(&mut self, space: &SpaceInfo) -> Result<bool, StorageError>;
    fn drop_space(&mut self, space: &str) -> Result<bool, StorageError>;
    fn get_space(&self, space: &str) -> Result<Option<SpaceInfo>, StorageError>;
    fn list_spaces(&self) -> Result<Vec<SpaceInfo>, StorageError>;

    fn create_tag(&mut self, space: &str, info: &TagInfo) -> Result<bool, StorageError>;
    fn alter_tag(&mut self, space: &str, tag: &str, additions: Vec<PropertyDef>, deletions: Vec<String>) -> Result<bool, StorageError>;
    fn get_tag(&self, space: &str, tag: &str) -> Result<Option<TagInfo>, StorageError>;
    fn drop_tag(&mut self, space: &str, tag: &str) -> Result<bool, StorageError>;
    fn list_tags(&self, space: &str) -> Result<Vec<TagInfo>, StorageError>;

    fn create_edge_type(&mut self, space: &str, info: &EdgeTypeSchema) -> Result<bool, StorageError>;
    fn alter_edge_type(&mut self, space: &str, edge_type: &str, additions: Vec<PropertyDef>, deletions: Vec<String>) -> Result<bool, StorageError>;
    fn get_edge_type(&self, space: &str, edge_type: &str) -> Result<Option<EdgeTypeSchema>, StorageError>;
    fn drop_edge_type(&mut self, space: &str, edge_type: &str) -> Result<bool, StorageError>;
    fn list_edge_types(&self, space: &str) -> Result<Vec<EdgeTypeSchema>, StorageError>;

    fn create_tag_index(&mut self, space: &str, info: &IndexInfo) -> Result<bool, StorageError>;
    fn drop_tag_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError>;
    fn get_tag_index(&self, space: &str, index: &str) -> Result<Option<IndexInfo>, StorageError>;
    fn list_tag_indexes(&self, space: &str) -> Result<Vec<IndexInfo>, StorageError>;
    fn rebuild_tag_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError>;

    fn create_edge_index(&mut self, space: &str, info: &IndexInfo) -> Result<bool, StorageError>;
    fn drop_edge_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError>;
    fn get_edge_index(&self, space: &str, index: &str) -> Result<Option<IndexInfo>, StorageError>;
    fn list_edge_indexes(&self, space: &str) -> Result<Vec<IndexInfo>, StorageError>;
    fn rebuild_edge_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError>;

    fn insert_vertex_data(&mut self, space: &str, info: &InsertVertexInfo) -> Result<bool, StorageError>;
    fn insert_edge_data(&mut self, space: &str, info: &InsertEdgeInfo) -> Result<bool, StorageError>;
    fn delete_vertex_data(&mut self, space: &str, vertex_id: &str) -> Result<bool, StorageError>;
    fn delete_edge_data(&mut self, space: &str, src: &str, dst: &str, rank: i64) -> Result<bool, StorageError>;
    fn update_data(&mut self, space: &str, info: &UpdateInfo) -> Result<bool, StorageError>;

    fn change_password(&mut self, info: &PasswordInfo) -> Result<bool, StorageError>;

    fn lookup_index(&self, space: &str, index: &str, value: &Value) -> Result<Vec<Value>, StorageError>;

    fn get_vertex_with_schema(&self, space: &str, tag: &str, id: &Value) -> Result<Option<(Schema, Vec<u8>)>, StorageError>;
    fn get_edge_with_schema(&self, space: &str, edge_type: &str, src: &Value, dst: &Value) -> Result<Option<(Schema, Vec<u8>)>, StorageError>;
    fn scan_vertices_with_schema(&self, space: &str, tag: &str) -> Result<Vec<(Schema, Vec<u8>)>, StorageError>;
    fn scan_edges_with_schema(&self, space: &str, edge_type: &str) -> Result<Vec<(Schema, Vec<u8>)>, StorageError>;
}

