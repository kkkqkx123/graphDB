//! 语句级规划器
//!
//! 提供语句级规划器的统一接口，处理完整语句的规划逻辑。
//! 架构：Planner trait -> StatementPlanner trait -> ClausePlanner
//!
//! ## 架构设计
//!
//! - **Planner**：基础 trait，定义规划器的通用接口
//! - **StatementPlanner**：语句级 trait，处理完整语句的规划
//! - **ClausePlanner**：子句级 trait，处理单个子句的规划

use crate::query::QueryContext;
use crate::query::parser::ast::Stmt;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::Planner;
use crate::query::validator::structs::CypherClauseKind;
use std::sync::Arc;

/// 语句级规划器 trait
///
/// 定义语句级规划器的统一接口，封装完整语句的规划逻辑。
/// 组合多个子句规划器来完成语句的规划。
pub trait StatementPlanner: Planner {
    /// 获取语句类型
    fn statement_type(&self) -> &'static str;

    /// 获取支持的子句类型列表
    fn supported_clause_kinds(&self) -> &[CypherClauseKind];
}

/// 子句级规划器 trait
///
/// 定义子句级规划器的统一接口，处理单个子句的规划逻辑。
pub trait ClausePlanner: std::fmt::Debug {
    /// 获取子句类型
    fn clause_kind(&self) -> CypherClauseKind;

    /// 转换子句为核心计划
    fn transform_clause(
        &self,
        qctx: Arc<QueryContext>,
        stmt: &Stmt,
        input_plan: SubPlan,
    ) -> Result<SubPlan, crate::query::planner::planner::PlannerError>;
}
