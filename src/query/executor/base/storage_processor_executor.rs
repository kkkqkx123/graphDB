//! 存储处理器执行器
//!
//! 此模块提供将 storage::ProcessorBase 与 query::Executor 集成的功能。
//! 它统一了存储层处理和查询层执行的概念，提供了：
//! - 统一的计时和统计
//! - 内存管理
//! - 错误收集和处理
//! - 分区支持

use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::core::error::{DBError, DBResult, StorageError};
use crate::query::context::runtime_context::RuntimeContext;
use crate::query::executor::base::{
    BaseExecutor, ExecutionResult, ExecutorStats, HasStorage,
};
use crate::storage::{ProcessorBase, StorageClient};

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
/// 结合了 ProcessorBase 的功能，提供统一的执行接口
/// 注意：此结构体不实现 Clone，因为 StorageClient 不支持 Clone
pub struct StorageProcessorExecutor<S: StorageClient, RESP> {
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

impl<S: StorageClient, RESP> StorageProcessorExecutor<S, RESP> {
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

/// 通用存储处理器执行 trait
///
/// 为具体的处理器执行器提供通用实现
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

/// 支持转换为执行结果的 trait
pub trait IntoExecutionResult {
    fn into_execution_result(self) -> ExecutionResult;
}

impl IntoExecutionResult for Vec<crate::core::Value> {
    fn into_execution_result(self) -> ExecutionResult {
        ExecutionResult::Values(self)
    }
}

impl IntoExecutionResult for Vec<crate::core::Vertex> {
    fn into_execution_result(self) -> ExecutionResult {
        ExecutionResult::Vertices(self)
    }
}

impl IntoExecutionResult for Vec<crate::core::Edge> {
    fn into_execution_result(self) -> ExecutionResult {
        ExecutionResult::Edges(self)
    }
}

impl IntoExecutionResult for crate::core::DataSet {
    fn into_execution_result(self) -> ExecutionResult {
        ExecutionResult::DataSet(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::runtime_context::{PlanContext, RuntimeContext, StorageEnv};
    use crate::storage::metadata::SchemaManager;
    use crate::storage::index::IndexManager;
    use std::sync::Mutex;

    struct DummySchemaManager;
    impl SchemaManager for DummySchemaManager {}

    struct DummyIndexManager;
    impl IndexManager for DummyIndexManager {}

    struct DummyStorage;
    impl StorageClient for DummyStorage {
        fn get(&self, _space: u32, _part: u32, _key: &[u8]) -> StorageResult<Option<Vec<u8>>> {
            Ok(None)
        }

        fn put(&self, _space: u32, _part: u32, _key: &[u8], _value: &[u8]) -> StorageResult<()> {
            Ok(())
        }

        fn delete(&self, _space: u32, _part: u32, _key: &[u8]) -> StorageResult<()> {
            Ok(())
        }

        fn scan(
            &self,
            _space: u32,
            _part: u32,
            _start: &[u8],
            _end: &[u8],
        ) -> StorageResult<Box<dyn crate::storage::StorageIter>> {
            Ok(Box::new(crate::storage::DefaultIter::default()))
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
