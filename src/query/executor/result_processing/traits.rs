//! 结果处理执行器统一接口
//!
//! 定义了所有结果处理执行器的统一接口和公共行为

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::core::error::{DBError, DBResult};
use crate::core::value::DataSet;
use crate::query::executor::traits::ExecutionResult;
use crate::storage::StorageEngine;

/// 结果处理器上下文
///
/// 提供执行器运行时所需的上下文信息
#[derive(Debug, Clone)]
pub struct ResultProcessorContext {
    /// 内存限制（字节）
    pub memory_limit: Option<usize>,
    /// 是否启用并行处理
    pub enable_parallel: bool,
    /// 并行度
    pub parallel_degree: Option<usize>,
    /// 是否启用磁盘溢出
    pub enable_disk_spill: bool,
    /// 临时目录路径
    pub temp_dir: Option<String>,
}

impl Default for ResultProcessorContext {
    fn default() -> Self {
        Self {
            memory_limit: Some(100 * 1024 * 1024), // 默认100MB
            enable_parallel: true,
            parallel_degree: None, // 使用系统默认
            enable_disk_spill: false,
            temp_dir: None,
        }
    }
}

/// 结果处理器统一接口
///
/// 所有结果处理执行器都应该实现此接口
#[async_trait]
pub trait ResultProcessor<S: StorageEngine> {
    /// 处理输入数据并返回结果
    async fn process(&mut self, input: ExecutionResult) -> DBResult<ExecutionResult>;

    /// 设置输入数据
    fn set_input(&mut self, input: ExecutionResult);

    /// 获取当前输入数据
    fn get_input(&self) -> Option<&ExecutionResult>;

    /// 获取处理上下文
    fn context(&self) -> &ResultProcessorContext;

    /// 设置处理上下文
    fn set_context(&mut self, context: ResultProcessorContext);

    /// 获取内存使用量
    fn memory_usage(&self) -> usize;

    /// 重置处理器状态
    fn reset(&mut self);

    /// 验证输入数据是否有效
    fn validate_input(&self, input: &ExecutionResult) -> DBResult<()> {
        match input {
            ExecutionResult::DataSet(_) => Ok(()),
            ExecutionResult::Values(_) => Ok(()),
            ExecutionResult::Vertices(_) => Ok(()),
            ExecutionResult::Edges(_) => Ok(()),
            ExecutionResult::Paths(_) => Ok(()),
            ExecutionResult::Count(_) => Ok(()),
            ExecutionResult::Success => Ok(()),
            ExecutionResult::Error(_) => Ok(()),
            ExecutionResult::Result(_) => Ok(()),
        }
    }
}

/// 结果处理器基础实现
///
/// 提供通用的结果处理器功能，其他执行器可以继承此基础实现
pub struct BaseResultProcessor<S: StorageEngine> {
    /// 执行器ID
    pub id: i64,
    /// 执行器名称
    pub name: String,
    /// 执行器描述
    pub description: String,
    /// 存储引擎引用
    pub storage: Arc<Mutex<S>>,
    /// 输入数据
    pub input: Option<ExecutionResult>,
    /// 处理上下文
    pub context: ResultProcessorContext,
    /// 当前内存使用量
    pub memory_usage: usize,
    /// 执行统计信息
    pub stats: crate::query::executor::traits::ExecutorStats,
}

impl<S: StorageEngine> BaseResultProcessor<S> {
    /// 创建新的基础结果处理器
    pub fn new(id: i64, name: String, description: String, storage: Arc<Mutex<S>>) -> Self {
        Self {
            id,
            name,
            description,
            storage,
            input: None,
            context: ResultProcessorContext::default(),
            memory_usage: 0,
            stats: crate::query::executor::traits::ExecutorStats::new(),
        }
    }

    /// 获取执行统计信息
    pub fn get_stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        &self.stats
    }

    /// 获取可变的执行统计信息
    pub fn get_stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        &mut self.stats
    }

    /// 设置内存限制
    pub fn with_memory_limit(mut self, limit: usize) -> Self {
        self.context.memory_limit = Some(limit);
        self
    }

    /// 启用并行处理
    pub fn with_parallel(mut self, enable: bool) -> Self {
        self.context.enable_parallel = enable;
        self
    }

    /// 设置并行度
    pub fn with_parallel_degree(mut self, degree: usize) -> Self {
        self.context.parallel_degree = Some(degree);
        self
    }

    /// 启用磁盘溢出
    pub fn with_disk_spill(mut self, enable: bool) -> Self {
        self.context.enable_disk_spill = enable;
        self
    }

    /// 设置临时目录
    pub fn with_temp_dir(mut self, dir: String) -> Self {
        self.context.temp_dir = Some(dir);
        self
    }

    /// 检查内存限制
    pub fn check_memory_limit(&self) -> DBResult<()> {
        if let Some(limit) = self.context.memory_limit {
            if self.memory_usage > limit {
                return Err(DBError::Query(
                    crate::core::error::QueryError::ExecutionError(format!(
                        "内存使用超出限制: {} > {}",
                        self.memory_usage, limit
                    )),
                ));
            }
        }
        Ok(())
    }

    /// 更新内存使用量
    pub fn update_memory_usage(&mut self, delta: isize) {
        if delta >= 0 {
            self.memory_usage += delta as usize;
        } else if self.memory_usage >= (-delta) as usize {
            self.memory_usage -= (-delta) as usize;
        } else {
            self.memory_usage = 0;
        }
    }

    /// 估算数据集内存使用量
    pub fn estimate_dataset_memory_usage(dataset: &DataSet) -> usize {
        let mut usage = std::mem::size_of::<DataSet>();
        usage += dataset.col_names.len() * std::mem::size_of::<String>();

        for row in &dataset.rows {
            usage += std::mem::size_of::<Vec<crate::core::Value>>();
            for _value in row {
                usage += std::mem::size_of::<crate::core::Value>();
                // 这里可以添加更精确的值大小估算
            }
        }

        usage
    }

    /// 重置处理器状态
    pub fn reset_state(&mut self) {
        self.memory_usage = 0;
        self.input = None;
        self.stats = crate::query::executor::traits::ExecutorStats::new();
    }
}

/// 可流式处理的结果处理器
///
/// 支持流式处理大数据集，避免一次性加载所有数据到内存
#[async_trait]
pub trait StreamableResultProcessor<S: StorageEngine>: ResultProcessor<S> {
    /// 流式处理数据集
    async fn process_stream(
        &mut self,
        input_stream: Box<dyn futures::Stream<Item = DBResult<ExecutionResult>> + Send + Unpin>,
    ) -> DBResult<ExecutionResult>;

    /// 设置批处理大小
    fn set_batch_size(&mut self, batch_size: usize);

    /// 获取批处理大小
    fn batch_size(&self) -> usize;
}

/// 可并行处理的结果处理器
///
/// 支持多线程并行处理以提高性能
#[async_trait]
pub trait ParallelResultProcessor<S: StorageEngine>: ResultProcessor<S> {
    /// 并行处理数据集
    async fn process_parallel(&mut self, input: ExecutionResult) -> DBResult<ExecutionResult>;

    /// 设置并行度
    fn set_parallel_degree(&mut self, degree: usize);

    /// 获取并行度
    fn parallel_degree(&self) -> usize;
}

/// 结果处理器工厂
///
/// 用于创建不同类型的结果处理器实例
pub struct ResultProcessorFactory;

impl ResultProcessorFactory {
    /// 创建投影处理器
    pub fn create_projector<S: StorageEngine>(
        id: i64,
        storage: Arc<Mutex<S>>,
        columns: Vec<crate::query::executor::result_processing::projection::ProjectionColumn>,
    ) -> crate::query::executor::result_processing::projection::ProjectExecutor<S> {
        crate::query::executor::result_processing::projection::ProjectExecutor::new(
            id, storage, columns,
        )
    }

    /// 创建排序处理器
    pub fn create_sorter<S: StorageEngine>(
        id: i64,
        storage: Arc<Mutex<S>>,
        sort_keys: Vec<crate::query::executor::result_processing::sort::SortKey>,
        limit: Option<usize>,
    ) -> DBResult<crate::query::executor::result_processing::sort::SortExecutor<S>> {
        let config = crate::query::executor::result_processing::sort::SortConfig::default();
        crate::query::executor::result_processing::sort::SortExecutor::new(
            id, storage, sort_keys, limit, config,
        )
    }

    /// 创建限制处理器
    pub fn create_limiter<S: StorageEngine>(
        id: i64,
        storage: Arc<Mutex<S>>,
        limit: Option<usize>,
        offset: usize,
    ) -> crate::query::executor::result_processing::limit::LimitExecutor<S> {
        crate::query::executor::result_processing::limit::LimitExecutor::new(
            id, storage, limit, offset,
        )
    }

    /// 创建聚合处理器
    pub fn create_aggregator<S: StorageEngine>(
        id: i64,
        storage: Arc<Mutex<S>>,
        aggregate_functions: Vec<crate::core::types::operators::AggregateFunction>,
        group_keys: Vec<crate::core::Expression>,
    ) -> crate::query::executor::result_processing::aggregation::AggregateExecutor<S> {
        let agg_specs = aggregate_functions
            .into_iter()
            .map(|func| crate::query::executor::result_processing::aggregation::AggregateFunctionSpec::from_agg_function(func))
            .collect();

        crate::query::executor::result_processing::aggregation::AggregateExecutor::new(
            id, storage, agg_specs, group_keys,
        )
    }

    /// 创建去重处理器
    pub fn create_deduper<S: StorageEngine>(
        id: i64,
        storage: Arc<Mutex<S>>,
        strategy: crate::query::executor::result_processing::dedup::DedupStrategy,
        memory_limit: Option<usize>,
    ) -> crate::query::executor::result_processing::dedup::DedupExecutor<S> {
        crate::query::executor::result_processing::dedup::DedupExecutor::new(
            id,
            storage,
            strategy,
            memory_limit,
        )
    }

    /// 创建过滤处理器
    pub fn create_filter<S: StorageEngine>(
        id: i64,
        storage: Arc<Mutex<S>>,
        condition: crate::core::Expression,
    ) -> crate::query::executor::result_processing::filter::FilterExecutor<S> {
        crate::query::executor::result_processing::filter::FilterExecutor::new(
            id, storage, condition,
        )
    }

    /// 创建采样处理器
    pub fn create_sampler<S: StorageEngine>(
        id: i64,
        storage: Arc<Mutex<S>>,
        method: crate::query::executor::result_processing::sample::SampleMethod,
        count: usize,
        seed: Option<u64>,
    ) -> crate::query::executor::result_processing::sample::SampleExecutor<S> {
        crate::query::executor::result_processing::sample::SampleExecutor::new(
            id, storage, method, count, seed,
        )
    }

    /// 创建TopN处理器
    pub fn create_topn<S: StorageEngine>(
        id: i64,
        storage: Arc<Mutex<S>>,
        n: usize,
        sort_columns: Vec<String>,
        ascending: bool,
        _offset: usize,
    ) -> crate::query::executor::result_processing::topn::TopNExecutor<S> {
        crate::query::executor::result_processing::topn::TopNExecutor::new(
            id,
            storage,
            n,
            sort_columns,
            ascending,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value::DataSet;

    #[test]
    fn test_result_processor_context_default() {
        let context = ResultProcessorContext::default();
        assert_eq!(context.memory_limit, Some(100 * 1024 * 1024));
        assert!(context.enable_parallel);
        assert!(context.parallel_degree.is_none());
        assert!(!context.enable_disk_spill);
        assert!(context.temp_dir.is_none());
    }

    #[test]
    fn test_base_result_processor() {
        // 这里需要模拟存储引擎，暂时跳过具体实现
        // let storage = Arc::new(Mutex::new(MockStorageEngine::new()));
        // let processor = BaseResultProcessor::new(
        //     1,
        //     "test".to_string(),
        //     "test processor".to_string(),
        //     storage,
        // );
        // assert_eq!(processor.id, 1);
        // assert_eq!(processor.name, "test");
    }

    #[test]
    fn test_estimate_dataset_memory_usage() {
        use crate::storage::test_mock::MockStorage;

        let mut dataset = DataSet::new();
        dataset.col_names = vec!["col1".to_string(), "col2".to_string()];
        dataset.rows.push(vec![
            crate::core::Value::Int(1),
            crate::core::Value::String("test".to_string()),
        ]);

        // 测试内存使用估算
        let usage = BaseResultProcessor::<MockStorage>::estimate_dataset_memory_usage(&dataset);
        assert!(usage > 0);
    }
}
