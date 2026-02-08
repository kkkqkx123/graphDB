pub mod async_scheduler;
pub mod execution_schedule;
pub mod execution_plan_analyzer;
pub mod types;

pub use types::{ExecutorDep, ExecutionEvent, ExecutorType, QueryScheduler, SchedulerConfig, VariableLifetime};

pub use execution_schedule::ExecutionSchedule;
pub use execution_plan_analyzer::{ExecutionPlanAnalysis, ExecutionPlanAnalyzer};

pub use async_scheduler::{AsyncMsgNotifyBasedScheduler, SchedulerExecutionState};

/// 已废弃：请使用 `SchedulerExecutionState`
#[deprecated(since = "0.1.0", note = "请使用 SchedulerExecutionState")]
pub use async_scheduler::ExecutionState;
