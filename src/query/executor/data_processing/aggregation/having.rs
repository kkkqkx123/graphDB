use std::sync::{Arc, Mutex};
use async_trait::async_trait;

use crate::core::{Value, DataSet};
use crate::graph::expression::{Expression, ExpressionContext};
use crate::query::executor::base::{Executor, ExecutionResult, BaseExecutor};
use crate::query::QueryError;
use crate::storage::StorageEngine;

/// Having执行器 - 对分组聚合后的结果进行过滤
pub struct HavingExecutor<S: StorageEngine + Send + 'static> {
    base: BaseExecutor<S>,
    /// Having条件表达式
    condition_expr: Expression,
    /// 输入执行器
    input_executor: Option<Box<dyn Executor<S>>>,
}

impl<S: StorageEngine + Send + 'static> HavingExecutor<S> {
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
        condition_expr: Expression,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "HavingExecutor".to_string(), storage),
            condition_expr,
            input_executor: None,
        }
    }

    /// 过滤数据集
    fn filter_data_set(&self, mut data_set: DataSet) -> Result<DataSet, QueryError> {
        let mut filtered_rows = Vec::new();
        
        for row in data_set.rows {
            // 构建表达式上下文
            let mut expr_context = ExpressionContext::new();
            for (i, col_name) in data_set.col_names.iter().enumerate() {
                if i < row.len() {
                    expr_context.set_variable(col_name.clone(), row[i].clone());
                }
            }

            // 评估Having条件
            let condition_result = self.condition_expr.evaluate(&expr_context)
                .map_err(|e| QueryError::ExpressionError(e.to_string()))?;

            // 只保留满足条件的行
            let is_truthy = match &condition_result {
                Value::Bool(b) => *b,
                Value::Int(i) => *i != 0,
                Value::Float(f) => *f != 0.0,
                Value::Null(_) => false,
                Value::Empty => false,
                Value::String(s) => !s.is_empty(),
                _ => true, // Consider other non-empty values as truthy
            };

            if is_truthy {
                filtered_rows.push(row);
            }
        }

        data_set.rows = filtered_rows;
        Ok(data_set)
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for HavingExecutor<S> {
    async fn execute(&mut self) -> Result<ExecutionResult, QueryError> {
        if let Some(ref mut input_exec) = self.input_executor {
            let input_result = input_exec.execute().await?;
            
            match input_result {
                ExecutionResult::DataSet(data_set) => {
                    let filtered_data_set = self.filter_data_set(data_set)?;
                    Ok(ExecutionResult::DataSet(filtered_data_set))
                }
                _ => Err(QueryError::ExecutionError("Having executor expects DataSet input".to_string())),
            }
        } else {
            Err(QueryError::ExecutionError("Having executor requires input executor".to_string()))
        }
    }

    fn open(&mut self) -> Result<(), QueryError> {
        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.open()?;
        }
        Ok(())
    }

    fn close(&mut self) -> Result<(), QueryError> {
        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.close()?;
        }
        Ok(())
    }

    fn id(&self) -> usize {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }
}

impl<S: StorageEngine + Send + 'static> crate::query::executor::base::InputExecutor<S> for HavingExecutor<S> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>) {
        self.input_executor = Some(input);
    }

    fn get_input(&self) -> Option<&Box<dyn Executor<S>>> {
        self.input_executor.as_ref()
    }
}