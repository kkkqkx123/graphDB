//! 预编译语句配置模块
//!
//! 提供预编译语句的配置、统计信息和参数信息定义

use crate::core::{DataType, Value};
use std::time::{Duration, Instant};

/// 预编译语句配置
///
/// 用于配置预编译语句的行为
#[derive(Debug, Clone)]
pub struct StatementConfig {
    /// 是否启用类型检查
    pub enable_type_check: bool,
    /// 最大执行历史记录数
    pub max_history_size: usize,
}

impl Default for StatementConfig {
    fn default() -> Self {
        Self {
            enable_type_check: true,
            max_history_size: 100,
        }
    }
}

impl StatementConfig {
    /// 创建默认配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 禁用类型检查
    pub fn disable_type_check(mut self) -> Self {
        self.enable_type_check = false;
        self
    }

    /// 设置最大历史记录数
    pub fn with_max_history(mut self, size: usize) -> Self {
        self.max_history_size = size;
        self
    }
}

/// 执行统计信息
#[derive(Debug, Clone, Default)]
pub struct ExecutionStats {
    /// 执行次数
    pub execution_count: u64,
    /// 总执行时间（毫秒）
    pub total_execution_time_ms: u64,
    /// 平均执行时间（毫秒）
    pub avg_execution_time_ms: f64,
    /// 最小执行时间（毫秒）
    pub min_execution_time_ms: u64,
    /// 最大执行时间（毫秒）
    pub max_execution_time_ms: u64,
    /// 最后执行时间
    pub last_execution_time: Option<Instant>,
}

impl ExecutionStats {
    /// 创建新的统计信息
    pub fn new() -> Self {
        Self {
            min_execution_time_ms: u64::MAX,
            ..Default::default()
        }
    }

    /// 记录一次执行
    pub fn record_execution(&mut self, duration: Duration) {
        let ms = duration.as_millis() as u64;
        self.execution_count += 1;
        self.total_execution_time_ms += ms;
        self.avg_execution_time_ms =
            self.total_execution_time_ms as f64 / self.execution_count as f64;
        self.min_execution_time_ms = self.min_execution_time_ms.min(ms);
        self.max_execution_time_ms = self.max_execution_time_ms.max(ms);
        self.last_execution_time = Some(Instant::now());
    }
}

/// 参数信息
#[derive(Debug, Clone)]
pub struct ParameterInfo {
    /// 参数名称
    pub name: String,
    /// 参数类型
    pub data_type: DataType,
    /// 是否必需
    pub required: bool,
    /// 默认值
    pub default_value: Option<Value>,
}
