//! 聚合操作执行器模块
//!
//! 包含聚合操作相关的执行器，包括：
//! - GroupBy（分组聚合）
//! - Aggregate（整体聚合）
//! - Having（分组后过滤）

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::core::Value;
use crate::query::executor::base::{BaseExecutor, InputExecutor};
use crate::query::executor::traits::{Executor, ExecutionResult, ExecutorCore, ExecutorLifecycle, ExecutorMetadata, DBResult};
use crate::query::QueryError;
use crate::storage::StorageEngine;

/// 聚合函数类型
#[derive(Debug, Clone)]
pub enum AggregateFunction {
    Count,
    Sum(String), // 字段名
    Avg(String), // 字段名
    Max(String), // 字段名
    Min(String), // 字段名
}

/// 聚合数据状态
#[derive(Debug, Clone)]
struct AggData {
    count: usize,
    sum: Option<Value>,
    max: Option<Value>,
    min: Option<Value>,
}

impl AggData {
    fn new() -> Self {
        Self {
            count: 0,
            sum: None,
            max: None,
            min: None,
        }
    }

    /// 应用聚合函数到新值
    fn apply(&mut self, func: &AggregateFunction, value: &Value) -> Result<(), QueryError> {
        match func {
            AggregateFunction::Count => {
                self.count += 1;
            }
            AggregateFunction::Sum(_) => {
                self.apply_sum(value)?;
            }
            AggregateFunction::Avg(_) => {
                self.apply_sum(value)?;
                self.count += 1;
            }
            AggregateFunction::Max(_) => {
                self.apply_max(value)?;
            }
            AggregateFunction::Min(_) => {
                self.apply_min(value)?;
            }
        }
        Ok(())
    }

    /// 应用求和操作
    fn apply_sum(&mut self, value: &Value) -> Result<(), QueryError> {
        if value.is_null() {
            return Ok(());
        }

        match value {
            Value::Int(i) => {
                if let Some(Value::Int(current)) = &self.sum {
                    self.sum = Some(Value::Int(current + i));
                } else {
                    self.sum = Some(Value::Int(*i));
                }
            }
            Value::Float(f) => {
                if let Some(Value::Float(current)) = &self.sum {
                    self.sum = Some(Value::Float(current + f));
                } else {
                    self.sum = Some(Value::Float(*f));
                }
            }
            _ => {
                return Err(QueryError::ExecutionError(format!(
                    "Cannot sum non-numeric value: {}",
                    value
                )));
            }
        }
        Ok(())
    }

    /// 应用最大值操作
    fn apply_max(&mut self, value: &Value) -> Result<(), QueryError> {
        if value.is_null() {
            return Ok(());
        }

        match value {
            Value::Int(i) => {
                if let Some(Value::Int(current)) = &self.max {
                    if i > current {
                        self.max = Some(Value::Int(*i));
                    }
                } else {
                    self.max = Some(Value::Int(*i));
                }
            }
            Value::Float(f) => {
                if let Some(Value::Float(current)) = &self.max {
                    if f > current {
                        self.max = Some(Value::Float(*f));
                    }
                } else {
                    self.max = Some(Value::Float(*f));
                }
            }
            Value::String(s) => {
                if let Some(Value::String(current)) = &self.max {
                    if s > current {
                        self.max = Some(Value::String(s.clone()));
                    }
                } else {
                    self.max = Some(Value::String(s.clone()));
                }
            }
            _ => {
                return Err(QueryError::ExecutionError(format!(
                    "Cannot compare value for max: {}",
                    value
                )));
            }
        }
        Ok(())
    }

    /// 应用最小值操作
    fn apply_min(&mut self, value: &Value) -> Result<(), QueryError> {
        if value.is_null() {
            return Ok(());
        }

        match value {
            Value::Int(i) => {
                if let Some(Value::Int(current)) = &self.min {
                    if i < current {
                        self.min = Some(Value::Int(*i));
                    }
                } else {
                    self.min = Some(Value::Int(*i));
                }
            }
            Value::Float(f) => {
                if let Some(Value::Float(current)) = &self.min {
                    if f < current {
                        self.min = Some(Value::Float(*f));
                    }
                } else {
                    self.min = Some(Value::Float(*f));
                }
            }
            Value::String(s) => {
                if let Some(Value::String(current)) = &self.min {
                    if s < current {
                        self.min = Some(Value::String(s.clone()));
                    }
                } else {
                    self.min = Some(Value::String(s.clone()));
                }
            }
            _ => {
                return Err(QueryError::ExecutionError(format!(
                    "Cannot compare value for min: {}",
                    value
                )));
            }
        }
        Ok(())
    }

    /// 获取聚合结果
    fn result(&self, func: &AggregateFunction) -> Result<Value, QueryError> {
        match func {
            AggregateFunction::Count => Ok(Value::Int(self.count as i64)),
            AggregateFunction::Sum(_) => {
                if let Some(sum) = &self.sum {
                    Ok(sum.clone())
                } else {
                    Ok(Value::Int(0))
                }
            }
            AggregateFunction::Avg(_) => {
                if self.count == 0 {
                    Ok(Value::Float(0.0))
                } else if let Some(Value::Int(sum)) = &self.sum {
                    Ok(Value::Float(*sum as f64 / self.count as f64))
                } else if let Some(Value::Float(sum)) = &self.sum {
                    Ok(Value::Float(*sum / self.count as f64))
                } else {
                    Ok(Value::Float(0.0))
                }
            }
            AggregateFunction::Max(_) => {
                if let Some(max) = &self.max {
                    Ok(max.clone())
                } else {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
            }
            AggregateFunction::Min(_) => {
                if let Some(min) = &self.min {
                    Ok(min.clone())
                } else {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
            }
        }
    }
}

/// AggregateExecutor - 聚合执行器
///
/// 执行聚合操作，支持 COUNT, SUM, AVG, MAX, MIN 等聚合函数
pub struct AggregateExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    group_keys: Vec<String>,                     // 分组键
    aggregate_functions: Vec<AggregateFunction>, // 聚合函数
    input_executor: Option<Box<dyn Executor<S>>>,
}

impl<S: StorageEngine> AggregateExecutor<S> {
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
        group_keys: Vec<String>,
        aggregate_functions: Vec<AggregateFunction>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "AggregateExecutor".to_string(), storage),
            group_keys,
            aggregate_functions,
            input_executor: None,
        }
    }

    /// 从行中提取分组键值
    fn extract_group_key(&self, row: &[Value]) -> Vec<Value> {
        self.group_keys
            .iter()
            .map(|key| {
                // 在实际实现中，这里需要根据列名找到对应的值
                // 现在简化实现，假设行顺序与列名顺序一致
                if let Some(index) = self.group_keys.iter().position(|k| k == key) {
                    if index < row.len() {
                        row[index].clone()
                    } else {
                        Value::Null(crate::core::value::NullType::Null)
                    }
                } else {
                    Value::Null(crate::core::value::NullType::Null)
                }
            })
            .collect()
    }

    /// 从行中提取聚合函数需要的值
    fn extract_aggregate_value(&self, row: &[Value], func: &AggregateFunction) -> Value {
        match func {
            AggregateFunction::Count => Value::Int(1), // COUNT 总是返回 1
            AggregateFunction::Sum(column)
            | AggregateFunction::Avg(column)
            | AggregateFunction::Max(column)
            | AggregateFunction::Min(column) => {
                // 在实际实现中，这里需要根据列名找到对应的值
                // 现在简化实现，假设列名存在
                if let Some(index) = self.group_keys.iter().position(|k| k == column) {
                    if index < row.len() {
                        row[index].clone()
                    } else {
                        Value::Null(crate::core::value::NullType::Null)
                    }
                } else {
                    Value::Null(crate::core::value::NullType::Null)
                }
            }
        }
    }
}

impl<S: StorageEngine> InputExecutor<S> for AggregateExecutor<S> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>) {
        self.input_executor = Some(input);
    }

    fn get_input(&self) -> Option<&Box<dyn Executor<S>>> {
        self.input_executor.as_ref()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> ExecutorCore for AggregateExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        // 首先执行输入执行器（如果存在）
        let _input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute().await?
        } else {
            // 如果没有输入执行器，返回空结果
            ExecutionResult::Values(Vec::new())
        };

        // 处理结果
        // 简化实现，暂时返回空结果
        Ok(ExecutionResult::Values(Vec::new()))
    }
}

impl<S: StorageEngine + Send + 'static> ExecutorLifecycle for AggregateExecutor<S> {
    fn open(&mut self) -> DBResult<()> {
        // 初始化聚合所需的任何资源
        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.open()?;
        }
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        // 清理资源
        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.close()?;
        }
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }
}

impl<S: StorageEngine + Send + 'static> ExecutorMetadata for AggregateExecutor<S> {
    fn id(&self) -> usize {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        "AggregateExecutor - performs aggregation operations"
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for AggregateExecutor<S> {
    fn storage(&self) -> &Arc<Mutex<S>> {
        &self.base.storage
    }
}

/// GroupByExecutor - 分组聚合执行器
///
/// 实现 GROUP BY 操作
pub struct GroupByExecutor<S: StorageEngine> {
    aggregate_executor: AggregateExecutor<S>,
}

impl<S: StorageEngine> GroupByExecutor<S> {
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
        group_keys: Vec<String>,
        aggregate_functions: Vec<AggregateFunction>,
    ) -> Self {
        Self {
            aggregate_executor: AggregateExecutor::new(id, storage, group_keys, aggregate_functions),
        }
    }
}

impl<S: StorageEngine> InputExecutor<S> for GroupByExecutor<S> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>) {
        self.aggregate_executor.set_input(input);
    }

    fn get_input(&self) -> Option<&Box<dyn Executor<S>>> {
        self.aggregate_executor.get_input()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> ExecutorCore for GroupByExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        self.aggregate_executor.execute().await
    }
}

impl<S: StorageEngine + Send + 'static> ExecutorLifecycle for GroupByExecutor<S> {
    fn open(&mut self) -> DBResult<()> {
        self.aggregate_executor.open()
    }

    fn close(&mut self) -> DBResult<()> {
        self.aggregate_executor.close()
    }

    fn is_open(&self) -> bool {
        true
    }
}

impl<S: StorageEngine + Send + 'static> ExecutorMetadata for GroupByExecutor<S> {
    fn id(&self) -> usize {
        self.aggregate_executor.id()
    }

    fn name(&self) -> &str {
        "GroupByExecutor"
    }

    fn description(&self) -> &str {
        "GroupByExecutor - performs GROUP BY operations"
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for GroupByExecutor<S> {
    fn storage(&self) -> &Arc<Mutex<S>> {
        &self.aggregate_executor.storage()
    }
}

/// HavingExecutor - HAVING 子句执行器
///
/// 实现 HAVING 子句，对分组后的结果进行过滤
pub struct HavingExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    input_executor: Option<Box<dyn Executor<S>>>,
    // 条件表达式（简化实现）
    condition: String,
}

impl<S: StorageEngine> HavingExecutor<S> {
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
        condition: String,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "HavingExecutor".to_string(), storage),
            input_executor: None,
            condition,
        }
    }
}

impl<S: StorageEngine> InputExecutor<S> for HavingExecutor<S> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>) {
        self.input_executor = Some(input);
    }

    fn get_input(&self) -> Option<&Box<dyn Executor<S>>> {
        self.input_executor.as_ref()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> ExecutorCore for HavingExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        // 首先执行输入执行器（如果存在）
        let _input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute().await?
        } else {
            // 如果没有输入执行器，返回空结果
            ExecutionResult::Values(Vec::new())
        };

        // 在实际实现中，这里会评估 HAVING 条件
        // 暂时返回原始结果
        Ok(_input_result)
    }
}

impl<S: StorageEngine + Send + 'static> ExecutorLifecycle for HavingExecutor<S> {
    fn open(&mut self) -> DBResult<()> {
        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.open()?;
        }
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.close()?;
        }
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }
}

impl<S: StorageEngine + Send + 'static> ExecutorMetadata for HavingExecutor<S> {
    fn id(&self) -> usize {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        "HavingExecutor - filters grouped results using HAVING clause"
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for HavingExecutor<S> {
    fn storage(&self) -> &Arc<Mutex<S>> {
        &self.base.storage
    }
}

/// 聚合状态
#[derive(Debug, Clone)]
pub struct AggregateState {
    pub count: usize,
    pub sum: Option<Value>,
    pub avg: Option<Value>,
    pub max: Option<Value>,
    pub min: Option<Value>,
}

/// 分组聚合状态
#[derive(Debug, Clone)]
pub struct GroupAggregateState {
    pub groups: HashMap<Vec<Value>, AggregateState>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // 测试用例稍后添加
}