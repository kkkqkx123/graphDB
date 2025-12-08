//! 查询验证器模块
//! 对应 NebulaGraph src/graph/validator 的功能
//! 用于验证 AST 的合法性

mod base_validator;
mod match_validator;
mod match_structs;
pub mod validate_context;

pub use base_validator::Validator;
pub use match_validator::MatchValidator;
pub use validate_context::{ValidateContext, Variable};
