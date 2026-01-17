//! 聚合功能性能测试

#[cfg(feature = "redb")]
use crate::storage::redb_storage::RedbStorage;
use crate::core::types::operators::AggregateFunction;
use crate::core::Value;
use crate::query::executor::aggregation::AggregationExecutor;
use crate::query::executor::base::BaseExecutor;
use std::sync::{Arc, Mutex};

#[cfg(feature = "redb")]
/// 性能测试函数
pub fn benchmark_aggregation_performance() {
    println!("开始聚合功能性能测试...");
    
    // 创建测试数据
    let test_data = create_test_data(10000); // 10,000条测试数据
    
    // 测试COUNT聚合
    let start_time = std::time::Instant::now();
    let count_result = execute_count_aggregation(&test_data);
    let count_duration = start_time.elapsed();
    
    println!("COUNT聚合执行时间: {:?}", count_duration);
    println!("COUNT结果: {:?}", count_result);
    
    // 测试SUM聚合
    let start_time = std::time::Instant::now();
    let sum_result = execute_sum_aggregation(&test_data);
    let sum_duration = start_time.elapsed();
    
    println!("SUM聚合执行时间: {:?}", sum_duration);
    println!("SUM结果: {:?}", sum_result);
    
    // 测试AVG聚合
    let start_time = std::time::Instant::now();
    let avg_result = execute_avg_aggregation(&test_data);
    let avg_duration = start_time.elapsed();
    
    println!("AVG聚合执行时间: {:?}", avg_duration);
    println!("AVG结果: {:?}", avg_result);
    
    println!("聚合功能性能测试完成");
}

/// 创建测试数据
fn create_test_data(size: usize) -> Vec<Value> {
    let mut data = Vec::with_capacity(size);
    
    for i in 0..size {
        // 创建包含数值的映射作为测试数据
        let mut map = std::collections::HashMap::new();
        map.insert("value".to_string(), Value::Int(i as i64));
        map.insert("category".to_string(), Value::String((i % 10).to_string()));
        
        data.push(Value::Map(map));
    }
    
    data
}

/// 执行COUNT聚合
fn execute_count_aggregation(data: &[Value]) -> Value {
    // 模拟COUNT(*)操作
    Value::Int(data.len() as i64)
}

/// 执行SUM聚合
fn execute_sum_aggregation(data: &[Value]) -> Value {
    let mut sum = 0.0;
    
    for value in data {
        if let Value::Map(ref map) = value {
            if let Some(Value::Int(num)) = map.get("value") {
                sum += *num as f64;
            }
        }
    }
    
    Value::Float(sum)
}

/// 执行AVG聚合
fn execute_avg_aggregation(data: &[Value]) -> Value {
    if data.is_empty() {
        return Value::Null(crate::core::value::NullType::NaN);
    }
    
    let mut sum = 0.0;
    let mut count = 0;
    
    for value in data {
        if let Value::Map(ref map) = value {
            if let Some(Value::Int(num)) = map.get("value") {
                sum += *num as f64;
                count += 1;
            }
        }
    }
    
    if count == 0 {
        Value::Null(crate::core::value::NullType::NaN)
    } else {
        Value::Float(sum / count as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregation_performance() {
        benchmark_aggregation_performance();
    }
}