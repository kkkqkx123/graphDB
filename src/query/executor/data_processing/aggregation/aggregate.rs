use std::sync::{Arc, Mutex};
use async_trait::async_trait;

use crate::core::{Value, DataSet};
use crate::graph::expression::{Expression, ExpressionContext};
use crate::query::executor::base::{Executor, ExecutionResult, BaseExecutor};
use crate::query::QueryError;
use crate::storage::StorageEngine;

/// 聚合执行器 - 对整个输入数据集执行聚合操作
pub struct AggregateExecutor<S: StorageEngine + Send + 'static> {
    base: BaseExecutor<S>,
    /// 聚合表达式列表
    aggregate_exprs: Vec<(String, Expression)>, // (函数名, 表达式)
    /// 输入执行器
    input_executor: Option<Box<dyn Executor<S>>>,
    /// 输出列名
    output_column_names: Vec<String>,
    /// 聚合状态
    aggregate_state: AggregateState,
}

/// 聚合状态，保存整个数据集的聚合信息
#[derive(Debug, Clone)]
pub struct AggregateState {
    pub counts: Vec<i64>,
    pub sums: Vec<f64>,
    pub mins: Vec<Option<Value>>,
    pub maxs: Vec<Option<Value>>,
    pub values: Vec<Vec<Value>>,
}

impl AggregateState {
    pub fn new(aggregate_count: usize) -> Self {
        Self {
            counts: vec![0; aggregate_count],
            sums: vec![0.0; aggregate_count],
            mins: vec![None; aggregate_count],
            maxs: vec![None; aggregate_count],
            values: vec![Vec::new(); aggregate_count],
        }
    }

    pub fn reset(&mut self) {
        for i in 0..self.counts.len() {
            self.counts[i] = 0;
            self.sums[i] = 0.0;
            self.mins[i] = None;
            self.maxs[i] = None;
            self.values[i].clear();
        }
    }

    /// 更新聚合状态
    pub fn update(&mut self, index: usize, value: &Value) {
        if index >= self.counts.len() {
            return;
        }

        // 计数更新
        self.counts[index] += 1;

        // 求和更新（仅适用于数字类型）
        if let Value::Int(i) = value {
            self.sums[index] += *i as f64;
        } else if let Value::Float(f) = value {
            self.sums[index] += *f;
        }

        // 最小值更新
        if self.mins[index].is_none() {
            self.mins[index] = Some(value.clone());
        } else if let Some(ref current_min) = &self.mins[index] {
            if value.lt(current_min) {
                self.mins[index] = Some(value.clone());
            }
        }

        // 最大值更新
        if self.maxs[index].is_none() {
            self.maxs[index] = Some(value.clone());
        } else if let Some(ref current_max) = &self.maxs[index] {
            if value.gt(current_max) {
                self.maxs[index] = Some(value.clone());
            }
        }

        // 值列表更新
        self.values[index].push(value.clone());
    }

    /// 获取聚合结果
    pub fn get_result(&self, index: usize, func_name: &str) -> Value {
        if index >= self.counts.len() {
            return Value::Null(crate::core::value::NullType::Null);
        }

        match func_name.to_lowercase().as_str() {
            "count" => Value::Int(self.counts[index]),
            "sum" => Value::Float(self.sums[index]),
            "avg" => {
                let count = self.counts[index] as f64;
                if count > 0.0 {
                    Value::Float(self.sums[index] / count)
                } else {
                    Value::Float(0.0)
                }
            },
            "min" => self.mins[index].clone().unwrap_or(Value::Null(crate::core::value::NullType::Null)),
            "max" => self.maxs[index].clone().unwrap_or(Value::Null(crate::core::value::NullType::Null)),
            "collect" => Value::List(self.values[index].clone()),
            _ => Value::Null(crate::core::value::NullType::Null),
        }
    }
}

impl<S: StorageEngine + Send + 'static> AggregateExecutor<S> {
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
        aggregate_exprs: Vec<(String, Expression)>,
        output_column_names: Vec<String>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "AggregateExecutor".to_string(), storage),
            aggregate_exprs: aggregate_exprs.clone(),
            input_executor: None,
            output_column_names,
            aggregate_state: AggregateState::new(aggregate_exprs.len()),
        }
    }

    /// 处理输入数据并进行聚合
    async fn process_input(&mut self) -> Result<(), QueryError> {
        if let Some(ref mut input_exec) = self.input_executor {
            let input_result = input_exec.execute().await?;
            
            match input_result {
                ExecutionResult::DataSet(data_set) => {
                    self.process_data_set(data_set).await?;
                }
                ExecutionResult::Values(values) => {
                    self.process_values(values).await?;
                }
                ExecutionResult::Vertices(vertices) => {
                    self.process_vertices(vertices).await?;
                }
                ExecutionResult::Edges(edges) => {
                    self.process_edges(edges).await?;
                }
                _ => return Err(QueryError::ExecutionError("Unsupported input type for Aggregate".to_string())),
            }
        }
        
        Ok(())
    }

    /// 处理数据集
    async fn process_data_set(&mut self, data_set: DataSet) -> Result<(), QueryError> {
        for row in data_set.rows {
            // 构建表达式上下文
            let mut expr_context = ExpressionContext::new();
            for (i, col_name) in data_set.col_names.iter().enumerate() {
                if i < row.len() {
                    expr_context.set_variable(col_name.clone(), row[i].clone());
                }
            }

            // 计算并更新聚合值
            for (i, (func_name, expr)) in self.aggregate_exprs.iter().enumerate() {
                let value = expr.evaluate(&expr_context)
                    .map_err(|e| QueryError::ExpressionError(e.to_string()))?;
                
                self.aggregate_state.update(i, &value);
            }
        }

        Ok(())
    }

    /// 处理值列表
    async fn process_values(&mut self, values: Vec<Value>) -> Result<(), QueryError> {
        for value in values {
            let mut expr_context = ExpressionContext::new();
            expr_context.set_variable("_current_value".to_string(), value);

            // 计算并更新聚合值
            for (i, (func_name, expr)) in self.aggregate_exprs.iter().enumerate() {
                let value = expr.evaluate(&expr_context)
                    .map_err(|e| QueryError::ExpressionError(e.to_string()))?;
                
                self.aggregate_state.update(i, &value);
            }
        }

        Ok(())
    }

    /// 处理顶点列表
    async fn process_vertices(&mut self, vertices: Vec<crate::core::Vertex>) -> Result<(), QueryError> {
        for vertex in vertices {
            let mut expr_context = ExpressionContext::new();
            expr_context.set_variable("_vertex".to_string(), Value::Vertex(Box::new(vertex)));

            // 计算并更新聚合值
            for (i, (func_name, expr)) in self.aggregate_exprs.iter().enumerate() {
                let value = expr.evaluate(&expr_context)
                    .map_err(|e| QueryError::ExpressionError(e.to_string()))?;
                
                self.aggregate_state.update(i, &value);
            }
        }

        Ok(())
    }

    /// 处理边列表
    async fn process_edges(&mut self, edges: Vec<crate::core::Edge>) -> Result<(), QueryError> {
        for edge in edges {
            let mut expr_context = ExpressionContext::new();
            expr_context.set_variable("_edge".to_string(), Value::Edge(edge));

            // 计算并更新聚合值
            for (i, (func_name, expr)) in self.aggregate_exprs.iter().enumerate() {
                let value = expr.evaluate(&expr_context)
                    .map_err(|e| QueryError::ExpressionError(e.to_string()))?;
                
                self.aggregate_state.update(i, &value);
            }
        }

        Ok(())
    }

    /// 构建结果数据集
    fn build_result(&self) -> Result<DataSet, QueryError> {
        let mut result_dataset = DataSet {
            col_names: self.output_column_names.clone(),
            rows: Vec::new(),
        };

        // 创建一行结果
        let mut row = Vec::new();
        for (i, (func_name, _)) in self.aggregate_exprs.iter().enumerate() {
            let result = self.aggregate_state.get_result(i, func_name);
            row.push(result);
        }

        result_dataset.rows.push(row);

        Ok(result_dataset)
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for AggregateExecutor<S> {
    async fn execute(&mut self) -> Result<ExecutionResult, QueryError> {
        // 处理输入数据并进行聚合
        self.process_input().await?;

        // 构建结果数据集
        let dataset = self.build_result()?;
        
        Ok(ExecutionResult::DataSet(dataset))
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

impl<S: StorageEngine + Send + 'static> crate::query::executor::base::InputExecutor<S> for AggregateExecutor<S> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>) {
        self.input_executor = Some(input);
    }

    fn get_input(&self) -> Option<&Box<dyn Executor<S>>> {
        self.input_executor.as_ref()
    }
}