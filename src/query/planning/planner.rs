//! Planner registration mechanism
//! Implement static registration using type-safe enumerations to completely eliminate dynamic distribution.
//!
//! # Explanation of the reconstruction process
//!
//! This module has been completely restructured, and the old mechanism for matching SentenceKind strings has been removed.
//! Now, use the direct enumeration mode to match the planner created from the Stmt.

use std::sync::Arc;

use crate::query::parser::ast::Stmt;
use crate::query::planning::plan::ExecutionPlan;
use crate::query::planning::plan::SubPlan;
use crate::query::QueryContext;

// The ValidatedStatement is publicly exported for use by the planner implementation.
pub use crate::query::validator::ValidatedStatement;

use crate::query::planning::rewrite::{rewrite_plan, RewriteError};
use crate::query::planning::statements::ddl::maintain_planner::MaintainPlanner;
use crate::query::planning::statements::ddl::use_planner::UsePlanner;
use crate::query::planning::statements::ddl::user_management_planner::UserManagementPlanner;
use crate::query::planning::statements::dml::delete_planner::DeletePlanner;
use crate::query::planning::statements::dml::insert_planner::InsertPlanner;
use crate::query::planning::statements::dml::merge_planner::MergePlanner;
use crate::query::planning::statements::dml::remove_planner::RemovePlanner;
use crate::query::planning::statements::dml::update_planner::UpdatePlanner;
use crate::query::planning::statements::dql::fetch_edges_planner::FetchEdgesPlanner;
use crate::query::planning::statements::dql::fetch_vertices_planner::FetchVerticesPlanner;
use crate::query::planning::statements::dql::go_planner::GoPlanner;
use crate::query::planning::statements::dql::group_by_planner::GroupByPlanner;
use crate::query::planning::statements::dql::lookup_planner::LookupPlanner;
use crate::query::planning::statements::dql::path_planner::PathPlanner;
use crate::query::planning::statements::dql::return_planner::ReturnPlanner;
use crate::query::planning::statements::dql::set_operation_planner::SetOperationPlanner;
use crate::query::planning::statements::dql::subgraph_planner::SubgraphPlanner;
use crate::query::planning::statements::dql::with_planner::WithPlanner;
use crate::query::planning::statements::dql::yield_planner::YieldPlanner;
use crate::query::planning::statements::match_statement_planner::MatchStatementPlanner;

///  Planner Configuration
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

/// Match function type
pub type MatchFunc = fn(&Stmt) -> bool;

///  Planner Features
///
/// # Design Principles
/// The `transform` method accepts an `Arc<QueryContext>` and a `&ValidatedStatement`.
/// The `match_planner` method receives an `&Stmt` object, which is used for matching and making judgments.
pub trait Planner: std::fmt::Debug {
    /// Translate the verified sentence into English: “Execute the sub-plan.”
    ///
    /// # Parameters
    /// Validated: A verified statement that contains ValidationInfo and Ast.
    /// `qctx`: Query context
    fn transform(
        &mut self,
        validated: &ValidatedStatement,
        qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError>;

    /// Check whether this planner can handle the given sentence.
    fn match_planner(&self, stmt: &Stmt) -> bool;

    /// Use the verified statements to complete the translation.
    fn transform_with_full_context(
        &mut self,
        qctx: Arc<QueryContext>,
        validated: &ValidatedStatement,
    ) -> Result<ExecutionPlan, PlannerError> {
        let sub_plan = self.transform(validated, qctx)?;
        let plan = ExecutionPlan::new(sub_plan.root().clone());

        // Application plan rewrite and optimization
        let plan = rewrite_plan(plan)?;

        Ok(plan)
    }

    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

// ============================================================================
// Implementation of static registration – complete elimination of dynamic distribution
// ============================================================================

/// Planner Enumeration – Core for Static Distribution
/// Eliminate dynamic distribution completely and use compile-time polymorphism instead.
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
    Remove(RemovePlanner),
    Merge(MergePlanner),
    GroupBy(GroupByPlanner),
    SetOperation(SetOperationPlanner),
    Use(UsePlanner),
    With(WithPlanner),
    Return(ReturnPlanner),
    Yield(YieldPlanner),
}

impl PlannerEnum {
    /// Create a planner directly from Arc<Stmt> (the recommended method).
    /// Use the enumeration pattern for matching to completely eliminate the need for string matching.
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
            Stmt::Remove(_) => Some(PlannerEnum::Remove(RemovePlanner::new())),
            Stmt::Merge(_) => Some(PlannerEnum::Merge(MergePlanner::new())),
            Stmt::GroupBy(_) => Some(PlannerEnum::GroupBy(GroupByPlanner::new())),
            Stmt::SetOperation(_) => Some(PlannerEnum::SetOperation(SetOperationPlanner::new())),
            Stmt::Use(_) => Some(PlannerEnum::Use(UsePlanner::new())),
            Stmt::With(_) => Some(PlannerEnum::With(WithPlanner::new())),
            Stmt::Return(_) => Some(PlannerEnum::Return(ReturnPlanner::new())),
            Stmt::Yield(_) => Some(PlannerEnum::Yield(YieldPlanner::new())),
            // DDL/DML operations use the Maintain planner.
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
            | Stmt::UpdateConfigs(_)
            | Stmt::ClearSpace(_) => Some(PlannerEnum::Maintain(MaintainPlanner::new())),
            // The type of the following sentence does not currently support direct planning.
            _ => None,
        }
    }

    /// Create a planner from Arc<Ast>.
    /// This is the new recommendation method; the context of the expressions is defined within Ast.
    pub fn from_ast(ast: &Arc<crate::query::parser::ast::stmt::Ast>) -> Option<Self> {
        Self::from_stmt(&Arc::new(ast.stmt.clone()))
    }

    /// Convert the verified statement into an execution plan.
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
            PlannerEnum::Remove(planner) => planner.transform(validated, qctx),
            PlannerEnum::Merge(planner) => planner.transform(validated, qctx),
            PlannerEnum::GroupBy(planner) => planner.transform(validated, qctx),
            PlannerEnum::SetOperation(planner) => planner.transform(validated, qctx),
            PlannerEnum::Use(planner) => planner.transform(validated, qctx),
            PlannerEnum::With(planner) => planner.transform(validated, qctx),
            PlannerEnum::Return(planner) => planner.transform(validated, qctx),
            PlannerEnum::Yield(planner) => planner.transform(validated, qctx),
        }
    }

    /// Obtain the name of the planner.
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
            PlannerEnum::Remove(_) => "RemovePlanner",
            PlannerEnum::Merge(_) => "MergePlanner",
            PlannerEnum::GroupBy(_) => "GroupByPlanner",
            PlannerEnum::SetOperation(_) => "SetOperationPlanner",
            PlannerEnum::Use(_) => "UsePlanner",
            PlannerEnum::With(_) => "WithPlanner",
            PlannerEnum::Return(_) => "ReturnPlanner",
            PlannerEnum::Yield(_) => "YieldPlanner",
        }
    }

    /// Check whether there is a match.
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
            PlannerEnum::Remove(planner) => planner.match_planner(stmt),
            PlannerEnum::Merge(planner) => planner.match_planner(stmt),
            PlannerEnum::GroupBy(planner) => planner.match_planner(stmt),
            PlannerEnum::SetOperation(planner) => planner.match_planner(stmt),
            PlannerEnum::Use(planner) => planner.match_planner(stmt),
            PlannerEnum::With(planner) => planner.match_planner(stmt),
            PlannerEnum::Return(planner) => planner.match_planner(stmt),
            PlannerEnum::Yield(planner) => planner.match_planner(stmt),
        }
    }
}

/// Error handling macros
#[macro_export]
macro_rules! ng_return_if_error {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => return Err(e.into()),
        }
    };
}

/// Error handling macro variants
#[macro_export]
macro_rules! ng_ok_or_err {
    ($expr:expr, $msg:expr) => {
        match $expr {
            Ok(val) => val,
            Err(_) => return Err(PlannerError::PlanGenerationFailed($msg.to_string())),
        }
    };
}

/// Planner error type
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

// Implement the From conversion for the DBError class.
impl From<crate::core::error::DBError> for PlannerError {
    fn from(err: crate::core::error::DBError) -> Self {
        PlannerError::PlanGenerationFailed(err.to_string())
    }
}

// Implement the From conversion for the RewriteError
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
        // Testing the creation of a planner from a Stmt
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
        assert_eq!(planner.expect("Planner should exist").name(), "MatchPlanner");
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
