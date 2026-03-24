//! 数据处理执行器构建器
//!
//! 负责创建数据处理类型的执行器（Filter, Project, Limit, Sort, TopN, Sample, Aggregate, Dedup）

use crate::core::error::QueryError;
use crate::query::executor::base::ExecutionContext;
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::executor::result_processing::{
    AggregateExecutor, AggregateFunctionSpec, DedupExecutor, FilterExecutor, LimitExecutor,
    ProjectExecutor, ProjectionColumn, SampleExecutor, SampleMethod, SortExecutor, SortKey,
    TopNExecutor,
};
use crate::query::planning::plan::core::nodes::{
    AggregateNode, DedupNode, FilterNode, LimitNode, ProjectNode, SampleNode, SortNode, TopNNode,
};
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::sync::Arc;

/// 数据处理执行器构建器
pub struct DataProcessingBuilder<S: StorageClient + Send + 'static> {
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StorageClient + Send + 'static> DataProcessingBuilder<S> {
    /// 创建新的数据处理构建器
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    /// 构建 Filter 执行器
    pub fn build_filter(
        &self,
        node: &FilterNode,
        storage: Arc<Mutex<S>>,
        _context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // FilterExecutor::new 需要 ContextualExpression
        let condition = node.condition().clone();

        let executor = FilterExecutor::new(node.id(), storage, condition);
        Ok(ExecutorEnum::Filter(executor))
    }

    /// 构建 Project 执行器
    pub fn build_project(
        &self,
        node: &ProjectNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // 将 YieldColumn 转换为 ProjectionColumn
        // YieldColumn 的 expression 字段是 ContextualExpression
        let columns: Vec<ProjectionColumn> = node
            .columns()
            .iter()
            .map(|col| ProjectionColumn::new(col.alias.clone(), col.expression.clone()))
            .collect();

        let executor = ProjectExecutor::new(
            node.id(),
            storage,
            columns,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::Project(executor))
    }

    /// 构建 Limit 执行器
    pub fn build_limit(
        &self,
        node: &LimitNode,
        storage: Arc<Mutex<S>>,
        _context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // LimitExecutor::new 参数: id, storage, limit, offset
        // 注意：LimitNode 的 offset() 和 count() 返回 i64，需要转换
        let executor = LimitExecutor::new(
            node.id(),
            storage,
            Some(node.count() as usize),
            node.offset() as usize,
        );
        Ok(ExecutorEnum::Limit(executor))
    }

    /// 构建 Sort 执行器
    pub fn build_sort(
        &self,
        node: &SortNode,
        storage: Arc<Mutex<S>>,
        _context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // SortItem 包含 column (String) 和 direction (OrderDirection)
        let sort_keys: Vec<SortKey> = node
            .sort_items()
            .iter()
            .map(|item| {
                let expr = crate::core::Expression::Variable(item.column.clone());
                let order =
                    if item.direction == crate::core::types::graph_schema::OrderDirection::Desc {
                        crate::query::executor::result_processing::SortOrder::Desc
                    } else {
                        crate::query::executor::result_processing::SortOrder::Asc
                    };
                SortKey::new(expr, order)
            })
            .collect();

        use crate::query::executor::result_processing::SortConfig;
        let executor = SortExecutor::new(
            node.id(),
            storage,
            sort_keys,
            None, // limit
            SortConfig::default(),
        )
        .map_err(|e| QueryError::ExecutionError(e.to_string()))?;

        Ok(ExecutorEnum::Sort(executor))
    }

    /// 构建 TopN 执行器
    pub fn build_topn(
        &self,
        node: &TopNNode,
        storage: Arc<Mutex<S>>,
        _context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // TopNExecutor::new 参数: id, storage, n, sort_columns, ascending
        // sort_columns 是 Vec<String>，不是 Vec<SortKey>
        let sort_columns: Vec<String> = node
            .sort_items()
            .iter()
            .map(|item| item.column.clone())
            .collect();
        // 假设所有排序方向一致，使用第一个排序项的方向
        let ascending = node
            .sort_items()
            .first()
            .map(|item| item.direction != crate::core::types::graph_schema::OrderDirection::Desc)
            .unwrap_or(true);

        let executor = TopNExecutor::new(
            node.id(),
            storage,
            node.limit() as usize,
            sort_columns,
            ascending,
        );
        Ok(ExecutorEnum::TopN(executor))
    }

    /// 构建 Sample 执行器
    pub fn build_sample(
        &self,
        node: &SampleNode,
        storage: Arc<Mutex<S>>,
        _context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // SampleExecutor::new 参数: id, storage, method, count, seed
        let executor = SampleExecutor::new(
            node.id(),
            storage,
            SampleMethod::Random,
            node.count() as usize,
            None, // seed
        );
        Ok(ExecutorEnum::Sample(executor))
    }

    /// 构建 Aggregate 执行器
    pub fn build_aggregate(
        &self,
        node: &AggregateNode,
        storage: Arc<Mutex<S>>,
        _context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // group_keys 是 Vec<String>，直接转换为 Expression::Variable
        let group_keys: Vec<crate::core::Expression> = node
            .group_keys()
            .iter()
            .map(|key| crate::core::Expression::Variable(key.clone()))
            .collect();

        // AggregateFunction 需要转换为 Vec<AggregateFunctionSpec>
        let aggregate_functions: Vec<AggregateFunctionSpec> = node
            .aggregation_functions()
            .iter()
            .map(|agg_func| {
                // AggregateFunction 的 name() 返回函数名
                let func_name = agg_func.name().to_string();
                AggregateFunctionSpec::new(
                    crate::query::executor::result_processing::AggregateFunction::Count(None),
                )
                .with_field(func_name)
            })
            .collect();

        // AggregateExecutor::new 只需要4个参数: id, storage, aggregate_functions, group_keys
        let executor = AggregateExecutor::new(node.id(), storage, aggregate_functions, group_keys);
        Ok(ExecutorEnum::Aggregate(executor))
    }

    /// 构建 Dedup 执行器
    pub fn build_dedup(
        &self,
        node: &DedupNode,
        storage: Arc<Mutex<S>>,
        _context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::query::executor::result_processing::dedup::DedupStrategy;
        // DedupNode 没有 keys 字段，使用完全去重策略
        let strategy = DedupStrategy::Full;

        let executor = DedupExecutor::new(
            node.id(),
            storage,
            strategy,
            None, // 使用默认内存限制
        );
        Ok(ExecutorEnum::Dedup(executor))
    }
}

impl<S: StorageClient + 'static> Default for DataProcessingBuilder<S> {
    fn default() -> Self {
        Self::new()
    }
}
