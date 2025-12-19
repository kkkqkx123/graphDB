//! 查询验证器模块（重构版）
//! 对应 NebulaGraph src/graph/validator 的功能
//! 实现验证+规划的一体化设计
//!
//! 重构说明：
//! 1. 采用Validator trait统一接口，实现验证+规划一体化
//! 2. 引入工厂模式，统一管理验证器的创建
//! 3. 消除循环依赖，提高模块的可维护性和可测试性
//! 4. 合并冗余文件，拆分大型文件

mod base_validator;
mod match_validator;
mod create_validator;
mod validator_factory;
mod validator_registry;
mod validator_trait;

pub use validator_trait::{Validator, ValidatorExt, ValidatorCreator};
pub use base_validator::{
    BaseValidator, YieldColumn, ExpressionProperties,
    CypherClauseKind, CypherClauseContext,
    MatchClauseContext, WhereClauseContext, ReturnClauseContext, WithClauseContext,
    OrderByClauseContext, OrderByColumn, OrderType, PaginationContext,
    UnwindClauseContext, YieldClauseContext,
    Path, PathType, NodeInfo, EdgeInfo, Direction,
    AliasType, QueryPart
};
pub use match_validator::MatchValidator;
pub use create_validator::CreateValidator;
pub use validator_factory::{ValidatorFactory, CypherValidatorFactory};
pub use validator_registry::ValidatorRegistry;