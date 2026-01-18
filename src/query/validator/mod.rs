//! 查询验证器模块（重构版）
//! 对应 NebulaGraph src/graph/validator 的功能
//! 用于验证 AST 的合法性
//!
//! 重构说明：
//! 1. 采用策略模式，将验证逻辑分解为独立的策略类
//! 2. 引入工厂模式，统一管理验证策略的创建
//! 3. 引入注册表模式，支持动态注册验证器
//! 4. 消除循环依赖，提高模块的可维护性和可测试性
//! 5. 合并冗余文件，拆分大型文件

pub mod base_validator;
pub mod match_validator;
pub mod go_validator;
pub mod fetch_vertices_validator;
pub mod fetch_edges_validator;
pub mod pipe_validator;
pub mod yield_validator;
pub mod order_by_validator;
pub mod limit_validator;
pub mod use_validator;
pub mod unwind_validator;
pub mod validation_factory;
pub mod validation_interface;

pub mod strategies;
pub mod structs;

pub use base_validator::{
    Validator,
    ColumnDef,
    ValueType,
    ExpressionProps,
    InputProperty,
    VarProperty,
    TagProperty,
    EdgeProperty,
};
pub use match_validator::MatchValidator;
pub use go_validator::{GoValidator, GoContext, GoSource, GoYieldColumn};
pub use fetch_vertices_validator::{FetchVerticesValidator, FetchVerticesContext, FetchVertexId};
pub use fetch_edges_validator::{FetchEdgesValidator, FetchEdgesContext, FetchEdgeKey};
pub use pipe_validator::{PipeValidator, ColumnInfo};
pub use yield_validator::YieldValidator;
pub use order_by_validator::{OrderByValidator, OrderColumn, SortDirection};
pub use limit_validator::LimitValidator;
pub use use_validator::UseValidator;
pub use unwind_validator::UnwindValidator;

pub use validation_factory::{
    ValidationFactory,
    ValidatorRegistry,
    ValidatorBuilder,
    StatementType,
};

pub use validation_interface::{
    ValidationError,
    ValidationErrorType,
    ValidationStrategy,
    ValidationStrategyType,
};

// 重新导出context版本的ValidationContext
pub use crate::query::context::validate::ValidationContext;

// 为了向后兼容，导出类型定义
pub use crate::query::context::validate::types::{Column, Variable};

// 导出策略模块
pub use strategies::*;
// 导出结构模块
pub use structs::*;
