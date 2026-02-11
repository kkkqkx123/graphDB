//! 聚合操作执行器模块
//!
//! 包含聚合操作相关的执行器，包括：
//! - GroupBy（分组聚合）
//! - Aggregate（整体聚合）
//! - Having（分组后过滤）
//!
//! CPU 密集型操作，使用 Rayon 进行并行化
//!
//! 参考 nebula-graph 的 AggregateExecutor 实现：
//! - 使用 AggData 管理聚合状态
//! - 使用 AggFunctionManager 管理聚合函数
//! - 统一处理 NULL 和空值

use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::core::types::operators::AggregateFunction;
use crate::core::value::{NullType, Value};
use crate::core::Expression;
use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::expression::evaluator::traits::ExpressionContext;
use crate::expression::DefaultExpressionContext;
use crate::query::executor::base::InputExecutor;
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::executor::recursion_detector::ParallelConfig;
use crate::query::executor::result_processing::agg_data::AggData;
use crate::query::executor::result_processing::agg_function_manager::AggFunctionManager;
use crate::query::executor::result_processing::traits::{
    BaseResultProcessor, ResultProcessor, ResultProcessorContext,
};
use crate::query::executor::traits::{DBResult, ExecutionResult, Executor, ExecutorStats};
use crate::storage::StorageClient;

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
        Self {
            function: AggregateFunction::Avg(field.clone()),
            field: Some(field),
            distinct: false,
        }
    }

    pub fn max(field: String) -> Self {
        Self::new(AggregateFunction::Max(field))
    }

    pub fn min(field: String) -> Self {
        Self::new(AggregateFunction::Min(field))
    }

    pub fn collect(field: String) -> Self {
        Self::new(AggregateFunction::Collect(field))
    }

    pub fn collect_set(field: String) -> Self {
        Self::new(AggregateFunction::CollectSet(field))
    }

    /// 从 AggregateFunction 创建 AggregateFunctionSpec
    pub fn from_agg_function(function: AggregateFunction) -> Self {
        let field = function.field_name().map(|s| s.to_string());
        Self {
            function,
            field,
            distinct: false,
        }
    }

    /// 获取聚合函数名称（用于 AggFunctionManager）
    pub fn agg_function_name(&self) -> String {
        let base_name = self.function.name().to_string();
        if self.distinct && matches!(self.function, AggregateFunction::Count(_)) {
            // COUNT DISTINCT 使用 COLLECT_SET 去重后计数
            "COLLECT_SET".to_string()
        } else {
            base_name
        }
    }
}

/// 分组聚合状态（使用新的 AggData）
#[derive(Debug, Clone)]
pub struct GroupAggregateState {
    /// 每个分组键对应的聚合数据列表
    /// 每个聚合函数对应一个 AggData
    pub groups: HashMap<Vec<Value>, Vec<AggData>>,
    /// 聚合函数数量
    pub agg_func_count: usize,
}

impl GroupAggregateState {
    pub fn new(agg_func_count: usize) -> Self {
        Self {
            groups: HashMap::new(),
            agg_func_count,
        }
    }

    /// 获取或创建分组的聚合数据
    pub fn get_or_create_agg_data(&mut self, group_key: Vec<Value>) -> &mut Vec<AggData> {
        self.groups.entry(group_key).or_insert_with(|| {
            (0..self.agg_func_count).map(|_| AggData::new()).collect()
        })
    }

    /// 合并另一个 GroupAggregateState
    pub fn merge(&mut self, other: GroupAggregateState) -> DBResult<()> {
        for (group_key, other_agg_data_list) in other.groups {
            let self_agg_data_list = self.get_or_create_agg_data(group_key);
            for (i, other_agg_data) in other_agg_data_list.iter().enumerate() {
                if i < self_agg_data_list.len() {
                    Self::merge_agg_data(&mut self_agg_data_list[i], other_agg_data)?;
                }
            }
        }
        Ok(())
    }

    /// 合并两个 AggData
    fn merge_agg_data(target: &mut AggData, source: &AggData) -> DBResult<()> {
        // 合并 COUNT
        if !source.cnt().is_null() && !source.cnt().is_empty() {
            if target.cnt().is_null() || target.cnt().is_empty() {
                target.set_cnt(source.cnt().clone());
            } else {
                match target.cnt().add(source.cnt()) {
                    Ok(new_cnt) => target.set_cnt(new_cnt),
                    Err(_) => {}
                }
            }
        }

        // 合并 SUM
        if !source.sum().is_null() && !source.sum().is_empty() {
            if target.sum().is_null() || target.sum().is_empty() {
                target.set_sum(source.sum().clone());
            } else {
                match target.sum().add(source.sum()) {
                    Ok(new_sum) => target.set_sum(new_sum),
                    Err(_) => {}
                }
            }
        }

        // 合并 MAX
        if !source.result().is_null() && !source.result().is_empty() {
            if target.result().is_null() || target.result().is_empty() {
                target.set_result(source.result().clone());
            } else if source.result() > target.result() {
                target.set_result(source.result().clone());
            }
        }

        // 合并去重集合
        if let Some(source_uniques) = source.uniques() {
            if target.uniques().is_none() {
                target.set_uniques(source_uniques.clone());
            } else if let Some(target_uniques) = target.uniques_mut() {
                for val in source_uniques {
                    target_uniques.insert(val.clone());
                }
            }
        }

        Ok(())
    }
}

/// AggregateExecutor - 聚合执行器
///
/// 执行聚合操作，支持 COUNT, SUM, AVG, MAX, MIN 等聚合函数
/// CPU 密集型操作，使用 Rayon 进行并行化
pub struct AggregateExecutor<S: StorageClient + Send + 'static> {
    /// 基础处理器
    base: BaseResultProcessor<S>,
    /// 聚合函数列表
    aggregate_functions: Vec<AggregateFunctionSpec>,
    /// 分组键列表
    group_keys: Vec<Expression>,
    /// 输入执行器
    input_executor: Option<Box<ExecutorEnum<S>>>,
    /// 并行计算配置
    parallel_config: ParallelConfig,
    /// 聚合函数管理器
    agg_function_manager: AggFunctionManager,
}

impl<S: StorageClient> AggregateExecutor<S> {
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
            aggregate_functions: aggregate_functions.clone(),
            group_keys,
            input_executor: None,
            parallel_config: ParallelConfig::default(),
            agg_function_manager: AggFunctionManager::new(),
        }
    }

    /// 设置并行计算配置
    pub fn with_parallel_config(mut self, config: ParallelConfig) -> Self {
        self.parallel_config = config;
        self
    }

    fn process_input(&mut self) -> DBResult<crate::core::value::DataSet> {
        if let Some(ref mut input_exec) = self.input_executor {
            let input_result = input_exec.execute()?;

            match input_result {
                ExecutionResult::DataSet(dataset) => self.aggregate_dataset(dataset),
                _ => Err(crate::core::error::DBError::Query(
                    crate::core::error::QueryError::ExecutionError(
                        "Aggregate executor expects DataSet input".to_string(),
                    ),
                )),
            }
        } else if let Some(input) = &self.base.input {
            match input {
                ExecutionResult::DataSet(dataset) => self.aggregate_dataset(dataset.clone()),
                _ => Err(crate::core::error::DBError::Query(
                    crate::core::error::QueryError::ExecutionError(
                        "Aggregate executor expects DataSet input".to_string(),
                    ),
                ))
            }
        } else {
            Err(crate::core::error::DBError::Query(
                crate::core::error::QueryError::ExecutionError(
                    "Aggregate executor requires input executor".to_string(),
                ),
            ))
        }
    }

    fn aggregate_dataset(
        &mut self,
        dataset: crate::core::value::DataSet,
    ) -> DBResult<crate::core::value::DataSet> {
        let total_size = dataset.rows.len();

        // 处理 COUNT(*) 的特殊情况（无分组键且只有一个 COUNT(*)）
        if self.group_keys.is_empty() 
            && self.aggregate_functions.len() == 1 
            && matches!(self.aggregate_functions[0].function, AggregateFunction::Count(None)) {
            return self.handle_count_star(dataset);
        }

        if self.parallel_config.should_use_parallel(total_size) {
            self.aggregate_dataset_parallel(dataset)
        } else {
            self.aggregate_dataset_serial(dataset)
        }
    }

    /// 处理 COUNT(*) 特殊情况
    fn handle_count_star(
        &self,
        dataset: crate::core::value::DataSet,
    ) -> DBResult<crate::core::value::DataSet> {
        let mut result_dataset = crate::core::value::DataSet::new();
        result_dataset.col_names.push("count".to_string());
        result_dataset.rows.push(vec![Value::Int(dataset.rows.len() as i64)]);
        Ok(result_dataset)
    }

    fn aggregate_dataset_serial(
        &mut self,
        dataset: crate::core::value::DataSet,
    ) -> DBResult<crate::core::value::DataSet> {
        let agg_func_count = self.aggregate_functions.len();
        let mut group_state = GroupAggregateState::new(agg_func_count);

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
            let group_key: Vec<Value> = self.group_keys
                .iter()
                .map(|expr| {
                    ExpressionEvaluator::evaluate(expr, &mut context)
                        .unwrap_or(Value::Null(NullType::NaN))
                })
                .collect();

            // 获取或创建聚合数据
            let agg_data_list = group_state.get_or_create_agg_data(group_key);

            // 对每个聚合函数进行求值
            for (i, agg_func) in self.aggregate_functions.iter().enumerate() {
                if i >= agg_data_list.len() {
                    continue;
                }

                let agg_data = &mut agg_data_list[i];
                let value = self.get_value_for_agg(&mut context, agg_func, row, &dataset.col_names);

                // 获取聚合函数并执行
                let func_name = agg_func.agg_function_name();
                if let Some(agg_fn) = self.agg_function_manager.get(&func_name) {
                    agg_fn(agg_data, &value)?;
                }
            }
        }

        // 构建结果数据集
        self.build_result_dataset(group_state)
    }

    /// 获取聚合函数需要的值
    fn get_value_for_agg(
        &self,
        context: &mut DefaultExpressionContext,
        agg_func: &AggregateFunctionSpec,
        row: &[Value],
        col_names: &[String],
    ) -> Value {
        match &agg_func.function {
            AggregateFunction::Count(None) => {
                // COUNT(*) - 计数 1
                Value::Int(1)
            }
            AggregateFunction::Count(Some(field)) | 
            AggregateFunction::Sum(field) |
            AggregateFunction::Avg(field) |
            AggregateFunction::Max(field) |
            AggregateFunction::Min(field) |
            AggregateFunction::Collect(field) |
            AggregateFunction::CollectSet(field) |
            AggregateFunction::Distinct(field) |
            AggregateFunction::Std(field) |
            AggregateFunction::BitAnd(field) |
            AggregateFunction::BitOr(field) => {
                // 从上下文中获取字段值
                if let Some(val) = context.get_variable(field) {
                    val.clone()
                } else if let Some(col_index) = col_names.iter().position(|name| name == field) {
                    if col_index < row.len() {
                        row[col_index].clone()
                    } else {
                        Value::Null(NullType::Null)
                    }
                } else {
                    Value::Null(NullType::Null)
                }
            }
            AggregateFunction::Percentile(field, _) => {
                if let Some(val) = context.get_variable(field) {
                    val.clone()
                } else if let Some(col_index) = col_names.iter().position(|name| name == field) {
                    if col_index < row.len() {
                        row[col_index].clone()
                    } else {
                        Value::Null(NullType::Null)
                    }
                } else {
                    Value::Null(NullType::Null)
                }
            }
            AggregateFunction::GroupConcat(field, _) => {
                if let Some(val) = context.get_variable(field) {
                    val.clone()
                } else if let Some(col_index) = col_names.iter().position(|name| name == field) {
                    if col_index < row.len() {
                        row[col_index].clone()
                    } else {
                        Value::Null(NullType::Null)
                    }
                } else {
                    Value::Null(NullType::Null)
                }
            }
        }
    }

    /// 并行聚合
    fn aggregate_dataset_parallel(
        &mut self,
        dataset: crate::core::value::DataSet,
    ) -> DBResult<crate::core::value::DataSet> {
        let batch_size = self.parallel_config.calculate_batch_size(dataset.rows.len());
        let aggregate_functions = self.aggregate_functions.clone();
        let group_keys = self.group_keys.clone();
        let col_names = dataset.col_names.clone();
        let agg_function_manager = self.agg_function_manager.clone();

        // 使用 rayon 并行处理数据批次
        let partial_results: Vec<GroupAggregateState> = dataset
            .rows
            .par_chunks(batch_size)
            .map(|chunk| {
                let agg_func_count = aggregate_functions.len();
                let mut local_state = GroupAggregateState::new(agg_func_count);

                for row in chunk {
                    // 构建表达式上下文
                    let mut context = DefaultExpressionContext::new();
                    for (i, col_name) in col_names.iter().enumerate() {
                        if i < row.len() {
                            context.set_variable(col_name.clone(), row[i].clone());
                        }
                    }

                    // 计算分组键
                    let group_key: Vec<Value> = group_keys
                        .iter()
                        .map(|expr| {
                            ExpressionEvaluator::evaluate(expr, &mut context)
                                .unwrap_or(Value::Null(NullType::NaN))
                        })
                        .collect();

                    // 获取或创建聚合数据
                    let agg_data_list = local_state.get_or_create_agg_data(group_key);

                    // 对每个聚合函数进行求值
                    for (i, agg_func) in aggregate_functions.iter().enumerate() {
                        if i >= agg_data_list.len() {
                            continue;
                        }

                        let agg_data = &mut agg_data_list[i];
                        
                        // 获取值
                        let value = match &agg_func.function {
                            AggregateFunction::Count(None) => Value::Int(1),
                            AggregateFunction::Count(Some(field)) |
                            AggregateFunction::Sum(field) |
                            AggregateFunction::Avg(field) |
                            AggregateFunction::Max(field) |
                            AggregateFunction::Min(field) |
                            AggregateFunction::Collect(field) |
                            AggregateFunction::CollectSet(field) |
                            AggregateFunction::Distinct(field) |
                            AggregateFunction::Std(field) |
                            AggregateFunction::BitAnd(field) |
                            AggregateFunction::BitOr(field) => {
                                if let Some(col_index) = col_names.iter().position(|name| name == field) {
                                    if col_index < row.len() {
                                        row[col_index].clone()
                                    } else {
                                        Value::Null(NullType::Null)
                                    }
                                } else {
                                    Value::Null(NullType::Null)
                                }
                            }
                            _ => Value::Null(NullType::Null),
                        };

                        // 获取聚合函数并执行
                        let func_name = agg_func.agg_function_name();
                        if let Some(agg_fn) = agg_function_manager.get(&func_name) {
                            let _ = agg_fn(agg_data, &value);
                        }
                    }
                }

                local_state
            })
            .collect();

        // Gather: 合并所有局部聚合结果
        let agg_func_count = self.aggregate_functions.len();
        let mut global_state = GroupAggregateState::new(agg_func_count);
        for partial_state in partial_results {
            global_state.merge(partial_state)?;
        }

        // 构建结果数据集
        self.build_result_dataset(global_state)
    }

    fn build_result_dataset(
        &self,
        group_state: GroupAggregateState,
    ) -> DBResult<crate::core::value::DataSet> {
        let mut result_dataset = crate::core::value::DataSet::new();

        // 设置列名
        for _ in &self.group_keys {
            result_dataset
                .col_names
                .push(format!("group_{}", result_dataset.col_names.len()));
        }

        for agg_func in &self.aggregate_functions {
            let col_name = match &agg_func.function {
                AggregateFunction::Count(_) => {
                    if agg_func.distinct {
                        if let Some(ref field) = agg_func.field {
                            format!("count_distinct_{}", field)
                        } else {
                            "count_distinct".to_string()
                        }
                    } else if let Some(ref field) = agg_func.field {
                        format!("count_{}", field)
                    } else {
                        "count".to_string()
                    }
                }
                AggregateFunction::Sum(_) => {
                    if let Some(ref field) = agg_func.field {
                        format!("sum_{}", field)
                    } else {
                        "sum".to_string()
                    }
                }
                AggregateFunction::Avg(_) => {
                    if let Some(ref field) = agg_func.field {
                        format!("avg_{}", field)
                    } else {
                        "avg".to_string()
                    }
                }
                AggregateFunction::Max(_) => {
                    if let Some(ref field) = agg_func.field {
                        format!("max_{}", field)
                    } else {
                        "max".to_string()
                    }
                }
                AggregateFunction::Min(_) => {
                    if let Some(ref field) = agg_func.field {
                        format!("min_{}", field)
                    } else {
                        "min".to_string()
                    }
                }
                AggregateFunction::Collect(_) => {
                    if let Some(ref field) = agg_func.field {
                        format!("collect_{}", field)
                    } else {
                        "collect".to_string()
                    }
                }
                AggregateFunction::CollectSet(_) => {
                    if let Some(ref field) = agg_func.field {
                        format!("collect_set_{}", field)
                    } else {
                        "collect_set".to_string()
                    }
                }
                AggregateFunction::Distinct(_) => {
                    if let Some(ref field) = agg_func.field {
                        format!("distinct_{}", field)
                    } else {
                        "distinct".to_string()
                    }
                }
                AggregateFunction::Percentile(_, _) => {
                    if let Some(ref field) = agg_func.field {
                        format!("percentile_{}", field)
                    } else {
                        "percentile".to_string()
                    }
                }
                AggregateFunction::Std(_) => {
                    if let Some(ref field) = agg_func.field {
                        format!("std_{}", field)
                    } else {
                        "std".to_string()
                    }
                }
                AggregateFunction::BitAnd(_) => {
                    if let Some(ref field) = agg_func.field {
                        format!("bitand_{}", field)
                    } else {
                        "bitand".to_string()
                    }
                }
                AggregateFunction::BitOr(_) => {
                    if let Some(ref field) = agg_func.field {
                        format!("bitor_{}", field)
                    } else {
                        "bitor".to_string()
                    }
                }
                AggregateFunction::GroupConcat(_, _) => {
                    if let Some(ref field) = agg_func.field {
                        format!("group_concat_{}", field)
                    } else {
                        "group_concat".to_string()
                    }
                }
            };
            result_dataset.col_names.push(col_name);
        }

        // 填充结果行
        for (group_key, agg_data_list) in &group_state.groups {
            let mut result_row = Vec::new();

            // 添加分组键值
            result_row.extend_from_slice(group_key);

            // 添加聚合结果
            for (i, agg_func) in self.aggregate_functions.iter().enumerate() {
                if i < agg_data_list.len() {
                    let agg_data = &agg_data_list[i];
                    
                    // 处理 COUNT DISTINCT 特殊情况
                    let agg_value = if agg_func.distinct && matches!(agg_func.function, AggregateFunction::Count(_)) {
                        // 使用去重集合的大小作为 COUNT DISTINCT 结果
                        if let Some(uniques) = agg_data.uniques() {
                            Value::Int(uniques.len() as i64)
                        } else {
                            Value::Int(0)
                        }
                    } else {
                        agg_data.result().clone()
                    };
                    
                    result_row.push(agg_value);
                } else {
                    result_row.push(Value::Null(NullType::NaN));
                }
            }

            result_dataset.rows.push(result_row);
        }

        Ok(result_dataset)
    }
}

impl<S: StorageClient + Send + 'static> ResultProcessor<S> for AggregateExecutor<S> {
    fn process(&mut self, input: ExecutionResult) -> DBResult<ExecutionResult> {
        ResultProcessor::set_input(self, input);
        let dataset = self.process_input()?;
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

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for AggregateExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute()?
        } else {
            self.base
                .input
                .clone()
                .unwrap_or(ExecutionResult::DataSet(crate::core::value::DataSet::new()))
        };

        self.process(input_result)
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

impl<S: StorageClient + Send + 'static> InputExecutor<S> for AggregateExecutor<S> {
    fn set_input(&mut self, input: ExecutorEnum<S>) {
        self.input_executor = Some(Box::new(input));
    }

    fn get_input(&self) -> Option<&ExecutorEnum<S>> {
        self.input_executor.as_deref()
    }
}

/// GroupByExecutor - 分组聚合执行器
///
/// 实现 GROUP BY 操作
pub struct GroupByExecutor<S: StorageClient + Send + 'static> {
    aggregate_executor: AggregateExecutor<S>,
}

impl<S: StorageClient + Send + 'static> GroupByExecutor<S> {
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

impl<S: StorageClient + Send + 'static> InputExecutor<S> for GroupByExecutor<S> {
    fn set_input(&mut self, input: ExecutorEnum<S>) {
        InputExecutor::set_input(&mut self.aggregate_executor, input);
    }

    fn get_input(&self) -> Option<&ExecutorEnum<S>> {
        InputExecutor::get_input(&self.aggregate_executor)
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for GroupByExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        self.aggregate_executor.execute()
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
pub struct HavingExecutor<S: StorageClient + Send + 'static> {
    /// 基础处理器
    base: BaseResultProcessor<S>,
    /// HAVING 条件表达式
    condition: Expression,
    /// 输入执行器
    input_executor: Option<Box<ExecutorEnum<S>>>,
}

impl<S: StorageClient> HavingExecutor<S> {
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

    fn process_input(&mut self) -> DBResult<crate::core::value::DataSet> {
        if let Some(ref mut input_exec) = self.input_executor {
            let input_result = input_exec.execute()?;

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
        } else if let Some(input) = &self.base.input {
            match input {
                ExecutionResult::DataSet(dataset) => {
                    let mut dataset = dataset.clone();
                    self.apply_having_condition(&mut dataset)?;
                    Ok(dataset)
                }
                _ => Err(crate::core::error::DBError::Query(
                    crate::core::error::QueryError::ExecutionError(
                        "Having executor expects DataSet input".to_string(),
                    ),
                ))
            }
        } else {
            Err(crate::core::error::DBError::Query(
                crate::core::error::QueryError::ExecutionError(
                    "Having executor requires input executor".to_string(),
                ),
            ))
        }
    }

    fn apply_having_condition(
        &self,
        dataset: &mut crate::core::value::DataSet,
    ) -> DBResult<()> {
        let mut filtered_rows = Vec::new();

        for row in &dataset.rows {
            // 构建表达式上下文
            let mut context = DefaultExpressionContext::new();
            for (i, col_name) in dataset.col_names.iter().enumerate() {
                if i < row.len() {
                    context.set_variable(col_name.clone(), row[i].clone());
                }
            }

            // 评估 HAVING 条件
            match ExpressionEvaluator::evaluate(&self.condition, &mut context) {
                Ok(Value::Bool(true)) => {
                    filtered_rows.push(row.clone());
                }
                Ok(Value::Bool(false)) => {
                    // 条件为 false，跳过该行
                }
                Ok(_) => {
                    // 非布尔值，视为 false
                }
                Err(e) => {
                    return Err(crate::core::error::DBError::Expression(
                        crate::core::error::ExpressionError::function_error(format!(
                            "Failed to evaluate HAVING condition: {}",
                            e
                        )),
                    ));
                }
            }
        }

        dataset.rows = filtered_rows;
        Ok(())
    }
}

impl<S: StorageClient + Send + 'static> ResultProcessor<S> for HavingExecutor<S> {
    fn process(&mut self, input: ExecutionResult) -> DBResult<ExecutionResult> {
        ResultProcessor::set_input(self, input);
        let dataset = self.process_input()?;
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

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for HavingExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute()?
        } else {
            self.base
                .input
                .clone()
                .unwrap_or(ExecutionResult::DataSet(crate::core::value::DataSet::new()))
        };

        self.process(input_result)
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

impl<S: StorageClient + Send + 'static> InputExecutor<S> for HavingExecutor<S> {
    fn set_input(&mut self, input: ExecutorEnum<S>) {
        self.input_executor = Some(Box::new(input));
    }

    fn get_input(&self) -> Option<&ExecutorEnum<S>> {
        self.input_executor.as_deref()
    }
}
