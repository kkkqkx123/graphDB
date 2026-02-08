//! Planner module for generating execution plans from AST
//! Contains the Planner trait, ExecutionPlan structure, and various specific planners

// 核心模块
pub mod plan;
pub mod planner;
pub mod connector;
pub mod operation_kind_support;

// 按功能组织的模块
pub mod statements;

// 重新导出主要的类型
pub use plan::execution_plan::{ExecutionPlan, SubPlan};
pub use planner::{
    MatchAndInstantiateEnum, Planner, PlannerConfig, PlannerError,
    PlanCache, PlanCacheKey, SentenceKind,
};
pub use connector::SegmentsConnector;

// 从 core 模块重新导出 JoinType
pub use crate::core::types::JoinType;
pub use statements::{
    MatchStatementPlanner,
};

// 静态注册相关导出
pub use planner::{
    PlannerEnum, StaticConfigurablePlannerRegistry, StaticPlannerRegistry,
    StaticSequentialPlanner,
    create_planner, plan,
};

use std::sync::atomic::{AtomicI64, Ordering};

pub struct PlanIdGenerator {
    counter: AtomicI64,
}

impl PlanIdGenerator {
    pub fn instance() -> &'static Self {
        static INSTANCE: PlanIdGenerator = PlanIdGenerator {
            counter: AtomicI64::new(0),
        };
        &INSTANCE
    }

    pub fn next_id(&self) -> i64 {
        self.counter.fetch_add(1, Ordering::SeqCst)
    }
}
