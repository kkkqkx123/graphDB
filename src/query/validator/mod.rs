//! 查询验证器模块
//! 对应 NebulaGraph src/graph/validator 的功能
//! 用于验证 AST 的合法性

mod match_validator;
mod base_validator;
pub mod validate_context;

pub use match_validator::MatchValidator;
pub use base_validator::Validator;
pub use validate_context::{ValidateContext, Variable};

#[cfg(test)]
mod tests;