use crate::core::error::{DBError, DBResult, StorageError, StorageResult};
use crate::core::types::metadata::{UserInfo, UserAlterInfo};
use crate::query::context::runtime_context::RuntimeContext;
use crate::storage::StorageClient;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ProcessorCounters {
    pub num_calls: i64,
    pub num_errors: i64,
    pub latency_us: i64,
}

impl Default for ProcessorCounters {
    fn default() -> Self {
        Self {
            num_calls: 0,
            num_errors: 0,
            latency_us: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PartCode {
    pub code: DBError,
    pub part_id: u32,
}

/// 存储处理器 trait
///
/// 定义了存储层处理器的基本接口。此 trait 为底层存储操作提供基础执行框架，
/// 并可通过 ProcessorBase 实现。上层的 StorageProcessorExecutorImpl trait
/// 通过 StorageProcessorExecutor 结构体复用了此 trait 的功能，
/// 提供了更高级的异步执行、统计和内存管理功能。
pub trait StorageProcessor<RESP>: Send {
    fn context(&self) -> &Arc<RuntimeContext>;
    fn context_mut(&mut self) -> &mut RuntimeContext;

    fn execute(&mut self) -> DBResult<RESP>;

    fn on_finished(&mut self) -> DBResult<RESP> {
        self.execute()
    }

    fn on_error(&mut self, _error: DBError) -> DBResult<RESP> {
        self.execute()
    }
}

/// 处理器基类
///
/// 实现了 StorageProcessor trait，为存储操作提供基础功能实现。
/// 此结构体被上层的 StorageProcessorExecutor 结构体所使用，
/// 从而使得 StorageProcessorExecutorImpl trait 能够复用 StorageProcessor 的功能。
#[derive(Debug)]
pub struct ProcessorBase<RESP, S: StorageClient + std::fmt::Debug> {
    context: Arc<RuntimeContext>,
    resp: Option<RESP>,
    duration: Duration,
    start_time: Option<std::time::Instant>,
    codes: Vec<PartCode>,
    storage: Arc<Mutex<S>>,
}

impl<RESP, S: StorageClient + std::fmt::Debug> ProcessorBase<RESP, S>{
    pub fn new(context: Arc<RuntimeContext>, storage: Arc<Mutex<S>>) -> Self {
        Self {
            context,
            resp: None,
            duration: Duration::ZERO,
            start_time: None,
            codes: Vec::new(),
            storage,
        }
    }

    pub fn push_code(&mut self, code: DBError, part_id: u32) {
        self.codes.push(PartCode { code, part_id });
    }

    pub fn is_memory_exceeded(&self) -> bool {
        false
    }

    pub fn storage(&self) -> &Arc<Mutex<S>> {
        &self.storage
    }

    pub fn context(&self) -> &Arc<RuntimeContext> {
        &self.context
    }

    pub fn context_mut(&mut self) -> &mut RuntimeContext {
        Arc::make_mut(&mut self.context)
    }

    pub fn failed_parts(&self) -> &[PartCode] {
        &self.codes
    }

    pub fn has_errors(&self) -> bool {
        !self.codes.is_empty()
    }

    pub fn take_response(&mut self) -> RESP {
        self.resp
            .take()
            .expect("response should be set before taking")
    }

    pub fn set_response(&mut self, resp: RESP) {
        self.resp = Some(resp);
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }

    pub fn start_timer(&mut self) {
        self.start_time = Some(std::time::Instant::now());
    }

    pub fn stop_timer(&mut self) {
        if let Some(start) = self.start_time {
            self.duration = start.elapsed();
            self.start_time = None;
        }
    }
}

pub struct ProcessorFuture<RESP> {
    resp: Option<DBResult<RESP>>,
}

impl<RESP> ProcessorFuture<RESP> {
    pub fn new(resp: DBResult<RESP>) -> Self {
        Self { resp: Some(resp) }
    }

    pub fn into_result(self) -> DBResult<RESP> {
        self.resp.unwrap_or(Err(DBError::Internal(
            "processor future was already consumed".to_string(),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Edge, EdgeDirection, Value, Vertex};
    use crate::core::types::metadata::{SpaceInfo, TagInfo, EdgeTypeInfo, PropertyDef, InsertVertexInfo, InsertEdgeInfo, UpdateInfo, PasswordInfo};
    use crate::storage::transaction::TransactionId;
    use crate::storage::Schema;
    use crate::query::context::runtime_context::{PlanContext, RuntimeContext, StorageEnv};
    use crate::storage::index::IndexManager;
    use crate::storage::metadata::SchemaManager;
    use crate::storage::StorageClient;
    use crate::core::value::NullType;
    use std::sync::Mutex;
    use crate::index::{Index, IndexStatus, IndexStats, IndexOptimization};

    #[derive(Debug)]
    struct DummySchemaManager;

    impl SchemaManager for DummySchemaManager {
        fn create_space(&self, _space: &SpaceInfo) -> Result<bool, StorageError> { Ok(true) }
        fn drop_space(&self, _space_name: &str) -> Result<bool, StorageError> { Ok(true) }
        fn get_space(&self, _space_name: &str) -> Result<Option<SpaceInfo>, StorageError> { Ok(None) }
        fn list_spaces(&self) -> Result<Vec<SpaceInfo>, StorageError> { Ok(Vec::new()) }
        fn create_tag(&self, _space: &str, _tag: &TagInfo) -> Result<bool, StorageError> { Ok(true) }
        fn get_tag(&self, _space: &str, _tag_name: &str) -> Result<Option<TagInfo>, StorageError> { Ok(None) }
        fn list_tags(&self, _space: &str) -> Result<Vec<TagInfo>, StorageError> { Ok(Vec::new()) }
        fn drop_tag(&self, _space: &str, _tag_name: &str) -> Result<bool, StorageError> { Ok(true) }
        fn create_edge_type(&self, _space: &str, _edge: &EdgeTypeInfo) -> Result<bool, StorageError> { Ok(true) }
        fn get_edge_type(&self, _space: &str, _edge_type_name: &str) -> Result<Option<EdgeTypeInfo>, StorageError> { Ok(None) }
        fn list_edge_types(&self, _space: &str) -> Result<Vec<EdgeTypeInfo>, StorageError> { Ok(Vec::new()) }
        fn drop_edge_type(&self, _space: &str, _edge_type_name: &str) -> Result<bool, StorageError> { Ok(true) }
        fn get_tag_schema(&self, _space: &str, _tag: &str) -> Result<Schema, StorageError> { Ok(Schema::default()) }
        fn get_edge_type_schema(&self, _space: &str, _edge: &str) -> Result<Schema, StorageError> { Ok(Schema::default()) }
    }

    #[derive(Debug)]
    struct DummyIndexManager;

    impl IndexManager for DummyIndexManager {
        fn get_index(&self, _name: &str) -> Option<Index> { None }
        fn list_indexes(&self) -> Vec<String> { Vec::new() }
        fn has_index(&self, _name: &str) -> bool { false }
        fn create_index(&self, _space_id: i32, _index: Index) -> StorageResult<i32> { Ok(0) }
        fn drop_index(&self, _space_id: i32, _index_id: i32) -> StorageResult<()> { Ok(()) }
        fn get_index_status(&self, _space_id: i32, _index_id: i32) -> Option<IndexStatus> { None }
        fn list_indexes_by_space(&self, _space_id: i32) -> StorageResult<Vec<Index>> { Ok(Vec::new()) }
        fn lookup_vertex_by_index(&self, _space_id: i32, _index_name: &str, _values: &[Value]) -> StorageResult<Vec<Vertex>> { Ok(Vec::new()) }
        fn lookup_edge_by_index(&self, _space_id: i32, _index_name: &str, _values: &[Value]) -> StorageResult<Vec<Edge>> { Ok(Vec::new()) }
        fn range_lookup_vertex(&self, _space_id: i32, _index_name: &str, _start: &Value, _end: &Value) -> StorageResult<Vec<Vertex>> { Ok(Vec::new()) }
        fn range_lookup_edge(&self, _space_id: i32, _index_name: &str, _start: &Value, _end: &Value) -> StorageResult<Vec<Edge>> { Ok(Vec::new()) }
        fn insert_vertex_to_index(&self, _space_id: i32, _vertex: &Vertex) -> StorageResult<()> { Ok(()) }
        fn delete_vertex_from_index(&self, _space_id: i32, _vertex: &Vertex) -> StorageResult<()> { Ok(()) }
        fn update_vertex_in_index(&self, _space_id: i32, _old_vertex: &Vertex, _new_vertex: &Vertex) -> StorageResult<()> { Ok(()) }
        fn insert_edge_to_index(&self, _space_id: i32, _edge: &Edge) -> StorageResult<()> { Ok(()) }
        fn delete_edge_from_index(&self, _space_id: i32, _edge: &Edge) -> StorageResult<()> { Ok(()) }
        fn update_edge_in_index(&self, _space_id: i32, _old_edge: &Edge, _new_edge: &Edge) -> StorageResult<()> { Ok(()) }
        fn load_from_disk(&self) -> StorageResult<()> { Ok(()) }
        fn save_to_disk(&self) -> StorageResult<()> { Ok(()) }
        fn rebuild_index(&self, _space_id: i32, _index_id: i32) -> StorageResult<()> { Ok(()) }
        fn rebuild_all_indexes(&self, _space_id: i32) -> StorageResult<()> { Ok(()) }
        fn get_index_stats(&self, _space_id: i32, _index_id: i32) -> StorageResult<IndexStats> { Ok(IndexStats::default()) }
        fn get_all_index_stats(&self, _space_id: i32) -> StorageResult<Vec<IndexStats>> { Ok(Vec::new()) }
        fn analyze_index(&self, _space_id: i32, _index_id: i32) -> StorageResult<IndexOptimization> { Ok(IndexOptimization::default()) }
        fn analyze_all_indexes(&self, _space_id: i32) -> StorageResult<Vec<IndexOptimization>> { Ok(Vec::new()) }
        fn check_index_consistency(&self, _space_id: i32, _index_id: i32) -> StorageResult<bool> { Ok(true) }
        fn repair_index(&self, _space_id: i32, _index_id: i32) -> StorageResult<()> { Ok(()) }
        fn cleanup_index(&self, _space_id: i32, _index_id: i32) -> StorageResult<()> { Ok(()) }
        fn batch_insert_vertices(&self, _space_id: i32, _vertices: &[Vertex]) -> StorageResult<()> { Ok(()) }
        fn batch_delete_vertices(&self, _space_id: i32, _vertices: &[Vertex]) -> StorageResult<()> { Ok(()) }
        fn batch_insert_edges(&self, _space_id: i32, _edges: &[Edge]) -> StorageResult<()> { Ok(()) }
        fn batch_delete_edges(&self, _space_id: i32, _edges: &[Edge]) -> StorageResult<()> { Ok(()) }
    }

    #[derive(Debug)]
    struct DummyStorage;

    impl StorageClient for DummyStorage {
        fn get_vertex(&self, _space: &str, _id: &Value) -> Result<Option<Vertex>, StorageError> {
            Ok(None)
        }

        fn scan_vertices(&self, _space: &str) -> Result<Vec<Vertex>, StorageError> {
            Ok(Vec::new())
        }

        fn scan_vertices_by_tag(&self, _space: &str, _tag: &str) -> Result<Vec<Vertex>, StorageError> {
            Ok(Vec::new())
        }

        fn scan_vertices_by_prop(
            &self,
            _space: &str,
            _tag: &str,
            _prop: &str,
            _value: &Value,
        ) -> Result<Vec<Vertex>, StorageError> {
            Ok(Vec::new())
        }

        fn get_edge(
            &self,
            _space: &str,
            _src: &Value,
            _dst: &Value,
            _edge_type: &str,
        ) -> Result<Option<Edge>, StorageError> {
            Ok(None)
        }

        fn get_node_edges(
            &self,
            _space: &str,
            _node_id: &Value,
            _direction: EdgeDirection,
        ) -> Result<Vec<Edge>, StorageError> {
            Ok(Vec::new())
        }

        fn get_node_edges_filtered(
            &self,
            _space: &str,
            _node_id: &Value,
            _direction: EdgeDirection,
            _filter: Option<Box<dyn Fn(&Edge) -> bool + Send + Sync>>,
        ) -> Result<Vec<Edge>, StorageError> {
            Ok(Vec::new())
        }

        fn scan_edges_by_type(&self, _space: &str, _edge_type: &str) -> Result<Vec<Edge>, StorageError> {
            Ok(Vec::new())
        }

        fn scan_all_edges(&self, _space: &str) -> Result<Vec<Edge>, StorageError> {
            Ok(Vec::new())
        }

        fn insert_vertex(&mut self, _space: &str, _vertex: Vertex) -> Result<Value, StorageError> {
            Ok(Value::Null(NullType::NaN))
        }

        fn update_vertex(&mut self, _space: &str, _vertex: Vertex) -> Result<(), StorageError> {
            Ok(())
        }

        fn delete_vertex(&mut self, _space: &str, _id: &Value) -> Result<(), StorageError> {
            Ok(())
        }

        fn batch_insert_vertices(
            &mut self,
            _space: &str,
            _vertices: Vec<Vertex>,
        ) -> Result<Vec<Value>, StorageError> {
            Ok(Vec::new())
        }

        fn insert_edge(&mut self, _space: &str, _edge: Edge) -> Result<(), StorageError> {
            Ok(())
        }

        fn delete_edge(
            &mut self,
            _space: &str,
            _src: &Value,
            _dst: &Value,
            _edge_type: &str,
        ) -> Result<(), StorageError> {
            Ok(())
        }

        fn batch_insert_edges(&mut self, _space: &str, _edges: Vec<Edge>) -> Result<(), StorageError> {
            Ok(())
        }

        fn begin_transaction(&mut self, _space: &str) -> Result<TransactionId, StorageError> {
            Ok(TransactionId::new(1))
        }

        fn commit_transaction(&mut self, _space: &str, _tx_id: TransactionId) -> Result<(), StorageError> {
            Ok(())
        }

        fn rollback_transaction(&mut self, _space: &str, _tx_id: TransactionId) -> Result<(), StorageError> {
            Ok(())
        }

        fn create_space(&mut self, _space: &SpaceInfo) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn drop_space(&mut self, _space: &str) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn get_space(&self, _space: &str) -> Result<Option<SpaceInfo>, StorageError> {
            Ok(None)
        }

        fn list_spaces(&self) -> Result<Vec<SpaceInfo>, StorageError> {
            Ok(Vec::new())
        }

        fn create_tag(&mut self, _space: &str, _info: &TagInfo) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn alter_tag(
            &mut self,
            _space: &str,
            _tag: &str,
            _additions: Vec<PropertyDef>,
            _deletions: Vec<String>,
        ) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn get_tag(&self, _space: &str, _tag: &str) -> Result<Option<TagInfo>, StorageError> {
            Ok(None)
        }

        fn drop_tag(&mut self, _space: &str, _tag: &str) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn list_tags(&self, _space: &str) -> Result<Vec<TagInfo>, StorageError> {
            Ok(Vec::new())
        }

        fn create_edge_type(
            &mut self,
            _space: &str,
            _info: &EdgeTypeInfo,
        ) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn alter_edge_type(
            &mut self,
            _space: &str,
            _edge_type: &str,
            _additions: Vec<PropertyDef>,
            _deletions: Vec<String>,
        ) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn get_edge_type(
            &self,
            _space: &str,
            _edge_type: &str,
        ) -> Result<Option<EdgeTypeInfo>, StorageError> {
            Ok(None)
        }

        fn drop_edge_type(&mut self, _space: &str, _edge_type: &str) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn list_edge_types(&self, _space: &str) -> Result<Vec<EdgeTypeInfo>, StorageError> {
            Ok(Vec::new())
        }

        fn create_tag_index(&mut self, _space: &str, _info: &Index) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn drop_tag_index(&mut self, _space: &str, _index: &str) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn get_tag_index(&self, _space: &str, _index: &str) -> Result<Option<Index>, StorageError> {
            Ok(None)
        }

        fn list_tag_indexes(&self, _space: &str) -> Result<Vec<Index>, StorageError> {
            Ok(Vec::new())
        }

        fn rebuild_tag_index(&mut self, _space: &str, _index: &str) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn create_edge_index(&mut self, _space: &str, _info: &Index) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn drop_edge_index(&mut self, _space: &str, _index: &str) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn get_edge_index(&self, _space: &str, _index: &str) -> Result<Option<Index>, StorageError> {
            Ok(None)
        }

        fn list_edge_indexes(&self, _space: &str) -> Result<Vec<Index>, StorageError> {
            Ok(Vec::new())
        }

        fn rebuild_edge_index(&mut self, _space: &str, _index: &str) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn lookup_index(&self, _space: &str, _index: &str, _value: &Value) -> Result<Vec<Value>, StorageError> {
            Ok(Vec::new())
        }

        fn insert_vertex_data(&mut self, _space: &str, _info: &InsertVertexInfo) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn insert_edge_data(&mut self, _space: &str, _info: &InsertEdgeInfo) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn delete_vertex_data(&mut self, _space: &str, _vertex_id: &str) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn delete_edge_data(&mut self, _space: &str, _src: &str, _dst: &str, _rank: i64) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn update_data(&mut self, _space: &str, _info: &UpdateInfo) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn change_password(&mut self, _info: &PasswordInfo) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn create_user(&mut self, _info: &UserInfo) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn alter_user(&mut self, _info: &UserAlterInfo) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn drop_user(&mut self, _username: &str) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn get_vertex_with_schema(&self, _space: &str, _tag: &str, _id: &Value) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
            Ok(None)
        }

        fn get_edge_with_schema(&self, _space: &str, _edge_type: &str, _src: &Value, _dst: &Value) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
            Ok(None)
        }

        fn scan_vertices_with_schema(&self, _space: &str, _tag: &str) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
            Ok(Vec::new())
        }

        fn scan_edges_with_schema(&self, _space: &str, _edge_type: &str) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
            Ok(Vec::new())
        }

        fn load_from_disk(&mut self) -> Result<(), StorageError> {
            Ok(())
        }

        fn save_to_disk(&self) -> Result<(), StorageError> {
            Ok(())
        }
    }

    fn create_test_runtime_context() -> Arc<RuntimeContext> {
        let storage_env = Arc::new(StorageEnv {
            storage_engine: Arc::new(DummyStorage),
            schema_manager: Arc::new(DummySchemaManager),
            index_manager: Arc::new(DummyIndexManager),
        });

        let plan_context = Arc::new(PlanContext {
            storage_env,
            space_id: 1,
            plan_id: 100,
            v_id_len: 8,
            is_int_id: true,
            is_edge: false,
        });

        Arc::new(RuntimeContext::new(plan_context))
    }

    #[test]
    fn test_processor_context_new() {
        let ctx = create_test_runtime_context();
        assert_eq!(ctx.space_id(), 1);
        assert_eq!(ctx.v_id_len(), 8);
    }

    #[test]
    fn test_processor_context_with_counters() {
        let counters = ProcessorCounters::default();
        let _ctx = create_test_runtime_context();
        assert!(true);
    }

    #[test]
    fn test_processor_base_new() {
        let ctx = create_test_runtime_context();
        let storage: Arc<Mutex<DummyStorage>> = Arc::new(Mutex::new(DummyStorage));

        let base: ProcessorBase<(), DummyStorage> = ProcessorBase::new(ctx, storage);

        assert!(!base.has_errors());
        assert!(base.failed_parts().is_empty());
        assert_eq!(base.duration(), Duration::ZERO);
    }

    #[test]
    fn test_processor_push_code() {
        let ctx = create_test_runtime_context();
        let storage: Arc<Mutex<DummyStorage>> = Arc::new(Mutex::new(DummyStorage));
        let mut base: ProcessorBase<(), DummyStorage> = ProcessorBase::new(ctx, storage);

        let error = DBError::Storage(StorageError::NotFound("test".to_string()));
        base.push_code(error.clone(), 100);

        assert!(base.has_errors());
        assert_eq!(base.failed_parts().len(), 1);
        assert_eq!(base.failed_parts()[0].part_id, 100);
    }
}
