//! 验证策略工厂
//! 负责创建和管理验证策略实例

use std::collections::HashMap;

use super::base_validator::Validator;
use super::strategies::*;
use super::validation_interface::ValidationStrategyType;

#[derive(Debug, Clone, Default)]
pub struct ValidatorConfig {
    pub enable_type_check: bool,
    pub enable_permission_check: bool,
    pub max_nesting_depth: usize,
    pub max_expression_count: usize,
}

impl ValidatorConfig {
    pub fn new() -> Self {
        Self {
            enable_type_check: true,
            enable_permission_check: true,
            max_nesting_depth: 100,
            max_expression_count: 1000,
        }
    }

    pub fn with_type_check(mut self, enable: bool) -> Self {
        self.enable_type_check = enable;
        self
    }

    pub fn with_permission_check(mut self, enable: bool) -> Self {
        self.enable_permission_check = enable;
        self
    }

    pub fn with_max_nesting_depth(mut self, depth: usize) -> Self {
        self.max_nesting_depth = depth;
        self
    }

    pub fn with_max_expression_count(mut self, count: usize) -> Self {
        self.max_expression_count = count;
        self
    }
}

pub struct ValidationFactory {
    validators: HashMap<&'static str, Box<dyn Fn() -> Validator>>,
    config: ValidatorConfig,
}

impl ValidationFactory {
    pub fn new() -> Self {
        let mut factory = Self {
            validators: HashMap::new(),
            config: ValidatorConfig::new(),
        };

        factory.register_default_validators();
        factory
    }

    pub fn create_all_strategies() -> Vec<Box<dyn super::validation_interface::ValidationStrategy>> {
        vec![
            Box::new(AliasValidationStrategy::new()),
            Box::new(ExpressionValidationStrategy::new()),
            Box::new(ClauseValidationStrategy::new()),
            Box::new(AggregateValidationStrategy::new()),
            Box::new(PaginationValidationStrategy::new()),
        ]
    }

    fn register_default_validators(&mut self) {
        self.register("MATCH", || Validator::new());
        self.register("GO", || Validator::new());
        self.register("LOOKUP", || Validator::new());
        self.register("FETCH_VERTICES", || Validator::new());
        self.register("FETCH_EDGES", || Validator::new());
        self.register("USE", || Validator::new());
        self.register("PIPE", || Validator::new());
        self.register("YIELD", || Validator::new());
        self.register("ORDER_BY", || Validator::new());
        self.register("LIMIT", || Validator::new());
        self.register("UNWIND", || Validator::new());
        self.register("FIND_PATH", || Validator::new());
        self.register("GET_SUBGRAPH", || Validator::new());
        self.register("SET", || Validator::new());
        self.register("SEQUENTIAL", || Validator::new());
        
        self.register("INSERT_VERTICES", || Validator::new());
        self.register("INSERT_EDGES", || Validator::new());
        self.register("UPDATE", || Validator::new());
        self.register("DELETE", || Validator::new());
        self.register("CREATE_SPACE", || Validator::new());
        self.register("DROP_SPACE", || Validator::new());
        self.register("CREATE_TAG", || Validator::new());
        self.register("ALTER_TAG", || Validator::new());
        self.register("DROP_TAG", || Validator::new());
        self.register("CREATE_EDGE", || Validator::new());
        self.register("ALTER_EDGE", || Validator::new());
        self.register("DROP_EDGE", || Validator::new());
        self.register("SHOW_SPACES", || Validator::new());
        self.register("SHOW_TAGS", || Validator::new());
        self.register("SHOW_EDGES", || Validator::new());
    }

    pub fn register<F>(&mut self, name: &'static str, creator: F)
    where
        F: Fn() -> Validator + 'static,
    {
        self.validators.insert(name, Box::new(creator));
    }

    pub fn create(&self, statement_type: &str) -> Validator {
        if let Some(creator) = self.validators.get(statement_type) {
            creator()
        } else {
            Validator::new()
        }
    }

    pub fn set_config(&mut self, config: ValidatorConfig) {
        self.config = config;
    }

    pub fn config(&self) -> &ValidatorConfig {
        &self.config
    }
}

impl Default for ValidationFactory {
    fn default() -> Self {
        Self::new()
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
