//! RollUpApplyExecutor实现
//! 
//! 负责处理聚合操作，将右输入中的值根据左输入的键进行聚合

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;

use crate::core::{Value, DataSet, List};
use crate::query::executor::base::{Executor, ExecutionResult, BaseExecutor};
use crate::query::QueryError;
use crate::storage::StorageEngine;
use crate::graph::expression::{Expression, ExpressionContext, InputPropertyExpression};

/// RollUpApply执行器
/// 用于将右输入中的值根据左输入的键进行聚合
pub struct RollUpApplyExecutor<S: StorageEngine + Send + 'static> {
    base: BaseExecutor<S>,
    /// 左输入变量名
    left_input_var: String,
    /// 右输入变量名
    right_input_var: String,
    /// 比较列表达式列表
    compare_cols: Vec<Expression>,
    /// 收集列表达式
    collect_col: InputPropertyExpression,
    /// 输出列名
    col_names: Vec<String>,
    /// 是否可以移动数据
    movable: bool,
}

impl<S: StorageEngine + Send + 'static> RollUpApplyExecutor<S> {
    /// 创建新的RollUpApplyExecutor
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
        left_input_var: String,
        right_input_var: String,
        compare_cols: Vec<Expression>,
        collect_col: InputPropertyExpression,
        col_names: Vec<String>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "RollUpApplyExecutor".to_string(), storage),
            left_input_var,
            right_input_var,
            compare_cols,
            collect_col,
            col_names,
            movable: false,
        }
    }

    /// 带上下文创建RollUpApplyExecutor
    pub fn with_context(
        id: usize,
        storage: Arc<Mutex<S>>,
        left_input_var: String,
        right_input_var: String,
        compare_cols: Vec<Expression>,
        collect_col: InputPropertyExpression,
        col_names: Vec<String>,
        context: crate::query::executor::base::ExecutionContext,
    ) -> Self {
        Self {
            base: BaseExecutor::with_context(id, "RollUpApplyExecutor".to_string(), storage, context),
            left_input_var,
            right_input_var,
            compare_cols,
            collect_col,
            col_names,
            movable: false,
        }
    }

    /// 检查双输入数据集
    fn check_bi_input_data_sets(&self) -> Result<(), QueryError> {
        // 检查左输入
        let _left_result = self.base.context.get_result(&self.left_input_var)
            .ok_or_else(|| QueryError::ExecutionError(format!("Left input variable '{}' not found", self.left_input_var)))?;

        // 检查右输入
        let _right_result = self.base.context.get_result(&self.right_input_var)
            .ok_or_else(|| QueryError::ExecutionError(format!("Right input variable '{}' not found", self.right_input_var)))?;

        Ok(())
    }

    /// 构建哈希表（多键）
    fn build_hash_table(
        &self,
        compare_cols: &[Expression],
        collect_col: &InputPropertyExpression,
        iter: &[Value],
        hash_table: &mut HashMap<List, List>,
        expr_context: &mut ExpressionContext,
    ) -> Result<(), QueryError> {
        for value in iter {
            // 设置当前值到表达式上下文
            expr_context.set_variable("_".to_string(), value.clone());
            
            // 构建键列表
            let mut key_list = List { values: Vec::new() };
            for col in compare_cols {
                let val = col.evaluate(expr_context)
                    .map_err(|e| QueryError::ExpressionError(e.to_string()))?;
                key_list.values.push(val);
            }

            // 获取收集列的值
            let collect_val = collect_col.evaluate(expr_context)
                .map_err(|e| QueryError::ExpressionError(e.to_string()))?;

            // 添加到哈希表
            let entry = hash_table.entry(key_list).or_insert_with(|| List { values: Vec::new() });
            entry.values.push(collect_val);
        }

        Ok(())
    }

    /// 构建单键哈希表
    fn build_single_key_hash_table(
        &self,
        compare_col: &Expression,
        collect_col: &InputPropertyExpression,
        iter: &[Value],
        hash_table: &mut HashMap<Value, List>,
        expr_context: &mut ExpressionContext,
    ) -> Result<(), QueryError> {
        for value in iter {
            // 设置当前值到表达式上下文
            expr_context.set_variable("_".to_string(), value.clone());
            
            // 获取键值
            let key_val = compare_col.evaluate(expr_context)
                .map_err(|e| QueryError::ExpressionError(e.to_string()))?;

            // 获取收集列的值
            let collect_val = collect_col.evaluate(expr_context)
                .map_err(|e| QueryError::ExpressionError(e.to_string()))?;

            // 添加到哈希表
            let entry = hash_table.entry(key_val).or_insert_with(|| List { values: Vec::new() });
            entry.values.push(collect_val);
        }

        Ok(())
    }

    /// 构建零键哈希表
    fn build_zero_key_hash_table(
        &self,
        collect_col: &InputPropertyExpression,
        iter: &[Value],
        hash_table: &mut List,
        expr_context: &mut ExpressionContext,
    ) -> Result<(), QueryError> {
        hash_table.values.reserve(iter.len());
        
        for value in iter {
            // 设置当前值到表达式上下文
            expr_context.set_variable("_".to_string(), value.clone());
            
            // 获取收集列的值
            let collect_val = collect_col.evaluate(expr_context)
                .map_err(|e| QueryError::ExpressionError(e.to_string()))?;
            
            hash_table.values.push(collect_val);
        }

        Ok(())
    }

    /// 探测零键哈希表
    fn probe_zero_key(
        &self,
        probe_iter: &[Value],
        hash_table: &List,
        expr_context: &mut ExpressionContext,
    ) -> Result<DataSet, QueryError> {
        let mut dataset = DataSet {
            col_names: self.col_names.clone(),
            rows: Vec::new(),
        };

        dataset.rows.reserve(probe_iter.len());

        for value in probe_iter {
            // 设置当前值到表达式上下文
            expr_context.set_variable("_".to_string(), value.clone());
            
            let mut row = Vec::new();
            
            if self.movable {
                row.push(value.clone());
            } else {
                row.push(value.clone());
            }
            
            row.push(Value::List(hash_table.values.clone()));
            dataset.rows.push(row);
        }

        Ok(dataset)
    }

    /// 探测单键哈希表
    fn probe_single_key(
        &self,
        probe_key: &Expression,
        probe_iter: &[Value],
        hash_table: &HashMap<Value, List>,
        expr_context: &mut ExpressionContext,
    ) -> Result<DataSet, QueryError> {
        let mut dataset = DataSet {
            col_names: self.col_names.clone(),
            rows: Vec::new(),
        };

        dataset.rows.reserve(probe_iter.len());

        for value in probe_iter {
            // 设置当前值到表达式上下文
            expr_context.set_variable("_".to_string(), value.clone());
            
            // 获取探测键值
            let key_val = probe_key.evaluate(expr_context)
                .map_err(|e| QueryError::ExpressionError(e.to_string()))?;

            // 查找哈希表
            let vals = hash_table.get(&key_val).cloned().unwrap_or(List { values: Vec::new() });

            let mut row = Vec::new();
            
            if self.movable {
                row.push(value.clone());
            } else {
                row.push(value.clone());
            }
            
            row.push(Value::List(vals.values));
            dataset.rows.push(row);
        }

        Ok(dataset)
    }

    /// 探测多键哈希表
    fn probe(
        &self,
        probe_keys: &[Expression],
        probe_iter: &[Value],
        hash_table: &HashMap<List, List>,
        expr_context: &mut ExpressionContext,
    ) -> Result<DataSet, QueryError> {
        let mut dataset = DataSet {
            col_names: self.col_names.clone(),
            rows: Vec::new(),
        };

        dataset.rows.reserve(probe_iter.len());

        for value in probe_iter {
            // 设置当前值到表达式上下文
            expr_context.set_variable("_".to_string(), value.clone());
            
            // 构建键列表
            let mut key_list = List { values: Vec::new() };
            for col in probe_keys {
                let val = col.evaluate(expr_context)
                    .map_err(|e| QueryError::ExpressionError(e.to_string()))?;
                key_list.values.push(val);
            }

            // 查找哈希表
            let vals = hash_table.get(&key_list).cloned().unwrap_or(List { values: Vec::new() });

            let mut row = Vec::new();
            
            if self.movable {
                row.push(value.clone());
            } else {
                row.push(value.clone());
            }
            
            row.push(Value::List(vals.values));
            dataset.rows.push(row);
        }

        Ok(dataset)
    }

    /// 执行RollUpApply操作
    fn execute_rollup_apply(&mut self) -> Result<DataSet, QueryError> {
        // 检查输入数据集
        self.check_bi_input_data_sets()?;

        // 获取输入结果
        let left_result = self.base.context.get_result(&self.left_input_var).unwrap();
        let right_result = self.base.context.get_result(&self.right_input_var).unwrap();

        // 将结果转换为值列表
        let left_values = match left_result {
            ExecutionResult::Values(values) => values.clone(),
            ExecutionResult::Vertices(vertices) => {
                vertices.iter().map(|v| Value::Vertex(Box::new(v.clone()))).collect::<Vec<_>>()
            },
            ExecutionResult::Edges(edges) => {
                edges.iter().map(|e| Value::Edge(e.clone())).collect::<Vec<_>>()
            },
            _ => return Err(QueryError::ExecutionError("Invalid left input result type".to_string())),
        };

        let right_values = match right_result {
            ExecutionResult::Values(values) => values.clone(),
            ExecutionResult::Vertices(vertices) => {
                vertices.iter().map(|v| Value::Vertex(Box::new(v.clone()))).collect::<Vec<_>>()
            },
            ExecutionResult::Edges(edges) => {
                edges.iter().map(|e| Value::Edge(e.clone())).collect::<Vec<_>>()
            },
            _ => return Err(QueryError::ExecutionError("Invalid right input result type".to_string())),
        };

        // 创建表达式上下文
        let mut expr_context = ExpressionContext::new();
        
        // 从执行上下文中设置变量
        for (name, value) in &self.base.context.variables.clone() {
            expr_context.set_variable(name.clone(), value.clone());
        }

        // 根据比较列数量选择不同的处理方式
        let result = if self.compare_cols.is_empty() {
            // 零键情况
            let mut hash_table = List { values: Vec::new() };
            self.build_zero_key_hash_table(&self.collect_col, &right_values, &mut hash_table, &mut expr_context)?;
            self.probe_zero_key(&left_values, &hash_table, &mut expr_context)?
        } else if self.compare_cols.len() == 1 {
            // 单键情况
            let mut hash_table = HashMap::new();
            self.build_single_key_hash_table(
                &self.compare_cols[0],
                &self.collect_col,
                &right_values,
                &mut hash_table,
                &mut expr_context,
            )?;
            self.probe_single_key(&self.compare_cols[0], &left_values, &hash_table, &mut expr_context)?
        } else {
            // 多键情况
            let mut hash_table = HashMap::new();
            self.build_hash_table(
                &self.compare_cols,
                &self.collect_col,
                &right_values,
                &mut hash_table,
                &mut expr_context,
            )?;
            self.probe(&self.compare_cols, &left_values, &hash_table, &mut expr_context)?
        };

        Ok(result)
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for RollUpApplyExecutor<S> {
    async fn execute(&mut self) -> Result<ExecutionResult, QueryError> {
        // 执行RollUpApply操作
        let dataset = self.execute_rollup_apply()?;
        
        // 将数据集转换为值列表
        let values: Vec<Value> = dataset.rows.into_iter()
            .flat_map(|row| row.into_iter())
            .collect();
        
        Ok(ExecutionResult::Values(values))
    }

    fn open(&mut self) -> Result<(), QueryError> {
        // 初始化资源
        Ok(())
    }

    fn close(&mut self) -> Result<(), QueryError> {
        // 清理资源
        Ok(())
    }

    fn id(&self) -> usize {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;
    use crate::graph::expression::Expression;
    use crate::storage::NativeStorage;
    use crate::config::test_config::test_config;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn test_rollup_apply_executor() {
        let config = test_config();
        let storage = Arc::new(Mutex::new(NativeStorage::new(config.test_db_path("test_db_rollup_apply")).unwrap()));
        
        // 创建左输入数据
        let left_values = vec![
            Value::Int(1),
            Value::Int(2),
        ];
        
        // 创建右输入数据
        let right_values = vec![
            Value::Int(1),
            Value::Int(1),
            Value::Int(2),
        ];
        
        // 创建执行上下文
        let mut context = crate::query::executor::base::ExecutionContext::new();
        context.set_result("left".to_string(), ExecutionResult::Values(left_values.clone()));
        context.set_result("right".to_string(), ExecutionResult::Values(right_values.clone()));
        
        // 创建RollUpApplyExecutor
        let compare_cols = vec![
            Expression::Constant(Value::Int(0)), // 简化的比较列
        ];
        let collect_col = InputPropertyExpression::new("_".to_string());
        
        let mut executor = RollUpApplyExecutor::with_context(
            1,
            storage,
            "left".to_string(),
            "right".to_string(),
            compare_cols,
            collect_col,
            vec!["key".to_string(), "collected".to_string()],
            context,
        );
        
        // 执行RollUpApply
        let result = executor.execute().await.unwrap();
        
        // 检查结果
        if let ExecutionResult::Values(values) = result {
            // 应该有4个值（2个左值 × 2个聚合组）
            assert_eq!(values.len(), 4);
        } else {
            panic!("Expected Values result");
        }
    }
}