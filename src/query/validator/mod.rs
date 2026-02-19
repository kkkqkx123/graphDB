//! 查询验证器模块（重构版）
//! 对应 NebulaGraph src/graph/validator 的功能
//! 用于验证 AST 的合法性
//!
//! 重构说明：
//! 1. 采用 trait + 枚举模式，替代原有的组合式继承
//! 2. 使用泛型策略模式，避免 dyn 开销
//! 3. 统一验证上下文，消除双重上下文问题
//! 4. 消除循环依赖，提高模块的可维护性和可测试性
//! 5. 提供统一的 StatementValidator trait 接口

// 核心模块（新架构）
pub mod core;
pub mod docs;

// 具体验证器实现（保持现有，后续逐一迁移）
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
pub mod lookup_validator;
pub mod find_path_validator;
pub mod get_subgraph_validator;
pub mod set_validator;
pub mod sequential_validator;
pub mod insert_vertices_validator;
pub mod insert_edges_validator;
pub mod update_validator;
pub mod delete_validator;
pub mod create_validator;
pub mod schema_validator;

// 策略模块
pub mod strategies;
pub mod structs;

// 核心模块导出（新架构）
pub use core::{
    ColumnDef, DefaultStrategySet, EdgeProperty, ExpressionProps, InputProperty,
    StatementType, StatementValidator, StrategyResult, StrategySet, TagProperty,
    ValidationStrategy, ValidationStrategyType, Validator, ValidatorBuilder, VarProperty,
};

// 错误类型导出
pub use crate::core::error::{ValidationError, ValidationErrorType};

// 验证上下文导出
pub use crate::query::context::validate::ValidationContext;

// 策略模块导出
pub use strategies::*;
pub use structs::*;

// 具体验证器导出（保持现有，后续迁移到新架构）
pub use match_validator::MatchValidator;
pub use go_validator::GoValidator;
pub use fetch_vertices_validator::FetchVerticesValidator;
pub use fetch_edges_validator::FetchEdgesValidator;
pub use pipe_validator::PipeValidator;
pub use yield_validator::YieldValidator;
pub use order_by_validator::OrderByValidator;
pub use limit_validator::LimitValidator;
pub use use_validator::UseValidator;
pub use unwind_validator::UnwindValidator;
pub use lookup_validator::LookupValidator;
pub use find_path_validator::FindPathValidator;
pub use get_subgraph_validator::GetSubgraphValidator;
pub use set_validator::SetValidator;
pub use sequential_validator::SequentialValidator;
pub use insert_vertices_validator::InsertVerticesValidator;
pub use insert_edges_validator::InsertEdgesValidator;
pub use update_validator::UpdateValidator;
pub use delete_validator::DeleteValidator;
pub use create_validator::CreateValidator;
pub use schema_validator::SchemaValidator;
