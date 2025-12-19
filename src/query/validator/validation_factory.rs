//! 验证策略工厂
//! 负责创建和管理验证策略实例

use super::strategies::*;
use super::validation_interface::*;

/// 验证策略工厂
pub struct ValidationFactory;

impl ValidationFactory {
    /// 创建指定类型的验证策略
    pub fn create_strategy(strategy_type: ValidationStrategyType) -> Box<dyn ValidationStrategy> {
        match strategy_type {
            ValidationStrategyType::Alias => Box::new(AliasValidationStrategy::new()),
            ValidationStrategyType::Expression => Box::new(ExpressionValidationStrategy::new()),
            ValidationStrategyType::Clause => Box::new(ClauseValidationStrategy::new()),
            ValidationStrategyType::Aggregate => Box::new(AggregateValidationStrategy::new()),
            ValidationStrategyType::Pagination => Box::new(PaginationValidationStrategy::new()),
        }
    }

    /// 创建所有验证策略
    pub fn create_all_strategies() -> Vec<Box<dyn ValidationStrategy>> {
        vec![
            Self::create_strategy(ValidationStrategyType::Alias),
            Self::create_strategy(ValidationStrategyType::Expression),
            Self::create_strategy(ValidationStrategyType::Clause),
            Self::create_strategy(ValidationStrategyType::Aggregate),
            Self::create_strategy(ValidationStrategyType::Pagination),
        ]
    }

    /// 创建特定验证策略组合
    pub fn create_strategy_set(
        strategy_types: &[ValidationStrategyType],
    ) -> Vec<Box<dyn ValidationStrategy>> {
        strategy_types
            .iter()
            .map(|strategy_type| Self::create_strategy(strategy_type.clone()))
            .collect()
    }
}
