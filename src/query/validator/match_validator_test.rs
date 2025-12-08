#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::validator::{Validator, ValidateContext};
    use crate::graph::expression::expr_type::{Expression, ConstantValue};
    use std::collections::HashMap;

    #[test]
    fn test_match_validator_creation() {
        let context = ValidateContext::default();
        let validator = MatchValidator::new(context);

        assert_eq!(validator.query_parts.len(), 0);
    }

    #[test]
    fn test_basic_validation() {
        let context = ValidateContext::default();
        let mut validator = MatchValidator::new(context);

        // 简单验证应该成功
        assert!(validator.validate().is_ok());
    }

    #[test]
    fn test_validate_pagination() {
        let context = ValidateContext::default();
        let mut validator = MatchValidator::new(context);

        // 测试有效的分页表达式
        let skip_expr = Expression::Constant(ConstantValue::Int(1));
        let limit_expr = Expression::Constant(ConstantValue::Int(10));
        let pagination_ctx = PaginationContext { skip: 0, limit: 10 };

        // 这里的测试需要根据实际的验证逻辑来设计
        assert!(validator.validate_pagination(Some(&skip_expr), Some(&limit_expr), &pagination_ctx).is_ok());
    }

    #[test]
    fn test_validate_aliases() {
        let context = ValidateContext::default();
        let mut validator = MatchValidator::new(context);

        // 创建一个别名映射
        let mut aliases = HashMap::new();
        aliases.insert("n".to_string(), AliasType::Node);
        aliases.insert("e".to_string(), AliasType::Edge);

        // 测试有效的别名引用
        let expr = Expression::Variable("n".to_string());
        assert!(validator.validate_aliases(&[expr], &aliases).is_ok());

        // 测试无效的别名引用
        let invalid_expr = Expression::Variable("invalid".to_string());
        assert!(validator.validate_aliases(&[invalid_expr], &aliases).is_err());
    }

    #[test]
    fn test_has_aggregate_expr() {
        let context = ValidateContext::default();
        let validator = MatchValidator::new(context);

        // 测试没有聚合函数的表达式
        let non_agg_expr = Expression::Constant(ConstantValue::Int(1));
        assert_eq!(validator.has_aggregate_expr(&non_agg_expr), false);

        // 测试包含聚合函数的表达式
        // 注意：这里需要一个聚合表达式实例，具体实现可能依赖Expression的定义
        // 对于测试目的，我们暂时忽略这个测试，因为Expression::Aggregate可能需要特定构造
    }

    #[test]
    fn test_combine_aliases() {
        let context = ValidateContext::default();
        let mut validator = MatchValidator::new(context);

        let mut cur_aliases = HashMap::new();
        cur_aliases.insert("a".to_string(), AliasType::Node);

        let mut last_aliases = HashMap::new();
        last_aliases.insert("b".to_string(), AliasType::Edge);
        last_aliases.insert("c".to_string(), AliasType::Path);

        // 组合别名
        assert!(validator.combine_aliases(&mut cur_aliases, &last_aliases).is_ok());
        assert_eq!(cur_aliases.len(), 3);
        assert!(cur_aliases.contains_key("a"));
        assert!(cur_aliases.contains_key("b"));
        assert!(cur_aliases.contains_key("c"));
    }

    #[test]
    fn test_validate_step_range() {
        let context = ValidateContext::default();
        let mut validator = MatchValidator::new(context);

        // 测试有效的范围（min <= max）
        let valid_range = MatchStepRange::new(1, 3);
        assert!(validator.validate_step_range(&valid_range).is_ok());

        // 测试无效的范围（min > max）
        let invalid_range = MatchStepRange::new(3, 1);
        assert!(validator.validate_step_range(&invalid_range).is_err());
    }
}