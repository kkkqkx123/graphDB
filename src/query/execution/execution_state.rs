//! 查询执行状态
//!
//! 管理查询执行过程中的状态信息，包括执行计划、终止标志等。

use crate::query::planner::plan::ExecutionPlan;
use std::sync::atomic::{AtomicBool, Ordering};

/// 查询执行状态
///
/// 管理查询执行过程中的状态信息，包括：
/// - 执行计划
/// - 是否被终止
/// - 其他执行相关的状态
pub struct QueryExecutionState {
    /// 执行计划
    plan: Option<Box<ExecutionPlan>>,

    /// 是否被标记为已终止
    killed: AtomicBool,
}

impl QueryExecutionState {
    /// 创建新的执行状态
    pub fn new() -> Self {
        Self {
            plan: None,
            killed: AtomicBool::new(false),
        }
    }

    /// 获取执行计划
    pub fn plan(&self) -> Option<ExecutionPlan> {
        self.plan.as_ref().map(|p| *p.clone())
    }

    /// 设置执行计划
    pub fn set_plan(&mut self, plan: ExecutionPlan) {
        self.plan = Some(Box::new(plan));
    }

    /// 获取执行计划 ID
    pub fn plan_id(&self) -> Option<i64> {
        self.plan.as_ref().map(|p| p.id)
    }

    /// 标记为已终止
    pub fn mark_killed(&self) {
        self.killed.store(true, Ordering::SeqCst);
        log::info!("查询执行状态被标记为已终止");
    }

    /// 检查是否被终止
    pub fn is_killed(&self) -> bool {
        self.killed.load(Ordering::SeqCst)
    }

    /// 重置执行状态
    pub fn reset(&mut self) {
        self.plan = None;
        self.killed.store(false, Ordering::SeqCst);
        log::info!("查询执行状态已重置");
    }
}

impl Default for QueryExecutionState {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for QueryExecutionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryExecutionState")
            .field("plan_id", &self.plan_id())
            .field("killed", &self.killed)
            .finish()
    }
}
