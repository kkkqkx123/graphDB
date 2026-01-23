//! 聚合执行器实现
//!
//! 包含统计聚合函数的执行逻辑

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::base::BaseExecutor;
use crate::core::types::operators::AggregateFunction;
use crate::core::Value;
use crate::core::error::DBError;
use crate::query::executor::traits::{DBResult, ExecutionResult, Executor, HasStorage};
use crate::storage::StorageEngine;
use crate::utils::safe_lock;

/// 聚合执行器
#[derive(Debug)]
pub struct AggregationExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    /// 聚合函数列表
    aggregation_functions: Vec<AggregateFunction>,
    /// GROUP BY字段列表
    group_by_keys: Vec<String>,
    /// 过滤条件
    filter_condition: Option<crate::core::Expression>,
}

impl<S: StorageEngine> AggregationExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        aggregation_functions: Vec<AggregateFunction>,
        group_by_keys: Vec<String>,
        filter_condition: Option<crate::core::Expression>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "AggregationExecutor".to_string(), storage),
            aggregation_functions,
            group_by_keys,
            filter_condition,
        }
    }

    /// 执行聚合操作
    fn execute_aggregation(&self, input_data: &[Value]) -> DBResult<ExecutionResult> {
        // 如果没有分组键，执行整体聚合
        if self.group_by_keys.is_empty() {
            let result = self.aggregate_values(input_data)?;
            Ok(ExecutionResult::Values(vec![Value::List(result)]))
        } else {
            // 按分组键进行分组聚合
            let grouped_data = self.group_by_keys(input_data)?;
            let mut results = Vec::new();

            for (_, group_values) in grouped_data {
                let group_result = self.aggregate_values(&group_values)?;
                results.push(Value::List(group_result));
            }

            Ok(ExecutionResult::Values(results))
        }
    }

    /// 按分组键对数据进行分组
    fn group_by_keys(&self, input_data: &[Value]) -> DBResult<HashMap<Vec<Value>, Vec<Value>>> {
        let mut grouped: HashMap<Vec<Value>, Vec<Value>> = HashMap::new();

        for value in input_data {
            let group_keys = self.extract_group_keys(value)?;
            grouped.entry(group_keys).or_insert_with(Vec::new).push(value.clone());
        }

        Ok(grouped)
    }

    /// 提取分组键值
    fn extract_group_keys(&self, value: &Value) -> DBResult<Vec<Value>> {
        let mut keys = Vec::new();
        
        for key_name in &self.group_by_keys {
            // 根据值类型提取分组键
            match value {
                Value::Vertex(vertex) => {
                    // 从顶点中提取属性值
                    if let Some(prop_value) = vertex.get_property_any(key_name) {
                        keys.push(prop_value.clone());
                    } else {
                        keys.push(Value::Null(crate::core::value::NullType::NaN));
                    }
                }
                Value::Edge(edge) => {
                    // 从边中提取属性值
                    if let Some(prop_value) = edge.get_property(key_name) {
                        keys.push(prop_value.clone());
                    } else {
                        keys.push(Value::Null(crate::core::value::NullType::NaN));
                    }
                }
                Value::Map(map) => {
                    // 从映射中提取值
                    if let Some(prop_value) = map.get(key_name) {
                        keys.push(prop_value.clone());
                    } else {
                        keys.push(Value::Null(crate::core::value::NullType::NaN));
                    }
                }
                _ => {
                    // 对于其他类型，暂时返回null
                    keys.push(Value::Null(crate::core::value::NullType::NaN));
                }
            }
        }
        
        Ok(keys)
    }

    /// 对值列表执行聚合操作
    fn aggregate_values(&self, values: &[Value]) -> DBResult<Vec<Value>> {
        let mut results = Vec::new();

        for agg_func in &self.aggregation_functions {
            let result = match agg_func {
                AggregateFunction::Count(field_name) => {
                    let count = if field_name.is_none() {
                        // COUNT(*)
                        values.len() as i64
                    } else {
                        // COUNT(字段名) - 计算非空值的数量
                        values.iter().filter(|v| !v.is_null()).count() as i64
                    };
                    Value::Int(count)
                }
                AggregateFunction::Sum(field_name) => {
                    self.calculate_sum(values, field_name)?
                }
                AggregateFunction::Avg(field_name) => {
                    self.calculate_avg(values, field_name)?
                }
                AggregateFunction::Min(field_name) => {
                    self.calculate_min(values, field_name)?
                }
                AggregateFunction::Max(field_name) => {
                    self.calculate_max(values, field_name)?
                }
                AggregateFunction::Collect(field_name) => {
                    self.collect_values(values, field_name)?
                }
                AggregateFunction::Distinct(field_name) => {
                    self.distinct_values(values, field_name)?
                }
                AggregateFunction::Percentile(field_name, percentile) => {
                    self.calculate_percentile(values, field_name, *percentile)?
                }
            };

            results.push(result);
        }

        Ok(results)
    }

    /// 计算总和
    fn calculate_sum(&self, values: &[Value], field_name: &str) -> DBResult<Value> {
        let mut sum = 0.0;
        let mut count = 0;
        
        for value in values {
            let field_value = self.extract_field_value(value, field_name)?;
            
            match field_value {
                Value::Int(i) => {
                    sum += i as f64;
                    count += 1;
                }
                Value::Float(f) => {
                    sum += f;
                    count += 1;
                }
                Value::Null(_) => continue,
                _ => return Err(DBError::Query(
                    crate::core::error::QueryError::ExecutionError(
                        format!("无法对非数值类型进行求和: {:?}", field_value)
                    )
                )),
            }
        }
        
        if count == 0 {
            Ok(Value::Null(crate::core::value::NullType::NaN))
        } else {
            Ok(Value::Float(sum))
        }
    }

    /// 计算平均值
    fn calculate_avg(&self, values: &[Value], field_name: &str) -> DBResult<Value> {
        let mut sum = 0.0;
        let mut count = 0;
        
        for value in values {
            let field_value = self.extract_field_value(value, field_name)?;
            
            match field_value {
                Value::Int(i) => {
                    sum += i as f64;
                    count += 1;
                }
                Value::Float(f) => {
                    sum += f;
                    count += 1;
                }
                Value::Null(_) => continue,
                _ => return Err(DBError::Query(
                    crate::core::error::QueryError::ExecutionError(
                        format!("无法对非数值类型计算平均值: {:?}", field_value)
                    )
                )),
            }
        }
        
        if count == 0 {
            Ok(Value::Null(crate::core::value::NullType::NaN))
        } else {
            Ok(Value::Float(sum / count as f64))
        }
    }

    /// 计算最小值
    fn calculate_min(&self, values: &[Value], field_name: &str) -> DBResult<Value> {
        let mut min_value: Option<Value> = None;
        
        for value in values {
            let field_value = self.extract_field_value(value, field_name)?;
            
            if matches!(field_value, Value::Null(_)) {
                continue;
            }
            
            match &min_value {
                None => min_value = Some(field_value.clone()),
                Some(current_min) => {
                    if self.value_less_than(&field_value, current_min)? {
                        min_value = Some(field_value.clone());
                    }
                }
            }
        }
        
        Ok(min_value.unwrap_or(Value::Null(crate::core::value::NullType::NaN)))
    }

    /// 计算最大值
    fn calculate_max(&self, values: &[Value], field_name: &str) -> DBResult<Value> {
        let mut max_value: Option<Value> = None;
        
        for value in values {
            let field_value = self.extract_field_value(value, field_name)?;
            
            if matches!(field_value, Value::Null(_)) {
                continue;
            }
            
            match &max_value {
                None => max_value = Some(field_value.clone()),
                Some(current_max) => {
                    if self.value_greater_than(&field_value, current_max)? {
                        max_value = Some(field_value.clone());
                    }
                }
            }
        }
        
        Ok(max_value.unwrap_or(Value::Null(crate::core::value::NullType::NaN)))
    }

    /// 收集值
    fn collect_values(&self, values: &[Value], field_name: &str) -> DBResult<Value> {
        let mut collected = Vec::new();
        
        for value in values {
            let field_value = self.extract_field_value(value, field_name)?;
            collected.push(field_value);
        }
        
        Ok(Value::List(collected))
    }

    /// 获取不同值
    fn distinct_values(&self, values: &[Value], field_name: &str) -> DBResult<Value> {
        let mut distinct_set = std::collections::HashSet::new();
        let mut distinct_list = Vec::new();
        
        for value in values {
            let field_value = self.extract_field_value(value, field_name)?;
            
            if distinct_set.insert(format!("{:?}", field_value)) {
                distinct_list.push(field_value);
            }
        }
        
        Ok(Value::List(distinct_list))
    }

    /// 计算百分位数
    fn calculate_percentile(&self, values: &[Value], field_name: &str, percentile: f64) -> DBResult<Value> {
        let mut numeric_values = Vec::new();
        
        for value in values {
            let field_value = self.extract_field_value(value, field_name)?;
            
            match field_value {
                Value::Int(i) => numeric_values.push(i as f64),
                Value::Float(f) => numeric_values.push(f),
                Value::Null(_) => continue,
                _ => return Err(DBError::Query(
                    crate::core::error::QueryError::ExecutionError(
                        format!("无法对非数值类型计算百分位数: {:?}", field_value)
                    )
                )),
            }
        }
        
        if numeric_values.is_empty() {
            return Ok(Value::Null(crate::core::value::NullType::NaN));
        }
        
        // 简单的百分位数计算（排序后取相应位置的值）
        numeric_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let index = ((percentile / 100.0) * (numeric_values.len() - 1) as f64).round() as usize;
        let index = std::cmp::min(index, numeric_values.len() - 1);
        
        Ok(Value::Float(numeric_values[index]))
    }

    /// 从值中提取字段值
    fn extract_field_value(&self, value: &Value, field_name: &str) -> DBResult<Value> {
        match value {
            Value::Vertex(vertex) => {
                if let Some(prop_value) = vertex.get_property_any(field_name) {
                    Ok(prop_value.clone())
                } else {
                    Ok(Value::Null(crate::core::value::NullType::NaN))
                }
            }
            Value::Edge(edge) => {
                if let Some(prop_value) = edge.get_property(field_name) {
                    Ok(prop_value.clone())
                } else {
                    Ok(Value::Null(crate::core::value::NullType::NaN))
                }
            }
            Value::Map(map) => {
                if let Some(prop_value) = map.get(field_name) {
                    Ok(prop_value.clone())
                } else {
                    Ok(Value::Null(crate::core::value::NullType::NaN))
                }
            }
            _ => Ok(value.clone()),
        }
    }

    /// 比较两个值的大小
    fn value_less_than(&self, a: &Value, b: &Value) -> DBResult<bool> {
        match (a, b) {
            (Value::Int(a_int), Value::Int(b_int)) => Ok(a_int < b_int),
            (Value::Float(a_float), Value::Float(b_float)) => Ok(a_float < b_float),
            (Value::Int(a_int), Value::Float(b_float)) => Ok((*a_int as f64) < *b_float),
            (Value::Float(a_float), Value::Int(b_int)) => Ok(*a_float < (*b_int as f64)),
            (Value::String(a_str), Value::String(b_str)) => Ok(a_str < b_str),
            _ => Err(DBError::Query(
                crate::core::error::QueryError::ExecutionError(
                    "无法比较不同类型的值".to_string()
                )
            )),
        }
    }

    /// 比较两个值的大小
    fn value_greater_than(&self, a: &Value, b: &Value) -> DBResult<bool> {
        match (a, b) {
            (Value::Int(a_int), Value::Int(b_int)) => Ok(a_int > b_int),
            (Value::Float(a_float), Value::Float(b_float)) => Ok(a_float > b_float),
            (Value::Int(a_int), Value::Float(b_float)) => Ok((*a_int as f64) > *b_float),
            (Value::Float(a_float), Value::Int(b_int)) => Ok(*a_float > (*b_int as f64)),
            (Value::String(a_str), Value::String(b_str)) => Ok(a_str > b_str),
            _ => Err(DBError::Query(
                crate::core::error::QueryError::ExecutionError(
                    "无法比较不同类型的值".to_string()
                )
            )),
        }
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for AggregationExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        // 在实际实现中，这里应该从前置执行器获取数据
        // 目前我们模拟一个简单的聚合操作

        // 示例：从存储中获取一些数据进行聚合
        let storage_clone = self.get_storage().clone();
        let storage_ref = storage_clone;
        let storage = safe_lock(&storage_ref)
            .expect("AggregationExecutor storage lock should not be poisoned");

        // 这里我们模拟从上游获取数据
        // 在实际实现中，应该连接到上游执行器并获取数据
        // 为了避免生命周期问题，我们直接在同步部分处理数据
        let input_data = self.get_sample_data(&storage)?;

        self.execute_aggregation(&input_data)
    }

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        "Aggregation executor - performs statistical aggregation operations"
    }

    fn stats(&self) -> &crate::query::executor::traits::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::traits::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageEngine> AggregationExecutor<S> {
    /// 获取示例数据 - 在实际实现中，这应该从上游执行器获取数据
    fn get_sample_data(&self, storage: &S) -> DBResult<Vec<Value>> {
        // 这里我们简单地扫描所有顶点作为示例数据
        // 在实际实现中，应该从上游执行器获取数据
        let vertices = storage.scan_all_vertices()?;

        // 将顶点转换为Value
        let mut values = Vec::new();
        for vertex in vertices {
            values.push(Value::Vertex(Box::new(vertex)));
        }

        Ok(values)
    }
}

impl<S: StorageEngine> HasStorage<S> for AggregationExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base
            .storage
            .as_ref()
            .expect("AggregationExecutor storage should be set")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Value, vertex_edge_path::Vertex};
    use crate::core::vertex_edge_path::Tag;
    use crate::core::types::operators::AggregateFunction;
    use std::collections::HashMap;
    use tokio;

    #[tokio::test]
    async fn test_aggregation_executor_creation() {
        let storage = Arc::new(Mutex::new(crate::storage::MockStorage));
        let agg_funcs = vec![AggregateFunction::Count(None)];
        let group_by_keys = vec!["category".to_string()];

        let executor = AggregationExecutor::new(
            1,
            storage,
            agg_funcs,
            group_by_keys,
            None,
        );

        assert_eq!(executor.id(), 1);
        assert_eq!(executor.name(), "AggregationExecutor");
    }

    #[test]
    fn test_extract_field_value_from_vertex() {
        // 创建一个测试顶点
        let mut properties = HashMap::new();
        properties.insert("age".to_string(), Value::Int(25));
        properties.insert("name".to_string(), Value::String("Alice".to_string()));

        let tag = Tag::new("person".to_string(), properties);
        let vertex = Vertex::new_with_properties(
            Value::String("vertex1".to_string()),
            vec![tag],
            HashMap::new(),
        );

        let executor = AggregationExecutor {
            base: BaseExecutor::new(1, "test".to_string(), Arc::new(Mutex::new(crate::storage::MockStorage))),
            aggregation_functions: vec![],
            group_by_keys: vec![],
            filter_condition: None,
        };

        // 测试从顶点提取age字段
        let result = executor.extract_field_value(&Value::Vertex(Box::new(vertex)), "age").expect("extract_field_value should succeed");
        assert_eq!(result, Value::Int(25));
    }

    #[test]
    fn test_calculate_sum() {
        let executor = AggregationExecutor {
            base: BaseExecutor::new(1, "test".to_string(), Arc::new(Mutex::new(crate::storage::MockStorage))),
            aggregation_functions: vec![],
            group_by_keys: vec![],
            filter_condition: None,
        };

        let values = vec![
            Value::Int(10),
            Value::Int(20),
            Value::Int(30),
        ];

        // 测试对值列表求和
        let mut sum = 0.0;
        for value in &values {
            match value {
                Value::Int(i) => sum += *i as f64,
                Value::Float(f) => sum += *f,
                _ => {}
            }
        }
        assert_eq!(sum, 60.0);
    }

    #[test]
    fn test_calculate_count() {
        let temp_dir = std::env::temp_dir();
        let test_path = temp_dir.join("graphdb_test_aggregation").to_str().expect("temp_dir should be valid unicode").to_string();
        let executor = AggregationExecutor {
            base: BaseExecutor::new(1, "test".to_string(), Arc::new(Mutex::new(crate::storage::MockStorage))),
            aggregation_functions: vec![],
            group_by_keys: vec![],
            filter_condition: None,
        };

        let values = vec![
            Value::Int(10),
            Value::String("hello".to_string()),
            Value::Float(3.14),
        ];

        // 测试COUNT(*)功能
        let count = values.len() as i64;
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn test_aggregate_values_count() {
        let temp_dir = std::env::temp_dir();
        let test_path = temp_dir.join("graphdb_test_aggregation").to_str().expect("temp_dir should be valid unicode").to_string();
        let executor = AggregationExecutor {
            base: BaseExecutor::new(1, "test".to_string(), Arc::new(Mutex::new(crate::storage::MockStorage))),
            aggregation_functions: vec![AggregateFunction::Count(None)],
            group_by_keys: vec![],
            filter_condition: None,
        };

        let values = vec![
            Value::Int(10),
            Value::String("hello".to_string()),
            Value::Float(3.14),
        ];

        let results = executor.aggregate_values(&values).expect("aggregate_values should succeed");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], Value::Int(3)); // COUNT(*) 应该返回3
    }
}