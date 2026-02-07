//! 存储处理器执行器
//!
//! 此模块提供将 storage::ProcessorBase 与 query::Executor 集成的功能。
//! 它统一了存储层处理和查询层执行的概念，提供了：
//! - 统一的计时和统计
//! - 内存管理
//! - 错误收集和处理
//! - 分区支持
//! - 批量操作支持

use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::core::error::{DBError, DBResult, StorageError, StorageResult};
use crate::core::{Edge, Value, Vertex};
use crate::query::context::runtime_context::RuntimeContext;
use crate::query::executor::base::{
    BaseExecutor, ExecutionResult, ExecutorStats, HasStorage, IntoExecutionResult,
};
use crate::storage::{ProcessorBase, StorageClient};
use crate::utils::safe_lock;

/// 处理器执行器计数器
///
/// 用于跟踪处理器执行的各种指标
#[derive(Debug, Clone, Default)]
pub struct ProcessorExecutorCounters {
    /// 调用次数
    pub num_calls: i64,
    /// 错误次数
    pub num_errors: i64,
    /// 总延迟（微秒）
    pub total_latency_us: i64,
    /// 最大延迟（微秒）
    pub max_latency_us: i64,
    /// 内存峰值（字节）
    pub peak_memory_bytes: u64,
}

/// 存储处理器执行器
///
/// 结合了 ProcessorBase 的功能，提供统一的执行接口。此结构体是连接
/// 底层 StorageProcessor trait 和上层 StorageProcessorExecutorImpl trait 的桥梁，
/// 通过包含 ProcessorBase 实例实现了对 StorageProcessor 功能的复用，
/// 同时提供了异步执行、统计监控和内存管理等高级功能。
/// 注意：此结构体不实现 Clone，因为 StorageClient 不支持 Clone
#[derive(Debug)]
pub struct StorageProcessorExecutor<S: StorageClient + std::fmt::Debug, RESP> {
    /// 基础执行器
    base: BaseExecutor<S>,
    /// 处理器基类
    processor: ProcessorBase<RESP, S>,
    /// 执行器计数器
    counters: ProcessorExecutorCounters,
    /// 执行器统计信息
    stats: ExecutorStats,
    /// 响应类型标识
    _phantom: std::marker::PhantomData<RESP>,
}

impl<S: StorageClient + std::fmt::Debug, RESP> StorageProcessorExecutor<S, RESP> {
    /// 创建新的存储处理器执行器
    pub fn new(
        id: i64,
        name: String,
        storage: Arc<Mutex<S>>,
        context: Arc<RuntimeContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, name, storage.clone()),
            processor: ProcessorBase::new(context, storage),
            counters: ProcessorExecutorCounters::default(),
            stats: ExecutorStats::default(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// 创建带计数器的存储处理器执行器
    pub fn with_counters(
        id: i64,
        name: String,
        storage: Arc<Mutex<S>>,
        context: Arc<RuntimeContext>,
        counters: ProcessorExecutorCounters,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, name, storage.clone()),
            processor: ProcessorBase::new(context, storage),
            counters,
            stats: ExecutorStats::default(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// 开始执行计时
    pub fn start_execute(&mut self) {
        self.processor.start_timer();
    }

    /// 结束执行计时并更新统计
    pub fn finish_execute(&mut self) {
        self.processor.stop_timer();
        let duration = self.processor.duration();
        self.counters.num_calls += 1;
        self.counters.total_latency_us += duration.as_micros() as i64;
        if duration.as_micros() as i64 > self.counters.max_latency_us {
            self.counters.max_latency_us = duration.as_micros() as i64;
        }
    }

    /// 检查内存是否超出限制
    pub fn check_memory_limit(&self) -> bool {
        self.processor.is_memory_exceeded()
    }

    /// 获取处理器基类引用
    pub fn processor(&self) -> &ProcessorBase<RESP, S> {
        &self.processor
    }

    /// 获取可变的处理器基类引用
    pub fn processor_mut(&mut self) -> &mut ProcessorBase<RESP, S> {
        &mut self.processor
    }

    /// 获取计数器
    pub fn counters(&self) -> &ProcessorExecutorCounters {
        &self.counters
    }

    /// 获取平均延迟（微秒）
    pub fn avg_latency_us(&self) -> f64 {
        if self.counters.num_calls > 0 {
            self.counters.total_latency_us as f64 / self.counters.num_calls as f64
        } else {
            0.0
        }
    }

    /// 获取基础执行器
    pub fn base(&self) -> &BaseExecutor<S> {
        &self.base
    }

    /// 获取可变的处理器基类引用（用于错误收集）
    pub fn collect_error(&mut self, error: DBError) {
        let part_id = self.processor.context().part_id.unwrap_or(0);
        self.processor.push_code(error, part_id);
    }

    /// 获取执行器统计信息（不可变引用）
    pub fn stats(&self) -> &ExecutorStats {
        &self.stats
    }

    /// 获取执行器统计信息（可变引用）
    pub fn stats_mut(&mut self) -> &mut ExecutorStats {
        &mut self.stats
    }
}

impl<S: StorageClient, RESP> HasStorage<S> for StorageProcessorExecutor<S, RESP> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.processor.storage()
    }
}

impl<S: StorageClient + Send + Sync + std::fmt::Debug, RESP> StorageProcessorExecutor<S, RESP> where RESP: Default {
    /// 批量获取顶点（便捷方法）
    ///
    /// 使用存储客户端批量获取顶点数据，自动处理错误
    pub async fn batch_get_vertices(&self, ids: &[Value]) -> DBResult<Vec<Option<Vertex>>> {
        let storage = safe_lock(&*self.processor.storage())
            .expect("Storage lock should not be poisoned");
        let mut results = Vec::with_capacity(ids.len());
        
        for id in ids {
            match storage.get_vertex("default", id) {
                Ok(vertex) => results.push(vertex),
                Err(e) => return Err(DBError::from(e)),
            }
        }
        
        Ok(results)
    }
    
    /// 批量获取边（便捷方法）
    ///
    /// 使用存储客户端批量获取边数据
    pub async fn batch_get_edges(
        &self,
        edge_keys: &[(Value, Value, String)],
    ) -> DBResult<Vec<Option<Edge>>> {
        let storage = safe_lock(&*self.processor.storage())
            .expect("Storage lock should not be poisoned");
        let mut results = Vec::with_capacity(edge_keys.len());
        
        for (src, dst, edge_type) in edge_keys {
            match storage.get_edge("default", src, dst, edge_type) {
                Ok(edge) => results.push(edge),
                Err(e) => return Err(DBError::from(e)),
            }
        }
        
        Ok(results)
    }
    
    /// 批量扫描顶点
    pub async fn batch_scan_vertices(&self, tag: Option<&str>, limit: Option<usize>) -> DBResult<Vec<Vertex>> {
        let storage = safe_lock(&*self.processor.storage())
            .expect("Storage lock should not be poisoned");
        
        let vertices = match tag {
            Some(tag_name) => storage.scan_vertices_by_tag("default", tag_name),
            None => storage.scan_vertices("default"),
        };
        
        let mut vertices = vertices.map_err(DBError::from)?;
        
        if let Some(limit) = limit {
            vertices.truncate(limit);
        }
        
        Ok(vertices)
    }
    
    /// 批量扫描边
    pub async fn batch_scan_edges(&self, edge_type: Option<&str>, limit: Option<usize>) -> DBResult<Vec<Edge>> {
        let storage = safe_lock(&*self.processor.storage())
            .expect("Storage lock should not be poisoned");
        
        let edges = match edge_type {
            Some(type_name) => storage.scan_edges_by_type("default", type_name),
            None => storage.scan_all_edges("default"),
        };
        
        let mut edges = edges.map_err(DBError::from)?;
        
        if let Some(limit) = limit {
            edges.truncate(limit);
        }
        
        Ok(edges)
    }
    
    /// 创建批量操作优化器
    pub fn batch_optimizer(&self) -> crate::query::executor::BatchOptimizer<S> {
        crate::query::executor::BatchOptimizer::with_default_config(self.processor.storage().clone())
    }
    
    /// 创建并发控制器
    pub fn concurrency_controller(&self) -> crate::query::executor::ConcurrencyController<S> {
        crate::query::executor::ConcurrencyController::with_default_config(self.processor.storage().clone())
    }
}

/// 通用存储处理器执行 trait
///
/// 为具体的处理器执行器提供通用实现。此 trait 通过 StorageProcessorExecutor
/// 结构体重用了 StorageProcessor trait 的功能（通过其中的 ProcessorBase 实现），
/// 提供了异步执行、统计监控和内存管理等高级功能。
#[async_trait]
pub trait StorageProcessorExecutorImpl<S, RESP>
where
    S: StorageClient,
    RESP: Send + Sync + Clone + IntoExecutionResult,
{
    /// 获取执行器实例
    fn get_executor(&mut self) -> &mut StorageProcessorExecutor<S, RESP>;

    /// 实际执行逻辑（由具体实现提供）
    async fn do_execute(&mut self) -> DBResult<RESP>;

    /// 执行并收集错误
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        // 内存检查
        if self.get_executor().check_memory_limit() {
            return Err(DBError::Storage(
                StorageError::DbError("Memory exceeded".to_string()),
            ));
        }

        // 开始计时
        self.get_executor().start_execute();

        // 执行实际逻辑
        let result = self.do_execute().await;

        // 结束计时并更新统计
        self.get_executor().finish_execute();

        // 处理结果或错误
        match result {
            Ok(response) => {
                self.get_executor().processor_mut().set_response(response.clone());
                Ok(response.into_execution_result())
            }
            Err(e) => {
                self.get_executor().collect_error(e.clone());
                Err(e)
            }
        }
    }
}

impl ExecutionResult {
    /// 从顶点列表创建执行结果
    pub fn from_vertices(vertices: Vec<crate::core::Vertex>) -> Self {
        ExecutionResult::Vertices(vertices)
    }

    /// 从边列表创建执行结果
    pub fn from_edges(edges: Vec<crate::core::Edge>) -> Self {
        ExecutionResult::Edges(edges)
    }

    /// 从顶点ID列表创建执行结果
    pub fn from_vertex_ids(ids: Vec<crate::core::Value>) -> Self {
        ExecutionResult::Values(ids)
    }

    /// 从泛型响应创建执行结果
    pub fn from_response<RESP: IntoExecutionResult>(response: RESP) -> Self {
        response.into_execution_result()
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

        fn create_user(&mut self, _info: &crate::core::types::metadata::UserInfo) -> Result<bool, StorageError> {
            Ok(true)
        }

        fn alter_user(&mut self, _info: &crate::core::types::metadata::UserAlterInfo) -> Result<bool, StorageError> {
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
    fn test_storage_processor_executor_new() {
        let storage: Arc<Mutex<DummyStorage>> = Arc::new(Mutex::new(DummyStorage));
        let context = create_test_runtime_context();

        let executor = StorageProcessorExecutor::<DummyStorage, Vec<crate::core::Value>>::new(
            1,
            "TestExecutor".to_string(),
            storage,
            context,
        );

        assert_eq!(executor.base.id, 1);
        assert_eq!(executor.base.name, "TestExecutor");
        assert_eq!(executor.counters.num_calls, 0);
    }

    #[test]
    fn test_execute_timing() {
        let storage: Arc<Mutex<DummyStorage>> = Arc::new(Mutex::new(DummyStorage));
        let context = create_test_runtime_context();

        let mut executor = StorageProcessorExecutor::<DummyStorage, Vec<crate::core::Value>>::new(
            1,
            "TestExecutor".to_string(),
            storage,
            context,
        );

        executor.start_execute();
        std::thread::sleep(Duration::from_millis(10));
        executor.finish_execute();

        assert_eq!(executor.counters.num_calls, 1);
        assert!(executor.counters.total_latency_us >= 10000); // 至少10ms
    }

    #[test]
    fn test_memory_limit_check() {
        let storage: Arc<Mutex<DummyStorage>> = Arc::new(Mutex::new(DummyStorage));
        let context = create_test_runtime_context();

        let executor = StorageProcessorExecutor::<DummyStorage, Vec<crate::core::Value>>::new(
            1,
            "TestExecutor".to_string(),
            storage,
            context,
        );

        // 默认不超限
        assert!(!executor.check_memory_limit());
    }

    #[test]
    fn test_into_execution_result_values() {
        let values = vec![crate::core::Value::Int(1), crate::core::Value::Int(2)];
        let result = ExecutionResult::from_response(values);
        
        match result {
            ExecutionResult::Values(v) => assert_eq!(v.len(), 2),
            _ => panic!("Expected Values"),
        }
    }
}
