//! 表达式求值器 V2 - 重构版本
//!
//! 分离求值逻辑和上下文访问，提供更清晰的接口设计

use super::expr_type::Expression;
use crate::core::error::{DBError, DBResult};
use crate::core::Value;

/// 表达式上下文访问接口
///
/// 这个 trait 定义了表达式求值时需要的上下文访问方法，
/// 将上下文访问与求值逻辑分离，提高可测试性和可扩展性
pub trait ExpressionContext {
    /// 获取变量值
    ///
    /// # 参数
    /// - `name`: 变量名
    ///
    /// # 返回
    /// - `Some(&Value)`: 变量值
    /// - `None`: 变量不存在
    fn get_variable(&self, name: &str) -> Option<&Value>;

    /// 获取对象属性值
    ///
    /// # 参数
    /// - `object`: 对象值
    /// - `property`: 属性名
    ///
    /// # 返回
    /// - `Ok(&Value)`: 属性值
    /// - `Err(DBError)`: 访问错误
    fn get_property(&self, object: &Value, property: &str) -> DBResult<&Value>;

    /// 获取函数
    ///
    /// # 参数
    /// - `name`: 函数名
    ///
    /// # 返回
    /// - `Some(&dyn Function)`: 函数实现
    /// - `None`: 函数不存在
    fn get_function(&self, name: &str) -> Option<&dyn Function>;
}

/// 函数接口
///
/// 定义了表达式系统中函数的基本接口
pub trait Function {
    /// 获取函数名
    fn name(&self) -> &str;

    /// 获取参数数量
    ///
    /// # 返回
    /// - `Some(usize)`: 固定参数数量
    /// - `None`: 可变参数
    fn arg_count(&self) -> Option<usize>;

    /// 调用函数
    ///
    /// # 参数
    /// - `args`: 参数列表
    /// - `ctx`: 表达式上下文
    ///
    /// # 返回
    /// - `Ok(Value)`: 函数结果
    /// - `Err(DBError)`: 执行错误
    fn call(&self, args: &[Value], ctx: &dyn ExpressionContext) -> DBResult<Value>;
}

/// 表达式求值器接口
///
/// 定义了表达式求值的基本接口，支持不同的求值策略
pub trait ExpressionEvaluator {
    /// 求值表达式
    ///
    /// # 参数
    /// - `expr`: 表达式
    /// - `ctx`: 表达式上下文
    ///
    /// # 返回
    /// - `Ok(Value)`: 求值结果
    /// - `Err(DBError)`: 求值错误
    fn evaluate(&self, expr: &Expression, ctx: &dyn ExpressionContext) -> DBResult<Value>;
}

/// 默认表达式求值器实现
///
/// 提供标准的表达式求值逻辑
pub struct DefaultExpressionEvaluator;

impl DefaultExpressionEvaluator {
    /// 创建新的默认求值器
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultExpressionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpressionEvaluator for DefaultExpressionEvaluator {
    fn evaluate(&self, expr: &Expression, ctx: &dyn ExpressionContext) -> DBResult<Value> {
        match expr {
            // 字面量常量
            Expression::Constant(value) => Ok(value.clone()),

            // 变量访问
            Expression::Variable(name) => ctx
                .get_variable(name)
                .cloned()
                .ok_or_else(|| DBError::Expression(format!("Variable '{}' not found", name))),

            // 属性访问
            Expression::Property { object, property } => {
                let obj_value = self.evaluate(object, ctx)?;
                ctx.get_property(&obj_value, property).cloned()
            }

            // 二元操作
            Expression::Binary { left, op, right } => {
                let left_val = self.evaluate(left, ctx)?;
                let right_val = self.evaluate(right, ctx)?;
                self.evaluate_binary_op(&left_val, op, &right_val)
            }

            // 一元操作
            Expression::Unary { op, operand } => {
                let operand_val = self.evaluate(operand, ctx)?;
                self.evaluate_unary_op(op, &operand_val)
            }

            // 函数调用
            Expression::Function { name, args } => {
                let func = ctx
                    .get_function(name)
                    .ok_or_else(|| DBError::Expression(format!("Function '{}' not found", name)))?;

                let mut arg_values = Vec::with_capacity(args.len());
                for arg in args {
                    arg_values.push(self.evaluate(arg, ctx)?);
                }

                func.call(&arg_values, ctx)
            }

            // 聚合函数
            Expression::Aggregate {
                func,
                arg,
                distinct,
            } => {
                let arg_val = self.evaluate(arg, ctx)?;
                self.evaluate_aggregate(func, &arg_val, *distinct, ctx)
            }

            // 列表
            Expression::List(items) => {
                let mut values = Vec::with_capacity(items.len());
                for item in items {
                    values.push(self.evaluate(item, ctx)?);
                }
                Ok(Value::List(values))
            }

            // 映射
            Expression::Map(pairs) => {
                let mut map = std::collections::HashMap::new();
                for (key, value_expr) in pairs {
                    let value = self.evaluate(value_expr, ctx)?;
                    map.insert(key.clone(), value);
                }
                Ok(Value::Map(map))
            }

            // 类型转换
            Expression::TypeCasting { expr, target_type } => {
                let value = self.evaluate(expr, ctx)?;
                self.cast_to_type(&value, target_type)
            }

            // 条件表达式
            Expression::Case {
                conditions,
                default,
            } => {
                for (condition, value) in conditions {
                    let cond_result = self.evaluate(condition, ctx)?;
                    if self.is_truthy(&cond_result) {
                        return self.evaluate(value, ctx);
                    }
                }

                if let Some(default_expr) = default {
                    self.evaluate(default_expr, ctx)
                } else {
                    Ok(Value::Null(crate::core::NullType::Null))
                }
            }

            // 其他表达式类型的处理...
            _ => Err(DBError::Expression(format!(
                "Unsupported expression type: {:?}",
                expr
            ))),
        }
    }
}

impl DefaultExpressionEvaluator {
    /// 求值二元操作
    fn evaluate_binary_op(
        &self,
        left: &Value,
        op: &BinaryOperator,
        right: &Value,
    ) -> DBResult<Value> {
        match op {
            BinaryOperator::Add => self.add_values(left, right),
            BinaryOperator::Subtract => self.subtract_values(left, right),
            BinaryOperator::Multiply => self.multiply_values(left, right),
            BinaryOperator::Divide => self.divide_values(left, right),
            BinaryOperator::Modulo => self.modulo_values(left, right),
            BinaryOperator::Equal => Ok(Value::Bool(self.equals_values(left, right))),
            BinaryOperator::NotEqual => Ok(Value::Bool(!self.equals_values(left, right))),
            BinaryOperator::LessThan => Ok(Value::Bool(self.less_than_values(left, right))),
            BinaryOperator::LessThanOrEqual => {
                Ok(Value::Bool(self.less_than_or_equal_values(left, right)))
            }
            BinaryOperator::GreaterThan => Ok(Value::Bool(self.greater_than_values(left, right))),
            BinaryOperator::GreaterThanOrEqual => {
                Ok(Value::Bool(self.greater_than_or_equal_values(left, right)))
            }
            BinaryOperator::And => Ok(Value::Bool(self.is_truthy(left) && self.is_truthy(right))),
            BinaryOperator::Or => Ok(Value::Bool(self.is_truthy(left) || self.is_truthy(right))),
        }
    }

    /// 求值一元操作
    fn evaluate_unary_op(&self, op: &UnaryOperator, operand: &Value) -> DBResult<Value> {
        match op {
            UnaryOperator::Plus => self.unary_plus(operand),
            UnaryOperator::Minus => self.unary_minus(operand),
            UnaryOperator::Not => Ok(Value::Bool(!self.is_truthy(operand))),
            UnaryOperator::IsNull => Ok(Value::Bool(matches!(operand, Value::Null(_)))),
            UnaryOperator::IsNotNull => Ok(Value::Bool(!matches!(operand, Value::Null(_)))),
        }
    }

    /// 求值聚合函数
    fn evaluate_aggregate(
        &self,
        func: &AggregateFunction,
        arg: &Value,
        distinct: bool,
        _ctx: &dyn ExpressionContext,
    ) -> DBResult<Value> {
        match func {
            AggregateFunction::Count => {
                match arg {
                    Value::List(items) => {
                        if distinct {
                            let unique_items: std::collections::HashSet<_> = items.iter().collect();
                            Ok(Value::Int(unique_items.len() as i64))
                        } else {
                            Ok(Value::Int(items.len() as i64))
                        }
                    }
                    _ => Ok(Value::Int(1)), // 单个值计数为1
                }
            }
            AggregateFunction::Sum => match arg {
                Value::List(items) => {
                    let sum: f64 = items.iter().map(|v| self.to_number(v)).sum();
                    Ok(Value::Float(sum))
                }
                _ => Ok(self.to_number(arg).into()),
            },
            AggregateFunction::Avg => match arg {
                Value::List(items) => {
                    if items.is_empty() {
                        return Ok(Value::Null(crate::core::NullType::Null));
                    }
                    let sum: f64 = items.iter().map(|v| self.to_number(v)).sum();
                    Ok(Value::Float(sum / items.len() as f64))
                }
                _ => Ok(self.to_number(arg).into()),
            },
            AggregateFunction::Min => match arg {
                Value::List(items) => {
                    if items.is_empty() {
                        return Ok(Value::Null(crate::core::NullType::Null));
                    }
                    let min = items
                        .iter()
                        .min_by(|a, b| self.compare_values(a, b))
                        .unwrap();
                    Ok(min.clone())
                }
                _ => Ok(arg.clone()),
            },
            AggregateFunction::Max => match arg {
                Value::List(items) => {
                    if items.is_empty() {
                        return Ok(Value::Null(crate::core::NullType::Null));
                    }
                    let max = items
                        .iter()
                        .max_by(|a, b| self.compare_values(a, b))
                        .unwrap();
                    Ok(max.clone())
                }
                _ => Ok(arg.clone()),
            },
        }
    }

    /// 类型转换
    fn cast_to_type(&self, value: &Value, target_type: &str) -> DBResult<Value> {
        match target_type {
            "int" => Ok(Value::Int(self.to_number(value) as i64)),
            "float" => Ok(Value::Float(self.to_number(value))),
            "string" => Ok(Value::String(self.to_string(value))),
            "bool" => Ok(Value::Bool(self.is_truthy(value))),
            _ => Err(DBError::Expression(format!(
                "Unsupported target type: {}",
                target_type
            ))),
        }
    }

    // 辅助方法

    fn add_values(&self, left: &Value, right: &Value) -> DBResult<Value> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f64)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            _ => Err(DBError::Expression(
                "Invalid operands for addition".to_string(),
            )),
        }
    }

    fn subtract_values(&self, left: &Value, right: &Value) -> DBResult<Value> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - *b as f64)),
            _ => Err(DBError::Expression(
                "Invalid operands for subtraction".to_string(),
            )),
        }
    }

    fn multiply_values(&self, left: &Value, right: &Value) -> DBResult<Value> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * *b as f64)),
            _ => Err(DBError::Expression(
                "Invalid operands for multiplication".to_string(),
            )),
        }
    }

    fn divide_values(&self, left: &Value, right: &Value) -> DBResult<Value> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    return Err(DBError::Expression("Division by zero".to_string()));
                }
                Ok(Value::Int(a / b))
            }
            (Value::Float(a), Value::Float(b)) => {
                if *b == 0.0 {
                    return Err(DBError::Expression("Division by zero".to_string()));
                }
                Ok(Value::Float(a / b))
            }
            (Value::Int(a), Value::Float(b)) => {
                if *b == 0.0 {
                    return Err(DBError::Expression("Division by zero".to_string()));
                }
                Ok(Value::Float(*a as f64 / b))
            }
            (Value::Float(a), Value::Int(b)) => {
                if *b == 0 {
                    return Err(DBError::Expression("Division by zero".to_string()));
                }
                Ok(Value::Float(a / *b as f64))
            }
            _ => Err(DBError::Expression(
                "Invalid operands for division".to_string(),
            )),
        }
    }

    fn modulo_values(&self, left: &Value, right: &Value) -> DBResult<Value> {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    return Err(DBError::Expression("Modulo by zero".to_string()));
                }
                Ok(Value::Int(a % b))
            }
            _ => Err(DBError::Expression(
                "Invalid operands for modulo".to_string(),
            )),
        }
    }

    fn equals_values(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Null(_), Value::Null(_)) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
            (Value::String(a), Value::String(b)) => a == b,
            _ => false,
        }
    }

    fn less_than_values(&self, left: &Value, right: &Value) -> bool {
        self.compare_values(left, right) == std::cmp::Ordering::Less
    }

    fn less_than_or_equal_values(&self, left: &Value, right: &Value) -> bool {
        let ord = self.compare_values(left, right);
        ord == std::cmp::Ordering::Less || ord == std::cmp::Ordering::Equal
    }

    fn greater_than_values(&self, left: &Value, right: &Value) -> bool {
        self.compare_values(left, right) == std::cmp::Ordering::Greater
    }

    fn greater_than_or_equal_values(&self, left: &Value, right: &Value) -> bool {
        let ord = self.compare_values(left, right);
        ord == std::cmp::Ordering::Greater || ord == std::cmp::Ordering::Equal
    }

    fn compare_values(&self, left: &Value, right: &Value) -> std::cmp::Ordering {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => a.cmp(b),
            (Value::Float(a), Value::Float(b)) => {
                a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
            }
            (Value::Int(a), Value::Float(b)) => (*a as f64)
                .partial_cmp(b)
                .unwrap_or(std::cmp::Ordering::Equal),
            (Value::Float(a), Value::Int(b)) => a
                .partial_cmp(&(*b as f64))
                .unwrap_or(std::cmp::Ordering::Equal),
            (Value::String(a), Value::String(b)) => a.cmp(b),
            _ => std::cmp::Ordering::Equal,
        }
    }

    fn unary_plus(&self, operand: &Value) -> DBResult<Value> {
        match operand {
            Value::Int(_) | Value::Float(_) => Ok(operand.clone()),
            _ => Err(DBError::Expression(
                "Invalid operand for unary plus".to_string(),
            )),
        }
    }

    fn unary_minus(&self, operand: &Value) -> DBResult<Value> {
        match operand {
            Value::Int(a) => Ok(Value::Int(-a)),
            Value::Float(a) => Ok(Value::Float(-a)),
            _ => Err(DBError::Expression(
                "Invalid operand for unary minus".to_string(),
            )),
        }
    }

    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Bool(b) => *b,
            Value::Null(_) => false,
            Value::Int(0) => false,
            Value::Float(0.0) => false,
            Value::String(s) => !s.is_empty(),
            Value::List(l) => !l.is_empty(),
            Value::Map(m) => !m.is_empty(),
            _ => true,
        }
    }

    fn to_number(&self, value: &Value) -> f64 {
        match value {
            Value::Int(a) => *a as f64,
            Value::Float(a) => *a,
            Value::Bool(a) => {
                if *a {
                    1.0
                } else {
                    0.0
                }
            }
            Value::String(a) => a.parse().unwrap_or(0.0),
            _ => 0.0,
        }
    }

    fn to_string(&self, value: &Value) -> String {
        match value {
            Value::Bool(a) => a.to_string(),
            Value::Int(a) => a.to_string(),
            Value::Float(a) => a.to_string(),
            Value::String(a) => a.clone(),
            Value::Null(_) => "null".to_string(),
            Value::List(a) => format!(
                "[{}]",
                a.iter()
                    .map(|v| self.to_string(v))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Map(a) => {
                let pairs: Vec<String> = a
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, self.to_string(v)))
                    .collect();
                format!("{{{}}}", pairs.join(", "))
            }
            _ => "unknown".to_string(),
        }
    }
}

// 操作符类型定义（临时，后续会移到专门的模块）
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Plus,
    Minus,
    Not,
    IsNull,
    IsNotNull,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AggregateFunction {
    Count,
    Sum,
    Avg,
    Min,
    Max,
}

// 为数字类型实现转换到 Value
impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Float(value)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Int(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct TestContext {
        variables: HashMap<String, Value>,
    }

    impl TestContext {
        fn new() -> Self {
            Self {
                variables: HashMap::new(),
            }
        }

        fn with_variable(mut self, name: &str, value: Value) -> Self {
            self.variables.insert(name.to_string(), value);
            self
        }
    }

    impl ExpressionContext for TestContext {
        fn get_variable(&self, name: &str) -> Option<&Value> {
            self.variables.get(name)
        }

        fn get_property(&self, _object: &Value, _property: &str) -> DBResult<&Value> {
            Err(DBError::Expression(
                "Property access not supported in test context".to_string(),
            ))
        }

        fn get_function(&self, _name: &str) -> Option<&dyn Function> {
            None
        }
    }

    #[test]
    fn test_constant_evaluation() {
        let evaluator = DefaultExpressionEvaluator::new();
        let context = TestContext::new();

        let expr = Expression::Constant(Value::Int(42));
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_variable_evaluation() {
        let evaluator = DefaultExpressionEvaluator::new();
        let context = TestContext::new().with_variable("x", Value::Int(100));

        let expr = Expression::Variable("x".to_string());
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::Int(100));
    }

    #[test]
    fn test_binary_operations() {
        let evaluator = DefaultExpressionEvaluator::new();
        let context = TestContext::new();

        // 测试加法
        let expr = Expression::Binary {
            left: Box::new(Expression::Constant(Value::Int(10))),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Constant(Value::Int(20))),
        };
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::Int(30));

        // 测试比较
        let expr = Expression::Binary {
            left: Box::new(Expression::Constant(Value::Int(10))),
            op: BinaryOperator::LessThan,
            right: Box::new(Expression::Constant(Value::Int(20))),
        };
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_unary_operations() {
        let evaluator = DefaultExpressionEvaluator::new();
        let context = TestContext::new();

        // 测试一元减
        let expr = Expression::Unary {
            op: UnaryOperator::Minus,
            operand: Box::new(Expression::Constant(Value::Int(10))),
        };
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::Int(-10));

        // 测试逻辑非
        let expr = Expression::Unary {
            op: UnaryOperator::Not,
            operand: Box::new(Expression::Constant(Value::Bool(true))),
        };
        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_list_evaluation() {
        let evaluator = DefaultExpressionEvaluator::new();
        let context = TestContext::new();

        let expr = Expression::List(vec![
            Expression::Constant(Value::Int(1)),
            Expression::Constant(Value::Int(2)),
            Expression::Constant(Value::Int(3)),
        ]);
        let result = evaluator.evaluate(&expr, &context).unwrap();

        if let Value::List(items) = result {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], Value::Int(1));
            assert_eq!(items[1], Value::Int(2));
            assert_eq!(items[2], Value::Int(3));
        } else {
            panic!("Expected list value");
        }
    }

    #[test]
    fn test_aggregate_count() {
        let evaluator = DefaultExpressionEvaluator::new();
        let context = TestContext::new();

        let list_expr = Expression::List(vec![
            Expression::Constant(Value::Int(1)),
            Expression::Constant(Value::Int(2)),
            Expression::Constant(Value::Int(3)),
        ]);

        let expr = Expression::Aggregate {
            func: AggregateFunction::Count,
            arg: Box::new(list_expr),
            distinct: false,
        };

        let result = evaluator.evaluate(&expr, &context).unwrap();
        assert_eq!(result, Value::Int(3));
    }
}
