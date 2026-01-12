//! Cypher表达式转换器
//!
//! 提供Cypher表达式与统一表达式系统之间的转换功能

use crate::core::error::ExpressionError;
use crate::core::types::expression::Expression;

/// Cypher表达式转换器
pub struct ExpressionConverter;

impl ExpressionConverter {
    /// 将Cypher表达式转换为统一表达式
    pub fn convert_cypher_to_unified(
        cypher_expr: &crate::query::parser::cypher::ast::expressions::Expression,
    ) -> Result<Expression, ExpressionError> {
        match cypher_expr {
            crate::query::parser::cypher::ast::expressions::Expression::Literal(literal) => {
                let value =
                    super::expression_evaluator::CypherEvaluator::cypher_literal_to_value(literal)?;
                Ok(Expression::Literal(value))
            }
            crate::query::parser::cypher::ast::expressions::Expression::Variable(name) => {
                Ok(Expression::Variable(name.clone()))
            }
            crate::query::parser::cypher::ast::expressions::Expression::Property(prop_expr) => {
                let object_expr = Self::convert_cypher_to_unified(&prop_expr.expression)?;
                Ok(Expression::Property {
                    object: Box::new(object_expr),
                    property: prop_expr.property_name.clone(),
                })
            }
            crate::query::parser::cypher::ast::expressions::Expression::FunctionCall(func_call) => {
                let args: Result<Vec<Expression>, ExpressionError> = func_call
                    .arguments
                    .iter()
                    .map(|arg| Self::convert_cypher_to_unified(arg))
                    .collect();
                Ok(Expression::Function {
                    name: func_call.function_name.clone(),
                    args: args?,
                })
            }
            crate::query::parser::cypher::ast::expressions::Expression::Binary(bin_expr) => {
                let left = Self::convert_cypher_to_unified(&bin_expr.left)?;
                let right = Self::convert_cypher_to_unified(&bin_expr.right)?;
                // BinaryOperator已统一，直接使用
                Ok(Expression::Binary {
                    left: Box::new(left),
                    op: bin_expr.operator,
                    right: Box::new(right),
                })
            }
            crate::query::parser::cypher::ast::expressions::Expression::Unary(unary_expr) => {
                let operand = Self::convert_cypher_to_unified(&unary_expr.expression)?;
                // UnaryOperator已统一，直接使用
                Ok(Expression::Unary {
                    op: unary_expr.operator,
                    operand: Box::new(operand),
                })
            }
            crate::query::parser::cypher::ast::expressions::Expression::List(list_expr) => {
                let elements: Result<Vec<Expression>, ExpressionError> = list_expr
                    .elements
                    .iter()
                    .map(|elem| Self::convert_cypher_to_unified(elem))
                    .collect();
                Ok(Expression::List(elements?))
            }
            crate::query::parser::cypher::ast::expressions::Expression::Map(map_expr) => {
                let pairs: Result<Vec<(String, Expression)>, ExpressionError> = map_expr
                    .properties
                    .iter()
                    .map(|(key, value)| {
                        let value_expr = Self::convert_cypher_to_unified(value)?;
                        Ok((key.clone(), value_expr))
                    })
                    .collect();
                Ok(Expression::Map(pairs?))
            }
            crate::query::parser::cypher::ast::expressions::Expression::PatternExpression(_) => Ok(
                Expression::Literal(crate::core::Value::String("Pattern".to_string())),
            ),
            crate::query::parser::cypher::ast::expressions::Expression::Case(case_expr) => {
                let mut conditions = Vec::new();
                for alternative in &case_expr.alternatives {
                    let when_expr = Self::convert_cypher_to_unified(&alternative.when_expression)?;
                    let then_expr = Self::convert_cypher_to_unified(&alternative.then_expression)?;
                    conditions.push((when_expr, then_expr));
                }

                let default = match &case_expr.default_alternative {
                    Some(default_expr) => {
                        Some(Box::new(Self::convert_cypher_to_unified(default_expr)?))
                    }
                    None => None,
                };

                Ok(Expression::Case {
                    conditions,
                    default,
                })
            }
        }
    }

    /// 将统一表达式转换为Cypher表达式
    ///
    /// 这个方法主要用于调试和测试，在实际查询执行中不常用
    pub fn convert_unified_to_cypher(
        expr: &Expression,
    ) -> Result<crate::query::parser::cypher::ast::expressions::Expression, ExpressionError> {
        match expr {
            Expression::Literal(value) => {
                let cypher_literal = match value {
                    crate::core::Value::String(s) => {
                        crate::query::parser::cypher::ast::expressions::Literal::String(s.clone())
                    }
                    crate::core::Value::Int(i) => {
                        crate::query::parser::cypher::ast::expressions::Literal::Integer(*i)
                    }
                    crate::core::Value::Float(f) => {
                        crate::query::parser::cypher::ast::expressions::Literal::Float(*f)
                    }
                    crate::core::Value::Bool(b) => {
                        crate::query::parser::cypher::ast::expressions::Literal::Boolean(*b)
                    }
                    crate::core::Value::Null(_) => {
                        crate::query::parser::cypher::ast::expressions::Literal::Null
                    }
                    _ => {
                        return Err(ExpressionError::invalid_operation(
                            "Unsupported value type".to_string(),
                        ))
                    }
                };
                Ok(
                    crate::query::parser::cypher::ast::expressions::Expression::Literal(
                        cypher_literal,
                    ),
                )
            }
            Expression::Variable(name) => Ok(
                crate::query::parser::cypher::ast::expressions::Expression::Variable(name.clone()),
            ),
            Expression::Property { object, property } => {
                let object_expr = Self::convert_unified_to_cypher(object)?;
                Ok(
                    crate::query::parser::cypher::ast::expressions::Expression::Property(
                        crate::query::parser::cypher::ast::expressions::PropertyExpression {
                            expression: Box::new(object_expr),
                            property_name: property.clone(),
                        },
                    ),
                )
            }
            Expression::Function { name, args } => {
                let converted_args: Result<
                    Vec<crate::query::parser::cypher::ast::expressions::Expression>,
                    ExpressionError,
                > = args
                    .iter()
                    .map(|arg| Self::convert_unified_to_cypher(arg))
                    .collect();
                Ok(
                    crate::query::parser::cypher::ast::expressions::Expression::FunctionCall(
                        crate::query::parser::cypher::ast::expressions::FunctionCall {
                            function_name: name.clone(),
                            arguments: converted_args?,
                        },
                    ),
                )
            }
            Expression::Binary { left, op, right } => {
                let left_expr = Self::convert_unified_to_cypher(left)?;
                let right_expr = Self::convert_unified_to_cypher(right)?;
                // BinaryOperator已统一，直接使用
                Ok(
                    crate::query::parser::cypher::ast::expressions::Expression::Binary(
                        crate::query::parser::cypher::ast::expressions::BinaryExpression {
                            left: Box::new(left_expr),
                            operator: *op,
                            right: Box::new(right_expr),
                        },
                    ),
                )
            }
            Expression::Unary { op, operand } => {
                let operand_expr = Self::convert_unified_to_cypher(operand)?;
                // UnaryOperator已统一，直接使用
                Ok(
                    crate::query::parser::cypher::ast::expressions::Expression::Unary(
                        crate::query::parser::cypher::ast::expressions::UnaryExpression {
                            operator: *op,
                            expression: Box::new(operand_expr),
                        },
                    ),
                )
            }
            Expression::List(elements) => {
                let converted_elements: Result<
                    Vec<crate::query::parser::cypher::ast::expressions::Expression>,
                    ExpressionError,
                > = elements
                    .iter()
                    .map(|elem| Self::convert_unified_to_cypher(elem))
                    .collect();
                Ok(
                    crate::query::parser::cypher::ast::expressions::Expression::List(
                        crate::query::parser::cypher::ast::expressions::ListExpression {
                            elements: converted_elements?,
                        },
                    ),
                )
            }
            Expression::Map(pairs) => {
                let mut properties = std::collections::HashMap::new();
                for (key, value) in pairs {
                    let value_expr = Self::convert_unified_to_cypher(value)?;
                    properties.insert(key.clone(), value_expr);
                }
                Ok(
                    crate::query::parser::cypher::ast::expressions::Expression::Map(
                        crate::query::parser::cypher::ast::expressions::MapExpression {
                            properties,
                        },
                    ),
                )
            }
            Expression::Case {
                conditions,
                default,
            } => {
                let mut alternatives = Vec::new();
                for (when_expr, then_expr) in conditions {
                    let when_cypher = Self::convert_unified_to_cypher(when_expr)?;
                    let then_cypher = Self::convert_unified_to_cypher(then_expr)?;
                    alternatives.push(
                        crate::query::parser::cypher::ast::expressions::CaseAlternative {
                            when_expression: when_cypher,
                            then_expression: then_cypher,
                        },
                    );
                }

                let default_alternative = match default {
                    Some(default_expr) => {
                        Some(Box::new(Self::convert_unified_to_cypher(default_expr)?))
                    }
                    None => None,
                };

                Ok(
                    crate::query::parser::cypher::ast::expressions::Expression::Case(
                        crate::query::parser::cypher::ast::expressions::CaseExpression {
                            expression: None,
                            alternatives,
                            default_alternative,
                        },
                    ),
                )
            }
            // 对于其他未处理的情况，返回错误
            _ => Err(ExpressionError::invalid_operation(format!(
                "Unsupported expression type: {:?}",
                expr
            ))),
        }
    }

    /// 批量转换Cypher表达式为统一表达式
    pub fn convert_cypher_batch_to_unified(
        cypher_exprs: &[crate::query::parser::cypher::ast::expressions::Expression],
    ) -> Result<Vec<Expression>, ExpressionError> {
        let mut results = Vec::new();
        for expr in cypher_exprs {
            results.push(Self::convert_cypher_to_unified(expr)?);
        }
        Ok(results)
    }

    /// 批量转换统一表达式为Cypher表达式
    pub fn convert_unified_batch_to_cypher(
        exprs: &[Expression],
    ) -> Result<Vec<crate::query::parser::cypher::ast::expressions::Expression>, ExpressionError>
    {
        let mut results = Vec::new();
        for expr in exprs {
            results.push(Self::convert_unified_to_cypher(expr)?);
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::expression::Expression as UnifiedExpression;
    use crate::query::parser::cypher::ast::expressions::{
        BinaryExpression, Expression as CypherExpression, FunctionCall, ListExpression,
        Literal as CypherLiteral, MapExpression, PropertyExpression, UnaryExpression,
    };
    use crate::query::parser::cypher::ast::BinaryOperator;

    #[test]
    fn test_convert_literal() {
        let cypher_expr = CypherExpression::Literal(CypherLiteral::Integer(42));
        let unified_expr = ExpressionConverter::convert_cypher_to_unified(&cypher_expr)
            .expect("Conversion from cypher to unified should succeed for literals");

        match unified_expr {
            UnifiedExpression::Literal(crate::core::Value::Int(i)) => assert_eq!(i, 42),
            _ => panic!("Expected integer literal"),
        }
    }

    #[test]
    fn test_convert_variable() {
        let cypher_expr = CypherExpression::Variable("x".to_string());
        let unified_expr = ExpressionConverter::convert_cypher_to_unified(&cypher_expr)
            .expect("Conversion from cypher to unified should succeed for variables");

        match unified_expr {
            UnifiedExpression::Variable(name) => assert_eq!(name, "x"),
            _ => panic!("Expected variable"),
        }
    }

    #[test]
    fn test_convert_binary_expression() {
        let left = Box::new(CypherExpression::Literal(CypherLiteral::Integer(1)));
        let right = Box::new(CypherExpression::Literal(CypherLiteral::Integer(2)));
        let cypher_expr = CypherExpression::Binary(BinaryExpression {
            left,
            operator: BinaryOperator::Add,
            right,
        });

        let unified_expr = ExpressionConverter::convert_cypher_to_unified(&cypher_expr)
            .expect("Conversion from cypher to unified should succeed for binary operations");

        match unified_expr {
            UnifiedExpression::Binary {
                left: _,
                op,
                right: _,
            } => {
                // 验证操作符转换正确
                assert_eq!(format!("{:?}", op), "Add");
            }
            _ => panic!("Expected binary expression"),
        }
    }

    #[test]
    fn test_round_trip_conversion() {
        let original = CypherExpression::Literal(CypherLiteral::String("test".to_string()));
        let unified = ExpressionConverter::convert_cypher_to_unified(&original)
            .expect("Conversion from cypher to unified should succeed for round trip");
        let back_to_cypher = ExpressionConverter::convert_unified_to_cypher(&unified)
            .expect("Conversion from unified to cypher should succeed for round trip");

        match back_to_cypher {
            CypherExpression::Literal(CypherLiteral::String(s)) => assert_eq!(s, "test"),
            _ => panic!("Expected string literal"),
        }
    }
}
