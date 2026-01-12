//! 重试机制
//!
//! 提供可配置的重试策略和指数退避算法

use crate::core::error::ManagerError;
use std::thread;
use std::time::Duration;

/// 重试配置
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// 最大重试次数
    pub max_attempts: u32,
    /// 初始延迟（毫秒）
    pub initial_delay_ms: u64,
    /// 最大延迟（毫秒）
    pub max_delay_ms: u64,
    /// 退避倍数
    pub backoff_multiplier: f64,
    /// 是否只重试可重试错误
    pub retry_only_retryable: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
            retry_only_retryable: true,
        }
    }
}

impl RetryConfig {
    /// 创建新的重试配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置最大重试次数
    pub fn with_max_attempts(mut self, max_attempts: u32) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    /// 设置初始延迟
    pub fn with_initial_delay(mut self, delay_ms: u64) -> Self {
        self.initial_delay_ms = delay_ms;
        self
    }

    /// 设置最大延迟
    pub fn with_max_delay(mut self, delay_ms: u64) -> Self {
        self.max_delay_ms = delay_ms;
        self
    }

    /// 设置退避倍数
    pub fn with_backoff_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    /// 设置是否只重试可重试错误
    pub fn with_retry_only_retryable(mut self, only_retryable: bool) -> Self {
        self.retry_only_retryable = only_retryable;
        self
    }
}

/// 使用指数退避算法重试操作
///
/// # 参数
/// - `config`: 重试配置
/// - `operation`: 要重试的操作
///
/// # 返回
/// 操作成功的结果或最后一次失败的错误
///
/// # 示例
/// ```rust
/// use managers::retry::{RetryConfig, retry_with_backoff};
/// use managers::error::ManagerError;
///
/// let config = RetryConfig::new()
///     .with_max_attempts(5)
///     .with_initial_delay(50);
///
/// let result = retry_with_backoff(&config, || {
///     // 可能失败的操作
///     Ok::<i32, ManagerError>(42)
/// });
/// ```
pub fn retry_with_backoff<F, T, E>(config: &RetryConfig, mut operation: F) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
    E: Into<ManagerError> + Clone + std::fmt::Display,
{
    let mut delay = config.initial_delay_ms;
    let mut last_error: Option<E> = None;

    for attempt in 0..config.max_attempts {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) => {
                let manager_error: ManagerError = e.clone().into();

                // 检查是否应该重试
                if config.retry_only_retryable && !manager_error.is_retryable() {
                    return Err(e);
                }

                // 如果是最后一次尝试，直接返回错误
                if attempt == config.max_attempts - 1 {
                    return Err(e);
                }

                last_error = Some(e.clone());

                // 计算延迟时间
                let sleep_time = delay.min(config.max_delay_ms);
                thread::sleep(Duration::from_millis(sleep_time));

                // 计算下一次延迟
                delay = (delay as f64 * config.backoff_multiplier) as u64;
            }
        }
    }

    // 理论上不会到达这里，但为了类型安全
    Err(last_error.expect("至少应该有一个错误"))
}

/// 使用指数退避算法重试异步操作（简化版，使用线程）
///
/// # 参数
/// - `config`: 重试配置
/// - `operation`: 要重试的操作
///
/// # 返回
/// 操作成功的结果或最后一次失败的错误
pub fn retry_with_backoff_async<F, T, E>(config: &RetryConfig, operation: F) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
    E: Into<ManagerError> + Clone + std::fmt::Display,
{
    retry_with_backoff(config, operation)
}

/// 重试策略枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryStrategy {
    /// 固定延迟
    Fixed,
    /// 线性退避
    Linear,
    /// 指数退避
    Exponential,
}

/// 带策略的重试函数
pub fn retry_with_strategy<F, T, E>(
    config: &RetryConfig,
    strategy: RetryStrategy,
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
    E: Into<ManagerError> + Clone + std::fmt::Display,
{
    let mut delay = config.initial_delay_ms;
    let mut last_error: Option<E> = None;

    for attempt in 0..config.max_attempts {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) => {
                let manager_error: ManagerError = e.clone().into();

                if config.retry_only_retryable && !manager_error.is_retryable() {
                    return Err(e);
                }

                if attempt == config.max_attempts - 1 {
                    return Err(e);
                }

                last_error = Some(e.clone());

                let sleep_time = delay.min(config.max_delay_ms);
                thread::sleep(Duration::from_millis(sleep_time));

                // 根据策略计算下一次延迟
                delay = match strategy {
                    RetryStrategy::Fixed => config.initial_delay_ms,
                    RetryStrategy::Linear => delay + config.initial_delay_ms,
                    RetryStrategy::Exponential => (delay as f64 * config.backoff_multiplier) as u64,
                };
            }
        }
    }

    Err(last_error.expect("至少应该有一个错误"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::error::ManagerError;

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_delay_ms, 100);
        assert_eq!(config.max_delay_ms, 5000);
        assert_eq!(config.backoff_multiplier, 2.0);
        assert!(config.retry_only_retryable);
    }

    #[test]
    fn test_retry_config_builder() {
        let config = RetryConfig::new()
            .with_max_attempts(5)
            .with_initial_delay(50)
            .with_max_delay(2000)
            .with_backoff_multiplier(1.5);

        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.initial_delay_ms, 50);
        assert_eq!(config.max_delay_ms, 2000);
        assert_eq!(config.backoff_multiplier, 1.5);
    }

    #[test]
    fn test_retry_with_backoff_success() {
        let config = RetryConfig::new()
            .with_max_attempts(3)
            .with_initial_delay(10);

        let mut count = 0;
        let result = retry_with_backoff(&config, || {
            count += 1;
            if count < 2 {
                Err::<i32, ManagerError>(ManagerError::StorageError("模拟错误".to_string()))
            } else {
                Ok(42)
            }
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_retry_with_backoff_max_attempts() {
        let config = RetryConfig::new()
            .with_max_attempts(3)
            .with_initial_delay(10);

        let mut count = 0;
        let result = retry_with_backoff(&config, || {
            count += 1;
            Err::<i32, ManagerError>(ManagerError::StorageError("模拟错误".to_string()))
        });

        assert!(result.is_err());
        assert_eq!(count, 3);
    }

    #[test]
    fn test_retry_non_retryable_error() {
        let config = RetryConfig::new()
            .with_max_attempts(3)
            .with_initial_delay(10)
            .with_retry_only_retryable(true);

        let mut count = 0;
        let result = retry_with_backoff(&config, || {
            count += 1;
            Err::<i32, ManagerError>(ManagerError::InvalidInput("无效输入".to_string()))
        });

        assert!(result.is_err());
        assert_eq!(count, 1);
    }

    #[test]
    fn test_retry_strategy_exponential() {
        let config = RetryConfig::new()
            .with_max_attempts(3)
            .with_initial_delay(10)
            .with_backoff_multiplier(2.0);

        let mut count = 0;
        let result = retry_with_strategy(&config, RetryStrategy::Exponential, || {
            count += 1;
            if count < 3 {
                Err::<i32, ManagerError>(ManagerError::StorageError("模拟错误".to_string()))
            } else {
                Ok(42)
            }
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(count, 3);
    }
}
