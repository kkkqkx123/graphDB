use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;

use crate::core::{Value, DataSet, List};
use crate::graph::expression::{Expression, ExpressionContext};
use crate::query::executor::base::{Executor, ExecutionResult, BaseExecutor};
use crate::query::QueryError;
use crate::storage::StorageEngine;

/// 聚合状态，用于保存每个分组的聚合信息
#[derive(Debug, Clone)]
pub struct AggregateState {
    pub counts: HashMap<String, i64>,  // 记录不同聚合函数的数量
    pub sums: HashMap<String, f64>,    // 记录不同聚合函数的总和
    pub mins: HashMap<String, Option<Value>>, // 记录不同聚合函数的最小值
    pub maxs: HashMap<String, Option<Value>>, // 记录不同聚合函数的最大值
    pub values: HashMap<String, Vec<Value>>,  // 记录不同聚合函数的值列表（用于collect等）
}

impl AggregateState {
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
            sums: HashMap::new(),
            mins: HashMap::new(),
            maxs: HashMap::new(),
            values: HashMap::new(),
        }
    }

    pub fn reset(&mut self) {
        self.counts.clear();
        self.sums.clear();
        self.mins.clear();
        self.maxs.clear();
        self.values.clear();
    }

    /// 更新聚合状态
    pub fn update(&mut self, func_name: &str, value: &Value) {
        // 计数更新
        *self.counts.entry(func_name.to_string()).or_insert(0) += 1;

        // 求和更新（仅适用于数字类型）
        if let Value::Int(i) = value {
            *self.sums.entry(func_name.to_string()).or_insert(0.0) += *i as f64;
        } else if let Value::Float(f) = value {
            *self.sums.entry(func_name.to_string()).or_insert(0.0) += *f;
        }

        // 最小值更新
        let min_entry = self.mins.entry(func_name.to_string()).or_insert(None);
        if min_entry.is_none() {
            *min_entry = Some(value.clone());
        } else if let Some(ref current_min) = min_entry {
            if value.lt(current_min) {
                *min_entry = Some(value.clone());
            }
        }

        // 最大值更新
        let max_entry = self.maxs.entry(func_name.to_string()).or_insert(None);
        if max_entry.is_none() {
            *max_entry = Some(value.clone());
        } else if let Some(ref current_max) = max_entry {
            if value.gt(current_max) {
                *max_entry = Some(value.clone());
            }
        }

        // 值列表更新
        self.values.entry(func_name.to_string()).or_insert_with(Vec::new).push(value.clone());
    }

    /// 获取聚合结果
    pub fn get_result(&self, func_name: &str) -> Option<Value> {
        match func_name.to_lowercase().as_str() {
            "count" => self.counts.get(func_name).cloned().map(Value::Int).or(Some(Value::Int(0))),
            "sum" => self.sums.get(func_name).cloned().map(Value::Float).or(Some(Value::Float(0.0))),
            "avg" => {
                let count = *self.counts.get(func_name).unwrap_or(&0) as f64;
                let sum = *self.sums.get(func_name).unwrap_or(&0.0);
                if count > 0.0 {
                    Some(Value::Float(sum / count))
                } else {
                    Some(Value::Float(0.0))
                }
            },
            "min" => self.mins.get(func_name).cloned().flatten().or(Some(Value::Null(crate::core::value::NullType::Null))),
            "max" => self.maxs.get(func_name).cloned().flatten().or(Some(Value::Null(crate::core::value::NullType::Null))),
            "collect" => Some(Value::List(self.values.get(func_name).cloned().unwrap_or_default())),
            _ => Some(Value::Null(crate::core::value::NullType::Null)),
        }
    }
}

/// GroupBy执行器
pub struct GroupByExecutor<S: StorageEngine + Send + 'static> {
    base: BaseExecutor<S>,
    /// 分组键表达式
    group_keys: Vec<Expression>,
    /// 聚合表达式
    aggregate_exprs: Vec<(String, Expression)>, // (函数名, 表达式)
    /// 输入执行器
    input_executor: Option<Box<dyn Executor<S>>>,
    /// 内部分组哈希表
    group_table: HashMap<List, AggregateState>,
    /// 输出列名
    output_column_names: Vec<String>,
}

impl<S: StorageEngine + Send + 'static> GroupByExecutor<S> {
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
        group_keys: Vec<Expression>,
        aggregate_exprs: Vec<(String, Expression)>,
        output_column_names: Vec<String>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "GroupByExecutor".to_string(), storage),
            group_keys,
            aggregate_exprs,
            input_executor: None,
            group_table: HashMap::new(),
            output_column_names,
        }
    }

    /// 处理输入数据并进行分组聚合
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
                _ => return Err(QueryError::ExecutionError("Unsupported input type for GroupBy".to_string())),
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

            // 计算分组键
            let group_key = self.compute_group_key(&expr_context)?;
            
            // 获取或创建聚合状态
            let agg_state = self.group_table.entry(group_key).or_insert_with(AggregateState::new);

            // 计算并更新聚合值
            for (func_name, expr) in &self.aggregate_exprs {
                let value = expr.evaluate(&expr_context)
                    .map_err(|e| QueryError::ExpressionError(e.to_string()))?;
                
                agg_state.update(func_name, &value);
            }
        }

        Ok(())
    }

    /// 处理值列表
    async fn process_values(&mut self, values: Vec<Value>) -> Result<(), QueryError> {
        for value in values {
            let mut expr_context = ExpressionContext::new();
            expr_context.set_variable("_current_value".to_string(), value);

            // 计算分组键
            let group_key = self.compute_group_key(&expr_context)?;
            
            // 获取或创建聚合状态
            let agg_state = self.group_table.entry(group_key).or_insert_with(AggregateState::new);

            // 计算并更新聚合值
            for (func_name, expr) in &self.aggregate_exprs {
                let value = expr.evaluate(&expr_context)
                    .map_err(|e| QueryError::ExpressionError(e.to_string()))?;
                
                agg_state.update(func_name, &value);
            }
        }

        Ok(())
    }

    /// 处理顶点列表
    async fn process_vertices(&mut self, vertices: Vec<crate::core::Vertex>) -> Result<(), QueryError> {
        for vertex in vertices {
            let mut expr_context = ExpressionContext::new();
            expr_context.set_variable("_vertex".to_string(), Value::Vertex(Box::new(vertex)));

            // 计算分组键
            let group_key = self.compute_group_key(&expr_context)?;
            
            // 获取或创建聚合状态
            let agg_state = self.group_table.entry(group_key).or_insert_with(AggregateState::new);

            // 计算并更新聚合值
            for (func_name, expr) in &self.aggregate_exprs {
                let value = expr.evaluate(&expr_context)
                    .map_err(|e| QueryError::ExpressionError(e.to_string()))?;
                
                agg_state.update(func_name, &value);
            }
        }

        Ok(())
    }

    /// 处理边列表
    async fn process_edges(&mut self, edges: Vec<crate::core::Edge>) -> Result<(), QueryError> {
        for edge in edges {
            let mut expr_context = ExpressionContext::new();
            expr_context.set_variable("_edge".to_string(), Value::Edge(edge));

            // 计算分组键
            let group_key = self.compute_group_key(&expr_context)?;
            
            // 获取或创建聚合状态
            let agg_state = self.group_table.entry(group_key).or_insert_with(AggregateState::new);

            // 计算并更新聚合值
            for (func_name, expr) in &self.aggregate_exprs {
                let value = expr.evaluate(&expr_context)
                    .map_err(|e| QueryError::ExpressionError(e.to_string()))?;
                
                agg_state.update(func_name, &value);
            }
        }

        Ok(())
    }

    /// 计算分组键
    fn compute_group_key(&self, expr_context: &ExpressionContext) -> Result<List, QueryError> {
        let mut key_list = List { values: Vec::new() };
        
        for expr in &self.group_keys {
            let value = expr.evaluate(expr_context)
                .map_err(|e| QueryError::ExpressionError(e.to_string()))?;
            key_list.values.push(value);
        }

        Ok(key_list)
    }

    /// 构建结果数据集
    fn build_result(&self) -> Result<DataSet, QueryError> {
        let mut result_dataset = DataSet {
            col_names: self.output_column_names.clone(),
            rows: Vec::new(),
        };

        for (group_key, agg_state) in &self.group_table {
            let mut row = Vec::new();
            
            // 添加分组键
            for key_value in &group_key.values {
                row.push(key_value.clone());
            }
            
            // 添加聚合结果
            for (func_name, _) in &self.aggregate_exprs {
                if let Some(result) = agg_state.get_result(func_name) {
                    row.push(result);
                } else {
                    row.push(Value::Null(crate::core::value::NullType::Null));
                }
            }

            result_dataset.rows.push(row);
        }

        Ok(result_dataset)
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for GroupByExecutor<S> {
    async fn execute(&mut self) -> Result<ExecutionResult, QueryError> {
        // 处理输入数据并进行分组聚合
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

impl<S: StorageEngine + Send + 'static> crate::query::executor::base::InputExecutor<S> for GroupByExecutor<S> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>) {
        self.input_executor = Some(input);
    }

    fn get_input(&self) -> Option<&Box<dyn Executor<S>>> {
        self.input_executor.as_ref()
    }
}