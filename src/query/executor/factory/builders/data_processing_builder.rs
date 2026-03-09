//! 数据处理执行器构建器
//!
//! 负责创建数据处理类型的执行器（Filter, Project, Limit, Sort, TopN, Sample, Aggregate, Dedup）

use crate::core::error::QueryError;
use crate::query::executor::base::ExecutionContext;
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::executor::result_processing::{
    AggregateExecutor, DedupExecutor, FilterExecutor, LimitExecutor, ProjectExecutor,
    SampleExecutor, SampleMethod, SortExecutor, TopNExecutor,
};
use crate::query::planner::plan::core::nodes::{
    AggregateNode, DedupNode, FilterNode, LimitNode, ProjectNode, SampleNode, SortNode, TopNNode,
};
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::sync::Arc;

/// 数据处理执行器构建器
pub struct DataProcessingBuilder<S: StorageClient + 'static> {
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StorageClient + 'static> DataProcessingBuilder<S> {
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
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let condition = node
            .condition()
            .expression()
            .map(|meta| meta.inner().clone())
            .ok_or_else(|| {
                QueryError::ExecutionError("Filter节点缺少条件表达式".to_string())
            })?;

        let executor = FilterExecutor::new(
            node.id(),
            storage,
            condition,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::Filter(executor))
    }

    /// 构建 Project 执行器
    pub fn build_project(
        &self,
        node: &ProjectNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // YieldColumn 包含 expression (ContextualExpression) 和 alias (String)
        let columns: Vec<(String, crate::core::Expression)> = node
            .columns()
            .iter()
            .filter_map(|col| {
                col.expression
                    .expression()
                    .map(|meta| (col.alias.clone(), meta.inner().clone()))
            })
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
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let executor = LimitExecutor::new(
            node.id(),
            storage,
            node.offset() as usize,
            node.count() as usize,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::Limit(executor))
    }

    /// 构建 Sort 执行器
    pub fn build_sort(
        &self,
        node: &SortNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // SortItem 包含 column (String) 和 direction (OrderDirection)
        let sort_keys: Vec<(crate::core::Expression, bool)> = node
            .sort_items()
            .iter()
            .map(|item| {
                let expr = crate::core::Expression::Variable(item.column.clone());
                let is_desc = item.direction == crate::core::types::graph_schema::OrderDirection::Desc;
                (expr, is_desc)
            })
            .collect();

        let executor = SortExecutor::new(
            node.id(),
            storage,
            sort_keys,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::Sort(executor))
    }

    /// 构建 TopN 执行器
    pub fn build_topn(
        &self,
        node: &TopNNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // SortItem 包含 column (String) 和 direction (OrderDirection)
        let sort_keys: Vec<(crate::core::Expression, bool)> = node
            .sort_items()
            .iter()
            .map(|item| {
                let expr = crate::core::Expression::Variable(item.column.clone());
                let is_desc = item.direction == crate::core::types::graph_schema::OrderDirection::Desc;
                (expr, is_desc)
            })
            .collect();

        let executor = TopNExecutor::new(
            node.id(),
            storage,
            node.limit() as usize,
            sort_keys,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::TopN(executor))
    }

    /// 构建 Sample 执行器
    pub fn build_sample(
        &self,
        node: &SampleNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // SampleNode 只有 count 字段，使用默认的 Random 采样方法
        let executor = SampleExecutor::new(
            node.id(),
            storage,
            node.count() as usize,
            SampleMethod::Random,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::Sample(executor))
    }

    /// 构建 Aggregate 执行器
    pub fn build_aggregate(
        &self,
        node: &AggregateNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // group_keys 是 Vec<String>，直接转换为 Expression::Variable
        let group_keys: Vec<crate::core::Expression> = node
            .group_keys()
            .iter()
            .map(|key| crate::core::Expression::Variable(key.clone()))
            .collect();

        // AggregateFunction 需要转换为 (String, String, Expression) 格式
        let aggregates: Vec<(String, String, crate::core::Expression)> = node
            .aggregation_functions()
            .iter()
            .map(|agg_func| {
                // AggregateFunction 的 name() 返回函数名
                let func_name = agg_func.name().to_string();
                // 使用函数名作为别名
                let alias = func_name.clone();
                // 聚合函数的参数表达式
                let expr = crate::core::Expression::Null(crate::core::NullType::Unknown);
                (alias, func_name, expr)
            })
            .collect();

        let executor = AggregateExecutor::new(
            node.id(),
            storage,
            group_keys,
            aggregates,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::Aggregate(executor))
    }

    /// 构建 Dedup 执行器
    pub fn build_dedup(
        &self,
        node: &DedupNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // DedupNode 没有 keys 字段，使用空列表
        let keys: Vec<crate::core::Expression> = Vec::new();

        let executor = DedupExecutor::new(
            node.id(),
            storage,
            keys,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::Dedup(executor))
    }
}

impl<S: StorageClient + 'static> Default for DataProcessingBuilder<S> {
    fn default() -> Self {
        Self::new()
    }
}
