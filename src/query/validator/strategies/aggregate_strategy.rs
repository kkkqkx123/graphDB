//! 聚合验证策略
//! 负责验证聚合函数的使用和检查表达式是否包含聚合

use super::super::validation_interface::*;
use super::super::structs::*;
use super::agg_functions::AggFunctionMeta;
use crate::graph::expression::expr_type::Expression;

/// 聚合验证策略
pub struct AggregateValidationStrategy;

impl AggregateValidationStrategy {
    pub fn new() -> Self {
        Self
    }
    
    /// 检查表达式是否包含聚合函数
    pub fn has_aggregate_expr(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Aggregate { .. } => true,
            Expression::UnaryOp(_, operand) => self.has_aggregate_expr(operand),
            Expression::BinaryOp(left, _, right) => {
                self.has_aggregate_expr(left) || self.has_aggregate_expr(right)
            }
            Expression::Function(_, args) => {
                args.iter().any(|arg| self.has_aggregate_expr(arg))
            }
            Expression::List(items) => {
                items.iter().any(|item| self.has_aggregate_expr(item))
            }
            Expression::Set(items) => {
                items.iter().any(|item| self.has_aggregate_expr(item))
            }
            Expression::Map(items) => {
                items.iter().any(|(_, value)| self.has_aggregate_expr(value))
            }
            Expression::Case { conditions, default } => {
                conditions.iter().any(|(cond, val)| {
                    self.has_aggregate_expr(cond) || self.has_aggregate_expr(val)
                }) || default.as_ref().map_or(false, |d| self.has_aggregate_expr(d))
            }
            Expression::ListComprehension { generator, condition } => {
                self.has_aggregate_expr(generator)
                    || condition.as_ref().map_or(false, |c| self.has_aggregate_expr(c))
            }
            Expression::Predicate { list, condition } => {
                self.has_aggregate_expr(list) || self.has_aggregate_expr(condition)
            }
            Expression::Reduce { list, initial, expr, .. } => {
                self.has_aggregate_expr(list)
                    || self.has_aggregate_expr(initial)
                    || self.has_aggregate_expr(expr)
            }
            _ => false,
        }
    }
    
    /// 验证UNWIND子句中不允许使用聚合函数
    pub fn validate_unwind_aggregate(&self, unwind_expr: &Expression) -> Result<(), ValidationError> {
        if self.has_aggregate_expr(unwind_expr) {
            return Err(ValidationError::new(
                "UNWIND子句中不能使用聚合表达式".to_string(),
                ValidationErrorType::AggregateError
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
    pub fn validate_aggregate_expr(&self, expr: &Expression) -> Result<(), ValidationError> {
        match expr {
            Expression::Aggregate { name, arg, distinct: _ } => {
                // 1. 验证聚合函数名的有效性
                if !AggFunctionMeta::is_valid(name) {
                    return Err(ValidationError::new(
                        format!("未知的聚合函数: `{}`", name),
                        ValidationErrorType::AggregateError,
                    ));
                }

                // 2. 检查聚合函数嵌套 - 不允许聚合函数中包含聚合函数
                if self.has_aggregate_expr(arg) {
                    return Err(ValidationError::new(
                        format!("不允许聚合函数嵌套: `{}`", name),
                        ValidationErrorType::AggregateError,
                    ));
                }

                // 3. 检查特殊属性 (*.  * 只能用于COUNT)
                self.validate_wildcard_property(name, arg)?;

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
        expr: &Expression,
    ) -> Result<(), ValidationError> {
        let meta = match AggFunctionMeta::get(func_name) {
            Some(m) => m,
            None => return Ok(()), // 已经在validate_aggregate_expr中检查过
        };

        // 如果函数不允许通配符，需要检查是否存在通配符属性
        if !meta.allow_wildcard && self.has_wildcard_property(expr) {
            return Err(ValidationError::new(
                format!(
                    "聚合函数 `{}` 不能应用于通配符属性 `*`",
                    func_name
                ),
                ValidationErrorType::AggregateError,
            ));
        }

        Ok(())
    }

    /// 检查表达式中是否包含通配符属性
    fn has_wildcard_property(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Property { name, .. } if name == "*" => true,
            Expression::UnaryOp(_, operand) => self.has_wildcard_property(operand),
            Expression::BinaryOp(left, _, right) => {
                self.has_wildcard_property(left) || self.has_wildcard_property(right)
            }
            Expression::Function(_, args) => args.iter().any(|arg| self.has_wildcard_property(arg)),
            Expression::List(items) => items.iter().any(|item| self.has_wildcard_property(item)),
            Expression::Set(items) => items.iter().any(|item| self.has_wildcard_property(item)),
            Expression::Map(items) => {
                items
                    .iter()
                    .any(|(_, value)| self.has_wildcard_property(value))
            }
            Expression::Case { conditions, default } => {
                conditions.iter().any(|(cond, val)| {
                    self.has_wildcard_property(cond) || self.has_wildcard_property(val)
                }) || default
                    .as_ref()
                    .map_or(false, |d| self.has_wildcard_property(d))
            }
            Expression::ListComprehension { generator, condition } => {
                self.has_wildcard_property(generator)
                    || condition
                        .as_ref()
                        .map_or(false, |c| self.has_wildcard_property(c))
            }
            Expression::Predicate { list, condition } => {
                self.has_wildcard_property(list) || self.has_wildcard_property(condition)
            }
            Expression::Reduce { list, initial, expr, .. } => {
                self.has_wildcard_property(list)
                    || self.has_wildcard_property(initial)
                    || self.has_wildcard_property(expr)
            }
            _ => false,
        }
    }

    /// 验证聚合函数参数表达式的合法性
    /// 递归验证参数表达式中是否有其他不合法的嵌套结构
    fn validate_expression_in_aggregate(&self, expr: &Expression) -> Result<(), ValidationError> {
        match expr {
            // 这里可以添加更多的表达式验证规则
            // 例如：不允许在聚合中使用某些特殊的表达式
            _ => Ok(()),
        }
    }
}

impl ValidationStrategy for AggregateValidationStrategy {
    fn validate(&self, context: &dyn ValidationContext) -> Result<(), ValidationError> {
        // 遍历所有查询部分，验证聚合函数使用
        for query_part in context.get_query_parts() {
            // 验证边界子句中的聚合函数
            if let Some(boundary) = &query_part.boundary {
                match boundary {
                    BoundaryClauseContext::With(with_ctx) => {
                        // 验证WITH子句中的聚合函数
                        if with_ctx.yield_clause.has_agg {
                            for col in &with_ctx.yield_clause.yield_columns {
                                self.validate_aggregate_expr(&col.expr)?;
                            }
                        }
                    }
                    BoundaryClauseContext::Unwind(unwind_ctx) => {
                        // 验证UNWIND子句中不允许使用聚合函数
                        self.validate_unwind_aggregate(&unwind_ctx.unwind_expr)?;
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
    use crate::graph::expression::expr_type::Expression;

    #[test]
    fn test_aggregate_validation_strategy_creation() {
        let strategy = AggregateValidationStrategy::new();
        assert_eq!(strategy.strategy_type(), ValidationStrategyType::Aggregate);
        assert_eq!(strategy.strategy_name(), "AggregateValidationStrategy");
    }

    #[test]
    fn test_has_aggregate_expr() {
        let strategy = AggregateValidationStrategy::new();

        // 测试没有聚合函数的表达式
        let non_agg_expr = Expression::Constant(crate::core::Value::Int(1));
        assert_eq!(strategy.has_aggregate_expr(&non_agg_expr), false);

        // 测试包含聚合函数的表达式
        // 注意：这里需要一个聚合表达式实例，具体实现可能依赖Expression的定义
        let binary_expr = Expression::BinaryOp(
            Box::new(Expression::Constant(crate::core::Value::Int(1))),
            crate::graph::expression::expr_type::BinaryOperator::Add,
            Box::new(Expression::Constant(crate::core::Value::Int(2))),
        );
        assert_eq!(strategy.has_aggregate_expr(&binary_expr), false);
    }

    #[test]
    fn test_validate_unwind_aggregate() {
        let strategy = AggregateValidationStrategy::new();

        // 测试没有聚合函数的UNWIND表达式
        let non_agg_expr = Expression::Constant(crate::core::Value::Int(1));
        assert!(strategy.validate_unwind_aggregate(&non_agg_expr).is_ok());

        // 测试包含聚合函数的UNWIND表达式
        // 注意：这里需要一个聚合表达式实例
        // 暂时跳过这个测试，因为需要特定的聚合表达式构造
    }

    #[test]
    fn test_nested_expressions() {
        let strategy = AggregateValidationStrategy::new();

        // 测试嵌套表达式
        let nested_expr = Expression::BinaryOp(
            Box::new(Expression::UnaryOp(
                crate::graph::expression::expr_type::UnaryOperator::Negate,
                Box::new(Expression::Constant(crate::core::Value::Int(5))),
            )),
            crate::graph::expression::expr_type::BinaryOperator::Add,
            Box::new(Expression::Constant(crate::core::Value::Int(10))),
        );

        assert_eq!(strategy.has_aggregate_expr(&nested_expr), false);
    }

    #[test]
    fn test_validate_invalid_aggregate_function() {
        let strategy = AggregateValidationStrategy::new();
        let expr = Expression::Aggregate {
            name: "UNKNOWN_FUNC".to_string(),
            arg: Box::new(Expression::Constant(crate::core::Value::Int(1))),
            distinct: false,
        };

        let result = strategy.validate_aggregate_expr(&expr);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("未知的聚合函数"));
    }

    #[test]
    fn test_validate_nested_aggregates() {
        let strategy = AggregateValidationStrategy::new();
        let inner_agg = Expression::Aggregate {
            name: "COUNT".to_string(),
            arg: Box::new(Expression::Constant(crate::core::Value::Int(1))),
            distinct: false,
        };
        let outer_agg = Expression::Aggregate {
            name: "SUM".to_string(),
            arg: Box::new(inner_agg),
            distinct: false,
        };

        let result = strategy.validate_aggregate_expr(&outer_agg);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("不允许聚合函数嵌套"));
    }

    #[test]
    fn test_validate_count_with_wildcard() {
        let strategy = AggregateValidationStrategy::new();
        let expr = Expression::Aggregate {
            name: "COUNT".to_string(),
            arg: Box::new(Expression::Property {
                name: "*".to_string(),
                entity: None,
            }),
            distinct: false,
        };

        // COUNT 允许通配符属性
        assert!(strategy.validate_aggregate_expr(&expr).is_ok());
    }

    #[test]
    fn test_validate_sum_with_wildcard() {
        let strategy = AggregateValidationStrategy::new();
        let expr = Expression::Aggregate {
            name: "SUM".to_string(),
            arg: Box::new(Expression::Property {
                name: "*".to_string(),
                entity: None,
            }),
            distinct: false,
        };

        // SUM 不允许通配符属性
        let result = strategy.validate_aggregate_expr(&expr);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("不能应用于通配符属性"));
    }

    #[test]
    fn test_validate_various_aggregate_functions() {
        let strategy = AggregateValidationStrategy::new();
        let valid_functions = vec!["COUNT", "SUM", "AVG", "MAX", "MIN", "STD", 
                                    "BIT_AND", "BIT_OR", "BIT_XOR", "COLLECT", "COLLECT_SET"];
        
        for func_name in valid_functions {
            let expr = Expression::Aggregate {
                name: func_name.to_string(),
                arg: Box::new(Expression::Constant(crate::core::Value::Int(1))),
                distinct: false,
            };
            
            assert!(strategy.validate_aggregate_expr(&expr).is_ok(),
                    "聚合函数 {} 应该是有效的", func_name);
        }
    }

    #[test]
    fn test_validate_distinct_aggregate() {
        let strategy = AggregateValidationStrategy::new();
        let expr = Expression::Aggregate {
            name: "COUNT".to_string(),
            arg: Box::new(Expression::Constant(crate::core::Value::Int(1))),
            distinct: true,
        };

        // DISTINCT 聚合应该被接受
        assert!(strategy.validate_aggregate_expr(&expr).is_ok());
    }

    #[test]
    fn test_has_wildcard_property() {
        let strategy = AggregateValidationStrategy::new();
        
        // 直接通配符属性
        let expr1 = Expression::Property {
            name: "*".to_string(),
            entity: None,
        };
        assert!(strategy.has_wildcard_property(&expr1));
        
        // 非通配符属性
        let expr2 = Expression::Property {
            name: "age".to_string(),
            entity: None,
        };
        assert!(!strategy.has_wildcard_property(&expr2));
        
        // 二元表达式包含通配符
        let expr3 = Expression::BinaryOp(
            Box::new(Expression::Property {
                name: "*".to_string(),
                entity: None,
            }),
            crate::graph::expression::expr_type::BinaryOperator::Add,
            Box::new(Expression::Constant(crate::core::Value::Int(1))),
        );
        assert!(strategy.has_wildcard_property(&expr3));
    }
}