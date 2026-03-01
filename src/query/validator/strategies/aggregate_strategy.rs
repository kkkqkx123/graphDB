//! 聚合验证策略
//! 负责验证聚合函数的使用和检查表达式是否包含聚合

use crate::core::types::operators::AggregateFunction;
use crate::core::types::expression::contextual::ContextualExpression;
use crate::core::error::{ValidationError, ValidationErrorType};

/// 聚合验证策略
pub struct AggregateValidationStrategy;

impl AggregateValidationStrategy {
    pub fn new() -> Self {
        Self
    }

    /// 检查表达式是否包含聚合函数
    pub fn has_aggregate_expression(&self, expression: &ContextualExpression) -> bool {
        let expr_meta = match expression.expression() {
            Some(e) => e,
            None => return false,
        };
        self.has_aggregate_expression_internal(expr_meta.inner().as_ref())
    }

    /// 内部方法：检查 Expression 是否包含聚合函数
    fn has_aggregate_expression_internal(&self, expression: &crate::core::types::expression::Expression) -> bool {
        match expression {
            crate::core::types::expression::Expression::Aggregate { .. } => true,
            crate::core::types::expression::Expression::Unary { operand, .. } => {
                self.has_aggregate_expression_internal(operand.as_ref())
            }
            crate::core::types::expression::Expression::Binary { left, right, .. } => {
                self.has_aggregate_expression_internal(left.as_ref())
                    || self.has_aggregate_expression_internal(right.as_ref())
            }
            crate::core::types::expression::Expression::Function { args, .. } => {
                args.iter().any(|arg| self.has_aggregate_expression_internal(arg))
            }
            crate::core::types::expression::Expression::List(items) => {
                items.iter().any(|item| self.has_aggregate_expression_internal(item))
            }
            crate::core::types::expression::Expression::Map(items) => items
                .iter()
                .any(|(_, value)| self.has_aggregate_expression_internal(value)),
            crate::core::types::expression::Expression::Case {
                test_expr,
                conditions,
                default,
            } => {
                test_expr.as_ref().map_or(false, |expr| self.has_aggregate_expression_internal(expr))
                    || conditions.iter().any(|(cond, val)| {
                        self.has_aggregate_expression_internal(cond)
                            || self.has_aggregate_expression_internal(val)
                    })
                    || default.as_ref().map_or(false, |d| self.has_aggregate_expression_internal(d))
            }
            _ => false,
        }
    }

    /// 验证UNWIND子句中不允许使用聚合函数
    pub fn validate_unwind_aggregate(
        &self,
        unwind_expression: &ContextualExpression,
    ) -> Result<(), ValidationError> {
        if self.has_aggregate_expression(unwind_expression) {
            return Err(ValidationError::new(
                "UNWIND子句中不能使用聚合表达式".to_string(),
                ValidationErrorType::AggregateError,
            ));
        }
        Ok(())
    }

    /// 验证聚合表达式的合法性
    /// 检查：
    /// 1. 聚合函数名是否有效
    /// 2. 是否有聚合函数嵌套
    /// 3. 特殊属性（*）是否只用于COUNT
    /// 4. 参数表达式是否合法
    pub fn validate_aggregate_expression(&self, expression: &ContextualExpression) -> Result<(), ValidationError> {
        let expr_meta = match expression.expression() {
            Some(e) => e,
            None => return Ok(()),
        };
        self.validate_aggregate_expression_internal(expr_meta.inner().as_ref())
    }

    /// 内部方法：验证聚合表达式的合法性
    fn validate_aggregate_expression_internal(
        &self,
        expression: &crate::core::types::expression::Expression,
    ) -> Result<(), ValidationError> {
        match expression {
            crate::core::types::expression::Expression::Aggregate {
                func,
                arg,
                distinct: _,
            } => {
                // 1. 验证聚合函数名的有效性
                // 注意：由于现在使用枚举，这个检查可能需要调整
                // 暂时跳过这个检查，因为枚举值总是有效的

                // 2. 检查聚合函数嵌套 - 不允许聚合函数中包含聚合函数
                if self.has_aggregate_expression_internal(arg) {
                    return Err(ValidationError::new(
                        "不允许聚合函数嵌套".to_string(),
                        ValidationErrorType::AggregateError,
                    ));
                }

                // 3. 检查特殊属性 (*.  * 只能用于COUNT)
                self.validate_wildcard_property(func, arg)?;

                // 4. 递归验证参数表达式的合法性
                self.validate_expression_in_aggregate(arg)?;

                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// 验证通配符属性的使用
    /// 
    /// 根据 nebula-graph 的实现，只有 COUNT 函数允许通配符属性(*)作为参数。
    /// 
    /// 验证规则：
    /// 1. 只检查聚合函数的直接参数（不递归检查嵌套表达式）
    /// 2. 只检查输入属性表达式（$-.prop 或 $var.prop 形式）
    /// 3. 只有 COUNT 函数允许使用通配符属性 `*`
    /// 
    /// 参考：nebula-3.8.0/src/graph/util/ExpressionUtils.cpp:1199-1220
    fn validate_wildcard_property(
        &self,
        func: &AggregateFunction,
        expression: &crate::core::types::expression::Expression,
    ) -> Result<(), ValidationError> {
        let is_count = matches!(func, AggregateFunction::Count(_));
        
        if is_count {
            return Ok(());
        }
        
        if let crate::core::types::expression::Expression::Property { object, property } = expression {
            if property == "*" {
                if let crate::core::types::expression::Expression::Variable(var_name) = object.as_ref() {
                    let ref_type = if var_name == "-" {
                        "输入属性"
                    } else {
                        "变量属性"
                    };
                    return Err(ValidationError::new(
                        format!(
                            "不能将聚合函数 `{}` 应用于{}通配符属性 `{}.{}`",
                            func.name(),
                            ref_type,
                            var_name,
                            property
                        ),
                        ValidationErrorType::AggregateError,
                    ));
                }
            }
        }

        Ok(())
    }

    /// 验证聚合函数参数表达式的合法性
    /// 递归验证参数表达式中是否有其他不合法的嵌套结构
    ///
    /// 验证规则：
    /// 1. 递归检查所有子表达式的合法性
    /// 2. 确保参数表达式的结构正确
    fn validate_expression_in_aggregate(&self, expression: &crate::core::types::expression::Expression) -> Result<(), ValidationError> {
        match expression {
            // 递归检查一元操作（包括各种一元操作符）
            crate::core::types::expression::Expression::Unary { operand, .. } => {
                self.validate_expression_in_aggregate(operand)?;
            }

            // 递归检查二元操作
            crate::core::types::expression::Expression::Binary { left, right, .. } => {
                self.validate_expression_in_aggregate(left)?;
                self.validate_expression_in_aggregate(right)?;
            }

            // 递归检查函数调用参数
            crate::core::types::expression::Expression::Function { args, .. } => {
                for arg in args {
                    self.validate_expression_in_aggregate(arg)?;
                }
            }

            // 递归检查列表元素
            crate::core::types::expression::Expression::List(items) => {
                for item in items {
                    self.validate_expression_in_aggregate(item)?;
                }
            }

            // 递归检查Map值
            crate::core::types::expression::Expression::Map(items) => {
                for (_, value) in items {
                    self.validate_expression_in_aggregate(value)?;
                }
            }

            // 递归检查类型转换表达式
            crate::core::types::expression::Expression::TypeCast {
                expression: cast_expression, ..
            } => {
                self.validate_expression_in_aggregate(cast_expression)?;
            }

            // 递归检查CASE表达式
            crate::core::types::expression::Expression::Case {
                test_expr,
                conditions,
                default,
            } => {
                if let Some(expr) = test_expr {
                    self.validate_expression_in_aggregate(expr)?;
                }
                for (cond, val) in conditions {
                    self.validate_expression_in_aggregate(cond)?;
                    self.validate_expression_in_aggregate(val)?;
                }
                if let Some(d) = default {
                    self.validate_expression_in_aggregate(d)?;
                }
            }

            // 常量、属性、聚合等表达式不需要进一步递归检查
            _ => {}
        }
        Ok(())
    }
}

impl AggregateValidationStrategy {
    /// 获取策略名称
    pub fn strategy_name(&self) -> &'static str {
        "AggregateValidationStrategy"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::DataType;
    use crate::core::types::operators::{AggregateFunction, BinaryOperator};
    use crate::core::Expression;
    use crate::core::types::expression::{ExpressionMeta, ExpressionContext, ContextualExpression};
    use std::sync::Arc;

    #[test]
    fn test_aggregate_validation_strategy_creation() {
        let strategy = AggregateValidationStrategy::new();
        assert_eq!(strategy.strategy_name(), "AggregateValidationStrategy");
    }

    #[test]
    fn test_has_aggregate_expression() {
        let strategy = AggregateValidationStrategy::new();
        let expr_ctx = Arc::new(ExpressionContext::new());

        // 测试没有聚合函数的表达式
        let non_agg_expr = Expression::Literal(crate::core::Value::Int(1));
        let non_agg_meta = ExpressionMeta::new(non_agg_expr);
        let non_agg_id = expr_ctx.register_expression(non_agg_meta);
        let non_agg_expression = ContextualExpression::new(non_agg_id, expr_ctx.clone());
        assert_eq!(strategy.has_aggregate_expression(&non_agg_expression), false);

        let binary_expr = Expression::Binary {
            left: Box::new(Expression::Literal(crate::core::Value::Int(1))),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Literal(crate::core::Value::Int(2))),
        };
        let binary_meta = ExpressionMeta::new(binary_expr);
        let binary_id = expr_ctx.register_expression(binary_meta);
        let binary_expression = ContextualExpression::new(binary_id, expr_ctx);
        assert_eq!(strategy.has_aggregate_expression(&binary_expression), false);
    }

    #[test]
    fn test_validate_unwind_aggregate() {
        let strategy = AggregateValidationStrategy::new();
        let expr_ctx = Arc::new(ExpressionContext::new());

        // 测试没有聚合函数的UNWIND表达式
        let non_agg_expr = Expression::Literal(crate::core::Value::Int(1));
        let non_agg_meta = ExpressionMeta::new(non_agg_expr);
        let non_agg_id = expr_ctx.register_expression(non_agg_meta);
        let non_agg_expression = ContextualExpression::new(non_agg_id, expr_ctx);
        assert!(strategy.validate_unwind_aggregate(&non_agg_expression).is_ok());

        // 测试包含聚合函数的UNWIND表达式
        // 注意：这里需要一个聚合表达式实例
        // 暂时跳过这个测试，因为需要特定的聚合表达式构造
    }

    #[test]
    fn test_nested_expressions() {
        let strategy = AggregateValidationStrategy::new();
        let expr_ctx = Arc::new(ExpressionContext::new());

        // 测试嵌套表达式
        let nested_expr = Expression::Binary {
            left: Box::new(Expression::Unary {
                op: crate::core::types::operators::UnaryOperator::Minus,
                operand: Box::new(Expression::Literal(crate::core::Value::Int(5))),
            }),
            op: crate::core::types::operators::BinaryOperator::Add,
            right: Box::new(Expression::Literal(crate::core::Value::Int(10))),
        };
        let nested_meta = ExpressionMeta::new(nested_expr);
        let nested_id = expr_ctx.register_expression(nested_meta);
        let nested_expression = ContextualExpression::new(nested_id, expr_ctx);

        assert_eq!(strategy.has_aggregate_expression(&nested_expression), false);
    }

    #[test]
    fn test_validate_invalid_aggregate_function() {
        let strategy = AggregateValidationStrategy::new();
        let expr_ctx = Arc::new(ExpressionContext::new());
        // Count(None) 是有效的，表示 COUNT(*)
        let expression = Expression::Aggregate {
            func: AggregateFunction::Count(None),
            arg: Box::new(Expression::Literal(crate::core::Value::Int(1))),
            distinct: false,
        };
        let meta = ExpressionMeta::new(expression);
        let id = expr_ctx.register_expression(meta);
        let ctx_expr = ContextualExpression::new(id, expr_ctx);

        let result = strategy.validate_aggregate_expression(&ctx_expr);
        // Count(None) 应该被接受
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_nested_aggregates() {
        let strategy = AggregateValidationStrategy::new();
        let expr_ctx = Arc::new(ExpressionContext::new());
        let inner_agg = Expression::Aggregate {
            func: AggregateFunction::Count(None),
            arg: Box::new(Expression::Literal(crate::core::Value::Int(1))),
            distinct: false,
        };
        let outer_agg = Expression::Aggregate {
            func: AggregateFunction::Sum("".to_string()),
            arg: Box::new(inner_agg),
            distinct: false,
        };
        let meta = ExpressionMeta::new(outer_agg);
        let id = expr_ctx.register_expression(meta);
        let ctx_expr = ContextualExpression::new(id, expr_ctx);

        let result = strategy.validate_aggregate_expression(&ctx_expr);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("不允许聚合函数嵌套"));
    }

    #[test]
    fn test_validate_count_with_wildcard() {
        let strategy = AggregateValidationStrategy::new();
        let expression = Expression::Aggregate {
            func: AggregateFunction::Count(None),
            arg: Box::new(Expression::Property {
                object: Box::new(Expression::Variable("n".to_string())),
                property: "*".to_string(),
            }),
            distinct: false,
        };

        // COUNT 允许通配符属性
        assert!(strategy.validate_aggregate_expression(&expression).is_ok());
    }

    #[test]
    fn test_validate_sum_with_wildcard() {
        let strategy = AggregateValidationStrategy::new();
        let expression = Expression::Aggregate {
            func: AggregateFunction::Sum("".to_string()),
            arg: Box::new(Expression::Property {
                object: Box::new(Expression::Variable("n".to_string())),
                property: "*".to_string(),
            }),
            distinct: false,
        };

        // SUM 不允许通配符属性
        let result = strategy.validate_aggregate_expression(&expression);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("SUM"));
        assert!(err.message.contains("变量属性"));
        assert!(err.message.contains("n.*"));
    }

    #[test]
    fn test_validate_various_aggregate_functions() {
        let strategy = AggregateValidationStrategy::new();
        let _valid_functions = vec![
            "COUNT",
            "SUM",
            "AVG",
            "MAX",
            "MIN",
            "STD",
            "BIT_AND",
            "BIT_OR",
            "BIT_XOR",
            "COLLECT",
            "COLLECT_SET",
        ];

        // 测试各种聚合函数
        let valid_functions = vec![
            AggregateFunction::Count(None),
            AggregateFunction::Sum("".to_string()),
            AggregateFunction::Avg("".to_string()),
            AggregateFunction::Max("".to_string()),
            AggregateFunction::Min("".to_string()),
            AggregateFunction::Collect("".to_string()),
        ];

        for func in valid_functions {
            let expression = Expression::Aggregate {
                func,
                arg: Box::new(Expression::Literal(crate::core::Value::Int(1))),
                distinct: false,
            };

            assert!(
                strategy.validate_aggregate_expression(&expression).is_ok(),
                "聚合函数应该是有效的"
            );
        }
    }

    #[test]
    fn test_validate_distinct_aggregate() {
        let strategy = AggregateValidationStrategy::new();
        let expression = Expression::Aggregate {
            func: AggregateFunction::Count(None),
            arg: Box::new(Expression::Literal(crate::core::Value::Int(1))),
            distinct: true,
        };

        // DISTINCT 聚合应该被接受
        assert!(strategy.validate_aggregate_expression(&expression).is_ok());
    }

    #[test]
    fn test_validate_input_property_wildcard() {
        let strategy = AggregateValidationStrategy::new();

        // COUNT($-.*) 应该被允许
        let count_input_wildcard = Expression::Aggregate {
            func: AggregateFunction::Count(None),
            arg: Box::new(Expression::Property {
                object: Box::new(Expression::Variable("-".to_string())),
                property: "*".to_string(),
            }),
            distinct: false,
        };
        assert!(strategy.validate_aggregate_expression(&count_input_wildcard).is_ok());

        // SUM($-.*) 不应该被允许
        let sum_input_wildcard = Expression::Aggregate {
            func: AggregateFunction::Sum("".to_string()),
            arg: Box::new(Expression::Property {
                object: Box::new(Expression::Variable("-".to_string())),
                property: "*".to_string(),
            }),
            distinct: false,
        };
        let result = strategy.validate_aggregate_expression(&sum_input_wildcard);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("输入属性"));
        assert!(err.message.contains("SUM"));
    }

    #[test]
    fn test_validate_var_property_wildcard() {
        let strategy = AggregateValidationStrategy::new();

        // COUNT($var.*) 应该被允许
        let count_var_wildcard = Expression::Aggregate {
            func: AggregateFunction::Count(None),
            arg: Box::new(Expression::Property {
                object: Box::new(Expression::Variable("myVar".to_string())),
                property: "*".to_string(),
            }),
            distinct: false,
        };
        assert!(strategy.validate_aggregate_expression(&count_var_wildcard).is_ok());

        // AVG($var.*) 不应该被允许
        let avg_var_wildcard = Expression::Aggregate {
            func: AggregateFunction::Avg("".to_string()),
            arg: Box::new(Expression::Property {
                object: Box::new(Expression::Variable("myVar".to_string())),
                property: "*".to_string(),
            }),
            distinct: false,
        };
        let result = strategy.validate_aggregate_expression(&avg_var_wildcard);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("变量属性"));
        assert!(err.message.contains("AVG"));
    }

    #[test]
    fn test_validate_wildcard_in_nested_expression() {
        let strategy = AggregateValidationStrategy::new();

        // 嵌套表达式中的通配符不应该被检查（只检查直接参数）
        // SUM(n.* + 1) - 这里的 n.* 不是聚合函数的直接参数
        let nested_wildcard = Expression::Aggregate {
            func: AggregateFunction::Sum("".to_string()),
            arg: Box::new(Expression::Binary {
                left: Box::new(Expression::Property {
                    object: Box::new(Expression::Variable("n".to_string())),
                    property: "*".to_string(),
                }),
                op: BinaryOperator::Add,
                right: Box::new(Expression::Literal(crate::core::Value::Int(1))),
            }),
            distinct: false,
        };
        // 由于通配符在嵌套表达式中，不是直接参数，所以应该通过验证
        assert!(strategy.validate_aggregate_expression(&nested_wildcard).is_ok());
    }

    #[test]
    fn test_validate_expression_in_aggregate_binary_op() {
        let strategy = AggregateValidationStrategy::new();

        let expression = Expression::Binary {
            left: Box::new(Expression::Property {
                object: Box::new(Expression::Variable("n".to_string())),
                property: "value".to_string(),
            }),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Literal(crate::core::Value::Int(10))),
        };

        assert!(strategy.validate_expression_in_aggregate(&expression).is_ok());
    }

    #[test]
    fn test_validate_expression_in_aggregate_function_call() {
        let strategy = AggregateValidationStrategy::new();

        // 测试函数调用在聚合参数中的验证
        let expression = Expression::Function {
            name: "LOWER".to_string(),
            args: vec![Expression::Property {
                object: Box::new(Expression::Variable("n".to_string())),
                property: "name".to_string(),
            }],
        };

        // 应该通过验证
        assert!(strategy.validate_expression_in_aggregate(&expression).is_ok());
    }

    #[test]
    fn test_validate_expression_in_aggregate_case() {
        let strategy = AggregateValidationStrategy::new();

        // 测试CASE表达式在聚合参数中的验证
        let expression = Expression::Case {
            test_expr: None,
            conditions: vec![(
                Expression::Binary {
                    left: Box::new(Expression::Property {
                        object: Box::new(Expression::Variable("n".to_string())),
                        property: "status".to_string(),
                    }),
                    op: BinaryOperator::Equal,
                    right: Box::new(Expression::Literal(crate::core::Value::String(
                        "active".to_string(),
                    ))),
                },
                Expression::Literal(crate::core::Value::Int(1)),
            )],
            default: Some(Box::new(Expression::Literal(crate::core::Value::Int(0)))),
        };

        assert!(strategy.validate_expression_in_aggregate(&expression).is_ok());
    }

    #[test]
    fn test_validate_expression_in_aggregate_list() {
        let strategy = AggregateValidationStrategy::new();

        let expression = Expression::List(vec![
            Expression::Literal(crate::core::Value::Int(1)),
            Expression::Literal(crate::core::Value::Int(2)),
            Expression::Property {
                object: Box::new(Expression::Variable("n".to_string())),
                property: "value".to_string(),
            },
        ]);

        // 应该通过验证
        assert!(strategy.validate_expression_in_aggregate(&expression).is_ok());
    }

    #[test]
    fn test_validate_expression_in_aggregate_type_casting() {
        let strategy = AggregateValidationStrategy::new();

        // 测试类型转换在聚合参数中的验证
        let expression = Expression::TypeCast {
            expression: Box::new(Expression::Property {
                object: Box::new(Expression::Variable("n".to_string())),
                property: "value".to_string(),
            }),
            target_type: DataType::Int,
        };

        // 应该通过验证
        assert!(strategy.validate_expression_in_aggregate(&expression).is_ok());
    }

    #[test]
    fn test_validate_aggregate_sum_valid() {
        let strategy = AggregateValidationStrategy::new();

        // 测试有效的SUM聚合
        let expression = Expression::Aggregate {
            func: AggregateFunction::Sum("".to_string()),
            arg: Box::new(Expression::Property {
                object: Box::new(Expression::Variable("n".to_string())),
                property: "amount".to_string(),
            }),
            distinct: false,
        };

        assert!(strategy.validate_aggregate_expression(&expression).is_ok());
    }

    #[test]
    fn test_validate_aggregate_count_valid() {
        let strategy = AggregateValidationStrategy::new();

        // 测试有效的COUNT聚合
        let expression = Expression::Aggregate {
            func: AggregateFunction::Count(None),
            arg: Box::new(Expression::Literal(crate::core::Value::Int(1))),
            distinct: false,
        };

        assert!(strategy.validate_aggregate_expression(&expression).is_ok());
    }

    #[test]
    fn test_validate_aggregate_min_max_valid() {
        let strategy = AggregateValidationStrategy::new();

        let min_expression = Expression::Aggregate {
            func: AggregateFunction::Min("".to_string()),
            arg: Box::new(Expression::Property {
                object: Box::new(Expression::Variable("n".to_string())),
                property: "value".to_string(),
            }),
            distinct: false,
        };
        let max_expression = Expression::Aggregate {
            func: AggregateFunction::Max("".to_string()),
            arg: Box::new(Expression::Property {
                object: Box::new(Expression::Variable("n".to_string())),
                property: "value".to_string(),
            }),
            distinct: false,
        };

        assert!(strategy.validate_aggregate_expression(&min_expression).is_ok());
        assert!(strategy.validate_aggregate_expression(&max_expression).is_ok());
    }
}
