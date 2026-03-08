//! 忙等待处理器模块
//!
//! 提供多线程环境下的并发控制机制，支持超时和指数退避等待策略

use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

/// 忙等待处理器
///
/// 当多个线程同时访问数据库时，用于处理资源冲突
/// 支持指数退避算法，避免忙等待消耗过多 CPU
#[derive(Debug)]
pub struct BusyHandler {
    /// 超时时间（毫秒），0 表示不等待
    timeout_ms: u32,
    /// 当前重试次数
    retry_count: AtomicU32,
    /// 开始时间
    start_time: Instant,
}

impl BusyHandler {
    /// 创建新的忙等待处理器
    ///
    /// # 参数
    /// - `timeout_ms` - 超时时间（毫秒），0 表示不等待
    ///
    /// # 示例
    ///
    /// ```rust
    /// use graphdb::api::embedded::BusyHandler;
    ///
    /// let handler = BusyHandler::new(5000); // 5 秒超时
    /// ```
    pub fn new(timeout_ms: u32) -> Self {
        Self {
            timeout_ms,
            retry_count: AtomicU32::new(0),
            start_time: Instant::now(),
        }
    }

    /// 处理忙状态
    ///
    /// 返回 true 表示继续等待，false 表示放弃（超时）
    ///
    /// # 说明
    ///
    /// 使用指数退避算法计算等待时间：
    /// - 第 0 次：1ms
    /// - 第 1 次：2ms
    /// - 第 2 次：4ms
    /// - ...
    /// - 最大 100ms
    pub fn handle_busy(&self) -> bool {
        // 不等待模式
        if self.timeout_ms == 0 {
            return false;
        }

        let count = self.retry_count.fetch_add(1, Ordering::SeqCst);

        // 检查是否超时
        let elapsed = self.start_time.elapsed().as_millis() as u64;
        if elapsed >= self.timeout_ms as u64 {
            return false;
        }

        // 计算等待时间（指数退避）
        let wait_ms = Self::calculate_wait_time(count);

        // 确保不超过剩余超时时间
        let remaining = self.timeout_ms as u64 - elapsed;
        let actual_wait = std::cmp::min(wait_ms, remaining);

        std::thread::sleep(Duration::from_millis(actual_wait));
        true
    }

    /// 检查是否已超时
    pub fn is_timeout(&self) -> bool {
        if self.timeout_ms == 0 {
            return true;
        }
        self.start_time.elapsed().as_millis() as u64 >= self.timeout_ms as u64
    }

    /// 获取当前重试次数
    pub fn retry_count(&self) -> u32 {
        self.retry_count.load(Ordering::SeqCst)
    }

    /// 获取已等待时间（毫秒）
    pub fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }

    /// 重置处理器状态
    pub fn reset(&self) {
        self.retry_count.store(0, Ordering::SeqCst);
    }

    /// 计算等待时间（指数退避）
    ///
    /// 公式：min(2^retry_count, 100) 毫秒
    fn calculate_wait_time(retry_count: u32) -> u64 {
        let base = 1u64;
        let max_wait = 100u64; // 最大 100ms

        // 防止移位溢出
        if retry_count >= 63 {
            return max_wait;
        }

        std::cmp::min(base << retry_count, max_wait)
    }
}

impl Clone for BusyHandler {
    fn clone(&self) -> Self {
        Self {
            timeout_ms: self.timeout_ms,
            retry_count: AtomicU32::new(0),
            start_time: Instant::now(),
        }
    }
}

/// 忙等待结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusyResult {
    /// 成功获取资源
    Success,
    /// 超时
    Timeout,
    /// 放弃（不等待）
    Abort,
}

/// 忙等待配置
#[derive(Debug, Clone, Copy)]
pub struct BusyConfig {
    /// 超时时间（毫秒）
    pub timeout_ms: u32,
    /// 最大重试次数（0 表示无限制）
    pub max_retries: u32,
}

impl BusyConfig {
    /// 创建新的忙等待配置
    pub fn new(timeout_ms: u32) -> Self {
        Self {
            timeout_ms,
            max_retries: 0, // 无限制
        }
    }

    /// 设置最大重试次数
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// 禁用忙等待（立即失败）
    pub fn no_wait() -> Self {
        Self {
            timeout_ms: 0,
            max_retries: 0,
        }
    }
}

impl Default for BusyConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 5000, // 默认 5 秒
            max_retries: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_busy_handler_no_wait() {
        let handler = BusyHandler::new(0);
        assert!(!handler.handle_busy());
        assert!(handler.is_timeout());
    }

    #[test]
    fn test_busy_handler_wait() {
        let handler = BusyHandler::new(100); // 100ms 超时
        assert!(handler.handle_busy()); // 第一次应该返回 true
        assert!(!handler.is_timeout());
        assert_eq!(handler.retry_count(), 1);
    }

    #[test]
    fn test_busy_handler_timeout() {
        let handler = BusyHandler::new(1); // 1ms 超时
        std::thread::sleep(Duration::from_millis(2));
        assert!(!handler.handle_busy()); // 应该超时
        assert!(handler.is_timeout());
    }

    #[test]
    fn test_calculate_wait_time() {
        assert_eq!(BusyHandler::calculate_wait_time(0), 1);
        assert_eq!(BusyHandler::calculate_wait_time(1), 2);
        assert_eq!(BusyHandler::calculate_wait_time(2), 4);
        assert_eq!(BusyHandler::calculate_wait_time(6), 64);
        assert_eq!(BusyHandler::calculate_wait_time(7), 100); // 达到最大值
        assert_eq!(BusyHandler::calculate_wait_time(10), 100); // 保持最大值
    }

    #[test]
    fn test_busy_config() {
        let config = BusyConfig::default();
        assert_eq!(config.timeout_ms, 5000);
        assert_eq!(config.max_retries, 0);

        let config = BusyConfig::new(1000).with_max_retries(10);
        assert_eq!(config.timeout_ms, 1000);
        assert_eq!(config.max_retries, 10);

        let config = BusyConfig::no_wait();
        assert_eq!(config.timeout_ms, 0);
    }
}
