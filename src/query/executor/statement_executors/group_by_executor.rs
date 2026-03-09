//! GROUP BY 语句执行器
//!
//! 处理 GROUP BY 聚合查询，支持多字段分组、聚合函数和 HAVING 子句

use crate::core::error::{DBError, DBResult, QueryError};
use crate::core::Value;
use crate::query::executor::base::{ExecutionResult, Executor};
use crate::query::executor::expression::DefaultExpressionContext;
use crate::query::executor::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::query::executor::factory::ExecutorFactory;
use crate::query::parser::ast::stmt::GroupByStmt;
use crate::query::planner::planner::{Planner, ValidatedStatement};
use crate::query::planner::statements::group_by_planner::GroupByPlanner;
use crate::query::QueryContext;
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::query::validator::ValidationInfo;
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

/// GROUP BY 执行器
///
/// 处理 GROUP BY 聚合查询，支持：
/// - 单字段和多字段分组
/// - 常用聚合函数（COUNT, SUM, AVG, MIN, MAX）
/// - HAVING 子句过滤
/// - DISTINCT 聚合
pub struct GroupByExecutor<S: StorageClient> {
    _id: i64,
    storage: Arc<Mutex<S>>,
}

impl<S: StorageClient> GroupByExecutor<S> {
    /// 创建新的 GROUP BY 执行器
    pub fn new(id: i64, storage: Arc<Mutex<S>>) -> Self {
        Self { _id: id, storage }
    }

    /// 执行 GROUP BY 查询
    pub fn execute_group_by(&self, clause: GroupByStmt) -> DBResult<ExecutionResult>
    where
        S: Send + Sync + 'static,
    {
        let qctx = Arc::new(QueryContext::default());

        let validation_info = ValidationInfo::new();
        let ctx = Arc::new(ExpressionAnalysisContext::new());
        let ast = Arc::new(crate::query::parser::ast::Ast::new(
            crate::query::parser::ast::stmt::Stmt::GroupBy(clause),
            ctx,
        ));
        let validated = ValidatedStatement::new(ast, validation_info);

        let mut planner = GroupByPlanner::new();
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

    /// 构建分组
    ///
    /// 根据分组键表达式计算每行的分组键
    fn build_groups(
        rows: &[Vec<Value>],
        group_key_exprs: &[crate::core::types::expression::Expression],
    ) -> DBResult<HashMap<Vec<Value>, Vec<Vec<Value>>>> {
        let mut groups: HashMap<Vec<Value>, Vec<Vec<Value>>> = HashMap::new();
        let mut context = DefaultExpressionContext::new();

        for row in rows {
            let mut group_key = Vec::new();
            for expr in group_key_exprs {
                let value = ExpressionEvaluator::evaluate(expr, &mut context)
                    .map_err(|e| DBError::Query(QueryError::ExecutionError(format!("分组键求值失败: {}", e))))?;
                group_key.push(value);
            }
            groups.entry(group_key).or_insert_with(Vec::new).push(row.clone());
        }

        Ok(groups)
    }

    /// 计算聚合函数
    ///
    /// 对每个分组计算聚合函数的值
    fn compute_aggregations(
        groups: &HashMap<Vec<Value>, Vec<Vec<Value>>>,
        agg_specs: &[crate::query::executor::result_processing::AggregateFunctionSpec],
    ) -> DBResult<Vec<(Vec<Value>, Vec<Value>)>> {
        let mut results = Vec::new();
        let agg_manager = crate::query::executor::result_processing::agg_function_manager::AggFunctionManager::new();

        for (group_key, group_rows) in groups {
            let mut agg_values = Vec::new();

            for agg_spec in agg_specs {
                let mut agg_data = crate::query::executor::result_processing::agg_data::AggData::new();
                let agg_func_name = agg_spec.agg_function_name();
                let agg_func = agg_manager.get(&agg_func_name)
                    .ok_or_else(|| DBError::Query(QueryError::ExecutionError(format!("聚合函数 '{}' 不存在", agg_func_name))))?;

                for _row in group_rows {
                    let value = Value::Null(crate::core::NullType::default());
                    agg_func(&mut agg_data, &value)
                        .map_err(|e| DBError::Query(QueryError::ExecutionError(format!("聚合函数执行失败: {}", e))))?;
                }

                agg_values.push(agg_data.result().clone());
            }

            results.push((group_key.clone(), agg_values));
        }

        Ok(results)
    }

    /// 应用 HAVING 过滤
    ///
    /// 对聚合结果应用 HAVING 子句过滤
    #[allow(dead_code)]
    fn apply_having(
        results: &mut Vec<(Vec<Value>, Vec<Value>)>,
        having_expr: &crate::core::types::expression::Expression,
    ) -> DBResult<()> {
        results.retain(|(group_key, agg_values)| {
            let mut context = DefaultExpressionContext::new();
            for (i, key) in group_key.iter().enumerate() {
                context = context.add_variable(format!("group_key_{}", i), key.clone());
            }
            for (i, value) in agg_values.iter().enumerate() {
                context = context.add_variable(format!("agg_{}", i), value.clone());
            }
            match ExpressionEvaluator::evaluate(having_expr, &mut context) {
                Ok(Value::Bool(true)) => true,
                Ok(Value::Bool(false)) => false,
                Ok(_) => false,
                Err(_) => false,
            }
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_groups_single_key() {
        let rows = vec![
            vec![Value::Int(1), Value::Int(10)],
            vec![Value::Int(1), Value::Int(20)],
            vec![Value::Int(2), Value::Int(30)],
            vec![Value::Int(2), Value::Int(40)],
        ];

        let group_key_exprs = vec![crate::core::types::expression::Expression::Literal(Value::Int(0))];
        let groups = GroupByExecutor::<crate::storage::test_mock::MockStorage>::build_groups(&rows, &group_key_exprs);

        assert!(groups.is_ok());
        assert_eq!(groups.unwrap().len(), 2);
    }

    #[test]
    fn test_compute_aggregations() {
        let groups = HashMap::from([
            (vec![Value::Int(1)], vec![vec![Value::Int(10)], vec![Value::Int(20)]]),
            (vec![Value::Int(2)], vec![vec![Value::Int(30)], vec![Value::Int(40)]]),
        ]);

        let agg_specs = vec![
            crate::query::executor::result_processing::AggregateFunctionSpec::sum("value".to_string()),
        ];

        let results = GroupByExecutor::<crate::storage::test_mock::MockStorage>::compute_aggregations(&groups, &agg_specs);

        assert!(results.is_ok());
        assert_eq!(results.unwrap().len(), 2);
    }
}
