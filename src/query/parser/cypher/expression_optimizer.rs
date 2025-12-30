//! Cypher表达式优化器
//!
//! 提供Cypher表达式的优化功能

/// Cypher表达式优化器
pub struct CypherExpressionOptimizer;

impl CypherExpressionOptimizer {
    /// 优化Cypher表达式
    pub fn optimize_cypher_expression(
        expr: &crate::query::parser::cypher::ast::expressions::Expression,
    ) -> crate::query::parser::cypher::ast::expressions::Expression {
        // 使用克隆来创建可变副本
        match expr.clone() {
            // 常量折叠：将常量表达式计算为单个值
            crate::query::parser::cypher::ast::expressions::Expression::Binary(ref bin_expr) => {
                // 递归优化左右子表达式
                let optimized_left = Box::new(Self::optimize_cypher_expression(&bin_expr.left));
                let optimized_right = Box::new(Self::optimize_cypher_expression(&bin_expr.right));

                // 尝试常量折叠
                match (&*optimized_left, &*optimized_right, bin_expr.operator) {
                    // 对于加法，如果两个操作数都是整数常量，则计算结果
                    (
                        crate::query::parser::cypher::ast::expressions::Expression::Literal(
                            crate::query::parser::cypher::ast::expressions::Literal::Integer(
                                left_val,
                            ),
                        ),
                        crate::query::parser::cypher::ast::expressions::Expression::Literal(
                            crate::query::parser::cypher::ast::expressions::Literal::Integer(
                                right_val,
                            ),
                        ),
                        crate::query::parser::cypher::ast::BinaryOperator::Add,
                    ) => crate::query::parser::cypher::ast::expressions::Expression::Literal(
                        crate::query::parser::cypher::ast::expressions::Literal::Integer(
                            left_val + right_val,
                        ),
                    ),
                    // 对于乘法，如果两个操作数都是整数常量，则计算结果
                    (
                        crate::query::parser::cypher::ast::expressions::Expression::Literal(
                            crate::query::parser::cypher::ast::expressions::Literal::Integer(
                                left_val,
                            ),
                        ),
                        crate::query::parser::cypher::ast::expressions::Expression::Literal(
                            crate::query::parser::cypher::ast::expressions::Literal::Integer(
                                right_val,
                            ),
                        ),
                        crate::query::parser::cypher::ast::BinaryOperator::Multiply,
                    ) => crate::query::parser::cypher::ast::expressions::Expression::Literal(
                        crate::query::parser::cypher::ast::expressions::Literal::Integer(
                            left_val * right_val,
                        ),
                    ),
                    // 其他情况，返回优化后的子表达式
                    _ => crate::query::parser::cypher::ast::expressions::Expression::Binary(
                        crate::query::parser::cypher::ast::expressions::BinaryExpression {
                            left: optimized_left,
                            operator: bin_expr.operator,
                            right: optimized_right,
                        },
                    ),
                }
            }
            // 递归优化其他类型的表达式
            crate::query::parser::cypher::ast::expressions::Expression::Literal(_) => expr.clone(),
            crate::query::parser::cypher::ast::expressions::Expression::Variable(_) => expr.clone(),
            crate::query::parser::cypher::ast::expressions::Expression::Property(prop_expr) => {
                crate::query::parser::cypher::ast::expressions::Expression::Property(
                    crate::query::parser::cypher::ast::expressions::PropertyExpression {
                        expression: Box::new(Self::optimize_cypher_expression(
                            &prop_expr.expression,
                        )),
                        property_name: prop_expr.property_name,
                    },
                )
            }
            crate::query::parser::cypher::ast::expressions::Expression::FunctionCall(func_call) => {
                let optimized_args: Vec<_> = func_call
                    .arguments
                    .iter()
                    .map(|arg| Self::optimize_cypher_expression(arg))
                    .collect();

                crate::query::parser::cypher::ast::expressions::Expression::FunctionCall(
                    crate::query::parser::cypher::ast::expressions::FunctionCall {
                        function_name: func_call.function_name,
                        arguments: optimized_args,
                    },
                )
            }
            crate::query::parser::cypher::ast::expressions::Expression::Unary(unary_expr) => {
                crate::query::parser::cypher::ast::expressions::Expression::Unary(
                    crate::query::parser::cypher::ast::expressions::UnaryExpression {
                        operator: unary_expr.operator,
                        expression: Box::new(Self::optimize_cypher_expression(
                            &unary_expr.expression,
                        )),
                    },
                )
            }
            crate::query::parser::cypher::ast::expressions::Expression::List(list_expr) => {
                let optimized_elements: Vec<_> = list_expr
                    .elements
                    .iter()
                    .map(|elem| Self::optimize_cypher_expression(elem))
                    .collect();

                crate::query::parser::cypher::ast::expressions::Expression::List(
                    crate::query::parser::cypher::ast::expressions::ListExpression {
                        elements: optimized_elements,
                    },
                )
            }
            crate::query::parser::cypher::ast::expressions::Expression::Map(map_expr) => {
                let mut optimized_properties = std::collections::HashMap::new();
                for (key, value) in map_expr.properties {
                    let optimized_value = Self::optimize_cypher_expression(&value);
                    optimized_properties.insert(key, optimized_value);
                }

                crate::query::parser::cypher::ast::expressions::Expression::Map(
                    crate::query::parser::cypher::ast::expressions::MapExpression {
                        properties: optimized_properties,
                    },
                )
            }
            crate::query::parser::cypher::ast::expressions::Expression::Case(case_expr) => {
                let optimized_alternatives: Vec<_> = case_expr
                    .alternatives
                    .into_iter()
                    .map(
                        |alt| crate::query::parser::cypher::ast::expressions::CaseAlternative {
                            when_expression: Self::optimize_cypher_expression(&alt.when_expression),
                            then_expression: Self::optimize_cypher_expression(&alt.then_expression),
                        },
                    )
                    .collect();

                let optimized_default = match case_expr.default_alternative {
                    Some(default_expr) => {
                        Some(Box::new(Self::optimize_cypher_expression(&default_expr)))
                    }
                    None => None,
                };

                crate::query::parser::cypher::ast::expressions::Expression::Case(
                    crate::query::parser::cypher::ast::expressions::CaseExpression {
                        expression: case_expr.expression,
                        alternatives: optimized_alternatives,
                        default_alternative: optimized_default,
                    },
                )
            }
            crate::query::parser::cypher::ast::expressions::Expression::PatternExpression(
                pattern_expr,
            ) => {
                // 模式表达式暂时不优化
                crate::query::parser::cypher::ast::expressions::Expression::PatternExpression(
                    pattern_expr,
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::parser::cypher::ast::expressions::{
        BinaryExpression, Expression as CypherExpression, Literal as CypherLiteral,
    };
    use crate::query::parser::cypher::ast::BinaryOperator;

    #[test]
    fn test_constant_folding() {
        // 创建表达式 2 + 3
        let left = Box::new(CypherExpression::Literal(CypherLiteral::Integer(2)));
        let right = Box::new(CypherExpression::Literal(CypherLiteral::Integer(3)));
        let expr = CypherExpression::Binary(BinaryExpression {
            left,
            operator: BinaryOperator::Add,
            right,
        });

        let optimized = CypherExpressionOptimizer::optimize_cypher_expression(&expr);

        // 应该优化为 5
        match optimized {
            CypherExpression::Literal(CypherLiteral::Integer(5)) => (),
            _ => panic!("Expected constant 5 after optimization"),
        }
    }

    #[test]
    fn test_nested_constant_folding() {
        // 创建表达式 (2 + 3) * 4
        let left_inner = Box::new(CypherExpression::Literal(CypherLiteral::Integer(2)));
        let right_inner = Box::new(CypherExpression::Literal(CypherLiteral::Integer(3)));
        let inner_expr = Box::new(CypherExpression::Binary(BinaryExpression {
            left: left_inner,
            operator: BinaryOperator::Add,
            right: right_inner,
        }));
        let outer_expr = Box::new(CypherExpression::Literal(CypherLiteral::Integer(4)));

        let expr = CypherExpression::Binary(BinaryExpression {
            left: inner_expr,
            operator: BinaryOperator::Multiply,
            right: outer_expr,
        });

        let optimized = CypherExpressionOptimizer::optimize_cypher_expression(&expr);

        // 由于我们只优化直接的常量操作，这里应该返回包含优化后子表达式的表达式
        match optimized {
            CypherExpression::Binary(ref bin) => {
                // 左侧应该是优化后的 2+3=5
                match &*bin.left {
                    CypherExpression::Literal(CypherLiteral::Integer(5)) => (),
                    _ => panic!("Expected left side to be optimized to 5"),
                }
            }
            _ => panic!("Expected binary expression"),
        }
    }

    #[test]
    fn test_no_optimization_for_variables() {
        // 创建表达式 x + 5
        let left = Box::new(CypherExpression::Variable("x".to_string()));
        let right = Box::new(CypherExpression::Literal(CypherLiteral::Integer(5)));
        let expr = CypherExpression::Binary(BinaryExpression {
            left,
            operator: BinaryOperator::Add,
            right,
        });

        let optimized = CypherExpressionOptimizer::optimize_cypher_expression(&expr);

        // 包含变量的表达式不应被优化
        match optimized {
            CypherExpression::Binary(_) => (), // 仍然是二元表达式
            _ => panic!("Expected binary expression with variable"),
        }
    }
}
