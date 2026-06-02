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
pub(crate) mod type_utils;
mod user_ops;
mod writer;

#[cfg(test)]
mod test;

pub use context::GraphStorageContext;

use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::core::metadata::SchemaManager;
use crate::core::types::TransactionContextInfo;
use crate::core::types::{
    EdgeTypeInfo, Index, InsertEdgeInfo, InsertVertexInfo, PasswordInfo, PropertyDef, SpaceInfo,
    TagInfo, UpdateInfo, UserAlterInfo, UserInfo, VertexId,
};
use crate::core::{Edge, EdgeDirection, RoleType, StorageError, StorageResult, Value, Vertex};
use crate::storage::engine::{PersistenceConfig, PropertyGraph};
use crate::storage::index::{IndexGcConfig, IndexGcManager};
use crate::storage::{
    StorageAdmin, StorageAuthOps, StorageReader, StorageSchemaOps, StorageStats, StorageWriter,
};

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
        GraphStorageContext::new_with_path(path).map(|ctx| Self { ctx: Arc::new(ctx) })
    }

    pub fn new_with_persistence(path: PathBuf, config: PersistenceConfig) -> StorageResult<Self> {
        GraphStorageContext::new_with_persistence(path, config)
            .map(|ctx| Self { ctx: Arc::new(ctx) })
    }

    pub fn with_index_gc(mut self, config: IndexGcConfig) -> Self {
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

    pub(crate) fn index_gc_manager(&self) -> Option<Arc<IndexGcManager>> {
        self.ctx.index_gc_manager.clone()
    }

    pub(crate) fn get_db(&self) -> Arc<PropertyGraph> {
        self.ctx.graph.clone()
    }

    pub fn get_schema_manager(&self) -> Arc<SchemaManager> {
        self.ctx.schema_manager.clone()
    }

    pub fn get_transaction_context(&self) -> Option<Arc<TransactionContextInfo>> {
        self.ctx.get_transaction_context()
    }

    pub fn set_transaction_context(&self, context: Option<Arc<TransactionContextInfo>>) {
        self.ctx.set_transaction_context(context);
    }

    pub(crate) fn persistence(
        &self,
    ) -> Option<Arc<RwLock<crate::storage::engine::PersistenceCoordinator>>> {
        self.ctx.persistence.clone()
    }

    pub fn is_persistence_enabled(&self) -> bool {
        self.ctx.is_persistence_enabled()
    }
}

impl Default for GraphStorage {
    fn default() -> Self {
        Self::new().expect("Failed to create GraphStorage")
    }
}

impl StorageReader for GraphStorage {
    fn get_vertex(&self, space: &str, id: &VertexId) -> Result<Option<Vertex>, StorageError> {
        reader::get_vertex(&self.ctx, space, id)
    }

    fn scan_vertices(&self, space: &str) -> Result<Vec<Vertex>, StorageError> {
        reader::scan_vertices(&self.ctx, space)
    }

    fn scan_vertices_by_tag(&self, space: &str, tag: &str) -> Result<Vec<Vertex>, StorageError> {
        reader::scan_vertices_by_tag(&self.ctx, space, tag)
    }

    fn scan_vertices_by_prop(
        &self,
        space: &str,
        tag: &str,
        prop: &str,
        value: &Value,
    ) -> Result<Vec<Vertex>, StorageError> {
        reader::scan_vertices_by_prop(&self.ctx, space, tag, prop, value)
    }

    fn get_edge(
        &self,
        space: &str,
        src: &VertexId,
        dst: &VertexId,
        edge_type: &str,
        rank: i64,
    ) -> Result<Option<Edge>, StorageError> {
        reader::get_edge(&self.ctx, space, src, dst, edge_type, rank)
    }

    fn get_node_edges(
        &self,
        space: &str,
        node_id: &VertexId,
        direction: EdgeDirection,
    ) -> Result<Vec<Edge>, StorageError> {
        reader::get_node_edges(&self.ctx, space, node_id, direction)
    }

    fn scan_edges_by_type(&self, space: &str, edge_type: &str) -> Result<Vec<Edge>, StorageError> {
        reader::scan_edges_by_type(&self.ctx, space, edge_type)
    }

    fn scan_all_edges(&self, space: &str) -> Result<Vec<Edge>, StorageError> {
        reader::scan_all_edges(&self.ctx, space)
    }

    fn lookup_index(
        &self,
        space: &str,
        index_name: &str,
        value: &Value,
    ) -> Result<Vec<Value>, StorageError> {
        index_manager::lookup_index(&self.ctx, space, index_name, value)
    }

    fn lookup_index_with_score(
        &self,
        space: &str,
        index_name: &str,
        value: &Value,
    ) -> Result<Vec<(Value, f32)>, StorageError> {
        index_manager::lookup_index_with_score(&self.ctx, space, index_name, value)
    }

    fn get_vertex_with_schema(
        &self,
        space: &str,
        tag: &str,
        id: &Value,
    ) -> Result<Option<(TagInfo, Vec<u8>)>, StorageError> {
        reader::get_vertex_with_schema(&self.ctx, space, tag, id)
    }

    fn get_edge_with_schema(
        &self,
        space: &str,
        edge_type: &str,
        src: &Value,
        dst: &Value,
    ) -> Result<Option<(EdgeTypeInfo, Vec<u8>)>, StorageError> {
        reader::get_edge_with_schema(&self.ctx, space, edge_type, src, dst)
    }

    fn scan_vertices_with_schema(
        &self,
        space: &str,
        tag: &str,
    ) -> Result<Vec<(TagInfo, Vec<u8>)>, StorageError> {
        reader::scan_vertices_with_schema(&self.ctx, space, tag)
    }

    fn scan_edges_with_schema(
        &self,
        space: &str,
        edge_type: &str,
    ) -> Result<Vec<(EdgeTypeInfo, Vec<u8>)>, StorageError> {
        reader::scan_edges_with_schema(&self.ctx, space, edge_type)
    }

    fn get_space(&self, space: &str) -> Result<Option<SpaceInfo>, StorageError> {
        schema_adapter::get_space(&self.ctx, space)
    }

    fn get_space_by_id(&self, space_id: u64) -> Result<Option<SpaceInfo>, StorageError> {
        schema_adapter::get_space_by_id(&self.ctx, space_id)
    }

    fn list_spaces(&self) -> Result<Vec<SpaceInfo>, StorageError> {
        schema_adapter::list_spaces(&self.ctx)
    }

    fn get_space_id(&self, space: &str) -> Result<u64, StorageError> {
        schema_adapter::get_space_id(&self.ctx, space)
    }

    fn space_exists(&self, space: &str) -> bool {
        schema_adapter::space_exists(&self.ctx, space)
    }

    fn get_tag(&self, space: &str, tag: &str) -> Result<Option<TagInfo>, StorageError> {
        schema_adapter::get_tag(&self.ctx, space, tag)
    }

    fn list_tags(&self, space: &str) -> Result<Vec<TagInfo>, StorageError> {
        schema_adapter::list_tags(&self.ctx, space)
    }

    fn get_edge_type(
        &self,
        space: &str,
        edge_type: &str,
    ) -> Result<Option<EdgeTypeInfo>, StorageError> {
        schema_adapter::get_edge_type(&self.ctx, space, edge_type)
    }

    fn list_edge_types(&self, space: &str) -> Result<Vec<EdgeTypeInfo>, StorageError> {
        schema_adapter::list_edge_types(&self.ctx, space)
    }

    fn get_tag_index(&self, space: &str, index_name: &str) -> Result<Option<Index>, StorageError> {
        index_manager::get_tag_index(&self.ctx, space, index_name)
    }

    fn list_tag_indexes(&self, space: &str) -> Result<Vec<Index>, StorageError> {
        index_manager::list_tag_indexes(&self.ctx, space)
    }

    fn get_edge_index(&self, space: &str, index_name: &str) -> Result<Option<Index>, StorageError> {
        index_manager::get_edge_index(&self.ctx, space, index_name)
    }

    fn list_edge_indexes(&self, space: &str) -> Result<Vec<Index>, StorageError> {
        index_manager::list_edge_indexes(&self.ctx, space)
    }
}

impl StorageWriter for GraphStorage {
    fn insert_vertex(&mut self, space: &str, vertex: Vertex) -> Result<VertexId, StorageError> {
        writer::insert_vertex(&self.ctx, space, vertex)
    }

    fn update_vertex(&mut self, space: &str, vertex: Vertex) -> Result<(), StorageError> {
        writer::update_vertex(&self.ctx, space, vertex)
    }

    fn delete_vertex(&mut self, space: &str, id: &VertexId) -> Result<(), StorageError> {
        writer::delete_vertex(&self.ctx, space, id)
    }

    fn delete_vertex_with_edges(&mut self, space: &str, id: &VertexId) -> Result<(), StorageError> {
        writer::delete_vertex_with_edges(&self.ctx, space, id)
    }

    fn batch_insert_vertices(
        &mut self,
        space: &str,
        vertices: Vec<Vertex>,
    ) -> Result<Vec<VertexId>, StorageError> {
        writer::batch_insert_vertices(&self.ctx, space, vertices)
    }

    fn delete_tags(
        &mut self,
        space: &str,
        vertex_id: &VertexId,
        tag_names: &[String],
    ) -> Result<usize, StorageError> {
        writer::delete_tags(&self.ctx, space, vertex_id, tag_names)
    }

    fn insert_edge(&mut self, space: &str, edge: Edge) -> Result<(), StorageError> {
        writer::insert_edge(&self.ctx, space, edge)
    }

    fn delete_edge(
        &mut self,
        space: &str,
        src: &VertexId,
        dst: &VertexId,
        edge_type: &str,
        rank: i64,
    ) -> Result<(), StorageError> {
        writer::delete_edge(&self.ctx, space, src, dst, edge_type, rank)
    }

    fn batch_insert_edges(&mut self, space: &str, edges: Vec<Edge>) -> Result<(), StorageError> {
        writer::batch_insert_edges(&self.ctx, space, edges)
    }

    fn insert_vertex_data(
        &mut self,
        space: &str,
        info: &InsertVertexInfo,
    ) -> Result<bool, StorageError> {
        writer::insert_vertex_data(&self.ctx, space, info)
    }

    fn insert_edge_data(
        &mut self,
        space: &str,
        info: &InsertEdgeInfo,
    ) -> Result<bool, StorageError> {
        writer::insert_edge_data(&self.ctx, space, info)
    }

    fn delete_vertex_data(&mut self, space: &str, vertex_id: &str) -> Result<bool, StorageError> {
        writer::delete_vertex_data(&self.ctx, space, vertex_id)
    }

    fn delete_edge_data(
        &mut self,
        space: &str,
        src: &str,
        dst: &str,
        rank: i64,
    ) -> Result<bool, StorageError> {
        writer::delete_edge_data(&self.ctx, space, src, dst, rank)
    }

    fn update_data(
        &mut self,
        space: &str,
        space_id: u64,
        info: &UpdateInfo,
    ) -> Result<bool, StorageError> {
        writer::update_data(&self.ctx, space, space_id, info)
    }
}

impl StorageSchemaOps for GraphStorage {
    fn create_space(&mut self, space: &mut SpaceInfo) -> Result<bool, StorageError> {
        schema_adapter::create_space(&self.ctx, space)
    }

    fn drop_space(&mut self, space: &str) -> Result<bool, StorageError> {
        schema_adapter::drop_space(&self.ctx, space)
    }

    fn clear_space(&mut self, space: &str) -> Result<bool, StorageError> {
        schema_adapter::clear_space(&self.ctx, space)
    }

    fn alter_space_comment(
        &mut self,
        space_id: u64,
        comment: String,
    ) -> Result<bool, StorageError> {
        schema_adapter::alter_space_comment(&self.ctx, space_id, comment)
    }

    fn create_tag(&mut self, space: &str, tag: &TagInfo) -> Result<u32, StorageError> {
        schema_adapter::create_tag(&self.ctx, space, tag)
    }

    fn alter_tag(
        &mut self,
        space: &str,
        tag_name: &str,
        additions: Vec<PropertyDef>,
        deletions: Vec<String>,
    ) -> Result<bool, StorageError> {
        schema_adapter::alter_tag(&self.ctx, space, tag_name, additions, deletions)
    }

    fn drop_tag(&mut self, space: &str, tag: &str) -> Result<bool, StorageError> {
        schema_adapter::drop_tag(&self.ctx, space, tag)
    }

    fn create_edge_type(
        &mut self,
        space: &str,
        edge_type: &EdgeTypeInfo,
    ) -> Result<u32, StorageError> {
        schema_adapter::create_edge_type(&self.ctx, space, edge_type)
    }

    fn alter_edge_type(
        &mut self,
        space: &str,
        edge_type_name: &str,
        additions: Vec<PropertyDef>,
        deletions: Vec<String>,
    ) -> Result<bool, StorageError> {
        schema_adapter::alter_edge_type(&self.ctx, space, edge_type_name, additions, deletions)
    }

    fn drop_edge_type(&mut self, space: &str, edge_type: &str) -> Result<bool, StorageError> {
        schema_adapter::drop_edge_type(&self.ctx, space, edge_type)
    }

    fn create_tag_index(&mut self, space: &str, index: &Index) -> Result<bool, StorageError> {
        index_manager::create_tag_index(&self.ctx, space, index)
    }

    fn drop_tag_index(&mut self, space: &str, index_name: &str) -> Result<bool, StorageError> {
        index_manager::drop_tag_index(&self.ctx, space, index_name)
    }

    fn rebuild_tag_index(&mut self, space: &str, index_name: &str) -> Result<bool, StorageError> {
        let vertices = reader::scan_vertices(&self.ctx, space)?;
        index_manager::rebuild_tag_index(&self.ctx, space, index_name, &vertices)
    }

    fn create_edge_index(&mut self, space: &str, index: &Index) -> Result<bool, StorageError> {
        index_manager::create_edge_index(&self.ctx, space, index)
    }

    fn drop_edge_index(&mut self, space: &str, index_name: &str) -> Result<bool, StorageError> {
        index_manager::drop_edge_index(&self.ctx, space, index_name)
    }

    fn rebuild_edge_index(&mut self, space: &str, index_name: &str) -> Result<bool, StorageError> {
        let edges = reader::scan_all_edges(&self.ctx, space)?;
        index_manager::rebuild_edge_index(&self.ctx, space, index_name, &edges)
    }
}

impl StorageAuthOps for GraphStorage {
    fn change_password(&mut self, info: &PasswordInfo) -> Result<bool, StorageError> {
        user_ops::change_password(&self.ctx, info)
    }

    fn create_user(&mut self, info: &UserInfo) -> Result<bool, StorageError> {
        user_ops::create_user(&self.ctx, info)
    }

    fn alter_user(&mut self, info: &UserAlterInfo) -> Result<bool, StorageError> {
        user_ops::alter_user(&self.ctx, info)
    }

    fn drop_user(&mut self, username: &str) -> Result<bool, StorageError> {
        user_ops::drop_user(&self.ctx, username)
    }

    fn user_exists(&self, username: &str) -> bool {
        self.ctx.user_storage.user_exists(username)
    }

    fn grant_role(
        &mut self,
        username: &str,
        space_id: u64,
        role: RoleType,
    ) -> Result<bool, StorageError> {
        user_ops::grant_role(&self.ctx, username, space_id, role)
    }

    fn revoke_role(&mut self, username: &str, space_id: u64) -> Result<bool, StorageError> {
        user_ops::revoke_role(&self.ctx, username, space_id)
    }
}

impl StorageAdmin for GraphStorage {
    fn load_from_disk(&mut self) -> Result<(), StorageError> {
        persistence::load_from_disk(&self.ctx)
    }

    fn save_to_disk(&self) -> Result<(), StorageError> {
        persistence::save_to_disk(&self.ctx)
    }

    fn get_storage_stats(&self) -> StorageStats {
        maintenance::get_storage_stats(&self.ctx)
    }

    fn find_dangling_edges(&self, space: &str) -> Result<Vec<Edge>, StorageError> {
        maintenance::find_dangling_edges(&self.ctx, space)
    }

    fn repair_dangling_edges(&mut self, space: &str) -> Result<usize, StorageError> {
        maintenance::repair_dangling_edges(&self.ctx, space)
    }

    fn get_db_path(&self) -> &str {
        &self.ctx.db_path
    }

    fn flush(&self) -> StorageResult<()> {
        persistence::flush(&self.ctx)
    }

    fn save_data(&self) -> StorageResult<()> {
        persistence::save_data(&self.ctx)
    }

    fn save_data_to_dir(&self, dir: &std::path::Path) -> StorageResult<()> {
        persistence::save_data_to_dir(&self.ctx, dir)
    }

    fn create_checkpoint(&self) -> StorageResult<Option<crate::storage::CheckpointStats>> {
        persistence::create_checkpoint(&self.ctx)
    }

    fn compact(&self, compact_csr: bool, reserve_ratio: f32) -> StorageResult<()> {
        persistence::compact_transactional(&self.ctx, compact_csr, reserve_ratio)
    }

    fn auto_flush_if_needed(&self) -> StorageResult<bool> {
        persistence::auto_flush_if_needed(&self.ctx)
    }

    fn auto_checkpoint_if_needed(&self) -> StorageResult<Option<crate::storage::CheckpointStats>> {
        persistence::auto_checkpoint_if_needed(&self.ctx)
    }

    fn should_flush(&self) -> bool {
        persistence::should_flush(&self.ctx)
    }

    fn should_checkpoint(&self) -> bool {
        persistence::should_checkpoint(&self.ctx)
    }

    fn needs_recovery(&self) -> bool {
        persistence::needs_recovery(&self.ctx)
    }

    fn recover_from_wal(&self) -> StorageResult<crate::transaction::wal::recovery::RecoveryStats> {
        persistence::recover_from_wal(&self.ctx)
    }

    fn recover_from_wal_with_config(
        &self,
        config: crate::transaction::wal::recovery::RecoveryConfig,
    ) -> StorageResult<crate::transaction::wal::recovery::RecoveryStats> {
        persistence::recover_from_wal_with_config(&self.ctx, config)
    }

    fn init_with_recovery(
        &self,
    ) -> StorageResult<Option<crate::transaction::wal::recovery::RecoveryStats>> {
        if !self.needs_recovery() {
            return Ok(None);
        }

        log::info!("WAL recovery needed, starting recovery...");

        // Load schema and index metadata from the work directory (these are not checkpointed)
        if let Some(ref path) = self.ctx.work_dir {
            let schema_path = path.join("schema");
            if schema_path.exists() {
                self.ctx.schema_manager.load_schema(&schema_path)?;
            }
            let index_meta_path = path.join("index_meta");
            if index_meta_path.exists() {
                self.ctx
                    .index_metadata_manager
                    .load_indexes(&index_meta_path)?;
            }
        }

        schema_adapter::ensure_graph_types_from_schema(&self.ctx)?;

        // Try checkpoint recovery first
        let checkpoint_loaded = persistence::load_latest_checkpoint(&self.ctx)?;

        if checkpoint_loaded.is_none() {
            // No checkpoint available, load full state from disk
            persistence::load_from_disk(&self.ctx)?;
        }

        // Replay remaining WAL entries on top of restored state
        let stats = self.recover_from_wal()?;

        log::info!(
            "WAL recovery completed: {} entries replayed in {}ms",
            stats.wal_entries_replayed,
            stats.recovery_time_ms
        );
        Ok(Some(stats))
    }

    fn is_index_gc_running(&self) -> bool {
        self.ctx.is_index_gc_running()
    }

    fn start_index_gc(&self) -> Option<std::thread::JoinHandle<()>> {
        self.ctx.start_index_gc()
    }

    fn stop_index_gc(&self) {
        self.ctx.stop_index_gc();
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
}
