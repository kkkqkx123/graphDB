use std::sync::{Arc, Mutex};
use async_trait::async_trait;

use crate::core::{Value, DataSet};
use crate::graph::expression::{Expression, ExpressionContext};
use crate::query::executor::base::BaseExecutor;
use crate::query::executor::traits::{Executor, ExecutionResult, ExecutorCore, ExecutorLifecycle, ExecutorMetadata};
use crate::query::QueryError;
use crate::storage::StorageEngine;

/// 排序顺序枚举
#[derive(Debug, Clone, PartialEq)]
pub enum SortOrder {
    Asc,
    Desc,
}

/// 排序键定义
#[derive(Debug, Clone)]
pub struct SortKey {
    pub expression: Expression,
    pub order: SortOrder,
}

impl SortKey {
    pub fn new(expression: Expression, order: SortOrder) -> Self {
        Self { expression, order }
    }
}

/// 排序执行器
pub struct SortExecutor<S: StorageEngine + Send + 'static> {
    base: BaseExecutor<S>,
    /// 排序键列表
    sort_keys: Vec<SortKey>,
    /// 限制数量
    limit: Option<usize>,
    /// 输入执行器
    input_executor: Option<Box<dyn Executor<S>>>,
    /// 内存限制（字节）
    memory_limit: usize,
    /// 是否使用磁盘溢出
    use_disk: bool,
}

impl<S: StorageEngine + Send + 'static> SortExecutor<S> {
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
        sort_keys: Vec<SortKey>,
        limit: Option<usize>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "SortExecutor".to_string(), storage),
            sort_keys,
            limit,
            input_executor: None,
            memory_limit: 1024 * 1024 * 100, // 默认100MB
            use_disk: false,
        }
    }

    /// 带内存限制创建SortExecutor
    pub fn with_memory_limit(
        mut self,
        memory_limit: usize,
    ) -> Self {
        self.memory_limit = memory_limit;
        self
    }

    /// 启用磁盘溢出
    pub fn with_disk_spill(
        mut self,
        enable: bool,
    ) -> Self {
        self.use_disk = enable;
        self
    }

    /// 处理输入数据并排序
    async fn process_input(&mut self) -> Result<DataSet, QueryError> {
        if let Some(ref mut input_exec) = self.input_executor {
            let input_result = input_exec.execute().await?;
            
            match input_result {
                ExecutionResult::DataSet(mut data_set) => {
                    // 执行排序
                    self.sort_dataset(&mut data_set)?;
                    
                    // 应用限制
                    if let Some(limit) = self.limit {
                        data_set.rows.truncate(limit);
                    }
                    
                    Ok(data_set)
                }
                _ => Err(QueryError::ExecutionError("Sort executor expects DataSet input".to_string())),
            }
        } else {
            Err(QueryError::ExecutionError("Sort executor requires input executor".to_string()))
        }
    }

    /// 对数据集进行排序
    fn sort_dataset(&self, data_set: &mut DataSet) -> Result<(), QueryError> {
        // 如果没有排序键，直接返回
        if self.sort_keys.is_empty() {
            return Ok(());
        }

        // 为每行计算排序键值
        let mut rows_with_keys: Vec<(Vec<Value>, Vec<Value>)> = Vec::new();
        
        for row in &data_set.rows {
            // 构建表达式上下文
            let mut expr_context = ExpressionContext::new();
            for (i, col_name) in data_set.col_names.iter().enumerate() {
                if i < row.len() {
                    expr_context.set_variable(col_name.clone(), row[i].clone());
                }
            }

            // 计算排序键值
            let mut sort_values = Vec::new();
            for sort_key in &self.sort_keys {
                let sort_value = sort_key.expression.evaluate(&expr_context)
                    .map_err(|e| QueryError::ExpressionError(e.to_string()))?;
                sort_values.push(sort_value);
            }

            rows_with_keys.push((sort_values, row.clone()));
        }

        // 执行排序
        rows_with_keys.sort_by(|a, b| {
            // 逐个比较排序键
            for ((idx, sort_val_a), sort_val_b) in a.0.iter().enumerate().zip(b.0.iter()) {
                let comparison = self.compare_values(sort_val_a, sort_val_b, &self.sort_keys[idx].order);
                if !comparison.is_eq() {
                    return comparison;
                }
            }
            std::cmp::Ordering::Equal
        });

        // 提取排序后的行
        data_set.rows = rows_with_keys.into_iter().map(|(_, row)| row).collect();

        Ok(())
    }

    /// 比较两个值，根据排序方向
    fn compare_values(&self, a: &Value, b: &Value, order: &SortOrder) -> std::cmp::Ordering {
        let comparison = a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal);
        
        match order {
            SortOrder::Asc => comparison,
            SortOrder::Desc => comparison.reverse(),
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> ExecutorCore for SortExecutor<S> {
    async fn execute(&mut self) -> Result<ExecutionResult, QueryError> {
        let dataset = self.process_input().await?;
        Ok(ExecutionResult::DataSet(dataset))
    }
}

impl<S: StorageEngine> ExecutorLifecycle for SortExecutor<S> {
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

    fn is_open(&self) -> bool {
        self.base.is_open()
    }
}

impl<S: StorageEngine> ExecutorMetadata for SortExecutor<S> {
    fn id(&self) -> usize {
        self.base.id
    }

    fn name(&self) -> &str {
        self.base.name
    }

    fn description(&self) -> &str {
        &self.base.description
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for SortExecutor<S> {
    fn storage(&self) -> &S {
        &self.base.storage
    }
}

impl<S: StorageEngine + Send + 'static> crate::query::executor::base::InputExecutor<S> for SortExecutor<S> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>) {
        self.input_executor = Some(input);
    }

    fn get_input(&self) -> Option<&Box<dyn Executor<S>>> {
        self.input_executor.as_ref()
    }
}