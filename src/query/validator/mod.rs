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

// 表达式分析器
pub mod expression_analyzer;

// 导出数据结构
pub use structs::{
    AggregateCallInfo, AliasType, ClauseKind, HintSeverity, IndexHint, MatchClauseContext,
    MatchStepRange, OptimizationHint, PaginationContext, Path, PathAnalysis, QueryPart,
    ReturnClauseContext, SemanticInfo, UnwindClauseContext, ValidatedStatement, ValidationInfo,
    WhereClauseContext, WithClauseContext, YieldClauseContext,
};

// 从 core 重新导出 YieldColumn
pub use crate::core::YieldColumn;

// 导出新的验证器体系（trait + 枚举）
pub use validator_enum::{Validator, ValidatorCollection};
pub use validator_trait::{
    is_global_statement_type, ColumnDef, EdgeProperty, ExpressionProps, InputProperty,
    StatementType, StatementValidator, TagProperty, ValidationResult, ValueType, VarProperty,
};

// 导出语句级验证器
pub use statements::{
    CreateValidator, DeleteValidator, FetchEdgesValidator, FetchVerticesValidator,
    FindPathValidator, GetSubgraphValidator, GoValidator, InsertEdgesValidator,
    InsertVerticesValidator, LookupValidator, MatchValidator, MergeValidator, RemoveValidator,
    SetItem, SetStatementType, SetValidator, UnwindValidator, UpdateValidator, ValidatedSet,
    ValidatedSetItem, ValidatedUnwind,
};

// 导出子句级验证器
pub use clauses::{
    GroupByValidator, LimitValidator, OrderByValidator, OrderColumn, ReturnValidator,
    SequentialStatement, SequentialValidator, ValidatedGroupBy, ValidatedYield, WithValidator,
    YieldValidator,
};

// 导出 DDL 验证器
pub use ddl::{
    AlterTargetType, AlterValidator, DescTargetType, DescValidator, DropTargetType, DropValidator,
    KillQueryValidator, ShowConfigsValidator, ShowCreateValidator, ShowQueriesValidator,
    ShowSessionsValidator, ShowTargetType, ShowValidator, ValidatedAlter, ValidatedDesc,
    ValidatedDrop, ValidatedShow,
};

// 导出 DML 验证器
pub use dml::{
    ColumnInfo, PipeValidator, QueryValidator, SetOperationValidator, UseValidator,
    ValidatedSetOperation, ValidatedUse,
};

// 导出工具验证器
pub use utility::{
    AlterUserValidator, ChangePasswordValidator, CreateUserValidator, DescribeUserValidator,
    DropUserValidator, ExplainValidator, GrantValidator, ProfileValidator, RevokeValidator,
    ShowRolesValidator, ShowUsersValidator, UpdateConfigsValidator, ValidatedExplain,
    ValidatedGrant, ValidatedUser,
};

// 导出辅助工具
pub use helpers::SchemaValidator;

// 导出 assignment 验证器
pub use assignment_validator::{AssignmentValidator, ValidatedAssignment};

// 导出表达式分析器
pub use expression_analyzer::{ExpressionAnalysisResult, ExpressionAnalyzer};
