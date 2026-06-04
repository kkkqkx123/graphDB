//! Storage Layer Synchronous Wrapper
//!
//! Decorator pattern implementation that wraps any StorageClient to automatically
//! synchronize storage operations with external index systems (fulltext, vector).

use crate::core::metadata::SchemaManager;
use crate::core::types::{EdgeTypeInfo, TagInfo, VertexId};
use crate::core::{Edge, StorageError, Value, Vertex};
use crate::storage::{
    StorageAdmin, StorageAuthOps, StorageClient, StorageGcOps, StoragePersistenceOps,
    StorageReader, StorageRecoveryOps, StorageSchemaContextOps, StorageSchemaOps,
    StorageSyncContextOps, StorageTransactionContextOps, StorageWriter,
};
use crate::sync::coordinator::ChangeType;
use std::fmt::Debug;
use std::sync::Arc;

/// Decorator that wraps a StorageClient to provide automatic index synchronization.
#[derive(Clone, Debug)]
pub struct SyncWrapper<S: StorageClient + Debug> {
    inner: S,
    sync_manager: Option<Arc<crate::sync::SyncManager>>,
    enabled: bool,
}

impl<S: StorageClient> SyncWrapper<S> {
    /// Detect changed properties between two vertices.
    fn detect_changed_properties(
        tag_name: &str,
        old_vertex: &Vertex,
        new_vertex: &Vertex,
    ) -> Vec<(String, Value)> {
        let mut changed_props = Vec::new();

        let old_tag = old_vertex.tags.iter().find(|t| t.name == tag_name);
        let new_tag = new_vertex.tags.iter().find(|t| t.name == tag_name);

        match (old_tag, new_tag) {
            (Some(old_tag), Some(new_tag)) => {
                for (prop_name, new_value) in &new_tag.properties {
                    match old_tag.properties.get(prop_name) {
                        Some(old_value) if old_value != new_value => {
                            changed_props.push((prop_name.clone(), new_value.clone()));
                        }
                        None => {
                            changed_props.push((prop_name.clone(), new_value.clone()));
                        }
                        _ => {}
                    }
                }
            }
            (None, Some(new_tag)) => {
                for (prop_name, value) in &new_tag.properties {
                    changed_props.push((prop_name.clone(), value.clone()));
                }
            }
            _ => {}
        }

        changed_props
    }

    /// Get the current transaction ID from storage context.
    fn get_current_txn_id(&self) -> crate::core::types::TransactionId {
        if let Some(ctx) = self.inner.get_transaction_context() {
            return ctx.id;
        }
        0
    }

    /// Create a new wrapper without synchronization.
    pub fn new(storage: S) -> Self {
        Self {
            inner: storage,
            sync_manager: None,
            enabled: false,
        }
    }

    /// Create a new wrapper with a SyncManager for index synchronization.
    pub fn with_sync_manager(storage: S, sync_manager: Arc<crate::sync::SyncManager>) -> Self {
        Self {
            inner: storage,
            sync_manager: Some(sync_manager),
            enabled: true,
        }
    }

    /// Enable or disable synchronization.
    pub fn enable_sync(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if synchronization is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get reference to the sync manager.
    pub fn get_sync_manager(&self) -> Option<Arc<crate::sync::SyncManager>> {
        self.sync_manager.clone()
    }

    /// Get reference to the inner storage client.
    pub fn inner(&self) -> &S {
        &self.inner
    }

    /// Get mutable reference to the inner storage client.
    pub fn inner_mut(&mut self) -> &mut S {
        &mut self.inner
    }
}

macro_rules! forward_storage_methods {
    ($field:ident; $(fn $name:ident(&self $(, $arg:ident : $ty:ty)* $(,)?);)+) => {
        $(
            fn $name(&self, $($arg: $ty),*) {
                self.$field.$name($($arg),*)
            }
        )+
    };
    ($field:ident; $(fn $name:ident(&mut self $(, $arg:ident : $ty:ty)* $(,)?);)+) => {
        $(
            fn $name(&mut self, $($arg: $ty),*) {
                self.$field.$name($($arg),*)
            }
        )+
    };
    ($field:ident; $(fn $name:ident(&self $(, $arg:ident : $ty:ty)* $(,)?) -> $ret:ty;)+) => {
        $(
            fn $name(&self, $($arg: $ty),*) -> $ret {
                self.$field.$name($($arg),*)
            }
        )+
    };
    ($field:ident; $(fn $name:ident(&mut self $(, $arg:ident : $ty:ty)* $(,)?) -> $ret:ty;)+) => {
        $(
            fn $name(&mut self, $($arg: $ty),*) -> $ret {
                self.$field.$name($($arg),*)
            }
        )+
    };
}

impl<S: StorageClient + 'static> StorageReader for SyncWrapper<S> {
    forward_storage_methods!(inner;
        fn get_vertex(&self, space: &str, id: &VertexId) -> Result<Option<Vertex>, StorageError>;
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
            src: &VertexId,
            dst: &VertexId,
            edge_type: &str,
            rank: i64,
        ) -> Result<Option<Edge>, StorageError>;
        fn get_node_edges(
            &self,
            space: &str,
            node_id: &VertexId,
            direction: crate::core::EdgeDirection,
        ) -> Result<Vec<Edge>, StorageError>;
        fn scan_edges_by_type(&self, space: &str, edge_type: &str) -> Result<Vec<Edge>, StorageError>;
        fn scan_all_edges(&self, space: &str) -> Result<Vec<Edge>, StorageError>;
        fn lookup_index(
            &self,
            space: &str,
            index: &str,
            value: &Value,
        ) -> Result<Vec<Value>, StorageError>;
        fn get_vertex_with_schema(
            &self,
            space: &str,
            tag: &str,
            id: &Value,
        ) -> Result<Option<(TagInfo, Vec<u8>)>, StorageError>;
        fn get_edge_with_schema(
            &self,
            space: &str,
            edge_type: &str,
            src: &Value,
            dst: &Value,
        ) -> Result<Option<(EdgeTypeInfo, Vec<u8>)>, StorageError>;
        fn scan_vertices_with_schema(
            &self,
            space: &str,
            tag: &str,
        ) -> Result<Vec<(TagInfo, Vec<u8>)>, StorageError>;
        fn scan_edges_with_schema(
            &self,
            space: &str,
            edge_type: &str,
        ) -> Result<Vec<(EdgeTypeInfo, Vec<u8>)>, StorageError>;
        fn get_space(
            &self,
            space: &str,
        ) -> Result<Option<crate::core::types::SpaceInfo>, StorageError>;
        fn get_space_by_id(
            &self,
            space_id: u64,
        ) -> Result<Option<crate::core::types::SpaceInfo>, StorageError>;
        fn list_spaces(&self) -> Result<Vec<crate::core::types::SpaceInfo>, StorageError>;
        fn get_space_id(&self, space: &str) -> Result<u64, StorageError>;
        fn space_exists(&self, space: &str) -> bool;
        fn get_tag(
            &self,
            space: &str,
            tag: &str,
        ) -> Result<Option<crate::core::types::TagInfo>, StorageError>;
        fn list_tags(&self, space: &str) -> Result<Vec<crate::core::types::TagInfo>, StorageError>;
        fn get_edge_type(
            &self,
            space: &str,
            edge: &str,
        ) -> Result<Option<crate::core::types::EdgeTypeInfo>, StorageError>;
        fn list_edge_types(
            &self,
            space: &str,
        ) -> Result<Vec<crate::core::types::EdgeTypeInfo>, StorageError>;
        fn get_tag_index(
            &self,
            space: &str,
            index: &str,
        ) -> Result<Option<crate::core::types::Index>, StorageError>;
        fn list_tag_indexes(
            &self,
            space: &str,
        ) -> Result<Vec<crate::core::types::Index>, StorageError>;
        fn get_edge_index(
            &self,
            space: &str,
            index: &str,
        ) -> Result<Option<crate::core::types::Index>, StorageError>;
        fn list_edge_indexes(
            &self,
            space: &str,
        ) -> Result<Vec<crate::core::types::Index>, StorageError>;
    );
}

impl<S: StorageClient + 'static> StorageWriter for SyncWrapper<S> {
    fn insert_vertex(&mut self, space: &str, vertex: Vertex) -> Result<VertexId, StorageError> {
        let result = self.inner.insert_vertex(space, vertex.clone())?;

        if self.enabled {
            if let Some(sync_manager) = self.get_sync_manager() {
                let space_id = self.inner.get_space_id(space)?;
                let txn_id = self.get_current_txn_id();

                for tag in &vertex.tags {
                    let tag_name = &tag.name;
                    let props: Vec<(String, crate::core::Value)> = tag
                        .properties
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();

                    if !props.is_empty() {
                        let vid_value = Value::from(vertex.vid);
                        sync_manager
                            .on_vertex_change_with_txn(
                                txn_id,
                                space_id,
                                tag_name,
                                &vid_value,
                                &props,
                                ChangeType::Insert,
                            )
                            .map_err(|e| {
                                StorageError::db_error(format!(
                                    "Failed to sync vertex insert: {}",
                                    e
                                ))
                            })?;
                    }
                }
            }
        }

        Ok(result)
    }

    fn update_vertex(&mut self, space: &str, vertex: Vertex) -> Result<(), StorageError> {
        let old_vertex = self
            .inner
            .get_vertex(space, &vertex.vid)?
            .ok_or_else(|| StorageError::node_not_found(vertex.vid))?;

        self.inner.update_vertex(space, vertex.clone())?;

        if self.enabled {
            if let Some(sync_manager) = self.get_sync_manager() {
                let space_id = self.inner.get_space_id(space)?;
                let txn_id = self.get_current_txn_id();

                for tag in &vertex.tags {
                    let tag_name = &tag.name;
                    let changed_props =
                        Self::detect_changed_properties(tag_name, &old_vertex, &vertex);

                    if !changed_props.is_empty() {
                        let vid_value = Value::from(vertex.vid);
                        sync_manager
                            .on_vertex_change_with_txn(
                                txn_id,
                                space_id,
                                tag_name,
                                &vid_value,
                                &changed_props,
                                ChangeType::Update,
                            )
                            .map_err(|e| {
                                StorageError::db_error(format!(
                                    "Failed to sync vertex update: {}",
                                    e
                                ))
                            })?;
                    }
                }
            }
        }

        Ok(())
    }

    fn delete_vertex(&mut self, space: &str, id: &VertexId) -> Result<(), StorageError> {
        let vertex = self
            .inner
            .get_vertex(space, id)?
            .ok_or_else(|| StorageError::node_not_found(*id))?;

        self.inner.delete_vertex(space, id)?;

        if self.enabled {
            if let Some(sync_manager) = self.get_sync_manager() {
                let space_id = self.inner.get_space_id(space)?;
                let txn_id = self.get_current_txn_id();

                for tag in &vertex.tags {
                    let tag_name = &tag.name;
                    let id_value = Value::from(*id);
                    let props: Vec<(String, Value)> = tag
                        .properties
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();

                    if !props.is_empty() {
                        sync_manager
                            .on_vertex_change_with_txn(
                                txn_id,
                                space_id,
                                tag_name,
                                &id_value,
                                &props,
                                ChangeType::Delete,
                            )
                            .map_err(|e| {
                                StorageError::db_error(format!(
                                    "Failed to sync vertex delete: {}",
                                    e
                                ))
                            })?;
                    }
                }
            }
        }

        Ok(())
    }

    fn delete_vertex_with_edges(&mut self, space: &str, id: &VertexId) -> Result<(), StorageError> {
        let vertex = self
            .inner
            .get_vertex(space, id)?
            .ok_or_else(|| StorageError::node_not_found(*id))?;

        self.inner.delete_vertex_with_edges(space, id)?;

        if self.enabled {
            if let Some(sync_manager) = self.get_sync_manager() {
                let space_id = self.inner.get_space_id(space)?;
                let txn_id = self.get_current_txn_id();

                for tag in &vertex.tags {
                    let tag_name = &tag.name;
                    let id_value = Value::from(*id);
                    let props: Vec<(String, Value)> = tag
                        .properties
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();

                    if !props.is_empty() {
                        sync_manager
                            .on_vertex_change_with_txn(
                                txn_id,
                                space_id,
                                tag_name,
                                &id_value,
                                &props,
                                ChangeType::Delete,
                            )
                            .map_err(|e| {
                                StorageError::db_error(format!(
                                    "Failed to sync vertex delete: {}",
                                    e
                                ))
                            })?;
                    }
                }
            }
        }

        Ok(())
    }

    fn batch_insert_vertices(
        &mut self,
        space: &str,
        vertices: Vec<Vertex>,
    ) -> Result<Vec<VertexId>, StorageError> {
        let results = self.inner.batch_insert_vertices(space, vertices.clone())?;

        if self.enabled {
            if let Some(sync_manager) = self.get_sync_manager() {
                let space_id = self.inner.get_space_id(space)?;
                let txn_id = self.get_current_txn_id();

                for vertex in &vertices {
                    for tag in &vertex.tags {
                        let tag_name = &tag.name;
                        let props: Vec<(String, crate::core::Value)> = tag
                            .properties
                            .iter()
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect();

                        if !props.is_empty() {
                            let vid_value = Value::from(vertex.vid);
                            sync_manager
                                .on_vertex_change_with_txn(
                                    txn_id,
                                    space_id,
                                    tag_name,
                                    &vid_value,
                                    &props,
                                    ChangeType::Insert,
                                )
                                .map_err(|e| {
                                    StorageError::db_error(format!(
                                        "Failed to sync vertex insert: {}",
                                        e
                                    ))
                                })?;
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    fn delete_tags(
        &mut self,
        space: &str,
        vertex_id: &VertexId,
        tag_names: &[String],
    ) -> Result<usize, StorageError> {
        self.inner.delete_tags(space, vertex_id, tag_names)
    }

    fn insert_edge(&mut self, space: &str, edge: Edge) -> Result<(), StorageError> {
        let result = self.inner.insert_edge(space, edge.clone());

        if result.is_ok() && self.enabled {
            if let Some(sync_manager) = self.get_sync_manager() {
                if let Ok(space_id) = self.inner.get_space_id(space) {
                    let txn_id = self.get_current_txn_id();

                    sync_manager
                        .on_edge_insert(txn_id, space_id, &edge)
                        .map_err(|e| {
                            StorageError::db_error(format!("Failed to sync edge insert: {}", e))
                        })?;
                }
            }
        }

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
        let result = self.inner.delete_edge(space, src, dst, edge_type, rank);

        if result.is_ok() && self.enabled {
            if let Some(sync_manager) = self.get_sync_manager() {
                if let Ok(space_id) = self.inner.get_space_id(space) {
                    let txn_id = self.get_current_txn_id();
                    let src_value = Value::from(*src);
                    let dst_value = Value::from(*dst);

                    sync_manager
                        .on_edge_delete(txn_id, space_id, &src_value, &dst_value, edge_type)
                        .map_err(|e| {
                            StorageError::db_error(format!("Failed to sync edge delete: {}", e))
                        })?;
                }
            }
        }

        result
    }

    fn batch_insert_edges(&mut self, space: &str, edges: Vec<Edge>) -> Result<(), StorageError> {
        let result = self.inner.batch_insert_edges(space, edges.clone());

        if result.is_ok() && self.enabled {
            if let Some(sync_manager) = self.get_sync_manager() {
                if let Ok(space_id) = self.inner.get_space_id(space) {
                    let txn_id = self.get_current_txn_id();

                    for edge in &edges {
                        sync_manager
                            .on_edge_insert(txn_id, space_id, edge)
                            .map_err(|e| {
                                StorageError::db_error(format!("Failed to sync edge insert: {}", e))
                            })?;
                    }
                }
            }
        }

        result
    }

    fn insert_vertex_data(
        &mut self,
        space: &str,
        info: &crate::core::types::InsertVertexInfo,
    ) -> Result<bool, StorageError> {
        self.inner.insert_vertex_data(space, info)
    }

    fn delete_vertex_data(&mut self, space: &str, vertex_id: &str) -> Result<bool, StorageError> {
        self.inner.delete_vertex_data(space, vertex_id)
    }

    fn insert_edge_data(
        &mut self,
        space: &str,
        info: &crate::core::types::InsertEdgeInfo,
    ) -> Result<bool, StorageError> {
        self.inner.insert_edge_data(space, info)
    }

    fn delete_edge_data(
        &mut self,
        space: &str,
        src: &str,
        dst: &str,
        rank: i64,
    ) -> Result<bool, StorageError> {
        self.inner.delete_edge_data(space, src, dst, rank)
    }

    fn update_data(
        &mut self,
        space: &str,
        space_id: u64,
        info: &crate::core::types::UpdateInfo,
    ) -> Result<bool, StorageError> {
        self.inner.update_data(space, space_id, info)
    }
}

impl<S: StorageClient + 'static> StorageSchemaOps for SyncWrapper<S> {
    forward_storage_methods!(inner;
        fn create_space(&mut self, space: &mut crate::core::types::SpaceInfo) -> Result<bool, StorageError>;
        fn drop_space(&mut self, space: &str) -> Result<bool, StorageError>;
        fn clear_space(&mut self, space: &str) -> Result<bool, StorageError>;
        fn alter_space_comment(&mut self, space_id: u64, comment: String) -> Result<bool, StorageError>;
        fn create_tag(&mut self, space: &str, tag: &crate::core::types::TagInfo) -> Result<u32, StorageError>;
        fn alter_tag(
            &mut self,
            space: &str,
            tag: &str,
            additions: Vec<crate::core::types::PropertyDef>,
            deletions: Vec<String>,
        ) -> Result<bool, StorageError>;
        fn drop_tag(&mut self, space: &str, tag: &str) -> Result<bool, StorageError>;
        fn create_edge_type(
            &mut self,
            space: &str,
            edge: &crate::core::types::EdgeTypeInfo,
        ) -> Result<u32, StorageError>;
        fn alter_edge_type(
            &mut self,
            space: &str,
            edge_type: &str,
            additions: Vec<crate::core::types::PropertyDef>,
            deletions: Vec<String>,
        ) -> Result<bool, StorageError>;
        fn drop_edge_type(&mut self, space: &str, edge: &str) -> Result<bool, StorageError>;
        fn create_tag_index(
            &mut self,
            space: &str,
            info: &crate::core::types::Index,
        ) -> Result<bool, StorageError>;
        fn drop_tag_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError>;
        fn rebuild_tag_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError>;
        fn create_edge_index(
            &mut self,
            space: &str,
            info: &crate::core::types::Index,
        ) -> Result<bool, StorageError>;
        fn drop_edge_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError>;
        fn rebuild_edge_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError>;
    );
}

impl<S: StorageClient + 'static> StorageAuthOps for SyncWrapper<S> {
    forward_storage_methods!(inner;
        fn change_password(&mut self, info: &crate::core::types::PasswordInfo) -> Result<bool, StorageError>;
        fn create_user(&mut self, info: &crate::core::types::UserInfo) -> Result<bool, StorageError>;
        fn alter_user(&mut self, info: &crate::core::types::UserAlterInfo) -> Result<bool, StorageError>;
        fn drop_user(&mut self, username: &str) -> Result<bool, StorageError>;
        fn grant_role(
            &mut self,
            username: &str,
            space_id: u64,
            role: crate::core::RoleType,
        ) -> Result<bool, StorageError>;
        fn revoke_role(&mut self, username: &str, space_id: u64) -> Result<bool, StorageError>;
    );

    fn user_exists(&self, username: &str) -> bool {
        self.inner.user_exists(username)
    }
}

impl<S: StorageClient + 'static> StorageAdmin for SyncWrapper<S> {
    forward_storage_methods!(inner;
        fn load_from_disk(&mut self) -> Result<(), StorageError>;
        fn repair_dangling_edges(&mut self, space: &str) -> Result<usize, StorageError>;
    );

    forward_storage_methods!(inner;
        fn save_to_disk(&self) -> Result<(), StorageError>;
        fn get_storage_stats(&self) -> crate::storage::StorageStats;
        fn find_dangling_edges(&self, space: &str) -> Result<Vec<Edge>, StorageError>;
        fn get_db_path(&self) -> &str;
    );
}

impl<S: StorageClient + 'static> StoragePersistenceOps for SyncWrapper<S> {
    forward_storage_methods!(inner;
        fn flush(&self) -> Result<(), StorageError>;
        fn save_data(&self) -> crate::core::StorageResult<()>;
        fn save_data_to_dir(&self, dir: &std::path::Path) -> crate::core::StorageResult<()>;
        fn create_checkpoint(&self) -> crate::core::StorageResult<Option<crate::storage::CheckpointStats>>;
        fn verify_snapshot(&self, snapshot_id: u64) -> crate::core::StorageResult<bool>;
        fn cleanup_snapshots(&self) -> crate::core::StorageResult<usize>;
        fn snapshot_stats(&self) -> crate::storage::SnapshotStats;
        fn compact(&self, compact_csr: bool, reserve_ratio: f32) -> crate::core::StorageResult<()>;
        fn auto_flush_if_needed(&self) -> crate::core::StorageResult<bool>;
        fn auto_checkpoint_if_needed(&self) -> crate::core::StorageResult<Option<crate::storage::CheckpointStats>>;
        fn should_flush(&self) -> bool;
        fn should_checkpoint(&self) -> bool;
    );
}

impl<S: StorageClient + 'static> StorageSchemaContextOps for SyncWrapper<S> {
    forward_storage_methods!(inner;
        fn get_schema_manager(&self) -> Option<Arc<SchemaManager>>;
    );
}

impl<S: StorageClient + 'static> StorageTransactionContextOps for SyncWrapper<S> {
    forward_storage_methods!(inner;
        fn get_transaction_context(&self) -> Option<Arc<crate::core::types::TransactionContextInfo>>;
    );

    forward_storage_methods!(inner;
        fn set_transaction_context(&self, context: Option<Arc<crate::core::types::TransactionContextInfo>>);
    );
}

impl<S: StorageClient + 'static> StorageSyncContextOps for SyncWrapper<S> {
    fn get_sync_manager(&self) -> Option<Arc<crate::sync::SyncManager>> {
        self.sync_manager.clone()
    }
}

impl<S: StorageClient + 'static> StorageRecoveryOps for SyncWrapper<S> {
    forward_storage_methods!(inner;
        fn needs_recovery(&self) -> bool;
        fn recover_from_wal(&self) -> crate::core::StorageResult<crate::transaction::wal::recovery::RecoveryStats>;
        fn recover_from_wal_with_config(
            &self,
            config: crate::transaction::wal::recovery::RecoveryConfig,
        ) -> crate::core::StorageResult<crate::transaction::wal::recovery::RecoveryStats>;
        fn init_with_recovery(&self) -> crate::core::StorageResult<Option<crate::transaction::wal::recovery::RecoveryStats>>;
    );
}

impl<S: StorageClient + 'static> StorageGcOps for SyncWrapper<S> {
    forward_storage_methods!(inner;
        fn is_index_gc_running(&self) -> bool;
        fn start_index_gc(&self) -> Option<std::thread::JoinHandle<()>>;
    );

    forward_storage_methods!(inner;
        fn stop_index_gc(&self);
    );
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use crate::core::stats::{MetricType, StatsManager};
    use crate::core::types::VertexId;
    use crate::core::Edge;
    use crate::storage::{
        GraphStorage, MetricsStorage, MockStorage, StoragePersistenceOps, StorageReader,
        StorageWriter, SyncWrapper,
    };
    use crate::sync::batch::BatchConfig;
    use crate::sync::coordinator::SyncCoordinator;
    use crate::sync::SyncManager;
    use graphdb_sync::search::manager::FulltextIndexManager;
    use graphdb_sync::search::FulltextConfig;

    #[test]
    fn records_read_and_write_metrics() {
        let stats_manager = Arc::new(StatsManager::new());
        let inner = MockStorage::new().expect("Failed to create MockStorage");
        let mut storage = MetricsStorage::new(inner, stats_manager.clone());

        storage
            .get_vertex("test", &VertexId::from_int64(1))
            .expect("Failed to read vertex");
        storage
            .batch_insert_edges("test", Vec::new())
            .expect("Failed to write edges");

        assert_eq!(stats_manager.get_value(MetricType::StorageReadOps), Some(1));
        assert_eq!(
            stats_manager.get_value(MetricType::StorageWriteOps),
            Some(1)
        );
    }

    #[test]
    fn delegates_admin_checkpoint_operations() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let inner = GraphStorage::new_with_path(temp_dir.path().to_path_buf())
            .expect("Failed to create GraphStorage");
        let stats_manager = Arc::new(StatsManager::new());
        let storage = MetricsStorage::new(inner, stats_manager);

        let checkpoint = storage
            .create_checkpoint()
            .expect("checkpoint should succeed");

        assert!(checkpoint.is_some());
    }

    #[test]
    fn does_not_buffer_sync_events_when_edge_insert_fails() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let fulltext_config = FulltextConfig {
            index_path: temp_dir.path().join("fulltext"),
            ..Default::default()
        };
        let fulltext_manager = Arc::new(
            FulltextIndexManager::new(fulltext_config).expect("Failed to create fulltext manager"),
        );
        let sync_manager = Arc::new(SyncManager::new(Arc::new(SyncCoordinator::new(
            fulltext_manager,
            BatchConfig::default(),
        ))));

        let inner = MockStorage::new().expect("Failed to create MockStorage");
        inner.set_fail_insert_edge(true);

        let mut storage = SyncWrapper::with_sync_manager(inner, sync_manager.clone());
        let edge = Edge {
            src: VertexId::from_int64(1),
            dst: VertexId::from_int64(2),
            edge_type: "KNOWS".to_string(),
            ranking: 0,
            id: 0,
            props: HashMap::new(),
        };

        let result = storage.insert_edge("test", edge);

        assert!(result.is_err());
        assert_eq!(
            sync_manager.sync_coordinator().transaction_buffer_count(0),
            0
        );
    }
}
