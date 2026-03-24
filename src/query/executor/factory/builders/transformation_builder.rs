//! 数据转换执行器构建器
//!
//! 负责创建数据转换类型的执行器（Unwind, Assign, Materialize, AppendVertices, RollUpApply, PatternApply）

use crate::core::error::QueryError;
use crate::query::executor::base::ExecutionContext;
use crate::query::executor::data_processing::MaterializeExecutor;
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::executor::result_processing::{
    AppendVerticesExecutor, AssignExecutor, PatternApplyExecutor, RollUpApplyExecutor,
    UnwindExecutor,
};
use crate::query::planner::plan::core::nodes::{
    AppendVerticesNode, AssignNode, MaterializeNode, PatternApplyNode, RollUpApplyNode, UnwindNode,
};
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::sync::Arc;

/// 数据转换执行器构建器
pub struct TransformationBuilder<S: StorageClient + Send + 'static> {
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StorageClient + Send + 'static> TransformationBuilder<S> {
    /// 创建新的数据转换构建器
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    /// 构建 Unwind 执行器
    pub fn build_unwind(
        &self,
        node: &UnwindNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let unwind_expression = node
            .list_expression()
            .expression()
            .map(|meta| meta.inner().clone())
            .ok_or_else(|| QueryError::ExecutionError("表达式不存在于上下文中".to_string()))?;

        let executor = UnwindExecutor::new(
            node.id(),
            storage,
            node.alias().to_string(),
            unwind_expression,
            node.col_names().to_vec(),
            false,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::Unwind(executor))
    }

    /// 构建 Assign 执行器
    pub fn build_assign(
        &self,
        node: &AssignNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let mut parsed_assignments = Vec::new();
        for (var_name, ctx_expr) in node.assignments() {
            let expression = ctx_expr
                .expression()
                .map(|meta| meta.inner().clone())
                .ok_or_else(|| QueryError::ExecutionError("表达式不存在于上下文中".to_string()))?;
            parsed_assignments.push((var_name.clone(), expression));
        }

        let executor = AssignExecutor::new(
            node.id(),
            storage,
            parsed_assignments,
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::Assign(executor))
    }

    /// 构建 Materialize 执行器
    pub fn build_materialize(
        &self,
        node: &MaterializeNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // 创建物化执行器，使用默认内存限制（100MB）
        let executor = MaterializeExecutor::new(
            node.id(),
            storage,
            None, // 使用默认内存限制
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::Materialize(executor))
    }

    /// 构建 AppendVertices 执行器
    pub fn build_append_vertices(
        &self,
        node: &AppendVerticesNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let input_var = node
            .input_var()
            .map(|v| v.to_string())
            .unwrap_or_else(|| format!("input_{}", node.id()));

        let src_expression = node
            .src_expression()
            .and_then(|ctx_expr| ctx_expr.expression())
            .map(|meta| meta.inner().clone())
            .unwrap_or_else(|| crate::core::Expression::Variable("_".to_string()));

        let executor = AppendVerticesExecutor::new(
            node.id(),
            storage,
            input_var,
            src_expression,
            None,
            node.col_names().to_vec(),
            node.dedup(),
            node.need_fetch_prop(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::AppendVertices(executor))
    }

    /// 构建 RollUpApply 执行器
    pub fn build_rollup_apply(
        &self,
        node: &RollUpApplyNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let left_input_var = node
            .left_input_var()
            .cloned()
            .unwrap_or_else(|| format!("left_{}", node.id()));
        let right_input_var = node
            .right_input_var()
            .cloned()
            .unwrap_or_else(|| format!("right_{}", node.id()));

        let compare_cols: Vec<crate::core::Expression> = node
            .compare_cols()
            .iter()
            .map(|col| crate::core::Expression::Variable(col.clone()))
            .collect();

        let collect_col = node
            .collect_col()
            .map(|col| crate::core::Expression::Variable(col.clone()))
            .unwrap_or_else(|| crate::core::Expression::Variable("_".to_string()));

        let executor = RollUpApplyExecutor::new(
            node.id(),
            storage,
            left_input_var,
            right_input_var,
            compare_cols,
            collect_col,
            node.col_names().to_vec(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::RollUpApply(executor))
    }

    /// 构建 PatternApply 执行器
    pub fn build_pattern_apply(
        &self,
        node: &PatternApplyNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let left_input_var = node
            .left_input_var()
            .cloned()
            .unwrap_or_else(|| format!("left_{}", node.id()));
        let right_input_var = node
            .right_input_var()
            .cloned()
            .unwrap_or_else(|| format!("right_{}", node.id()));

        let key_cols: Vec<crate::core::Expression> = node
            .key_cols()
            .iter()
            .filter_map(|ctx_expr| ctx_expr.get_expression())
            .collect();

        let executor = PatternApplyExecutor::new(
            node.id(),
            storage,
            left_input_var,
            right_input_var,
            key_cols,
            node.col_names().to_vec(),
            node.is_anti_predicate(),
            context.expression_context().clone(),
        );
        Ok(ExecutorEnum::PatternApply(executor))
    }
}

impl<S: StorageClient + 'static> Default for TransformationBuilder<S> {
    fn default() -> Self {
        Self::new()
    }
}
