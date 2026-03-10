//! 查询执行管理器
//!
//! 管理查询执行过程中的执行计划和终止信号。

use crate::query::planner::plan::ExecutionPlan;
use std::sync::atomic::{AtomicBool, Ordering};

/// 查询执行管理器
///
/// 管理查询执行过程中的关键信息，包括：
/// - 执行计划
/// - 是否被终止
/// - 其他执行相关的管理功能
pub struct QueryExecutionManager {
    /// 执行计划
    plan: Option<Box<ExecutionPlan>>,

    /// 是否被标记为已终止
    killed: AtomicBool,
}

impl QueryExecutionManager {
    /// 创建新的执行管理器
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
        log::info!("Query execution manager marked as killed");
    }

    /// 检查是否被终止
    pub fn is_killed(&self) -> bool {
        self.killed.load(Ordering::SeqCst)
    }

    /// 重置执行管理器
    pub fn reset(&mut self) {
        self.plan = None;
        self.killed.store(false, Ordering::SeqCst);
        log::info!("Query execution manager reset");
    }
}

impl Default for QueryExecutionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for QueryExecutionManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryExecutionManager")
            .field("plan_id", &self.plan_id())
            .field("killed", &self.killed)
            .finish()
    }
}
