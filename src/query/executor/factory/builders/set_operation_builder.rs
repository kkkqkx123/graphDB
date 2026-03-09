//! 集合操作执行器构建器
//!
//! 负责创建集合操作类型的执行器（Union, Minus, Intersect）

use crate::core::error::QueryError;
use crate::query::executor::base::ExecutionContext;
use crate::query::executor::data_processing::set_operations::{
    IntersectExecutor, MinusExecutor, UnionExecutor,
};
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::planner::plan::core::nodes::{IntersectNode, MinusNode, UnionNode};
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::sync::Arc;

/// 集合操作执行器构建器
pub struct SetOperationBuilder<S: StorageClient + 'static> {
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StorageClient + 'static> SetOperationBuilder<S> {
    /// 创建新的集合操作构建器
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    /// 构建 Union 执行器
    pub fn build_union(
        &self,
        node: &UnionNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // UnionNode 使用 output_var 获取输入变量名
        let input_var = node
            .output_var()
            .map(|v| v.to_string())
            .unwrap_or_else(|| format!("input_{}", node.id()));

        let executor = UnionExecutor::new(
            node.id(),
            storage,
            input_var,
            node.col_names().to_vec(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::Union(executor))
    }

    /// 构建 Minus 执行器
    pub fn build_minus(
        &self,
        node: &MinusNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // MinusNode 使用 output_var 获取输入变量名
        let input_var = node
            .output_var()
            .map(|v| v.to_string())
            .unwrap_or_else(|| format!("input_{}", node.id()));

        let executor = MinusExecutor::new(
            node.id(),
            storage,
            input_var,
            node.col_names().to_vec(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::Minus(executor))
    }

    /// 构建 Intersect 执行器
    pub fn build_intersect(
        &self,
        node: &IntersectNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // IntersectNode 使用 output_var 获取输入变量名
        let input_var = node
            .output_var()
            .map(|v| v.to_string())
            .unwrap_or_else(|| format!("input_{}", node.id()));

        let executor = IntersectExecutor::new(
            node.id(),
            storage,
            input_var,
            node.col_names().to_vec(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::Intersect(executor))
    }
}

impl<S: StorageClient + 'static> Default for SetOperationBuilder<S> {
    fn default() -> Self {
        Self::new()
    }
}
