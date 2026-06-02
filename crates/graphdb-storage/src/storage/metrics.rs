use std::sync::Arc;
use std::time::Instant;

use crate::core::metadata::SchemaManager;
use crate::core::stats::StatsManager;
use crate::core::types::{
    EdgeTypeInfo, Index, InsertEdgeInfo, InsertVertexInfo, PasswordInfo, PropertyDef, SpaceInfo,
    TagInfo, TransactionContextInfo, UpdateInfo, UserAlterInfo, UserInfo, VertexId,
};
use crate::core::{Edge, EdgeDirection, RoleType, StorageError, Value, Vertex};
use crate::storage::{
    StorageAdmin, StorageAuthOps, StorageClient, StorageReader, StorageSchemaOps, StorageStats,
    StorageWriter,
};
use crate::sync::SyncManager;

macro_rules! wrap_read {
    ($fn:ident($self:ident $(, $arg:ident: $ty:ty)*) -> $ret:ty) => {
        fn $fn(&$self, $($arg: $ty),*) -> $ret {
            let start = Instant::now();
            let result = $self.inner.$fn($($arg),*);
            $self.record_read(start.elapsed().as_micros() as u64, result.is_ok());
            result
        }
    };
}

macro_rules! wrap_write {
    ($fn:ident($self:ident $(, $arg:ident: $ty:ty)*) -> $ret:ty) => {
        fn $fn(&mut $self, $($arg: $ty),*) -> $ret {
            let start = Instant::now();
            let result = $self.inner.$fn($($arg),*);
            $self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
            result
        }
    };
}

macro_rules! wrap_read_mut {
    ($fn:ident($self:ident $(, $arg:ident: $ty:ty)*) -> $ret:ty) => {
        fn $fn(&mut $self, $($arg: $ty),*) -> $ret {
            let start = Instant::now();
            let result = $self.inner.$fn($($arg),*);
            $self.record_read(start.elapsed().as_micros() as u64, result.is_ok());
            result
        }
    };
}

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
    wrap_read!(get_vertex(self, space: &str, id: &VertexId) -> Result<Option<Vertex>, StorageError>);
    wrap_read!(scan_vertices(self, space: &str) -> Result<Vec<Vertex>, StorageError>);
    wrap_read!(scan_vertices_by_tag(self, space: &str, tag: &str) -> Result<Vec<Vertex>, StorageError>);
    wrap_read!(scan_vertices_by_prop(self, space: &str, tag: &str, prop: &str, value: &Value) -> Result<Vec<Vertex>, StorageError>);
    wrap_read!(get_edge(self, space: &str, src: &VertexId, dst: &VertexId, edge_type: &str, rank: i64) -> Result<Option<Edge>, StorageError>);
    wrap_read!(get_node_edges(self, space: &str, node_id: &VertexId, direction: EdgeDirection) -> Result<Vec<Edge>, StorageError>);
    wrap_read!(scan_edges_by_type(self, space: &str, edge_type: &str) -> Result<Vec<Edge>, StorageError>);
    wrap_read!(scan_all_edges(self, space: &str) -> Result<Vec<Edge>, StorageError>);
    wrap_read!(lookup_index(self, space: &str, index: &str, value: &Value) -> Result<Vec<Value>, StorageError>);
    wrap_read!(lookup_index_with_score(self, space: &str, index: &str, value: &Value) -> Result<Vec<(Value, f32)>, StorageError>);
    wrap_read!(get_vertex_with_schema(self, space: &str, tag: &str, id: &Value) -> Result<Option<(TagInfo, Vec<u8>)>, StorageError>);
    wrap_read!(get_edge_with_schema(self, space: &str, edge_type: &str, src: &Value, dst: &Value) -> Result<Option<(EdgeTypeInfo, Vec<u8>)>, StorageError>);
    wrap_read!(scan_vertices_with_schema(self, space: &str, tag: &str) -> Result<Vec<(TagInfo, Vec<u8>)>, StorageError>);
    wrap_read!(scan_edges_with_schema(self, space: &str, edge_type: &str) -> Result<Vec<(EdgeTypeInfo, Vec<u8>)>, StorageError>);
    wrap_read!(get_space(self, space: &str) -> Result<Option<SpaceInfo>, StorageError>);
    wrap_read!(get_space_by_id(self, space_id: u64) -> Result<Option<SpaceInfo>, StorageError>);
    wrap_read!(list_spaces(self) -> Result<Vec<SpaceInfo>, StorageError>);
    wrap_read!(get_space_id(self, space: &str) -> Result<u64, StorageError>);

    fn space_exists(&self, space: &str) -> bool {
        self.inner.space_exists(space)
    }

    wrap_read!(get_tag(self, space: &str, tag: &str) -> Result<Option<TagInfo>, StorageError>);
    wrap_read!(list_tags(self, space: &str) -> Result<Vec<TagInfo>, StorageError>);
    wrap_read!(get_edge_type(self, space: &str, edge_type: &str) -> Result<Option<EdgeTypeInfo>, StorageError>);
    wrap_read!(list_edge_types(self, space: &str) -> Result<Vec<EdgeTypeInfo>, StorageError>);
    wrap_read!(get_tag_index(self, space: &str, index: &str) -> Result<Option<Index>, StorageError>);
    wrap_read!(list_tag_indexes(self, space: &str) -> Result<Vec<Index>, StorageError>);
    wrap_read!(get_edge_index(self, space: &str, index: &str) -> Result<Option<Index>, StorageError>);
    wrap_read!(list_edge_indexes(self, space: &str) -> Result<Vec<Index>, StorageError>);
}

impl<S: StorageClient> StorageWriter for MetricsStorage<S> {
    wrap_write!(insert_vertex(self, space: &str, vertex: Vertex) -> Result<VertexId, StorageError>);
    wrap_write!(update_vertex(self, space: &str, vertex: Vertex) -> Result<(), StorageError>);
    wrap_write!(delete_vertex(self, space: &str, id: &VertexId) -> Result<(), StorageError>);
    wrap_write!(delete_vertex_with_edges(self, space: &str, id: &VertexId) -> Result<(), StorageError>);
    wrap_write!(batch_insert_vertices(self, space: &str, vertices: Vec<Vertex>) -> Result<Vec<VertexId>, StorageError>);
    wrap_write!(delete_tags(self, space: &str, vertex_id: &VertexId, tag_names: &[String]) -> Result<usize, StorageError>);
    wrap_write!(insert_edge(self, space: &str, edge: Edge) -> Result<(), StorageError>);
    wrap_write!(delete_edge(self, space: &str, src: &VertexId, dst: &VertexId, edge_type: &str, rank: i64) -> Result<(), StorageError>);
    wrap_write!(batch_insert_edges(self, space: &str, edges: Vec<Edge>) -> Result<(), StorageError>);
    wrap_write!(insert_vertex_data(self, space: &str, info: &InsertVertexInfo) -> Result<bool, StorageError>);
    wrap_write!(insert_edge_data(self, space: &str, info: &InsertEdgeInfo) -> Result<bool, StorageError>);
    wrap_write!(delete_vertex_data(self, space: &str, vertex_id: &str) -> Result<bool, StorageError>);
    wrap_write!(delete_edge_data(self, space: &str, src: &str, dst: &str, rank: i64) -> Result<bool, StorageError>);
    wrap_write!(update_data(self, space: &str, space_id: u64, info: &UpdateInfo) -> Result<bool, StorageError>);
}

impl<S: StorageClient> StorageSchemaOps for MetricsStorage<S> {
    wrap_write!(create_space(self, space: &mut SpaceInfo) -> Result<bool, StorageError>);
    wrap_write!(drop_space(self, space: &str) -> Result<bool, StorageError>);
    wrap_write!(clear_space(self, space: &str) -> Result<bool, StorageError>);
    wrap_write!(alter_space_comment(self, space_id: u64, comment: String) -> Result<bool, StorageError>);
    wrap_write!(create_tag(self, space: &str, tag: &TagInfo) -> Result<u32, StorageError>);
    wrap_write!(alter_tag(self, space: &str, tag: &str, additions: Vec<PropertyDef>, deletions: Vec<String>) -> Result<bool, StorageError>);
    wrap_write!(drop_tag(self, space: &str, tag: &str) -> Result<bool, StorageError>);
    wrap_write!(create_edge_type(self, space: &str, edge: &EdgeTypeInfo) -> Result<u32, StorageError>);
    wrap_write!(alter_edge_type(self, space: &str, edge_type: &str, additions: Vec<PropertyDef>, deletions: Vec<String>) -> Result<bool, StorageError>);
    wrap_write!(drop_edge_type(self, space: &str, edge_type: &str) -> Result<bool, StorageError>);
    wrap_write!(create_tag_index(self, space: &str, info: &Index) -> Result<bool, StorageError>);
    wrap_write!(drop_tag_index(self, space: &str, index: &str) -> Result<bool, StorageError>);
    wrap_write!(rebuild_tag_index(self, space: &str, index: &str) -> Result<bool, StorageError>);
    wrap_write!(create_edge_index(self, space: &str, info: &Index) -> Result<bool, StorageError>);
    wrap_write!(drop_edge_index(self, space: &str, index: &str) -> Result<bool, StorageError>);
    wrap_write!(rebuild_edge_index(self, space: &str, index: &str) -> Result<bool, StorageError>);
}

impl<S: StorageClient> StorageAuthOps for MetricsStorage<S> {
    wrap_write!(change_password(self, info: &PasswordInfo) -> Result<bool, StorageError>);
    wrap_write!(create_user(self, info: &UserInfo) -> Result<bool, StorageError>);
    wrap_write!(alter_user(self, info: &UserAlterInfo) -> Result<bool, StorageError>);
    wrap_write!(drop_user(self, username: &str) -> Result<bool, StorageError>);
    fn user_exists(&self, username: &str) -> bool {
        let start = Instant::now();
        let result = self.inner.user_exists(username);
        self.record_read(start.elapsed().as_micros() as u64, true);
        result
    }
    wrap_write!(grant_role(self, username: &str, space_id: u64, role: RoleType) -> Result<bool, StorageError>);
    wrap_write!(revoke_role(self, username: &str, space_id: u64) -> Result<bool, StorageError>);
}

impl<S: StorageClient> StorageAdmin for MetricsStorage<S> {
    wrap_read_mut!(load_from_disk(self) -> Result<(), StorageError>);

    fn save_to_disk(&self) -> Result<(), StorageError> {
        let start = Instant::now();
        let result = self.inner.save_to_disk();
        self.record_write(start.elapsed().as_micros() as u64, result.is_ok());
        result
    }

    fn get_storage_stats(&self) -> StorageStats {
        self.inner.get_storage_stats()
    }

    wrap_read!(find_dangling_edges(self, space: &str) -> Result<Vec<Edge>, StorageError>);
    wrap_write!(repair_dangling_edges(self, space: &str) -> Result<usize, StorageError>);

    fn get_db_path(&self) -> &str {
        self.inner.get_db_path()
    }
    fn get_sync_manager(&self) -> Option<Arc<SyncManager>> {
        self.inner.get_sync_manager()
    }
    fn get_schema_manager(&self) -> Option<Arc<SchemaManager>> {
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::core::stats::{MetricType, StatsManager};
    use crate::core::types::VertexId;
    use crate::storage::{MetricsStorage, MockStorage, StorageReader, StorageWriter};

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
}
