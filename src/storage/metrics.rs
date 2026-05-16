//! Metrics wrapper for Storage operations
//!
//! Provides a decorator pattern for recording storage metrics via StatsManager.
//! Similar to MetricsSearchEngine, this wraps storage operations to transparently
//! record read/write latency, error counts, and cache hit rates.

use std::sync::Arc;
use std::time::Instant;

use crate::core::stats::StatsManager;
use crate::core::types::{
    EdgeTypeInfo, Index, InsertEdgeInfo, InsertVertexInfo, PasswordInfo, PropertyDef, SpaceInfo,
    TagInfo, TransactionContextInfo, UpdateInfo, UserAlterInfo, UserInfo, VertexId,
};
use crate::core::{Edge, EdgeDirection, RoleType, StorageError, Value, Vertex};
use crate::storage::metadata::Schema;
use crate::storage::{StorageAdmin, StorageAuthOps, StorageClient, StorageReader, StorageSchemaOps, StorageStats, StorageWriter};

/// A decorator that wraps a StorageClient and records metrics via StatsManager.
///
/// This allows transparent instrumentation of storage read/write operations
/// without modifying the underlying storage implementations.
pub struct MetricsStorage<S: StorageClient> {
    inner: S,
    stats_manager: Arc<StatsManager>,
}

impl<S: StorageClient> MetricsStorage<S> {
    pub fn new(inner: S, stats_manager: Arc<StatsManager>) -> Self {
        Self {
            inner,
            stats_manager,
        }
    }

    pub fn into_inner(self) -> S {
        self.inner
    }

    pub fn stats_manager(&self) -> &Arc<StatsManager> {
        &self.stats_manager
    }

    fn record_read(&self, latency_us: u64, success: bool) {
        self.stats_manager.record_storage_read(latency_us, success);
    }

    fn record_write(&self, latency_us: u64, success: bool) {
        self.stats_manager.record_storage_write(latency_us, success);
    }
}

impl<S: StorageClient> StorageReader for MetricsStorage<S> {
    fn get_vertex(&self, space: &str, id: &VertexId) -> Result<Option<Vertex>, StorageError> {
        let start = Instant::now();
        let result = self.inner.get_vertex(space, id);
        self.record_read(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn scan_vertices(&self, space: &str) -> Result<Vec<Vertex>, StorageError> {
        let start = Instant::now();
        let result = self.inner.scan_vertices(space);
        self.record_read(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn scan_vertices_by_tag(&self, space: &str, tag: &str) -> Result<Vec<Vertex>, StorageError> {
        let start = Instant::now();
        let result = self.inner.scan_vertices_by_tag(space, tag);
        self.record_read(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn scan_vertices_by_prop(
        &self,
        space: &str,
        tag: &str,
        prop: &str,
        value: &Value,
    ) -> Result<Vec<Vertex>, StorageError> {
        let start = Instant::now();
        let result = self.inner.scan_vertices_by_prop(space, tag, prop, value);
        self.record_read(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn get_edge(
        &self,
        space: &str,
        src: &VertexId,
        dst: &VertexId,
        edge_type: &str,
        rank: i64,
    ) -> Result<Option<Edge>, StorageError> {
        let start = Instant::now();
        let result = self.inner.get_edge(space, src, dst, edge_type, rank);
        self.record_read(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn get_node_edges(
        &self,
        space: &str,
        node_id: &VertexId,
        direction: EdgeDirection,
    ) -> Result<Vec<Edge>, StorageError> {
        let start = Instant::now();
        let result = self.inner.get_node_edges(space, node_id, direction);
        self.record_read(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn scan_edges_by_type(&self, space: &str, edge_type: &str) -> Result<Vec<Edge>, StorageError> {
        let start = Instant::now();
        let result = self.inner.scan_edges_by_type(space, edge_type);
        self.record_read(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn scan_all_edges(&self, space: &str) -> Result<Vec<Edge>, StorageError> {
        let start = Instant::now();
        let result = self.inner.scan_all_edges(space);
        self.record_read(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn lookup_index(
        &self,
        space: &str,
        index: &str,
        value: &Value,
    ) -> Result<Vec<Value>, StorageError> {
        let start = Instant::now();
        let result = self.inner.lookup_index(space, index, value);
        self.record_read(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn lookup_index_with_score(
        &self,
        space: &str,
        index: &str,
        value: &Value,
    ) -> Result<Vec<(Value, f32)>, StorageError> {
        let start = Instant::now();
        let result = self.inner.lookup_index_with_score(space, index, value);
        self.record_read(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn get_vertex_with_schema(
        &self,
        space: &str,
        tag: &str,
        id: &Value,
    ) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        let start = Instant::now();
        let result = self.inner.get_vertex_with_schema(space, tag, id);
        self.record_read(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn get_edge_with_schema(
        &self,
        space: &str,
        edge_type: &str,
        src: &Value,
        dst: &Value,
    ) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        let start = Instant::now();
        let result = self.inner.get_edge_with_schema(space, edge_type, src, dst);
        self.record_read(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn scan_vertices_with_schema(
        &self,
        space: &str,
        tag: &str,
    ) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        let start = Instant::now();
        let result = self.inner.scan_vertices_with_schema(space, tag);
        self.record_read(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn scan_edges_with_schema(
        &self,
        space: &str,
        edge_type: &str,
    ) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        let start = Instant::now();
        let result = self.inner.scan_edges_with_schema(space, edge_type);
        self.record_read(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn get_space(&self, space: &str) -> Result<Option<SpaceInfo>, StorageError> {
        let start = Instant::now();
        let result = self.inner.get_space(space);
        self.record_read(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn get_space_by_id(&self, space_id: u64) -> Result<Option<SpaceInfo>, StorageError> {
        let start = Instant::now();
        let result = self.inner.get_space_by_id(space_id);
        self.record_read(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn list_spaces(&self) -> Result<Vec<SpaceInfo>, StorageError> {
        let start = Instant::now();
        let result = self.inner.list_spaces();
        self.record_read(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn get_space_id(&self, space: &str) -> Result<u64, StorageError> {
        let start = Instant::now();
        let result = self.inner.get_space_id(space);
        self.record_read(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn space_exists(&self, space: &str) -> bool {
        self.inner.space_exists(space)
    }

    fn get_tag(&self, space: &str, tag: &str) -> Result<Option<TagInfo>, StorageError> {
        let start = Instant::now();
        let result = self.inner.get_tag(space, tag);
        self.record_read(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn list_tags(&self, space: &str) -> Result<Vec<TagInfo>, StorageError> {
        let start = Instant::now();
        let result = self.inner.list_tags(space);
        self.record_read(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn get_edge_type(
        &self,
        space: &str,
        edge_type: &str,
    ) -> Result<Option<EdgeTypeInfo>, StorageError> {
        let start = Instant::now();
        let result = self.inner.get_edge_type(space, edge_type);
        self.record_read(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn list_edge_types(&self, space: &str) -> Result<Vec<EdgeTypeInfo>, StorageError> {
        let start = Instant::now();
        let result = self.inner.list_edge_types(space);
        self.record_read(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn get_tag_index(&self, space: &str, index: &str) -> Result<Option<Index>, StorageError> {
        let start = Instant::now();
        let result = self.inner.get_tag_index(space, index);
        self.record_read(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn list_tag_indexes(&self, space: &str) -> Result<Vec<Index>, StorageError> {
        let start = Instant::now();
        let result = self.inner.list_tag_indexes(space);
        self.record_read(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn get_edge_index(&self, space: &str, index: &str) -> Result<Option<Index>, StorageError> {
        let start = Instant::now();
        let result = self.inner.get_edge_index(space, index);
        self.record_read(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn list_edge_indexes(&self, space: &str) -> Result<Vec<Index>, StorageError> {
        let start = Instant::now();
        let result = self.inner.list_edge_indexes(space);
        self.record_read(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }
}

impl<S: StorageClient> StorageWriter for MetricsStorage<S> {
    fn insert_vertex(&mut self, space: &str, vertex: Vertex) -> Result<VertexId, StorageError> {
        let start = Instant::now();
        let result = self.inner.insert_vertex(space, vertex);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn update_vertex(&mut self, space: &str, vertex: Vertex) -> Result<(), StorageError> {
        let start = Instant::now();
        let result = self.inner.update_vertex(space, vertex);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn delete_vertex(&mut self, space: &str, id: &VertexId) -> Result<(), StorageError> {
        let start = Instant::now();
        let result = self.inner.delete_vertex(space, id);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn delete_vertex_with_edges(&mut self, space: &str, id: &VertexId) -> Result<(), StorageError> {
        let start = Instant::now();
        let result = self.inner.delete_vertex_with_edges(space, id);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn batch_insert_vertices(
        &mut self,
        space: &str,
        vertices: Vec<Vertex>,
    ) -> Result<Vec<VertexId>, StorageError> {
        let start = Instant::now();
        let result = self.inner.batch_insert_vertices(space, vertices);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn delete_tags(
        &mut self,
        space: &str,
        vertex_id: &VertexId,
        tag_names: &[String],
    ) -> Result<usize, StorageError> {
        let start = Instant::now();
        let result = self.inner.delete_tags(space, vertex_id, tag_names);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn insert_edge(&mut self, space: &str, edge: Edge) -> Result<(), StorageError> {
        let start = Instant::now();
        let result = self.inner.insert_edge(space, edge);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn delete_edge(
        &mut self,
        space: &str,
        src: &VertexId,
        dst: &VertexId,
        edge_type: &str,
        rank: i64,
    ) -> Result<(), StorageError> {
        let start = Instant::now();
        let result = self.inner.delete_edge(space, src, dst, edge_type, rank);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn batch_insert_edges(&mut self, space: &str, edges: Vec<Edge>) -> Result<(), StorageError> {
        let start = Instant::now();
        let result = self.inner.batch_insert_edges(space, edges);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn insert_vertex_data(
        &mut self,
        space: &str,
        info: &InsertVertexInfo,
    ) -> Result<bool, StorageError> {
        let start = Instant::now();
        let result = self.inner.insert_vertex_data(space, info);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn insert_edge_data(
        &mut self,
        space: &str,
        info: &InsertEdgeInfo,
    ) -> Result<bool, StorageError> {
        let start = Instant::now();
        let result = self.inner.insert_edge_data(space, info);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn delete_vertex_data(&mut self, space: &str, vertex_id: &str) -> Result<bool, StorageError> {
        let start = Instant::now();
        let result = self.inner.delete_vertex_data(space, vertex_id);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn delete_edge_data(
        &mut self,
        space: &str,
        src: &str,
        dst: &str,
        rank: i64,
    ) -> Result<bool, StorageError> {
        let start = Instant::now();
        let result = self.inner.delete_edge_data(space, src, dst, rank);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn update_data(
        &mut self,
        space: &str,
        space_id: u64,
        info: &UpdateInfo,
    ) -> Result<bool, StorageError> {
        let start = Instant::now();
        let result = self.inner.update_data(space, space_id, info);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }
}

impl<S: StorageClient> StorageSchemaOps for MetricsStorage<S> {
    fn create_space(&mut self, space: &mut SpaceInfo) -> Result<bool, StorageError> {
        let start = Instant::now();
        let result = self.inner.create_space(space);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn drop_space(&mut self, space: &str) -> Result<bool, StorageError> {
        let start = Instant::now();
        let result = self.inner.drop_space(space);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn clear_space(&mut self, space: &str) -> Result<bool, StorageError> {
        let start = Instant::now();
        let result = self.inner.clear_space(space);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn alter_space_comment(
        &mut self,
        space_id: u64,
        comment: String,
    ) -> Result<bool, StorageError> {
        let start = Instant::now();
        let result = self.inner.alter_space_comment(space_id, comment);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn create_tag(&mut self, space: &str, tag: &TagInfo) -> Result<u32, StorageError> {
        let start = Instant::now();
        let result = self.inner.create_tag(space, tag);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn alter_tag(
        &mut self,
        space: &str,
        tag: &str,
        additions: Vec<PropertyDef>,
        deletions: Vec<String>,
    ) -> Result<bool, StorageError> {
        let start = Instant::now();
        let result = self.inner.alter_tag(space, tag, additions, deletions);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn drop_tag(&mut self, space: &str, tag: &str) -> Result<bool, StorageError> {
        let start = Instant::now();
        let result = self.inner.drop_tag(space, tag);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn create_edge_type(&mut self, space: &str, edge: &EdgeTypeInfo) -> Result<u32, StorageError> {
        let start = Instant::now();
        let result = self.inner.create_edge_type(space, edge);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn alter_edge_type(
        &mut self,
        space: &str,
        edge_type: &str,
        additions: Vec<PropertyDef>,
        deletions: Vec<String>,
    ) -> Result<bool, StorageError> {
        let start = Instant::now();
        let result = self.inner.alter_edge_type(space, edge_type, additions, deletions);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn drop_edge_type(&mut self, space: &str, edge_type: &str) -> Result<bool, StorageError> {
        let start = Instant::now();
        let result = self.inner.drop_edge_type(space, edge_type);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn create_tag_index(&mut self, space: &str, info: &Index) -> Result<bool, StorageError> {
        let start = Instant::now();
        let result = self.inner.create_tag_index(space, info);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn drop_tag_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        let start = Instant::now();
        let result = self.inner.drop_tag_index(space, index);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn rebuild_tag_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        let start = Instant::now();
        let result = self.inner.rebuild_tag_index(space, index);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn create_edge_index(&mut self, space: &str, info: &Index) -> Result<bool, StorageError> {
        let start = Instant::now();
        let result = self.inner.create_edge_index(space, info);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn drop_edge_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        let start = Instant::now();
        let result = self.inner.drop_edge_index(space, index);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn rebuild_edge_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        let start = Instant::now();
        let result = self.inner.rebuild_edge_index(space, index);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }
}

impl<S: StorageClient> StorageAuthOps for MetricsStorage<S> {
    fn change_password(&mut self, info: &PasswordInfo) -> Result<bool, StorageError> {
        let start = Instant::now();
        let result = self.inner.change_password(info);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn create_user(&mut self, info: &UserInfo) -> Result<bool, StorageError> {
        let start = Instant::now();
        let result = self.inner.create_user(info);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn alter_user(&mut self, info: &UserAlterInfo) -> Result<bool, StorageError> {
        let start = Instant::now();
        let result = self.inner.alter_user(info);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn drop_user(&mut self, username: &str) -> Result<bool, StorageError> {
        let start = Instant::now();
        let result = self.inner.drop_user(username);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn grant_role(
        &mut self,
        username: &str,
        space_id: u64,
        role: RoleType,
    ) -> Result<bool, StorageError> {
        let start = Instant::now();
        let result = self.inner.grant_role(username, space_id, role);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn revoke_role(&mut self, username: &str, space_id: u64) -> Result<bool, StorageError> {
        let start = Instant::now();
        let result = self.inner.revoke_role(username, space_id);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }
}

impl<S: StorageClient> StorageAdmin for MetricsStorage<S> {
    fn load_from_disk(&mut self) -> Result<(), StorageError> {
        let start = Instant::now();
        let result = self.inner.load_from_disk();
        self.record_read(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn save_to_disk(&self) -> Result<(), StorageError> {
        let start = Instant::now();
        let result = self.inner.save_to_disk();
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn get_storage_stats(&self) -> StorageStats {
        self.inner.get_storage_stats()
    }

    fn find_dangling_edges(&self, space: &str) -> Result<Vec<Edge>, StorageError> {
        let start = Instant::now();
        let result = self.inner.find_dangling_edges(space);
        self.record_read(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn repair_dangling_edges(&mut self, space: &str) -> Result<usize, StorageError> {
        let start = Instant::now();
        let result = self.inner.repair_dangling_edges(space);
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn get_db_path(&self) -> &str {
        self.inner.get_db_path()
    }

    fn get_sync_manager(&self) -> Option<Arc<crate::sync::SyncManager>> {
        self.inner.get_sync_manager()
    }

    fn get_schema_manager(&self) -> Option<Arc<crate::storage::metadata::SchemaManager>> {
        self.inner.get_schema_manager()
    }

    fn get_transaction_context(&self) -> Option<Arc<TransactionContextInfo>> {
        self.inner.get_transaction_context()
    }

    fn set_transaction_context(&self, ctx: Option<Arc<TransactionContextInfo>>) {
        self.inner.set_transaction_context(ctx);
    }
}

impl<S: StorageClient> std::fmt::Debug for MetricsStorage<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetricsStorage")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<S: StorageClient> Clone for MetricsStorage<S>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            stats_manager: self.stats_manager.clone(),
        }
    }
}
