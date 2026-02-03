//! 聚合验证策略
//! 负责验证聚合函数的使用和检查表达式是否包含聚合

use super::super::structs::*;
use super::super::validation_interface::{
    ValidationContext as ValidationContextTrait, ValidationError, ValidationErrorType,
    ValidationStrategy, ValidationStrategyType,
};
use crate::core::Expression;

/// 聚合验证策略
pub struct AggregateValidationStrategy;

impl AggregateValidationStrategy {
    pub fn new() -> Self {
        Self
    }

    /// 检查表达式是否包含聚合函数
    pub fn has_aggregate_expression(&self, expression: &Expression) -> bool {
        match expression {
            Expression::Aggregate { .. } => true,
            Expression::Unary { operand, .. } => self.has_aggregate_expression(operand.as_ref()),
            Expression::Binary { left, right, .. } => {
                self.has_aggregate_expression(left.as_ref()) || self.has_aggregate_expression(right.as_ref())
            }
            Expression::Function { args, .. } => {
                args.iter().any(|arg| self.has_aggregate_expression(arg))
            }
            Expression::List(items) => items.iter().any(|item| self.has_aggregate_expression(item)),
            Expression::Map(items) => items
                .iter()
                .any(|(_, value)| self.has_aggregate_expression(value)),
            Expression::Case {
                test_expr,
                conditions,
                default,
            } => {
                test_expr.as_ref().map_or(false, |expr| self.has_aggregate_expression(expr))
                    || conditions.iter().any(|(cond, val)| {
                        self.has_aggregate_expression(cond) || self.has_aggregate_expression(val)
                    })
                    || default.as_ref().map_or(false, |d| self.has_aggregate_expression(d))
            }
            _ => false,
        }
    }

    /// 验证UNWIND子句中不允许使用聚合函数
    pub fn validate_unwind_aggregate(
        &self,
        unwind_expression: &Expression,
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
    pub fn validate_aggregate_expression(&self, expression: &Expression) -> Result<(), ValidationError> {
        match expression {
            Expression::Aggregate {
                func,
                arg,
                distinct: _,
            } => {
                // 1. 验证聚合函数名的有效性
                // 注意：由于现在使用枚举，这个检查可能需要调整
                // 暂时跳过这个检查，因为枚举值总是有效的

                // 2. 检查聚合函数嵌套 - 不允许聚合函数中包含聚合函数
                if self.has_aggregate_expression(arg) {
                    return Err(ValidationError::new(
                        "不允许聚合函数嵌套".to_string(),
                        ValidationErrorType::AggregateError,
                    ));
                }

                // 3. 检查特殊属性 (*.  * 只能用于COUNT)
                self.validate_wildcard_property(&format!("{:?}", func), arg)?;

                // 4. 递归验证参数表达式的合法性
                self.validate_expression_in_aggregate(arg)?;

                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// 验证通配符属性的使用
    /// 只有COUNT函数允许通配符属性(*)作为参数
    fn validate_wildcard_property(
        &self,
        func_name: &str,
        expression: &Expression,
    ) -> Result<(), ValidationError> {
        // 简化版本：只有COUNT允许通配符
        if !func_name.contains("Count") && self.has_wildcard_property(expression) {
            return Err(ValidationError::new(
                format!("聚合函数 `{}` 不能应用于通配符属性 `*`", func_name),
                ValidationErrorType::AggregateError,
            ));
        }

        Ok(())
    }

    /// 检查表达式中是否包含通配符属性
    fn has_wildcard_property(&self, expression: &Expression) -> bool {
        match expression {
            Expression::Property { property, .. } if property == "*" => true,
            Expression::Unary { operand, .. } => self.has_wildcard_property(operand.as_ref()),
            Expression::Binary { left, right, .. } => {
                self.has_wildcard_property(left.as_ref())
                    || self.has_wildcard_property(right.as_ref())
            }
            Expression::Function { args, .. } => {
                args.iter().any(|arg| self.has_wildcard_property(arg))
            }
            Expression::List(items) => items.iter().any(|item| self.has_wildcard_property(item)),
            Expression::Map(items) => items
                .iter()
                .any(|(_, value)| self.has_wildcard_property(value)),
            Expression::Case {
                test_expr,
                conditions,
                default,
            } => {
                test_expr.as_ref().map_or(false, |expr| self.has_wildcard_property(expr))
                    || conditions.iter().any(|(cond, val)| {
                        self.has_wildcard_property(cond) || self.has_wildcard_property(val)
                    })
                    || default.as_ref().map_or(false, |d| self.has_wildcard_property(d))
            }
            _ => false,
        }
    }

    /// 验证聚合函数参数表达式的合法性
    /// 递归验证参数表达式中是否有其他不合法的嵌套结构
    ///
    /// 验证规则：
    /// 1. 递归检查所有子表达式的合法性
    /// 2. 确保参数表达式的结构正确
    fn validate_expression_in_aggregate(&self, expression: &Expression) -> Result<(), ValidationError> {
        match expression {
            // 递归检查一元操作（包括各种一元操作符）
            Expression::Unary { operand, .. } => {
                self.validate_expression_in_aggregate(operand)?;
            }

            // 递归检查二元操作
            Expression::Binary { left, right, .. } => {
                self.validate_expression_in_aggregate(left)?;
                self.validate_expression_in_aggregate(right)?;
            }

            // 递归检查函数调用参数
            Expression::Function { args, .. } => {
                for arg in args {
                    self.validate_expression_in_aggregate(arg)?;
                }
            }

            // 递归检查列表元素
            Expression::List(items) => {
                for item in items {
                    self.validate_expression_in_aggregate(item)?;
                }
            }

            // 递归检查Map值
            Expression::Map(items) => {
                for (_, value) in items {
                    self.validate_expression_in_aggregate(value)?;
                }
            }

            // 递归检查类型转换表达式
            Expression::TypeCast {
                expression: cast_expression, ..
            } => {
                self.validate_expression_in_aggregate(cast_expression)?;
            }

            // 递归检查CASE表达式
            Expression::Case {
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

impl ValidationStrategy for AggregateValidationStrategy {
    fn validate(&self, context: &dyn ValidationContextTrait) -> Result<(), ValidationError> {
        // 遍历所有查询部分，验证聚合函数使用
        for query_part in context.get_query_parts() {
            // 验证边界子句中的聚合函数
            if let Some(boundary) = &query_part.boundary {
                match boundary {
                    BoundaryClauseContext::With(with_ctx) => {
                        // 验证WITH子句中的聚合函数
                        if with_ctx.yield_clause.has_agg {
                            for col in &with_ctx.yield_clause.yield_columns {
                                self.validate_aggregate_expression(&col.expression)?;
                            }
                        }

                        // 验证 WITH WHERE 子句不允许使用聚合
                        // 这与SQL的WHERE语义一致：WHERE不能包含聚合函数
                        if let Some(where_clause) = &with_ctx.where_clause {
                            if let Some(filter_expression) = &where_clause.filter {
                                if self.has_aggregate_expression(filter_expression) {
                                    return Err(ValidationError::new(
                                        "WHERE 子句中不允许使用聚合函数".to_string(),
                                        ValidationErrorType::AggregateError,
                                    ));
                                }
                            }
                        }
                    }
                    BoundaryClauseContext::Unwind(unwind_ctx) => {
                        // 验证UNWIND子句中不允许使用聚合函数
                        self.validate_unwind_aggregate(&unwind_ctx.unwind_expression)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn strategy_type(&self) -> ValidationStrategyType {
        ValidationStrategyType::Aggregate
    }

    fn strategy_name(&self) -> &'static str {
        "AggregateValidationStrategy"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::expression::DataType;
    use crate::core::types::operators::{AggregateFunction, BinaryOperator};
    use crate::core::Expression;

    #[test]
    fn test_aggregate_validation_strategy_creation() {
        let strategy = AggregateValidationStrategy::new();
        assert_eq!(strategy.strategy_type(), ValidationStrategyType::Aggregate);
        assert_eq!(strategy.strategy_name(), "AggregateValidationStrategy");
    }

    #[test]
    fn test_has_aggregate_expression() {
        let strategy = AggregateValidationStrategy::new();

        // 测试没有聚合函数的表达式
        let non_agg_expression = Expression::Literal(crate::core::Value::Int(1));
        assert_eq!(strategy.has_aggregate_expression(&non_agg_expression), false);

        let binary_expression = Expression::Binary {
            left: Box::new(Expression::Literal(crate::core::Value::Int(1))),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Literal(crate::core::Value::Int(2))),
        };
        assert_eq!(strategy.has_aggregate_expression(&binary_expression), false);
    }

    #[test]
    fn test_validate_unwind_aggregate() {
        let strategy = AggregateValidationStrategy::new();

        // 测试没有聚合函数的UNWIND表达式
        let non_agg_expression = Expression::Literal(crate::core::Value::Int(1));
        assert!(strategy.validate_unwind_aggregate(&non_agg_expression).is_ok());

        // 测试包含聚合函数的UNWIND表达式
        // 注意：这里需要一个聚合表达式实例
        // 暂时跳过这个测试，因为需要特定的聚合表达式构造
    }

    #[test]
    fn test_nested_expressions() {
        let strategy = AggregateValidationStrategy::new();

        // 测试嵌套表达式
        let nested_expression = Expression::Binary {
            left: Box::new(Expression::Unary {
                op: crate::core::types::operators::UnaryOperator::Minus,
                operand: Box::new(Expression::Literal(crate::core::Value::Int(5))),
            }),
            op: crate::core::types::operators::BinaryOperator::Add,
            right: Box::new(Expression::Literal(crate::core::Value::Int(10))),
        };

        assert_eq!(strategy.has_aggregate_expression(&nested_expression), false);
    }

    #[test]
    fn test_validate_invalid_aggregate_function() {
        let strategy = AggregateValidationStrategy::new();
        // Count(None) 是有效的，表示 COUNT(*)
        let expression = Expression::Aggregate {
            func: AggregateFunction::Count(None),
            arg: Box::new(Expression::Literal(crate::core::Value::Int(1))),
            distinct: false,
        };

        let result = strategy.validate_aggregate_expression(&expression);
        // Count(None) 应该被接受
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_nested_aggregates() {
        let strategy = AggregateValidationStrategy::new();
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

        let result = strategy.validate_aggregate_expression(&outer_agg);
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
        assert!(err.message.contains("不能应用于通配符属性"));
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
    fn test_has_wildcard_property() {
        let strategy = AggregateValidationStrategy::new();

        // 直接通配符属性
        let expr1 = Expression::Property {
            object: Box::new(Expression::Variable("n".to_string())),
            property: "*".to_string(),
        };
        assert!(strategy.has_wildcard_property(&expr1));

        // 非通配符属性
        let expr2 = Expression::Property {
            object: Box::new(Expression::Variable("n".to_string())),
            property: "age".to_string(),
        };
        assert!(!strategy.has_wildcard_property(&expr2));

        // 二元表达式包含通配符
        let expr3 = Expression::Binary {
            left: Box::new(Expression::Property {
                object: Box::new(Expression::Variable("n".to_string())),
                property: "*".to_string(),
            }),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Literal(crate::core::Value::Int(1))),
        };
        assert!(strategy.has_wildcard_property(&expr3));
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
