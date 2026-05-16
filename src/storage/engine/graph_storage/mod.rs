//! Storage Interface Implementation
//!
//! Implements the StorageClient trait for PropertyGraph storage.
//! This module acts as an adapter layer between the high-level StorageClient API
//! and the low-level PropertyGraph storage engine.

mod context;
mod index_manager;
mod maintenance;
mod persistence;
mod reader;
mod schema_adapter;
mod transaction_config;
mod transaction_support;
mod transactional_writer;
mod type_utils;
mod user_ops;
mod writer;

pub use context::GraphStorageContext;
pub use persistence::PersistenceOps;
pub use transaction_config::{DurabilityLevel, IsolationLevel, TransactionConfig, WalSyncMode};
pub use transaction_support::{execute_in_transaction, with_rollback};
pub use transactional_writer::TransactionalWriter;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::RwLock;

use crate::core::types::{
    EdgeTypeInfo, Index, InsertEdgeInfo, InsertVertexInfo, PasswordInfo, PropertyDef, SpaceInfo,
    TagInfo, UpdateInfo, UserAlterInfo, UserInfo, VertexId,
};
use crate::core::{Edge, EdgeDirection, RoleType, StorageError, StorageResult, Value, Vertex};
use crate::storage::engine::persistence_coordinator::{CheckpointInfo, CheckpointStats};
use crate::storage::engine::{PersistenceConfig, PropertyGraph};
use crate::storage::index::secondary::IndexGcManager;
use crate::storage::{
    StorageAdmin, StorageAuthOps, StorageReader, StorageSchemaOps, StorageStats,
    StorageWriter,
};
use crate::storage::metadata::{SchemaManager, Schema};
use crate::core::types::TransactionContextInfo;

#[derive(Clone)]
pub struct GraphStorage {
    ctx: Arc<GraphStorageContext>,
}

impl std::fmt::Debug for GraphStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphStorage")
            .field("work_dir", &self.ctx.work_dir)
            .field("db_path", &self.ctx.db_path)
            .finish()
    }
}

impl GraphStorage {
    pub fn new() -> StorageResult<Self> {
        Ok(Self {
            ctx: Arc::new(GraphStorageContext::new()),
        })
    }

    pub fn new_with_path(path: PathBuf) -> StorageResult<Self> {
        Ok(Self {
            ctx: Arc::new(GraphStorageContext::new_with_path(path)),
        })
    }

    pub fn new_with_persistence(path: PathBuf, config: PersistenceConfig) -> StorageResult<Self> {
        GraphStorageContext::new_with_persistence(path, config).map(|ctx| Self {
            ctx: Arc::new(ctx),
        })
    }

    pub fn with_index_gc(mut self, config: crate::storage::index::secondary::IndexGcConfig) -> Self {
        let new_ctx = Arc::new((*self.ctx).clone().with_index_gc(config));
        self.ctx = new_ctx;
        self
    }

    pub fn start_index_gc(&self) -> Option<std::thread::JoinHandle<()>> {
        self.ctx.start_index_gc()
    }

    pub fn stop_index_gc(&self) {
        self.ctx.stop_index_gc();
    }

    pub fn is_index_gc_running(&self) -> bool {
        self.ctx.is_index_gc_running()
    }

    pub fn index_gc_manager(&self) -> Option<Arc<IndexGcManager>> {
        self.ctx.index_gc_manager.clone()
    }

    pub fn with_fulltext_storage(mut self, fulltext: Arc<crate::storage::extend::FulltextStorage>) -> Self {
        let new_ctx = Arc::new((*self.ctx).clone().with_fulltext_storage(fulltext));
        self.ctx = new_ctx;
        self
    }

    pub fn fulltext_storage(&self) -> Option<Arc<crate::storage::extend::FulltextStorage>> {
        self.ctx.fulltext_storage.clone()
    }

    pub fn is_fulltext_enabled(&self) -> bool {
        self.ctx.is_fulltext_enabled()
    }

    pub fn get_db(&self) -> Arc<PropertyGraph> {
        self.ctx.graph.clone()
    }

    pub fn get_schema_manager(&self) -> Arc<SchemaManager> {
        self.ctx.schema_manager.clone()
    }

    pub fn get_extended_schema_manager(&self) -> Arc<crate::storage::metadata::ExtendedSchemaManager> {
        self.ctx.extended_schema_manager.clone()
    }

    pub fn get_transaction_context(&self) -> Option<Arc<TransactionContextInfo>> {
        self.ctx.get_transaction_context()
    }

    pub fn set_transaction_context(&self, context: Option<Arc<TransactionContextInfo>>) {
        self.ctx.set_transaction_context(context);
    }

    pub fn persistence(&self) -> Option<Arc<RwLock<crate::storage::engine::PersistenceCoordinator>>> {
        self.ctx.persistence.clone()
    }

    pub fn is_persistence_enabled(&self) -> bool {
        self.ctx.is_persistence_enabled()
    }

    pub fn save_data(&self) -> StorageResult<()> {
        PersistenceOps::new(&self.ctx).save_data()
    }

    pub fn save_data_to_dir(&self, dir: &Path) -> StorageResult<()> {
        PersistenceOps::new(&self.ctx).save_data_to_dir(dir)
    }

    pub fn flush(&self) -> StorageResult<()> {
        PersistenceOps::new(&self.ctx).flush()
    }

    pub fn create_checkpoint(&self) -> StorageResult<Option<CheckpointStats>> {
        PersistenceOps::new(&self.ctx).create_checkpoint()
    }

    pub fn load_latest_checkpoint(&self) -> StorageResult<Option<CheckpointInfo>> {
        PersistenceOps::new(&self.ctx).load_latest_checkpoint()
    }

    pub fn should_flush(&self) -> bool {
        PersistenceOps::new(&self.ctx).should_flush()
    }

    pub fn should_checkpoint(&self) -> bool {
        PersistenceOps::new(&self.ctx).should_checkpoint()
    }

    pub fn auto_flush_if_needed(&self) -> StorageResult<bool> {
        PersistenceOps::new(&self.ctx).auto_flush_if_needed()
    }

    pub fn auto_checkpoint_if_needed(&self) -> StorageResult<Option<CheckpointStats>> {
        PersistenceOps::new(&self.ctx).auto_checkpoint_if_needed()
    }

    pub fn compact_all(&self, ts: crate::storage::vertex::Timestamp) -> StorageResult<()> {
        PersistenceOps::new(&self.ctx).compact_all(ts)
    }

    /// Recover from WAL using RecoveryApplier trait
    ///
    /// This method performs crash recovery by replaying WAL entries.
    /// It uses the RecoveryApplier implementation on PropertyGraph.
    ///
    /// # Returns
    ///
    /// Recovery statistics including entries replayed and recovery time.
    pub fn recover_from_wal(&self) -> StorageResult<crate::transaction::wal::recovery::RecoveryStats> {
        PersistenceOps::new(&self.ctx).recover_from_wal()
    }

    /// Recover from WAL with custom configuration
    pub fn recover_from_wal_with_config(
        &self,
        config: crate::transaction::wal::recovery::RecoveryConfig,
    ) -> StorageResult<crate::transaction::wal::recovery::RecoveryStats> {
        PersistenceOps::new(&self.ctx).recover_from_wal_with_config(config)
    }

    /// Check if WAL recovery is needed
    pub fn needs_recovery(&self) -> bool {
        PersistenceOps::new(&self.ctx).needs_recovery()
    }

    /// Initialize with recovery
    ///
    /// This method should be called during startup to recover from
    /// any uncommitted transactions from a previous crash.
    pub fn init_with_recovery(&self) -> StorageResult<Option<crate::transaction::wal::recovery::RecoveryStats>> {
        if self.needs_recovery() {
            log::info!("WAL recovery needed, starting recovery...");
            let stats = self.recover_from_wal()?;
            log::info!(
                "WAL recovery completed: {} entries replayed in {}ms",
                stats.wal_entries_replayed,
                stats.recovery_time_ms
            );
            Ok(Some(stats))
        } else {
            Ok(None)
        }
    }
}

impl Default for GraphStorage {
    fn default() -> Self {
        Self::new().expect("Failed to create GraphStorage")
    }
}

impl StorageReader for GraphStorage {
    fn get_vertex(&self, space: &str, id: &VertexId) -> Result<Option<Vertex>, StorageError> {
        reader::GraphStorageReader::new(&self.ctx).get_vertex(space, id)
    }

    fn scan_vertices(&self, space: &str) -> Result<Vec<Vertex>, StorageError> {
        reader::GraphStorageReader::new(&self.ctx).scan_vertices(space)
    }

    fn scan_vertices_by_tag(&self, space: &str, tag: &str) -> Result<Vec<Vertex>, StorageError> {
        reader::GraphStorageReader::new(&self.ctx).scan_vertices_by_tag(space, tag)
    }

    fn scan_vertices_by_prop(
        &self,
        space: &str,
        tag: &str,
        prop: &str,
        value: &Value,
    ) -> Result<Vec<Vertex>, StorageError> {
        reader::GraphStorageReader::new(&self.ctx).scan_vertices_by_prop(space, tag, prop, value)
    }

    fn get_edge(
        &self,
        space: &str,
        src: &VertexId,
        dst: &VertexId,
        edge_type: &str,
        rank: i64,
    ) -> Result<Option<Edge>, StorageError> {
        reader::GraphStorageReader::new(&self.ctx).get_edge(space, src, dst, edge_type, rank)
    }

    fn get_node_edges(
        &self,
        space: &str,
        node_id: &VertexId,
        direction: EdgeDirection,
    ) -> Result<Vec<Edge>, StorageError> {
        reader::GraphStorageReader::new(&self.ctx).get_node_edges(space, node_id, direction)
    }

    fn scan_edges_by_type(&self, space: &str, edge_type: &str) -> Result<Vec<Edge>, StorageError> {
        reader::GraphStorageReader::new(&self.ctx).scan_edges_by_type(space, edge_type)
    }

    fn scan_all_edges(&self, space: &str) -> Result<Vec<Edge>, StorageError> {
        reader::GraphStorageReader::new(&self.ctx).scan_all_edges(space)
    }

    fn lookup_index(
        &self,
        space: &str,
        index_name: &str,
        value: &Value,
    ) -> Result<Vec<Value>, StorageError> {
        index_manager::IndexManagerOps::new(&self.ctx).lookup_index(space, index_name, value)
    }

    fn lookup_index_with_score(
        &self,
        space: &str,
        index_name: &str,
        value: &Value,
    ) -> Result<Vec<(Value, f32)>, StorageError> {
        index_manager::IndexManagerOps::new(&self.ctx).lookup_index_with_score(space, index_name, value)
    }

    fn get_vertex_with_schema(
        &self,
        space: &str,
        tag: &str,
        id: &Value,
    ) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        reader::GraphStorageReader::new(&self.ctx).get_vertex_with_schema(space, tag, id)
    }

    fn get_edge_with_schema(
        &self,
        space: &str,
        edge_type: &str,
        src: &Value,
        dst: &Value,
    ) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        reader::GraphStorageReader::new(&self.ctx).get_edge_with_schema(space, edge_type, src, dst)
    }

    fn scan_vertices_with_schema(
        &self,
        space: &str,
        tag: &str,
    ) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        reader::GraphStorageReader::new(&self.ctx).scan_vertices_with_schema(space, tag)
    }

    fn scan_edges_with_schema(
        &self,
        space: &str,
        edge_type: &str,
    ) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        reader::GraphStorageReader::new(&self.ctx).scan_edges_with_schema(space, edge_type)
    }

    fn get_space(&self, space: &str) -> Result<Option<SpaceInfo>, StorageError> {
        schema_adapter::SchemaAdapterOps::new(&self.ctx).get_space(space)
    }

    fn get_space_by_id(&self, space_id: u64) -> Result<Option<SpaceInfo>, StorageError> {
        schema_adapter::SchemaAdapterOps::new(&self.ctx).get_space_by_id(space_id)
    }

    fn list_spaces(&self) -> Result<Vec<SpaceInfo>, StorageError> {
        schema_adapter::SchemaAdapterOps::new(&self.ctx).list_spaces()
    }

    fn get_space_id(&self, space: &str) -> Result<u64, StorageError> {
        schema_adapter::SchemaAdapterOps::new(&self.ctx).get_space_id(space)
    }

    fn space_exists(&self, space: &str) -> bool {
        schema_adapter::SchemaAdapterOps::new(&self.ctx).space_exists(space)
    }

    fn get_tag(&self, space: &str, tag: &str) -> Result<Option<TagInfo>, StorageError> {
        schema_adapter::SchemaAdapterOps::new(&self.ctx).get_tag(space, tag)
    }

    fn list_tags(&self, space: &str) -> Result<Vec<TagInfo>, StorageError> {
        schema_adapter::SchemaAdapterOps::new(&self.ctx).list_tags(space)
    }

    fn get_edge_type(
        &self,
        space: &str,
        edge_type: &str,
    ) -> Result<Option<EdgeTypeInfo>, StorageError> {
        schema_adapter::SchemaAdapterOps::new(&self.ctx).get_edge_type(space, edge_type)
    }

    fn list_edge_types(&self, space: &str) -> Result<Vec<EdgeTypeInfo>, StorageError> {
        schema_adapter::SchemaAdapterOps::new(&self.ctx).list_edge_types(space)
    }

    fn get_tag_index(&self, space: &str, index_name: &str) -> Result<Option<Index>, StorageError> {
        index_manager::IndexManagerOps::new(&self.ctx).get_tag_index(space, index_name)
    }

    fn list_tag_indexes(&self, space: &str) -> Result<Vec<Index>, StorageError> {
        index_manager::IndexManagerOps::new(&self.ctx).list_tag_indexes(space)
    }

    fn get_edge_index(&self, space: &str, index_name: &str) -> Result<Option<Index>, StorageError> {
        index_manager::IndexManagerOps::new(&self.ctx).get_edge_index(space, index_name)
    }

    fn list_edge_indexes(&self, space: &str) -> Result<Vec<Index>, StorageError> {
        index_manager::IndexManagerOps::new(&self.ctx).list_edge_indexes(space)
    }
}

impl StorageWriter for GraphStorage {
    fn insert_vertex(&mut self, space: &str, vertex: Vertex) -> Result<VertexId, StorageError> {
        writer::GraphStorageWriter::new(&self.ctx).insert_vertex(space, vertex)
    }

    fn update_vertex(&mut self, space: &str, vertex: Vertex) -> Result<(), StorageError> {
        writer::GraphStorageWriter::new(&self.ctx).update_vertex(space, vertex)
    }

    fn delete_vertex(&mut self, space: &str, id: &VertexId) -> Result<(), StorageError> {
        writer::GraphStorageWriter::new(&self.ctx).delete_vertex(space, id)
    }

    fn delete_vertex_with_edges(&mut self, space: &str, id: &VertexId) -> Result<(), StorageError> {
        let reader = reader::GraphStorageReader::new(&self.ctx);
        writer::GraphStorageWriter::new(&self.ctx).delete_vertex_with_edges(space, id, &reader)
    }

    fn batch_insert_vertices(
        &mut self,
        space: &str,
        vertices: Vec<Vertex>,
    ) -> Result<Vec<VertexId>, StorageError> {
        writer::GraphStorageWriter::new(&self.ctx).batch_insert_vertices(space, vertices)
    }

    fn delete_tags(
        &mut self,
        space: &str,
        vertex_id: &VertexId,
        tag_names: &[String],
    ) -> Result<usize, StorageError> {
        writer::GraphStorageWriter::new(&self.ctx).delete_tags(space, vertex_id, tag_names)
    }

    fn insert_edge(&mut self, space: &str, edge: Edge) -> Result<(), StorageError> {
        writer::GraphStorageWriter::new(&self.ctx).insert_edge(space, edge)
    }

    fn delete_edge(
        &mut self,
        space: &str,
        src: &VertexId,
        dst: &VertexId,
        edge_type: &str,
        rank: i64,
    ) -> Result<(), StorageError> {
        writer::GraphStorageWriter::new(&self.ctx).delete_edge(space, src, dst, edge_type, rank)
    }

    fn batch_insert_edges(&mut self, space: &str, edges: Vec<Edge>) -> Result<(), StorageError> {
        writer::GraphStorageWriter::new(&self.ctx).batch_insert_edges(space, edges)
    }

    fn insert_vertex_data(
        &mut self,
        space: &str,
        info: &InsertVertexInfo,
    ) -> Result<bool, StorageError> {
        writer::GraphStorageWriter::new(&self.ctx).insert_vertex_data(space, info)
    }

    fn insert_edge_data(
        &mut self,
        space: &str,
        info: &InsertEdgeInfo,
    ) -> Result<bool, StorageError> {
        writer::GraphStorageWriter::new(&self.ctx).insert_edge_data(space, info)
    }

    fn delete_vertex_data(&mut self, space: &str, vertex_id: &str) -> Result<bool, StorageError> {
        writer::GraphStorageWriter::new(&self.ctx).delete_vertex_data(space, vertex_id)
    }

    fn delete_edge_data(
        &mut self,
        space: &str,
        src: &str,
        dst: &str,
        rank: i64,
    ) -> Result<bool, StorageError> {
        writer::GraphStorageWriter::new(&self.ctx).delete_edge_data(space, src, dst, rank)
    }

    fn update_data(
        &mut self,
        space: &str,
        space_id: u64,
        info: &UpdateInfo,
    ) -> Result<bool, StorageError> {
        writer::GraphStorageWriter::new(&self.ctx).update_data(space, space_id, info)
    }
}

impl StorageSchemaOps for GraphStorage {
    fn create_space(&mut self, space: &mut SpaceInfo) -> Result<bool, StorageError> {
        schema_adapter::SchemaAdapterOps::new(&self.ctx).create_space(space)
    }

    fn drop_space(&mut self, space: &str) -> Result<bool, StorageError> {
        schema_adapter::SchemaAdapterOps::new(&self.ctx).drop_space(space)
    }

    fn clear_space(&mut self, space: &str) -> Result<bool, StorageError> {
        schema_adapter::SchemaAdapterOps::new(&self.ctx).clear_space(space)
    }

    fn alter_space_comment(
        &mut self,
        space_id: u64,
        comment: String,
    ) -> Result<bool, StorageError> {
        schema_adapter::SchemaAdapterOps::new(&self.ctx).alter_space_comment(space_id, comment)
    }

    fn create_tag(&mut self, space: &str, tag: &TagInfo) -> Result<u32, StorageError> {
        schema_adapter::SchemaAdapterOps::new(&self.ctx).create_tag(space, tag)
    }

    fn alter_tag(
        &mut self,
        space: &str,
        tag_name: &str,
        additions: Vec<PropertyDef>,
        deletions: Vec<String>,
    ) -> Result<bool, StorageError> {
        schema_adapter::SchemaAdapterOps::new(&self.ctx).alter_tag(space, tag_name, additions, deletions)
    }

    fn drop_tag(&mut self, space: &str, tag: &str) -> Result<bool, StorageError> {
        schema_adapter::SchemaAdapterOps::new(&self.ctx).drop_tag(space, tag)
    }

    fn create_edge_type(
        &mut self,
        space: &str,
        edge_type: &EdgeTypeInfo,
    ) -> Result<u32, StorageError> {
        schema_adapter::SchemaAdapterOps::new(&self.ctx).create_edge_type(space, edge_type)
    }

    fn alter_edge_type(
        &mut self,
        space: &str,
        edge_type_name: &str,
        additions: Vec<PropertyDef>,
        deletions: Vec<String>,
    ) -> Result<bool, StorageError> {
        schema_adapter::SchemaAdapterOps::new(&self.ctx).alter_edge_type(space, edge_type_name, additions, deletions)
    }

    fn drop_edge_type(&mut self, space: &str, edge_type: &str) -> Result<bool, StorageError> {
        schema_adapter::SchemaAdapterOps::new(&self.ctx).drop_edge_type(space, edge_type)
    }

    fn create_tag_index(&mut self, space: &str, index: &Index) -> Result<bool, StorageError> {
        index_manager::IndexManagerOps::new(&self.ctx).create_tag_index(space, index)
    }

    fn drop_tag_index(&mut self, space: &str, index_name: &str) -> Result<bool, StorageError> {
        index_manager::IndexManagerOps::new(&self.ctx).drop_tag_index(space, index_name)
    }

    fn rebuild_tag_index(&mut self, space: &str, index_name: &str) -> Result<bool, StorageError> {
        let vertices = reader::GraphStorageReader::new(&self.ctx).scan_vertices(space)?;
        index_manager::IndexManagerOps::new(&self.ctx).rebuild_tag_index(space, index_name, &vertices)
    }

    fn create_edge_index(&mut self, space: &str, index: &Index) -> Result<bool, StorageError> {
        index_manager::IndexManagerOps::new(&self.ctx).create_edge_index(space, index)
    }

    fn drop_edge_index(&mut self, space: &str, index_name: &str) -> Result<bool, StorageError> {
        index_manager::IndexManagerOps::new(&self.ctx).drop_edge_index(space, index_name)
    }

    fn rebuild_edge_index(&mut self, space: &str, index_name: &str) -> Result<bool, StorageError> {
        let edges = reader::GraphStorageReader::new(&self.ctx).scan_all_edges(space)?;
        index_manager::IndexManagerOps::new(&self.ctx).rebuild_edge_index(space, index_name, &edges)
    }
}

impl StorageAuthOps for GraphStorage {
    fn change_password(&mut self, info: &PasswordInfo) -> Result<bool, StorageError> {
        user_ops::UserOps::new(&self.ctx).change_password(info)
    }

    fn create_user(&mut self, info: &UserInfo) -> Result<bool, StorageError> {
        user_ops::UserOps::new(&self.ctx).create_user(info)
    }

    fn alter_user(&mut self, info: &UserAlterInfo) -> Result<bool, StorageError> {
        user_ops::UserOps::new(&self.ctx).alter_user(info)
    }

    fn drop_user(&mut self, username: &str) -> Result<bool, StorageError> {
        user_ops::UserOps::new(&self.ctx).drop_user(username)
    }

    fn grant_role(
        &mut self,
        username: &str,
        space_id: u64,
        role: RoleType,
    ) -> Result<bool, StorageError> {
        user_ops::UserOps::new(&self.ctx).grant_role(username, space_id, role)
    }

    fn revoke_role(&mut self, username: &str, space_id: u64) -> Result<bool, StorageError> {
        user_ops::UserOps::new(&self.ctx).revoke_role(username, space_id)
    }
}

impl StorageAdmin for GraphStorage {
    fn load_from_disk(&mut self) -> Result<(), StorageError> {
        persistence::PersistenceOps::new(&self.ctx).load_from_disk()
    }

    fn save_to_disk(&self) -> Result<(), StorageError> {
        persistence::PersistenceOps::new(&self.ctx).save_to_disk()
    }

    fn get_storage_stats(&self) -> StorageStats {
        maintenance::MaintenanceOps::new(&self.ctx).get_storage_stats()
    }

    fn find_dangling_edges(&self, space: &str) -> Result<Vec<Edge>, StorageError> {
        maintenance::MaintenanceOps::new(&self.ctx).find_dangling_edges(space)
    }

    fn repair_dangling_edges(&mut self, space: &str) -> Result<usize, StorageError> {
        let writer = writer::GraphStorageWriter::new(&self.ctx);
        maintenance::MaintenanceOps::new(&self.ctx).repair_dangling_edges(space, &writer)
    }

    fn get_db_path(&self) -> &str {
        &self.ctx.db_path
    }

    fn get_transaction_context(&self) -> Option<Arc<TransactionContextInfo>> {
        self.ctx.get_transaction_context()
    }

    fn set_transaction_context(&self, context: Option<Arc<TransactionContextInfo>>) {
        self.ctx.set_transaction_context(context);
    }

    fn get_schema_manager(&self) -> Option<Arc<SchemaManager>> {
        Some(self.ctx.schema_manager.clone())
    }

    fn get_sync_manager(&self) -> Option<Arc<crate::sync::SyncManager>> {
        None
    }
}
