//! 规划器注册机制
//! 使用类型安全的枚举实现静态注册，完全消除动态分发
//!
//! # 重构说明
//!
//! 本模块已完全重构，删除了旧的 SentenceKind 字符串匹配机制。
//! 现在使用直接的枚举模式匹配从 Stmt 创建规划器。

use std::sync::Arc;

use crate::query::parser::ast::Stmt;
use crate::query::planner::plan::ExecutionPlan;
use crate::query::planner::plan::SubPlan;
use crate::query::QueryContext;

// 公开导出 ValidatedStatement，供 planner 实现使用
pub use crate::query::validator::ValidatedStatement;

use crate::query::planner::rewrite::{rewrite_plan, RewriteError};
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
    pub max_plan_depth: usize,
    pub enable_parallel_planning: bool,
    pub enable_rewrite: bool,
}

impl Default for PlannerConfig {
    fn default() -> Self {
        Self {
            max_plan_depth: 100,
            enable_parallel_planning: false,
            enable_rewrite: true,
        }
    }
}

/// 匹配函数类型
pub type MatchFunc = fn(&Stmt) -> bool;

/// 规划器特征
///
/// # 设计原则
/// - transform 方法接收 Arc<QueryContext> 和 &ValidatedStatement
/// - match_planner 方法接收 &Stmt 用于匹配判断
pub trait Planner: std::fmt::Debug {
    /// 转换验证后的语句为执行子计划
    ///
    /// # 参数
    /// - `validated`: 验证后的语句，包含 ValidationInfo 和 Ast
    /// - `qctx`: 查询上下文
    fn transform(
        &mut self,
        validated: &ValidatedStatement,
        qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError>;

    /// 检查此规划器是否能处理给定的语句
    fn match_planner(&self, stmt: &Stmt) -> bool;

    /// 使用验证后的语句进行完整转换
    fn transform_with_full_context(
        &mut self,
        qctx: Arc<QueryContext>,
        validated: &ValidatedStatement,
    ) -> Result<ExecutionPlan, PlannerError> {
        let sub_plan = self.transform(validated, qctx)?;
        let plan = ExecutionPlan::new(sub_plan.root().clone());

        // 应用计划重写优化
        let plan = rewrite_plan(plan)?;

        Ok(plan)
    }

    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

// ============================================================================
// 静态注册实现 - 完全消除动态分发
// ============================================================================

/// 规划器枚举 - 静态分发核心
/// 完全消除动态分发，使用编译时多态
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
    /// 直接从 Arc<Stmt> 创建规划器（推荐方式）
    /// 使用枚举模式匹配，完全消除字符串匹配
    pub fn from_stmt(stmt: &Arc<Stmt>) -> Option<Self> {
        match stmt.as_ref() {
            Stmt::Match(_) => Some(PlannerEnum::Match(MatchStatementPlanner::new())),
            Stmt::Go(_) => Some(PlannerEnum::Go(GoPlanner::new())),
            Stmt::Lookup(_) => Some(PlannerEnum::Lookup(LookupPlanner::new())),
            Stmt::FindPath(_) => Some(PlannerEnum::Path(PathPlanner::new())),
            Stmt::Subgraph(_) => Some(PlannerEnum::Subgraph(SubgraphPlanner::new())),
            Stmt::Fetch(fetch_stmt) => match &fetch_stmt.target {
                crate::query::parser::ast::FetchTarget::Vertices { .. } => {
                    Some(PlannerEnum::FetchVertices(FetchVerticesPlanner::new()))
                }
                crate::query::parser::ast::FetchTarget::Edges { .. } => {
                    Some(PlannerEnum::FetchEdges(FetchEdgesPlanner::new()))
                }
            },
            Stmt::Insert(_) => Some(PlannerEnum::Insert(InsertPlanner::new())),
            Stmt::Delete(_) => Some(PlannerEnum::Delete(DeletePlanner::new())),
            Stmt::Update(_) => Some(PlannerEnum::Update(UpdatePlanner::new())),
            Stmt::GroupBy(_) => Some(PlannerEnum::GroupBy(GroupByPlanner::new())),
            Stmt::SetOperation(_) => Some(PlannerEnum::SetOperation(SetOperationPlanner::new())),
            Stmt::Use(_) => Some(PlannerEnum::Use(UsePlanner::new())),
            // DDL/DML 操作使用 Maintain 规划器
            Stmt::Create(_)
            | Stmt::Drop(_)
            | Stmt::Show(_)
            | Stmt::Desc(_)
            | Stmt::Alter(_)
            | Stmt::CreateUser(_)
            | Stmt::DropUser(_)
            | Stmt::AlterUser(_)
            | Stmt::ChangePassword(_)
            | Stmt::Grant(_)
            | Stmt::Revoke(_)
            | Stmt::DescribeUser(_)
            | Stmt::ShowUsers(_)
            | Stmt::ShowRoles(_)
            | Stmt::ShowCreate(_)
            | Stmt::ShowSessions(_)
            | Stmt::ShowQueries(_)
            | Stmt::KillQuery(_)
            | Stmt::ShowConfigs(_)
            | Stmt::UpdateConfigs(_) => Some(PlannerEnum::Maintain(MaintainPlanner::new())),
            // 以下语句类型暂不支持直接规划
            _ => None,
        }
    }

    /// 从 Arc<Ast> 创建规划器
    /// 这是新的推荐方式，表达式上下文在 Ast 中
    pub fn from_ast(ast: &Arc<crate::query::parser::ast::stmt::Ast>) -> Option<Self> {
        Self::from_stmt(&Arc::new(ast.stmt.clone()))
    }

    /// 将验证后的语句转换为执行计划
    pub fn transform(
        &mut self,
        validated: &ValidatedStatement,
        qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        match self {
            PlannerEnum::Match(planner) => planner.transform(validated, qctx),
            PlannerEnum::Go(planner) => planner.transform(validated, qctx),
            PlannerEnum::Lookup(planner) => planner.transform(validated, qctx),
            PlannerEnum::Path(planner) => planner.transform(validated, qctx),
            PlannerEnum::Subgraph(planner) => planner.transform(validated, qctx),
            PlannerEnum::FetchVertices(planner) => planner.transform(validated, qctx),
            PlannerEnum::FetchEdges(planner) => planner.transform(validated, qctx),
            PlannerEnum::Maintain(planner) => planner.transform(validated, qctx),
            PlannerEnum::UserManagement(planner) => planner.transform(validated, qctx),
            PlannerEnum::Insert(planner) => planner.transform(validated, qctx),
            PlannerEnum::Delete(planner) => planner.transform(validated, qctx),
            PlannerEnum::Update(planner) => planner.transform(validated, qctx),
            PlannerEnum::GroupBy(planner) => planner.transform(validated, qctx),
            PlannerEnum::SetOperation(planner) => planner.transform(validated, qctx),
            PlannerEnum::Use(planner) => planner.transform(validated, qctx),
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
}

/// 错误处理宏
#[macro_export]
macro_rules! ng_return_if_error {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => return Err(e.into()),
        }
    };
}

/// 错误处理宏变体
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

// 为 DBError 实现 From 转换
impl From<crate::core::error::DBError> for PlannerError {
    fn from(err: crate::core::error::DBError) -> Self {
        PlannerError::PlanGenerationFailed(err.to_string())
    }
}

// 为 RewriteError 实现 From 转换
impl From<RewriteError> for PlannerError {
    fn from(err: RewriteError) -> Self {
        PlannerError::PlanGenerationFailed(format!("Plan rewrite failed: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planner_enum_from_stmt() {
        // 测试从 Stmt 创建规划器
        let match_stmt = Stmt::Match(crate::query::parser::ast::MatchStmt {
            span: crate::core::types::Span::default(),
            patterns: vec![],
            where_clause: None,
            return_clause: None,
            order_by: None,
            limit: None,
            skip: None,
            optional: false,
        });

        let planner = PlannerEnum::from_stmt(&Arc::new(match_stmt));
        assert!(planner.is_some());
        assert_eq!(planner.unwrap().name(), "MatchPlanner");
    }

    #[test]
    fn test_planner_enum_matches() {
        let match_stmt = Stmt::Match(crate::query::parser::ast::MatchStmt {
            span: crate::core::types::Span::default(),
            patterns: vec![],
            where_clause: None,
            return_clause: None,
            order_by: None,
            limit: None,
            skip: None,
            optional: false,
        });

        let planner = PlannerEnum::Match(MatchStatementPlanner::new());
        assert!(planner.matches(&match_stmt));
    }
}
