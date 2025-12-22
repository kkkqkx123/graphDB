use crate::core::ExpressionError;
use crate::core::Value;
use crate::core::Expression;
use serde::{Deserialize, Serialize};

/// 聚合函数类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AggregateFunction {
    Count,
    Sum,
    Avg,
    Min,
    Max,
    Collect,
}

impl std::fmt::Display for AggregateFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AggregateFunction::Count => write!(f, "COUNT"),
            AggregateFunction::Sum => write!(f, "SUM"),
            AggregateFunction::Avg => write!(f, "AVG"),
            AggregateFunction::Min => write!(f, "MIN"),
            AggregateFunction::Max => write!(f, "MAX"),
            AggregateFunction::Collect => write!(f, "COLLECT"),
        }
    }
}

impl AggregateFunction {
    /// 从字符串创建聚合函数
    pub fn from_str(func_name: &str) -> Result<Self, ExpressionError> {
        match func_name.to_uppercase().as_str() {
            "COUNT" => Ok(AggregateFunction::Count),
            "SUM" => Ok(AggregateFunction::Sum),
            "AVG" => Ok(AggregateFunction::Avg),
            "MIN" => Ok(AggregateFunction::Min),
            "MAX" => Ok(AggregateFunction::Max),
            "COLLECT" => Ok(AggregateFunction::Collect),
            _ => Err(ExpressionError::FunctionError(format!(
                "Unknown aggregate function: {}",
                func_name
            ))),
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
    pub fn evaluate(
        &self,
        context: &crate::core::ExpressionContext,
        state: &mut AggregateState,
    ) -> Result<Value, ExpressionError> {
        // 计算参数值
        let evaluator = super::evaluator::ExpressionEvaluator;
        let arg_value = evaluator
            .evaluate(&self.argument, context)
            .map_err(|e| ExpressionError::FunctionError(e.to_string()))?;

        // 更新聚合状态
        state.update(&self.function, &arg_value);

        // 返回当前状态的聚合结果
        match self.function {
            AggregateFunction::Count => Ok(Value::Int(state.count)),
            AggregateFunction::Sum => Ok(state.sum.clone()),
            AggregateFunction::Min => Ok(state
                .min
                .clone()
                .unwrap_or(Value::Null(crate::core::value::NullType::Null))),
            AggregateFunction::Max => Ok(state
                .max
                .clone()
                .unwrap_or(Value::Null(crate::core::value::NullType::Null))),
            AggregateFunction::Avg => {
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
            AggregateFunction::Collect => Ok(Value::List(state.values.clone())),
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
    pub fn update(&mut self, function: &AggregateFunction, value: &Value) {
        // 如果是去重函数，检查是否已存在
        if matches!(function, AggregateFunction::Count)
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
        if matches!(function, AggregateFunction::Count) {
            self.distinct_values.insert(value.to_string());
        }
    }
}
