//! 查询验证器模块
//! 对应 NebulaGraph src/graph/validator 的功能
//! 用于验证 AST 的合法性
//!
//! 设计说明：
//! 采用 trait + 枚举模式管理验证器
//! - trait 定义统一接口
//! - 枚举实现静态分发
//! - 工厂模式创建验证器

// 新的验证器体系（trait + 枚举）
pub mod validator_trait;
pub mod validator_enum;
pub mod create_validator;
pub mod schema_validator;

// 导出新的验证器体系（trait + 枚举）
pub use validator_trait::{
    StatementType,
    StatementValidator,
    ValidationResult,
    ValidatorBuilder,
    ValidatorRegistry,
    ColumnDef,
    ValueType,
    ExpressionProps,
    InputProperty,
    VarProperty,
    TagProperty,
    EdgeProperty,
};
pub use validator_enum::{
    Validator,
    ValidatorFactory,
    ValidatorCollection,
};

// 导出具体验证器
pub use create_validator::CreateValidator;
pub use schema_validator::SchemaValidator;
