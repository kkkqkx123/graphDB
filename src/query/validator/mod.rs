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

// 验证策略子模块
pub mod strategies;

// 新的验证器体系（trait + 枚举）
pub mod validator_trait;
pub mod validator_enum;
pub mod admin_validator;
pub mod acl_validator;
pub mod alter_validator;
pub mod assignment_validator;
pub mod create_validator;
pub mod delete_validator;
pub mod drop_validator;
pub mod explain_validator;
pub mod fetch_edges_validator;
pub mod fetch_vertices_validator;
pub mod find_path_validator;
pub mod get_subgraph_validator;
pub mod go_validator;
pub mod group_by_validator;
pub mod insert_edges_validator;
pub mod insert_vertices_validator;
pub mod limit_validator;
pub mod lookup_validator;
pub mod match_validator;
pub mod order_by_validator;
pub mod pipe_validator;
pub mod schema_validator;
pub mod sequential_validator;
pub mod set_operation_validator;
pub mod set_validator;
pub mod unwind_validator;
pub mod update_config_validator;
pub mod update_validator;
pub mod use_validator;
pub mod yield_validator;
pub mod merge_validator;
pub mod return_validator;
pub mod with_validator;
pub mod remove_validator;
pub mod query_validator;
pub mod validation_info;

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
};

// 从 core 重新导出 YieldColumn
pub use crate::core::YieldColumn;

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
pub use admin_validator::{
    ShowValidator, DescValidator, ShowCreateValidator, ShowConfigsValidator,
    ShowSessionsValidator, ShowQueriesValidator, KillQueryValidator,
    ValidatedShow, ShowTargetType, ValidatedDesc, DescTargetType,
};
pub use acl_validator::{
    CreateUserValidator, DropUserValidator, AlterUserValidator, ChangePasswordValidator,
    GrantValidator, RevokeValidator, DescribeUserValidator, ShowUsersValidator, ShowRolesValidator,
    ValidatedUser, ValidatedGrant,
};
pub use alter_validator::{AlterValidator, ValidatedAlter, AlterTargetType};
pub use assignment_validator::{AssignmentValidator, ValidatedAssignment};
pub use create_validator::CreateValidator;
pub use delete_validator::DeleteValidator;
pub use drop_validator::{DropValidator, ValidatedDrop, DropTargetType};
pub use explain_validator::{ExplainValidator, ProfileValidator, ValidatedExplain};
pub use fetch_edges_validator::FetchEdgesValidator;
pub use fetch_vertices_validator::FetchVerticesValidator;
pub use find_path_validator::FindPathValidator;
pub use get_subgraph_validator::GetSubgraphValidator;
pub use go_validator::GoValidator;
pub use group_by_validator::{GroupByValidator, ValidatedGroupBy};
pub use insert_edges_validator::InsertEdgesValidator;
pub use insert_vertices_validator::InsertVerticesValidator;
pub use limit_validator::LimitValidator;
pub use lookup_validator::LookupValidator;
pub use match_validator::MatchValidator;
pub use order_by_validator::{OrderByValidator, OrderColumn};
pub use pipe_validator::{PipeValidator, ColumnInfo};
pub use schema_validator::SchemaValidator;
pub use sequential_validator::{SequentialValidator, SequentialStatement};
pub use set_operation_validator::{SetOperationValidator, ValidatedSetOperation};
pub use set_validator::{SetValidator, SetItem, SetStatementType, ValidatedSet, ValidatedSetItem};
pub use unwind_validator::{UnwindValidator, ValidatedUnwind};
pub use update_config_validator::UpdateConfigsValidator;
pub use update_validator::UpdateValidator;
pub use use_validator::{UseValidator, ValidatedUse};
pub use yield_validator::{YieldValidator, ValidatedYield};
pub use merge_validator::MergeValidator;
pub use return_validator::ReturnValidator;
pub use with_validator::WithValidator;
pub use remove_validator::RemoveValidator;
pub use query_validator::QueryValidator;

// 导出验证信息相关类型
pub use validation_info::{
    ValidatedStatement,
    ValidationInfo,
    PathAnalysis,
    OptimizationHint,
    IndexHint,
    SemanticInfo,
    AggregateCallInfo,
    ClauseKind,
    HintSeverity,
};
