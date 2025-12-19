use super::error::ExpressionError;
use super::operator_conversion;
use crate::core::Value;
use crate::graph::expression::{Expression, LiteralValue};
use crate::query::context::EvalContext;
use crate::query::parser::cypher::ast::expressions::{
    BinaryExpression, BinaryOperator, CaseExpression,
    Expression as CypherExpression, FunctionCall, ListExpression, Literal as CypherLiteral,
    MapExpression, PatternExpression, PropertyExpression, UnaryExpression, UnaryOperator,
};

/// Cypher兼容性模块
/// 提供Cypher表达式与统一表达式之间的转换和评估功能

/// 直接评估Cypher表达式
pub fn evaluate_cypher(
    cypher_expr: &CypherExpression,
    context: &EvalContext,
) -> Result<Value, ExpressionError> {
    match cypher_expr {
        CypherExpression::Literal(literal) => evaluate_cypher_literal(literal),
        CypherExpression::Variable(name) => evaluate_cypher_variable(name, context),
        CypherExpression::Property(prop_expr) => evaluate_cypher_property(prop_expr, context),
        CypherExpression::FunctionCall(func_call) => {
            evaluate_cypher_function_call(func_call, context)
        }
        CypherExpression::Binary(bin_expr) => evaluate_cypher_binary(bin_expr, context),
        CypherExpression::Unary(unary_expr) => evaluate_cypher_unary(unary_expr, context),
        CypherExpression::Case(case_expr) => evaluate_cypher_case(case_expr, context),
        CypherExpression::List(list_expr) => evaluate_cypher_list(list_expr, context),
        CypherExpression::Map(map_expr) => evaluate_cypher_map(map_expr, context),
        CypherExpression::PatternExpression(pattern_expr) => {
            evaluate_cypher_pattern(pattern_expr, context)
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
    context: &EvalContext,
) -> Result<Value, ExpressionError> {
    context
        .vars
        .get(name)
        .cloned()
        .ok_or_else(|| ExpressionError::PropertyNotFound(format!("Variable ${}", name)))
}

/// 评估Cypher属性表达式
fn evaluate_cypher_property(
    prop_expr: &PropertyExpression,
    context: &EvalContext,
) -> Result<Value, ExpressionError> {
    let object_value = evaluate_cypher(&prop_expr.expression, context)?;

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
    context: &EvalContext,
) -> Result<Value, ExpressionError> {
    // 将Cypher函数调用转换为统一的函数调用
    let args: Result<Vec<Expression>, ExpressionError> = func_call
        .arguments
        .iter()
        .map(|arg| convert_cypher_to_unified(arg))
        .collect();

    let unified_func = Expression::Function {
        name: func_call.function_name.clone(),
        args: args?,
    };

    // 使用ExpressionEvaluator评估统一函数
    super::evaluator::ExpressionEvaluator::new().evaluate(&unified_func, context)
}

/// 评估Cypher二元表达式
fn evaluate_cypher_binary(
    bin_expr: &BinaryExpression,
    context: &EvalContext,
) -> Result<Value, ExpressionError> {
    let left_value = evaluate_cypher(&bin_expr.left, context)?;
    let right_value = evaluate_cypher(&bin_expr.right, context)?;

    evaluate_binary_operation(&left_value, &bin_expr.operator, &right_value)
}

/// 评估Cypher一元表达式
fn evaluate_cypher_unary(
    unary_expr: &UnaryExpression,
    context: &EvalContext,
) -> Result<Value, ExpressionError> {
    let value = evaluate_cypher(&unary_expr.expression, context)?;

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
    context: &EvalContext,
) -> Result<Value, ExpressionError> {
    for alternative in &case_expr.alternatives {
        let cond_result = evaluate_cypher(&alternative.when_expression, context)?;
        if super::comparison::value_to_bool(&cond_result) {
            return evaluate_cypher(&alternative.then_expression, context);
        }
    }

    if let Some(default_expr) = &case_expr.default_alternative {
        evaluate_cypher(default_expr, context)
    } else {
        Ok(Value::Null(crate::core::NullType::Null))
    }
}

/// 评估Cypher列表表达式
fn evaluate_cypher_list(
    list_expr: &ListExpression,
    context: &EvalContext,
) -> Result<Value, ExpressionError> {
    let mut elements = Vec::new();
    for element in &list_expr.elements {
        elements.push(evaluate_cypher(element, context)?);
    }
    Ok(Value::List(elements))
}

/// 评估Cypher映射表达式
fn evaluate_cypher_map(
    map_expr: &MapExpression,
    context: &EvalContext,
) -> Result<Value, ExpressionError> {
    let mut properties = std::collections::HashMap::new();
    for (key, value) in &map_expr.properties {
        properties.insert(key.clone(), evaluate_cypher(value, context)?);
    }
    Ok(Value::Map(properties))
}

/// 评估Cypher模式表达式
fn evaluate_cypher_pattern(
    _pattern_expr: &PatternExpression,
    _context: &EvalContext,
) -> Result<Value, ExpressionError> {
    // 模式表达式的简化实现
    Ok(Value::String("PatternExpression".to_string()))
}

/// 将Cypher表达式转换为统一表达式
pub fn convert_cypher_to_unified(
    cypher_expr: &CypherExpression,
) -> Result<Expression, ExpressionError> {
    match cypher_expr {
        CypherExpression::Literal(literal) => {
            let unified_literal = match literal {
                CypherLiteral::String(s) => LiteralValue::String(s.clone()),
                CypherLiteral::Integer(i) => LiteralValue::Int(*i),
                CypherLiteral::Float(f) => LiteralValue::Float(*f),
                CypherLiteral::Boolean(b) => LiteralValue::Bool(*b),
                CypherLiteral::Null => LiteralValue::Null,
            };
            Ok(Expression::Literal(unified_literal))
        }
        CypherExpression::Variable(name) => Ok(Expression::Variable(name.clone())),
        CypherExpression::Property(prop_expr) => {
            let object_expr = convert_cypher_to_unified(&prop_expr.expression)?;
            Ok(Expression::Property {
                object: Box::new(object_expr),
                property: prop_expr.property_name.clone(),
            })
        }
        CypherExpression::FunctionCall(func_call) => {
            let args: Result<Vec<Expression>, ExpressionError> = func_call
                .arguments
                .iter()
                .map(|arg| convert_cypher_to_unified(arg))
                .collect();
            Ok(Expression::Function {
                name: func_call.function_name.clone(),
                args: args?,
            })
        }
        CypherExpression::Binary(bin_expr) => {
            let left = convert_cypher_to_unified(&bin_expr.left)?;
            let right = convert_cypher_to_unified(&bin_expr.right)?;
            let op = operator_conversion::convert_cypher_binary_operator(&bin_expr.operator);
            Ok(Expression::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            })
        }
        CypherExpression::Unary(unary_expr) => {
            let operand = convert_cypher_to_unified(&unary_expr.expression)?;
            let op = operator_conversion::convert_cypher_unary_operator(&unary_expr.operator);
            Ok(Expression::Unary {
                op,
                operand: Box::new(operand),
            })
        }
        CypherExpression::List(list_expr) => {
            let elements: Result<Vec<Expression>, ExpressionError> = list_expr
                .elements
                .iter()
                .map(|elem| convert_cypher_to_unified(elem))
                .collect();
            Ok(Expression::List(elements?))
        }
        CypherExpression::Map(map_expr) => {
            let pairs: Result<Vec<(String, Expression)>, ExpressionError> = map_expr
                .properties
                .iter()
                .map(|(key, value)| {
                    let value_expr = convert_cypher_to_unified(value)?;
                    Ok((key.clone(), value_expr))
                })
                .collect();
            Ok(Expression::Map(pairs?))
        }
        _ => Ok(Expression::string("UnsupportedExpression".to_string())),
    }
}

/// 评估二元操作
fn evaluate_binary_operation(
    left: &Value,
    op: &BinaryOperator,
    right: &Value,
) -> Result<Value, ExpressionError> {
    match op {
        BinaryOperator::Equal => Ok(Value::Bool(super::comparison::values_equal(left, right))),
        BinaryOperator::NotEqual => Ok(Value::Bool(!super::comparison::values_equal(left, right))),
        BinaryOperator::GreaterThan => Ok(Value::Bool(super::comparison::compare_values(left, right) > 0)),
        BinaryOperator::LessThan => Ok(Value::Bool(super::comparison::compare_values(left, right) < 0)),
        BinaryOperator::GreaterThanOrEqual => {
            Ok(Value::Bool(super::comparison::compare_values(left, right) >= 0))
        }
        BinaryOperator::LessThanOrEqual => {
            Ok(Value::Bool(super::comparison::compare_values(left, right) <= 0))
        }
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
        BinaryOperator::Add => super::arithmetic::arithmetic_add(left, right),
        BinaryOperator::Subtract => super::arithmetic::arithmetic_subtract(left, right),
        BinaryOperator::Multiply => super::arithmetic::arithmetic_multiply(left, right),
        BinaryOperator::Divide => super::arithmetic::arithmetic_divide(left, right),
        BinaryOperator::Modulo => super::arithmetic::arithmetic_modulo(left, right),
        BinaryOperator::Exponent => super::arithmetic::arithmetic_exponent(left, right),
        BinaryOperator::In => super::comparison::check_in(left, right),
        BinaryOperator::StartsWith => super::comparison::check_starts_with(left, right),
        BinaryOperator::EndsWith => super::comparison::check_ends_with(left, right),
        BinaryOperator::Contains => super::comparison::check_contains(left, right),
        BinaryOperator::RegexMatch => Err(ExpressionError::InvalidOperation(
            "RegexMatch not implemented".to_string(),
        )),
    }
}