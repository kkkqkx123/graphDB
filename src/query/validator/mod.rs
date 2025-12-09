//! 查询验证器模块（重构版）
//! 对应 NebulaGraph src/graph/validator 的功能
//! 用于验证 AST 的合法性
//!
//! 重构说明：
//! 1. 采用策略模式，将验证逻辑分解为独立的策略类
//! 2. 引入工厂模式，统一管理验证策略的创建
//! 3. 消除循环依赖，提高模块的可维护性和可测试性
//! 4. 合并冗余文件，拆分大型文件

mod base_validator;
mod match_validator;
mod validate_context;
mod validation_factory;
mod validation_interface;
mod aggregate_validator;
mod alias_validator;
mod clause_validator;
mod expression_validator;
mod pagination_validator;

pub mod strategies;
pub mod structs;

pub use base_validator::Validator;
pub use match_validator::MatchValidator;
pub use validate_context::{ValidateContext, Variable};
pub use validation_factory::ValidationFactory;

// 导出策略模块
pub use strategies::*;
// 导出结构模块
pub use structs::*;

// 为了向后兼容，保留旧的导出
// 这些将在重构完成后被移除
pub use crate::query::validator::aggregate_validator::AggregateValidator;
pub use crate::query::validator::alias_validator::AliasValidator;
pub use crate::query::validator::clause_validator::ClauseValidator;
pub use crate::query::validator::expression_validator::ExpressionValidator;
pub use crate::query::validator::pagination_validator::PaginationValidator;
