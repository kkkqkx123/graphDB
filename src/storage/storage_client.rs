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

    fn get_vertex_with_schema(&self, space: &str, tag: &str, id: &Value) -> Result<Option<(Schema, Vec<u8>)>, StorageError>;
    fn get_edge_with_schema(&self, space: &str, edge_type: &str, src: &Value, dst: &Value) -> Result<Option<(Schema, Vec<u8>)>, StorageError>;
    fn scan_vertices_with_schema(&self, space: &str, tag: &str) -> Result<Vec<(Schema, Vec<u8>)>, StorageError>;
    fn scan_edges_with_schema(&self, space: &str, edge_type: &str) -> Result<Vec<(Schema, Vec<u8>)>, StorageError>;
}

impl<T: super::StorageEngine> StorageClient for T {
    fn get_vertex(&self, space: &str, id: &Value) -> Result<Option<Vertex>, StorageError> {
        self.get_node(id)
    }

    fn scan_vertices(&self, space: &str) -> Result<Vec<Vertex>, StorageError> {
        self.scan_all_vertices()
    }

    fn scan_vertices_by_tag(&self, space: &str, tag: &str) -> Result<Vec<Vertex>, StorageError> {
        self.scan_vertices_by_tag(tag)
    }

    fn scan_vertices_by_prop(
        &self,
        space: &str,
        tag: &str,
        prop: &str,
        value: &Value,
    ) -> Result<Vec<Vertex>, StorageError> {
        self.scan_vertices_by_prop(tag, prop, value)
    }

    fn get_edge(
        &self,
        space: &str,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<Option<Edge>, StorageError> {
        self.get_edge(src, dst, edge_type)
    }

    fn get_node_edges(
        &self,
        space: &str,
        node_id: &Value,
        direction: EdgeDirection,
    ) -> Result<Vec<Edge>, StorageError> {
        self.get_node_edges(node_id, direction)
    }

    fn get_node_edges_filtered(
        &self,
        space: &str,
        node_id: &Value,
        direction: EdgeDirection,
        filter: Option<Box<dyn Fn(&Edge) -> bool + Send + Sync + 'static>>,
    ) -> Result<Vec<Edge>, StorageError> {
        self.get_node_edges_filtered(node_id, direction, filter)
    }

    fn scan_edges_by_type(&self, space: &str, edge_type: &str) -> Result<Vec<Edge>, StorageError> {
        self.scan_edges_by_type(edge_type)
    }

    fn scan_all_edges(&self, space: &str) -> Result<Vec<Edge>, StorageError> {
        self.scan_all_edges()
    }

    fn insert_vertex(&mut self, space: &str, vertex: Vertex) -> Result<Value, StorageError> {
        self.insert_node(vertex)
    }

    fn update_vertex(&mut self, space: &str, vertex: Vertex) -> Result<(), StorageError> {
        self.update_node(vertex)
    }

    fn delete_vertex(&mut self, space: &str, id: &Value) -> Result<(), StorageError> {
        self.delete_node(id)
    }

    fn batch_insert_vertices(&mut self, space: &str, vertices: Vec<Vertex>) -> Result<Vec<Value>, StorageError> {
        self.batch_insert_nodes(vertices)
    }

    fn insert_edge(&mut self, space: &str, edge: Edge) -> Result<(), StorageError> {
        self.insert_edge(edge)
    }

    fn delete_edge(&mut self, space: &str, src: &Value, dst: &Value, edge_type: &str) -> Result<(), StorageError> {
        self.delete_edge(src, dst, edge_type)
    }

    fn batch_insert_edges(&mut self, space: &str, edges: Vec<Edge>) -> Result<(), StorageError> {
        self.batch_insert_edges(edges)
    }

    fn begin_transaction(&mut self, space: &str) -> Result<TransactionId, StorageError> {
        self.begin_transaction()
    }

    fn commit_transaction(&mut self, space: &str, tx_id: TransactionId) -> Result<(), StorageError> {
        self.commit_transaction(tx_id)
    }

    fn rollback_transaction(&mut self, space: &str, tx_id: TransactionId) -> Result<(), StorageError> {
        self.rollback_transaction(tx_id)
    }

    fn create_space(&mut self, space: &SpaceInfo) -> Result<bool, StorageError> {
        self.create_space(space)
    }

    fn drop_space(&mut self, space: &str) -> Result<bool, StorageError> {
        self.drop_space(space)
    }

    fn get_space(&self, space: &str) -> Result<Option<SpaceInfo>, StorageError> {
        self.get_space(space)
    }

    fn list_spaces(&self) -> Result<Vec<SpaceInfo>, StorageError> {
        self.list_spaces()
    }

    fn create_tag(&mut self, space: &str, info: &TagInfo) -> Result<bool, StorageError> {
        self.create_tag(info)
    }

    fn alter_tag(&mut self, space: &str, tag: &str, additions: Vec<PropertyDef>, deletions: Vec<String>) -> Result<bool, StorageError> {
        self.alter_tag(space, tag, additions, deletions)
    }

    fn get_tag(&self, space: &str, tag: &str) -> Result<Option<TagInfo>, StorageError> {
        self.get_tag(space, tag)
    }

    fn drop_tag(&mut self, space: &str, tag: &str) -> Result<bool, StorageError> {
        self.drop_tag(space, tag)
    }

    fn list_tags(&self, space: &str) -> Result<Vec<TagInfo>, StorageError> {
        self.list_tags(space)
    }

    fn create_edge_type(&mut self, space: &str, info: &EdgeTypeSchema) -> Result<bool, StorageError> {
        self.create_edge_type(info)
    }

    fn alter_edge_type(&mut self, space: &str, edge_type: &str, additions: Vec<PropertyDef>, deletions: Vec<String>) -> Result<bool, StorageError> {
        self.alter_edge_type(space, edge_type, additions, deletions)
    }

    fn get_edge_type(&self, space: &str, edge_type: &str) -> Result<Option<EdgeTypeSchema>, StorageError> {
        self.get_edge_type(space, edge_type)
    }

    fn drop_edge_type(&mut self, space: &str, edge_type: &str) -> Result<bool, StorageError> {
        self.drop_edge_type(space, edge_type)
    }

    fn list_edge_types(&self, space: &str) -> Result<Vec<EdgeTypeSchema>, StorageError> {
        self.list_edge_types(space)
    }

    fn create_tag_index(&mut self, space: &str, info: &IndexInfo) -> Result<bool, StorageError> {
        self.create_tag_index(info)
    }

    fn drop_tag_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        self.drop_tag_index(space, index)
    }

    fn get_tag_index(&self, space: &str, index: &str) -> Result<Option<IndexInfo>, StorageError> {
        self.get_tag_index(space, index)
    }

    fn list_tag_indexes(&self, space: &str) -> Result<Vec<IndexInfo>, StorageError> {
        self.list_tag_indexes(space)
    }

    fn rebuild_tag_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        self.rebuild_tag_index(space, index)
    }

    fn create_edge_index(&mut self, space: &str, info: &IndexInfo) -> Result<bool, StorageError> {
        self.create_edge_index(info)
    }

    fn drop_edge_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        self.drop_edge_index(space, index)
    }

    fn get_edge_index(&self, space: &str, index: &str) -> Result<Option<IndexInfo>, StorageError> {
        self.get_edge_index(space, index)
    }

    fn list_edge_indexes(&self, space: &str) -> Result<Vec<IndexInfo>, StorageError> {
        self.list_edge_indexes(space)
    }

    fn rebuild_edge_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        self.rebuild_edge_index(space, index)
    }

    fn insert_vertex_data(&mut self, space: &str, info: &InsertVertexInfo) -> Result<bool, StorageError> {
        self.insert_vertex_data(info)
    }

    fn insert_edge_data(&mut self, space: &str, info: &InsertEdgeInfo) -> Result<bool, StorageError> {
        self.insert_edge_data(info)
    }

    fn delete_vertex_data(&mut self, space: &str, vertex_id: &str) -> Result<bool, StorageError> {
        self.delete_vertex_data(space, vertex_id)
    }

    fn delete_edge_data(&mut self, space: &str, src: &str, dst: &str, rank: i64) -> Result<bool, StorageError> {
        self.delete_edge_data(space, src, dst, rank)
    }

    fn update_data(&mut self, space: &str, info: &UpdateInfo) -> Result<bool, StorageError> {
        self.update_data(info)
    }

    fn change_password(&mut self, info: &PasswordInfo) -> Result<bool, StorageError> {
        self.change_password(info)
    }

    fn get_vertex_with_schema(&self, space: &str, tag: &str, id: &Value) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        self.get_vertex_with_schema(space, tag, id)
    }

    fn get_edge_with_schema(&self, space: &str, edge_type: &str, src: &Value, dst: &Value) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        self.get_edge_with_schema(space, edge_type, src, dst)
    }

    fn scan_vertices_with_schema(&self, space: &str, tag: &str) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        self.scan_vertices_with_schema(space, tag)
    }

    fn scan_edges_with_schema(&self, space: &str, edge_type: &str) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        self.scan_edges_with_schema(space, edge_type)
    }
}

