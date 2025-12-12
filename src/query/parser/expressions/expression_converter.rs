//! 表达式转换器
//! 将AST表达式转换为graph表达式

use crate::graph::expression::expr_type::Expression;
use crate::graph::expression::binary::BinaryOperator;
use crate::graph::expression::unary::UnaryOperator;
use crate::query::parser::ast::expression::*;

/// 将AST表达式转换为graph表达式
pub fn convert_ast_to_graph_expression(ast_expr: &crate::query::parser::ast::Expression) -> Result<Expression, String> {
    match ast_expr {
        crate::query::parser::ast::Expression::Constant(value) => {
            Ok(Expression::Constant(value.clone()))
        }
        crate::query::parser::ast::Expression::Variable(name) => {
            Ok(Expression::Variable(name.clone()))
        }
        crate::query::parser::ast::Expression::FunctionCall(func_call) => {
            let args: Result<Vec<Expression>, String> = func_call.args
                .iter()
                .map(|arg| convert_ast_to_graph_expression(arg))
                .collect();
            Ok(Expression::Function(func_call.name.clone(), args?))
        }
        crate::query::parser::ast::Expression::PropertyAccess(expr, prop) => {
            let graph_expr = convert_ast_to_graph_expression(expr)?;
            // 根据表达式类型决定如何处理属性访问
            match &graph_expr {
                Expression::Variable(var_name) => {
                    Ok(Expression::VariableProperty {
                        var: var_name.clone(),
                        prop: prop.clone(),
                    })
                }
                _ => {
                    // 对于其他类型的表达式，暂时使用Property
                    Ok(Expression::Property(prop.clone()))
                }
            }
        }
        crate::query::parser::ast::Expression::AttributeAccess(expr, attr) => {
            let graph_expr = convert_ast_to_graph_expression(expr)?;
            // 根据表达式类型决定如何处理属性访问
            match &graph_expr {
                Expression::Variable(var_name) => {
                    Ok(Expression::TagProperty {
                        tag: var_name.clone(),
                        prop: attr.clone(),
                    })
                }
                _ => {
                    // 对于其他类型的表达式，暂时使用Property
                    Ok(Expression::Property(attr.clone()))
                }
            }
        }
        crate::query::parser::ast::Expression::Arithmetic(left, op, right) => {
            let left_expr = convert_ast_to_graph_expression(left)?;
            let right_expr = convert_ast_to_graph_expression(right)?;
            let binary_op = convert_arithmetic_op(op)?;
            Ok(Expression::BinaryOp(
                Box::new(left_expr),
                binary_op,
                Box::new(right_expr),
            ))
        }
        crate::query::parser::ast::Expression::Logical(left, op, right) => {
            let left_expr = convert_ast_to_graph_expression(left)?;
            let right_expr = convert_ast_to_graph_expression(right)?;
            let binary_op = convert_logical_op(op)?;
            Ok(Expression::BinaryOp(
                Box::new(left_expr),
                binary_op,
                Box::new(right_expr),
            ))
        }
        crate::query::parser::ast::Expression::Relational(left, op, right) => {
            let left_expr = convert_ast_to_graph_expression(left)?;
            let right_expr = convert_ast_to_graph_expression(right)?;
            let binary_op = convert_relational_op(op)?;
            Ok(Expression::BinaryOp(
                Box::new(left_expr),
                binary_op,
                Box::new(right_expr),
            ))
        }
        crate::query::parser::ast::Expression::Unary(op, expr) => {
            let inner_expr = convert_ast_to_graph_expression(expr)?;
            let unary_op = convert_unary_op(op)?;
            Ok(Expression::UnaryOp(unary_op, Box::new(inner_expr)))
        }
        crate::query::parser::ast::Expression::List(elements) => {
            let graph_elements: Result<Vec<Expression>, String> = elements
                .iter()
                .map(|elem| convert_ast_to_graph_expression(elem))
                .collect();
            Ok(Expression::List(graph_elements?))
        }
        crate::query::parser::ast::Expression::Map(pairs) => {
            let graph_pairs: Result<Vec<(String, Expression)>, String> = pairs
                .iter()
                .map(|(key, value)| {
                    let graph_value = convert_ast_to_graph_expression(value)?;
                    Ok((key.clone(), graph_value))
                })
                .collect();
            Ok(Expression::Map(graph_pairs?))
        }
        crate::query::parser::ast::Expression::Subscript(collection, index) => {
            let graph_collection = convert_ast_to_graph_expression(collection)?;
            let graph_index = convert_ast_to_graph_expression(index)?;
            Ok(Expression::Subscript {
                collection: Box::new(graph_collection),
                index: Box::new(graph_index),
            })
        }
        crate::query::parser::ast::Expression::Case(case_expr) => {
            let _match_expr = case_expr.match_expr.as_ref()
                .map(|expr| convert_ast_to_graph_expression(expr))
                .transpose()?
                .map(Box::new);
            
            let when_then_pairs: Result<Vec<(Expression, Expression)>, String> = case_expr.when_then_pairs
                .iter()
                .map(|(when, then)| {
                    let graph_when = convert_ast_to_graph_expression(when)?;
                    let graph_then = convert_ast_to_graph_expression(then)?;
                    Ok((graph_when, graph_then))
                })
                .collect();
            
            let default = case_expr.default.as_ref()
                .map(|expr| convert_ast_to_graph_expression(expr))
                .transpose()?
                .map(Box::new);
            
            Ok(Expression::Case {
                conditions: when_then_pairs?,
                default,
            })
        }
        crate::query::parser::ast::Expression::InList(expr, list) => {
            let graph_expr = convert_ast_to_graph_expression(expr)?;
            let graph_list: Result<Vec<Expression>, String> = list
                .iter()
                .map(|elem| convert_ast_to_graph_expression(elem))
                .collect();
            
            // 创建一个包含操作的表达式
            Ok(Expression::Function(
                "in_list".to_string(),
                vec![graph_expr, Expression::List(graph_list?)],
            ))
        }
        crate::query::parser::ast::Expression::NotInList(expr, list) => {
            let graph_expr = convert_ast_to_graph_expression(expr)?;
            let graph_list: Result<Vec<Expression>, String> = list
                .iter()
                .map(|elem| convert_ast_to_graph_expression(elem))
                .collect();
            
            // 创建一个包含操作的表达式，然后取反
            Ok(Expression::UnaryOp(
                UnaryOperator::Not,
                Box::new(Expression::Function(
                    "in_list".to_string(),
                    vec![graph_expr, Expression::List(graph_list?)],
                )),
            ))
        }
        crate::query::parser::ast::Expression::Contains(left, right) => {
            let left_expr = convert_ast_to_graph_expression(left)?;
            let right_expr = convert_ast_to_graph_expression(right)?;
            
            Ok(Expression::Function(
                "contains".to_string(),
                vec![left_expr, right_expr],
            ))
        }
        crate::query::parser::ast::Expression::StartsWith(left, right) => {
            let left_expr = convert_ast_to_graph_expression(left)?;
            let right_expr = convert_ast_to_graph_expression(right)?;
            
            Ok(Expression::Function(
                "starts_with".to_string(),
                vec![left_expr, right_expr],
            ))
        }
        crate::query::parser::ast::Expression::EndsWith(left, right) => {
            let left_expr = convert_ast_to_graph_expression(left)?;
            let right_expr = convert_ast_to_graph_expression(right)?;
            
            Ok(Expression::Function(
                "ends_with".to_string(),
                vec![left_expr, right_expr],
            ))
        }
        crate::query::parser::ast::Expression::IsNull(expr) => {
            let inner_expr = convert_ast_to_graph_expression(expr)?;
            Ok(Expression::IsNull(Box::new(inner_expr)))
        }
        crate::query::parser::ast::Expression::IsNotNull(expr) => {
            let inner_expr = convert_ast_to_graph_expression(expr)?;
            Ok(Expression::IsNotNull(Box::new(inner_expr)))
        }
        crate::query::parser::ast::Expression::All(list, condition) => {
            let graph_list = convert_ast_to_graph_expression(list)?;
            let graph_condition = convert_ast_to_graph_expression(condition)?;
            
            Ok(Expression::Function(
                "all".to_string(),
                vec![graph_list, graph_condition],
            ))
        }
        crate::query::parser::ast::Expression::Single(list, condition) => {
            let graph_list = convert_ast_to_graph_expression(list)?;
            let graph_condition = convert_ast_to_graph_expression(condition)?;
            
            Ok(Expression::Function(
                "single".to_string(),
                vec![graph_list, graph_condition],
            ))
        }
        crate::query::parser::ast::Expression::Any(list, condition) => {
            let graph_list = convert_ast_to_graph_expression(list)?;
            let graph_condition = convert_ast_to_graph_expression(condition)?;
            
            Ok(Expression::Function(
                "any".to_string(),
                vec![graph_list, graph_condition],
            ))
        }
        crate::query::parser::ast::Expression::None(list, condition) => {
            let graph_list = convert_ast_to_graph_expression(list)?;
            let graph_condition = convert_ast_to_graph_expression(condition)?;
            
            Ok(Expression::Function(
                "none".to_string(),
                vec![graph_list, graph_condition],
            ))
        }
    }
}

/// 转换算术操作符
fn convert_arithmetic_op(op: &ArithmeticOp) -> Result<BinaryOperator, String> {
    match op {
        ArithmeticOp::Add => Ok(BinaryOperator::Add),
        ArithmeticOp::Sub => Ok(BinaryOperator::Sub),
        ArithmeticOp::Mul => Ok(BinaryOperator::Mul),
        ArithmeticOp::Div => Ok(BinaryOperator::Div),
        ArithmeticOp::Mod => Ok(BinaryOperator::Mod),
        ArithmeticOp::Exp => Err("Exponentiation operator not supported in graph expressions".to_string()),
    }
}

/// 转换逻辑操作符
fn convert_logical_op(op: &LogicalOp) -> Result<BinaryOperator, String> {
    match op {
        LogicalOp::And => Ok(BinaryOperator::And),
        LogicalOp::Or => Ok(BinaryOperator::Or),
        LogicalOp::Xor => Err("XOR operator not supported in graph expressions".to_string()),
    }
}

/// 转换关系操作符
fn convert_relational_op(op: &RelationalOp) -> Result<BinaryOperator, String> {
    match op {
        RelationalOp::Eq => Ok(BinaryOperator::Eq),
        RelationalOp::Ne => Ok(BinaryOperator::Ne),
        RelationalOp::Lt => Ok(BinaryOperator::Lt),
        RelationalOp::Le => Ok(BinaryOperator::Le),
        RelationalOp::Gt => Ok(BinaryOperator::Gt),
        RelationalOp::Ge => Ok(BinaryOperator::Ge),
        RelationalOp::Regex => Err("Regex operator not supported in graph expressions".to_string()),
    }
}

/// 转换一元操作符
fn convert_unary_op(op: &UnaryOp) -> Result<UnaryOperator, String> {
    match op {
        UnaryOp::Not => Ok(UnaryOperator::Not),
        UnaryOp::Plus => Ok(UnaryOperator::Plus),
        UnaryOp::Minus => Ok(UnaryOperator::Minus),
    }
}

/// 从字符串解析表达式
pub fn parse_expression_from_string(condition: &str) -> Result<Expression, String> {
    // 创建语法分析器
    let mut parser = crate::query::parser::parser::Parser::new(condition);
    let ast_expr = parser.parse_expression().map_err(|e| format!("语法分析错误: {:?}", e))?;
    
    // 转换为graph表达式
    convert_ast_to_graph_expression(&ast_expr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_constant() {
        let ast_expr = crate::query::parser::ast::Expression::Constant(crate::core::Value::Int(42));
        let graph_expr = convert_ast_to_graph_expression(&ast_expr).unwrap();
        
        assert!(matches!(graph_expr, Expression::Constant(crate::core::Value::Int(42))));
    }

    #[test]
    fn test_convert_variable() {
        let ast_expr = crate::query::parser::ast::Expression::Variable("x".to_string());
        let graph_expr = convert_ast_to_graph_expression(&ast_expr).unwrap();
        
        assert!(matches!(graph_expr, Expression::Variable(ref v) if v == "x"));
    }

    #[test]
    fn test_convert_property_access() {
        let ast_expr = crate::query::parser::ast::Expression::PropertyAccess(
            Box::new(crate::query::parser::ast::Expression::Variable("person".to_string())),
            "name".to_string(),
        );
        let graph_expr = convert_ast_to_graph_expression(&ast_expr).unwrap();
        
        assert!(matches!(
            graph_expr,
            Expression::VariableProperty { ref var, ref prop } if var == "person" && prop == "name"
        ));
    }

    #[test]
    fn test_convert_arithmetic() {
        let ast_expr = crate::query::parser::ast::Expression::Arithmetic(
            Box::new(crate::query::parser::ast::Expression::Constant(crate::core::Value::Int(5))),
            ArithmeticOp::Add,
            Box::new(crate::query::parser::ast::Expression::Constant(crate::core::Value::Int(3))),
        );
        let graph_expr = convert_ast_to_graph_expression(&ast_expr).unwrap();
        
        match &graph_expr {
            Expression::BinaryOp(left, op, right) => {
                assert!(matches!(op, BinaryOperator::Add));
                assert!(matches!(left.as_ref(), Expression::Constant(crate::core::Value::Int(5))));
                assert!(matches!(right.as_ref(), Expression::Constant(crate::core::Value::Int(3))));
            }
            _ => panic!("Expected BinaryOp expression"),
        }
    }

    #[test]
    fn test_convert_logical() {
        let ast_expr = crate::query::parser::ast::Expression::Logical(
            Box::new(crate::query::parser::ast::Expression::Constant(crate::core::Value::Bool(true))),
            LogicalOp::And,
            Box::new(crate::query::parser::ast::Expression::Constant(crate::core::Value::Bool(false))),
        );
        let graph_expr = convert_ast_to_graph_expression(&ast_expr).unwrap();
        
        match &graph_expr {
            Expression::BinaryOp(left, op, right) => {
                assert!(matches!(op, BinaryOperator::And));
                assert!(matches!(left.as_ref(), Expression::Constant(crate::core::Value::Bool(true))));
                assert!(matches!(right.as_ref(), Expression::Constant(crate::core::Value::Bool(false))));
            }
            _ => panic!("Expected BinaryOp expression"),
        }
    }

    #[test]
    fn test_convert_relational() {
        let ast_expr = crate::query::parser::ast::Expression::Relational(
            Box::new(crate::query::parser::ast::Expression::Variable("x".to_string())),
            RelationalOp::Gt,
            Box::new(crate::query::parser::ast::Expression::Constant(crate::core::Value::Int(10))),
        );
        let graph_expr = convert_ast_to_graph_expression(&ast_expr).unwrap();
        
        match &graph_expr {
            Expression::BinaryOp(left, op, right) => {
                assert!(matches!(op, BinaryOperator::Gt));
                assert!(matches!(left.as_ref(), Expression::Variable(v) if v == "x"));
                assert!(matches!(right.as_ref(), Expression::Constant(crate::core::Value::Int(10))));
            }
            _ => panic!("Expected BinaryOp expression"),
        }
    }

    #[test]
    fn test_convert_unary() {
        let ast_expr = crate::query::parser::ast::Expression::Unary(
            UnaryOp::Not,
            Box::new(crate::query::parser::ast::Expression::Constant(crate::core::Value::Bool(true))),
        );
        let graph_expr = convert_ast_to_graph_expression(&ast_expr).unwrap();
        
        match &graph_expr {
            Expression::UnaryOp(op, expr) => {
                assert!(matches!(op, UnaryOperator::Not));
                assert!(matches!(expr.as_ref(), Expression::Constant(crate::core::Value::Bool(true))));
            }
            _ => panic!("Expected UnaryOp expression"),
        }
    }

    #[test]
    fn test_convert_function_call() {
        let ast_expr = crate::query::parser::ast::Expression::FunctionCall(FunctionCall {
            name: "count".to_string(),
            args: vec![crate::query::parser::ast::Expression::Variable("x".to_string())],
            distinct: false,
        });
        let graph_expr = convert_ast_to_graph_expression(&ast_expr).unwrap();
        
        match &graph_expr {
            Expression::Function(name, args) => {
                assert_eq!(name, "count");
                assert_eq!(args.len(), 1);
                assert!(matches!(&args[0], Expression::Variable(v) if v == "x"));
            }
            _ => panic!("Expected Function expression"),
        }
    }
}