//! 表达式转换器
//! 将AST表达式转换为graph表达式

use crate::core::types::expression::Expression;
use crate::core::types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};
use crate::core::Value;
use crate::query::parser::ast::{
    BinaryExpr, BinaryOp, CaseExpr, ConstantExpr, Expr, FunctionCallExpr, LabelExpr, ListExpr,
    MapExpr, PathExpr, PropertyAccessExpr, RangeExpr, SubscriptExpr, TypeCastExpr, UnaryExpr,
    UnaryOp, VariableExpr,
};

/// 将AST表达式转换为graph表达式
pub fn convert_ast_to_graph_expression(ast_expr: &Expr) -> Result<Expression, String> {
    match ast_expr {
        Expr::Constant(expr) => convert_constant_expr(expr),
        Expr::Variable(expr) => convert_variable_expr(expr),
        Expr::Binary(expr) => convert_binary_expr(expr),
        Expr::Unary(expr) => convert_unary_expr(expr),
        Expr::FunctionCall(expr) => convert_function_call_expr(expr),
        Expr::PropertyAccess(expr) => convert_property_access_expr(expr),
        Expr::List(expr) => convert_list_expr(expr),
        Expr::Map(expr) => convert_map_expr(expr),
        Expr::Case(expr) => convert_case_expr(expr),
        Expr::Subscript(expr) => convert_subscript_expr(expr),
        Expr::TypeCast(expr) => convert_type_cast_expr(expr),
        Expr::Range(expr) => convert_range_expr(expr),
        Expr::Path(expr) => convert_path_expr(expr),
        Expr::Label(expr) => convert_label_expr(expr),
    }
}

/// 转换常量表达式
fn convert_constant_expr(expr: &ConstantExpr) -> Result<Expression, String> {
    let value = match &expr.value {
        Value::Bool(b) => Value::Bool(*b),
        Value::Int(i) => Value::Int(*i),
        Value::Float(f) => Value::Float(*f),
        Value::String(s) => Value::String(s.clone()),
        Value::Null(nt) => Value::Null(nt.clone()),
        _ => return Err(format!("不支持的常量值类型: {:?}", expr.value)),
    };
    Ok(Expression::Literal(value))
}

/// 转换类型转换表达式
fn convert_type_cast_expr(expr: &TypeCastExpr) -> Result<Expression, String> {
    let converted_expr = convert_ast_to_graph_expression(&expr.expr)?;
    let target_type = parse_data_type(&expr.target_type)?;
    Ok(Expression::TypeCast {
        expr: Box::new(converted_expr),
        target_type,
    })
}

/// 转换范围表达式
fn convert_range_expr(expr: &RangeExpr) -> Result<Expression, String> {
    let collection = convert_ast_to_graph_expression(&expr.collection)?;
    let start = if let Some(ref start_expr) = expr.start {
        Some(Box::new(convert_ast_to_graph_expression(start_expr)?))
    } else {
        None
    };
    let end = if let Some(ref end_expr) = expr.end {
        Some(Box::new(convert_ast_to_graph_expression(end_expr)?))
    } else {
        None
    };
    Ok(Expression::Range {
        collection: Box::new(collection),
        start,
        end,
    })
}

/// 转换路径表达式
fn convert_path_expr(expr: &PathExpr) -> Result<Expression, String> {
    let elements: Result<Vec<Expression>, String> = expr
        .elements
        .iter()
        .map(|elem| convert_ast_to_graph_expression(elem))
        .collect();
    Ok(Expression::Path(elements?))
}

/// 转换标签表达式
fn convert_label_expr(expr: &LabelExpr) -> Result<Expression, String> {
    Ok(Expression::Label(expr.label.clone()))
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
fn convert_variable_expr(expr: &VariableExpr) -> Result<Expression, String> {
    Ok(Expression::Variable(expr.name.clone()))
}

/// 转换二元表达式
fn convert_binary_expr(expr: &BinaryExpr) -> Result<Expression, String> {
    let left = convert_ast_to_graph_expression(&expr.left)?;
    let right = convert_ast_to_graph_expression(&expr.right)?;
    let op = convert_binary_op(&expr.op)?;

    Ok(Expression::Binary {
        left: Box::new(left),
        op,
        right: Box::new(right),
    })
}

/// 转换一元表达式
fn convert_unary_expr(expr: &UnaryExpr) -> Result<Expression, String> {
    let operand = convert_ast_to_graph_expression(&expr.operand)?;
    let op = convert_unary_op(&expr.op)?;

    Ok(Expression::Unary {
        op,
        operand: Box::new(operand),
    })
}

/// 转换函数调用表达式
fn convert_function_call_expr(expr: &FunctionCallExpr) -> Result<Expression, String> {
    let args: Result<Vec<Expression>, String> = expr
        .args
        .iter()
        .map(|arg| convert_ast_to_graph_expression(arg))
        .collect();

    let args = args?;

    // 检查是否为聚合函数
    let func_name = expr.name.to_uppercase();
    if is_aggregate_function(&func_name) {
        if args.len() != 1 {
            return Err(format!(
                "聚合函数 {} 需要一个参数，但提供了 {}",
                expr.name,
                args.len()
            ));
        }
        let arg = Box::new(args[0].clone());
        let aggregate_func = convert_aggregate_function(&func_name)?;

        Ok(Expression::Aggregate {
            func: aggregate_func,
            arg,
            distinct: expr.distinct,
        })
    } else {
        // 普通函数调用
        Ok(Expression::Function {
            name: expr.name.clone(),
            args,
        })
    }
}

/// 转换属性访问表达式
fn convert_property_access_expr(expr: &PropertyAccessExpr) -> Result<Expression, String> {
    let object = convert_ast_to_graph_expression(&expr.object)?;
    Ok(Expression::Property {
        object: Box::new(object),
        property: expr.property.clone(),
    })
}

/// 转换列表表达式
fn convert_list_expr(expr: &ListExpr) -> Result<Expression, String> {
    let elements: Result<Vec<Expression>, String> = expr
        .elements
        .iter()
        .map(|elem| convert_ast_to_graph_expression(elem))
        .collect();

    Ok(Expression::List(elements?))
}

/// 转换映射表达式
fn convert_map_expr(expr: &MapExpr) -> Result<Expression, String> {
    let pairs: Result<Vec<(String, Expression)>, String> = expr
        .pairs
        .iter()
        .map(|(key, value)| {
            let converted_value = convert_ast_to_graph_expression(value)?;
            Ok((key.clone(), converted_value))
        })
        .collect();

    Ok(Expression::Map(pairs?))
}

/// 转换CASE表达式
fn convert_case_expr(expr: &CaseExpr) -> Result<Expression, String> {
    let mut conditions = Vec::new();

    // 处理WHEN-THEN条件对
    for (when, then) in &expr.when_then_pairs {
        let when_expr = convert_ast_to_graph_expression(when)?;
        let then_expr = convert_ast_to_graph_expression(then)?;
        conditions.push((when_expr, then_expr));
    }

    let default = if let Some(ref default_expr) = expr.default {
        Some(Box::new(convert_ast_to_graph_expression(default_expr)?))
    } else {
        None
    };

    // 如果存在match表达式，需要特殊处理
    if let Some(ref match_expr) = expr.match_expr {
        // 对于有match表达式的CASE，需要将每个WHEN条件转换为与match表达式的比较
        let match_expr = convert_ast_to_graph_expression(match_expr)?;
        let mut new_conditions = Vec::new();

        for (when, then) in conditions {
            let condition = Expression::Binary {
                left: Box::new(match_expr.clone()),
                op: BinaryOperator::Equal,
                right: Box::new(when),
            };
            new_conditions.push((condition, then));
        }

        Ok(Expression::Case {
            conditions: new_conditions,
            default,
        })
    } else {
        Ok(Expression::Case {
            conditions,
            default,
        })
    }
}

/// 转换下标表达式
fn convert_subscript_expr(expr: &SubscriptExpr) -> Result<Expression, String> {
    let collection = convert_ast_to_graph_expression(&expr.collection)?;
    let index = convert_ast_to_graph_expression(&expr.index)?;

    Ok(Expression::Subscript {
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
pub fn parse_expression_from_string(condition: &str) -> Result<Expression, String> {
    // 创建语法分析器
    let mut parser = crate::query::parser::Parser::new(condition);
    let ast_expr = parser
        .parse_expression()
        .map_err(|e| format!("语法分析错误: {:?}", e))?;

    // 转换为graph表达式
    convert_ast_to_graph_expression(&ast_expr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;
    use crate::query::parser::ast::{
        BinaryExpr, BinaryOp, ConstantExpr, Expr, LabelExpr, ListExpr, MapExpr, PathExpr,
        PropertyAccessExpr, RangeExpr, Span, SubscriptExpr, TypeCastExpr, UnaryExpr, UnaryOp,
        VariableExpr,
    };

    #[test]
    fn test_convert_constant_expr() {
        let ast_expr = Expr::Constant(ConstantExpr::new(Value::Int(42), Span::default()));
        let result = convert_ast_to_graph_expression(&ast_expr)
            .expect("Expected successful conversion of constant expression");

        if let Expression::Literal(Value::Int(value)) = result {
            assert_eq!(value, 42);
        } else {
            panic!("Expected Literal(Int(42)), got {:?}", result);
        }
    }

    #[test]
    fn test_convert_variable_expr() {
        let ast_expr = Expr::Variable(VariableExpr::new("test_var".to_string(), Span::default()));
        let result = convert_ast_to_graph_expression(&ast_expr)
            .expect("Expected successful conversion of variable expression");

        if let Expression::Variable(name) = result {
            assert_eq!(name, "test_var");
        } else {
            panic!("Expected Variable(\"test_var\"), got {:?}", result);
        }
    }

    #[test]
    fn test_convert_type_cast_expr() {
        let inner_expr = Expr::Constant(ConstantExpr::new(Value::Int(42), Span::default()));
        let ast_expr = Expr::TypeCast(TypeCastExpr::new(
            inner_expr,
            "FLOAT".to_string(),
            Span::default(),
        ));
        let result = convert_ast_to_graph_expression(&ast_expr)
            .expect("Expected successful conversion of type cast expression");

        if let Expression::TypeCast { expr, target_type } = result {
            assert_eq!(*expr, Expression::Literal(Value::Int(42)));
            assert_eq!(target_type, crate::core::types::expression::DataType::Float);
        } else {
            panic!("Expected TypeCast, got {:?}", result);
        }
    }

    #[test]
    fn test_convert_label_expr() {
        let ast_expr = Expr::Label(LabelExpr::new("Person".to_string(), Span::default()));
        let result = convert_ast_to_graph_expression(&ast_expr)
            .expect("Expected successful conversion of label expression");

        if let Expression::Label(label) = result {
            assert_eq!(label, "Person");
        } else {
            panic!("Expected Label, got {:?}", result);
        }
    }

    #[test]
    fn test_convert_binary_expr() {
        let left = Expr::Constant(ConstantExpr::new(Value::Int(5), Span::default()));
        let right = Expr::Constant(ConstantExpr::new(Value::Int(3), Span::default()));
        let ast_expr = Expr::Binary(BinaryExpr::new(left, BinaryOp::Add, right, Span::default()));

        let result = convert_ast_to_graph_expression(&ast_expr)
            .expect("Expected successful conversion of binary expression");

        if let Expression::Binary { left, op, right } = result {
            assert_eq!(*left, Expression::Literal(Value::Int(5)));
            assert_eq!(op, BinaryOperator::Add);
            assert_eq!(*right, Expression::Literal(Value::Int(3)));
        } else {
            panic!("Expected Binary expression, got {:?}", result);
        }
    }

    #[test]
    fn test_convert_unary_expr() {
        let operand = Expr::Constant(ConstantExpr::new(Value::Bool(true), Span::default()));
        let ast_expr = Expr::Unary(UnaryExpr::new(UnaryOp::Not, operand, Span::default()));

        let result = convert_ast_to_graph_expression(&ast_expr)
            .expect("Expected successful conversion of unary expression");

        if let Expression::Unary { op, operand } = result {
            assert_eq!(op, UnaryOperator::Not);
            assert_eq!(*operand, Expression::Literal(Value::Bool(true)));
        } else {
            panic!("Expected Unary expression, got {:?}", result);
        }
    }

    #[test]
    fn test_convert_unsupported_operator() {
        let left = Expr::Constant(ConstantExpr::new(Value::Int(5), Span::default()));
        let right = Expr::Constant(ConstantExpr::new(Value::Int(3), Span::default()));
        let ast_expr = Expr::Binary(BinaryExpr::new(left, BinaryOp::Xor, right, Span::default()));

        let result = convert_ast_to_graph_expression(&ast_expr);
        assert!(result.is_err());
        assert!(result
            .expect_err("Expected error for unsupported operator")
            .contains("XOR操作符在graph表达式中不支持"));
    }

    #[test]
    fn test_parse_expression_from_string() {
        let result = parse_expression_from_string("5 + 3");
        assert!(result.is_ok());

        let expr = result.expect("Expected successful parsing of expression from string");
        assert!(matches!(expr, Expression::Binary { .. }));
    }
}
