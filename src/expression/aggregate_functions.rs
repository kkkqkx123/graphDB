use crate::core::{ExpressionError, Value};
use crate::core::Expression;
use crate::core::types::operators::AggregateFunction as CoreAggregateFunction;
use crate::expression::operators_ext::ExtendedAggregateFunction;
use serde::{Deserialize, Serialize};

/// 为了向后兼容，保留原有的AggregateFunction类型别名
/// 
/// 注意：新代码应该使用ExtendedAggregateFunction
#[deprecated(note = "使用 ExtendedAggregateFunction 替代")]
pub type AggregateFunction = ExtendedAggregateFunction;

impl std::fmt::Display for ExtendedAggregateFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtendedAggregateFunction::Core(core_func) => {
                match core_func {
                    CoreAggregateFunction::Count => write!(f, "COUNT"),
                    CoreAggregateFunction::Sum => write!(f, "SUM"),
                    CoreAggregateFunction::Avg => write!(f, "AVG"),
                    CoreAggregateFunction::Min => write!(f, "MIN"),
                    CoreAggregateFunction::Max => write!(f, "MAX"),
                    CoreAggregateFunction::Collect => write!(f, "COLLECT"),
                    CoreAggregateFunction::Distinct => write!(f, "DISTINCT"),
                }
            }
        }
    }
}

impl ExtendedAggregateFunction {
    /// 从字符串创建聚合函数
    pub fn from_str(func_name: &str) -> Result<Self, ExpressionError> {
        let core_func = match func_name.to_uppercase().as_str() {
            "COUNT" => CoreAggregateFunction::Count,
            "SUM" => CoreAggregateFunction::Sum,
            "AVG" => CoreAggregateFunction::Avg,
            "MIN" => CoreAggregateFunction::Min,
            "MAX" => CoreAggregateFunction::Max,
            "COLLECT" => CoreAggregateFunction::Collect,
            "DISTINCT" => CoreAggregateFunction::Distinct,
            _ => {
                return Err(ExpressionError::function_error(format!(
                    "Unknown aggregate function: {}",
                    func_name
                )));
            }
        };
        
        Ok(ExtendedAggregateFunction::Core(core_func))
    }
    
    /// 获取Core聚合函数（如果可能）
    pub fn as_core(&self) -> Option<&CoreAggregateFunction> {
        match self {
            ExtendedAggregateFunction::Core(core_func) => Some(core_func),
        }
    }
    
    /// 检查是否是数值聚合函数
    pub fn is_numeric(&self) -> bool {
        match self {
            ExtendedAggregateFunction::Core(core_func) => core_func.is_numeric(),
        }
    }
    
    /// 检查是否是集合聚合函数
    pub fn is_collection(&self) -> bool {
        match self {
            ExtendedAggregateFunction::Core(core_func) => core_func.is_collection(),
        }
    }
}

/// 聚合表达式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateExpression {
    pub function: ExtendedAggregateFunction,
    pub argument: Box<Expression>,
    pub distinct: bool,
}

impl AggregateExpression {
    pub fn new(function: ExtendedAggregateFunction, argument: Expression, distinct: bool) -> Self {
        Self {
            function,
            argument: Box::new(argument),
            distinct,
        }
    }

    /// 计算聚合表达式的值
    pub fn evaluate(
        &self,
        context: &crate::core::ExpressionContext,
        state: &mut AggregateState,
    ) -> Result<Value, ExpressionError> {
        // 计算参数值
        let evaluator = crate::core::evaluator::ExpressionEvaluator;
        let arg_value = evaluator
            .evaluate(&self.argument, context)
            .map_err(|e| ExpressionError::function_error(e.to_string()))?;

        // 更新聚合状态
        state.update(&self.function, &arg_value);

        // 返回当前状态的聚合结果
        match &self.function {
            ExtendedAggregateFunction::Core(core_func) => {
                match core_func {
                    CoreAggregateFunction::Count => Ok(Value::Int(state.count)),
                    CoreAggregateFunction::Sum => Ok(state.sum.clone()),
                    CoreAggregateFunction::Min => Ok(state
                        .min
                        .clone()
                        .unwrap_or(Value::Null(crate::core::value::NullType::Null))),
                    CoreAggregateFunction::Max => Ok(state
                        .max
                        .clone()
                        .unwrap_or(Value::Null(crate::core::value::NullType::Null))),
                    CoreAggregateFunction::Avg => {
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
                    CoreAggregateFunction::Collect => Ok(Value::List(state.values.clone())),
                    CoreAggregateFunction::Distinct => Ok(Value::List(
                        state.values.iter().cloned().collect::<std::collections::HashSet<_>>()
                            .into_iter().collect()
                    )),
                }
            }
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
    pub distinct_values: std::collections::HashSet<String>, // 用于去重
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
        }
    }

    pub fn reset(&mut self) {
        self.count = 0;
        self.sum = Value::Int(0);
        self.min = None;
        self.max = None;
        self.values.clear();
        self.distinct_values.clear();
    }

    /// 更新聚合状态
    pub fn update(&mut self, function: &ExtendedAggregateFunction, value: &Value) {
        // 如果是去重函数，检查是否已存在
        if matches!(function, ExtendedAggregateFunction::Core(CoreAggregateFunction::Count))
            && self.distinct_values.contains(&value.to_string())
        {
            return; // 跳过重复值
        }

        self.count += 1;
        self.values.push(value.clone());

        // 更新最小值
        if self.min.as_ref().map_or(true, |min_val| value.lt(min_val)) {
            self.min = Some(value.clone());
        }

        // 更新最大值
        if self.max.as_ref().map_or(true, |max_val| value.gt(max_val)) {
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
            _ => {} // 不兼容的类型，跳过求和
        }

        // 记录值用于去重
        if matches!(function, ExtendedAggregateFunction::Core(CoreAggregateFunction::Count)) {
            self.distinct_values.insert(value.to_string());
        }
    }
}

// 为了向后兼容，保留原有的操作符枚举定义
#[deprecated(note = "使用 ExtendedAggregateFunction 替代")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LegacyAggregateFunction {
    Count,
    Sum,
    Avg,
    Min,
    Max,
    Collect,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::expression::Expression;
    use crate::core::context::expression::default_context::DefaultExpressionContext;

    #[test]
    fn test_extended_aggregate_function() {
        // 测试从字符串创建
        let func = ExtendedAggregateFunction::from_str("COUNT").unwrap();
        assert!(matches!(func, ExtendedAggregateFunction::Core(CoreAggregateFunction::Count)));
        
        let func = ExtendedAggregateFunction::from_str("SUM").unwrap();
        assert!(matches!(func, ExtendedAggregateFunction::Core(CoreAggregateFunction::Sum)));
        
        // 测试数值聚合函数检查
        let sum_func = ExtendedAggregateFunction::from_str("SUM").unwrap();
        assert!(sum_func.is_numeric());
        assert!(!sum_func.is_collection());
        
        let collect_func = ExtendedAggregateFunction::from_str("COLLECT").unwrap();
        assert!(!collect_func.is_numeric());
        assert!(collect_func.is_collection());
    }
}