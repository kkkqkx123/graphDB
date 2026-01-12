//! 执行上下文定义
//!
//! 提供查询执行过程中的执行上下文管理

use super::base::ContextType;
use super::query::QueryContext;
use super::traits::BaseContext;
use crate::core::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 记录类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub fields: HashMap<String, Value>,
}

impl Record {
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    pub fn add_field(&mut self, name: String, value: Value) {
        self.fields.insert(name, value);
    }

    pub fn get_field(&self, name: &str) -> Option<&Value> {
        self.fields.get(name)
    }
}

impl Default for Record {
    fn default() -> Self {
        Self::new()
    }
}

/// 执行上下文
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// 查询上下文
    pub query_context: QueryContext,
    /// 执行状态
    pub execution_state: ExecutionState,
    /// 变量绑定
    pub variable_bindings: HashMap<String, Value>,
    /// 中间结果
    pub intermediate_results: Vec<Record>,
    /// 执行统计
    pub execution_stats: ExecutionStatistics,
    /// 资源限制
    pub resource_limits: ResourceLimits,
}

/// 执行状态
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExecutionState {
    /// 初始化
    Initialized,
    /// 执行中
    Running,
    /// 暂停
    Paused,
    /// 已完成
    Completed,
    /// 已取消
    Cancelled,
    /// 错误
    Error,
}

/// 执行统计信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionStatistics {
    /// 已处理的记录数
    pub records_processed: usize,
    /// 已产生的记录数
    pub records_produced: usize,
    /// 执行时间（毫秒）
    pub execution_time_ms: u64,
    /// 内存使用量（字节）
    pub memory_used_bytes: usize,
    /// 网络IO量（字节）
    pub network_io_bytes: usize,
    /// 磁盘IO量（字节）
    pub disk_io_bytes: usize,
}

/// 资源限制
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// 最大内存使用量（字节）
    pub max_memory_bytes: Option<usize>,
    /// 最大执行时间（毫秒）
    pub max_execution_time_ms: Option<u64>,
    /// 最大网络IO量（字节）
    pub max_network_io_bytes: Option<usize>,
    /// 最大磁盘IO量（字节）
    pub max_disk_io_bytes: Option<usize>,
    /// 最大中间结果数量
    pub max_intermediate_results: Option<usize>,
}

impl ExecutionContext {
    /// 创建新的执行上下文
    pub fn new(query_context: QueryContext) -> Self {
        Self {
            query_context,
            execution_state: ExecutionState::Initialized,
            variable_bindings: HashMap::new(),
            intermediate_results: Vec::new(),
            execution_stats: ExecutionStatistics::default(),
            resource_limits: ResourceLimits::default(),
        }
    }

    /// 设置执行状态
    pub fn set_execution_state(&mut self, state: ExecutionState) {
        self.execution_state = state;
    }

    /// 绑定变量
    pub fn bind_variable(&mut self, name: impl Into<String>, value: Value) {
        self.variable_bindings.insert(name.into(), value);
    }

    /// 获取变量值
    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variable_bindings.get(name)
    }

    /// 添加中间结果
    pub fn add_intermediate_result(&mut self, record: Record) {
        self.intermediate_results.push(record);
        self.execution_stats.records_produced += 1;
    }

    /// 清空中间结果
    pub fn clear_intermediate_results(&mut self) {
        self.intermediate_results.clear();
    }

    /// 获取中间结果数量
    pub fn intermediate_result_count(&self) -> usize {
        self.intermediate_results.len()
    }

    /// 更新执行统计
    pub fn update_execution_stats(&mut self) {
        self.execution_stats.execution_time_ms = self.query_context.elapsed_ms();
    }

    /// 增加已处理的记录数
    pub fn add_records_processed(&mut self, count: usize) {
        self.execution_stats.records_processed += count;
    }

    /// 设置内存使用量
    pub fn set_memory_used(&mut self, bytes: usize) {
        self.execution_stats.memory_used_bytes = bytes;
    }

    /// 增加网络IO量
    pub fn add_network_io(&mut self, bytes: usize) {
        self.execution_stats.network_io_bytes += bytes;
    }

    /// 增加磁盘IO量
    pub fn add_disk_io(&mut self, bytes: usize) {
        self.execution_stats.disk_io_bytes += bytes;
    }

    /// 检查是否超出资源限制
    pub fn check_resource_limits(&self) -> ResourceLimitViolation {
        let mut violations = Vec::new();

        if let Some(max_memory) = self.resource_limits.max_memory_bytes {
            if self.execution_stats.memory_used_bytes > max_memory {
                violations.push(ResourceLimitType::Memory);
            }
        }

        if let Some(max_time) = self.resource_limits.max_execution_time_ms {
            if self.execution_stats.execution_time_ms > max_time {
                violations.push(ResourceLimitType::ExecutionTime);
            }
        }

        if let Some(max_network_io) = self.resource_limits.max_network_io_bytes {
            if self.execution_stats.network_io_bytes > max_network_io {
                violations.push(ResourceLimitType::NetworkIO);
            }
        }

        if let Some(max_disk_io) = self.resource_limits.max_disk_io_bytes {
            if self.execution_stats.disk_io_bytes > max_disk_io {
                violations.push(ResourceLimitType::DiskIO);
            }
        }

        if let Some(max_results) = self.resource_limits.max_intermediate_results {
            if self.intermediate_results.len() > max_results {
                violations.push(ResourceLimitType::IntermediateResults);
            }
        }

        if violations.is_empty() {
            ResourceLimitViolation::None
        } else {
            ResourceLimitViolation::Violations(violations)
        }
    }

    /// 检查是否应该暂停执行
    pub fn should_pause(&self) -> bool {
        matches!(self.execution_state, ExecutionState::Paused)
    }

    /// 检查是否应该取消执行
    pub fn should_cancel(&self) -> bool {
        matches!(self.execution_state, ExecutionState::Cancelled)
            || self.query_context.is_timeout()
            || !matches!(self.check_resource_limits(), ResourceLimitViolation::None)
    }

    /// 检查是否有错误
    pub fn has_error(&self) -> bool {
        matches!(self.execution_state, ExecutionState::Error)
    }

    /// 检查是否已完成
    pub fn is_completed(&self) -> bool {
        matches!(
            self.execution_state,
            ExecutionState::Completed | ExecutionState::Cancelled | ExecutionState::Error
        )
    }
}

impl BaseContext for ExecutionContext {
    fn id(&self) -> &str {
        &self.query_context.query_id
    }

    fn context_type(&self) -> ContextType {
        ContextType::Execution
    }

    fn created_at(&self) -> std::time::SystemTime {
        std::time::SystemTime::now()
    }

    fn updated_at(&self) -> std::time::SystemTime {
        std::time::SystemTime::now()
    }

    fn is_valid(&self) -> bool {
        !self.should_cancel() && !self.has_error()
    }

    fn touch(&mut self) {
        self.update_execution_stats();
    }

    fn invalidate(&mut self) {
        self.execution_state = ExecutionState::Error;
    }

    fn revalidate(&mut self) -> bool {
        if self.should_cancel() {
            self.execution_state = ExecutionState::Cancelled;
            false
        } else if self.has_error() {
            false
        } else {
            if self.execution_state == ExecutionState::Error {
                self.execution_state = ExecutionState::Running;
            }
            true
        }
    }

    fn parent_id(&self) -> Option<&str> {
        Some(&self.query_context.query_id)
    }

    fn depth(&self) -> usize {
        2
    }
}

/// 资源限制违规
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceLimitViolation {
    /// 无违规
    None,
    /// 有违规
    Violations(Vec<ResourceLimitType>),
}

/// 资源限制类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceLimitType {
    /// 内存限制
    Memory,
    /// 执行时间限制
    ExecutionTime,
    /// 网络IO限制
    NetworkIO,
    /// 磁盘IO限制
    DiskIO,
    /// 中间结果数量限制
    IntermediateResults,
}

impl Default for ExecutionStatistics {
    fn default() -> Self {
        Self {
            records_processed: 0,
            records_produced: 0,
            execution_time_ms: 0,
            memory_used_bytes: 0,
            network_io_bytes: 0,
            disk_io_bytes: 0,
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: Some(1024 * 1024 * 1024),    // 默认1GB
            max_execution_time_ms: Some(300000),           // 默认5分钟
            max_network_io_bytes: Some(1024 * 1024 * 100), // 默认100MB
            max_disk_io_bytes: Some(1024 * 1024 * 1024),   // 默认1GB
            max_intermediate_results: Some(1000000),       // 默认100万条记录
        }
    }
}

/// 执行阶段
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExecutionPhase {
    /// 初始化阶段
    Initialization,
    /// 验证阶段
    Validation,
    /// 优化阶段
    Optimization,
    /// 执行阶段
    Execution,
    /// 结果处理阶段
    ResultProcessing,
    /// 清理阶段
    Cleanup,
}

/// 执行进度
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionProgress {
    /// 当前阶段
    pub current_phase: ExecutionPhase,
    /// 总阶段数
    pub total_phases: usize,
    /// 当前阶段进度（0-100）
    pub phase_progress: u8,
    /// 总体进度（0-100）
    pub overall_progress: u8,
    /// 阶段描述
    pub phase_description: String,
}

impl ExecutionProgress {
    /// 创建新的执行进度
    pub fn new(total_phases: usize) -> Self {
        Self {
            current_phase: ExecutionPhase::Initialization,
            total_phases,
            phase_progress: 0,
            overall_progress: 0,
            phase_description: "初始化中".to_string(),
        }
    }

    /// 进入下一个阶段
    pub fn next_phase(&mut self, phase: ExecutionPhase, description: impl Into<String>) {
        self.current_phase = phase;
        self.phase_progress = 0;
        self.phase_description = description.into();
        self.update_overall_progress();
    }

    /// 更新阶段进度
    pub fn update_phase_progress(&mut self, progress: u8) {
        self.phase_progress = progress.min(100);
        self.update_overall_progress();
    }

    /// 更新总体进度
    fn update_overall_progress(&mut self) {
        let phase_index = match self.current_phase {
            ExecutionPhase::Initialization => 0,
            ExecutionPhase::Validation => 1,
            ExecutionPhase::Optimization => 2,
            ExecutionPhase::Execution => 3,
            ExecutionPhase::ResultProcessing => 4,
            ExecutionPhase::Cleanup => 5,
        };

        if self.total_phases > 0 {
            let completed_phases = phase_index;
            let phase_weight = 100 / self.total_phases;
            self.overall_progress = ((completed_phases * phase_weight)
                + (self.phase_progress as usize / self.total_phases))
                as u8;
        }
    }

    /// 检查是否完成
    pub fn is_completed(&self) -> bool {
        matches!(self.current_phase, ExecutionPhase::Cleanup) && self.phase_progress >= 100
    }
}

impl Default for ExecutionProgress {
    fn default() -> Self {
        Self::new(6) // 默认6个阶段
    }
}
