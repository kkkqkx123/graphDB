//! 统一执行状态枚举定义
//!
//! 此模块提供查询执行过程中的统一状态枚举，整合分散在各处的状态定义。
//! 采用分层状态机设计，区分不同层次的状态管理。

use std::fmt;

/// 查询执行状态 - 顶层执行流程状态
///
/// 表示整个查询执行的生命周期状态，用于查询流程管理。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QueryExecutionState {
    /// 查询已创建，等待执行
    Pending,
    /// 查询正在执行中
    Running,
    /// 查询执行完成
    Completed,
    /// 查询执行失败
    Failed,
    /// 查询被取消
    Cancelled,
    /// 查询执行超时
    Timeout,
}

impl QueryExecutionState {
    /// 检查状态是否为终态
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            QueryExecutionState::Completed
                | QueryExecutionState::Failed
                | QueryExecutionState::Cancelled
                | QueryExecutionState::Timeout
        )
    }

    /// 检查状态是否允许取消
    pub fn can_cancel(&self) -> bool {
        matches!(self, QueryExecutionState::Pending | QueryExecutionState::Running)
    }

    /// 获取状态的中文描述
    pub fn description(&self) -> &'static str {
        match self {
            QueryExecutionState::Pending => "等待执行",
            QueryExecutionState::Running => "执行中",
            QueryExecutionState::Completed => "已完成",
            QueryExecutionState::Failed => "执行失败",
            QueryExecutionState::Cancelled => "已取消",
            QueryExecutionState::Timeout => "执行超时",
        }
    }
}

impl fmt::Display for QueryExecutionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl Default for QueryExecutionState {
    fn default() -> Self {
        QueryExecutionState::Pending
    }
}

/// 执行器状态 - 单个执行器的运行状态
///
/// 表示单个执行器实例的执行状态，用于执行器生命周期管理。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExecutorState {
    /// 执行器已创建，未开始执行
    Initialized,
    /// 执行器正在执行
    Executing,
    /// 执行器执行完成
    Completed,
    /// 执行器执行失败
    Failed,
    /// 执行器被取消
    Cancelled,
    /// 执行器被暂停（用于断点调试）
    Paused,
}

impl ExecutorState {
    /// 检查状态是否允许转换为目标状态
    pub fn can_transition_to(&self, target: ExecutorState) -> bool {
        match (self, target) {
            // 初始化状态可以转换到执行中、失败或取消
            (ExecutorState::Initialized, ExecutorState::Executing) => true,
            (ExecutorState::Initialized, ExecutorState::Failed) => true,
            (ExecutorState::Initialized, ExecutorState::Cancelled) => true,
            // 执行中可以转换到完成、失败、取消或暂停
            (ExecutorState::Executing, ExecutorState::Completed) => true,
            (ExecutorState::Executing, ExecutorState::Failed) => true,
            (ExecutorState::Executing, ExecutorState::Cancelled) => true,
            (ExecutorState::Executing, ExecutorState::Paused) => true,
            // 暂停可以恢复执行、失败或取消
            (ExecutorState::Paused, ExecutorState::Executing) => true,
            (ExecutorState::Paused, ExecutorState::Failed) => true,
            (ExecutorState::Paused, ExecutorState::Cancelled) => true,
            // 终态不能再转换
            (ExecutorState::Completed, _) => false,
            (ExecutorState::Failed, _) => false,
            (ExecutorState::Cancelled, _) => false,
            _ => false,
        }
    }

    /// 获取状态的中文描述
    pub fn description(&self) -> &'static str {
        match self {
            ExecutorState::Initialized => "已初始化",
            ExecutorState::Executing => "执行中",
            ExecutorState::Completed => "已完成",
            ExecutorState::Failed => "执行失败",
            ExecutorState::Cancelled => "已取消",
            ExecutorState::Paused => "已暂停",
        }
    }
}

impl fmt::Display for ExecutorState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl Default for ExecutorState {
    fn default() -> Self {
        ExecutorState::Initialized
    }
}

/// 循环执行状态 - 循环控制专用状态
///
/// 专门用于循环执行器（LoopExecutor）的状态管理。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LoopExecutionState {
    /// 循环未开始
    NotStarted,
    /// 循环执行中
    Running { iteration: usize },
    /// 循环正常结束
    Finished,
    /// 循环因错误终止
    Error(String),
    /// 循环因达到最大迭代次数而终止
    MaxIterationsReached { max: usize },
}

impl LoopExecutionState {
    /// 获取当前迭代次数
    pub fn iteration(&self) -> Option<usize> {
        match self {
            LoopExecutionState::Running { iteration } => Some(*iteration),
            _ => None,
        }
    }

    /// 检查循环是否已结束
    pub fn is_finished(&self) -> bool {
        matches!(
            self,
            LoopExecutionState::Finished
                | LoopExecutionState::Error(_)
                | LoopExecutionState::MaxIterationsReached { .. }
        )
    }

    /// 获取状态的中文描述
    pub fn description(&self) -> String {
        match self {
            LoopExecutionState::NotStarted => "未开始".to_string(),
            LoopExecutionState::Running { iteration } => format!("执行中 (第 {} 次迭代)", iteration),
            LoopExecutionState::Finished => "已完成".to_string(),
            LoopExecutionState::Error(msg) => format!("错误: {}", msg),
            LoopExecutionState::MaxIterationsReached { max } => format!("达到最大迭代次数 ({})", max),
        }
    }
}

impl fmt::Display for LoopExecutionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl Default for LoopExecutionState {
    fn default() -> Self {
        LoopExecutionState::NotStarted
    }
}

/// 结果行状态 - 单行数据处理状态
///
/// 表示单条数据记录的处理结果状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RowStatus {
    /// 正常数据
    Valid,
    /// 数据被过滤掉
    Filtered,
    /// 数据无效
    Invalid,
    /// 标签被过滤
    TagFiltered,
}

impl RowStatus {
    /// 检查是否为有效数据
    pub fn is_valid(&self) -> bool {
        matches!(self, RowStatus::Valid)
    }

    /// 转换为整数表示（用于兼容旧代码）
    pub fn to_i32(&self) -> i32 {
        match self {
            RowStatus::Valid => 0,
            RowStatus::Invalid => -1,
            RowStatus::Filtered => -2,
            RowStatus::TagFiltered => -3,
        }
    }
}

impl fmt::Display for RowStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RowStatus::Valid => write!(f, "有效"),
            RowStatus::Filtered => write!(f, "已过滤"),
            RowStatus::Invalid => write!(f, "无效"),
            RowStatus::TagFiltered => write!(f, "标签过滤"),
        }
    }
}

impl Default for RowStatus {
    fn default() -> Self {
        RowStatus::Valid
    }
}

/// 优化阶段状态 - 查询优化过程状态
///
/// 表示查询优化器的工作阶段。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OptimizationState {
    /// 未开始优化
    NotStarted,
    /// 重写阶段
    Rewriting,
    /// 逻辑优化阶段
    LogicalOptimizing,
    /// 物理优化阶段
    PhysicalOptimizing,
    /// 优化完成
    Completed,
    /// 优化失败
    Failed,
}

impl OptimizationState {
    /// 获取阶段的中文描述
    pub fn description(&self) -> &'static str {
        match self {
            OptimizationState::NotStarted => "未开始",
            OptimizationState::Rewriting => "重写阶段",
            OptimizationState::LogicalOptimizing => "逻辑优化",
            OptimizationState::PhysicalOptimizing => "物理优化",
            OptimizationState::Completed => "优化完成",
            OptimizationState::Failed => "优化失败",
        }
    }

    /// 检查是否处于优化阶段
    pub fn is_optimizing(&self) -> bool {
        matches!(
            self,
            OptimizationState::Rewriting
                | OptimizationState::LogicalOptimizing
                | OptimizationState::PhysicalOptimizing
        )
    }
}

impl fmt::Display for OptimizationState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl Default for OptimizationState {
    fn default() -> Self {
        OptimizationState::NotStarted
    }
}

/// 优化阶段 - 用于优化规则分类
///
/// 表示优化规则所属的阶段，用于控制规则的执行顺序。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OptimizationPhase {
    /// 重写阶段 - 逻辑重写规则
    Rewrite,
    /// 逻辑优化阶段 - 逻辑计划优化
    Logical,
    /// 物理优化阶段 - 物理计划优化
    Physical,
    /// 未知阶段
    Unknown,
}

impl OptimizationPhase {
    /// 获取阶段的中文描述
    pub fn description(&self) -> &'static str {
        match self {
            OptimizationPhase::Rewrite => "重写阶段",
            OptimizationPhase::Logical => "逻辑优化",
            OptimizationPhase::Physical => "物理优化",
            OptimizationPhase::Unknown => "未知阶段",
        }
    }

    /// 检查是否为逻辑优化阶段
    pub fn is_logical(&self) -> bool {
        matches!(self, OptimizationPhase::Rewrite | OptimizationPhase::Logical)
    }

    /// 检查是否为物理优化阶段
    pub fn is_physical(&self) -> bool {
        matches!(self, OptimizationPhase::Physical)
    }
}

impl fmt::Display for OptimizationPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl Default for OptimizationPhase {
    fn default() -> Self {
        OptimizationPhase::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_execution_state_transitions() {
        assert!(!QueryExecutionState::Running.is_terminal());
        assert!(QueryExecutionState::Completed.is_terminal());
        assert!(QueryExecutionState::Pending.can_cancel());
        assert!(!QueryExecutionState::Completed.can_cancel());
    }

    #[test]
    fn test_executor_state_transitions() {
        assert!(ExecutorState::Initialized.can_transition_to(ExecutorState::Executing));
        assert!(!ExecutorState::Completed.can_transition_to(ExecutorState::Executing));
        assert!(ExecutorState::Executing.can_transition_to(ExecutorState::Paused));
        assert!(ExecutorState::Paused.can_transition_to(ExecutorState::Executing));
    }

    #[test]
    fn test_loop_execution_state() {
        let state = LoopExecutionState::Running { iteration: 5 };
        assert_eq!(state.iteration(), Some(5));
        assert!(!state.is_finished());

        let finished = LoopExecutionState::Finished;
        assert!(finished.is_finished());
    }

    #[test]
    fn test_row_status_conversion() {
        assert_eq!(RowStatus::Valid.to_i32(), 0);
        assert_eq!(RowStatus::Invalid.to_i32(), -1);
        assert_eq!(RowStatus::Filtered.to_i32(), -2);
        assert_eq!(RowStatus::TagFiltered.to_i32(), -3);
    }
}
