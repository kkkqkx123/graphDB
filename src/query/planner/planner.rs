//! 规划器注册机制
//! 使用类型安全的枚举实现静态注册，完全消除动态分发
//!
//! # 重构变更
//! - 使用 Arc<QueryContext> 替代 &QueryContext
//! - 使用 &Stmt 替代 &AstContext

use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;

use crate::query::QueryContext;
use crate::query::parser::ast::Stmt;
use crate::query::planner::plan::ExecutionPlan;
use crate::query::planner::plan::SubPlan;
use crate::query::validator::StatementType;
use lru::LruCache;
use parking_lot::Mutex;

use crate::query::planner::statements::delete_planner::DeletePlanner;
use crate::query::planner::statements::fetch_edges_planner::FetchEdgesPlanner;
use crate::query::planner::statements::fetch_vertices_planner::FetchVerticesPlanner;
use crate::query::planner::statements::go_planner::GoPlanner;
use crate::query::planner::statements::group_by_planner::GroupByPlanner;
use crate::query::planner::statements::insert_planner::InsertPlanner;
use crate::query::planner::statements::lookup_planner::LookupPlanner;
use crate::query::planner::statements::maintain_planner::MaintainPlanner;
use crate::query::planner::statements::match_statement_planner::MatchStatementPlanner;
use crate::query::planner::statements::path_planner::PathPlanner;
use crate::query::planner::statements::set_operation_planner::SetOperationPlanner;
use crate::query::planner::statements::subgraph_planner::SubgraphPlanner;
use crate::query::planner::statements::update_planner::UpdatePlanner;
use crate::query::planner::statements::use_planner::UsePlanner;
use crate::query::planner::statements::user_management_planner::UserManagementPlanner;

/// 规划器配置
#[derive(Debug, Clone)]
pub struct PlannerConfig {
    pub enable_caching: bool,
    pub max_plan_depth: usize,
    pub enable_parallel_planning: bool,
    pub default_timeout: Duration,
    pub cache_size: usize,
}

impl Default for PlannerConfig {
    fn default() -> Self {
        Self {
            enable_caching: true,
            max_plan_depth: 100,
            enable_parallel_planning: false,
            default_timeout: Duration::from_secs(30),
            cache_size: 1000,
        }
    }
}

/// 计划缓存键
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct PlanCacheKey {
    query_text: String,
    space_id: Option<i32>,
    statement_type: String,
}

impl PlanCacheKey {
    pub fn new(query_text: String, space_id: Option<i32>, statement_type: String) -> Self {
        Self {
            query_text,
            space_id,
            statement_type,
        }
    }
}

/// 计划缓存
#[derive(Debug)]
pub struct PlanCache {
    cache: Mutex<LruCache<PlanCacheKey, ExecutionPlan>>,
}

impl PlanCache {
    pub fn new(max_size: usize) -> Result<Self, PlannerError> {
        if max_size == 0 {
            return Err(PlannerError::InvalidOperation(
                "Plan cache size must be greater than 0".to_string(),
            ));
        }
        let cache_size = NonZeroUsize::new(max_size)
            .ok_or_else(|| PlannerError::InvalidOperation(
                "Failed to create plan cache with size".to_string(),
            ))?;
        Ok(Self {
            cache: Mutex::new(LruCache::new(cache_size)),
        })
    }

    pub fn get(&self, key: &PlanCacheKey) -> Result<Option<ExecutionPlan>, PlannerError> {
        let mut cache = self.cache.lock();
        Ok(cache.get(key).cloned())
    }

    pub fn insert(&self, key: PlanCacheKey, plan: ExecutionPlan) -> Result<(), PlannerError> {
        let mut cache = self.cache.lock();
        cache.push(key, plan);
        Ok(())
    }

    pub fn clear(&self) -> Result<(), PlannerError> {
        let mut cache = self.cache.lock();
        cache.clear();
        Ok(())
    }

    pub fn size(&self) -> Result<usize, PlannerError> {
        let cache = self.cache.lock();
        Ok(cache.len())
    }
}

/// 语句类型枚举（替代字符串）
#[derive(Debug, Clone, PartialEq, Hash, Eq, Copy)]
pub enum SentenceKind {
    Match,
    Go,
    Lookup,
    Path,
    Subgraph,
    FetchVertices,
    FetchEdges,
    Maintain,
    UserManagement,
    Create,
    Drop,
    Use,
    Show,
    Desc,
    Insert,
    Delete,
    Update,
    GroupBy,
    SetOperation,
}

impl SentenceKind {
    /// 从字符串解析语句类型
    pub fn from_str(s: &str) -> Result<Self, PlannerError> {
        match s.to_uppercase().as_str() {
            "MATCH" => Ok(SentenceKind::Match),
            "GO" => Ok(SentenceKind::Go),
            "LOOKUP" => Ok(SentenceKind::Lookup),
            "PATH" | "FIND PATH" => Ok(SentenceKind::Path),
            "SUBGRAPH" => Ok(SentenceKind::Subgraph),
            "FETCH VERTICES" => Ok(SentenceKind::FetchVertices),
            "FETCH EDGES" => Ok(SentenceKind::FetchEdges),
            "MAINTAIN" => Ok(SentenceKind::Maintain),
            "CREATE_USER" | "ALTER_USER" | "DROP_USER" | "CHANGE_PASSWORD" |
            "CREATE USER" | "ALTER USER" | "DROP USER" | "CHANGE PASSWORD" => {
                Ok(SentenceKind::UserManagement)
            }
            "CREATE" => Ok(SentenceKind::Create),
            "DROP" => Ok(SentenceKind::Drop),
            "USE" => Ok(SentenceKind::Use),
            "DELETE" => Ok(SentenceKind::Delete),
            "UPDATE" => Ok(SentenceKind::Update),
            "GROUP BY" => Ok(SentenceKind::GroupBy),
            "SET OPERATION" | "UNION" | "UNION ALL" | "INTERSECT" | "MINUS" => Ok(SentenceKind::SetOperation),
            "SHOW" => Ok(SentenceKind::Show),
            "DESC" => Ok(SentenceKind::Desc),
            "INSERT" | "INSERT VERTEX" | "INSERT EDGE" => Ok(SentenceKind::Insert),
            _ => Err(PlannerError::UnsupportedOperation(format!(
                "Unsupported statement type: {}",
                s
            ))),
        }
    }

    /// 从语句枚举解析语句类型
    pub fn from_stmt(stmt: &Stmt) -> Result<Self, PlannerError> {
        match stmt.kind().to_uppercase().as_str() {
            "MATCH" => Ok(SentenceKind::Match),
            "GO" => Ok(SentenceKind::Go),
            "LOOKUP" => Ok(SentenceKind::Lookup),
            "FIND PATH" => Ok(SentenceKind::Path),
            "SUBGRAPH" => Ok(SentenceKind::Subgraph),
            "FETCH" => {
                if let Stmt::Fetch(fetch_stmt) = stmt {
                    match &fetch_stmt.target {
                        crate::query::parser::ast::FetchTarget::Vertices { .. } => Ok(SentenceKind::FetchVertices),
                        crate::query::parser::ast::FetchTarget::Edges { .. } => Ok(SentenceKind::FetchEdges),
                    }
                } else {
                    Err(PlannerError::UnsupportedOperation("Invalid FETCH statement".to_string()))
                }
            }
            "CREATE USER" | "ALTER USER" | "DROP USER" | "CHANGE PASSWORD" => Ok(SentenceKind::UserManagement),
            "CREATE" => Ok(SentenceKind::Create),
            "DROP" => Ok(SentenceKind::Drop),
            "USE" => Ok(SentenceKind::Use),
            "DELETE" => Ok(SentenceKind::Delete),
            "UPDATE" => Ok(SentenceKind::Update),
            "GROUP BY" => Ok(SentenceKind::GroupBy),
            "SET OPERATION" => Ok(SentenceKind::SetOperation),
            "SHOW" => Ok(SentenceKind::Show),
            "DESC" => Ok(SentenceKind::Desc),
            "INSERT" => Ok(SentenceKind::Insert),
            _ => Err(PlannerError::UnsupportedOperation(format!(
                "Unsupported statement type: {}",
                stmt.kind()
            ))),
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            SentenceKind::Match => "MATCH",
            SentenceKind::Go => "GO",
            SentenceKind::Lookup => "LOOKUP",
            SentenceKind::Path => "PATH",
            SentenceKind::Subgraph => "SUBGRAPH",
            SentenceKind::FetchVertices => "FETCH VERTICES",
            SentenceKind::FetchEdges => "FETCH EDGES",
            SentenceKind::Maintain => "MAINTAIN",
            SentenceKind::UserManagement => "USER_MANAGEMENT",
            SentenceKind::Create => "CREATE",
            SentenceKind::Drop => "DROP",
            SentenceKind::Use => "USE",
            SentenceKind::Delete => "DELETE",
            SentenceKind::Update => "UPDATE",
            SentenceKind::GroupBy => "GROUP BY",
            SentenceKind::SetOperation => "SET OPERATION",
            SentenceKind::Show => "SHOW",
            SentenceKind::Desc => "DESC",
            SentenceKind::Insert => "INSERT",
        }
    }

    /// 从 StatementType 转换到 SentenceKind
    /// 建立验证层和规划层之间的显式映射关系
    pub fn from_statement_type(stmt_type: &StatementType) -> Option<Self> {
        match stmt_type {
            StatementType::Match => Some(SentenceKind::Match),
            StatementType::Go => Some(SentenceKind::Go),
            StatementType::Lookup => Some(SentenceKind::Lookup),
            StatementType::FindPath => Some(SentenceKind::Path),
            StatementType::GetSubgraph => Some(SentenceKind::Subgraph),
            StatementType::FetchVertices => Some(SentenceKind::FetchVertices),
            StatementType::FetchEdges => Some(SentenceKind::FetchEdges),
            // INSERT 语句映射到 Insert
            StatementType::InsertVertices |
            StatementType::InsertEdges => Some(SentenceKind::Insert),
            // DELETE 和 UPDATE 有独立的规划器
            StatementType::Delete => Some(SentenceKind::Delete),
            StatementType::Update => Some(SentenceKind::Update),
            // GROUP BY 有独立的规划器
            StatementType::GroupBy => Some(SentenceKind::GroupBy),
            // USE 有独立的规划器
            StatementType::Use => Some(SentenceKind::Use),
            // 集合操作有独立的规划器
            StatementType::SetOperation => Some(SentenceKind::SetOperation),
            // 其他DDL和DML操作映射到 Maintain
            StatementType::Create |
            StatementType::CreateSpace |
            StatementType::CreateTag |
            StatementType::CreateEdge |
            StatementType::Alter |
            StatementType::AlterTag |
            StatementType::AlterEdge |
            StatementType::Drop |
            StatementType::DropSpace |
            StatementType::DropTag |
            StatementType::DropEdge |
            StatementType::DescribeSpace |
            StatementType::DescribeTag |
            StatementType::DescribeEdge |
            StatementType::ShowSpaces |
            StatementType::ShowTags |
            StatementType::ShowEdges => Some(SentenceKind::Maintain),
            // 以下类型没有对应的规划器，返回 None
            StatementType::Unwind |
            StatementType::Yield |
            StatementType::OrderBy |
            StatementType::Limit |
            StatementType::Assignment |
            StatementType::Set |
            StatementType::Pipe |
            StatementType::Sequential |
            StatementType::Explain |
            StatementType::Profile |
            StatementType::Query |
            StatementType::Merge |
            StatementType::Return |
            StatementType::With |
            StatementType::Remove |
            StatementType::UpdateConfigs |
            StatementType::Show |
            StatementType::Desc |
            StatementType::ShowCreate |
            StatementType::ShowConfigs |
            StatementType::ShowSessions |
            StatementType::ShowQueries |
            StatementType::KillQuery |
            StatementType::CreateUser |
            StatementType::DropUser |
            StatementType::AlterUser |
            StatementType::Grant |
            StatementType::Revoke |
            StatementType::ChangePassword |
            StatementType::DescribeUser |
            StatementType::ShowUsers |
            StatementType::ShowRoles => None,
        }
    }
}

/// 匹配函数类型
///
/// # 重构变更
/// - 使用 &Stmt 替代 &AstContext
pub type MatchFunc = fn(&Stmt) -> bool;

/// 静态匹配和实例化枚举 - 完全消除动态分发
#[derive(Debug, Clone)]
pub enum MatchAndInstantiateEnum {
    Match(MatchStatementPlanner),
    Go(GoPlanner),
    Lookup(LookupPlanner),
    Path(PathPlanner),
    Subgraph(SubgraphPlanner),
    FetchVertices(FetchVerticesPlanner),
    FetchEdges(FetchEdgesPlanner),
    Maintain(MaintainPlanner),
    UserManagement(UserManagementPlanner),
    Insert(InsertPlanner),
    Delete(DeletePlanner),
    Update(UpdatePlanner),
    GroupBy(GroupByPlanner),
    SetOperation(SetOperationPlanner),
    Use(UsePlanner),
}

impl MatchAndInstantiateEnum {
    pub fn priority(&self) -> i32 {
        match self {
            MatchAndInstantiateEnum::Match(_) => 100,
            MatchAndInstantiateEnum::Go(_) => 100,
            MatchAndInstantiateEnum::Lookup(_) => 100,
            MatchAndInstantiateEnum::Path(_) => 100,
            MatchAndInstantiateEnum::Subgraph(_) => 100,
            MatchAndInstantiateEnum::FetchVertices(_) => 100,
            MatchAndInstantiateEnum::FetchEdges(_) => 100,
            MatchAndInstantiateEnum::Maintain(_) => 100,
            MatchAndInstantiateEnum::UserManagement(_) => 100,
            MatchAndInstantiateEnum::Insert(_) => 100,
            MatchAndInstantiateEnum::Delete(_) => 100,
            MatchAndInstantiateEnum::Update(_) => 100,
            MatchAndInstantiateEnum::GroupBy(_) => 100,
            MatchAndInstantiateEnum::SetOperation(_) => 100,
            MatchAndInstantiateEnum::Use(_) => 100,
        }
    }

    /// # 重构变更
    /// - 使用 &Stmt 替代 &AstContext
    pub fn transform(
        &mut self,
        stmt: &Stmt,
        qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        match self {
            MatchAndInstantiateEnum::Match(planner) => planner.transform(stmt, qctx),
            MatchAndInstantiateEnum::Go(planner) => planner.transform(stmt, qctx),
            MatchAndInstantiateEnum::Lookup(planner) => planner.transform(stmt, qctx),
            MatchAndInstantiateEnum::Path(planner) => planner.transform(stmt, qctx),
            MatchAndInstantiateEnum::Subgraph(planner) => planner.transform(stmt, qctx),
            MatchAndInstantiateEnum::FetchVertices(planner) => planner.transform(stmt, qctx),
            MatchAndInstantiateEnum::FetchEdges(planner) => planner.transform(stmt, qctx),
            MatchAndInstantiateEnum::Maintain(planner) => planner.transform(stmt, qctx),
            MatchAndInstantiateEnum::UserManagement(planner) => planner.transform(stmt, qctx),
            MatchAndInstantiateEnum::Insert(planner) => planner.transform(stmt, qctx),
            MatchAndInstantiateEnum::Delete(planner) => planner.transform(stmt, qctx),
            MatchAndInstantiateEnum::Update(planner) => planner.transform(stmt, qctx),
            MatchAndInstantiateEnum::GroupBy(planner) => planner.transform(stmt, qctx),
            MatchAndInstantiateEnum::SetOperation(planner) => planner.transform(stmt, qctx),
            MatchAndInstantiateEnum::Use(planner) => planner.transform(stmt, qctx),
        }
    }

    /// # 重构变更
    /// - 使用 Arc<QueryContext> 替代 &mut QueryContext
    /// - 使用 &Stmt 替代 &AstContext
    pub fn transform_with_full_context(
        &mut self,
        qctx: Arc<QueryContext>,
        stmt: &Stmt,
    ) -> Result<ExecutionPlan, PlannerError> {
        let sub_plan = self.transform(stmt, qctx)?;
        Ok(ExecutionPlan::new(sub_plan.root().clone()))
    }
}

/// 规划器特征（重构后接口）
///
/// # 重构变更
/// - transform 方法接收 Arc<QueryContext> 和 &Stmt 替代 &AstContext
/// - match_planner 方法接收 &Stmt 替代 &AstContext
pub trait Planner: std::fmt::Debug {
    fn transform(
        &mut self,
        stmt: &Stmt,
        qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError>;
    fn match_planner(&self, stmt: &Stmt) -> bool;

    fn transform_with_full_context(
        &mut self,
        qctx: Arc<QueryContext>,
        stmt: &Stmt,
    ) -> Result<ExecutionPlan, PlannerError> {
        let sub_plan = self.transform(stmt, qctx)?;
        Ok(ExecutionPlan::new(sub_plan.root().clone()))
    }

    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

// ============================================================================
// 静态注册实现 - 完全消除动态分发
// ============================================================================

/// 规划器枚举 - 静态分发核心
/// 完全消除 Box<dyn Planner> 动态分发，使用编译时多态
#[derive(Debug, Clone)]
pub enum PlannerEnum {
    Match(MatchStatementPlanner),
    Go(GoPlanner),
    Lookup(LookupPlanner),
    Path(PathPlanner),
    Subgraph(SubgraphPlanner),
    FetchVertices(FetchVerticesPlanner),
    FetchEdges(FetchEdgesPlanner),
    Maintain(MaintainPlanner),
    UserManagement(UserManagementPlanner),
    Insert(InsertPlanner),
    Delete(DeletePlanner),
    Update(UpdatePlanner),
    GroupBy(GroupByPlanner),
    SetOperation(SetOperationPlanner),
    Use(UsePlanner),
}

impl PlannerEnum {
    /// 根据语句类型创建规划器
    pub fn from_sentence_kind(kind: SentenceKind) -> Option<Self> {
        match kind {
            SentenceKind::Match => Some(PlannerEnum::Match(MatchStatementPlanner::new())),
            SentenceKind::Go => Some(PlannerEnum::Go(GoPlanner::new())),
            SentenceKind::Lookup => Some(PlannerEnum::Lookup(LookupPlanner::new())),
            SentenceKind::Path => Some(PlannerEnum::Path(PathPlanner::new())),
            SentenceKind::Subgraph => Some(PlannerEnum::Subgraph(SubgraphPlanner::new())),
            SentenceKind::FetchVertices => Some(PlannerEnum::FetchVertices(FetchVerticesPlanner::new())),
            SentenceKind::FetchEdges => Some(PlannerEnum::FetchEdges(FetchEdgesPlanner::new())),
            SentenceKind::Maintain => Some(PlannerEnum::Maintain(MaintainPlanner::new())),
            SentenceKind::UserManagement => Some(PlannerEnum::UserManagement(UserManagementPlanner::new())),
            SentenceKind::Insert => Some(PlannerEnum::Insert(InsertPlanner::new())),
            SentenceKind::Delete => Some(PlannerEnum::Delete(DeletePlanner::new())),
            SentenceKind::Update => Some(PlannerEnum::Update(UpdatePlanner::new())),
            SentenceKind::GroupBy => Some(PlannerEnum::GroupBy(GroupByPlanner::new())),
            SentenceKind::SetOperation => Some(PlannerEnum::SetOperation(SetOperationPlanner::new())),
            SentenceKind::Use => Some(PlannerEnum::Use(UsePlanner::new())),
            // DDL/DML 操作使用 Maintain 规划器
            SentenceKind::Create | SentenceKind::Drop | SentenceKind::Show | SentenceKind::Desc => {
                Some(PlannerEnum::Maintain(MaintainPlanner::new()))
            }
        }
    }

    /// 将语句转换为执行计划
    pub fn transform(&mut self, stmt: &Stmt, qctx: Arc<QueryContext>) -> Result<SubPlan, PlannerError> {
        match self {
            PlannerEnum::Match(planner) => planner.transform(stmt, qctx),
            PlannerEnum::Go(planner) => planner.transform(stmt, qctx),
            PlannerEnum::Lookup(planner) => planner.transform(stmt, qctx),
            PlannerEnum::Path(planner) => planner.transform(stmt, qctx),
            PlannerEnum::Subgraph(planner) => planner.transform(stmt, qctx),
            PlannerEnum::FetchVertices(planner) => planner.transform(stmt, qctx),
            PlannerEnum::FetchEdges(planner) => planner.transform(stmt, qctx),
            PlannerEnum::Maintain(planner) => planner.transform(stmt, qctx),
            PlannerEnum::UserManagement(planner) => planner.transform(stmt, qctx),
            PlannerEnum::Insert(planner) => planner.transform(stmt, qctx),
            PlannerEnum::Delete(planner) => planner.transform(stmt, qctx),
            PlannerEnum::Update(planner) => planner.transform(stmt, qctx),
            PlannerEnum::GroupBy(planner) => planner.transform(stmt, qctx),
            PlannerEnum::SetOperation(planner) => planner.transform(stmt, qctx),
            PlannerEnum::Use(planner) => planner.transform(stmt, qctx),
        }
    }

    /// 获取规划器名称
    pub fn name(&self) -> &'static str {
        match self {
            PlannerEnum::Match(_) => "MatchPlanner",
            PlannerEnum::Go(_) => "GoPlanner",
            PlannerEnum::Lookup(_) => "LookupPlanner",
            PlannerEnum::Path(_) => "PathPlanner",
            PlannerEnum::Subgraph(_) => "SubgraphPlanner",
            PlannerEnum::FetchVertices(_) => "FetchVerticesPlanner",
            PlannerEnum::FetchEdges(_) => "FetchEdgesPlanner",
            PlannerEnum::Maintain(_) => "MaintainPlanner",
            PlannerEnum::UserManagement(_) => "UserManagementPlanner",
            PlannerEnum::Insert(_) => "InsertPlanner",
            PlannerEnum::Delete(_) => "DeletePlanner",
            PlannerEnum::Update(_) => "UpdatePlanner",
            PlannerEnum::GroupBy(_) => "GroupByPlanner",
            PlannerEnum::SetOperation(_) => "SetOperationPlanner",
            PlannerEnum::Use(_) => "UsePlanner",
        }
    }

    /// 检查是否匹配
    pub fn matches(&self, stmt: &Stmt) -> bool {
        match self {
            PlannerEnum::Match(planner) => planner.match_planner(stmt),
            PlannerEnum::Go(planner) => planner.match_planner(stmt),
            PlannerEnum::Lookup(planner) => planner.match_planner(stmt),
            PlannerEnum::Path(planner) => planner.match_planner(stmt),
            PlannerEnum::Subgraph(planner) => planner.match_planner(stmt),
            PlannerEnum::FetchVertices(planner) => planner.match_planner(stmt),
            PlannerEnum::FetchEdges(planner) => planner.match_planner(stmt),
            PlannerEnum::Maintain(planner) => planner.match_planner(stmt),
            PlannerEnum::UserManagement(planner) => planner.match_planner(stmt),
            PlannerEnum::Insert(planner) => planner.match_planner(stmt),
            PlannerEnum::Delete(planner) => planner.match_planner(stmt),
            PlannerEnum::Update(planner) => planner.match_planner(stmt),
            PlannerEnum::GroupBy(planner) => planner.match_planner(stmt),
            PlannerEnum::SetOperation(planner) => planner.match_planner(stmt),
            PlannerEnum::Use(planner) => planner.match_planner(stmt),
        }
    }

    /// 转换为动态分发类型（用于向后兼容）
    pub fn into_dynamic(self) -> Box<dyn Planner> {
        match self {
            PlannerEnum::Match(planner) => Box::new(planner),
            PlannerEnum::Go(planner) => Box::new(planner),
            PlannerEnum::Lookup(planner) => Box::new(planner),
            PlannerEnum::Path(planner) => Box::new(planner),
            PlannerEnum::Subgraph(planner) => Box::new(planner),
            PlannerEnum::FetchVertices(planner) => Box::new(planner),
            PlannerEnum::FetchEdges(planner) => Box::new(planner),
            PlannerEnum::Maintain(planner) => Box::new(planner),
            PlannerEnum::UserManagement(planner) => Box::new(planner),
            PlannerEnum::Insert(planner) => Box::new(planner),
            PlannerEnum::Delete(planner) => Box::new(planner),
            PlannerEnum::Update(planner) => Box::new(planner),
            PlannerEnum::GroupBy(planner) => Box::new(planner),
            PlannerEnum::SetOperation(planner) => Box::new(planner),
            PlannerEnum::Use(planner) => Box::new(planner),
        }
    }
}

/// 错误处理宏
///
/// 类似于 C++ 中的 NG_RETURN_IF_ERROR 宏，用于简化错误传播
#[macro_export]
macro_rules! ng_return_if_error {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => return Err(e.into()),
        }
    };
}

/// 错误处理宏变体，返回默认错误消息
///
/// 当表达式返回错误时，返回一个带有默认消息的 PlannerError
#[macro_export]
macro_rules! ng_ok_or_err {
    ($expr:expr, $msg:expr) => {
        match $expr {
            Ok(val) => val,
            Err(_) => return Err(PlannerError::PlanGenerationFailed($msg.to_string())),
        }
    };
}

/// 规划器错误类型
#[derive(Debug, thiserror::Error)]
pub enum PlannerError {
    #[error("No suitable planner found: {0}")]
    NoSuitablePlanner(String),

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("Plan generation failed: {0}")]
    PlanGenerationFailed(String),

    #[error("Join operation failed: {0}")]
    JoinFailed(String),

    #[error("Invalid AST context: {0}")]
    InvalidAstContext(String),

    #[error("Missing input: {0}")]
    MissingInput(String),

    #[error("Missing variable: {0}")]
    MissingVariable(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

// 为 DBError 实现 From 转换，以便在规划器中使用 ? 操作符
impl From<crate::core::error::DBError> for PlannerError {
    fn from(err: crate::core::error::DBError) -> Self {
        PlannerError::PlanGenerationFailed(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sentence_kind_from_str() {
        assert_eq!(
            SentenceKind::from_str("MATCH").expect("Expected successful parsing of 'MATCH'"),
            SentenceKind::Match
        );
        assert_eq!(
            SentenceKind::from_str("match").expect("Expected successful parsing of 'match'"),
            SentenceKind::Match
        );
        assert_eq!(
            SentenceKind::from_str("GO").expect("Expected successful parsing of 'GO'"),
            SentenceKind::Go
        );
        assert_eq!(
            SentenceKind::from_str("FETCH VERTICES")
                .expect("Expected successful parsing of 'FETCH VERTICES'"),
            SentenceKind::FetchVertices
        );

        assert!(SentenceKind::from_str("INVALID").is_err());
    }

    #[test]
    fn test_sentence_kind_as_str() {
        assert_eq!(SentenceKind::Match.as_str(), "MATCH");
        assert_eq!(SentenceKind::Go.as_str(), "GO");
        assert_eq!(SentenceKind::FetchVertices.as_str(), "FETCH VERTICES");
    }

    #[test]
    fn test_match_and_instantiate() {
        let mi = MatchAndInstantiateEnum::Match(MatchStatementPlanner::new());
        assert_eq!(mi.priority(), 100);
    }

    #[test]
    fn test_planner_enum_from_sentence_kind() {
        let planner = PlannerEnum::from_sentence_kind(SentenceKind::Match);
        assert!(planner.is_some());
    }
}
