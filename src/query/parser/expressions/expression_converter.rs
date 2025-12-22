//! 表达式转换器
//! 将AST表达式转换为graph表达式

use crate::core::Value;
use crate::core::types::expression::{BinaryOperator, Expression, LiteralValue, UnaryOperator};
use crate::query::parser::ast::{
    BinaryExpr, BinaryOp, CaseExpr, ConstantExpr, Expr, FunctionCallExpr, ListExpr, MapExpr,
    PredicateExpr, PropertyAccessExpr, SubscriptExpr, UnaryExpr, UnaryOp, VariableExpr,
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
        Expr::Predicate(expr) => convert_predicate_expr(expr),
    }
}

/// 转换常量表达式
fn convert_constant_expr(expr: &ConstantExpr) -> Result<Expression, String> {
    let literal_value = match &expr.value {
        Value::Bool(b) => LiteralValue::Bool(*b),
        Value::Int(i) => LiteralValue::Int(*i),
        Value::Float(f) => LiteralValue::Float(*f),
        Value::String(s) => LiteralValue::String(s.clone()),
        Value::Null(_) => LiteralValue::Null,
        _ => return Err(format!("不支持的常量值类型: {:?}", expr.value)),
    };
    Ok(Expression::Literal(literal_value))
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

/// 转换谓词表达式
fn convert_predicate_expr(expr: &PredicateExpr) -> Result<Expression, String> {
    let list = convert_ast_to_graph_expression(&expr.list)?;
    let condition = convert_ast_to_graph_expression(&expr.condition)?;

    Ok(Expression::Predicate {
        list: Box::new(list),
        condition: Box::new(condition),
    })
}

/// 转换二元操作符
fn convert_binary_op(op: &BinaryOp) -> Result<BinaryOperator, String> {
    match op {
        // 算术操作符
        BinaryOp::Add => Ok(BinaryOperator::Add),
        BinaryOp::Sub => Ok(BinaryOperator::Subtract),
        BinaryOp::Mul => Ok(BinaryOperator::Multiply),
        BinaryOp::Div => Ok(BinaryOperator::Divide),
        BinaryOp::Mod => Ok(BinaryOperator::Modulo),
        BinaryOp::Exp => Err("指数操作符在graph表达式中不支持".to_string()),

        // 逻辑操作符
        BinaryOp::And => Ok(BinaryOperator::And),
        BinaryOp::Or => Ok(BinaryOperator::Or),
        BinaryOp::Xor => Err("XOR操作符在graph表达式中不支持".to_string()),

        // 关系操作符
        BinaryOp::Eq => Ok(BinaryOperator::Equal),
        BinaryOp::Ne => Ok(BinaryOperator::NotEqual),
        BinaryOp::Lt => Ok(BinaryOperator::LessThan),
        BinaryOp::Le => Ok(BinaryOperator::LessThanOrEqual),
        BinaryOp::Gt => Ok(BinaryOperator::GreaterThan),
        BinaryOp::Ge => Ok(BinaryOperator::GreaterThanOrEqual),

        // 字符串操作符
        BinaryOp::Regex => Err("正则表达式操作符在graph表达式中不支持".to_string()),
        BinaryOp::In => Ok(BinaryOperator::In),
        BinaryOp::NotIn => Err("NOT IN操作符在graph表达式中不支持".to_string()),
        BinaryOp::Contains => Err("CONTAINS操作符在graph表达式中不支持".to_string()),
        BinaryOp::StartsWith => Err("STARTS WITH操作符在graph表达式中不支持".to_string()),
        BinaryOp::EndsWith => Err("ENDS WITH操作符在graph表达式中不支持".to_string()),
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
fn convert_aggregate_function(
    func_name: &str,
) -> Result<crate::core::types::expression::AggregateFunction, String> {
    match func_name {
        "COUNT" => Ok(crate::core::types::expression::AggregateFunction::Count),
        "SUM" => Ok(crate::core::types::expression::AggregateFunction::Sum),
        "AVG" => Ok(crate::core::types::expression::AggregateFunction::Avg),
        "MIN" => Ok(crate::core::types::expression::AggregateFunction::Min),
        "MAX" => Ok(crate::core::types::expression::AggregateFunction::Max),
        "COLLECT" => Ok(crate::core::types::expression::AggregateFunction::Collect),
        "DISTINCT" => Ok(crate::core::types::expression::AggregateFunction::Distinct),
        _ => Err(format!("不支持的聚合函数: {}", func_name)),
    }
}

/// 检查是否为聚合函数
fn is_aggregate_function(func_name: &str) -> bool {
    matches!(
        func_name,
        "COUNT" | "SUM" | "AVG" | "MIN" | "MAX" | "COLLECT" | "DISTINCT"
    )
}

/// 从字符串解析表达式
pub fn parse_expression_from_string(condition: &str) -> Result<Expression, String> {
    // 创建语法分析器
    let mut parser = crate::query::parser::parser::Parser::new(condition);
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
        BinaryExpr, BinaryOp, ConstantExpr, Expr, UnaryExpr, UnaryOp, VariableExpr,
    };

    #[test]
    fn test_convert_constant_expr() {
        let ast_expr = Expr::Constant(ConstantExpr::new(
            Value::Int(42),
            crate::query::parser::ast::Span::default(),
        ));
        let result = convert_ast_to_graph_expression(&ast_expr)
            .expect("Expected successful conversion of constant expression");

        if let Expression::Literal(LiteralValue::Int(value)) = result {
            assert_eq!(value, 42);
        } else {
            panic!("Expected Literal(Int(42)), got {:?}", result);
        }
    }

    #[test]
    fn test_convert_variable_expr() {
        let ast_expr = Expr::Variable(VariableExpr::new(
            "test_var".to_string(),
            crate::query::parser::ast::Span::default(),
        ));
        let result = convert_ast_to_graph_expression(&ast_expr)
            .expect("Expected successful conversion of variable expression");

        if let Expression::Variable(name) = result {
            assert_eq!(name, "test_var");
        } else {
            panic!("Expected Variable(\"test_var\"), got {:?}", result);
        }
    }

    #[test]
    fn test_convert_binary_expr() {
        let left = Expr::Constant(ConstantExpr::new(
            Value::Int(5),
            crate::query::parser::ast::Span::default(),
        ));
        let right = Expr::Constant(ConstantExpr::new(
            Value::Int(3),
            crate::query::parser::ast::Span::default(),
        ));
        let ast_expr = Expr::Binary(BinaryExpr::new(
            left,
            BinaryOp::Add,
            right,
            crate::query::parser::ast::Span::default(),
        ));

        let result = convert_ast_to_graph_expression(&ast_expr)
            .expect("Expected successful conversion of binary expression");

        if let Expression::Binary { left, op, right } = result {
            assert_eq!(*left, Expression::Literal(LiteralValue::Int(5)));
            assert_eq!(op, BinaryOperator::Add);
            assert_eq!(*right, Expression::Literal(LiteralValue::Int(3)));
        } else {
            panic!("Expected Binary expression, got {:?}", result);
        }
    }

    #[test]
    fn test_convert_unary_expr() {
        let operand = Expr::Constant(ConstantExpr::new(
            Value::Bool(true),
            crate::query::parser::ast::Span::default(),
        ));
        let ast_expr = Expr::Unary(UnaryExpr::new(
            UnaryOp::Not,
            operand,
            crate::query::parser::ast::Span::default(),
        ));

        let result = convert_ast_to_graph_expression(&ast_expr)
            .expect("Expected successful conversion of unary expression");

        if let Expression::Unary { op, operand } = result {
            assert_eq!(op, UnaryOperator::Not);
            assert_eq!(*operand, Expression::Literal(LiteralValue::Bool(true)));
        } else {
            panic!("Expected Unary expression, got {:?}", result);
        }
    }

    #[test]
    fn test_convert_unsupported_operator() {
        let left = Expr::Constant(ConstantExpr::new(
            Value::Int(5),
            crate::query::parser::ast::Span::default(),
        ));
        let right = Expr::Constant(ConstantExpr::new(
            Value::Int(3),
            crate::query::parser::ast::Span::default(),
        ));
        let ast_expr = Expr::Binary(BinaryExpr::new(
            left,
            BinaryOp::Exp,
            right,
            crate::query::parser::ast::Span::default(),
        ));

        let result = convert_ast_to_graph_expression(&ast_expr);
        assert!(result.is_err());
        assert!(result
            .expect_err("Expected error for unsupported operator")
            .contains("指数操作符在graph表达式中不支持"));
    }

    #[test]
    fn test_parse_expression_from_string() {
        let result = parse_expression_from_string("5 + 3");
        assert!(result.is_ok());

        let expr = result.expect("Expected successful parsing of expression from string");
        // 由于解析器可能返回复杂的表达式结构，我们只检查是否成功转换
        assert!(matches!(expr, Expression::Binary { .. }));
    }
}
