//! 聚合操作执行器模块
//!
//! 包含聚合操作相关的执行器，包括：
//! - GroupBy（分组聚合）
//! - Aggregate（整体聚合）
//! - Having（分组后过滤）

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::core::types::operators::AggregateFunction;
use crate::core::Expression;
use crate::core::Value;
use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::expression::evaluator::traits::ExpressionContext;
use crate::expression::DefaultExpressionContext;
use crate::query::executor::base::InputExecutor;
use crate::query::executor::result_processing::traits::{
    BaseResultProcessor, ResultProcessor, ResultProcessorContext,
};
use crate::query::executor::traits::{DBResult, ExecutionResult, Executor, ExecutorStats};
use crate::storage::StorageEngine;

/// 聚合函数规范
/// 包含聚合函数类型和可选的字段名参数
#[derive(Debug, Clone)]
pub struct AggregateFunctionSpec {
    pub function: AggregateFunction,
    pub field: Option<String>,
    pub distinct: bool,
}

impl AggregateFunctionSpec {
    pub fn new(function: AggregateFunction) -> Self {
        Self {
            function,
            field: None,
            distinct: false,
        }
    }

    pub fn with_field(mut self, field: String) -> Self {
        self.field = Some(field);
        self
    }

    pub fn with_distinct(mut self) -> Self {
        self.distinct = true;
        self
    }

    // 便捷构造函数
    pub fn count() -> Self {
        Self::new(AggregateFunction::Count(None))
    }

    pub fn count_with_field(field: String) -> Self {
        Self::new(AggregateFunction::Count(Some(field)))
    }

    pub fn count_distinct(field: String) -> Self {
        Self::new(AggregateFunction::Distinct(field))
    }

    pub fn sum(field: String) -> Self {
        Self::new(AggregateFunction::Sum(field))
    }

    pub fn avg(field: String) -> Self {
        Self::new(AggregateFunction::Avg(field))
    }

    pub fn max(field: String) -> Self {
        Self::new(AggregateFunction::Max(field))
    }

    pub fn min(field: String) -> Self {
        Self::new(AggregateFunction::Min(field))
    }

    /// 从AggregateFunction创建AggregateFunctionSpec
    pub fn from_agg_function(function: AggregateFunction) -> Self {
        Self {
            function,
            field: None,
            distinct: false,
        }
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
    pub collect: Vec<Value>,
    pub distinct_values: std::collections::HashSet<Value>,
    pub percentile_values: Vec<f64>, // 用于计算百分位数
}

impl AggregateState {
    pub fn new() -> Self {
        Self {
            count: 0,
            sum: None,
            avg: None,
            max: None,
            min: None,
            collect: Vec::new(),
            distinct_values: std::collections::HashSet::new(),
            percentile_values: Vec::new(),
        }
    }

    /// 更新聚合状态
    pub fn update(&mut self, value: &Value) -> DBResult<()> {
        self.count += 1;

        // 更新 sum
        let new_sum = match &self.sum {
            Some(sum) => Some(Self::add_values_static(sum, value)?),
            None => Some(value.clone()),
        };
        self.sum = new_sum;

        // 更新 max
        match &mut self.max {
            Some(max) => {
                if value > max {
                    self.max = Some(value.clone());
                }
            }
            None => {
                self.max = Some(value.clone());
            }
        }

        // 更新 min
        match &mut self.min {
            Some(min) => {
                if value < min {
                    self.min = Some(value.clone());
                }
            }
            None => {
                self.min = Some(value.clone());
            }
        }

        // 更新 avg
        if let Some(sum) = &self.sum {
            self.avg = Some(Self::divide_value_static(sum, self.count)?);
        }

        Ok(())
    }

    /// 添加两个值
    fn add_values_static(a: &Value, b: &Value) -> DBResult<Value> {
        match (a, b) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f64)),
            _ => Err(crate::core::error::DBError::Query(
                crate::core::error::QueryError::ExecutionError(
                    "Cannot add these value types".to_string(),
                ),
            )),
        }
    }

    /// 除法运算
    fn divide_value_static(value: &Value, divisor: usize) -> DBResult<Value> {
        if divisor == 0 {
            return Err(crate::core::error::DBError::Query(
                crate::core::error::QueryError::ExecutionError("Division by zero".to_string()),
            ));
        }

        match value {
            Value::Int(v) => Ok(Value::Float(*v as f64 / divisor as f64)),
            Value::Float(v) => Ok(Value::Float(v / divisor as f64)),
            _ => Err(crate::core::error::DBError::Query(
                crate::core::error::QueryError::ExecutionError(
                    "Cannot divide this value type".to_string(),
                ),
            )),
        }
    }

    /// 更新收集状态（COLLECT函数）
    pub fn update_collect(&mut self, value: &Value) -> DBResult<()> {
        self.collect.push(value.clone());
        self.count += 1;
        Ok(())
    }

    /// 更新去重状态（DISTINCT函数）
    pub fn update_distinct(&mut self, value: &Value) -> DBResult<()> {
        let old_size = self.distinct_values.len();
        self.distinct_values.insert(value.clone());
        if self.distinct_values.len() > old_size {
            self.count += 1;
        }
        Ok(())
    }

    /// 更新百分位数状态（PERCENTILE函数）
    pub fn update_percentile(&mut self, value: &Value) -> DBResult<()> {
        match value {
            Value::Int(v) => {
                self.percentile_values.push(*v as f64);
                self.count += 1;
            }
            Value::Float(v) => {
                self.percentile_values.push(*v);
                self.count += 1;
            }
            _ => {
                return Err(crate::core::error::DBError::Query(
                    crate::core::error::QueryError::ExecutionError(
                        "PERCENTILE function only supports numeric values".to_string(),
                    ),
                ));
            }
        }
        Ok(())
    }

    /// 计算百分位数
    pub fn calculate_percentile(&self, percentile: f64) -> DBResult<Value> {
        if self.percentile_values.is_empty() {
            return Ok(Value::Null(crate::core::value::NullType::NaN));
        }

        if percentile < 0.0 || percentile > 100.0 {
            return Err(crate::core::error::DBError::Query(
                crate::core::error::QueryError::ExecutionError(
                    "Percentile must be between 0 and 100".to_string(),
                ),
            ));
        }

        let mut sorted_values = self.percentile_values.clone();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let index = (percentile / 100.0) * (sorted_values.len() - 1) as f64;
        let lower_index = index.floor() as usize;
        let upper_index = index.ceil() as usize;

        if lower_index == upper_index {
            Ok(Value::Float(sorted_values[lower_index]))
        } else {
            let lower_value = sorted_values[lower_index];
            let upper_value = sorted_values[upper_index];
            let weight = index - lower_index as f64;
            let interpolated = lower_value + weight * (upper_value - lower_value);
            Ok(Value::Float(interpolated))
        }
    }
}

/// 分组聚合状态
#[derive(Debug, Clone)]
pub struct GroupAggregateState {
    pub groups: HashMap<Vec<Value>, AggregateState>,
}

impl GroupAggregateState {
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
        }
    }

    /// 更新分组聚合状态
    pub fn update(&mut self, group_key: Vec<Value>, value: &Value) -> DBResult<()> {
        let state = self
            .groups
            .entry(group_key)
            .or_insert_with(AggregateState::new);
        state.update(value)
    }
}

/// AggregateExecutor - 聚合执行器
///
/// 执行聚合操作，支持 COUNT, SUM, AVG, MAX, MIN 等聚合函数
pub struct AggregateExecutor<S: StorageEngine> {
    /// 基础处理器
    base: BaseResultProcessor<S>,
    /// 聚合函数列表
    aggregate_functions: Vec<AggregateFunctionSpec>,
    /// 分组键列表
    group_keys: Vec<Expression>,
    /// 输入执行器
    input_executor: Option<Box<dyn Executor<S>>>,
}

impl<S: StorageEngine> AggregateExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        aggregate_functions: Vec<AggregateFunctionSpec>,
        group_keys: Vec<Expression>,
    ) -> Self {
        let base = BaseResultProcessor::new(
            id,
            "AggregateExecutor".to_string(),
            "Performs aggregation operations on query results".to_string(),
            storage,
        );

        Self {
            base,
            aggregate_functions,
            group_keys,
            input_executor: None,
        }
    }

    /// 处理输入数据并执行聚合
    async fn process_input(&mut self) -> DBResult<crate::core::value::DataSet> {
        if let Some(ref mut input_exec) = self.input_executor {
            let input_result = input_exec.execute().await?;

            match input_result {
                ExecutionResult::DataSet(dataset) => self.aggregate_dataset(dataset).await,
                _ => Err(crate::core::error::DBError::Query(
                    crate::core::error::QueryError::ExecutionError(
                        "Aggregate executor expects DataSet input".to_string(),
                    ),
                )),
            }
        } else {
            Err(crate::core::error::DBError::Query(
                crate::core::error::QueryError::ExecutionError(
                    "Aggregate executor requires input executor".to_string(),
                ),
            ))
        }
    }

    /// 对数据集执行聚合
    async fn aggregate_dataset(
        &mut self,
        dataset: crate::core::value::DataSet,
    ) -> DBResult<crate::core::value::DataSet> {
        let mut group_state = GroupAggregateState::new();

        // 处理每一行数据
        for row in &dataset.rows {
            // 构建表达式上下文
            let mut context = DefaultExpressionContext::new();
            for (i, col_name) in dataset.col_names.iter().enumerate() {
                if i < row.len() {
                    context.set_variable(col_name.clone(), row[i].clone());
                }
            }

            // 计算分组键
            let mut group_key = Vec::new();
            for group_expr in &self.group_keys {
                let key_value =
                    ExpressionEvaluator::evaluate(group_expr, &mut context).map_err(|e| {
                        crate::core::error::DBError::Expression(
                            crate::core::error::ExpressionError::function_error(format!(
                                "Failed to evaluate group key: {}",
                                e
                            )),
                        )
                    })?;
                group_key.push(key_value);
            }

            // 更新聚合状态
            for agg_func in &self.aggregate_functions {
                match &agg_func.function {
                    AggregateFunction::Count(_) => {
                        if agg_func.distinct {
                            // COUNT(DISTINCT field)
                            if let Some(field) = &agg_func.field {
                                if let Some(col_index) =
                                    dataset.col_names.iter().position(|name| name == field)
                                {
                                    if col_index < row.len() {
                                        group_state.update(group_key.clone(), &row[col_index])?;
                                    }
                                }
                            } else {
                                // COUNT(DISTINCT *) - 使用整行作为键
                                group_state.update(group_key.clone(), &Value::Int(1))?;
                            }
                        } else if let Some(field) = &agg_func.field {
                            // COUNT(field)
                            if let Some(col_index) =
                                dataset.col_names.iter().position(|name| name == field)
                            {
                                if col_index < row.len() {
                                    group_state.update(group_key.clone(), &row[col_index])?;
                                }
                            }
                        } else {
                            // COUNT(*) 或 COUNT(1)
                            group_state.update(group_key.clone(), &Value::Int(1))?;
                        }
                    }
                    AggregateFunction::Sum(_)
                    | AggregateFunction::Avg(_)
                    | AggregateFunction::Max(_)
                    | AggregateFunction::Min(_) => {
                        // 需要字段名的聚合函数
                        if let Some(field) = &agg_func.field {
                            if let Some(col_index) =
                                dataset.col_names.iter().position(|name| name == field)
                            {
                                if col_index < row.len() {
                                    group_state.update(group_key.clone(), &row[col_index])?;
                                }
                            }
                        }
                    }
                    AggregateFunction::Collect(_) => {
                        // COLLECT函数 - 收集所有值到列表
                        if let Some(field) = &agg_func.field {
                            if let Some(col_index) =
                                dataset.col_names.iter().position(|name| name == field)
                            {
                                if col_index < row.len() {
                                    // 获取或创建聚合状态
                                    let state = group_state
                                        .groups
                                        .entry(group_key.clone())
                                        .or_insert_with(AggregateState::new);
                                    state.update_collect(&row[col_index])?;
                                }
                            }
                        }
                    }
                    AggregateFunction::Distinct(_) => {
                        // DISTINCT函数 - 收集去重后的值
                        if let Some(field) = &agg_func.field {
                            if let Some(col_index) =
                                dataset.col_names.iter().position(|name| name == field)
                            {
                                if col_index < row.len() {
                                    // 获取或创建聚合状态
                                    let state = group_state
                                        .groups
                                        .entry(group_key.clone())
                                        .or_insert_with(AggregateState::new);
                                    state.update_distinct(&row[col_index])?;
                                }
                            }
                        }
                    }
                    AggregateFunction::Percentile(_, _) => {
                        // PERCENTILE函数 - 需要字段和百分位数两个参数
                        if let Some(field) = &agg_func.field {
                            if let Some(col_index) =
                                dataset.col_names.iter().position(|name| name == field)
                            {
                                if col_index < row.len() {
                                    // 获取或创建聚合状态
                                    let state = group_state
                                        .groups
                                        .entry(group_key.clone())
                                        .or_insert_with(AggregateState::new);
                                    state.update_percentile(&row[col_index])?;
                                }
                            }
                        }
                    }
                }
            }
        }

        // 构建结果数据集
        let mut result_dataset = crate::core::value::DataSet::new();

        // 设置列名
        for _group_expr in &self.group_keys {
            result_dataset
                .col_names
                .push(format!("group_{}", result_dataset.col_names.len()));
        }

        for agg_func in &self.aggregate_functions {
            let col_name = match &agg_func.function {
                AggregateFunction::Count(_) => {
                    if agg_func.distinct {
                        if let Some(field) = &agg_func.field {
                            format!("count_distinct_{}", field)
                        } else {
                            "count_distinct".to_string()
                        }
                    } else if let Some(field) = &agg_func.field {
                        format!("count_{}", field)
                    } else {
                        "count".to_string()
                    }
                }
                AggregateFunction::Sum(_) => {
                    if let Some(field) = &agg_func.field {
                        format!("sum_{}", field)
                    } else {
                        "sum".to_string()
                    }
                }
                AggregateFunction::Avg(_) => {
                    if let Some(field) = &agg_func.field {
                        format!("avg_{}", field)
                    } else {
                        "avg".to_string()
                    }
                }
                AggregateFunction::Max(_) => {
                    if let Some(field) = &agg_func.field {
                        format!("max_{}", field)
                    } else {
                        "max".to_string()
                    }
                }
                AggregateFunction::Min(_) => {
                    if let Some(field) = &agg_func.field {
                        format!("min_{}", field)
                    } else {
                        "min".to_string()
                    }
                }
                AggregateFunction::Collect(_) => "collect".to_string(),
                AggregateFunction::Distinct(_) => "distinct".to_string(),
                AggregateFunction::Percentile(_, _) => {
                    if let Some(field) = &agg_func.field {
                        format!("percentile_{}", field)
                    } else {
                        "percentile".to_string()
                    }
                }
            };
            result_dataset.col_names.push(col_name);
        }

        // 填充结果行
        for (group_key, agg_state) in &group_state.groups {
            let mut result_row = Vec::new();

            // 添加分组键值
            result_row.extend_from_slice(group_key);

            // 添加聚合结果
            for agg_func in &self.aggregate_functions {
                let agg_value = match &agg_func.function {
                    AggregateFunction::Count(_) => Value::Int(agg_state.count as i64),
                    AggregateFunction::Sum(_) => agg_state
                        .sum
                        .clone()
                        .unwrap_or(Value::Null(crate::core::value::NullType::NaN)),
                    AggregateFunction::Avg(_) => agg_state
                        .avg
                        .clone()
                        .unwrap_or(Value::Null(crate::core::value::NullType::NaN)),
                    AggregateFunction::Max(_) => agg_state
                        .max
                        .clone()
                        .unwrap_or(Value::Null(crate::core::value::NullType::NaN)),
                    AggregateFunction::Min(_) => agg_state
                        .min
                        .clone()
                        .unwrap_or(Value::Null(crate::core::value::NullType::NaN)),
                    AggregateFunction::Collect(_) => {
                        // COLLECT函数 - 返回收集的所有值
                        if agg_state.collect.is_empty() {
                            Value::List(Vec::new())
                        } else {
                            Value::List(agg_state.collect.clone())
                        }
                    }
                    AggregateFunction::Distinct(_) => {
                        // DISTINCT函数 - 返回去重后的值集合
                        if agg_state.distinct_values.is_empty() {
                            Value::Set(std::collections::HashSet::new())
                        } else {
                            Value::Set(agg_state.distinct_values.clone())
                        }
                    }
                    AggregateFunction::Percentile(_, _) => {
                        // PERCENTILE函数 - 计算百分位数
                        // 这里简化处理，使用默认的50%百分位数（中位数）
                        // 在实际应用中，应该从查询参数中获取百分位数值
                        agg_state
                            .calculate_percentile(50.0)
                            .unwrap_or(Value::Null(crate::core::value::NullType::NaN))
                    }
                };
                result_row.push(agg_value);
            }

            result_dataset.rows.push(result_row);
        }

        Ok(result_dataset)
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> ResultProcessor<S> for AggregateExecutor<S> {
    async fn process(&mut self, input: ExecutionResult) -> DBResult<ExecutionResult> {
        ResultProcessor::set_input(self, input);
        let dataset = self.process_input().await?;
        Ok(ExecutionResult::DataSet(dataset))
    }

    fn set_input(&mut self, input: ExecutionResult) {
        self.base.input = Some(input);
    }

    fn get_input(&self) -> Option<&ExecutionResult> {
        self.base.input.as_ref()
    }

    fn context(&self) -> &ResultProcessorContext {
        &self.base.context
    }

    fn set_context(&mut self, context: ResultProcessorContext) {
        self.base.context = context;
    }

    fn memory_usage(&self) -> usize {
        self.base.memory_usage
    }

    fn reset(&mut self) {
        self.base.reset_state();
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for AggregateExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute().await?
        } else {
            self.base
                .input
                .clone()
                .unwrap_or(ExecutionResult::DataSet(crate::core::value::DataSet::new()))
        };

        self.process(input_result).await
    }

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
        self.base.id > 0
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        &self.base.description
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageEngine + Send + 'static> InputExecutor<S> for AggregateExecutor<S> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>) {
        self.input_executor = Some(input);
    }

    fn get_input(&self) -> Option<&Box<dyn Executor<S>>> {
        self.input_executor.as_ref()
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
        id: i64,
        storage: Arc<Mutex<S>>,
        aggregate_functions: Vec<AggregateFunctionSpec>,
        group_keys: Vec<Expression>,
    ) -> Self {
        Self {
            aggregate_executor: AggregateExecutor::new(
                id,
                storage,
                aggregate_functions,
                group_keys,
            ),
        }
    }
}

impl<S: StorageEngine + 'static> InputExecutor<S> for GroupByExecutor<S> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>) {
        InputExecutor::set_input(&mut self.aggregate_executor, input);
    }

    fn get_input(&self) -> Option<&Box<dyn Executor<S>>> {
        InputExecutor::get_input(&self.aggregate_executor)
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for GroupByExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        self.aggregate_executor.execute().await
    }

    fn open(&mut self) -> DBResult<()> {
        self.aggregate_executor.open()
    }

    fn close(&mut self) -> DBResult<()> {
        self.aggregate_executor.close()
    }

    fn is_open(&self) -> bool {
        self.aggregate_executor.is_open()
    }

    fn id(&self) -> i64 {
        self.aggregate_executor.id()
    }

    fn name(&self) -> &str {
        "GroupByExecutor"
    }

    fn description(&self) -> &str {
        "GroupByExecutor - performs GROUP BY operations"
    }

    fn stats(&self) -> &ExecutorStats {
        self.aggregate_executor.stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.aggregate_executor.stats_mut()
    }
}

/// HavingExecutor - HAVING 子句执行器
///
/// 实现 HAVING 子句，对分组后的结果进行过滤
pub struct HavingExecutor<S: StorageEngine> {
    /// 基础处理器
    base: BaseResultProcessor<S>,
    /// HAVING 条件表达式
    condition: Expression,
    /// 输入执行器
    input_executor: Option<Box<dyn Executor<S>>>,
}

impl<S: StorageEngine> HavingExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, condition: Expression) -> Self {
        let base = BaseResultProcessor::new(
            id,
            "HavingExecutor".to_string(),
            "Filters grouped results using HAVING clause".to_string(),
            storage,
        );

        Self {
            base,
            condition,
            input_executor: None,
        }
    }

    /// 处理输入数据并应用 HAVING 条件
    async fn process_input(&mut self) -> DBResult<crate::core::value::DataSet> {
        if let Some(ref mut input_exec) = self.input_executor {
            let input_result = input_exec.execute().await?;

            match input_result {
                ExecutionResult::DataSet(mut dataset) => {
                    self.apply_having_condition(&mut dataset)?;
                    Ok(dataset)
                }
                _ => Err(crate::core::error::DBError::Query(
                    crate::core::error::QueryError::ExecutionError(
                        "Having executor expects DataSet input".to_string(),
                    ),
                )),
            }
        } else {
            Err(crate::core::error::DBError::Query(
                crate::core::error::QueryError::ExecutionError(
                    "Having executor requires input executor".to_string(),
                ),
            ))
        }
    }

    /// 应用 HAVING 条件过滤
    fn apply_having_condition(&self, dataset: &mut crate::core::value::DataSet) -> DBResult<()> {
        let mut filtered_rows = Vec::new();

        for row in &dataset.rows {
            let mut context = DefaultExpressionContext::new();
            for (i, col_name) in dataset.col_names.iter().enumerate() {
                if i < row.len() {
                    context.set_variable(col_name.clone(), row[i].clone());
                }
            }

            let condition_result = ExpressionEvaluator::evaluate(&self.condition, &mut context)
                .map_err(|e| {
                    crate::core::error::DBError::Expression(
                        crate::core::error::ExpressionError::function_error(format!(
                            "Failed to evaluate HAVING condition: {}",
                            e
                        )),
                    )
                })?;

            if let Value::Bool(true) = condition_result {
                filtered_rows.push(row.clone());
            }
        }

        dataset.rows = filtered_rows;
        Ok(())
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> ResultProcessor<S> for HavingExecutor<S> {
    async fn process(&mut self, input: ExecutionResult) -> DBResult<ExecutionResult> {
        ResultProcessor::set_input(self, input);
        let dataset = self.process_input().await?;
        Ok(ExecutionResult::DataSet(dataset))
    }

    fn set_input(&mut self, input: ExecutionResult) {
        self.base.input = Some(input);
    }

    fn get_input(&self) -> Option<&ExecutionResult> {
        self.base.input.as_ref()
    }

    fn context(&self) -> &ResultProcessorContext {
        &self.base.context
    }

    fn set_context(&mut self, context: ResultProcessorContext) {
        self.base.context = context;
    }

    fn memory_usage(&self) -> usize {
        self.base.memory_usage
    }

    fn reset(&mut self) {
        self.base.reset_state();
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for HavingExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute().await?
        } else {
            self.base
                .input
                .clone()
                .unwrap_or(ExecutionResult::DataSet(crate::core::value::DataSet::new()))
        };

        self.process(input_result).await
    }

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
        self.base.id > 0
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        &self.base.description
    }

    fn stats(&self) -> &ExecutorStats {
        &self.base.stats
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        &mut self.base.stats
    }
}

impl<S: StorageEngine + Send + 'static> InputExecutor<S> for HavingExecutor<S> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>) {
        self.input_executor = Some(input);
    }

    fn get_input(&self) -> Option<&Box<dyn Executor<S>>> {
        self.input_executor.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value::NullType;
    use crate::storage::test_mock::MockStorage;

    #[tokio::test]
    async fn test_aggregate_executor_basic() {
        let storage = Arc::new(Mutex::new(MockStorage));

        // 创建测试数据
        let mut dataset = crate::core::value::DataSet::new();
        dataset.col_names = vec!["department".to_string(), "salary".to_string()];
        dataset
            .rows
            .push(vec![Value::String("IT".to_string()), Value::Int(50000)]);
        dataset
            .rows
            .push(vec![Value::String("HR".to_string()), Value::Int(45000)]);
        dataset
            .rows
            .push(vec![Value::String("IT".to_string()), Value::Int(60000)]);
        dataset
            .rows
            .push(vec![Value::String("HR".to_string()), Value::Int(48000)]);

        // 创建聚合执行器 (按部门分组，计算平均薪资)
        let aggregate_functions = vec![AggregateFunctionSpec::avg("salary".to_string())];
        let group_keys = vec![Expression::Property {
            object: Box::new(Expression::Variable("row".to_string())),
            property: "department".to_string(),
        }];

        let mut executor = AggregateExecutor::new(1, storage, aggregate_functions, group_keys);

        // 设置输入数据
        ResultProcessor::set_input(&mut executor, ExecutionResult::DataSet(dataset));

        // 执行聚合
        let result = executor
            .process(ExecutionResult::DataSet(crate::core::value::DataSet::new()))
            .await
            .expect("Failed to process aggregation");

        // 验证结果
        match result {
            ExecutionResult::DataSet(agg_dataset) => {
                assert_eq!(agg_dataset.rows.len(), 2); // 两个部门
                assert_eq!(agg_dataset.col_names, vec!["group_0", "avg_salary"]);

                // 验证聚合结果
                for row in &agg_dataset.rows {
                    if let Value::String(dept) = &row[0] {
                        if dept == "IT" {
                            // IT部门平均薪资: (50000 + 60000) / 2 = 55000
                            assert_eq!(row[1], Value::Float(55000.0));
                        } else if dept == "HR" {
                            // HR部门平均薪资: (45000 + 48000) / 2 = 46500
                            assert_eq!(row[1], Value::Float(46500.0));
                        }
                    }
                }
            }
            _ => panic!("Expected DataSet result"),
        }
    }
}
