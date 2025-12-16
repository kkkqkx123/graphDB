//! 表达式求值器模块
//!
//! 提供各种表达式的求值逻辑，包括字面量、变量、属性和二元表达式

use crate::core::error::DBError;
use crate::core::Value;
use crate::query::executor::cypher::context::CypherExecutionContext;
use crate::query::parser::cypher::ast::expressions::*;

/// 表达式求值器
#[derive(Debug)]
pub struct ExpressionEvaluator;

impl ExpressionEvaluator {
    /// 求值表达式
    pub fn evaluate(
        &self,
        expr: &Expression,
        context: &CypherExecutionContext,
    ) -> Result<Value, DBError> {
        match expr {
            Expression::Literal(literal) => self.evaluate_literal(literal),
            Expression::Variable(name) => self.evaluate_variable(name, context),
            Expression::Property(prop_expr) => {
                self.evaluate_property_expression(prop_expr, context)
            }
            Expression::Binary(bin_expr) => self.evaluate_binary_expression(bin_expr, context),
            Expression::Unary(unary_expr) => self.evaluate_unary_expression(unary_expr, context),
            Expression::FunctionCall(func_call) => self.evaluate_function_call(func_call, context),
            Expression::List(list_expr) => self.evaluate_list_expression(list_expr, context),
            Expression::Map(map_expr) => self.evaluate_map_expression(map_expr, context),
            Expression::Case(case_expr) => self.evaluate_case_expression(case_expr, context),
            Expression::PatternExpression(_) => {
                // 模式表达式的临时实现
                Err(DBError::Query(
                    crate::core::error::QueryError::ExecutionError(
                        "模式表达式暂未实现".to_string(),
                    ),
                ))
            }
        }
    }

    /// 求值字面量表达式
    fn evaluate_literal(&self, literal: &Literal) -> Result<Value, DBError> {
        match literal {
            Literal::String(s) => Ok(Value::String(s.clone())),
            Literal::Integer(i) => Ok(Value::Int(*i)),
            Literal::Float(f) => Ok(Value::Float(*f)),
            Literal::Boolean(b) => Ok(Value::Bool(*b)),
            Literal::Null => Ok(Value::Null(crate::core::value::NullType::Null)),
        }
    }

    /// 求值变量表达式
    fn evaluate_variable(
        &self,
        name: &str,
        context: &CypherExecutionContext,
    ) -> Result<Value, DBError> {
        context.get_variable_value(name).cloned().ok_or_else(|| {
            DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                "未找到变量: {}",
                name
            )))
        })
    }

    /// 求值属性表达式
    fn evaluate_property_expression(
        &self,
        prop_expr: &PropertyExpression,
        context: &CypherExecutionContext,
    ) -> Result<Value, DBError> {
        // 求值基础表达式
        let base_value = self.evaluate(&prop_expr.expression, context)?;

        match base_value {
            Value::Vertex(vertex) => {
                // 获取节点属性
                if let Some(value) = vertex.get_property_any(&prop_expr.property_name) {
                    Ok(value.clone())
                } else {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
            }
            Value::Edge(edge) => {
                // 获取边属性
                if let Some(value) = edge.get_property(&prop_expr.property_name) {
                    Ok(value.clone())
                } else {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
            }
            _ => Ok(Value::Null(crate::core::value::NullType::Null)),
        }
    }

    /// 求值二元表达式
    fn evaluate_binary_expression(
        &self,
        bin_expr: &BinaryExpression,
        context: &CypherExecutionContext,
    ) -> Result<Value, DBError> {
        let left_value = self.evaluate(&bin_expr.left, context)?;
        let right_value = self.evaluate(&bin_expr.right, context)?;

        match bin_expr.operator {
            BinaryOperator::Equal => Ok(Value::Bool(self.values_equal(&left_value, &right_value))),
            BinaryOperator::NotEqual => {
                Ok(Value::Bool(!self.values_equal(&left_value, &right_value)))
            }
            BinaryOperator::GreaterThan => Ok(Value::Bool(
                self.compare_values(&left_value, &right_value) > 0,
            )),
            BinaryOperator::LessThan => Ok(Value::Bool(
                self.compare_values(&left_value, &right_value) < 0,
            )),
            BinaryOperator::GreaterThanOrEqual => Ok(Value::Bool(
                self.compare_values(&left_value, &right_value) >= 0,
            )),
            BinaryOperator::LessThanOrEqual => Ok(Value::Bool(
                self.compare_values(&left_value, &right_value) <= 0,
            )),
            BinaryOperator::And => {
                if let (Value::Bool(left_bool), Value::Bool(right_bool)) =
                    (&left_value, &right_value)
                {
                    Ok(Value::Bool(*left_bool && *right_bool))
                } else {
                    Ok(Value::Bool(false))
                }
            }
            BinaryOperator::Or => {
                if let (Value::Bool(left_bool), Value::Bool(right_bool)) =
                    (&left_value, &right_value)
                {
                    Ok(Value::Bool(*left_bool || *right_bool))
                } else {
                    Ok(Value::Bool(false))
                }
            }
            BinaryOperator::Xor => {
                if let (Value::Bool(left_bool), Value::Bool(right_bool)) =
                    (&left_value, &right_value)
                {
                    Ok(Value::Bool(*left_bool ^ *right_bool))
                } else {
                    Ok(Value::Bool(false))
                }
            }
            BinaryOperator::Add => self.arithmetic_add(&left_value, &right_value),
            BinaryOperator::Subtract => self.arithmetic_subtract(&left_value, &right_value),
            BinaryOperator::Multiply => self.arithmetic_multiply(&left_value, &right_value),
            BinaryOperator::Divide => self.arithmetic_divide(&left_value, &right_value),
            BinaryOperator::Modulo => self.arithmetic_modulo(&left_value, &right_value),
            BinaryOperator::Exponent => self.arithmetic_exponent(&left_value, &right_value),
            BinaryOperator::In => self.check_in(&left_value, &right_value),
            BinaryOperator::StartsWith => self.check_starts_with(&left_value, &right_value),
            BinaryOperator::EndsWith => self.check_ends_with(&left_value, &right_value),
            BinaryOperator::Contains => self.check_contains(&left_value, &right_value),
            BinaryOperator::RegexMatch => {
                // 正则表达式匹配的临时实现
                Err(DBError::Query(
                    crate::core::error::QueryError::ExecutionError(
                        "正则表达式匹配暂未实现".to_string(),
                    ),
                ))
            }
        }
    }

    /// 求值一元表达式
    fn evaluate_unary_expression(
        &self,
        unary_expr: &UnaryExpression,
        context: &CypherExecutionContext,
    ) -> Result<Value, DBError> {
        let value = self.evaluate(&unary_expr.expression, context)?;

        match unary_expr.operator {
            UnaryOperator::Not => {
                if let Value::Bool(b) = value {
                    Ok(Value::Bool(!b))
                } else {
                    Ok(Value::Bool(false))
                }
            }
            UnaryOperator::Positive => Ok(value),
            UnaryOperator::Negative => match value {
                Value::Int(i) => Ok(Value::Int(-i)),
                Value::Float(f) => Ok(Value::Float(-f)),
                _ => Ok(Value::Null(crate::core::value::NullType::Null)),
            },
        }
    }

    /// 求值函数调用
    fn evaluate_function_call(
        &self,
        _func_call: &FunctionCall,
        _context: &CypherExecutionContext,
    ) -> Result<Value, DBError> {
        // 函数调用的临时实现
        Err(DBError::Query(
            crate::core::error::QueryError::ExecutionError("函数调用暂未实现".to_string()),
        ))
    }

    /// 求值列表表达式
    fn evaluate_list_expression(
        &self,
        list_expr: &ListExpression,
        context: &CypherExecutionContext,
    ) -> Result<Value, DBError> {
        let mut elements = Vec::new();
        for element in &list_expr.elements {
            elements.push(self.evaluate(element, context)?);
        }
        Ok(Value::List(elements))
    }

    /// 求值Map表达式
    fn evaluate_map_expression(
        &self,
        map_expr: &MapExpression,
        context: &CypherExecutionContext,
    ) -> Result<Value, DBError> {
        let mut properties = std::collections::HashMap::new();
        for (key, value) in &map_expr.properties {
            properties.insert(key.clone(), self.evaluate(value, context)?);
        }
        Ok(Value::Map(properties))
    }

    /// 求值CASE表达式
    fn evaluate_case_expression(
        &self,
        _case_expr: &CaseExpression,
        _context: &CypherExecutionContext,
    ) -> Result<Value, DBError> {
        // CASE表达式的临时实现
        Err(DBError::Query(
            crate::core::error::QueryError::ExecutionError("CASE表达式暂未实现".to_string()),
        ))
    }

    /// 比较两个值是否相等
    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::String(l), Value::String(r)) => l == r,
            (Value::Int(l), Value::Int(r)) => l == r,
            (Value::Float(l), Value::Float(r)) => (l - r).abs() < f64::EPSILON,
            (Value::Bool(l), Value::Bool(r)) => l == r,
            (Value::Null(_), Value::Null(_)) => true,
            (Value::List(l), Value::List(r)) => {
                l.len() == r.len() && l.iter().zip(r.iter()).all(|(a, b)| self.values_equal(a, b))
            }
            (Value::Map(l), Value::Map(r)) => {
                l.len() == r.len()
                    && l.iter()
                        .all(|(k, v)| r.get(k).map_or(false, |rv| self.values_equal(v, rv)))
            }
            _ => false,
        }
    }

    /// 比较两个值的大小
    fn compare_values(&self, left: &Value, right: &Value) -> i32 {
        match (left, right) {
            (Value::String(l), Value::String(r)) => match l.cmp(r) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            },
            (Value::Int(l), Value::Int(r)) => match l.cmp(r) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            },
            (Value::Float(l), Value::Float(r)) => {
                match l.partial_cmp(r).unwrap_or(std::cmp::Ordering::Equal) {
                    std::cmp::Ordering::Less => -1,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Greater => 1,
                }
            }
            (Value::Bool(l), Value::Bool(r)) => match l.cmp(r) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            },
            _ => 0, // 无法比较的类型返回相等
        }
    }

    /// 算术加法
    fn arithmetic_add(&self, left: &Value, right: &Value) -> Result<Value, DBError> {
        match (left, right) {
            (Value::Int(l), Value::Int(r)) => Ok(Value::Int(l + r)),
            (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l + r)),
            (Value::Int(l), Value::Float(r)) => Ok(Value::Float(*l as f64 + r)),
            (Value::Float(l), Value::Int(r)) => Ok(Value::Float(l + *r as f64)),
            (Value::String(l), Value::String(r)) => Ok(Value::String(l.clone() + r)),
            _ => Ok(Value::Null(crate::core::value::NullType::Null)),
        }
    }

    /// 算术减法
    fn arithmetic_subtract(&self, left: &Value, right: &Value) -> Result<Value, DBError> {
        match (left, right) {
            (Value::Int(l), Value::Int(r)) => Ok(Value::Int(l - r)),
            (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l - r)),
            (Value::Int(l), Value::Float(r)) => Ok(Value::Float(*l as f64 - r)),
            (Value::Float(l), Value::Int(r)) => Ok(Value::Float(l - *r as f64)),
            _ => Ok(Value::Null(crate::core::value::NullType::Null)),
        }
    }

    /// 算术乘法
    fn arithmetic_multiply(&self, left: &Value, right: &Value) -> Result<Value, DBError> {
        match (left, right) {
            (Value::Int(l), Value::Int(r)) => Ok(Value::Int(l * r)),
            (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l * r)),
            (Value::Int(l), Value::Float(r)) => Ok(Value::Float(*l as f64 * r)),
            (Value::Float(l), Value::Int(r)) => Ok(Value::Float(l * *r as f64)),
            _ => Ok(Value::Null(crate::core::value::NullType::Null)),
        }
    }

    /// 算术除法
    fn arithmetic_divide(&self, left: &Value, right: &Value) -> Result<Value, DBError> {
        match (left, right) {
            (Value::Int(l), Value::Int(r)) => {
                if *r == 0 {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                } else {
                    Ok(Value::Int(l / r))
                }
            }
            (Value::Float(l), Value::Float(r)) => {
                if *r == 0.0 {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                } else {
                    Ok(Value::Float(l / r))
                }
            }
            (Value::Int(l), Value::Float(r)) => {
                if *r == 0.0 {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                } else {
                    Ok(Value::Float(*l as f64 / r))
                }
            }
            (Value::Float(l), Value::Int(r)) => {
                if *r == 0 {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                } else {
                    Ok(Value::Float(l / *r as f64))
                }
            }
            _ => Ok(Value::Null(crate::core::value::NullType::Null)),
        }
    }

    /// 算术取模
    fn arithmetic_modulo(&self, left: &Value, right: &Value) -> Result<Value, DBError> {
        match (left, right) {
            (Value::Int(l), Value::Int(r)) => {
                if *r == 0 {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                } else {
                    Ok(Value::Int(l % r))
                }
            }
            (Value::Float(l), Value::Float(r)) => {
                if *r == 0.0 {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                } else {
                    Ok(Value::Float(l % r))
                }
            }
            _ => Ok(Value::Null(crate::core::value::NullType::Null)),
        }
    }

    /// 算术指数
    fn arithmetic_exponent(&self, left: &Value, right: &Value) -> Result<Value, DBError> {
        match (left, right) {
            (Value::Int(l), Value::Int(r)) => Ok(Value::Float((*l as f64).powf(*r as f64))),
            (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l.powf(*r))),
            (Value::Int(l), Value::Float(r)) => Ok(Value::Float((*l as f64).powf(*r))),
            (Value::Float(l), Value::Int(r)) => Ok(Value::Float(l.powf(*r as f64))),
            _ => Ok(Value::Null(crate::core::value::NullType::Null)),
        }
    }

    /// 检查IN操作
    fn check_in(&self, left: &Value, right: &Value) -> Result<Value, DBError> {
        match right {
            Value::List(list) => Ok(Value::Bool(
                list.iter().any(|item| self.values_equal(left, item)),
            )),
            _ => Ok(Value::Bool(false)),
        }
    }

    /// 检查STARTS WITH操作
    fn check_starts_with(&self, left: &Value, right: &Value) -> Result<Value, DBError> {
        match (left, right) {
            (Value::String(l), Value::String(r)) => Ok(Value::Bool(l.starts_with(r))),
            _ => Ok(Value::Bool(false)),
        }
    }

    /// 检查ENDS WITH操作
    fn check_ends_with(&self, left: &Value, right: &Value) -> Result<Value, DBError> {
        match (left, right) {
            (Value::String(l), Value::String(r)) => Ok(Value::Bool(l.ends_with(r))),
            _ => Ok(Value::Bool(false)),
        }
    }

    /// 检查CONTAINS操作
    fn check_contains(&self, left: &Value, right: &Value) -> Result<Value, DBError> {
        match (left, right) {
            (Value::String(l), Value::String(r)) => Ok(Value::Bool(l.contains(r))),
            (Value::List(list), _) => Ok(Value::Bool(
                list.iter().any(|item| self.values_equal(item, right)),
            )),
            _ => Ok(Value::Bool(false)),
        }
    }
}

impl Default for ExpressionEvaluator {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::executor::cypher::context::{
        CypherExecutionContext, CypherVariable, CypherVariableType,
    };

    #[test]
    fn test_evaluate_literal() {
        let evaluator = ExpressionEvaluator;
        let context = CypherExecutionContext::new();

        let string_expr = Expression::Literal(Literal::String("test".to_string()));
        let result = evaluator.evaluate(&string_expr, &context).unwrap();
        assert_eq!(result, Value::String("test".to_string()));

        let int_expr = Expression::Literal(Literal::Integer(42));
        let result = evaluator.evaluate(&int_expr, &context).unwrap();
        assert_eq!(result, Value::Int(42));

        let bool_expr = Expression::Literal(Literal::Boolean(true));
        let result = evaluator.evaluate(&bool_expr, &context).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_evaluate_binary_expression() {
        let evaluator = ExpressionEvaluator;
        let context = CypherExecutionContext::new();

        // 测试相等比较
        let equal_expr = Expression::Binary(BinaryExpression {
            left: Box::new(Expression::Literal(Literal::Integer(42))),
            operator: BinaryOperator::Equal,
            right: Box::new(Expression::Literal(Literal::Integer(42))),
        });
        let result = evaluator.evaluate(&equal_expr, &context).unwrap();
        assert_eq!(result, Value::Bool(true));

        // 测试不相等比较
        let not_equal_expr = Expression::Binary(BinaryExpression {
            left: Box::new(Expression::Literal(Literal::Integer(42))),
            operator: BinaryOperator::NotEqual,
            right: Box::new(Expression::Literal(Literal::Integer(43))),
        });
        let result = evaluator.evaluate(&not_equal_expr, &context).unwrap();
        assert_eq!(result, Value::Bool(true));

        // 测试AND操作
        let and_expr = Expression::Binary(BinaryExpression {
            left: Box::new(Expression::Literal(Literal::Boolean(true))),
            operator: BinaryOperator::And,
            right: Box::new(Expression::Literal(Literal::Boolean(true))),
        });
        let result = evaluator.evaluate(&and_expr, &context).unwrap();
        assert_eq!(result, Value::Bool(true));

        // 测试OR操作
        let or_expr = Expression::Binary(BinaryExpression {
            left: Box::new(Expression::Literal(Literal::Boolean(true))),
            operator: BinaryOperator::Or,
            right: Box::new(Expression::Literal(Literal::Boolean(false))),
        });
        let result = evaluator.evaluate(&or_expr, &context).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_evaluate_variable() {
        let evaluator = ExpressionEvaluator;
        let mut context = CypherExecutionContext::new();

        // 添加变量到上下文
        let var = CypherVariable::with_value(
            "test_var".to_string(),
            CypherVariableType::Scalar,
            Value::String("test_value".to_string()),
        );
        context.add_variable(var);

        let var_expr = Expression::Variable("test_var".to_string());
        let result = evaluator.evaluate(&var_expr, &context).unwrap();
        assert_eq!(result, Value::String("test_value".to_string()));
    }

    #[test]
    fn test_evaluate_unary_expression() {
        let evaluator = ExpressionEvaluator;
        let context = CypherExecutionContext::new();

        // 测试NOT操作
        let not_expr = Expression::Unary(UnaryExpression {
            operator: UnaryOperator::Not,
            expression: Box::new(Expression::Literal(Literal::Boolean(true))),
        });
        let result = evaluator.evaluate(&not_expr, &context).unwrap();
        assert_eq!(result, Value::Bool(false));

        // 测试负号
        let neg_expr = Expression::Unary(UnaryExpression {
            operator: UnaryOperator::Negative,
            expression: Box::new(Expression::Literal(Literal::Integer(42))),
        });
        let result = evaluator.evaluate(&neg_expr, &context).unwrap();
        assert_eq!(result, Value::Int(-42));
    }

    #[test]
    fn test_arithmetic_operations() {
        let evaluator = ExpressionEvaluator;
        let context = CypherExecutionContext::new();

        // 测试加法
        let add_expr = Expression::Binary(BinaryExpression {
            left: Box::new(Expression::Literal(Literal::Integer(10))),
            operator: BinaryOperator::Add,
            right: Box::new(Expression::Literal(Literal::Integer(5))),
        });
        let result = evaluator.evaluate(&add_expr, &context).unwrap();
        assert_eq!(result, Value::Int(15));

        // 测试字符串连接
        let concat_expr = Expression::Binary(BinaryExpression {
            left: Box::new(Expression::Literal(Literal::String("Hello".to_string()))),
            operator: BinaryOperator::Add,
            right: Box::new(Expression::Literal(Literal::String(" World".to_string()))),
        });
        let result = evaluator.evaluate(&concat_expr, &context).unwrap();
        assert_eq!(result, Value::String("Hello World".to_string()));
    }

    #[test]
    fn test_string_operations() {
        let evaluator = ExpressionEvaluator;
        let context = CypherExecutionContext::new();

        // 测试STARTS WITH
        let starts_with_expr = Expression::Binary(BinaryExpression {
            left: Box::new(Expression::Literal(Literal::String(
                "Hello World".to_string(),
            ))),
            operator: BinaryOperator::StartsWith,
            right: Box::new(Expression::Literal(Literal::String("Hello".to_string()))),
        });
        let result = evaluator.evaluate(&starts_with_expr, &context).unwrap();
        assert_eq!(result, Value::Bool(true));

        // 测试ENDS WITH
        let ends_with_expr = Expression::Binary(BinaryExpression {
            left: Box::new(Expression::Literal(Literal::String(
                "Hello World".to_string(),
            ))),
            operator: BinaryOperator::EndsWith,
            right: Box::new(Expression::Literal(Literal::String("World".to_string()))),
        });
        let result = evaluator.evaluate(&ends_with_expr, &context).unwrap();
        assert_eq!(result, Value::Bool(true));

        // 测试CONTAINS
        let contains_expr = Expression::Binary(BinaryExpression {
            left: Box::new(Expression::Literal(Literal::String(
                "Hello World".to_string(),
            ))),
            operator: BinaryOperator::Contains,
            right: Box::new(Expression::Literal(Literal::String("lo Wo".to_string()))),
        });
        let result = evaluator.evaluate(&contains_expr, &context).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_list_expression() {
        let evaluator = ExpressionEvaluator;
        let context = CypherExecutionContext::new();

        let list_expr = Expression::List(ListExpression {
            elements: vec![
                Expression::Literal(Literal::Integer(1)),
                Expression::Literal(Literal::Integer(2)),
                Expression::Literal(Literal::Integer(3)),
            ],
        });
        let result = evaluator.evaluate(&list_expr, &context).unwrap();

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
    fn test_map_expression() {
        let evaluator = ExpressionEvaluator;
        let context = CypherExecutionContext::new();

        let mut properties = std::collections::HashMap::new();
        properties.insert(
            "key1".to_string(),
            Expression::Literal(Literal::String("value1".to_string())),
        );
        properties.insert(
            "key2".to_string(),
            Expression::Literal(Literal::Integer(42)),
        );

        let map_expr = Expression::Map(MapExpression { properties });
        let result = evaluator.evaluate(&map_expr, &context).unwrap();

        if let Value::Map(props) = result {
            assert_eq!(props.len(), 2);
            assert_eq!(
                props.get("key1"),
                Some(&Value::String("value1".to_string()))
            );
            assert_eq!(props.get("key2"), Some(&Value::Int(42)));
        } else {
            panic!("Expected map value");
        }
    }
}
