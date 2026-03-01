//! 分页验证策略
//! 负责验证SKIP、LIMIT和分页相关的表达式

use crate::core::types::expression::contextual::ContextualExpression;
use crate::core::YieldColumn;
use crate::core::error::{ValidationError, ValidationErrorType};
use crate::core::types::expression::utils::is_evaluable;
use crate::query::validator::structs::{MatchStepRange, PaginationContext, OrderByClauseContext};

/// 分页验证策略
pub struct PaginationValidationStrategy;

impl PaginationValidationStrategy {
    pub fn new() -> Self {
        Self
    }

    /// 验证分页参数的有效性
    pub fn validate_pagination(
        &self,
        skip_expression: Option<&ContextualExpression>,
        limit_expression: Option<&ContextualExpression>,
        context: &PaginationContext,
    ) -> Result<(), ValidationError> {
        // 验证分页参数的有效性
        if context.skip < 0 {
            return Err(ValidationError::new(
                "SKIP不能为负数".to_string(),
                ValidationErrorType::PaginationError,
            ));
        }
        if context.limit < 0 {
            return Err(ValidationError::new(
                "LIMIT不能为负数".to_string(),
                ValidationErrorType::PaginationError,
            ));
        }

        // 验证 SKIP 表达式
        if let Some(expr) = skip_expression {
            self.validate_pagination_expression(expr, "SKIP")?;
        }

        // 验证 LIMIT 表达式
        if let Some(expr) = limit_expression {
            self.validate_pagination_expression(expr, "LIMIT")?;
        }

        Ok(())
    }

    /// 验证分页表达式
    fn validate_pagination_expression(
        &self,
        expression: &ContextualExpression,
        clause_name: &str,
    ) -> Result<(), ValidationError> {
        if let Some(expr) = expression.expression() {
            self.validate_pagination_expression_internal(&expr, clause_name)
        } else {
            Err(ValidationError::new(
                format!("{}表达式无效", clause_name),
                ValidationErrorType::PaginationError,
            ))
        }
    }

    /// 内部方法：验证分页表达式
    fn validate_pagination_expression_internal(
        &self,
        expression: &crate::core::types::expression::Expression,
        clause_name: &str,
    ) -> Result<(), ValidationError> {
        if !is_evaluable(expression) {
            return Err(ValidationError::new(
                format!("{}表达式必须是可立即计算的常量表达式", clause_name),
                ValidationErrorType::PaginationError,
            ));
        }

        match expression {
            crate::core::types::expression::Expression::Literal(crate::core::Value::Int(n)) => {
                if *n >= 0 {
                    Ok(())
                } else {
                    Err(ValidationError::new(
                        format!("{}表达式必须是非负整数", clause_name),
                        ValidationErrorType::PaginationError,
                    ))
                }
            }
            crate::core::types::expression::Expression::Literal(_) => Err(ValidationError::new(
                format!("{}表达式必须求值为整数类型", clause_name),
                ValidationErrorType::PaginationError,
            )),
            _ => {
                use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
                use crate::expression::context::DefaultExpressionContext;

                let mut context = DefaultExpressionContext::new();
                match ExpressionEvaluator::evaluate(expression, &mut context) {
                    Ok(crate::core::Value::Int(n)) => {
                        if n >= 0 {
                            Ok(())
                        } else {
                            Err(ValidationError::new(
                                format!("{}表达式必须是非负整数", clause_name),
                                ValidationErrorType::PaginationError,
                            ))
                        }
                    }
                    Ok(_) => Err(ValidationError::new(
                        format!("{}表达式必须求值为整数类型", clause_name),
                        ValidationErrorType::PaginationError,
                    )),
                    Err(e) => Err(ValidationError::new(
                        format!("{}表达式求值失败: {}", clause_name, e),
                        ValidationErrorType::PaginationError,
                    )),
                }
            }
        }
    }

    /// 验证步数范围
    pub fn validate_step_range(&self, range: &MatchStepRange) -> Result<(), ValidationError> {
        if range.min > range.max {
            return Err(ValidationError::new(
                format!(
                    "最大跳数必须大于等于最小跳数: {} vs. {}",
                    range.max, range.min
                ),
                ValidationErrorType::PaginationError,
            ));
        }
        Ok(())
    }

    /// 验证排序子句
    pub fn validate_order_by(
        &self,
        _factors: &[ContextualExpression], // 排序因子
        yield_columns: &[YieldColumn],
        context: &OrderByClauseContext,
    ) -> Result<(), ValidationError> {
        // 验证OrderBy子句
        for &(index, _) in &context.indexed_order_factors {
            if index >= yield_columns.len() {
                return Err(ValidationError::new(
                    format!("列索引{}超出范围", index),
                    ValidationErrorType::PaginationError,
                ));
            }
        }

        Ok(())
    }
}

impl PaginationValidationStrategy {
    /// 获取策略名称
    pub fn strategy_name(&self) -> &'static str {
        "PaginationValidationStrategy"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;

    #[test]
    fn test_pagination_validation_strategy_creation() {
        let strategy = PaginationValidationStrategy::new();
        assert_eq!(strategy.strategy_name(), "PaginationValidationStrategy");
    }

    #[test]
    fn test_validate_pagination() {
        let strategy = PaginationValidationStrategy::new();

        // 测试有效的分页表达式
        let skip_expression = Expression::Literal(crate::core::Value::Int(1));
        let limit_expression = Expression::Literal(crate::core::Value::Int(10));
        let pagination_ctx = PaginationContext { skip: 0, limit: 10 };

        assert!(strategy
            .validate_pagination(Some(&skip_expression), Some(&limit_expression), &pagination_ctx)
            .is_ok());

        // 测试无效的分页参数
        let invalid_pagination_ctx = PaginationContext {
            skip: -1,
            limit: 10,
        };
        assert!(strategy
            .validate_pagination(None, None, &invalid_pagination_ctx)
            .is_err());

        let invalid_pagination_ctx2 = PaginationContext { skip: 0, limit: -5 };
        assert!(strategy
            .validate_pagination(None, None, &invalid_pagination_ctx2)
            .is_err());
    }

    #[test]
    fn test_validate_step_range() {
        let strategy = PaginationValidationStrategy::new();

        // 测试有效的范围（min <= max）
        let valid_range = MatchStepRange::new(1, 3);
        assert!(strategy.validate_step_range(&valid_range).is_ok());

        // 测试无效的范围（min > max）
        let invalid_range = MatchStepRange::new(3, 1);
        assert!(strategy.validate_step_range(&invalid_range).is_err());
    }

    #[test]
    fn test_validate_order_by() {
        let strategy = PaginationValidationStrategy::new();

        // 创建测试数据
        let yield_columns = vec![
            YieldColumn::new(
                Expression::Literal(crate::core::Value::Int(1)),
                "col1".to_string(),
            ),
            YieldColumn::new(
                Expression::Literal(crate::core::Value::Int(2)),
                "col2".to_string(),
            ),
        ];

        let valid_context = OrderByClauseContext {
            indexed_order_factors: vec![(0, crate::core::types::OrderDirection::Asc), (1, crate::core::types::OrderDirection::Desc)],
        };

        assert!(strategy
            .validate_order_by(&[], &yield_columns, &valid_context)
            .is_ok());

        // 测试无效的索引
        let invalid_context = OrderByClauseContext {
            indexed_order_factors: vec![(5, crate::core::types::OrderDirection::Asc)], // 索引超出范围
        };

        assert!(strategy
            .validate_order_by(&[], &yield_columns, &invalid_context)
            .is_err());
    }

    #[test]
    fn test_pagination_expr_validation() {
        let strategy = PaginationValidationStrategy::new();

        // 测试有效的整数表达式
        let int_expression = Expression::Literal(crate::core::Value::Int(10));
        assert!(strategy
            .validate_pagination_expression(&int_expression, "LIMIT")
            .is_ok());

        let string_expression = Expression::Literal(crate::core::Value::String("invalid".to_string()));
        assert!(strategy
            .validate_pagination_expression(&string_expression, "LIMIT")
            .is_err());
    }

    #[test]
    fn test_edge_cases() {
        let strategy = PaginationValidationStrategy::new();

        // 测试边界情况
        let zero_pagination = PaginationContext { skip: 0, limit: 0 };
        assert!(strategy
            .validate_pagination(None, None, &zero_pagination)
            .is_ok());

        let large_pagination = PaginationContext {
            skip: 1000000,
            limit: 1000000,
        };
        assert!(strategy
            .validate_pagination(None, None, &large_pagination)
            .is_ok());
    }
}
