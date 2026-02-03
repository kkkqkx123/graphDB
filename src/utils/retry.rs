//! 重试机制
//!
//! 提供可配置的重试策略和指数退避算法

use crate::core::error::StorageError;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryStrategy {
    Fixed,
    Linear,
    Exponential,
}

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_attempts(mut self, max_attempts: u32) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    pub fn with_initial_delay(mut self, delay_ms: u64) -> Self {
        self.initial_delay_ms = delay_ms;
        self
    }

    pub fn with_max_delay(mut self, delay_ms: u64) -> Self {
        self.max_delay_ms = delay_ms;
        self
    }

    pub fn with_backoff_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    pub fn with_retry_only_retryable(mut self, only_retryable: bool) -> Self {
        self.retry_only_retryable = only_retryable;
        self
    }
}

pub fn retry_with_backoff<F, T, E>(config: &RetryConfig, mut operation: F) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
    E: Into<StorageError> + Clone + std::fmt::Display,
{
    let mut delay = config.initial_delay_ms;
    let mut last_error: Option<E> = None;

    for attempt in 0..config.max_attempts {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) => {
                let storage_error: StorageError = e.clone().into();

                if config.retry_only_retryable && !storage_error.is_retryable() {
                    return Err(e);
                }

                if attempt == config.max_attempts - 1 {
                    return Err(e);
                }

                last_error = Some(e.clone());

                let sleep_time = delay.min(config.max_delay_ms);
                thread::sleep(Duration::from_millis(sleep_time));

                delay = (delay as f64 * config.backoff_multiplier) as u64;
            }
        }
    }

    Err(last_error.expect("至少应该有一个错误"))
}

pub fn retry_with_strategy<F, T, E>(config: &RetryConfig, strategy: RetryStrategy, mut operation: F) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
    E: Into<StorageError> + Clone + std::fmt::Display,
{
    let mut delay = config.initial_delay_ms;
    let mut last_error: Option<E> = None;

    for attempt in 0..config.max_attempts {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) => {
                let storage_error: StorageError = e.clone().into();

                if config.retry_only_retryable && !storage_error.is_retryable() {
                    return Err(e);
                }

                if attempt == config.max_attempts - 1 {
                    return Err(e);
                }

                last_error = Some(e.clone());

                let sleep_time = delay.min(config.max_delay_ms);
                thread::sleep(Duration::from_millis(sleep_time));

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
    use crate::core::error::StorageError;

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
                Err::<i32, StorageError>(StorageError::LockTimeout("模拟锁超时".to_string()))
            } else {
                Ok(42)
            }
        });

        assert!(result.is_ok());
        assert_eq!(result.expect("result should be Ok"), 42);
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
            Err::<i32, StorageError>(StorageError::LockTimeout("模拟锁超时".to_string()))
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
            Err::<i32, StorageError>(StorageError::InvalidInput("无效输入".to_string()))
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
                Err::<i32, StorageError>(StorageError::LockTimeout("模拟锁超时".to_string()))
            } else {
                Ok(42)
            }
        });

        assert!(result.is_ok());
        assert_eq!(result.expect("result should be Ok"), 42);
        assert_eq!(count, 3);
    }
}
