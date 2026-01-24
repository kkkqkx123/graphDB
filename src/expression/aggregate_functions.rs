use crate::core::types::operators::AggregateFunction;
use crate::core::Expression;
use crate::core::{ExpressionError, Value};
use serde::{Deserialize, Serialize};

impl std::fmt::Display for AggregateFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl AggregateFunction {
    /// 从字符串创建聚合函数
    pub fn from_str(func_name: &str) -> Result<Self, ExpressionError> {
        match func_name.to_uppercase().as_str() {
            "COUNT" => Ok(AggregateFunction::Count(None)),
            "COUNT_DISTINCT" => Ok(AggregateFunction::Distinct("".to_string())), // 需要字段名
            "SUM" => Ok(AggregateFunction::Sum("".to_string())), // 需要字段名
            "AVG" => Ok(AggregateFunction::Avg("".to_string())), // 需要字段名
            "MIN" => Ok(AggregateFunction::Min("".to_string())), // 需要字段名
            "MAX" => Ok(AggregateFunction::Max("".to_string())), // 需要字段名
            "COLLECT" => Ok(AggregateFunction::Collect("".to_string())), // 需要字段名
            "DISTINCT" => Ok(AggregateFunction::Distinct("".to_string())), // 需要字段名
            "PERCENTILE" => Ok(AggregateFunction::Percentile("".to_string(), 50.0)), // 需要字段名和百分位数
            _ => {
                return Err(ExpressionError::function_error(format!(
                    "Unknown aggregate function: {}",
                    func_name
                )));
            }
        }
    }

    /// 从字符串和参数创建聚合函数
    pub fn from_str_with_args(func_name: &str, args: &[String]) -> Result<Self, ExpressionError> {
        match func_name.to_uppercase().as_str() {
            "COUNT" => {
                if args.is_empty() {
                    Ok(AggregateFunction::Count(None)) // COUNT(*)
                } else {
                    Ok(AggregateFunction::Count(Some(args[0].clone()))) // COUNT(field)
                }
            },
            "SUM" => {
                if args.is_empty() {
                    return Err(ExpressionError::function_error("SUM function requires a field name".to_string()));
                }
                Ok(AggregateFunction::Sum(args[0].clone()))
            },
            "AVG" => {
                if args.is_empty() {
                    return Err(ExpressionError::function_error("AVG function requires a field name".to_string()));
                }
                Ok(AggregateFunction::Avg(args[0].clone()))
            },
            "MIN" => {
                if args.is_empty() {
                    return Err(ExpressionError::function_error("MIN function requires a field name".to_string()));
                }
                Ok(AggregateFunction::Min(args[0].clone()))
            },
            "MAX" => {
                if args.is_empty() {
                    return Err(ExpressionError::function_error("MAX function requires a field name".to_string()));
                }
                Ok(AggregateFunction::Max(args[0].clone()))
            },
            "COLLECT" => {
                if args.is_empty() {
                    return Err(ExpressionError::function_error("COLLECT function requires a field name".to_string()));
                }
                Ok(AggregateFunction::Collect(args[0].clone()))
            },
            "DISTINCT" => {
                if args.is_empty() {
                    return Err(ExpressionError::function_error("DISTINCT function requires a field name".to_string()));
                }
                Ok(AggregateFunction::Distinct(args[0].clone()))
            },
            "PERCENTILE" => {
                if args.len() < 2 {
                    return Err(ExpressionError::function_error("PERCENTILE function requires a field name and percentile value".to_string()));
                }
                let percentile = args[1].parse::<f64>().map_err(|_| {
                    ExpressionError::function_error("Invalid percentile value".to_string())
                })?;
                Ok(AggregateFunction::Percentile(args[0].clone(), percentile))
            },
            _ => {
                return Err(ExpressionError::function_error(format!(
                    "Unknown aggregate function: {}",
                    func_name
                )));
            }
        }
    }
}

/// 聚合表达式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateExpression {
    pub function: AggregateFunction,
    pub argument: Box<Expression>,
    pub distinct: bool,
}

impl AggregateExpression {
    pub fn new(function: AggregateFunction, argument: Expression, distinct: bool) -> Self {
        Self {
            function,
            argument: Box::new(argument),
            distinct,
        }
    }

    /// 计算聚合表达式的值
    pub fn evaluate<C: crate::expression::ExpressionContext>(
        &self,
        context: &mut C,
        state: &mut AggregateState,
    ) -> Result<Value, ExpressionError> {
        let arg_value =
            crate::expression::evaluator::expression_evaluator::ExpressionEvaluator::evaluate(
                &self.argument,
                context,
            )
            .map_err(|e| ExpressionError::function_error(e.to_string()))?;

        // 更新聚合状态
        state.update(&self.function, &arg_value, self.distinct);

        // 返回当前状态的聚合结果
        match &self.function {
            AggregateFunction::Count(_) => Ok(Value::Int(state.count)),
            AggregateFunction::Sum(_) => Ok(state.sum.clone()),
            AggregateFunction::Min(_) => Ok(state
                .min
                .clone()
                .unwrap_or(Value::Null(crate::core::value::NullType::Null))),
            AggregateFunction::Max(_) => Ok(state
                .max
                .clone()
                .unwrap_or(Value::Null(crate::core::value::NullType::Null))),
            AggregateFunction::Avg(_) => {
                if state.count > 0 {
                    match &state.sum {
                        Value::Int(i) => Ok(Value::Float(*i as f64 / state.count as f64)),
                        Value::Float(f) => Ok(Value::Float(*f / state.count as f64)),
                        _ => Ok(Value::Float(0.0)),
                    }
                } else {
                    Ok(Value::Float(0.0))
                }
            }
            AggregateFunction::Collect(_) => Ok(Value::List(state.values.clone())),
            AggregateFunction::Distinct(_) => Ok(Value::List(
                state
                    .values
                    .iter()
                    .cloned()
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect(),
            )),
            AggregateFunction::Percentile(_, _) => state.calculate_percentile(50.0),
            AggregateFunction::Std(_) => state.calculate_std(),
            AggregateFunction::BitAnd(_) => state.calculate_bit_and(),
            AggregateFunction::BitOr(_) => state.calculate_bit_or(),
            AggregateFunction::GroupConcat(_, _) => state.calculate_group_concat(),
        }
    }
}

/// 聚合状态，用于累积聚合函数的中间结果
#[derive(Debug, Clone)]
pub struct AggregateState {
    pub count: i64,
    pub sum: Value,
    pub min: Option<Value>,
    pub max: Option<Value>,
    pub values: Vec<Value>,
    pub distinct_values: std::collections::HashSet<String>,
    pub percentile_values: Vec<f64>,
    pub std_values: Vec<f64>,
    pub bit_and_value: Option<i64>,
    pub bit_or_value: Option<i64>,
    pub group_concat_values: Vec<Value>,
}

impl AggregateState {
    pub fn new() -> Self {
        Self {
            count: 0,
            sum: Value::Int(0),
            min: None,
            max: None,
            values: Vec::new(),
            distinct_values: std::collections::HashSet::new(),
            percentile_values: Vec::new(),
            std_values: Vec::new(),
            bit_and_value: None,
            bit_or_value: None,
            group_concat_values: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.count = 0;
        self.sum = Value::Int(0);
        self.min = None;
        self.max = None;
        self.values.clear();
        self.distinct_values.clear();
        self.percentile_values.clear();
        self.std_values.clear();
        self.bit_and_value = None;
        self.bit_or_value = None;
        self.group_concat_values.clear();
    }

    /// 更新聚合状态
    pub fn update(&mut self, function: &AggregateFunction, value: &Value, distinct: bool) {
        let value_str = format!("{}", value);

        // 如果启用distinct，检查是否已存在
        if distinct && self.distinct_values.contains(&value_str) {
            return;
        }

        // 记录值用于去重
        if distinct {
            self.distinct_values.insert(value_str);
        }

        self.count += 1;
        self.values.push(value.clone());

        // 根据聚合函数类型进行特殊处理
        match function {
            AggregateFunction::Percentile(_, _) => {
                // PERCENTILE函数特殊处理：收集数值
                match value {
                    Value::Int(v) => self.percentile_values.push(*v as f64),
                    Value::Float(v) => self.percentile_values.push(*v),
                    _ => {}
                }
            }
            AggregateFunction::Std(_) => {
                // STD函数特殊处理：收集数值
                match value {
                    Value::Int(v) => self.std_values.push(*v as f64),
                    Value::Float(v) => self.std_values.push(*v),
                    _ => {}
                }
            }
            AggregateFunction::BitAnd(_) => {
                // BIT_AND函数特殊处理
                match value {
                    Value::Int(v) => {
                        if let Some(current) = self.bit_and_value {
                            self.bit_and_value = Some(current & v);
                        } else {
                            self.bit_and_value = Some(*v);
                        }
                    }
                    _ => {}
                }
            }
            AggregateFunction::BitOr(_) => {
                // BIT_OR函数特殊处理
                match value {
                    Value::Int(v) => {
                        if let Some(current) = self.bit_or_value {
                            self.bit_or_value = Some(current | v);
                        } else {
                            self.bit_or_value = Some(*v);
                        }
                    }
                    _ => {}
                }
            }
            AggregateFunction::GroupConcat(_, _) => {
                // GROUP_CONCAT函数特殊处理
                self.group_concat_values.push(value.clone());
            }
            _ => {
                // 其他聚合函数的通用处理
                // 更新最小值
                if self.min.as_ref().map_or(true, |min_val| value < min_val) {
                    self.min = Some(value.clone());
                }

                // 更新最大值
                if self.max.as_ref().map_or(true, |max_val| value > max_val) {
                    self.max = Some(value.clone());
                }

                // 更新总和
                match (&mut self.sum, value) {
                    (Value::Int(ref mut sum_int), Value::Int(val_int)) => {
                        *sum_int += *val_int;
                    }
                    (Value::Float(ref mut sum_float), Value::Float(val_float)) => {
                        *sum_float += *val_float;
                    }
                    (Value::Int(ref mut sum_int), Value::Float(val_float)) => {
                        self.sum = Value::Float(*sum_int as f64 + *val_float);
                    }
                    (Value::Float(ref mut sum_float), Value::Int(val_int)) => {
                        *sum_float += *val_int as f64;
                    }
                    _ => {}
                }
            }
        }
    }

    /// 计算百分位数
    pub fn calculate_percentile(&self, percentile: f64) -> Result<Value, ExpressionError> {
        if self.percentile_values.is_empty() {
            return Ok(Value::Null(crate::core::value::NullType::Null));
        }

        if percentile < 0.0 || percentile > 100.0 {
            return Err(ExpressionError::function_error(
                "Percentile must be between 0 and 100".to_string(),
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

    /// 计算标准差
    pub fn calculate_std(&self) -> Result<Value, ExpressionError> {
        if self.std_values.is_empty() {
            return Ok(Value::Null(crate::core::value::NullType::Null));
        }

        let n = self.std_values.len() as f64;
        let mean: f64 = self.std_values.iter().sum::<f64>() / n;
        let variance: f64 = self.std_values.iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>() / n;
        let std_dev = variance.sqrt();

        Ok(Value::Float(std_dev))
    }

    /// 计算按位与
    pub fn calculate_bit_and(&self) -> Result<Value, ExpressionError> {
        if let Some(value) = self.bit_and_value {
            Ok(Value::Int(value))
        } else {
            Ok(Value::Null(crate::core::value::NullType::Null))
        }
    }

    /// 计算按位或
    pub fn calculate_bit_or(&self) -> Result<Value, ExpressionError> {
        if let Some(value) = self.bit_or_value {
            Ok(Value::Int(value))
        } else {
            Ok(Value::Null(crate::core::value::NullType::Null))
        }
    }

    /// 计算分组连接
    pub fn calculate_group_concat(&self) -> Result<Value, ExpressionError> {
        if self.group_concat_values.is_empty() {
            return Ok(Value::String(String::new()));
        }

        let result: Vec<String> = self.group_concat_values.iter()
            .map(|v| format!("{}", v))
            .collect();
        Ok(Value::String(result.join(",")))
    }
}

// Legacy类型已移除 - 现在直接使用Core层的AggregateFunction
// 所有聚合函数都在Core层定义，无需转换

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_aggregate_function() {
        // 测试从字符串创建
        let func = AggregateFunction::from_str("COUNT").expect("from_str should succeed");
        assert!(matches!(func, AggregateFunction::Count(_)));

        let func = AggregateFunction::from_str("SUM").expect("from_str should succeed");
        assert!(matches!(func, AggregateFunction::Sum(_)));

        let sum_func = AggregateFunction::from_str_with_args("SUM", &["field".to_string()]).expect("from_str_with_args should succeed");
        assert!(sum_func.is_numeric());
        assert!(!sum_func.is_collection());

        let collect_func = AggregateFunction::from_str_with_args("COLLECT", &["field".to_string()]).expect("from_str_with_args should succeed");
        assert!(!collect_func.is_numeric());
        assert!(collect_func.is_collection());
    }
}
