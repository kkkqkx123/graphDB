use crate::core::{ExpressionError, Value};
use crate::expression::context::ExpressionContextCore;
use crate::expression::{Expression, ExpressionContext, LiteralValue};
use crate::query::parser::cypher::ast::expressions::{
    BinaryExpression, BinaryOperator, CaseAlternative, CaseExpression,
    Expression as CypherExpression, FunctionCall, ListExpression, Literal as CypherLiteral,
    MapExpression, PatternExpression, PropertyExpression, UnaryExpression, UnaryOperator,
};

/// Cypher表达式评估器
///
/// 专注于Cypher表达式的直接评估，不包含转换和优化逻辑，
/// 保持职责单一。
pub struct CypherEvaluator;

impl CypherEvaluator {
    /// 直接评估Cypher表达式
    pub fn evaluate_cypher(
        cypher_expr: &CypherExpression,
        context: &ExpressionContext,
    ) -> Result<Value, ExpressionError> {
        match cypher_expr {
            CypherExpression::Literal(literal) => Self::evaluate_cypher_literal(literal),
            CypherExpression::Variable(name) => Self::evaluate_cypher_variable(name, context),
            CypherExpression::Property(prop_expr) => {
                Self::evaluate_cypher_property(prop_expr, context)
            }
            CypherExpression::FunctionCall(func_call) => {
                Self::evaluate_cypher_function_call(func_call, context)
            }
            CypherExpression::Binary(bin_expr) => Self::evaluate_cypher_binary(bin_expr, context),
            CypherExpression::Unary(unary_expr) => Self::evaluate_cypher_unary(unary_expr, context),
            CypherExpression::Case(case_expr) => Self::evaluate_cypher_case(case_expr, context),
            CypherExpression::List(list_expr) => Self::evaluate_cypher_list(list_expr, context),
            CypherExpression::Map(map_expr) => Self::evaluate_cypher_map(map_expr, context),
            CypherExpression::PatternExpression(pattern_expr) => {
                Self::evaluate_cypher_pattern(pattern_expr, context)
            }
        }
    }

    /// 评估Cypher字面量
    fn evaluate_cypher_literal(literal: &CypherLiteral) -> Result<Value, ExpressionError> {
        match literal {
            CypherLiteral::String(s) => Ok(Value::String(s.clone())),
            CypherLiteral::Integer(i) => Ok(Value::Int(*i)),
            CypherLiteral::Float(f) => Ok(Value::Float(*f)),
            CypherLiteral::Boolean(b) => Ok(Value::Bool(*b)),
            CypherLiteral::Null => Ok(Value::Null(crate::core::NullType::Null)),
        }
    }

    /// 评估Cypher变量
    fn evaluate_cypher_variable(
        name: &str,
        context: &ExpressionContext,
    ) -> Result<Value, ExpressionError> {
        context
            .get_variable(name)
            .ok_or_else(|| ExpressionError::PropertyNotFound(format!("Variable ${}", name)))
    }

    /// 评估Cypher属性表达式
    fn evaluate_cypher_property(
        prop_expr: &PropertyExpression,
        context: &ExpressionContext,
    ) -> Result<Value, ExpressionError> {
        let object_value = Self::evaluate_cypher(&prop_expr.expression, context)?;

        match object_value {
            Value::Map(map) => map
                .get(&prop_expr.property_name)
                .cloned()
                .ok_or_else(|| ExpressionError::PropertyNotFound(prop_expr.property_name.clone())),
            Value::Vertex(vertex) => {
                if let Some(value) = vertex.get_property_any(&prop_expr.property_name) {
                    Ok(value.clone())
                } else {
                    Ok(Value::Null(crate::core::NullType::Null))
                }
            }
            Value::Edge(edge) => {
                if let Some(value) = edge.get_property(&prop_expr.property_name) {
                    Ok(value.clone())
                } else {
                    Ok(Value::Null(crate::core::NullType::Null))
                }
            }
            _ => Ok(Value::Null(crate::core::NullType::Null)),
        }
    }

    /// 评估Cypher函数调用
    fn evaluate_cypher_function_call(
        func_call: &FunctionCall,
        context: &ExpressionContext,
    ) -> Result<Value, ExpressionError> {
        // 将Cypher函数调用转换为统一的函数调用
        let args: Result<Vec<Expression>, ExpressionError> = func_call
            .arguments
            .iter()
            .map(|arg| {
                super::expression_converter::ExpressionConverter::convert_cypher_to_unified(arg)
            })
            .collect();

        let unified_func = Expression::Function {
            name: func_call.function_name.clone(),
            args: args?,
        };

        // 使用ExpressionEvaluator评估统一函数
        crate::expression::evaluator::ExpressionEvaluator::new().evaluate(&unified_func, context)
    }

    /// 评估Cypher二元表达式
    fn evaluate_cypher_binary(
        bin_expr: &BinaryExpression,
        context: &ExpressionContext,
    ) -> Result<Value, ExpressionError> {
        let left_value = Self::evaluate_cypher(&bin_expr.left, context)?;
        let right_value = Self::evaluate_cypher(&bin_expr.right, context)?;

        Self::evaluate_binary_operation(&left_value, &bin_expr.operator, &right_value)
    }

    /// 评估Cypher一元表达式
    fn evaluate_cypher_unary(
        unary_expr: &UnaryExpression,
        context: &ExpressionContext,
    ) -> Result<Value, ExpressionError> {
        let value = Self::evaluate_cypher(&unary_expr.expression, context)?;

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
                _ => Ok(Value::Null(crate::core::NullType::Null)),
            },
        }
    }

    /// 评估Cypher CASE表达式
    fn evaluate_cypher_case(
        case_expr: &CaseExpression,
        context: &ExpressionContext,
    ) -> Result<Value, ExpressionError> {
        for alternative in &case_expr.alternatives {
            let cond_result = Self::evaluate_cypher(&alternative.when_expression, context)?;
            if crate::expression::comparison::value_to_bool(&cond_result) {
                return Self::evaluate_cypher(&alternative.then_expression, context);
            }
        }

        if let Some(default_expr) = &case_expr.default_alternative {
            Self::evaluate_cypher(default_expr, context)
        } else {
            Ok(Value::Null(crate::core::NullType::Null))
        }
    }

    /// 评估Cypher列表表达式
    fn evaluate_cypher_list(
        list_expr: &ListExpression,
        context: &ExpressionContext,
    ) -> Result<Value, ExpressionError> {
        let mut elements = Vec::new();
        for element in &list_expr.elements {
            elements.push(Self::evaluate_cypher(element, context)?);
        }
        Ok(Value::List(elements))
    }

    /// 评估Cypher映射表达式
    fn evaluate_cypher_map(
        map_expr: &MapExpression,
        context: &ExpressionContext,
    ) -> Result<Value, ExpressionError> {
        let mut properties = std::collections::HashMap::new();
        for (key, value) in &map_expr.properties {
            properties.insert(key.clone(), Self::evaluate_cypher(value, context)?);
        }
        Ok(Value::Map(properties))
    }

    /// 评估Cypher模式表达式
    fn evaluate_cypher_pattern(
        pattern_expr: &PatternExpression,
        _context: &ExpressionContext,
    ) -> Result<Value, ExpressionError> {
        // 将模式表达式转换为路径表达式并评估
        // 这里需要更复杂的实现来处理图模式匹配
        // 暂时返回模式的字符串表示
        Ok(Value::String(format!("{:?}", pattern_expr.pattern)))
    }

    /// 评估二元操作
    fn evaluate_binary_operation(
        left: &Value,
        op: &BinaryOperator,
        right: &Value,
    ) -> Result<Value, ExpressionError> {
        match op {
            BinaryOperator::Equal => Ok(Value::Bool(crate::expression::comparison::values_equal(
                left, right,
            ))),
            BinaryOperator::NotEqual => Ok(Value::Bool(
                !crate::expression::comparison::values_equal(left, right),
            )),
            BinaryOperator::GreaterThan => Ok(Value::Bool(
                crate::expression::comparison::compare_values(left, right) > 0,
            )),
            BinaryOperator::LessThan => Ok(Value::Bool(
                crate::expression::comparison::compare_values(left, right) < 0,
            )),
            BinaryOperator::GreaterThanOrEqual => Ok(Value::Bool(
                crate::expression::comparison::compare_values(left, right) >= 0,
            )),
            BinaryOperator::LessThanOrEqual => Ok(Value::Bool(
                crate::expression::comparison::compare_values(left, right) <= 0,
            )),
            BinaryOperator::And => {
                if let (Value::Bool(left_bool), Value::Bool(right_bool)) = (left, right) {
                    Ok(Value::Bool(*left_bool && *right_bool))
                } else {
                    Ok(Value::Bool(false))
                }
            }
            BinaryOperator::Or => {
                if let (Value::Bool(left_bool), Value::Bool(right_bool)) = (left, right) {
                    Ok(Value::Bool(*left_bool || *right_bool))
                } else {
                    Ok(Value::Bool(false))
                }
            }
            BinaryOperator::Xor => {
                if let (Value::Bool(left_bool), Value::Bool(right_bool)) = (left, right) {
                    Ok(Value::Bool(*left_bool ^ *right_bool))
                } else {
                    Ok(Value::Bool(false))
                }
            }
            BinaryOperator::Add => crate::expression::arithmetic::arithmetic_add(left, right),
            BinaryOperator::Subtract => {
                crate::expression::arithmetic::arithmetic_subtract(left, right)
            }
            BinaryOperator::Multiply => {
                crate::expression::arithmetic::arithmetic_multiply(left, right)
            }
            BinaryOperator::Divide => crate::expression::arithmetic::arithmetic_divide(left, right),
            BinaryOperator::Modulo => crate::expression::arithmetic::arithmetic_modulo(left, right),
            BinaryOperator::Exponent => {
                crate::expression::arithmetic::arithmetic_exponent(left, right)
            }
            BinaryOperator::In => crate::expression::comparison::check_in(left, right),
            BinaryOperator::StartsWith => {
                crate::expression::comparison::check_starts_with(left, right)
            }
            BinaryOperator::EndsWith => crate::expression::comparison::check_ends_with(left, right),
            BinaryOperator::Contains => crate::expression::comparison::check_contains(left, right),
            BinaryOperator::RegexMatch => {
                // 简化的正则匹配实现
                match (left, right) {
                    (Value::String(text), Value::String(pattern)) => {
                        // 简单的包含检查作为正则匹配的替代
                        Ok(Value::Bool(text.contains(pattern.as_str())))
                    }
                    _ => Ok(Value::Bool(false)),
                }
            }
        }
    }

    /// 批量评估Cypher表达式
    pub fn evaluate_cypher_batch(
        cypher_exprs: &[CypherExpression],
        context: &ExpressionContext,
    ) -> Result<Vec<Value>, ExpressionError> {
        let mut results = Vec::new();
        for expr in cypher_exprs {
            results.push(Self::evaluate_cypher(expr, context)?);
        }
        Ok(results)
    }

    /// 检查Cypher表达式是否为常量
    pub fn is_cypher_constant(cypher_expr: &CypherExpression) -> bool {
        match cypher_expr {
            CypherExpression::Literal(_) => true,
            CypherExpression::List(list_expr) => {
                list_expr.elements.iter().all(Self::is_cypher_constant)
            }
            CypherExpression::Map(map_expr) => {
                map_expr.properties.values().all(Self::is_cypher_constant)
            }
            _ => false,
        }
    }

    /// 获取Cypher表达式中使用的所有变量
    pub fn get_cypher_variables(cypher_expr: &CypherExpression) -> Vec<String> {
        let mut variables = Vec::new();
        Self::collect_cypher_variables(cypher_expr, &mut variables);
        variables.sort();
        variables.dedup();
        variables
    }

    /// 递归收集Cypher表达式中的变量
    fn collect_cypher_variables(cypher_expr: &CypherExpression, variables: &mut Vec<String>) {
        match cypher_expr {
            CypherExpression::Variable(name) => {
                if !variables.contains(name) {
                    variables.push(name.clone());
                }
            }
            CypherExpression::Property(prop_expr) => {
                Self::collect_cypher_variables(&prop_expr.expression, variables);
            }
            CypherExpression::Binary(bin_expr) => {
                Self::collect_cypher_variables(&bin_expr.left, variables);
                Self::collect_cypher_variables(&bin_expr.right, variables);
            }
            CypherExpression::Unary(unary_expr) => {
                Self::collect_cypher_variables(&unary_expr.expression, variables);
            }
            CypherExpression::FunctionCall(func_call) => {
                for arg in &func_call.arguments {
                    Self::collect_cypher_variables(arg, variables);
                }
            }
            CypherExpression::List(list_expr) => {
                for element in &list_expr.elements {
                    Self::collect_cypher_variables(element, variables);
                }
            }
            CypherExpression::Map(map_expr) => {
                for (_, value) in &map_expr.properties {
                    Self::collect_cypher_variables(value, variables);
                }
            }
            CypherExpression::Case(case_expr) => {
                for alternative in &case_expr.alternatives {
                    Self::collect_cypher_variables(&alternative.when_expression, variables);
                    Self::collect_cypher_variables(&alternative.then_expression, variables);
                }
                if let Some(default_expr) = &case_expr.default_alternative {
                    Self::collect_cypher_variables(default_expr, variables);
                }
            }
            CypherExpression::PatternExpression(_) => {
                // 模式表达式的变量收集需要更复杂的实现
            }
            CypherExpression::Literal(_) => {
                // 字面量不包含变量
            }
        }
    }

    /// 检查Cypher表达式是否包含聚合函数
    pub fn contains_cypher_aggregate(cypher_expr: &CypherExpression) -> bool {
        match cypher_expr {
            CypherExpression::FunctionCall(func_call) => {
                // 检查是否是聚合函数
                matches!(
                    func_call.function_name.to_lowercase().as_str(),
                    "count" | "sum" | "avg" | "min" | "max" | "collect" | "distinct"
                )
            }
            CypherExpression::Binary(bin_expr) => {
                Self::contains_cypher_aggregate(&bin_expr.left)
                    || Self::contains_cypher_aggregate(&bin_expr.right)
            }
            CypherExpression::Unary(unary_expr) => {
                Self::contains_cypher_aggregate(&unary_expr.expression)
            }
            CypherExpression::Property(prop_expr) => {
                Self::contains_cypher_aggregate(&prop_expr.expression)
            }
            CypherExpression::List(list_expr) => list_expr
                .elements
                .iter()
                .any(|e| Self::contains_cypher_aggregate(e)),
            CypherExpression::Map(map_expr) => map_expr
                .properties
                .values()
                .any(|e| Self::contains_cypher_aggregate(e)),
            CypherExpression::Case(case_expr) => {
                let alternatives_contains = case_expr.alternatives.iter().any(|alt| {
                    Self::contains_cypher_aggregate(&alt.when_expression)
                        || Self::contains_cypher_aggregate(&alt.then_expression)
                });
                let default_contains = case_expr
                    .default_alternative
                    .as_ref()
                    .map_or(false, |e| Self::contains_cypher_aggregate(e));

                alternatives_contains || default_contains
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_evaluate_literal() {
        let context = ExpressionContext::default();
        let cypher_expr = CypherExpression::Literal(CypherLiteral::Integer(42));
        let result = CypherEvaluator::evaluate_cypher(&cypher_expr, &context).unwrap();

        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_evaluate_variable() {
        let mut context = ExpressionContext::default();
        context.set_variable("x".to_string(), Value::Int(100));

        let cypher_expr = CypherExpression::Variable("x".to_string());
        let result = CypherEvaluator::evaluate_cypher(&cypher_expr, &context).unwrap();

        assert_eq!(result, Value::Int(100));
    }

    #[test]
    fn test_evaluate_binary_add() {
        let context = ExpressionContext::default();
        let left = Box::new(CypherExpression::Literal(CypherLiteral::Integer(1)));
        let right = Box::new(CypherExpression::Literal(CypherLiteral::Integer(2)));
        let cypher_expr = CypherExpression::Binary(BinaryExpression {
            left,
            operator: BinaryOperator::Add,
            right,
        });

        let result = CypherEvaluator::evaluate_cypher(&cypher_expr, &context).unwrap();

        assert_eq!(result, Value::Int(3));
    }

    #[test]
    fn test_is_cypher_constant() {
        let constant_expr = CypherExpression::Literal(CypherLiteral::Integer(42));
        assert!(CypherEvaluator::is_cypher_constant(&constant_expr));

        let variable_expr = CypherExpression::Variable("x".to_string());
        assert!(!CypherEvaluator::is_cypher_constant(&variable_expr));
    }

    #[test]
    fn test_get_cypher_variables() {
        let left = Box::new(CypherExpression::Variable("x".to_string()));
        let right = Box::new(CypherExpression::Variable("y".to_string()));
        let binary_expr = CypherExpression::Binary(BinaryExpression {
            left,
            operator: BinaryOperator::Add,
            right,
        });

        let variables = CypherEvaluator::get_cypher_variables(&binary_expr);
        assert_eq!(variables.len(), 2);
        assert!(variables.contains(&"x".to_string()));
        assert!(variables.contains(&"y".to_string()));
    }

    #[test]
    fn test_contains_cypher_aggregate() {
        let args = vec![CypherExpression::Variable("x".to_string())];
        let func_call = CypherExpression::FunctionCall(FunctionCall {
            function_name: "count".to_string(),
            arguments: args,
        });

        assert!(CypherEvaluator::contains_cypher_aggregate(&func_call));

        let non_aggregate = CypherExpression::Variable("x".to_string());
        assert!(!CypherEvaluator::contains_cypher_aggregate(&non_aggregate));
    }
}
