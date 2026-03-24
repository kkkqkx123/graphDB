//! 连接执行器构建器
//!
//! 负责创建连接类型的执行器（InnerJoin, LeftJoin, FullOuterJoin, CrossJoin）

use crate::core::error::QueryError;
use crate::query::executor::base::ExecutionContext;
use crate::query::executor::data_processing::{
    CrossJoinExecutor, FullOuterJoinExecutor, InnerJoinExecutor, LeftJoinExecutor,
};
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::planning::plan::core::nodes::base::plan_node_traits::JoinNode;
use crate::query::planning::plan::core::nodes::{
    CrossJoinNode, FullOuterJoinNode, HashInnerJoinNode, HashLeftJoinNode, InnerJoinNode,
    LeftJoinNode,
};
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::sync::Arc;

/// 连接执行器构建器
pub struct JoinBuilder<S: StorageClient + Send + 'static> {
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StorageClient + Send + 'static> JoinBuilder<S> {
    /// 创建新的连接构建器
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    /// 提取连接操作的变量名
    fn extract_join_vars<N: JoinNode>(node: &N) -> (String, String) {
        // 使用节点的 output_var 作为变量名，如果未设置则生成默认值
        let left_var = node
            .left_input()
            .output_var()
            .map(|v| v.to_string())
            .unwrap_or_else(|| format!("left_{}", node.id()));
        let right_var = node
            .right_input()
            .output_var()
            .map(|v| v.to_string())
            .unwrap_or_else(|| format!("right_{}", node.id()));
        (left_var, right_var)
    }

    /// 构建 InnerJoin 执行器
    pub fn build_inner_join(
        &self,
        node: &InnerJoinNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let (left_var, right_var) = Self::extract_join_vars(node);
        let hash_keys: Vec<crate::core::types::ContextualExpression> = node.hash_keys().to_vec();
        let probe_keys: Vec<crate::core::types::ContextualExpression> = node.probe_keys().to_vec();

        let executor = InnerJoinExecutor::new(
            node.id(),
            storage,
            context.expression_context().clone(),
            hash_keys,
            probe_keys,
            left_var,
            right_var,
            node.col_names().to_vec(),
        );
        Ok(ExecutorEnum::InnerJoin(executor))
    }

    /// 构建 HashInnerJoin 执行器
    pub fn build_hash_inner_join(
        &self,
        node: &HashInnerJoinNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let (left_var, right_var) = Self::extract_join_vars(node);
        let hash_keys: Vec<crate::core::types::ContextualExpression> = node.hash_keys().to_vec();
        let probe_keys: Vec<crate::core::types::ContextualExpression> = node.probe_keys().to_vec();

        let executor = InnerJoinExecutor::new(
            node.id(),
            storage,
            context.expression_context().clone(),
            hash_keys,
            probe_keys,
            left_var,
            right_var,
            node.col_names().to_vec(),
        );
        Ok(ExecutorEnum::InnerJoin(executor))
    }

    /// 构建 LeftJoin 执行器
    pub fn build_left_join(
        &self,
        node: &LeftJoinNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let (left_var, right_var) = Self::extract_join_vars(node);
        let hash_keys: Vec<crate::core::types::ContextualExpression> = node.hash_keys().to_vec();
        let probe_keys: Vec<crate::core::types::ContextualExpression> = node.probe_keys().to_vec();

        let executor = LeftJoinExecutor::new(
            node.id(),
            storage,
            context.expression_context().clone(),
            hash_keys,
            probe_keys,
            left_var,
            right_var,
            node.col_names().to_vec(),
        );
        Ok(ExecutorEnum::LeftJoin(executor))
    }

    /// 构建 HashLeftJoin 执行器
    pub fn build_hash_left_join(
        &self,
        node: &HashLeftJoinNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let (left_var, right_var) = Self::extract_join_vars(node);
        let hash_keys: Vec<crate::core::types::ContextualExpression> = node.hash_keys().to_vec();
        let probe_keys: Vec<crate::core::types::ContextualExpression> = node.probe_keys().to_vec();

        let executor = LeftJoinExecutor::new(
            node.id(),
            storage,
            context.expression_context().clone(),
            hash_keys,
            probe_keys,
            left_var,
            right_var,
            node.col_names().to_vec(),
        );
        Ok(ExecutorEnum::LeftJoin(executor))
    }

    /// 构建 FullOuterJoin 执行器
    pub fn build_full_outer_join(
        &self,
        node: &FullOuterJoinNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let (left_var, right_var) = Self::extract_join_vars(node);
        let hash_keys: Vec<crate::core::types::ContextualExpression> = node.hash_keys().to_vec();
        let probe_keys: Vec<crate::core::types::ContextualExpression> = node.probe_keys().to_vec();

        let executor = FullOuterJoinExecutor::new(
            node.id(),
            storage,
            context.expression_context().clone(),
            hash_keys,
            probe_keys,
            left_var,
            right_var,
            node.col_names().to_vec(),
        );
        Ok(ExecutorEnum::FullOuterJoin(executor))
    }

    /// 构建 CrossJoin 执行器
    pub fn build_cross_join(
        &self,
        node: &CrossJoinNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // CrossJoinExecutor 需要 Vec<String> 作为输入变量列表
        let left_var = node
            .left_input()
            .output_var()
            .map(|v| v.to_string())
            .unwrap_or_else(|| format!("left_{}", node.id()));
        let right_var = node
            .right_input()
            .output_var()
            .map(|v| v.to_string())
            .unwrap_or_else(|| format!("right_{}", node.id()));

        let input_vars = vec![left_var, right_var];

        let executor = CrossJoinExecutor::new(
            node.id(),
            storage,
            input_vars,
            node.col_names().to_vec(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::CrossJoin(executor))
    }
}

impl<S: StorageClient + 'static> Default for JoinBuilder<S> {
    fn default() -> Self {
        Self::new()
    }
}
