//! 验证策略工厂
//! 负责创建和管理验证策略实例

use super::strategies::*;
use super::validation_interface::*;

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

/// 语句类型枚举
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum StatementType {
    Match,
    Go,
    FetchVertices,
    FetchEdges,
    Lookup,
    FindPath,
    GetSubgraph,
    InsertVertices,
    InsertEdges,
    Update,
    Delete,
    Unwind,
    Yield,
    OrderBy,
    Limit,
    GroupBy,
    CreateSpace,
    CreateTag,
    CreateEdge,
    AlterTag,
    AlterEdge,
    DropSpace,
    DropTag,
    DropEdge,
    DescribeSpace,
    DescribeTag,
    DescribeEdge,
    ShowSpaces,
    ShowTags,
    ShowEdges,
    Use,
    Assignment,
    Set,
    Pipe,
    Sequential,
    Explain,
}

/// 验证器构建器特质
pub trait ValidatorBuilder: Send + Sync {
    fn build(&self, context: &dyn super::validation_interface::ValidationContext) -> Result<Box<dyn super::ValidationStrategy>, super::ValidationError>;
}

/// 通用闭包构建器
pub struct ClosureValidatorBuilder<F>
where
    F: Fn(&dyn super::validation_interface::ValidationContext) -> Result<Box<dyn super::ValidationStrategy>, super::ValidationError> + Send + Sync + 'static,
{
    builder: F,
}

impl<F> ClosureValidatorBuilder<F>
where
    F: Fn(&dyn super::validation_interface::ValidationContext) -> Result<Box<dyn super::ValidationStrategy>, super::ValidationError> + Send + Sync + 'static,
{
    pub fn new(builder: F) -> Self {
        Self { builder }
    }
}

impl<F> ValidatorBuilder for ClosureValidatorBuilder<F>
where
    F: Fn(&dyn super::validation_interface::ValidationContext) -> Result<Box<dyn super::ValidationStrategy>, super::ValidationError> + Send + Sync + 'static,
{
    fn build(&self, context: &dyn super::validation_interface::ValidationContext) -> Result<Box<dyn super::ValidationStrategy>, super::ValidationError> {
        (self.builder)(context)
    }
}

/// 验证器注册表
pub struct ValidatorRegistry {
    builders: std::collections::HashMap<StatementType, Box<dyn ValidatorBuilder>>,
}

impl ValidatorRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            builders: std::collections::HashMap::new(),
        };

        registry.register_default_validators();
        registry
    }

    fn register_default_validators(&mut self) {
        // MatchValidator 是一种复合验证器，不直接作为 ValidationStrategy 使用
        // 它内部使用多个 ValidationStrategy 来执行验证
        // 如果需要单独的 MatchValidator，请直接构造
    }

    pub fn register<B: ValidatorBuilder + 'static>(&mut self, statement_type: StatementType, builder: B) {
        self.builders.insert(statement_type, Box::new(builder));
    }

    pub fn get_validator(
        &self,
        statement_type: &StatementType,
        context: &dyn super::validation_interface::ValidationContext,
    ) -> Option<Result<Box<dyn super::ValidationStrategy>, super::ValidationError>> {
        self.builders.get(statement_type).map(|builder| builder.build(context))
    }

    pub fn register_go_validator(&mut self) {
        self.register(StatementType::Go, ClosureValidatorBuilder::new(|_ctx| {
            Ok(Box::new(super::GoValidator::new(super::ValidationContext::new())))
        }));
    }

    pub fn register_fetch_vertices_validator(&mut self) {
        self.register(StatementType::FetchVertices, ClosureValidatorBuilder::new(|_ctx| {
            Ok(Box::new(super::FetchVerticesValidator::new(super::ValidationContext::new())))
        }));
    }

    pub fn register_fetch_edges_validator(&mut self) {
        self.register(StatementType::FetchEdges, ClosureValidatorBuilder::new(|_ctx| {
            Ok(Box::new(super::FetchEdgesValidator::new(super::ValidationContext::new())))
        }));
    }
}

impl Default for ValidatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}
