//! 查询验证器模块
//! 对应 NebulaGraph src/graph/validator 的功能
//! 用于验证 AST 的合法性
//!
//! 设计说明：
//! 采用 trait + 枚举模式管理验证器
//! - trait 定义统一接口
//! - 枚举实现静态分发
//! - 工厂模式创建验证器

// 数据结构模块
pub mod structs;

// 新的验证器体系（trait + 枚举）
pub mod validator_trait;
pub mod validator_enum;
pub mod create_validator;
pub mod schema_validator;
pub mod delete_validator;
pub mod fetch_edges_validator;
pub mod find_path_validator;
pub mod fetch_vertices_validator;
pub mod get_subgraph_validator;
pub mod go_validator;
pub mod insert_edges_validator;
pub mod insert_vertices_validator;
pub mod limit_validator;
pub mod lookup_validator;
pub mod match_validator;
pub mod order_by_validator;
pub mod pipe_validator;
pub mod sequential_validator;
pub mod set_validator;
pub mod unwind_validator;
pub mod update_validator;
pub mod use_validator;
pub mod yield_validator;

// 导出数据结构
pub use structs::{
    AliasType,
    MatchClauseContext,
    MatchStepRange,
    PaginationContext,
    Path,
    QueryPart,
    ReturnClauseContext,
    UnwindClauseContext,
    WhereClauseContext,
    WithClauseContext,
    YieldClauseContext,
    YieldColumn,
};

// 导出新的验证器体系（trait + 枚举）
pub use validator_trait::{
    StatementType,
    StatementValidator,
    ValidationResult,
    is_global_statement_type,
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
pub use delete_validator::DeleteValidator;
pub use fetch_edges_validator::FetchEdgesValidator;
pub use fetch_vertices_validator::FetchVerticesValidator;
pub use go_validator::GoValidator;
pub use insert_edges_validator::InsertEdgesValidator;
pub use insert_vertices_validator::InsertVerticesValidator;
pub use limit_validator::LimitValidator;
pub use lookup_validator::LookupValidator;
pub use match_validator::MatchValidator;
pub use order_by_validator::{OrderByValidator, OrderColumn};
pub use pipe_validator::{PipeValidator, ColumnInfo};
pub use sequential_validator::{SequentialValidator, SequentialStatement};
pub use set_validator::{SetValidator, SetItem, SetStatementType, ValidatedSet, ValidatedSetItem};
pub use unwind_validator::{UnwindValidator, ValidatedUnwind};
pub use use_validator::{UseValidator, ValidatedUse};
pub use yield_validator::{YieldValidator, ValidatedYield};
