//! 查询验证器模块
//! 对应 NebulaGraph src/graph/validator 的功能
//! 用于验证 AST 的合法性

mod base_validator;
mod match_validator_main;
mod match_structs;
mod alias_validator;
mod aggregate_validator;
mod pagination_validator;
mod expression_validator;
mod clause_validator;
pub mod validate_context;

pub use base_validator::Validator;
pub use match_validator_main::MatchValidator;
pub use validate_context::{ValidateContext, Variable};

// 导出子验证器模块
pub use alias_validator::AliasValidator;
pub use aggregate_validator::AggregateValidator;
pub use pagination_validator::PaginationValidator;
pub use expression_validator::ExpressionValidator;
pub use clause_validator::ClauseValidator;
