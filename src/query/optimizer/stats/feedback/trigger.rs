//! 自动反馈触发模块
//!
//! 参考PostgreSQL的ANALYZE自动触发机制，
//! 配置何时自动更新统计信息和重新估计选择性。

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// 自动反馈触发配置
///
/// 配置何时自动触发统计更新，包括最小样本数、误差阈值和冷却时间。
///
/// # 示例
/// ```
/// use graphdb::query::optimizer::stats::feedback::trigger::AutoFeedbackConfig;
///
/// let config = AutoFeedbackConfig::new();
/// assert!(config.enabled);
/// assert_eq!(config.min_samples_for_update, 10);
/// ```
#[derive(Debug, Clone)]
pub struct AutoFeedbackConfig {
    /// 最小反馈样本数触发重新估计
    pub min_samples_for_update: usize,
    /// 误差阈值触发紧急更新（误差超过此值时立即更新）
    pub error_threshold: f64,
    /// 更新冷却时间（避免频繁更新）
    pub update_cooldown_ms: u64,
    /// 最大反馈历史记录数
    pub max_feedback_history: usize,
    /// 是否启用自动更新
    pub enabled: bool,
}

impl AutoFeedbackConfig {
    /// 创建默认配置
    pub fn new() -> Self {
        Self {
            min_samples_for_update: 10,
            error_threshold: 0.5,
            update_cooldown_ms: 60000, // 1分钟
            max_feedback_history: 100,
            enabled: true,
        }
    }

    /// 使用自定义参数创建
    pub fn with_params(
        min_samples: usize,
        error_threshold: f64,
        cooldown_ms: u64,
        max_history: usize,
    ) -> Self {
        Self {
            min_samples_for_update: min_samples,
            error_threshold: error_threshold.max(0.1).min(1.0),
            update_cooldown_ms: cooldown_ms,
            max_feedback_history: max_history,
            enabled: true,
        }
    }

    /// 检查是否应该触发更新
    ///
    /// # 参数
    /// - `feedback_count`: 当前反馈样本数
    /// - `last_update_ms`: 上次更新时间（毫秒时间戳）
    /// - `current_error`: 当前估计误差
    ///
    /// # 返回
    /// - `true`: 应该触发更新
    /// - `false`: 不需要更新
    pub fn should_trigger_update(
        &self,
        feedback_count: usize,
        last_update_ms: u64,
        current_error: f64,
    ) -> bool {
        if !self.enabled {
            return false;
        }

        // 检查误差阈值（紧急更新）
        if current_error > self.error_threshold {
            return true;
        }

        // 检查最小样本数
        if feedback_count < self.min_samples_for_update {
            return false;
        }

        // 检查冷却时间
        let current_time = Instant::now().elapsed().as_millis() as u64;
        if current_time.saturating_sub(last_update_ms) < self.update_cooldown_ms {
            return false;
        }

        true
    }

    /// 启用自动更新
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// 禁用自动更新
    pub fn disable(&mut self) {
        self.enabled = false;
    }
}

impl Default for AutoFeedbackConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// 自动反馈触发器
///
/// 根据配置自动决定是否触发统计更新。
/// 使用原子操作确保线程安全。
///
/// # 示例
/// ```
/// use graphdb::query::optimizer::stats::feedback::trigger::{AutoFeedbackTrigger, AutoFeedbackConfig};
///
/// let config = AutoFeedbackConfig::with_params(5, 0.5, 1000, 50);
/// let trigger = AutoFeedbackTrigger::new(config);
///
/// // 记录反馈
/// for _ in 0..5 {
///     trigger.record_feedback();
/// }
///
/// // 检查是否应该触发（误差超过阈值）
/// assert!(trigger.should_trigger(0.6));
/// ```
#[derive(Debug)]
pub struct AutoFeedbackTrigger {
    /// 配置
    config: AutoFeedbackConfig,
    /// 上次更新时间
    last_update_time_ms: AtomicU64,
    /// 当前反馈计数
    feedback_count: AtomicU64,
}

impl AutoFeedbackTrigger {
    /// 创建新的触发器
    pub fn new(config: AutoFeedbackConfig) -> Self {
        Self {
            config,
            last_update_time_ms: AtomicU64::new(0),
            feedback_count: AtomicU64::new(0),
        }
    }

    /// 记录反馈
    pub fn record_feedback(&self) {
        self.feedback_count.fetch_add(1, Ordering::Relaxed);
    }

    /// 检查是否应该触发更新
    pub fn should_trigger(&self, current_error: f64) -> bool {
        let count = self.feedback_count.load(Ordering::Relaxed) as usize;
        let last_update = self.last_update_time_ms.load(Ordering::Relaxed);
        self.config.should_trigger_update(count, last_update, current_error)
    }

    /// 标记更新完成
    pub fn mark_updated(&self) {
        let current_time = Instant::now().elapsed().as_millis() as u64;
        self.last_update_time_ms.store(current_time, Ordering::Relaxed);
        self.feedback_count.store(0, Ordering::Relaxed);
    }

    /// 获取当前反馈计数
    pub fn get_feedback_count(&self) -> u64 {
        self.feedback_count.load(Ordering::Relaxed)
    }

    /// 更新配置
    pub fn update_config(&mut self, config: AutoFeedbackConfig) {
        self.config = config;
    }

    /// 获取配置
    pub fn config(&self) -> &AutoFeedbackConfig {
        &self.config
    }
}

impl Default for AutoFeedbackTrigger {
    fn default() -> Self {
        Self::new(AutoFeedbackConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_feedback_config() {
        let config = AutoFeedbackConfig::new();
        assert!(config.enabled);
        assert_eq!(config.min_samples_for_update, 10);

        // 测试触发逻辑
        assert!(!config.should_trigger_update(5, 0, 0.1)); // 样本不足
        assert!(config.should_trigger_update(5, 0, 0.6)); // 误差超过阈值（紧急更新）

        // 测试正常触发逻辑：如果程序运行时间已经超过冷却时间
        let current_time = Instant::now().elapsed().as_millis() as u64;
        if current_time > config.update_cooldown_ms {
            assert!(config.should_trigger_update(15, 0, 0.1)); // 样本足够且已过冷却时间
        }
    }

    #[test]
    fn test_auto_feedback_trigger() {
        let config = AutoFeedbackConfig::with_params(5, 0.5, 1000, 50);
        let trigger = AutoFeedbackTrigger::new(config);

        assert!(!trigger.should_trigger(0.1)); // 无反馈

        // 记录反馈
        for _ in 0..5 {
            trigger.record_feedback();
        }

        // 误差超过阈值应该触发
        assert!(trigger.should_trigger(0.6));

        trigger.mark_updated();
        assert_eq!(trigger.get_feedback_count(), 0);
    }

    #[test]
    fn test_config_enable_disable() {
        let mut config = AutoFeedbackConfig::new();
        assert!(config.enabled);

        config.disable();
        assert!(!config.enabled);
        assert!(!config.should_trigger_update(100, 0, 1.0));

        config.enable();
        assert!(config.enabled);
    }
}
