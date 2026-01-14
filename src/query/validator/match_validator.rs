//! Match语句验证器（重构版）
//! 对应原match_validator_main.rs的功能，使用新的策略模式架构

use super::base_validator::Validator;
use super::structs::{
    AliasType, MatchStepRange, PaginationContext, Path, QueryPart, ReturnClauseContext,
    UnwindClauseContext, WhereClauseContext, WithClauseContext, YieldClauseContext, YieldColumn,
};
// 使用context版本的ValidationContext
use super::validation_factory::ValidationFactory;
use super::validation_interface::{ValidationError, ValidationErrorType, ValidationStrategy};
use super::ValidationContext;
use crate::core::Expression;
use std::collections::HashMap;

/// Match语句验证器
pub struct MatchValidator {
    base: Validator,
    validation_strategies: Vec<Box<dyn ValidationStrategy>>,
}

impl MatchValidator {
    pub fn new(context: ValidationContext) -> Self {
        Self {
            base: Validator::new(context),
            validation_strategies: ValidationFactory::create_all_strategies(),
        }
    }

    pub fn validate(&mut self) -> Result<(), ValidationError> {
        // 执行所有验证策略
        for strategy in &self.validation_strategies {
            // 现在ValidationContext已经实现了ValidationContext trait
            if let Err(error) = strategy.validate(self.base.context()) {
                self.base.context_mut().add_validation_error(error);
            }
        }

        if self.base.context().has_validation_errors() {
            return Err(ValidationError::new(
                "验证失败".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        Ok(())
    }

    /// 使用统一错误类型的验证方法
    pub fn validate_unified(&mut self) -> Result<(), crate::core::error::DBError> {
        // 执行所有验证策略
        for strategy in &self.validation_strategies {
            // 现在ValidationContext已经实现了ValidationContext trait
            if let Err(error) = strategy.validate(self.base.context()) {
                self.base.context_mut().add_validation_error(error);
            }
        }

        if self.base.context().has_validation_errors() {
            // 将ValidationError转换为DBError
            let errors = self.base.context().get_validation_errors();
            if let Some(first_error) = errors.first() {
                return Err(first_error.clone().to_db_error());
            }
        }

        Ok(())
    }

    /// 获取验证上下文的可变引用
    pub fn context_mut(&mut self) -> &mut ValidationContext {
        self.base.context_mut()
    }

    /// 获取验证上下文的引用
    pub fn context(&self) -> &ValidationContext {
        self.base.context()
    }

    /// 验证别名（委托给AliasValidationStrategy）
    pub fn validate_aliases(
        &mut self,
        exprs: &[Expression],
        aliases: &HashMap<String, AliasType>,
    ) -> Result<(), ValidationError> {
        use super::strategies::AliasValidationStrategy;
        let strategy = AliasValidationStrategy::new();
        strategy.validate_aliases(exprs, aliases)
    }

    /// 检查表达式是否包含聚合函数（委托给AggregateValidationStrategy）
    pub fn has_aggregate_expr(&self, expr: &Expression) -> bool {
        use super::strategies::AggregateValidationStrategy;
        let strategy = AggregateValidationStrategy::new();
        strategy.has_aggregate_expr(expr)
    }

    /// 验证分页（委托给PaginationValidationStrategy）
    pub fn validate_pagination(
        &mut self,
        skip_expr: Option<&Expression>,
        limit_expr: Option<&Expression>,
        context: &PaginationContext,
    ) -> Result<(), ValidationError> {
        use super::strategies::PaginationValidationStrategy;
        let strategy = PaginationValidationStrategy::new();
        strategy.validate_pagination(skip_expr, limit_expr, context)
    }

    /// 验证步数范围（委托给PaginationValidationStrategy）
    pub fn validate_step_range(&self, range: &MatchStepRange) -> Result<(), ValidationError> {
        use super::strategies::PaginationValidationStrategy;
        let strategy = PaginationValidationStrategy::new();
        strategy.validate_step_range(range)
    }

    /// 验证过滤条件（委托给ExpressionValidationStrategy）
    pub fn validate_filter(
        &mut self,
        filter: &Expression,
        context: &WhereClauseContext,
    ) -> Result<(), ValidationError> {
        use super::strategies::ExpressionValidationStrategy;
        let strategy = ExpressionValidationStrategy::new();
        strategy.validate_filter(filter, context)
    }

    /// 验证Return子句（委托给ExpressionValidationStrategy）
    pub fn validate_return(
        &mut self,
        return_expr: &Expression,
        return_items: &[YieldColumn],
        context: &ReturnClauseContext,
    ) -> Result<(), ValidationError> {
        use super::strategies::ExpressionValidationStrategy;
        let strategy = ExpressionValidationStrategy::new();
        strategy.validate_return(return_expr, return_items, context)
    }

    /// 验证With子句（委托给ExpressionValidationStrategy）
    pub fn validate_with(
        &mut self,
        with_expr: &Expression,
        with_items: &[YieldColumn],
        context: &WithClauseContext,
    ) -> Result<(), ValidationError> {
        use super::strategies::ExpressionValidationStrategy;
        let strategy = ExpressionValidationStrategy::new();
        strategy.validate_with(with_expr, with_items, context)
    }

    /// 验证Unwind子句（委托给ExpressionValidationStrategy）
    pub fn validate_unwind(
        &mut self,
        unwind_expr: &Expression,
        context: &UnwindClauseContext,
    ) -> Result<(), ValidationError> {
        use super::strategies::ExpressionValidationStrategy;
        let strategy = ExpressionValidationStrategy::new();
        strategy.validate_unwind(unwind_expr, context)
    }

    /// 验证Yield子句（委托给ClauseValidationStrategy）
    pub fn validate_yield(&mut self, context: &YieldClauseContext) -> Result<(), ValidationError> {
        use super::strategies::ClauseValidationStrategy;
        let strategy = ClauseValidationStrategy::new();
        strategy.validate_yield_clause(context)
    }

    /// 构建所有命名别名的列（委托给ClauseValidationStrategy）
    pub fn build_columns_for_all_named_aliases(
        &mut self,
        query_parts: &[QueryPart],
        columns: &mut Vec<YieldColumn>,
    ) -> Result<(), ValidationError> {
        use super::strategies::ClauseValidationStrategy;
        let strategy = ClauseValidationStrategy::new();
        strategy.build_columns_for_all_named_aliases(query_parts, columns)
    }

    /// 结合别名（委托给AliasValidationStrategy）
    pub fn combine_aliases(
        &mut self,
        cur_aliases: &mut HashMap<String, AliasType>,
        last_aliases: &HashMap<String, AliasType>,
    ) -> Result<(), ValidationError> {
        use super::strategies::AliasValidationStrategy;
        let strategy = AliasValidationStrategy::new();
        strategy.combine_aliases(cur_aliases, last_aliases)
    }

    /// 构建输出（委托给ClauseValidationStrategy）
    pub fn build_outputs(&mut self, paths: &mut Vec<Path>) -> Result<(), ValidationError> {
        use super::strategies::ClauseValidationStrategy;
        let strategy = ClauseValidationStrategy::new();
        strategy.build_outputs(paths)
    }

    /// 检查别名（委托给AliasValidationStrategy）
    pub fn check_alias(
        &mut self,
        ref_expr: &Expression,
        aliases_available: &HashMap<String, AliasType>,
    ) -> Result<(), ValidationError> {
        use super::strategies::AliasValidationStrategy;
        let strategy = AliasValidationStrategy::new();
        strategy.check_alias(ref_expr, aliases_available)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;

    #[test]
    fn test_match_validator_creation() {
        let context = ValidationContext::new();
        let validator = MatchValidator::new(context);

        assert_eq!(validator.validation_strategies.len(), 5); // 应该有5个策略
    }

    #[test]
    fn test_basic_validation() {
        let context = ValidationContext::new();
        let mut validator = MatchValidator::new(context);

        // 简单验证应该成功
        assert!(validator.validate().is_ok());
    }

    #[test]
    fn test_validate_pagination() {
        let context = ValidationContext::new();
        let mut validator = MatchValidator::new(context);

        // 测试有效的分页表达式
        let skip_expr = Expression::Literal(crate::core::Value::Int(1));
        let limit_expr = Expression::Literal(crate::core::Value::Int(10));
        let pagination_ctx = PaginationContext { skip: 0, limit: 10 };

        assert!(validator
            .validate_pagination(Some(&skip_expr), Some(&limit_expr), &pagination_ctx)
            .is_ok());
    }

    #[test]
    fn test_validate_aliases() {
        let context = ValidationContext::new();
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
        assert!(validator
            .validate_aliases(&[invalid_expr], &aliases)
            .is_err());
    }

    #[test]
    fn test_has_aggregate_expr() {
        let context = ValidationContext::new();
        let validator = MatchValidator::new(context);

        // 测试没有聚合函数的表达式
        let non_agg_expr = Expression::Literal(crate::core::Value::Int(1));
        assert_eq!(validator.has_aggregate_expr(&non_agg_expr), false);
    }

    #[test]
    fn test_combine_aliases() {
        let context = ValidationContext::new();
        let mut validator = MatchValidator::new(context);

        let mut cur_aliases = HashMap::new();
        cur_aliases.insert("a".to_string(), AliasType::Node);

        let mut last_aliases = HashMap::new();
        last_aliases.insert("b".to_string(), AliasType::Edge);
        last_aliases.insert("c".to_string(), AliasType::Path);

        // 组合别名
        assert!(validator
            .combine_aliases(&mut cur_aliases, &last_aliases)
            .is_ok());
        assert_eq!(cur_aliases.len(), 3);
        assert!(cur_aliases.contains_key("a"));
        assert!(cur_aliases.contains_key("b"));
        assert!(cur_aliases.contains_key("c"));
    }

    #[test]
    fn test_validate_step_range() {
        let context = ValidationContext::new();
        let validator = MatchValidator::new(context);

        // 测试有效的范围（min <= max）
        let valid_range = MatchStepRange::new(1, 3);
        assert!(validator.validate_step_range(&valid_range).is_ok());

        // 测试无效的范围（min > max）
        let invalid_range = MatchStepRange::new(3, 1);
        assert!(validator.validate_step_range(&invalid_range).is_err());
    }
}
