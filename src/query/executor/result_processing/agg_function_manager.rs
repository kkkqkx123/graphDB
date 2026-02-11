//! 聚合函数管理器模块
//!
//! 参考 nebula-graph 的 AggFunctionManager 设计
//! 统一管理内置聚合函数，支持动态注册和获取

use super::agg_data::AggData;
use crate::core::value::{NullType, Value};
use crate::core::error::{DBError, QueryError};
use std::collections::HashMap;
use std::sync::Arc;

/// 聚合函数类型
pub type AggFunction = Arc<dyn Fn(&mut AggData, &Value) -> Result<(), DBError> + Send + Sync>;

/// 聚合函数管理器
///
/// 管理所有聚合函数，提供统一的获取和执行接口
#[derive(Clone)]
pub struct AggFunctionManager {
    functions: HashMap<String, AggFunction>,
}

impl AggFunctionManager {
    /// 创建新的聚合函数管理器并注册内置函数
    pub fn new() -> Self {
        let mut manager = Self {
            functions: HashMap::new(),
        };
        manager.register_builtin_functions();
        manager
    }

    /// 获取单例实例
    pub fn instance() -> Self {
        Self::new()
    }

    /// 注册内置聚合函数
    fn register_builtin_functions(&mut self) {
        // COUNT 函数
        self.functions.insert(
            "COUNT".to_string(),
            Arc::new(|agg_data: &mut AggData, val: &Value| {
                let res = agg_data.result_mut();
                if res.is_bad_null() {
                    return Ok(());
                }
                if res.is_null() {
                    *res = Value::Int(0);
                }
                if val.is_null() || val.is_empty() {
                    return Ok(());
                }
                if let Value::Int(n) = res {
                    *res = Value::Int(*n + 1);
                }
                Ok(())
            }),
        );

        // SUM 函数
        self.functions.insert(
            "SUM".to_string(),
            Arc::new(|agg_data: &mut AggData, val: &Value| {
                let res = agg_data.result_mut();
                if res.is_bad_null() {
                    return Ok(());
                }
                if !val.is_null() && !val.is_empty() && !val.is_numeric() {
                    *res = Value::Null(NullType::BadType);
                    return Ok(());
                }
                if val.is_null() || val.is_empty() {
                    return Ok(());
                }
                if res.is_null() {
                    *res = val.clone();
                } else {
                    match res.add(val) {
                        Ok(new_val) => *res = new_val,
                        Err(_) => *res = Value::Null(NullType::BadType),
                    }
                }
                Ok(())
            }),
        );

        // AVG 函数
        self.functions.insert(
            "AVG".to_string(),
            Arc::new(|agg_data: &mut AggData, val: &Value| {
                // 首先检查 result 是否为 BadNull
                if agg_data.result().is_bad_null() {
                    return Ok(());
                }
                if !val.is_null() && !val.is_empty() && !val.is_numeric() {
                    *agg_data.result_mut() = Value::Null(NullType::BadType);
                    return Ok(());
                }
                if val.is_null() || val.is_empty() {
                    return Ok(());
                }

                // 初始化
                if agg_data.result().is_null() {
                    *agg_data.result_mut() = Value::Float(0.0);
                    *agg_data.sum_mut() = Value::Float(0.0);
                    *agg_data.cnt_mut() = Value::Float(0.0);
                }

                // 更新 sum
                let sum = agg_data.sum_mut();
                match sum.add(val) {
                    Ok(new_sum) => *sum = new_sum,
                    Err(_) => {
                        *agg_data.result_mut() = Value::Null(NullType::BadType);
                        return Ok(());
                    }
                }

                // 更新 count
                let cnt = agg_data.cnt_mut();
                if let Value::Float(n) = cnt {
                    *cnt = Value::Float(*n + 1.0);
                }

                // 计算 avg = sum / count
                let sum = agg_data.sum().clone();
                let cnt = agg_data.cnt().clone();
                if let Value::Float(c) = cnt {
                    if c > 0.0 {
                        match sum.div(&Value::Float(c)) {
                            Ok(avg_val) => *agg_data.result_mut() = avg_val,
                            Err(_) => *agg_data.result_mut() = Value::Null(NullType::DivByZero),
                        }
                    }
                }
                Ok(())
            }),
        );

        // MAX 函数
        self.functions.insert(
            "MAX".to_string(),
            Arc::new(|agg_data: &mut AggData, val: &Value| {
                let res = agg_data.result_mut();
                if res.is_bad_null() {
                    return Ok(());
                }
                if val.is_null() || val.is_empty() {
                    return Ok(());
                }
                if res.is_null() {
                    *res = val.clone();
                    return Ok(());
                }
                if val > res {
                    *res = val.clone();
                }
                Ok(())
            }),
        );

        // MIN 函数
        self.functions.insert(
            "MIN".to_string(),
            Arc::new(|agg_data: &mut AggData, val: &Value| {
                let res = agg_data.result_mut();
                if res.is_bad_null() {
                    return Ok(());
                }
                if val.is_null() || val.is_empty() {
                    return Ok(());
                }
                if res.is_null() {
                    *res = val.clone();
                    return Ok(());
                }
                if val < res {
                    *res = val.clone();
                }
                Ok(())
            }),
        );

        // STD 函数（标准差）
        self.functions.insert(
            "STD".to_string(),
            Arc::new(|agg_data: &mut AggData, val: &Value| {
                // 首先检查 result 是否为 BadNull
                if agg_data.result().is_bad_null() {
                    return Ok(());
                }
                if !val.is_null() && !val.is_empty() && !val.is_numeric() {
                    *agg_data.result_mut() = Value::Null(NullType::BadType);
                    return Ok(());
                }
                if val.is_null() || val.is_empty() {
                    return Ok(());
                }

                // 获取数值
                let val_f64 = match val {
                    Value::Int(v) => *v as f64,
                    Value::Float(v) => *v,
                    _ => return Ok(()),
                };

                // 初始化
                if agg_data.result().is_null() {
                    *agg_data.result_mut() = Value::Float(0.0);
                    *agg_data.cnt_mut() = Value::Float(0.0);
                    *agg_data.avg_mut() = Value::Float(0.0);
                    *agg_data.deviation_mut() = Value::Float(0.0);
                }

                // 获取当前值
                let cnt = agg_data.cnt().clone();
                let avg = agg_data.avg().clone();
                let deviation = agg_data.deviation().clone();

                if let (Value::Float(c), Value::Float(a), Value::Float(d)) = (cnt, avg, deviation) {
                    let new_cnt = c + 1.0;
                    // Welford 算法计算标准差
                    let delta = val_f64 - a;
                    let new_avg = a + delta / new_cnt;
                    let delta2 = val_f64 - new_avg;
                    let new_deviation = d + delta * delta2;

                    *agg_data.cnt_mut() = Value::Float(new_cnt);
                    *agg_data.avg_mut() = Value::Float(new_avg);
                    *agg_data.deviation_mut() = Value::Float(new_deviation);

                    if new_cnt >= 2.0 {
                        let variance = new_deviation / (new_cnt - 1.0);
                        *agg_data.result_mut() = Value::Float(variance.sqrt());
                    }
                }
                Ok(())
            }),
        );

        // BIT_AND 函数
        self.functions.insert(
            "BIT_AND".to_string(),
            Arc::new(|agg_data: &mut AggData, val: &Value| {
                let res = agg_data.result_mut();
                if res.is_bad_null() {
                    return Ok(());
                }
                if !val.is_null() && !val.is_empty() && !matches!(val, Value::Int(_)) {
                    *res = Value::Null(NullType::BadType);
                    return Ok(());
                }
                if val.is_null() || val.is_empty() {
                    return Ok(());
                }
                if let Value::Int(v) = val {
                    if res.is_null() {
                        *res = Value::Int(*v);
                    } else if let Value::Int(r) = res {
                        *res = Value::Int(*r & *v);
                    }
                }
                Ok(())
            }),
        );

        // BIT_OR 函数
        self.functions.insert(
            "BIT_OR".to_string(),
            Arc::new(|agg_data: &mut AggData, val: &Value| {
                let res = agg_data.result_mut();
                if res.is_bad_null() {
                    return Ok(());
                }
                if !val.is_null() && !val.is_empty() && !matches!(val, Value::Int(_)) {
                    *res = Value::Null(NullType::BadType);
                    return Ok(());
                }
                if val.is_null() || val.is_empty() {
                    return Ok(());
                }
                if let Value::Int(v) = val {
                    if res.is_null() {
                        *res = Value::Int(*v);
                    } else if let Value::Int(r) = res {
                        *res = Value::Int(*r | *v);
                    }
                }
                Ok(())
            }),
        );

        // BIT_XOR 函数
        self.functions.insert(
            "BIT_XOR".to_string(),
            Arc::new(|agg_data: &mut AggData, val: &Value| {
                let res = agg_data.result_mut();
                if res.is_bad_null() {
                    return Ok(());
                }
                if !val.is_null() && !val.is_empty() && !matches!(val, Value::Int(_)) {
                    *res = Value::Null(NullType::BadType);
                    return Ok(());
                }
                if val.is_null() || val.is_empty() {
                    return Ok(());
                }
                if let Value::Int(v) = val {
                    if res.is_null() {
                        *res = Value::Int(*v);
                    } else if let Value::Int(r) = res {
                        *res = Value::Int(*r ^ *v);
                    }
                }
                Ok(())
            }),
        );

        // COLLECT 函数
        self.functions.insert(
            "COLLECT".to_string(),
            Arc::new(|agg_data: &mut AggData, val: &Value| {
                let res = agg_data.result_mut();
                if res.is_bad_null() {
                    return Ok(());
                }
                if res.is_null() {
                    *res = Value::List(Vec::new());
                }
                if val.is_null() || val.is_empty() {
                    return Ok(());
                }
                if let Value::List(ref mut list) = res {
                    list.push(val.clone());
                } else {
                    *res = Value::Null(NullType::BadData);
                }
                Ok(())
            }),
        );

        // COLLECT_SET 函数（对应 nebula-graph 的 COLLECT_SET）
        self.functions.insert(
            "COLLECT_SET".to_string(),
            Arc::new(|agg_data: &mut AggData, val: &Value| {
                let res = agg_data.result_mut();
                if res.is_bad_null() {
                    return Ok(());
                }
                if res.is_null() {
                    *res = Value::Set(std::collections::HashSet::new());
                }
                if val.is_null() || val.is_empty() {
                    return Ok(());
                }
                if let Value::Set(ref mut set) = res {
                    set.insert(val.clone());
                } else {
                    *res = Value::Null(NullType::BadData);
                }
                Ok(())
            }),
        );
    }

    /// 获取聚合函数
    pub fn get(&self, name: &str) -> Option<AggFunction> {
        self.functions.get(&name.to_uppercase()).cloned()
    }

    /// 检查聚合函数是否存在
    pub fn find(&self, name: &str) -> bool {
        self.functions.contains_key(&name.to_uppercase())
    }

    /// 注册自定义聚合函数
    pub fn register(&mut self, name: &str, func: AggFunction) -> Result<(), DBError> {
        let upper_name = name.to_uppercase();
        if self.functions.contains_key(&upper_name) {
            return Err(DBError::Query(QueryError::ExecutionError(format!(
                "聚合函数 '{}' 已存在",
                name
            ))));
        }
        self.functions.insert(upper_name, func);
        Ok(())
    }
}

impl Default for AggFunctionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_function() {
        let manager = AggFunctionManager::new();
        let count_func = manager.get("COUNT").unwrap();

        let mut agg_data = AggData::new();

        // 测试空值 - COUNT 应该初始化为 0
        count_func(&mut agg_data, &Value::Null(NullType::Null)).unwrap();
        assert_eq!(agg_data.result(), &Value::Int(0));

        // 测试正常值
        count_func(&mut agg_data, &Value::Int(1)).unwrap();
        assert_eq!(agg_data.result(), &Value::Int(1));

        count_func(&mut agg_data, &Value::Int(2)).unwrap();
        assert_eq!(agg_data.result(), &Value::Int(2));

        // 测试 NULL 不计数
        count_func(&mut agg_data, &Value::Null(NullType::Null)).unwrap();
        assert_eq!(agg_data.result(), &Value::Int(2));
    }

    #[test]
    fn test_sum_function() {
        let manager = AggFunctionManager::new();
        let sum_func = manager.get("SUM").unwrap();

        let mut agg_data = AggData::new();

        sum_func(&mut agg_data, &Value::Int(10)).unwrap();
        assert_eq!(agg_data.result(), &Value::Int(10));

        sum_func(&mut agg_data, &Value::Int(20)).unwrap();
        assert_eq!(agg_data.result(), &Value::Int(30));

        // 测试 NULL 不加入
        sum_func(&mut agg_data, &Value::Null(NullType::Null)).unwrap();
        assert_eq!(agg_data.result(), &Value::Int(30));
    }

    #[test]
    fn test_collect_set_function() {
        let manager = AggFunctionManager::new();
        let collect_set_func = manager.get("COLLECT_SET").unwrap();

        let mut agg_data = AggData::new();

        collect_set_func(&mut agg_data, &Value::Int(1)).unwrap();
        collect_set_func(&mut agg_data, &Value::Int(2)).unwrap();
        collect_set_func(&mut agg_data, &Value::Int(1)).unwrap(); // 重复值

        if let Value::Set(set) = agg_data.result() {
            assert_eq!(set.len(), 2);
        } else {
            panic!("结果应该是 Set 类型");
        }
    }
}
