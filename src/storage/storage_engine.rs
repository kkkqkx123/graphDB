use crate::core::{Edge, StorageError, Value, Vertex, EdgeDirection};
use crate::core::types::{
    SpaceInfo, TagInfo, EdgeTypeSchema, IndexInfo,
    PropertyDef, InsertVertexInfo, InsertEdgeInfo, UpdateInfo,
    PasswordInfo,
};
use crate::expression::storage::{RowReaderWrapper, Schema};

/// Transaction identifier
pub type TransactionId = u64;

/// Storage engine trait defining the interface for graph storage
pub trait StorageEngine: Send + Sync {
    // ========== 节点操作 ==========
    fn insert_node(&mut self, vertex: Vertex) -> Result<Value, StorageError>;
    fn get_node(&self, id: &Value) -> Result<Option<Vertex>, StorageError>;
    fn update_node(&mut self, vertex: Vertex) -> Result<(), StorageError>;
    fn delete_node(&mut self, id: &Value) -> Result<(), StorageError>;

    fn scan_all_vertices(&self) -> Result<Vec<Vertex>, StorageError>;
    fn scan_vertices_by_tag(&self, tag: &str) -> Result<Vec<Vertex>, StorageError>;
    fn scan_vertices_by_prop(&self, tag: &str, prop: &str, value: &Value) -> Result<Vec<Vertex>, StorageError>;

    // ========== 边操作 ==========
    fn insert_edge(&mut self, edge: Edge) -> Result<(), StorageError>;
    fn get_edge(
        &self,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<Option<Edge>, StorageError>;
    fn get_node_edges(
        &self,
        node_id: &Value,
        direction: EdgeDirection,
    ) -> Result<Vec<Edge>, StorageError>;
    fn get_node_edges_filtered(
        &self,
        node_id: &Value,
        direction: EdgeDirection,
        filter: Option<Box<dyn Fn(&Edge) -> bool + Send + Sync>>,
    ) -> Result<Vec<Edge>, StorageError>;
    fn delete_edge(
        &mut self,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<(), StorageError>;
    fn scan_edges_by_type(&self, edge_type: &str) -> Result<Vec<Edge>, StorageError>;
    fn scan_all_edges(&self) -> Result<Vec<Edge>, StorageError>;

    // ========== 批量操作 ==========
    fn batch_insert_nodes(&mut self, vertices: Vec<Vertex>) -> Result<Vec<Value>, StorageError>;
    fn batch_insert_edges(&mut self, edges: Vec<Edge>) -> Result<(), StorageError>;

    // ========== 事务操作 ==========
    fn begin_transaction(&mut self) -> Result<TransactionId, StorageError>;
    fn commit_transaction(&mut self, tx_id: TransactionId) -> Result<(), StorageError>;
    fn rollback_transaction(&mut self, tx_id: TransactionId) -> Result<(), StorageError>;

    fn get_input(&self, input_var: &str) -> Result<Option<Vec<Value>>, StorageError>;

    // ========== 空间管理 ==========
    fn create_space(&mut self, space: &SpaceInfo) -> Result<bool, StorageError>;
    fn drop_space(&mut self, space_name: &str) -> Result<bool, StorageError>;
    fn get_space(&self, space_name: &str) -> Result<Option<SpaceInfo>, StorageError>;
    fn list_spaces(&self) -> Result<Vec<SpaceInfo>, StorageError>;

    // ========== 标签管理 ==========
    fn create_tag(&mut self, info: &TagInfo) -> Result<bool, StorageError>;
    fn alter_tag(&mut self, space_name: &str, tag_name: &str, additions: Vec<PropertyDef>, deletions: Vec<String>) -> Result<bool, StorageError>;
    fn get_tag(&self, space_name: &str, tag_name: &str) -> Result<Option<TagInfo>, StorageError>;
    fn drop_tag(&mut self, space_name: &str, tag_name: &str) -> Result<bool, StorageError>;
    fn list_tags(&self, space_name: &str) -> Result<Vec<TagInfo>, StorageError>;

    // ========== 边类型管理 ==========
    fn create_edge_type(&mut self, info: &EdgeTypeSchema) -> Result<bool, StorageError>;
    fn alter_edge_type(&mut self, space_name: &str, edge_type_name: &str, additions: Vec<PropertyDef>, deletions: Vec<String>) -> Result<bool, StorageError>;
    fn get_edge_type(&self, space_name: &str, edge_type_name: &str) -> Result<Option<EdgeTypeSchema>, StorageError>;
    fn drop_edge_type(&mut self, space_name: &str, edge_type_name: &str) -> Result<bool, StorageError>;
    fn list_edge_types(&self, space_name: &str) -> Result<Vec<EdgeTypeSchema>, StorageError>;

    // ========== 索引管理 ==========
    fn create_tag_index(&mut self, info: &IndexInfo) -> Result<bool, StorageError>;
    fn drop_tag_index(&mut self, space_name: &str, index_name: &str) -> Result<bool, StorageError>;
    fn get_tag_index(&self, space_name: &str, index_name: &str) -> Result<Option<IndexInfo>, StorageError>;
    fn list_tag_indexes(&self, space_name: &str) -> Result<Vec<IndexInfo>, StorageError>;
    fn rebuild_tag_index(&mut self, space_name: &str, index_name: &str) -> Result<bool, StorageError>;

    fn create_edge_index(&mut self, info: &IndexInfo) -> Result<bool, StorageError>;
    fn drop_edge_index(&mut self, space_name: &str, index_name: &str) -> Result<bool, StorageError>;
    fn get_edge_index(&self, space_name: &str, index_name: &str) -> Result<Option<IndexInfo>, StorageError>;
    fn list_edge_indexes(&self, space_name: &str) -> Result<Vec<IndexInfo>, StorageError>;
    fn rebuild_edge_index(&mut self, space_name: &str, index_name: &str) -> Result<bool, StorageError>;

    // ========== 数据变更 ==========
    fn insert_vertex_data(&mut self, info: &InsertVertexInfo) -> Result<bool, StorageError>;
    fn insert_edge_data(&mut self, info: &InsertEdgeInfo) -> Result<bool, StorageError>;
    fn delete_vertex_data(&mut self, space_name: &str, vertex_id: &str) -> Result<bool, StorageError>;
    fn delete_edge_data(&mut self, space_name: &str, src: &str, dst: &str, rank: i64) -> Result<bool, StorageError>;
    fn update_data(&mut self, info: &UpdateInfo) -> Result<bool, StorageError>;

    // ========== 用户管理 ==========
    fn change_password(&mut self, info: &PasswordInfo) -> Result<bool, StorageError>;

    // ========== 二进制数据接口（用于 expression::storage 集成） ==========
    fn get_vertex_with_schema(&self, space_name: &str, tag_name: &str, id: &Value) -> Result<Option<(Schema, Vec<u8>)>, StorageError>;
    fn get_edge_with_schema(&self, space_name: &str, edge_type_name: &str, src: &Value, dst: &Value) -> Result<Option<(Schema, Vec<u8>)>, StorageError>;
    fn scan_vertices_with_schema(&self, space_name: &str, tag_name: &str) -> Result<Vec<(Schema, Vec<u8>)>, StorageError>;
    fn scan_edges_with_schema(&self, space_name: &str, edge_type_name: &str) -> Result<Vec<(Schema, Vec<u8>)>, StorageError>;
}
