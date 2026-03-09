//! SET OPERATION 语句执行器
//!
//! 处理 UNION, UNION ALL, INTERSECT, MINUS 等集合操作

use crate::core::error::{DBError, DBResult, QueryError};
use crate::core::Value;
use crate::query::executor::base::{ExecutionResult, Executor};
use crate::query::executor::factory::ExecutorFactory;
use crate::query::parser::ast::stmt::{SetOperationStmt, SetOperationType};
use crate::query::planner::planner::{Planner, ValidatedStatement};
use crate::query::planner::statements::set_operation_planner::SetOperationPlanner;
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::query::validator::ValidationInfo;
use crate::query::QueryContext;
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::sync::Arc;

/// SET OPERATION 执行器
///
/// 处理集合操作，包括：
/// - UNION: 合并两个数据集并去除重复行
/// - UNION ALL: 合并两个数据集不去重
/// - INTERSECT: 返回两个数据集的交集
/// - MINUS: 返回左数据集中不在右数据集中的行
pub struct SetOperationExecutor<S: StorageClient> {
    _id: i64,
    storage: Arc<Mutex<S>>,
}

impl<S: StorageClient> SetOperationExecutor<S> {
    /// 创建新的 SET OPERATION 执行器
    pub fn new(id: i64, storage: Arc<Mutex<S>>) -> Self {
        Self { _id: id, storage }
    }

    /// 执行 SET OPERATION 查询
    pub fn execute_set_operation(&self, clause: SetOperationStmt) -> DBResult<ExecutionResult>
    where
        S: Send + Sync + 'static,
    {
        let qctx = Arc::new(QueryContext::default());

        let validation_info = ValidationInfo::new();
        let ctx = Arc::new(ExpressionAnalysisContext::new());
        let ast = Arc::new(crate::query::parser::ast::Ast::new(
            crate::query::parser::ast::stmt::Stmt::SetOperation(clause),
            ctx,
        ));
        let validated = ValidatedStatement::new(ast, validation_info);

        let mut planner = SetOperationPlanner::new();
        let plan = planner
            .transform(&validated, qctx)
            .map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))?;

        let root_node = plan
            .root()
            .as_ref()
            .ok_or_else(|| DBError::Query(QueryError::ExecutionError("执行计划为空".to_string())))?
            .clone();

        let mut executor_factory = ExecutorFactory::with_storage(self.storage.clone());
        let mut executor = executor_factory
            .create_executor(&root_node, self.storage.clone(), &Default::default())
            .map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))?;

        executor
            .open()
            .map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))?;

        let result = executor
            .execute()
            .map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))?;

        executor
            .close()
            .map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))?;

        Ok(result)
    }

    /// 执行 UNION 操作
    ///
    /// 合并两个数据集并去除重复行
    pub fn execute_union(
        &self,
        left_result: ExecutionResult,
        right_result: ExecutionResult,
    ) -> DBResult<ExecutionResult> {
        self.execute_set_operation_internal(
            left_result,
            right_result,
            SetOperationType::Union,
        )
    }

    /// 执行 UNION ALL 操作
    ///
    /// 合并两个数据集不去重
    pub fn execute_union_all(
        &self,
        left_result: ExecutionResult,
        right_result: ExecutionResult,
    ) -> DBResult<ExecutionResult> {
        self.execute_set_operation_internal(
            left_result,
            right_result,
            SetOperationType::UnionAll,
        )
    }

    /// 执行 INTERSECT 操作
    ///
    /// 返回两个数据集的交集
    pub fn execute_intersect(
        &self,
        left_result: ExecutionResult,
        right_result: ExecutionResult,
    ) -> DBResult<ExecutionResult> {
        self.execute_set_operation_internal(
            left_result,
            right_result,
            SetOperationType::Intersect,
        )
    }

    /// 执行 MINUS 操作
    ///
    /// 返回左数据集中不在右数据集中的行
    pub fn execute_minus(
        &self,
        left_result: ExecutionResult,
        right_result: ExecutionResult,
    ) -> DBResult<ExecutionResult> {
        self.execute_set_operation_internal(
            left_result,
            right_result,
            SetOperationType::Minus,
        )
    }

    /// 内部执行集合操作
    fn execute_set_operation_internal(
        &self,
        left_result: ExecutionResult,
        right_result: ExecutionResult,
        op_type: SetOperationType,
    ) -> DBResult<ExecutionResult> {
        let left_rows = self.extract_rows(left_result)?;
        let right_rows = self.extract_rows(right_result)?;

        let result_rows = match op_type {
            SetOperationType::Union => self.union(&left_rows, &right_rows, true)?,
            SetOperationType::UnionAll => self.union(&left_rows, &right_rows, false)?,
            SetOperationType::Intersect => self.intersect(&left_rows, &right_rows)?,
            SetOperationType::Minus => self.minus(&left_rows, &right_rows)?,
        };

        Ok(ExecutionResult::Values(result_rows.into_iter().flatten().collect()))
    }

    /// 从 ExecutionResult 中提取行数据
    fn extract_rows(&self, result: ExecutionResult) -> DBResult<Vec<Vec<Value>>> {
        match result {
            ExecutionResult::Values(values) => {
                let rows = values
                    .chunks(1)
                    .map(|chunk| chunk.to_vec())
                    .collect();
                Ok(rows)
            }
            ExecutionResult::DataSet(dataset) => Ok(dataset.rows),
            ExecutionResult::Result(data_set) => Ok(data_set.rows().to_vec()),
            ExecutionResult::Empty => Ok(Vec::new()),
            _ => Err(DBError::Query(QueryError::ExecutionError(
                "不支持的结果类型".to_string(),
            ))),
        }
    }

    /// UNION 操作
    ///
    /// 合并两个数据集，可选是否去重
    fn union(
        &self,
        left_rows: &[Vec<Value>],
        right_rows: &[Vec<Value>],
        distinct: bool,
    ) -> DBResult<Vec<Vec<Value>>> {
        let mut combined = left_rows.to_vec();
        combined.extend_from_slice(right_rows);

        if distinct {
            combined.sort_by(|a, b| {
                a.iter()
                    .zip(b.iter())
                    .find_map(|(a_val, b_val)| {
                        if a_val != b_val {
                            Some(a_val.cmp(b_val))
                        } else {
                            None
                        }
                    })
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            combined.dedup();
        }

        Ok(combined)
    }

    /// INTERSECT 操作
    ///
    /// 返回两个数据集的交集
    fn intersect(
        &self,
        left_rows: &[Vec<Value>],
        right_rows: &[Vec<Value>],
    ) -> DBResult<Vec<Vec<Value>>> {
        let mut result = Vec::new();

        for row in left_rows {
            if right_rows.contains(row) {
                result.push(row.clone());
            }
        }

        Ok(result)
    }

    /// MINUS 操作
    ///
    /// 返回左数据集中不在右数据集中的行
    fn minus(
        &self,
        left_rows: &[Vec<Value>],
        right_rows: &[Vec<Value>],
    ) -> DBResult<Vec<Vec<Value>>> {
        let mut result = Vec::new();

        for row in left_rows {
            if !right_rows.contains(row) {
                result.push(row.clone());
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_union_distinct() {
        let left = vec![vec![Value::Int(1)], vec![Value::Int(2)]];
        let right = vec![vec![Value::Int(2)], vec![Value::Int(3)]];

        let storage = crate::storage::test_mock::MockStorage::new().unwrap();
        let executor = SetOperationExecutor::new(0, Arc::new(Mutex::new(storage)));
        let result = executor.union(&left, &right, true).unwrap();

        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_union_all() {
        let left = vec![vec![Value::Int(1)], vec![Value::Int(2)]];
        let right = vec![vec![Value::Int(2)], vec![Value::Int(3)]];

        let storage = crate::storage::test_mock::MockStorage::new().unwrap();
        let executor = SetOperationExecutor::new(0, Arc::new(Mutex::new(storage)));
        let result = executor.union(&left, &right, false).unwrap();

        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_intersect() {
        let left = vec![vec![Value::Int(1)], vec![Value::Int(2)], vec![Value::Int(3)]];
        let right = vec![vec![Value::Int(2)], vec![Value::Int(3)], vec![Value::Int(4)]];

        let storage = crate::storage::test_mock::MockStorage::new().unwrap();
        let executor = SetOperationExecutor::new(0, Arc::new(Mutex::new(storage)));
        let result = executor.intersect(&left, &right).unwrap();

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_minus() {
        let left = vec![vec![Value::Int(1)], vec![Value::Int(2)], vec![Value::Int(3)]];
        let right = vec![vec![Value::Int(2)], vec![Value::Int(4)]];

        let storage = crate::storage::test_mock::MockStorage::new().unwrap();
        let executor = SetOperationExecutor::new(0, Arc::new(Mutex::new(storage)));
        let result = executor.minus(&left, &right).unwrap();

        assert_eq!(result.len(), 2);
    }
}
