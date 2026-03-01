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

// 语句级验证器
pub mod statements;

// 子句级验证器
pub mod clauses;

// DDL 验证器
pub mod ddl;

// DML 验证器
pub mod dml;

// 工具验证器
pub mod utility;

// 辅助工具
pub mod helpers;

// 验证器 trait 定义
pub mod validator_trait;

// 验证器枚举
pub mod validator_enum;

// assignment 验证器
pub mod assignment_validator;

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

// 导出语句级验证器
pub use statements::{
    MatchValidator,
    CreateValidator,
    InsertVerticesValidator,
    InsertEdgesValidator,
    UpdateValidator,
    DeleteValidator,
    MergeValidator,
    RemoveValidator,
    SetValidator,
    SetItem,
    SetStatementType,
    ValidatedSet,
    ValidatedSetItem,
    UnwindValidator,
    ValidatedUnwind,
    LookupValidator,
    FetchVerticesValidator,
    FetchEdgesValidator,
    GoValidator,
    FindPathValidator,
    GetSubgraphValidator,
};

// 导出子句级验证器
pub use clauses::{
    GroupByValidator,
    ValidatedGroupBy,
    OrderByValidator,
    OrderColumn,
    LimitValidator,
    YieldValidator,
    ValidatedYield,
    ReturnValidator,
    WithValidator,
    SequentialValidator,
    SequentialStatement,
};

// 导出 DDL 验证器
pub use ddl::{
    DropValidator,
    ValidatedDrop,
    DropTargetType,
    AlterValidator,
    ValidatedAlter,
    AlterTargetType,
    ShowValidator,
    DescValidator,
    ShowCreateValidator,
    ShowConfigsValidator,
    ShowSessionsValidator,
    ShowQueriesValidator,
    KillQueryValidator,
    ValidatedShow,
    ShowTargetType,
    ValidatedDesc,
    DescTargetType,
};

// 导出 DML 验证器
pub use dml::{
    UseValidator,
    ValidatedUse,
    PipeValidator,
    ColumnInfo,
    QueryValidator,
    SetOperationValidator,
    ValidatedSetOperation,
};

// 导出工具验证器
pub use utility::{
    ExplainValidator,
    ProfileValidator,
    ValidatedExplain,
    CreateUserValidator,
    DropUserValidator,
    AlterUserValidator,
    ChangePasswordValidator,
    GrantValidator,
    RevokeValidator,
    DescribeUserValidator,
    ShowUsersValidator,
    ShowRolesValidator,
    ValidatedUser,
    ValidatedGrant,
    UpdateConfigsValidator,
};

// 导出辅助工具
pub use helpers::SchemaValidator;

// 导出 assignment 验证器
pub use assignment_validator::{AssignmentValidator, ValidatedAssignment};
