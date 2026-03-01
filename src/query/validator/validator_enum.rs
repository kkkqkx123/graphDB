//! 验证器枚举
//! 使用枚举统一管理所有验证器类型
//! 这是新验证器体系的核心组件，替代 Box<dyn> 的动态分发
//!
//! 设计原则：
//! 1. 保留 base_validator.rs 的完整功能
//! 2. 使用枚举避免动态分发开销
//! 3. 统一接口，便于管理和扩展
//!
//! # 重构变更
//! - validate 方法接收 &Stmt 和 Arc<QueryContext> 替代 &mut AstContext

use std::sync::Arc;

use crate::core::error::ValidationError;
use crate::query::QueryContext;
use crate::query::parser::ast::{Stmt, FetchTarget, CreateTarget};
use crate::query::validator::validator_trait::{
    StatementType, StatementValidator, ValidationResult, ColumnDef, ExpressionProps,
};

// 导入具体验证器
use crate::query::validator::ddl::admin_validator::{
    ShowValidator, DescValidator, ShowCreateValidator, ShowConfigsValidator,
    ShowSessionsValidator, ShowQueriesValidator, KillQueryValidator,
};
use crate::query::validator::utility::acl_validator::{
    CreateUserValidator, DropUserValidator, AlterUserValidator, ChangePasswordValidator,
    GrantValidator, RevokeValidator, DescribeUserValidator, ShowUsersValidator, ShowRolesValidator,
};
use crate::query::validator::ddl::alter_validator::AlterValidator;
use crate::query::validator::assignment_validator::AssignmentValidator;
use crate::query::validator::statements::create_validator::CreateValidator;
use crate::query::validator::statements::delete_validator::DeleteValidator;
use crate::query::validator::ddl::drop_validator::DropValidator;
use crate::query::validator::utility::explain_validator::{ExplainValidator, ProfileValidator};
use crate::query::validator::statements::fetch_edges_validator::FetchEdgesValidator;
use crate::query::validator::statements::fetch_vertices_validator::FetchVerticesValidator;
use crate::query::validator::statements::find_path_validator::FindPathValidator;
use crate::query::validator::statements::get_subgraph_validator::GetSubgraphValidator;
use crate::query::validator::statements::go_validator::GoValidator;
use crate::query::validator::clauses::group_by_validator::GroupByValidator;
use crate::query::validator::statements::insert_edges_validator::InsertEdgesValidator;
use crate::query::validator::statements::insert_vertices_validator::InsertVerticesValidator;
use crate::query::validator::clauses::limit_validator::LimitValidator;
use crate::query::validator::statements::lookup_validator::LookupValidator;
use crate::query::validator::statements::match_validator::MatchValidator;
use crate::query::validator::clauses::order_by_validator::OrderByValidator;
use crate::query::validator::dml::pipe_validator::PipeValidator;
use crate::query::validator::clauses::sequential_validator::SequentialValidator;
use crate::query::validator::dml::set_operation_validator::SetOperationValidator;
use crate::query::validator::statements::set_validator::SetValidator;
use crate::query::validator::statements::unwind_validator::UnwindValidator;
use crate::query::validator::statements::update_validator::UpdateValidator;
use crate::query::validator::dml::use_validator::UseValidator;
use crate::query::validator::clauses::yield_validator::YieldValidator;
use crate::query::validator::utility::update_config_validator::UpdateConfigsValidator;
use crate::query::validator::statements::merge_validator::MergeValidator;
use crate::query::validator::clauses::return_validator::ReturnValidator;
use crate::query::validator::clauses::with_validator::WithValidator;
use crate::query::validator::statements::remove_validator::RemoveValidator;
use crate::query::validator::dml::query_validator::QueryValidator;

/// 统一验证器枚举
///
/// 设计优势：
/// 1. 编译期确定类型，避免动态分发开销
/// 2. 统一接口，便于管理和扩展
/// 3. 模式匹配支持，便于针对特定验证器处理
/// 4. 保留完整的验证生命周期功能
#[derive(Debug)]
pub enum Validator {
    // 管理类验证器
    /// SHOW 语句验证器
    Show(ShowValidator),
    /// DESCRIBE 语句验证器
    Desc(DescValidator),
    /// SHOW CREATE 语句验证器
    ShowCreate(ShowCreateValidator),
    /// SHOW CONFIGS 语句验证器
    ShowConfigs(ShowConfigsValidator),
    /// SHOW SESSIONS 语句验证器
    ShowSessions(ShowSessionsValidator),
    /// SHOW QUERIES 语句验证器
    ShowQueries(ShowQueriesValidator),
    /// KILL QUERY 语句验证器
    KillQuery(KillQueryValidator),

    // 权限类验证器
    /// CREATE USER 语句验证器
    CreateUser(CreateUserValidator),
    /// DROP USER 语句验证器
    DropUser(DropUserValidator),
    /// ALTER USER 语句验证器
    AlterUser(AlterUserValidator),
    /// CHANGE PASSWORD 语句验证器
    ChangePassword(ChangePasswordValidator),
    /// GRANT 语句验证器
    Grant(GrantValidator),
    /// REVOKE 语句验证器
    Revoke(RevokeValidator),
    /// DESCRIBE USER 语句验证器
    DescribeUser(DescribeUserValidator),
    /// SHOW USERS 语句验证器
    ShowUsers(ShowUsersValidator),
    /// SHOW ROLES 语句验证器
    ShowRoles(ShowRolesValidator),

    // DDL 验证器
    /// ALTER 语句验证器
    Alter(AlterValidator),
    /// DROP 语句验证器
    Drop(DropValidator),
    /// CREATE 语句验证器
    Create(CreateValidator),

    // DML 验证器
    /// USE 语句验证器
    Use(UseValidator),
    /// SET 语句验证器
    Set(SetValidator),
    /// ASSIGNMENT 语句验证器
    Assignment(AssignmentValidator),
    /// PIPE 语句验证器
    Pipe(PipeValidator),
    /// QUERY 语句验证器
    Query(QueryValidator),
    /// SET OPERATION 语句验证器
    SetOperation(SetOperationValidator),

    // 查询类验证器
    /// MATCH 语句验证器
    Match(MatchValidator),
    /// LOOKUP 语句验证器
    Lookup(LookupValidator),
    /// GO 语句验证器
    Go(GoValidator),
    /// FIND PATH 语句验证器
    FindPath(FindPathValidator),
    /// GET SUBGRAPH 语句验证器
    GetSubgraph(GetSubgraphValidator),
    /// FETCH VERTICES 语句验证器
    FetchVertices(FetchVerticesValidator),
    /// FETCH EDGES 语句验证器
    FetchEdges(FetchEdgesValidator),
    /// INSERT VERTICES 语句验证器
    InsertVertices(InsertVerticesValidator),
    /// INSERT EDGES 语句验证器
    InsertEdges(InsertEdgesValidator),
    /// UPDATE 语句验证器
    Update(UpdateValidator),
    /// DELETE 语句验证器
    Delete(DeleteValidator),
    /// MERGE 语句验证器
    Merge(MergeValidator),
    /// REMOVE 语句验证器
    Remove(RemoveValidator),
    /// UNWIND 语句验证器
    Unwind(UnwindValidator),

    // 子句类验证器
    /// ORDER BY 语句验证器
    OrderBy(OrderByValidator),
    /// GROUP BY 语句验证器
    GroupBy(GroupByValidator),
    /// YIELD 语句验证器
    Yield(YieldValidator),
    /// RETURN 语句验证器
    Return(ReturnValidator),
    /// WITH 语句验证器
    With(WithValidator),
    /// LIMIT 语句验证器
    Limit(LimitValidator),
    /// SEQUENTIAL 语句验证器
    Sequential(SequentialValidator),

    // 工具类验证器
    /// EXPLAIN 语句验证器
    Explain(ExplainValidator),
    /// PROFILE 语句验证器
    Profile(ProfileValidator),
    /// UPDATE CONFIG 语句验证器
    UpdateConfig(UpdateConfigsValidator),
}

impl Validator {
    /// 获取验证器类型
    pub fn get_type(&self) -> StatementType {
        match self {
            Validator::Show(v) => v.statement_type(),
            Validator::Desc(v) => v.statement_type(),
            Validator::ShowCreate(v) => v.statement_type(),
            Validator::ShowConfigs(v) => v.statement_type(),
            Validator::ShowSessions(v) => v.statement_type(),
            Validator::ShowQueries(v) => v.statement_type(),
            Validator::KillQuery(v) => v.statement_type(),
            Validator::CreateUser(v) => v.statement_type(),
            Validator::DropUser(v) => v.statement_type(),
            Validator::AlterUser(v) => v.statement_type(),
            Validator::ChangePassword(v) => v.statement_type(),
            Validator::Grant(v) => v.statement_type(),
            Validator::Revoke(v) => v.statement_type(),
            Validator::DescribeUser(v) => v.statement_type(),
            Validator::ShowUsers(v) => v.statement_type(),
            Validator::ShowRoles(v) => v.statement_type(),
            Validator::Alter(v) => v.statement_type(),
            Validator::Drop(v) => v.statement_type(),
            Validator::Create(v) => v.statement_type(),
            Validator::Use(v) => v.statement_type(),
            Validator::Set(v) => v.statement_type(),
            Validator::Assignment(v) => v.statement_type(),
            Validator::Pipe(v) => v.statement_type(),
            Validator::Query(v) => v.statement_type(),
            Validator::SetOperation(v) => v.statement_type(),
            Validator::Match(v) => v.statement_type(),
            Validator::Lookup(v) => v.statement_type(),
            Validator::Go(v) => v.statement_type(),
            Validator::FindPath(v) => v.statement_type(),
            Validator::GetSubgraph(v) => v.statement_type(),
            Validator::FetchVertices(v) => v.statement_type(),
            Validator::FetchEdges(v) => v.statement_type(),
            Validator::InsertVertices(v) => v.statement_type(),
            Validator::InsertEdges(v) => v.statement_type(),
            Validator::Update(v) => v.statement_type(),
            Validator::Delete(v) => v.statement_type(),
            Validator::Merge(v) => v.statement_type(),
            Validator::Remove(v) => v.statement_type(),
            Validator::Unwind(v) => v.statement_type(),
            Validator::OrderBy(v) => v.statement_type(),
            Validator::GroupBy(v) => v.statement_type(),
            Validator::Yield(v) => v.statement_type(),
            Validator::Return(v) => v.statement_type(),
            Validator::With(v) => v.statement_type(),
            Validator::Limit(v) => v.statement_type(),
            Validator::Sequential(v) => v.statement_type(),
            Validator::Explain(v) => v.statement_type(),
            Validator::Profile(v) => v.statement_type(),
            Validator::UpdateConfig(v) => v.statement_type(),
        }
    }

    /// 验证语句
    pub fn validate(
        &mut self,
        stmt: &Stmt,
        qctx: Arc<QueryContext>,
    ) -> ValidationResult {
        match self {
            Validator::Show(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::Desc(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::ShowCreate(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::ShowConfigs(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::ShowSessions(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::ShowQueries(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::KillQuery(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::CreateUser(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::DropUser(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::AlterUser(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::ChangePassword(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::Grant(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::Revoke(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::DescribeUser(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::ShowUsers(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::ShowRoles(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::Alter(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::Drop(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::Create(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::Use(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::Set(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::Assignment(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::Pipe(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::Query(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::SetOperation(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::Match(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::Lookup(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::Go(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::FindPath(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::GetSubgraph(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::FetchVertices(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::FetchEdges(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::InsertVertices(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::InsertEdges(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::Update(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::Delete(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::Merge(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::Remove(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::Unwind(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::OrderBy(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::GroupBy(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::Yield(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::Return(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::With(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::Limit(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::Sequential(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::Explain(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::Profile(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            Validator::UpdateConfig(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
        }
    }

    /// 获取输入列
    pub fn get_inputs(&self) -> Vec<ColumnDef> {
        match self {
            Validator::Show(v) => v.inputs().to_vec(),
            Validator::Desc(v) => v.inputs().to_vec(),
            Validator::ShowCreate(v) => v.inputs().to_vec(),
            Validator::ShowConfigs(v) => v.inputs().to_vec(),
            Validator::ShowSessions(v) => v.inputs().to_vec(),
            Validator::ShowQueries(v) => v.inputs().to_vec(),
            Validator::KillQuery(v) => v.inputs().to_vec(),
            Validator::CreateUser(v) => v.inputs().to_vec(),
            Validator::DropUser(v) => v.inputs().to_vec(),
            Validator::AlterUser(v) => v.inputs().to_vec(),
            Validator::ChangePassword(v) => v.inputs().to_vec(),
            Validator::Grant(v) => v.inputs().to_vec(),
            Validator::Revoke(v) => v.inputs().to_vec(),
            Validator::DescribeUser(v) => v.inputs().to_vec(),
            Validator::ShowUsers(v) => v.inputs().to_vec(),
            Validator::ShowRoles(v) => v.inputs().to_vec(),
            Validator::Alter(v) => v.inputs().to_vec(),
            Validator::Drop(v) => v.inputs().to_vec(),
            Validator::Create(v) => v.inputs().to_vec(),
            Validator::Use(v) => v.inputs().to_vec(),
            Validator::Set(v) => v.inputs().to_vec(),
            Validator::Assignment(v) => v.inputs().to_vec(),
            Validator::Pipe(v) => v.inputs().to_vec(),
            Validator::Query(v) => v.inputs().to_vec(),
            Validator::SetOperation(v) => v.inputs().to_vec(),
            Validator::Match(v) => v.inputs().to_vec(),
            Validator::Lookup(v) => v.inputs().to_vec(),
            Validator::Go(v) => v.inputs().to_vec(),
            Validator::FindPath(v) => v.inputs().to_vec(),
            Validator::GetSubgraph(v) => v.inputs().to_vec(),
            Validator::FetchVertices(v) => v.inputs().to_vec(),
            Validator::FetchEdges(v) => v.inputs().to_vec(),
            Validator::InsertVertices(v) => v.inputs().to_vec(),
            Validator::InsertEdges(v) => v.inputs().to_vec(),
            Validator::Update(v) => v.inputs().to_vec(),
            Validator::Delete(v) => v.inputs().to_vec(),
            Validator::Merge(v) => v.inputs().to_vec(),
            Validator::Remove(v) => v.inputs().to_vec(),
            Validator::Unwind(v) => v.inputs().to_vec(),
            Validator::OrderBy(v) => v.inputs().to_vec(),
            Validator::GroupBy(v) => v.inputs().to_vec(),
            Validator::Yield(v) => v.inputs().to_vec(),
            Validator::Return(v) => v.inputs().to_vec(),
            Validator::With(v) => v.inputs().to_vec(),
            Validator::Limit(v) => v.inputs().to_vec(),
            Validator::Sequential(v) => v.inputs().to_vec(),
            Validator::Explain(v) => v.inputs().to_vec(),
            Validator::Profile(v) => v.inputs().to_vec(),
            Validator::UpdateConfig(v) => v.inputs().to_vec(),
        }
    }

    /// 获取输出列
    pub fn get_outputs(&self) -> Vec<ColumnDef> {
        match self {
            Validator::Show(v) => v.outputs().to_vec(),
            Validator::Desc(v) => v.outputs().to_vec(),
            Validator::ShowCreate(v) => v.outputs().to_vec(),
            Validator::ShowConfigs(v) => v.outputs().to_vec(),
            Validator::ShowSessions(v) => v.outputs().to_vec(),
            Validator::ShowQueries(v) => v.outputs().to_vec(),
            Validator::KillQuery(v) => v.outputs().to_vec(),
            Validator::CreateUser(v) => v.outputs().to_vec(),
            Validator::DropUser(v) => v.outputs().to_vec(),
            Validator::AlterUser(v) => v.outputs().to_vec(),
            Validator::ChangePassword(v) => v.outputs().to_vec(),
            Validator::Grant(v) => v.outputs().to_vec(),
            Validator::Revoke(v) => v.outputs().to_vec(),
            Validator::DescribeUser(v) => v.outputs().to_vec(),
            Validator::ShowUsers(v) => v.outputs().to_vec(),
            Validator::ShowRoles(v) => v.outputs().to_vec(),
            Validator::Alter(v) => v.outputs().to_vec(),
            Validator::Drop(v) => v.outputs().to_vec(),
            Validator::Create(v) => v.outputs().to_vec(),
            Validator::Use(v) => v.outputs().to_vec(),
            Validator::Set(v) => v.outputs().to_vec(),
            Validator::Assignment(v) => v.outputs().to_vec(),
            Validator::Pipe(v) => v.outputs().to_vec(),
            Validator::Query(v) => v.outputs().to_vec(),
            Validator::SetOperation(v) => v.outputs().to_vec(),
            Validator::Match(v) => v.outputs().to_vec(),
            Validator::Lookup(v) => v.outputs().to_vec(),
            Validator::Go(v) => v.outputs().to_vec(),
            Validator::FindPath(v) => v.outputs().to_vec(),
            Validator::GetSubgraph(v) => v.outputs().to_vec(),
            Validator::FetchVertices(v) => v.outputs().to_vec(),
            Validator::FetchEdges(v) => v.outputs().to_vec(),
            Validator::InsertVertices(v) => v.outputs().to_vec(),
            Validator::InsertEdges(v) => v.outputs().to_vec(),
            Validator::Update(v) => v.outputs().to_vec(),
            Validator::Delete(v) => v.outputs().to_vec(),
            Validator::Merge(v) => v.outputs().to_vec(),
            Validator::Remove(v) => v.outputs().to_vec(),
            Validator::Unwind(v) => v.outputs().to_vec(),
            Validator::OrderBy(v) => v.outputs().to_vec(),
            Validator::GroupBy(v) => v.outputs().to_vec(),
            Validator::Yield(v) => v.outputs().to_vec(),
            Validator::Return(v) => v.outputs().to_vec(),
            Validator::With(v) => v.outputs().to_vec(),
            Validator::Limit(v) => v.outputs().to_vec(),
            Validator::Sequential(v) => v.outputs().to_vec(),
            Validator::Explain(v) => v.outputs().to_vec(),
            Validator::Profile(v) => v.outputs().to_vec(),
            Validator::UpdateConfig(v) => v.outputs().to_vec(),
        }
    }
}

impl Validator {
    /// 根据语句创建验证器
    pub fn create_from_stmt(stmt: &Stmt) -> Option<Validator> {
        let stmt_type = Self::infer_statement_type(stmt);
        Some(Self::create(stmt_type))
    }

    /// 从语句推断语句类型
    fn infer_statement_type(stmt: &Stmt) -> StatementType {
        match stmt {
            Stmt::Query(_) => StatementType::Query,
            Stmt::Match(_) => StatementType::Match,
            Stmt::Delete(_) => StatementType::Delete,
            Stmt::Update(_) => StatementType::Update,
            Stmt::Go(_) => StatementType::Go,
            Stmt::Fetch(f) => match &f.target {
                FetchTarget::Vertices { .. } => StatementType::FetchVertices,
                FetchTarget::Edges { .. } => StatementType::FetchEdges,
            },
            Stmt::Use(_) => StatementType::Use,
            Stmt::Show(_) => StatementType::Show,
            Stmt::Explain(_) => StatementType::Explain,
            Stmt::Profile(_) => StatementType::Profile,
            Stmt::GroupBy(_) => StatementType::GroupBy,
            Stmt::Lookup(_) => StatementType::Lookup,
            Stmt::Subgraph(_) => StatementType::GetSubgraph,
            Stmt::FindPath(_) => StatementType::FindPath,
            Stmt::Insert(_) => StatementType::InsertVertices,
            Stmt::Merge(_) => StatementType::Merge,
            Stmt::Unwind(_) => StatementType::Unwind,
            Stmt::Return(_) => StatementType::Return,
            Stmt::With(_) => StatementType::With,
            Stmt::Yield(_) => StatementType::Yield,
            Stmt::Set(_) => StatementType::Set,
            Stmt::Remove(_) => StatementType::Remove,
            Stmt::Pipe(_) => StatementType::Pipe,
            Stmt::Drop(_) => StatementType::Drop,
            Stmt::Desc(_) => StatementType::Desc,
            Stmt::Alter(_) => StatementType::Alter,
            Stmt::CreateUser(_) => StatementType::CreateUser,
            Stmt::AlterUser(_) => StatementType::AlterUser,
            Stmt::DropUser(_) => StatementType::DropUser,
            Stmt::ChangePassword(_) => StatementType::ChangePassword,
            Stmt::Grant(_) => StatementType::Grant,
            Stmt::Revoke(_) => StatementType::Revoke,
            Stmt::DescribeUser(_) => StatementType::DescribeUser,
            Stmt::ShowUsers(_) => StatementType::ShowUsers,
            Stmt::ShowRoles(_) => StatementType::ShowRoles,
            Stmt::ShowSpaces(_) => StatementType::ShowSpaces,
            Stmt::ShowTags(_) => StatementType::ShowTags,
            Stmt::ShowEdges(_) => StatementType::ShowEdges,
            Stmt::ShowCreate(_) => StatementType::ShowCreate,
            Stmt::ShowConfigs(_) => StatementType::ShowConfigs,
            Stmt::ShowSessions(_) => StatementType::ShowSessions,
            Stmt::ShowQueries(_) => StatementType::ShowQueries,
            Stmt::KillQuery(_) => StatementType::KillQuery,
            Stmt::Create(c) => match &c.target {
                CreateTarget::Space { .. } => StatementType::CreateSpace,
                CreateTarget::Tag { .. } => StatementType::CreateTag,
                CreateTarget::Edge { .. } => StatementType::CreateEdge,
                _ => StatementType::Create,
            },
            Stmt::Assignment(_) => StatementType::Assignment,
            Stmt::SetOperation(_) => StatementType::SetOperation,
            Stmt::UpdateConfigs(_) => StatementType::UpdateConfigs,
        }
    }

    /// 根据语句类型创建验证器
    pub fn create(stmt_type: StatementType) -> Validator {
        match stmt_type {
            StatementType::Show => Validator::Show(ShowValidator::new()),
            StatementType::Desc => Validator::Desc(DescValidator::new()),
            StatementType::ShowCreate => Validator::ShowCreate(ShowCreateValidator::new()),
            StatementType::ShowConfigs => Validator::ShowConfigs(ShowConfigsValidator::new()),
            StatementType::ShowSessions => Validator::ShowSessions(ShowSessionsValidator::new()),
            StatementType::ShowQueries => Validator::ShowQueries(ShowQueriesValidator::new()),
            StatementType::KillQuery => Validator::KillQuery(KillQueryValidator::new()),
            StatementType::CreateUser => Validator::CreateUser(CreateUserValidator::new()),
            StatementType::DropUser => Validator::DropUser(DropUserValidator::new()),
            StatementType::AlterUser => Validator::AlterUser(AlterUserValidator::new()),
            StatementType::ChangePassword => Validator::ChangePassword(ChangePasswordValidator::new()),
            StatementType::Grant => Validator::Grant(GrantValidator::new()),
            StatementType::Revoke => Validator::Revoke(RevokeValidator::new()),
            StatementType::DescribeUser => Validator::DescribeUser(DescribeUserValidator::new()),
            StatementType::ShowUsers => Validator::ShowUsers(ShowUsersValidator::new()),
            StatementType::ShowRoles => Validator::ShowRoles(ShowRolesValidator::new()),
            StatementType::Alter => Validator::Alter(AlterValidator::new()),
            StatementType::Drop => Validator::Drop(DropValidator::new()),
            StatementType::Create => Validator::Create(CreateValidator::new()),
            StatementType::Use => Validator::Use(UseValidator::new()),
            StatementType::Set => Validator::Set(SetValidator::new()),
            StatementType::Assignment => Validator::Assignment(AssignmentValidator::new()),
            StatementType::Pipe => Validator::Pipe(PipeValidator::new()),
            StatementType::Query => Validator::Query(QueryValidator::new()),
            StatementType::SetOperation => Validator::SetOperation(SetOperationValidator::new()),
            StatementType::Match => Validator::Match(MatchValidator::new()),
            StatementType::Lookup => Validator::Lookup(LookupValidator::new()),
            StatementType::Go => Validator::Go(GoValidator::new()),
            StatementType::FindPath => Validator::FindPath(FindPathValidator::new()),
            StatementType::GetSubgraph => Validator::GetSubgraph(GetSubgraphValidator::new()),
            StatementType::FetchVertices => Validator::FetchVertices(FetchVerticesValidator::new()),
            StatementType::FetchEdges => Validator::FetchEdges(FetchEdgesValidator::new()),
            StatementType::InsertVertices => Validator::InsertVertices(InsertVerticesValidator::new()),
            StatementType::InsertEdges => Validator::InsertEdges(InsertEdgesValidator::new()),
            StatementType::Update => Validator::Update(UpdateValidator::new()),
            StatementType::Delete => Validator::Delete(DeleteValidator::new()),
            StatementType::Merge => Validator::Merge(MergeValidator::new()),
            StatementType::Remove => Validator::Remove(RemoveValidator::new()),
            StatementType::Unwind => Validator::Unwind(UnwindValidator::new()),
            StatementType::OrderBy => Validator::OrderBy(OrderByValidator::new()),
            StatementType::GroupBy => Validator::GroupBy(GroupByValidator::new()),
            StatementType::Yield => Validator::Yield(YieldValidator::new()),
            StatementType::Return => Validator::Return(ReturnValidator::new()),
            StatementType::With => Validator::With(WithValidator::new()),
            StatementType::Limit => Validator::Limit(LimitValidator::new()),
            StatementType::Sequential => Validator::Sequential(SequentialValidator::new()),
            StatementType::Explain => Validator::Explain(ExplainValidator::new()),
            StatementType::Profile => Validator::Profile(ProfileValidator::new()),
            StatementType::UpdateConfigs => Validator::UpdateConfig(UpdateConfigsValidator::new()),
            _ => panic!("Unknown statement type: {:?}", stmt_type),
        }
    }

    /// 获取用户定义变量列表
    pub fn get_user_defined_vars(&self) -> &[String] {
        match self {
            Validator::Show(v) => v.user_defined_vars(),
            Validator::Desc(v) => v.user_defined_vars(),
            Validator::ShowCreate(v) => v.user_defined_vars(),
            Validator::ShowConfigs(v) => v.user_defined_vars(),
            Validator::ShowSessions(v) => v.user_defined_vars(),
            Validator::ShowQueries(v) => v.user_defined_vars(),
            Validator::KillQuery(v) => v.user_defined_vars(),
            Validator::CreateUser(v) => v.user_defined_vars(),
            Validator::DropUser(v) => v.user_defined_vars(),
            Validator::AlterUser(v) => v.user_defined_vars(),
            Validator::ChangePassword(v) => v.user_defined_vars(),
            Validator::Grant(v) => v.user_defined_vars(),
            Validator::Revoke(v) => v.user_defined_vars(),
            Validator::DescribeUser(v) => v.user_defined_vars(),
            Validator::ShowUsers(v) => v.user_defined_vars(),
            Validator::ShowRoles(v) => v.user_defined_vars(),
            Validator::Alter(v) => v.user_defined_vars(),
            Validator::Drop(v) => v.user_defined_vars(),
            Validator::Create(v) => v.user_defined_vars(),
            Validator::Use(v) => v.user_defined_vars(),
            Validator::Set(v) => v.user_defined_vars(),
            Validator::Assignment(v) => v.user_defined_vars(),
            Validator::Pipe(v) => v.user_defined_vars(),
            Validator::Query(v) => v.user_defined_vars(),
            Validator::SetOperation(v) => v.user_defined_vars(),
            Validator::Match(v) => v.user_defined_vars(),
            Validator::Lookup(v) => v.user_defined_vars(),
            Validator::Go(v) => v.user_defined_vars(),
            Validator::FindPath(v) => v.user_defined_vars(),
            Validator::GetSubgraph(v) => v.user_defined_vars(),
            Validator::FetchVertices(v) => v.user_defined_vars(),
            Validator::FetchEdges(v) => v.user_defined_vars(),
            Validator::InsertVertices(v) => v.user_defined_vars(),
            Validator::InsertEdges(v) => v.user_defined_vars(),
            Validator::Update(v) => v.user_defined_vars(),
            Validator::Delete(v) => v.user_defined_vars(),
            Validator::Merge(v) => v.user_defined_vars(),
            Validator::Remove(v) => v.user_defined_vars(),
            Validator::Unwind(v) => v.user_defined_vars(),
            Validator::OrderBy(v) => v.user_defined_vars(),
            Validator::GroupBy(v) => v.user_defined_vars(),
            Validator::Yield(v) => v.user_defined_vars(),
            Validator::Return(v) => v.user_defined_vars(),
            Validator::With(v) => v.user_defined_vars(),
            Validator::Limit(v) => v.user_defined_vars(),
            Validator::Sequential(v) => v.user_defined_vars(),
            Validator::Explain(v) => v.user_defined_vars(),
            Validator::Profile(v) => v.user_defined_vars(),
            Validator::UpdateConfig(v) => v.user_defined_vars(),
        }
    }

    /// 获取语句类型
    pub fn statement_type(&self) -> StatementType {
        self.get_type()
    }

    /// 获取表达式属性
    pub fn expression_props(&self) -> &ExpressionProps {
        match self {
            Validator::Show(v) => v.expression_props(),
            Validator::Desc(v) => v.expression_props(),
            Validator::ShowCreate(v) => v.expression_props(),
            Validator::ShowConfigs(v) => v.expression_props(),
            Validator::ShowSessions(v) => v.expression_props(),
            Validator::ShowQueries(v) => v.expression_props(),
            Validator::KillQuery(v) => v.expression_props(),
            Validator::CreateUser(v) => v.expression_props(),
            Validator::DropUser(v) => v.expression_props(),
            Validator::AlterUser(v) => v.expression_props(),
            Validator::ChangePassword(v) => v.expression_props(),
            Validator::Grant(v) => v.expression_props(),
            Validator::Revoke(v) => v.expression_props(),
            Validator::DescribeUser(v) => v.expression_props(),
            Validator::ShowUsers(v) => v.expression_props(),
            Validator::ShowRoles(v) => v.expression_props(),
            Validator::Alter(v) => v.expression_props(),
            Validator::Drop(v) => v.expression_props(),
            Validator::Create(v) => v.expression_props(),
            Validator::Use(v) => v.expression_props(),
            Validator::Set(v) => v.expression_props(),
            Validator::Assignment(v) => v.expression_props(),
            Validator::Pipe(v) => v.expression_props(),
            Validator::Query(v) => v.expression_props(),
            Validator::SetOperation(v) => v.expression_props(),
            Validator::Match(v) => v.expression_props(),
            Validator::Lookup(v) => v.expression_props(),
            Validator::Go(v) => v.expression_props(),
            Validator::FindPath(v) => v.expression_props(),
            Validator::GetSubgraph(v) => v.expression_props(),
            Validator::FetchVertices(v) => v.expression_props(),
            Validator::FetchEdges(v) => v.expression_props(),
            Validator::InsertVertices(v) => v.expression_props(),
            Validator::InsertEdges(v) => v.expression_props(),
            Validator::Update(v) => v.expression_props(),
            Validator::Delete(v) => v.expression_props(),
            Validator::Merge(v) => v.expression_props(),
            Validator::Remove(v) => v.expression_props(),
            Validator::Unwind(v) => v.expression_props(),
            Validator::OrderBy(v) => v.expression_props(),
            Validator::GroupBy(v) => v.expression_props(),
            Validator::Yield(v) => v.expression_props(),
            Validator::Return(v) => v.expression_props(),
            Validator::With(v) => v.expression_props(),
            Validator::Limit(v) => v.expression_props(),
            Validator::Sequential(v) => v.expression_props(),
            Validator::Explain(v) => v.expression_props(),
            Validator::Profile(v) => v.expression_props(),
            Validator::UpdateConfig(v) => v.expression_props(),
        }
    }
}

/// 验证器集合
pub struct ValidatorCollection {
    validators: Vec<Validator>,
}

impl ValidatorCollection {
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
        }
    }

    pub fn add(&mut self, validator: Validator) {
        self.validators.push(validator);
    }

    pub fn get_validators(&self) -> &[Validator] {
        &self.validators
    }

    pub fn get_validators_mut(&mut self) -> &mut Vec<Validator> {
        &mut self.validators
    }
}

impl Default for ValidatorCollection {
    fn default() -> Self {
        Self::new()
    }
}
