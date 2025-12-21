//! 表达式求值器统一接口
//!
//! 定义了表达式求值器的统一接口，提供类型安全和扩展性

use crate::core::ExpressionError;
use crate::core::Value;
use crate::expression::context::ExpressionContextCore;
use crate::expression::{Expression, ExpressionContext};

/// 表达式求值器统一接口
///
/// 这个trait定义了所有表达式求值器必须实现的基本方法，
/// 提供了类型安全和扩展性
pub trait ExpressionEvaluator {
    /// 求值表达式
    ///
    /// # 参数
    /// - `expr`: 要计算的表达式
    /// - `context`: 求值上下文
    ///
    /// # 返回
    /// - `Ok(Value)`: 求值结果
    /// - `Err(ExpressionError)`: 求值错误
    fn evaluate(
        &self,
        expr: &Expression,
        context: &ExpressionContext,
    ) -> Result<Value, ExpressionError>;

    /// 批量求值表达式
    ///
    /// # 参数
    /// - `exprs`: 表达式列表
    /// - `context`: 求值上下文
    ///
    /// # 返回
    /// - `Ok(Vec<Value>)`: 求值结果列表
    /// - `Err(ExpressionError)`: 求值错误
    fn evaluate_batch(
        &self,
        exprs: &[Expression],
        context: &ExpressionContext,
    ) -> Result<Vec<Value>, ExpressionError> {
        let mut results = Vec::with_capacity(exprs.len());
        for expr in exprs {
            results.push(self.evaluate(expr, context)?);
        }
        Ok(results)
    }

    /// 检查表达式是否为常量
    ///
    /// # 参数
    /// - `expr`: 要检查的表达式
    ///
    /// # 返回
    /// `true`: 如果表达式是常量，`false` 否则
    fn is_constant(&self, expr: &Expression) -> bool;

    /// 获取表达式中使用的所有变量
    ///
    /// # 参数
    /// - `expr`: 要分析的表达式
    ///
    /// # 返回
    /// 变量名列表（去重且排序）
    fn get_variables(&self, expr: &Expression) -> Vec<String>;

    /// 检查表达式是否包含聚合函数
    ///
    /// # 参数
    /// - `expr`: 要检查的表达式
    ///
    /// # 返回
    /// `true`: 如果包含聚合函数，`false` 否则
    fn contains_aggregate(&self, expr: &Expression) -> bool;

    /// 优化表达式（可选实现）
    ///
    /// 默认实现返回表达式本身，子类可以重写以提供优化
    ///
    /// # 参数
    /// - `expr`: 要优化的表达式
    ///
    /// # 返回
    /// 优化后的表达式
    fn optimize(&self, expr: Expression) -> Expression {
        expr
    }

    /// 验证表达式（可选实现）
    ///
    /// 默认实现返回Ok，子类可以重写以提供验证逻辑
    ///
    /// # 参数
    /// - `expr`: 要验证的表达式
    ///
    /// # 返回
    /// `Ok(())`: 验证通过，`Err(ExpressionError)` 验证失败
    fn validate(&self, _expr: &Expression) -> Result<(), ExpressionError> {
        Ok(())
    }

    /// 获取求值器名称（用于调试和日志）
    fn evaluator_name(&self) -> &'static str {
        "ExpressionEvaluator"
    }
}

/// 默认表达式求值器实现
///
/// 提供了基本的表达式求值功能
#[derive(Debug, Clone)]
pub struct DefaultExpressionEvaluator;

impl DefaultExpressionEvaluator {
    /// 创建新的默认求值器
    pub fn new() -> Self {
        Self
    }
}

impl ExpressionEvaluator for DefaultExpressionEvaluator {
    fn evaluate(
        &self,
        expr: &Expression,
        context: &ExpressionContext,
    ) -> Result<Value, ExpressionError> {
        // 委托给具体的求值实现
        self.eval_expression(expr, context)
    }

    fn evaluate_batch(
        &self,
        exprs: &[Expression],
        context: &ExpressionContext,
    ) -> Result<Vec<Value>, ExpressionError> {
        let mut results = Vec::with_capacity(exprs.len());
        for expr in exprs {
            results.push(self.evaluate(expr, context)?);
        }
        Ok(results)
    }

    fn is_constant(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Literal(_) => true,
            Expression::List(items) => items.iter().all(|e| self.is_constant(e)),
            Expression::Map(pairs) => pairs.iter().all(|(_, e)| self.is_constant(e)),
            _ => false,
        }
    }

    fn get_variables(&self, expr: &Expression) -> Vec<String> {
        let mut variables = Vec::new();
        self.collect_variables(expr, &mut variables);
        variables.sort();
        variables.dedup();
        variables
    }

    fn contains_aggregate(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Aggregate { .. } => true,
            Expression::Function { name, .. } => {
                matches!(
                    name.to_lowercase().as_str(),
                    "count" | "sum" | "avg" | "min" | "max" | "collect" | "distinct"
                )
            }
            _ => {
                // 递归检查子表达式
                for child in expr.children() {
                    if self.contains_aggregate(child) {
                        return true;
                    }
                }
                false
            }
        }
    }

    fn evaluator_name(&self) -> &'static str {
        "DefaultExpressionEvaluator"
    }
}

impl DefaultExpressionEvaluator {
    /// 内部求值实现
    fn eval_expression(
        &self,
        expr: &Expression,
        context: &ExpressionContext,
    ) -> Result<Value, ExpressionError> {
        match expr {
            Expression::Literal(literal_value) => {
                // 将 LiteralValue 转换为 Value
                match literal_value {
                    crate::expression::LiteralValue::Bool(b) => Ok(Value::Bool(*b)),
                    crate::expression::LiteralValue::Int(i) => Ok(Value::Int(*i)),
                    crate::expression::LiteralValue::Float(f) => Ok(Value::Float(*f)),
                    crate::expression::LiteralValue::String(s) => Ok(Value::String(s.clone())),
                    crate::expression::LiteralValue::Null => {
                        Ok(Value::Null(crate::core::NullType::Null))
                    }
                }
            }
            Expression::Variable(var_name) => {
                // 从上下文变量中获取值
                if let Some(value) = context.get_variable(var_name) {
                    Ok(value)
                } else {
                    Err(ExpressionError::PropertyNotFound(format!(
                        "Variable '${}' not found",
                        var_name
                    )))
                }
            }
            Expression::Binary { left, op, right } => {
                // 求值左右操作数
                let left_val = self.eval_expression(left, context)?;
                let right_val = self.eval_expression(right, context)?;

                // 执行二元操作
                self.eval_binary_operation(&left_val, op, &right_val)
            }
            Expression::Unary { op, operand } => {
                // 求值操作数
                let operand_val = self.eval_expression(operand, context)?;

                // 执行一元操作
                self.eval_unary_operation(op, &operand_val)
            }
            Expression::Function { name, args } => {
                // 求值所有参数
                let arg_values: Result<Vec<Value>, ExpressionError> = args
                    .iter()
                    .map(|arg| self.eval_expression(arg, context))
                    .collect();

                let arg_values = arg_values?;

                // 执行函数调用
                self.eval_function(name, &arg_values)
            }
            Expression::Aggregate {
                func,
                arg,
                distinct,
            } => {
                // 求值聚合参数
                let arg_val = self.eval_expression(arg, context)?;

                // 执行聚合函数
                self.eval_aggregate(func, &arg_val, *distinct)
            }
            Expression::List(items) => {
                // 求值列表中的所有元素
                let evaluated_items: Result<Vec<Value>, ExpressionError> = items
                    .iter()
                    .map(|item| self.eval_expression(item, context))
                    .collect();

                Ok(Value::List(evaluated_items?))
            }
            Expression::Map(pairs) => {
                // 求值映射中的所有值
                let evaluated_pairs: Result<Vec<(String, Value)>, ExpressionError> = pairs
                    .iter()
                    .map(|(key, value)| {
                        let evaluated_value = self.eval_expression(value, context)?;
                        Ok((key.clone(), evaluated_value))
                    })
                    .collect();

                let map = evaluated_pairs?.into_iter().collect();
                Ok(Value::Map(map))
            }
            Expression::Property { object, property } => {
                // 求值对象
                let obj_val = self.eval_expression(object, context)?;

                // 获取属性
                self.get_property(&obj_val, property)
            }
            Expression::TypeCast { expr, target_type } => {
                // 求值表达式
                let value = self.eval_expression(expr, context)?;

                // 执行类型转换
                self.cast_value(&value, target_type)
            }
            // 其他表达式类型的处理...
            _ => Err(ExpressionError::InvalidOperation(format!(
                "Expression type not yet supported: {:?}",
                expr
            ))),
        }
    }

    /// 递归收集表达式中的变量
    fn collect_variables(&self, expr: &Expression, variables: &mut Vec<String>) {
        match expr {
            Expression::Variable(name) => {
                if !variables.contains(name) {
                    variables.push(name.clone());
                }
            }
            Expression::Binary { left, right, .. } => {
                self.collect_variables(left, variables);
                self.collect_variables(right, variables);
            }
            Expression::Unary { operand, .. } => {
                self.collect_variables(operand, variables);
            }
            Expression::Function { args, .. } => {
                for arg in args {
                    self.collect_variables(arg, variables);
                }
            }
            Expression::Aggregate { arg, .. } => {
                self.collect_variables(arg, variables);
            }
            Expression::List(items) => {
                for item in items {
                    self.collect_variables(item, variables);
                }
            }
            Expression::Map(pairs) => {
                for (_, value) in pairs {
                    self.collect_variables(value, variables);
                }
            }
            Expression::Property { object, .. } => {
                self.collect_variables(object, variables);
            }
            Expression::TypeCast { expr, .. } => {
                self.collect_variables(expr, variables);
            }
            // 其他表达式类型...
            _ => {}
        }
    }

    /// 执行二元操作
    fn eval_binary_operation(
        &self,
        left: &Value,
        op: &crate::expression::BinaryOperator,
        right: &Value,
    ) -> Result<Value, ExpressionError> {
        use crate::expression::BinaryOperator;

        match op {
            BinaryOperator::Add => self.add_values(left, right),
            BinaryOperator::Subtract => self.subtract_values(left, right),
            BinaryOperator::Multiply => self.multiply_values(left, right),
            BinaryOperator::Divide => self.divide_values(left, right),
            BinaryOperator::Modulo => self.modulo_values(left, right),
            BinaryOperator::Equal => Ok(Value::Bool(self.values_equal(left, right))),
            BinaryOperator::NotEqual => Ok(Value::Bool(!self.values_equal(left, right))),
            BinaryOperator::LessThan => Ok(Value::Bool(matches!(
                self.compare_values(left, right),
                std::cmp::Ordering::Less
            ))),
            BinaryOperator::LessThanOrEqual => Ok(Value::Bool(!matches!(
                self.compare_values(left, right),
                std::cmp::Ordering::Greater
            ))),
            BinaryOperator::GreaterThan => Ok(Value::Bool(matches!(
                self.compare_values(left, right),
                std::cmp::Ordering::Greater
            ))),
            BinaryOperator::GreaterThanOrEqual => Ok(Value::Bool(!matches!(
                self.compare_values(left, right),
                std::cmp::Ordering::Less
            ))),
            BinaryOperator::And => Ok(self.logical_and(left, right)),
            BinaryOperator::Or => Ok(self.logical_or(left, right)),
            // 其他操作符...
            _ => Err(ExpressionError::InvalidOperation(format!(
                "Binary operator not yet supported: {:?}",
                op
            ))),
        }
    }

    /// 执行一元操作
    fn eval_unary_operation(
        &self,
        op: &crate::expression::UnaryOperator,
        operand: &Value,
    ) -> Result<Value, ExpressionError> {
        use crate::expression::UnaryOperator;

        match op {
            UnaryOperator::Plus => Ok(operand.clone()),
            UnaryOperator::Minus => Ok(self.negate_value(operand)),
            UnaryOperator::Not => Ok(self.logical_not(operand)),
            UnaryOperator::IsNull => Ok(Value::Bool(matches!(operand, Value::Null(_)))),
            UnaryOperator::IsNotNull => Ok(Value::Bool(!matches!(operand, Value::Null(_)))),
            // 其他操作符...
            _ => Err(ExpressionError::InvalidOperation(format!(
                "Unary operator not yet supported: {:?}",
                op
            ))),
        }
    }

    /// 执行函数调用
    fn eval_function(&self, name: &str, args: &[Value]) -> Result<Value, ExpressionError> {
        match name {
            "abs" => self.eval_abs(args),
            "length" => self.eval_length(args),
            "type" => self.eval_type(args),
            // 其他函数...
            _ => Err(ExpressionError::UnknownFunction(name.to_string())),
        }
    }

    /// 执行聚合函数
    fn eval_aggregate(
        &self,
        func: &crate::expression::AggregateFunction,
        arg: &Value,
        distinct: bool,
    ) -> Result<Value, ExpressionError> {
        use crate::expression::AggregateFunction;

        match func {
            AggregateFunction::Count => {
                if let Value::List(items) = arg {
                    let count = if distinct {
                        let mut unique_items = std::collections::HashSet::new();
                        for item in items {
                            unique_items.insert(format!("{:?}", item));
                        }
                        unique_items.len() as i64
                    } else {
                        items.len() as i64
                    };
                    Ok(Value::Int(count))
                } else {
                    Ok(Value::Int(1))
                }
            }
            AggregateFunction::Sum => {
                if let Value::List(items) = arg {
                    let sum = items.iter().try_fold(0i64, |acc, val| match val {
                        Value::Int(i) => Ok(acc + i),
                        Value::Float(f) => Ok(acc + *f as i64),
                        _ => Err(ExpressionError::TypeError(
                            "Sum requires numeric values".to_string(),
                        )),
                    });
                    Ok(Value::Int(sum?))
                } else {
                    Ok(arg.clone())
                }
            }
            AggregateFunction::Avg => {
                if let Value::List(items) = arg {
                    if items.is_empty() {
                        return Ok(Value::Null(crate::core::NullType::Null));
                    }
                    let sum = items.iter().try_fold(0.0, |acc, val| match val {
                        Value::Int(i) => Ok(acc + *i as f64),
                        Value::Float(f) => Ok(acc + f),
                        _ => Err(ExpressionError::TypeError(
                            "Avg requires numeric values".to_string(),
                        )),
                    });
                    Ok(Value::Float(sum? / items.len() as f64))
                } else {
                    Ok(arg.clone())
                }
            }
            AggregateFunction::Min => {
                if let Value::List(items) = arg {
                    if items.is_empty() {
                        return Ok(Value::Null(crate::core::NullType::Null));
                    }
                    let min = items
                        .iter()
                        .min_by(|a, b| self.compare_values(a, b))
                        .cloned();
                    Ok(min.unwrap_or(Value::Null(crate::core::NullType::Null)))
                } else {
                    Ok(arg.clone())
                }
            }
            AggregateFunction::Max => {
                if let Value::List(items) = arg {
                    if items.is_empty() {
                        return Ok(Value::Null(crate::core::NullType::Null));
                    }
                    let max = items
                        .iter()
                        .max_by(|a, b| self.compare_values(a, b))
                        .cloned();
                    Ok(max.unwrap_or(Value::Null(crate::core::NullType::Null)))
                } else {
                    Ok(arg.clone())
                }
            }
            // 其他聚合函数...
            _ => Err(ExpressionError::InvalidOperation(format!(
                "Aggregate function not yet supported: {:?}",
                func
            ))),
        }
    }

    // 辅助方法实现
    fn add_values(&self, left: &Value, right: &Value) -> Result<Value, ExpressionError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f64)),
            _ => Err(ExpressionError::TypeError(
                "Add operation requires numeric values".to_string(),
            )),
        }
    }

    fn subtract_values(&self, left: &Value, right: &Value) -> Result<Value, ExpressionError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - *b as f64)),
            _ => Err(ExpressionError::TypeError(
                "Subtract operation requires numeric values".to_string(),
            )),
        }
    }

    fn multiply_values(&self, left: &Value, right: &Value) -> Result<Value, ExpressionError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * *b as f64)),
            _ => Err(ExpressionError::TypeError(
                "Multiply operation requires numeric values".to_string(),
            )),
        }
    }

    fn divide_values(&self, left: &Value, right: &Value) -> Result<Value, ExpressionError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    Err(ExpressionError::InvalidOperation(
                        "Division by zero".to_string(),
                    ))
                } else {
                    Ok(Value::Int(a / b))
                }
            }
            (Value::Float(a), Value::Float(b)) => {
                if *b == 0.0 {
                    Err(ExpressionError::InvalidOperation(
                        "Division by zero".to_string(),
                    ))
                } else {
                    Ok(Value::Float(a / b))
                }
            }
            (Value::Int(a), Value::Float(b)) => {
                if *b == 0.0 {
                    Err(ExpressionError::InvalidOperation(
                        "Division by zero".to_string(),
                    ))
                } else {
                    Ok(Value::Float(*a as f64 / b))
                }
            }
            (Value::Float(a), Value::Int(b)) => {
                if *b == 0 {
                    Err(ExpressionError::InvalidOperation(
                        "Division by zero".to_string(),
                    ))
                } else {
                    Ok(Value::Float(a / *b as f64))
                }
            }
            _ => Err(ExpressionError::TypeError(
                "Divide operation requires numeric values".to_string(),
            )),
        }
    }

    fn modulo_values(&self, left: &Value, right: &Value) -> Result<Value, ExpressionError> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    Err(ExpressionError::InvalidOperation(
                        "Division by zero".to_string(),
                    ))
                } else {
                    Ok(Value::Int(a % b))
                }
            }
            _ => Err(ExpressionError::TypeError(
                "Modulo operation requires integer values".to_string(),
            )),
        }
    }

    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Null(_), Value::Null(_)) => true,
            _ => false,
        }
    }

    fn compare_values(&self, left: &Value, right: &Value) -> std::cmp::Ordering {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => a.cmp(b),
            (Value::Float(a), Value::Float(b)) => {
                if a < b {
                    std::cmp::Ordering::Less
                } else if a > b {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Equal
                }
            }
            (Value::String(a), Value::String(b)) => a.cmp(b),
            _ => std::cmp::Ordering::Equal,
        }
    }

    fn logical_and(&self, left: &Value, right: &Value) -> Value {
        match (left, right) {
            (Value::Bool(a), Value::Bool(b)) => Value::Bool(*a && *b),
            _ => Value::Bool(false),
        }
    }

    fn logical_or(&self, left: &Value, right: &Value) -> Value {
        match (left, right) {
            (Value::Bool(a), Value::Bool(b)) => Value::Bool(*a || *b),
            _ => Value::Bool(false),
        }
    }

    fn logical_not(&self, value: &Value) -> Value {
        match value {
            Value::Bool(b) => Value::Bool(!b),
            _ => Value::Bool(true),
        }
    }

    fn negate_value(&self, value: &Value) -> Value {
        match value {
            Value::Int(i) => Value::Int(-i),
            Value::Float(f) => Value::Float(-f),
            _ => value.clone(),
        }
    }

    fn get_property(&self, obj: &Value, prop: &str) -> Result<Value, ExpressionError> {
        match obj {
            Value::Map(map) => map
                .get(prop)
                .cloned()
                .ok_or_else(|| ExpressionError::PropertyNotFound(prop.to_string())),
            _ => Err(ExpressionError::PropertyNotFound(format!(
                "Cannot access property on non-map value: {}",
                prop
            ))),
        }
    }

    fn cast_value(
        &self,
        value: &Value,
        target_type: &crate::expression::DataType,
    ) -> Result<Value, ExpressionError> {
        use crate::expression::DataType;

        match (value, target_type) {
            (_, DataType::Bool) => match value {
                Value::Bool(_) => Ok(value.clone()),
                Value::Int(i) => Ok(Value::Bool(*i != 0)),
                Value::Float(f) => Ok(Value::Bool(*f != 0.0)),
                Value::String(s) => Ok(Value::Bool(!s.is_empty())),
                _ => Ok(Value::Bool(false)),
            },
            (_, DataType::Int) => match value {
                Value::Int(_) => Ok(value.clone()),
                Value::Float(f) => Ok(Value::Int(*f as i64)),
                Value::String(s) => s.parse::<i64>().map(Value::Int).map_err(|_| {
                    ExpressionError::TypeError("Cannot convert string to int".to_string())
                }),
                _ => Err(ExpressionError::TypeError(
                    "Cannot convert to int".to_string(),
                )),
            },
            (_, DataType::Float) => match value {
                Value::Int(i) => Ok(Value::Float(*i as f64)),
                Value::Float(_) => Ok(value.clone()),
                Value::String(s) => s.parse::<f64>().map(Value::Float).map_err(|_| {
                    ExpressionError::TypeError("Cannot convert string to float".to_string())
                }),
                _ => Err(ExpressionError::TypeError(
                    "Cannot convert to float".to_string(),
                )),
            },
            (_, DataType::String) => match value {
                Value::String(_) => Ok(value.clone()),
                _ => Ok(Value::String(format!("{:?}", value))),
            },
            _ => Ok(value.clone()),
        }
    }

    fn eval_abs(&self, args: &[Value]) -> Result<Value, ExpressionError> {
        if args.len() != 1 {
            return Err(ExpressionError::InvalidArgumentCount("abs".to_string()));
        }

        match &args[0] {
            Value::Int(i) => Ok(Value::Int(i.abs())),
            Value::Float(f) => Ok(Value::Float(f.abs())),
            _ => Err(ExpressionError::TypeError(
                "abs expects numeric argument".to_string(),
            )),
        }
    }

    fn eval_length(&self, args: &[Value]) -> Result<Value, ExpressionError> {
        if args.len() != 1 {
            return Err(ExpressionError::InvalidArgumentCount("length".to_string()));
        }

        match &args[0] {
            Value::String(s) => Ok(Value::Int(s.len() as i64)),
            Value::List(list) => Ok(Value::Int(list.len() as i64)),
            _ => Err(ExpressionError::TypeError(
                "length expects string or list argument".to_string(),
            )),
        }
    }

    fn eval_type(&self, args: &[Value]) -> Result<Value, ExpressionError> {
        if args.len() != 1 {
            return Err(ExpressionError::InvalidArgumentCount("type".to_string()));
        }

        let type_name = match &args[0] {
            Value::Null(_) => "NULL",
            Value::Bool(_) => "BOOLEAN",
            Value::Int(_) => "INTEGER",
            Value::Float(_) => "FLOAT",
            Value::String(_) => "STRING",
            Value::List(_) => "LIST",
            Value::Map(_) => "MAP",
            Value::Vertex(_) => "VERTEX",
            Value::Edge(_) => "EDGE",
            Value::Path(_) => "PATH",
            Value::DateTime(_) => "DATETIME",
            Value::Date(_) => "DATE",
            Value::Time(_) => "TIME",
            _ => "UNKNOWN",
        };

        Ok(Value::String(type_name.to_string()))
    }
}

impl Default for DefaultExpressionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// 便捷函数：创建默认求值器
pub fn default_evaluator() -> DefaultExpressionEvaluator {
    DefaultExpressionEvaluator::new()
}

/// 便捷函数：使用默认求值器求值表达式
pub fn evaluate_expression(
    expr: &Expression,
    context: &ExpressionContext,
) -> Result<Value, ExpressionError> {
    default_evaluator().evaluate(expr, context)
}

/// 便捷函数：使用默认求值器批量求值表达式
pub fn evaluate_expressions(
    exprs: &[Expression],
    context: &ExpressionContext,
) -> Result<Vec<Value>, ExpressionError> {
    default_evaluator().evaluate_batch(exprs, context)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expression::{
        AggregateFunction, BinaryOperator, Expression, LiteralValue, UnaryOperator,
    };

    #[test]
    fn test_default_evaluator() {
        let evaluator = DefaultExpressionEvaluator::new();
        let context = ExpressionContext::default();

        // 测试字面量求值
        let expr = Expression::Literal(LiteralValue::Int(42));
        let result = evaluator.evaluate(&expr, &context).expect("Evaluation should succeed for literal values");
        assert_eq!(result, Value::Int(42));

        // 测试变量求值
        let mut ctx = ExpressionContext::default();
        ctx.set_variable("x".to_string(), Value::Int(100));

        let expr = Expression::Variable("x".to_string());
        let result = evaluator.evaluate(&expr, &ctx).expect("Evaluation should succeed for variable values");
        assert_eq!(result, Value::Int(100));
    }

    #[test]
    fn test_binary_operations() {
        let evaluator = DefaultExpressionEvaluator::new();
        let context = ExpressionContext::default();

        // 测试加法
        let left = Expression::Literal(LiteralValue::Int(10));
        let right = Expression::Literal(LiteralValue::Int(20));
        let expr = Expression::Binary {
            left: Box::new(left),
            op: BinaryOperator::Add,
            right: Box::new(right),
        };

        let result = evaluator.evaluate(&expr, &context).expect("Evaluation should succeed for binary operations");
        assert_eq!(result, Value::Int(30));
    }

    #[test]
    fn test_constant_checking() {
        let evaluator = DefaultExpressionEvaluator::new();

        // 测试常量表达式
        let constant_expr = Expression::Literal(LiteralValue::Int(42));
        assert!(evaluator.is_constant(&constant_expr));

        // 测试非常量表达式
        let variable_expr = Expression::Variable("x".to_string());
        assert!(!evaluator.is_constant(&variable_expr));
    }

    #[test]
    fn test_variable_collection() {
        let evaluator = DefaultExpressionEvaluator::new();

        let expr = Expression::Variable("x".to_string());
        let variables = evaluator.get_variables(&expr);
        assert_eq!(variables, vec!["x"]);

        // 测试复杂表达式
        let complex_expr = Expression::Binary {
            left: Box::new(Expression::Variable("x".to_string())),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Variable("y".to_string())),
        };
        let variables = evaluator.get_variables(&complex_expr);
        assert_eq!(variables, vec!["x", "y"]);
    }
}
