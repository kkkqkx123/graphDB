//! Data Processing Executor Builder
//!
//! Responsible for creating executors of data processing types (Filter, Project, Limit, Sort, TopN, Sample, Aggregate, Dedup).

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

/// Data Processing Executor Builder
pub struct DataProcessingBuilder<S: StorageClient + Send + 'static> {
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StorageClient + Send + 'static> DataProcessingBuilder<S> {
    /// Create a new data processing builder.
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    /// Building a Filter Executor
    pub fn build_filter(
        node: &FilterNode,
        storage: Arc<Mutex<S>>,
        _context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // The `FilterExecutor::new` method requires a `ContextualExpression`.
        let condition = node.condition().clone();

        let executor = FilterExecutor::new(node.id(), storage, condition);
        Ok(ExecutorEnum::Filter(executor))
    }

    /// Building the Project Executor
    pub fn build_project(
        node: &ProjectNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // Convert YieldColumn to ProjectionColumn.
        // The expression field of the YieldColumn is a ContextualExpression.
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

    /// Building the Limit executor
    pub fn build_limit(
        node: &LimitNode,
        storage: Arc<Mutex<S>>,
        _context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // Parameters of LimitExecutor::new: id, storage, limit, offset
        // 注意：LimitNode 的 offset() 和 count() 返回 i64，需要转换
        let executor = LimitExecutor::new(
            node.id(),
            storage,
            Some(node.count() as usize),
            node.offset() as usize,
        );
        Ok(ExecutorEnum::Limit(executor))
    }

    /// Building the Sort executor
    pub fn build_sort(
        node: &SortNode,
        storage: Arc<Mutex<S>>,
        _context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // The `SortItem` class contains two properties: `column` (of type `String`) and `direction` (of type `OrderDirection`).
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

    /// Building a TopN executor
    pub fn build_topn(
        node: &TopNNode,
        storage: Arc<Mutex<S>>,
        _context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // TopNExecutor::new parameters: id, storage, n, sort_columns, ascending
        // `sort_columns` is a `Vec<String>`, not a `Vec<SortKey>`.
        let sort_columns: Vec<String> = node
            .sort_items()
            .iter()
            .map(|item| item.column.clone())
            .collect();
        // Assume that all sorting directions are consistent; use the direction of the first sorting criterion.
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

    /// Building the Sample Executor
    pub fn build_sample(
        node: &SampleNode,
        storage: Arc<Mutex<S>>,
        _context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // Parameters for SampleExecutor::new: id, storage, method, count, seed
        let executor = SampleExecutor::new(
            node.id(),
            storage,
            SampleMethod::Random,
            node.count() as usize,
            None, // seed
        );
        Ok(ExecutorEnum::Sample(executor))
    }

    /// Building the Aggregate Executor
    pub fn build_aggregate(
        node: &AggregateNode,
        storage: Arc<Mutex<S>>,
        _context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // `group_keys` is a `Vec<String>`, so it can be directly converted to an `Expression::Variable`.
        let group_keys: Vec<crate::core::Expression> = node
            .group_keys()
            .iter()
            .map(|key| crate::core::Expression::Variable(key.clone()))
            .collect();

        // The `AggregateFunction` needs to be converted to `Vec<AggregateFunctionSpec>`.
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

        // The `AggregateExecutor::new` method requires only 4 parameters: `id`, `storage`, `aggregate_functions`, and `group_keys`.
        let executor = AggregateExecutor::new(node.id(), storage, aggregate_functions, group_keys);
        Ok(ExecutorEnum::Aggregate(executor))
    }

    /// Building a Dedup Executor
    pub fn build_dedup(
        node: &DedupNode,
        storage: Arc<Mutex<S>>,
        _context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::query::executor::result_processing::dedup::DedupStrategy;
        // The DedupNode does not have a “keys” field and uses a strategy for complete data deduplication (i.e., removing all duplicate data).
        let strategy = DedupStrategy::Full;

        let executor = DedupExecutor::new(
            node.id(),
            storage,
            strategy,
            None, // Use the default memory limit.
        );
        Ok(ExecutorEnum::Dedup(executor))
    }
}

impl<S: StorageClient + 'static> Default for DataProcessingBuilder<S> {
    fn default() -> Self {
        Self::new()
    }
}
