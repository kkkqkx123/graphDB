//! Cypher表达式求值器
//!
//! 提供对Cypher表达式的求值功能

use crate::core::error::ExpressionError;
use crate::core::value::Value;
use crate::expression::ExpressionContext;

/// Cypher表达式求值器
pub struct CypherEvaluator;

impl CypherEvaluator {
    /// 求值Cypher表达式
    pub fn evaluate_cypher(
        cypher_expr: &crate::query::parser::cypher::ast::expressions::Expression,
        context: &mut dyn ExpressionContext,
    ) -> Result<Value, ExpressionError> {
        match cypher_expr {
            crate::query::parser::cypher::ast::expressions::Expression::Literal(literal) => {
                Ok(Self::cypher_literal_to_value(literal)?)
            }
            crate::query::parser::cypher::ast::expressions::Expression::Variable(name) => context
                .get_variable(name)
                .ok_or_else(|| ExpressionError::undefined_variable(name)),
            crate::query::parser::cypher::ast::expressions::Expression::Property(prop_expr) => {
                let obj_value = Self::evaluate_cypher(&prop_expr.expression, context)?;
                // 简单实现：对于Vertex类型的对象，获取其属性
                match obj_value {
                    Value::Vertex(vertex) => {
                        // 尝试从顶点获取属性
                        if let Some(prop_value) = vertex.properties.get(&prop_expr.property_name) {
                            Ok(prop_value.clone())
                        } else {
                            Ok(Value::Null(crate::core::NullType::UnknownProp))
                        }
                    }
                    _ => Err(ExpressionError::type_error(
                        "属性访问需要对象类型".to_string(),
                    )),
                }
            }
            crate::query::parser::cypher::ast::expressions::Expression::FunctionCall(func_call) => {
                // 暂时返回未实现的错误
                Err(ExpressionError::runtime_error(format!(
                    "函数调用: {}",
                    func_call.function_name
                )))
            }
            crate::query::parser::cypher::ast::expressions::Expression::Binary(bin_expr) => {
                let left_val = Self::evaluate_cypher(&bin_expr.left, context)?;
                let right_val = Self::evaluate_cypher(&bin_expr.right, context)?;
                crate::expression::evaluator::operations::BinaryOperationEvaluator::evaluate(
                    &left_val,
                    &bin_expr.operator,
                    &right_val,
                )
            }
            crate::query::parser::cypher::ast::expressions::Expression::Unary(unary_expr) => {
                let operand_val = Self::evaluate_cypher(&unary_expr.expression, context)?;
                crate::expression::evaluator::operations::UnaryOperationEvaluator::evaluate(
                    &unary_expr.operator,
                    &operand_val,
                )
            }
            crate::query::parser::cypher::ast::expressions::Expression::List(list_expr) => {
                let mut values = Vec::new();
                for element in &list_expr.elements {
                    values.push(Self::evaluate_cypher(element, context)?);
                }
                Ok(Value::List(values))
            }
            crate::query::parser::cypher::ast::expressions::Expression::Map(map_expr) => {
                let mut map_values = std::collections::HashMap::new();
                for (key, value_expr) in &map_expr.properties {
                    let value = Self::evaluate_cypher(value_expr, context)?;
                    map_values.insert(key.clone(), value);
                }
                Ok(Value::Map(map_values))
            }
            crate::query::parser::cypher::ast::expressions::Expression::Case(case_expr) => {
                // 简单实现CASE表达式
                for alternative in &case_expr.alternatives {
                    let condition_result =
                        Self::evaluate_cypher(&alternative.when_expression, context)?;
                    if matches!(condition_result, Value::Bool(true)) {
                        return Self::evaluate_cypher(&alternative.then_expression, context);
                    }
                }

                if let Some(default_expr) = &case_expr.default_alternative {
                    Self::evaluate_cypher(default_expr, context)
                } else {
                    Ok(Value::Null(crate::core::NullType::Null))
                }
            }
            crate::query::parser::cypher::ast::expressions::Expression::PatternExpression(_) => {
                Err(ExpressionError::runtime_error("模式表达式求值".to_string()))
            }
            crate::query::parser::cypher::ast::expressions::Expression::ListComprehension(_) => {
                Err(ExpressionError::runtime_error("列表推导式求值".to_string()))
            }
            crate::query::parser::cypher::ast::expressions::Expression::Reduce(_) => {
                Err(ExpressionError::runtime_error("Reduce表达式求值".to_string()))
            }
            crate::query::parser::cypher::ast::expressions::Expression::Aggregate(_) => {
                Err(ExpressionError::runtime_error("聚合表达式求值".to_string()))
            }
            crate::query::parser::cypher::ast::expressions::Expression::Predicate(_) => {
                Err(ExpressionError::runtime_error("谓词表达式求值".to_string()))
            }
            crate::query::parser::cypher::ast::expressions::Expression::TypeCasting(_) => {
                Err(ExpressionError::runtime_error("类型转换表达式求值".to_string()))
            }
        }
    }

    /// 将Cypher字面量转换为Value
    pub fn cypher_literal_to_value(
        literal: &crate::query::parser::cypher::ast::expressions::Literal,
    ) -> Result<Value, ExpressionError> {
        match literal {
            crate::query::parser::cypher::ast::expressions::Literal::String(s) => {
                Ok(Value::String(s.clone()))
            }
            crate::query::parser::cypher::ast::expressions::Literal::Integer(i) => {
                Ok(Value::Int(*i))
            }
            crate::query::parser::cypher::ast::expressions::Literal::Float(f) => {
                Ok(Value::Float(*f))
            }
            crate::query::parser::cypher::ast::expressions::Literal::Boolean(b) => {
                Ok(Value::Bool(*b))
            }
            crate::query::parser::cypher::ast::expressions::Literal::Null => {
                Ok(Value::Null(crate::core::NullType::Null))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expression::BasicExpressionContext;
    use crate::query::parser::cypher::ast::expressions::{
        BinaryExpression, Expression as CypherExpression, Literal as CypherLiteral,
    };
    use crate::query::parser::cypher::ast::BinaryOperator;

    #[test]
    fn test_evaluate_cypher_literal() {
        let mut context = BasicExpressionContext::default();
        let cypher_expr = CypherExpression::Literal(CypherLiteral::Integer(42));

        let result = CypherEvaluator::evaluate_cypher(&cypher_expr, &mut context)
            .expect("Cypher evaluation should succeed for literals");
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_evaluate_cypher_binary() {
        let mut context = BasicExpressionContext::default();
        let left = Box::new(CypherExpression::Literal(CypherLiteral::Integer(10)));
        let right = Box::new(CypherExpression::Literal(CypherLiteral::Integer(5)));
        let cypher_expr = CypherExpression::Binary(BinaryExpression {
            left,
            operator: BinaryOperator::Add,
            right,
        });

        let result = CypherEvaluator::evaluate_cypher(&cypher_expr, &mut context)
            .expect("Cypher evaluation should succeed for binary operations");
        assert_eq!(result, Value::Int(15));
    }
}
