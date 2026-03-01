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
use crate::query::parser::ast::Stmt;
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
            Validator::Show(v) => v.validate(stmt, qctx),
            Validator::Desc(v) => v.validate(stmt, qctx),
            Validator::ShowCreate(v) => v.validate(stmt, qctx),
            Validator::ShowConfigs(v) => v.validate(stmt, qctx),
            Validator::ShowSessions(v) => v.validate(stmt, qctx),
            Validator::ShowQueries(v) => v.validate(stmt, qctx),
            Validator::KillQuery(v) => v.validate(stmt, qctx),
            Validator::CreateUser(v) => v.validate(stmt, qctx),
            Validator::DropUser(v) => v.validate(stmt, qctx),
            Validator::AlterUser(v) => v.validate(stmt, qctx),
            Validator::ChangePassword(v) => v.validate(stmt, qctx),
            Validator::Grant(v) => v.validate(stmt, qctx),
            Validator::Revoke(v) => v.validate(stmt, qctx),
            Validator::DescribeUser(v) => v.validate(stmt, qctx),
            Validator::ShowUsers(v) => v.validate(stmt, qctx),
            Validator::ShowRoles(v) => v.validate(stmt, qctx),
            Validator::Alter(v) => v.validate(stmt, qctx),
            Validator::Drop(v) => v.validate(stmt, qctx),
            Validator::Create(v) => v.validate(stmt, qctx),
            Validator::Use(v) => v.validate(stmt, qctx),
            Validator::Set(v) => v.validate(stmt, qctx),
            Validator::Assignment(v) => v.validate(stmt, qctx),
            Validator::Pipe(v) => v.validate(stmt, qctx),
            Validator::Query(v) => v.validate(stmt, qctx),
            Validator::SetOperation(v) => v.validate(stmt, qctx),
            Validator::Match(v) => v.validate(stmt, qctx),
            Validator::Lookup(v) => v.validate(stmt, qctx),
            Validator::Go(v) => v.validate(stmt, qctx),
            Validator::FindPath(v) => v.validate(stmt, qctx),
            Validator::GetSubgraph(v) => v.validate(stmt, qctx),
            Validator::FetchVertices(v) => v.validate(stmt, qctx),
            Validator::FetchEdges(v) => v.validate(stmt, qctx),
            Validator::InsertVertices(v) => v.validate(stmt, qctx),
            Validator::InsertEdges(v) => v.validate(stmt, qctx),
            Validator::Update(v) => v.validate(stmt, qctx),
            Validator::Delete(v) => v.validate(stmt, qctx),
            Validator::Merge(v) => v.validate(stmt, qctx),
            Validator::Remove(v) => v.validate(stmt, qctx),
            Validator::Unwind(v) => v.validate(stmt, qctx),
            Validator::OrderBy(v) => v.validate(stmt, qctx),
            Validator::GroupBy(v) => v.validate(stmt, qctx),
            Validator::Yield(v) => v.validate(stmt, qctx),
            Validator::Return(v) => v.validate(stmt, qctx),
            Validator::With(v) => v.validate(stmt, qctx),
            Validator::Limit(v) => v.validate(stmt, qctx),
            Validator::Sequential(v) => v.validate(stmt, qctx),
            Validator::Explain(v) => v.validate(stmt, qctx),
            Validator::Profile(v) => v.validate(stmt, qctx),
            Validator::UpdateConfig(v) => v.validate(stmt, qctx),
        }
    }

    /// 获取输入列
    pub fn get_inputs(&self) -> Vec<ColumnDef> {
        match self {
            Validator::Show(v) => v.inputs(),
            Validator::Desc(v) => v.inputs(),
            Validator::ShowCreate(v) => v.inputs(),
            Validator::ShowConfigs(v) => v.inputs(),
            Validator::ShowSessions(v) => v.inputs(),
            Validator::ShowQueries(v) => v.inputs(),
            Validator::KillQuery(v) => v.inputs(),
            Validator::CreateUser(v) => v.inputs(),
            Validator::DropUser(v) => v.inputs(),
            Validator::AlterUser(v) => v.inputs(),
            Validator::ChangePassword(v) => v.inputs(),
            Validator::Grant(v) => v.inputs(),
            Validator::Revoke(v) => v.inputs(),
            Validator::DescribeUser(v) => v.inputs(),
            Validator::ShowUsers(v) => v.inputs(),
            Validator::ShowRoles(v) => v.inputs(),
            Validator::Alter(v) => v.inputs(),
            Validator::Drop(v) => v.inputs(),
            Validator::Create(v) => v.inputs(),
            Validator::Use(v) => v.inputs(),
            Validator::Set(v) => v.inputs(),
            Validator::Assignment(v) => v.inputs(),
            Validator::Pipe(v) => v.inputs(),
            Validator::Query(v) => v.inputs(),
            Validator::SetOperation(v) => v.inputs(),
            Validator::Match(v) => v.inputs(),
            Validator::Lookup(v) => v.inputs(),
            Validator::Go(v) => v.inputs(),
            Validator::FindPath(v) => v.inputs(),
            Validator::GetSubgraph(v) => v.inputs(),
            Validator::FetchVertices(v) => v.inputs(),
            Validator::FetchEdges(v) => v.inputs(),
            Validator::InsertVertices(v) => v.inputs(),
            Validator::InsertEdges(v) => v.inputs(),
            Validator::Update(v) => v.inputs(),
            Validator::Delete(v) => v.inputs(),
            Validator::Merge(v) => v.inputs(),
            Validator::Remove(v) => v.inputs(),
            Validator::Unwind(v) => v.inputs(),
            Validator::OrderBy(v) => v.inputs(),
            Validator::GroupBy(v) => v.inputs(),
            Validator::Yield(v) => v.inputs(),
            Validator::Return(v) => v.inputs(),
            Validator::With(v) => v.inputs(),
            Validator::Limit(v) => v.inputs(),
            Validator::Sequential(v) => v.inputs(),
            Validator::Explain(v) => v.inputs(),
            Validator::Profile(v) => v.inputs(),
            Validator::UpdateConfig(v) => v.inputs(),
        }
    }

    /// 获取输出列
    pub fn get_outputs(&self) -> Vec<ColumnDef> {
        match self {
            Validator::Show(v) => v.outputs(),
            Validator::Desc(v) => v.outputs(),
            Validator::ShowCreate(v) => v.outputs(),
            Validator::ShowConfigs(v) => v.outputs(),
            Validator::ShowSessions(v) => v.outputs(),
            Validator::ShowQueries(v) => v.outputs(),
            Validator::KillQuery(v) => v.outputs(),
            Validator::CreateUser(v) => v.outputs(),
            Validator::DropUser(v) => v.outputs(),
            Validator::AlterUser(v) => v.outputs(),
            Validator::ChangePassword(v) => v.outputs(),
            Validator::Grant(v) => v.outputs(),
            Validator::Revoke(v) => v.outputs(),
            Validator::DescribeUser(v) => v.outputs(),
            Validator::ShowUsers(v) => v.outputs(),
            Validator::ShowRoles(v) => v.outputs(),
            Validator::Alter(v) => v.outputs(),
            Validator::Drop(v) => v.outputs(),
            Validator::Create(v) => v.outputs(),
            Validator::Use(v) => v.outputs(),
            Validator::Set(v) => v.outputs(),
            Validator::Assignment(v) => v.outputs(),
            Validator::Pipe(v) => v.outputs(),
            Validator::Query(v) => v.outputs(),
            Validator::SetOperation(v) => v.outputs(),
            Validator::Match(v) => v.outputs(),
            Validator::Lookup(v) => v.outputs(),
            Validator::Go(v) => v.outputs(),
            Validator::FindPath(v) => v.outputs(),
            Validator::GetSubgraph(v) => v.outputs(),
            Validator::FetchVertices(v) => v.outputs(),
            Validator::FetchEdges(v) => v.outputs(),
            Validator::InsertVertices(v) => v.outputs(),
            Validator::InsertEdges(v) => v.outputs(),
            Validator::Update(v) => v.outputs(),
            Validator::Delete(v) => v.outputs(),
            Validator::Merge(v) => v.outputs(),
            Validator::Remove(v) => v.outputs(),
            Validator::Unwind(v) => v.outputs(),
            Validator::OrderBy(v) => v.outputs(),
            Validator::GroupBy(v) => v.outputs(),
            Validator::Yield(v) => v.outputs(),
            Validator::Return(v) => v.outputs(),
            Validator::With(v) => v.outputs(),
            Validator::Limit(v) => v.outputs(),
            Validator::Sequential(v) => v.outputs(),
            Validator::Explain(v) => v.outputs(),
            Validator::Profile(v) => v.outputs(),
            Validator::UpdateConfig(v) => v.outputs(),
        }
    }
}

/// 验证器工厂
pub struct ValidatorFactory;

impl ValidatorFactory {
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
            StatementType::UpdateConfig => Validator::UpdateConfig(UpdateConfigsValidator::new()),
            _ => panic!("Unknown statement type: {:?}", stmt_type),
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
