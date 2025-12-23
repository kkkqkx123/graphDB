use crate::core::ExpressionError;
use crate::query::parser::cypher::ast::expressions::{
    BinaryExpression, CaseAlternative, CaseExpression,
    Expression as CypherExpression, FunctionCall, ListExpression, Literal as CypherLiteral,
    MapExpression, UnaryExpression,
};
use crate::core::types::operators::UnaryOperator;

/// Cypher表达式优化器
///
/// 专注于Cypher表达式的优化，包括常量折叠、表达式简化等。
/// 与计划级别的优化器协调工作，避免重复优化。
pub struct CypherExpressionOptimizer;

impl CypherExpressionOptimizer {
    /// 优化Cypher表达式
    ///
    /// 对Cypher表达式进行基本优化，如常量折叠、简化布尔表达式等。
    /// 注意：这个函数只进行表达式级别的优化，不涉及计划级别的优化。
    pub fn optimize_cypher_expression(cypher_expr: &CypherExpression) -> CypherExpression {
        match cypher_expr {
            // 常量折叠
            CypherExpression::Binary(bin_expr) => {
                if Self::is_cypher_constant(&bin_expr.left)
                    && Self::is_cypher_constant(&bin_expr.right)
                {
                    // 如果两个操作数都是常量，可以预先计算结果
                    // 这里简化处理，实际实现需要创建临时上下文进行评估
                    Self::try_fold_constants(bin_expr).unwrap_or_else(|| {
                        CypherExpression::Binary(BinaryExpression {
                            left: Box::new(Self::optimize_cypher_expression(&bin_expr.left)),
                            operator: bin_expr.operator.clone(),
                            right: Box::new(Self::optimize_cypher_expression(&bin_expr.right)),
                        })
                    })
                } else {
                    CypherExpression::Binary(BinaryExpression {
                        left: Box::new(Self::optimize_cypher_expression(&bin_expr.left)),
                        operator: bin_expr.operator.clone(),
                        right: Box::new(Self::optimize_cypher_expression(&bin_expr.right)),
                    })
                }
            }
            // 递归优化子表达式
            CypherExpression::Unary(unary_expr) => {
                let optimized_operand = Self::optimize_cypher_expression(&unary_expr.expression);

                // 尝试对一元表达式进行常量折叠
                if Self::is_cypher_constant(&optimized_operand) {
                    Self::try_fold_unary_constant(&unary_expr.operator, &optimized_operand)
                        .unwrap_or_else(|| {
                            CypherExpression::Unary(UnaryExpression {
                                operator: unary_expr.operator.clone(),
                                expression: Box::new(optimized_operand),
                            })
                        })
                } else {
                    CypherExpression::Unary(UnaryExpression {
                        operator: unary_expr.operator.clone(),
                        expression: Box::new(optimized_operand),
                    })
                }
            }
            CypherExpression::FunctionCall(func_call) => {
                let optimized_args: Vec<CypherExpression> = func_call
                    .arguments
                    .iter()
                    .map(|arg| Self::optimize_cypher_expression(&arg))
                    .collect();

                // 尝试对常量参数的函数进行常量折叠
                if optimized_args.iter().all(Self::is_cypher_constant) {
                    Self::try_fold_function_constant(&func_call.function_name, &optimized_args)
                        .unwrap_or_else(|| {
                            CypherExpression::FunctionCall(FunctionCall {
                                function_name: func_call.function_name.clone(),
                                arguments: optimized_args,
                            })
                        })
                } else {
                    CypherExpression::FunctionCall(FunctionCall {
                        function_name: func_call.function_name.clone(),
                        arguments: optimized_args,
                    })
                }
            }
            CypherExpression::List(list_expr) => {
                let optimized_elements: Vec<CypherExpression> = list_expr
                    .elements
                    .iter()
                    .map(|elem| Self::optimize_cypher_expression(&elem))
                    .collect();
                CypherExpression::List(ListExpression {
                    elements: optimized_elements,
                })
            }
            CypherExpression::Map(map_expr) => {
                let mut optimized_properties = std::collections::HashMap::new();
                for (key, value) in &map_expr.properties {
                    optimized_properties
                        .insert(key.clone(), Self::optimize_cypher_expression(value));
                }
                CypherExpression::Map(MapExpression {
                    properties: optimized_properties,
                })
            }
            CypherExpression::Case(case_expr) => {
                let optimized_expression = case_expr
                    .expression
                    .as_ref()
                    .map(|expr| Box::new(Self::optimize_cypher_expression(expr)));

                let optimized_alternatives: Vec<CaseAlternative> = case_expr
                    .alternatives
                    .iter()
                    .map(|alt| CaseAlternative {
                        when_expression: Self::optimize_cypher_expression(&alt.when_expression),
                        then_expression: Self::optimize_cypher_expression(&alt.then_expression),
                    })
                    .collect();

                let optimized_default = case_expr
                     .default_alternative
                     .as_ref()
                     .map(|expr| Box::new(Self::optimize_cypher_expression(expr)));

                // 尝试简化CASE表达式
                Self::try_simplify_case_expression(
                    optimized_expression,
                    &optimized_alternatives,
                    optimized_default,
                )
            }
            // 其他表达式类型暂时不优化
            _ => cypher_expr.clone(),
        }
    }

    /// 尝试对二元表达式进行常量折叠
    fn try_fold_constants(bin_expr: &BinaryExpression) -> Option<CypherExpression> {
        // 创建临时的空上下文用于常量计算
        let mut context = crate::core::expressions::BasicExpressionContext::default();
        
        // 使用 CypherEvaluator 评估表达式
        match super::cypher_evaluator::CypherEvaluator::evaluate_cypher(
            &CypherExpression::Binary(bin_expr.clone()),
            &mut context
        ) {
            Ok(value) => {
                // 将评估结果转换回 Cypher 表达式
                match value {
                    crate::core::Value::Int(i) => Some(CypherExpression::Literal(CypherLiteral::Integer(i))),
                    crate::core::Value::Float(f) => Some(CypherExpression::Literal(CypherLiteral::Float(f))),
                    crate::core::Value::Bool(b) => Some(CypherExpression::Literal(CypherLiteral::Boolean(b))),
                    crate::core::Value::String(s) => Some(CypherExpression::Literal(CypherLiteral::String(s))),
                    crate::core::Value::Null(_) => Some(CypherExpression::Literal(CypherLiteral::Null)),
                    _ => None, // 复杂类型不进行常量折叠
                }
            }
            Err(_) => None, // 评估失败时不进行常量折叠
        }
    }

    /// 尝试对一元表达式进行常量折叠
    fn try_fold_unary_constant(
        operator: &UnaryOperator,
        operand: &CypherExpression,
    ) -> Option<CypherExpression> {
        match (operator, operand) {
            (UnaryOperator::Not, CypherExpression::Literal(CypherLiteral::Boolean(b))) => {
                Some(CypherExpression::Literal(CypherLiteral::Boolean(!b)))
            }
            (UnaryOperator::Plus, CypherExpression::Literal(lit)) => {
                Some(CypherExpression::Literal(lit.clone()))
            }
            (UnaryOperator::Minus, CypherExpression::Literal(CypherLiteral::Integer(i))) => {
                Some(CypherExpression::Literal(CypherLiteral::Integer(-i)))
            }
            (UnaryOperator::Minus, CypherExpression::Literal(CypherLiteral::Float(f))) => {
                Some(CypherExpression::Literal(CypherLiteral::Float(-f)))
            }
            _ => None,
        }
    }

    /// 尝试对函数调用进行常量折叠
    fn try_fold_function_constant(
        function_name: &str,
        args: &[CypherExpression],
    ) -> Option<CypherExpression> {
        match function_name.to_lowercase().as_str() {
            "abs" if args.len() == 1 => match &args[0] {
                CypherExpression::Literal(CypherLiteral::Integer(i)) => {
                    Some(CypherExpression::Literal(CypherLiteral::Integer(i.abs())))
                }
                CypherExpression::Literal(CypherLiteral::Float(f)) => {
                    Some(CypherExpression::Literal(CypherLiteral::Float(f.abs())))
                }
                _ => None,
            },
            "length" if args.len() == 1 => match &args[0] {
                CypherExpression::Literal(CypherLiteral::String(s)) => Some(
                    CypherExpression::Literal(CypherLiteral::Integer(s.len() as i64)),
                ),
                CypherExpression::List(list_expr) => Some(CypherExpression::Literal(
                    CypherLiteral::Integer(list_expr.elements.len() as i64),
                )),
                _ => None,
            },
            "size" if args.len() == 1 => match &args[0] {
                CypherExpression::List(list_expr) => Some(CypherExpression::Literal(
                    CypherLiteral::Integer(list_expr.elements.len() as i64),
                )),
                CypherExpression::Map(map_expr) => Some(CypherExpression::Literal(
                    CypherLiteral::Integer(map_expr.properties.len() as i64),
                )),
                _ => None,
            },
            _ => None,
        }
    }

    /// 尝试简化CASE表达式
    fn try_simplify_case_expression(
        expression: Option<Box<CypherExpression>>,
        alternatives: &[CaseAlternative],
        default: Option<Box<CypherExpression>>,
    ) -> CypherExpression {
        // 如果所有条件都是常量，可以简化为直接返回相应的结果
        let all_conditions_constant = alternatives
            .iter()
            .all(|alt| Self::is_cypher_constant(&alt.when_expression));

        if all_conditions_constant {
            // 找到第一个为真的条件
            for alt in alternatives {
                if Self::is_condition_true(&alt.when_expression) {
                    return alt.then_expression.clone();
                }
            }

            // 如果没有条件为真，返回默认值
            if let Some(default_expr) = default {
                return *default_expr;
            }
        }

        // 无法简化，返回优化后的CASE表达式
        CypherExpression::Case(CaseExpression {
            expression,
            alternatives: alternatives.to_vec(),
            default_alternative: default,
        })
    }

    /// 检查条件表达式是否为真
    fn is_condition_true(condition: &CypherExpression) -> bool {
        match condition {
            CypherExpression::Literal(CypherLiteral::Boolean(b)) => *b,
            CypherExpression::Literal(CypherLiteral::Integer(i)) => *i != 0,
            CypherExpression::Literal(CypherLiteral::Float(f)) => *f != 0.0,
            _ => false,
        }
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

    /// 批量优化Cypher表达式
    pub fn optimize_cypher_batch(cypher_exprs: &[CypherExpression]) -> Vec<CypherExpression> {
        cypher_exprs
            .iter()
            .map(|expr| Self::optimize_cypher_expression(&expr))
            .collect()
    }

    /// 检查表达式是否可以被优化
    pub fn can_optimize(cypher_expr: &CypherExpression) -> bool {
        match cypher_expr {
            CypherExpression::Binary(bin_expr) => {
                Self::can_optimize(&bin_expr.left)
                    || Self::can_optimize(&bin_expr.right)
                    || (Self::is_cypher_constant(&bin_expr.left)
                        && Self::is_cypher_constant(&bin_expr.right))
            }
            CypherExpression::Unary(unary_expr) => {
                Self::can_optimize(&unary_expr.expression)
                    || (Self::is_cypher_constant(&unary_expr.expression))
            }
            CypherExpression::FunctionCall(func_call) => {
                func_call.arguments.iter().any(Self::can_optimize)
                    || (func_call.arguments.iter().all(Self::is_cypher_constant)
                        && Self::is_foldable_function(&func_call.function_name))
            }
            CypherExpression::List(list_expr) => list_expr.elements.iter().any(Self::can_optimize),
            CypherExpression::Map(map_expr) => map_expr.properties.values().any(Self::can_optimize),
            CypherExpression::Case(case_expr) => {
                case_expr
                    .expression
                    .as_ref()
                    .map_or(false, |e| Self::can_optimize(e))
                    || case_expr.alternatives.iter().any(|alt| {
                        Self::can_optimize(&alt.when_expression)
                            || Self::can_optimize(&alt.then_expression)
                    })
                    || case_expr
                        .default_alternative
                        .as_ref()
                        .map_or(false, |e| Self::can_optimize(e))
            }
            _ => false,
        }
    }

    /// 检查函数是否可以进行常量折叠
    fn is_foldable_function(function_name: &str) -> bool {
        matches!(
            function_name.to_lowercase().as_str(),
            "abs" | "length" | "size" | "tostring" | "tointeger" | "tofloat" | "toboolean"
        )
    }

    /// 优化Cypher表达式（公开方法，与expression_evaluator兼容）
    pub fn optimize_cypher(cypher_expr: &CypherExpression) -> Result<CypherExpression, ExpressionError> {
        Ok(Self::optimize_cypher_expression(cypher_expr))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimize_unary_constant() {
        let expr = CypherExpression::Unary(UnaryExpression {
            operator: UnaryOperator::Not,
            expression: Box::new(CypherExpression::Literal(CypherLiteral::Boolean(true))),
        });

        let optimized = CypherExpressionOptimizer::optimize_cypher_expression(&expr);

        match optimized {
            CypherExpression::Literal(CypherLiteral::Boolean(b)) => assert_eq!(b, false),
            _ => panic!("Expected optimized boolean literal"),
        }
    }

    #[test]
    fn test_optimize_negative_constant() {
        let expr = CypherExpression::Unary(UnaryExpression {
            operator: UnaryOperator::Minus,
            expression: Box::new(CypherExpression::Literal(CypherLiteral::Integer(5))),
        });

        let optimized = CypherExpressionOptimizer::optimize_cypher_expression(&expr);

        match optimized {
            CypherExpression::Literal(CypherLiteral::Integer(i)) => assert_eq!(i, -5),
            _ => panic!("Expected optimized integer literal"),
        }
    }

    #[test]
    fn test_optimize_function_constant() {
        let args = vec![CypherExpression::Literal(CypherLiteral::Integer(-5))];
        let expr = CypherExpression::FunctionCall(FunctionCall {
            function_name: "abs".to_string(),
            arguments: args,
        });

        let optimized = CypherExpressionOptimizer::optimize_cypher_expression(&expr);

        match optimized {
            CypherExpression::Literal(CypherLiteral::Integer(i)) => assert_eq!(i, 5),
            _ => panic!("Expected optimized integer literal"),
        }
    }

    #[test]
    fn test_optimize_length_function() {
        let args = vec![CypherExpression::Literal(CypherLiteral::String(
            "hello".to_string(),
        ))];
        let expr = CypherExpression::FunctionCall(FunctionCall {
            function_name: "length".to_string(),
            arguments: args,
        });

        let optimized = CypherExpressionOptimizer::optimize_cypher_expression(&expr);

        match optimized {
            CypherExpression::Literal(CypherLiteral::Integer(i)) => assert_eq!(i, 5),
            _ => panic!("Expected optimized integer literal"),
        }
    }

    #[test]
    fn test_optimize_list_function() {
        let elements = vec![
            CypherExpression::Literal(CypherLiteral::Integer(1)),
            CypherExpression::Literal(CypherLiteral::Integer(2)),
            CypherExpression::Literal(CypherLiteral::Integer(3)),
        ];
        let args = vec![CypherExpression::List(ListExpression { elements })];
        let expr = CypherExpression::FunctionCall(FunctionCall {
            function_name: "size".to_string(),
            arguments: args,
        });

        let optimized = CypherExpressionOptimizer::optimize_cypher_expression(&expr);

        match optimized {
            CypherExpression::Literal(CypherLiteral::Integer(i)) => assert_eq!(i, 3),
            _ => panic!("Expected optimized integer literal"),
        }
    }

    #[test]
    fn test_can_optimize() {
        let constant_expr = CypherExpression::Literal(CypherLiteral::Integer(42));
        assert!(!CypherExpressionOptimizer::can_optimize(&constant_expr));

        let unary_expr = CypherExpression::Unary(UnaryExpression {
            operator: UnaryOperator::Not,
            expression: Box::new(CypherExpression::Literal(CypherLiteral::Boolean(true))),
        });
        assert!(CypherExpressionOptimizer::can_optimize(&unary_expr));

        let variable_expr = CypherExpression::Variable("x".to_string());
        assert!(!CypherExpressionOptimizer::can_optimize(&variable_expr));
    }

    #[test]
    fn test_is_foldable_function() {
        assert!(CypherExpressionOptimizer::is_foldable_function("abs"));
        assert!(CypherExpressionOptimizer::is_foldable_function("length"));
        assert!(CypherExpressionOptimizer::is_foldable_function("size"));
        assert!(!CypherExpressionOptimizer::is_foldable_function("count"));
        assert!(!CypherExpressionOptimizer::is_foldable_function("sum"));
    }
}
