//! 表达式转换器
//! 将AST表达式转换为graph表达式

use crate::core::types::expression::Expression as GraphExpression;
use crate::core::types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};
use crate::core::Value;
use crate::query::parser::ast::{
    BinaryExpression, BinaryOp, CaseExpression, ConstantExpression, Expression, FunctionCallExpression, LabelExpression, ListExpression,
    MapExpression, PathExpression, PropertyAccessExpression, RangeExpression, SubscriptExpression, TypeCastExpression, UnaryExpression,
    UnaryOp, VariableExpression,
};

/// 将AST表达式转换为graph表达式
pub fn convert_ast_to_graph_expression(ast_expression: &Expression) -> Result<GraphExpression, String> {
    match ast_expression {
        Expression::Constant(expression) => convert_constant_expression(expression),
        Expression::Variable(expression) => convert_variable_expression(expression),
        Expression::Binary(expression) => convert_binary_expression(expression),
        Expression::Unary(expression) => convert_unary_expression(expression),
        Expression::FunctionCall(expression) => convert_function_call_expression(expression),
        Expression::PropertyAccess(expression) => convert_property_access_expression(expression),
        Expression::List(expression) => convert_list_expression(expression),
        Expression::Map(expression) => convert_map_expression(expression),
        Expression::Case(expression) => convert_case_expression(expression),
        Expression::Subscript(expression) => convert_subscript_expression(expression),
        Expression::TypeCast(expression) => convert_type_cast_expression(expression),
        Expression::Range(expression) => convert_range_expression(expression),
        Expression::Path(expression) => convert_path_expression(expression),
        Expression::Label(expression) => convert_label_expression(expression),
    }
}

/// 转换常量表达式
fn convert_constant_expression(expression: &ConstantExpression) -> Result<GraphExpression, String> {
    let value = match &expression.value {
        Value::Bool(b) => Value::Bool(*b),
        Value::Int(i) => Value::Int(*i),
        Value::Float(f) => Value::Float(*f),
        Value::String(s) => Value::String(s.clone()),
        Value::Null(nt) => Value::Null(nt.clone()),
        _ => return Err(format!("不支持的常量值类型: {:?}", expression.value)),
    };
    Ok(GraphExpression::Literal(value))
}

/// 转换类型转换表达式
fn convert_type_cast_expression(expression: &TypeCastExpression) -> Result<GraphExpression, String> {
    let converted_expression = convert_ast_to_graph_expression(&expression.expression)?;
    let target_type = parse_data_type(&expression.target_type)?;
    Ok(GraphExpression::TypeCast {
        expression: Box::new(converted_expression),
        target_type,
    })
}

/// 转换范围表达式
fn convert_range_expression(expression: &RangeExpression) -> Result<GraphExpression, String> {
    let collection = convert_ast_to_graph_expression(&expression.collection)?;
    let start = if let Some(ref start_expression) = expression.start {
        Some(Box::new(convert_ast_to_graph_expression(start_expression)?))
    } else {
        None
    };
    let end = if let Some(ref end_expression) = expression.end {
        Some(Box::new(convert_ast_to_graph_expression(end_expression)?))
    } else {
        None
    };
    Ok(GraphExpression::Range {
        collection: Box::new(collection),
        start,
        end,
    })
}

/// 转换路径表达式
fn convert_path_expression(expression: &PathExpression) -> Result<GraphExpression, String> {
    let elements: Result<Vec<GraphExpression>, String> = expression
        .elements
        .iter()
        .map(|elem| convert_ast_to_graph_expression(elem))
        .collect();
    Ok(GraphExpression::Path(elements?))
}

/// 转换标签表达式
fn convert_label_expression(expression: &LabelExpression) -> Result<GraphExpression, String> {
    Ok(GraphExpression::Label(expression.label.clone()))
}

/// 解析数据类型字符串
fn parse_data_type(type_str: &str) -> Result<crate::core::types::expression::DataType, String> {
    match type_str.to_uppercase().as_str() {
        "BOOL" | "BOOLEAN" => Ok(crate::core::types::expression::DataType::Bool),
        "INT" | "INTEGER" => Ok(crate::core::types::expression::DataType::Int),
        "FLOAT" | "DOUBLE" => Ok(crate::core::types::expression::DataType::Float),
        "STRING" | "STR" => Ok(crate::core::types::expression::DataType::String),
        "LIST" => Ok(crate::core::types::expression::DataType::List),
        "MAP" => Ok(crate::core::types::expression::DataType::Map),
        "VERTEX" => Ok(crate::core::types::expression::DataType::Vertex),
        "EDGE" => Ok(crate::core::types::expression::DataType::Edge),
        "PATH" => Ok(crate::core::types::expression::DataType::Path),
        "DATETIME" => Ok(crate::core::types::expression::DataType::DateTime),
        "DATE" => Ok(crate::core::types::expression::DataType::Date),
        "TIME" => Ok(crate::core::types::expression::DataType::Time),
        "DURATION" => Ok(crate::core::types::expression::DataType::Duration),
        _ => Err(format!("不支持的数据类型: {}", type_str)),
    }
}

/// 转换变量表达式
fn convert_variable_expression(expression: &VariableExpression) -> Result<GraphExpression, String> {
    Ok(GraphExpression::Variable(expression.name.clone()))
}

/// 转换二元表达式
fn convert_binary_expression(expression: &BinaryExpression) -> Result<GraphExpression, String> {
    let left = convert_ast_to_graph_expression(&expression.left)?;
    let right = convert_ast_to_graph_expression(&expression.right)?;
    let op = convert_binary_op(&expression.op)?;

    Ok(GraphExpression::Binary {
        left: Box::new(left),
        op,
        right: Box::new(right),
    })
}

/// 转换一元表达式
fn convert_unary_expression(expression: &UnaryExpression) -> Result<GraphExpression, String> {
    let operand = convert_ast_to_graph_expression(&expression.operand)?;
    let op = convert_unary_op(&expression.op)?;

    Ok(GraphExpression::Unary {
        op,
        operand: Box::new(operand),
    })
}

/// 转换函数调用表达式
fn convert_function_call_expression(expression: &FunctionCallExpression) -> Result<GraphExpression, String> {
    let args: Result<Vec<GraphExpression>, String> = expression
        .args
        .iter()
        .map(|arg| convert_ast_to_graph_expression(arg))
        .collect();

    let args = args?;

    // 检查是否为聚合函数
    let func_name = expression.name.to_uppercase();
    if is_aggregate_function(&func_name) {
        if args.len() != 1 {
            return Err(format!(
                "聚合函数 {} 需要一个参数，但提供了 {}",
                expression.name,
                args.len()
            ));
        }
        let arg = Box::new(args[0].clone());
        let aggregate_func = convert_aggregate_function(&func_name)?;

        Ok(GraphExpression::Aggregate {
            func: aggregate_func,
            arg,
            distinct: expression.distinct,
        })
    } else {
        // 普通函数调用
        Ok(GraphExpression::Function {
            name: expression.name.clone(),
            args,
        })
    }
}

/// 转换属性访问表达式
fn convert_property_access_expression(expression: &PropertyAccessExpression) -> Result<GraphExpression, String> {
    let object = convert_ast_to_graph_expression(&expression.object)?;
    Ok(GraphExpression::Property {
        object: Box::new(object),
        property: expression.property.clone(),
    })
}

/// 转换列表表达式
fn convert_list_expression(expression: &ListExpression) -> Result<GraphExpression, String> {
    let elements: Result<Vec<GraphExpression>, String> = expression
        .elements
        .iter()
        .map(|elem| convert_ast_to_graph_expression(elem))
        .collect();

    Ok(GraphExpression::List(elements?))
}

/// 转换映射表达式
fn convert_map_expression(expression: &MapExpression) -> Result<GraphExpression, String> {
    let pairs: Result<Vec<(String, GraphExpression)>, String> = expression
        .pairs
        .iter()
        .map(|(key, value)| {
            let converted_value = convert_ast_to_graph_expression(value)?;
            Ok((key.clone(), converted_value))
        })
        .collect();

    Ok(GraphExpression::Map(pairs?))
}

/// 转换CASE表达式
fn convert_case_expression(expression: &CaseExpression) -> Result<GraphExpression, String> {
    let mut conditions = Vec::new();

    // 处理WHEN-THEN条件对
    for (when, then) in &expression.when_then_pairs {
        let when_expression = convert_ast_to_graph_expression(when)?;
        let then_expression = convert_ast_to_graph_expression(then)?;
        conditions.push((when_expression, then_expression));
    }

    let default = if let Some(ref default_expression) = expression.default {
        Some(Box::new(convert_ast_to_graph_expression(default_expression)?))
    } else {
        None
    };

    // 如果存在match表达式，需要特殊处理
    if let Some(ref match_expression) = expression.match_expression {
        // 对于有match表达式的CASE，需要将每个WHEN条件转换为与match表达式的比较
        let match_expression = convert_ast_to_graph_expression(match_expression)?;
        let mut new_conditions = Vec::new();

        for (when, then) in conditions {
            let condition = GraphExpression::Binary {
                left: Box::new(match_expression.clone()),
                op: BinaryOperator::Equal,
                right: Box::new(when),
            };
            new_conditions.push((condition, then));
        }

        Ok(GraphExpression::Case {
            conditions: new_conditions,
            default,
        })
    } else {
        Ok(GraphExpression::Case {
            conditions,
            default,
        })
    }
}

/// 转换下标表达式
fn convert_subscript_expression(expression: &SubscriptExpression) -> Result<GraphExpression, String> {
    let collection = convert_ast_to_graph_expression(&expression.collection)?;
    let index = convert_ast_to_graph_expression(&expression.index)?;

    Ok(GraphExpression::Subscript {
        collection: Box::new(collection),
        index: Box::new(index),
    })
}

/// 转换二元操作符
fn convert_binary_op(op: &BinaryOp) -> Result<BinaryOperator, String> {
    match op {
        // 算术操作符
        BinaryOp::Add => Ok(BinaryOperator::Add),
        BinaryOp::Subtract => Ok(BinaryOperator::Subtract),
        BinaryOp::Multiply => Ok(BinaryOperator::Multiply),
        BinaryOp::Divide => Ok(BinaryOperator::Divide),
        BinaryOp::Modulo => Ok(BinaryOperator::Modulo),
        BinaryOp::Exponent => Ok(BinaryOperator::Exponent),

        // 逻辑操作符
        BinaryOp::And => Ok(BinaryOperator::And),
        BinaryOp::Or => Ok(BinaryOperator::Or),
        BinaryOp::Xor => Err("XOR操作符在graph表达式中不支持".to_string()),

        // 关系操作符
        BinaryOp::Equal => Ok(BinaryOperator::Equal),
        BinaryOp::NotEqual => Ok(BinaryOperator::NotEqual),
        BinaryOp::LessThan => Ok(BinaryOperator::LessThan),
        BinaryOp::LessThanOrEqual => Ok(BinaryOperator::LessThanOrEqual),
        BinaryOp::GreaterThan => Ok(BinaryOperator::GreaterThan),
        BinaryOp::GreaterThanOrEqual => Ok(BinaryOperator::GreaterThanOrEqual),

        // 字符串操作符
        BinaryOp::Like => Ok(BinaryOperator::Like), // Like
        BinaryOp::In => Ok(BinaryOperator::In),
        BinaryOp::NotIn => Ok(BinaryOperator::NotIn),
        BinaryOp::Contains => Ok(BinaryOperator::Contains),
        BinaryOp::StartsWith => Ok(BinaryOperator::StartsWith),
        BinaryOp::EndsWith => Ok(BinaryOperator::EndsWith),

        // 其他操作符
        BinaryOp::StringConcat => Ok(BinaryOperator::StringConcat),
        BinaryOp::Subscript => Ok(BinaryOperator::Subscript),
        BinaryOp::Attribute => Ok(BinaryOperator::Attribute),
        BinaryOp::Union => Ok(BinaryOperator::Union),
        BinaryOp::Intersect => Ok(BinaryOperator::Intersect),
        BinaryOp::Except => Ok(BinaryOperator::Except),
    }
}

/// 转换一元操作符
fn convert_unary_op(op: &UnaryOp) -> Result<UnaryOperator, String> {
    match op {
        UnaryOp::Not => Ok(UnaryOperator::Not),
        UnaryOp::Plus => Ok(UnaryOperator::Plus),
        UnaryOp::Minus => Ok(UnaryOperator::Minus),
        UnaryOp::IsNull => Ok(UnaryOperator::IsNull),
        UnaryOp::IsNotNull => Ok(UnaryOperator::IsNotNull),
        UnaryOp::IsEmpty => Ok(UnaryOperator::IsEmpty),
        UnaryOp::IsNotEmpty => Ok(UnaryOperator::IsNotEmpty),
    }
}

/// 转换聚合函数
fn convert_aggregate_function(func_name: &str) -> Result<AggregateFunction, String> {
    match func_name {
        "COUNT" => Ok(AggregateFunction::Count(None)),
        "SUM" => Ok(AggregateFunction::Sum("".to_string())),
        "AVG" => Ok(AggregateFunction::Avg("".to_string())),
        "MIN" => Ok(AggregateFunction::Min("".to_string())),
        "MAX" => Ok(AggregateFunction::Max("".to_string())),
        "COLLECT" => Ok(AggregateFunction::Collect("".to_string())),
        "DISTINCT" => Ok(AggregateFunction::Distinct("".to_string())),
        "PERCENTILE" => Ok(AggregateFunction::Percentile("".to_string(), 50.0)), // 默认50%
        _ => Err(format!("不支持的聚合函数: {}", func_name)),
    }
}

/// 检查是否为聚合函数
fn is_aggregate_function(func_name: &str) -> bool {
    matches!(
        func_name,
        "COUNT" | "SUM" | "AVG" | "MIN" | "MAX" | "COLLECT" | "DISTINCT" | "PERCENTILE"
    )
}

/// 从字符串解析表达式
pub fn parse_expression_from_string(condition: &str) -> Result<GraphExpression, String> {
    // 创建语法分析器
    let mut parser = crate::query::parser::Parser::new(condition);
    let ast_expression = parser
        .parse_expression()
        .map_err(|e| format!("语法分析错误: {:?}", e))?;

    // 转换为graph表达式
    convert_ast_to_graph_expression(&ast_expression)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;
    use crate::query::parser::ast::{
        BinaryExpression, BinaryOp, ConstantExpression, Expression, LabelExpression, ListExpression, MapExpression, PathExpression,
        PropertyAccessExpression, RangeExpression, Span, SubscriptExpression, TypeCastExpression, UnaryExpression, UnaryOp,
        VariableExpression,
    };

    #[test]
    fn test_convert_constant_expression() {
        let ast_expression = Expression::Constant(ConstantExpression::new(Value::Int(42), Span::default()));
        let result = convert_ast_to_graph_expression(&ast_expression)
            .expect("Expected successful conversion of constant expression");

        if let GraphExpression::Literal(Value::Int(value)) = result {
            assert_eq!(value, 42);
        } else {
            panic!("Expected Literal(Int(42)), got {:?}", result);
        }
    }

    #[test]
    fn test_convert_variable_expression() {
        let ast_expression = Expression::Variable(VariableExpression::new("test_var".to_string(), Span::default()));
        let result = convert_ast_to_graph_expression(&ast_expression)
            .expect("Expected successful conversion of variable expression");

        if let GraphExpression::Variable(name) = result {
            assert_eq!(name, "test_var");
        } else {
            panic!("Expected Variable(\"test_var\"), got {:?}", result);
        }
    }

    #[test]
    fn test_convert_type_cast_expression() {
        let inner_expression = Expression::Constant(ConstantExpression::new(Value::Int(42), Span::default()));
        let ast_expression = Expression::TypeCast(TypeCastExpression::new(
            inner_expression,
            "FLOAT".to_string(),
            Span::default(),
        ));
        let result = convert_ast_to_graph_expression(&ast_expression)
            .expect("Expected successful conversion of type cast expression");

        if let GraphExpression::TypeCast { expression, target_type } = result {
            assert_eq!(*expression, GraphExpression::Literal(Value::Int(42)));
            assert_eq!(target_type, crate::core::types::expression::DataType::Float);
        } else {
            panic!("Expected TypeCast, got {:?}", result);
        }
    }

    #[test]
    fn test_convert_label_expression() {
        let ast_expression = Expression::Label(LabelExpression::new("Person".to_string(), Span::default()));
        let result = convert_ast_to_graph_expression(&ast_expression)
            .expect("Expected successful conversion of label expression");

        if let GraphExpression::Label(label) = result {
            assert_eq!(label, "Person");
        } else {
            panic!("Expected Label, got {:?}", result);
        }
    }

    #[test]
    fn test_convert_binary_expression() {
        let left = Expression::Constant(ConstantExpression::new(Value::Int(5), Span::default()));
        let right = Expression::Constant(ConstantExpression::new(Value::Int(3), Span::default()));
        let ast_expression = Expression::Binary(BinaryExpression::new(left, BinaryOp::Add, right, Span::default()));

        let result = convert_ast_to_graph_expression(&ast_expression)
            .expect("Expected successful conversion of binary expression");

        if let GraphExpression::Binary { left, op, right } = result {
            assert_eq!(*left, GraphExpression::Literal(Value::Int(5)));
            assert_eq!(op, BinaryOperator::Add);
            assert_eq!(*right, GraphExpression::Literal(Value::Int(3)));
        } else {
            panic!("Expected Binary expression, got {:?}", result);
        }
    }

    #[test]
    fn test_convert_unary_expression() {
        let operand = Expression::Constant(ConstantExpression::new(Value::Bool(true), Span::default()));
        let ast_expression = Expression::Unary(UnaryExpression::new(UnaryOp::Not, operand, Span::default()));

        let result = convert_ast_to_graph_expression(&ast_expression)
            .expect("Expected successful conversion of unary expression");

        if let GraphExpression::Unary { op, operand } = result {
            assert_eq!(op, UnaryOperator::Not);
            assert_eq!(*operand, GraphExpression::Literal(Value::Bool(true)));
        } else {
            panic!("Expected Unary expression, got {:?}", result);
        }
    }

    #[test]
    fn test_convert_unsupported_operator() {
        let left = Expression::Constant(ConstantExpression::new(Value::Int(5), Span::default()));
        let right = Expression::Constant(ConstantExpression::new(Value::Int(3), Span::default()));
        let ast_expression = Expression::Binary(BinaryExpression::new(left, BinaryOp::Xor, right, Span::default()));

        let result = convert_ast_to_graph_expression(&ast_expression);
        assert!(result.is_err());
        assert!(result
            .expect_err("Expected error for unsupported operator")
            .contains("XOR操作符在graph表达式中不支持"));
    }

    #[test]
    fn test_parse_expression_from_string() {
        let result = parse_expression_from_string("5 + 3");
        assert!(result.is_ok());

        let expression = result.expect("Expected successful parsing of expression from string");
        assert!(matches!(expression, GraphExpression::Binary { .. }));
    }
}
