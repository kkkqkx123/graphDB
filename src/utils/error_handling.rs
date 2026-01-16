//! 错误处理辅助函数
//!
//! 提供安全的锁操作和错误处理辅助函数

use crate::core::error::{DBError, LockError};
use std::sync::{Mutex, RwLock};

/// 安全地获取 Mutex 锁，提供有意义的错误信息
pub fn safe_lock<T>(mutex: &Mutex<T>) -> Result<std::sync::MutexGuard<T>, DBError> {
    mutex.lock().map_err(|e| {
        LockError::MutexPoisoned {
            reason: format!("Mutex is poisoned: {:?}", e),
        }
        .into()
    })
}

/// 安全地获取 RwLock 读锁
pub fn safe_read<T>(rwlock: &RwLock<T>) -> Result<std::sync::RwLockReadGuard<T>, DBError> {
    rwlock.read().map_err(|e| {
        LockError::RwLockReadPoisoned {
            reason: format!("RwLock read lock is poisoned: {:?}", e),
        }
        .into()
    })
}

/// 安全地获取 RwLock 写锁
pub fn safe_write<T>(rwlock: &RwLock<T>) -> Result<std::sync::RwLockWriteGuard<T>, DBError> {
    rwlock.write().map_err(|e| {
        LockError::RwLockWritePoisoned {
            reason: format!("RwLock write lock is poisoned: {:?}", e),
        }
        .into()
    })
}

/// 从 Option 中提取值或返回错误
pub fn expect_option<T>(option: Option<T>, error_msg: &str) -> Result<T, DBError> {
    option.ok_or_else(|| DBError::Internal(error_msg.to_string()))
}

/// 从 Result 中提取值或返回错误
pub fn expect_result<T, E>(result: Result<T, E>, error_msg: &str) -> Result<T, DBError>
where
    E: std::fmt::Debug,
{
    result.map_err(|e| DBError::Internal(format!("{}: {:?}", error_msg, e)))
}

/// 从迭代器中获取第一个元素或返回错误
pub fn expect_first<I>(mut iter: I, error_msg: &str) -> Result<I::Item, DBError>
where
    I: Iterator,
{
    iter.next()
        .ok_or_else(|| DBError::Internal(error_msg.to_string()))
}

/// 从迭代器中获取最小值或返回错误
pub fn expect_min<I>(iter: I, error_msg: &str) -> Result<I::Item, DBError>
where
    I: Iterator,
    I::Item: Ord,
{
    iter.min()
        .ok_or_else(|| DBError::Internal(error_msg.to_string()))
}

/// 从迭代器中获取最大值或返回错误
pub fn expect_max<I>(iter: I, error_msg: &str) -> Result<I::Item, DBError>
where
    I: Iterator,
    I::Item: Ord,
{
    iter.max()
        .ok_or_else(|| DBError::Internal(error_msg.to_string()))
}

/// 从迭代器中获取最后一个元素或返回错误
pub fn expect_last<I>(iter: I, error_msg: &str) -> Result<I::Item, DBError>
where
    I: Iterator,
{
    iter.last()
        .ok_or_else(|| DBError::Internal(error_msg.to_string()))
}

/// 从 Vec 中获取最后一个元素或返回错误
pub fn expect_vec_last<'a, T>(vec: &'a Vec<T>, error_msg: &str) -> Result<&'a T, DBError> {
    vec.last()
        .ok_or_else(|| DBError::Internal(error_msg.to_string()))
}

/// 从 Vec 中获取第一个元素或返回错误
pub fn expect_vec_first<'a, T>(vec: &'a Vec<T>, error_msg: &str) -> Result<&'a T, DBError> {
    vec.first()
        .ok_or_else(|| DBError::Internal(error_msg.to_string()))
}

/// 从 Arc 中获取可变引用或返回错误
pub fn expect_arc_mut<'a, T>(
    arc: &'a mut std::sync::Arc<T>,
    error_msg: &str,
) -> Result<&'a mut T, DBError> {
    std::sync::Arc::get_mut(arc).ok_or_else(|| DBError::Internal(error_msg.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[test]
    fn test_safe_lock_success() {
        let mutex = Mutex::new(42);
        let guard = safe_lock(&mutex).expect("safe_lock should succeed");
        assert_eq!(*guard, 42);
    }

    #[test]
    fn test_safe_lock_poisoned() {
        let mutex = Mutex::new(42);

        // 故意污染锁 - 通过在持有锁时 panic 来污染
        let result = std::panic::catch_unwind(|| {
            let _guard = mutex.lock().expect("mutex.lock should succeed");
            panic!("Intentional panic to poison the lock");
        });

        // 确认确实发生了 panic
        assert!(result.is_err());

        // 测试安全锁获取 - 应该返回错误
        let result = safe_lock(&mutex);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DBError::Lock(_)));
    }

    #[test]
    fn test_expect_option_some() {
        let option = Some(42);
        let result = expect_option(option, "Should have value");
        assert_eq!(result.expect("expect_option should succeed"), 42);
    }

    #[test]
    fn test_expect_option_none() {
        let option: Option<i32> = None;
        let result = expect_option(option, "Value should exist");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DBError::Internal(_)));
    }

    #[test]
    fn test_expect_vec_last() {
        let vec = vec![1, 2, 3];
        let result = expect_vec_last(&vec, "Vector should not be empty");
        assert_eq!(result.expect("expect_vec_last should succeed"), &3);
    }

    #[test]
    fn test_expect_vec_last_empty() {
        let vec: Vec<i32> = vec![];
        let result = expect_vec_last(&vec, "Vector should not be empty");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DBError::Internal(_)));
    }

    #[test]
    fn test_expect_first() {
        let vec = vec![1, 2, 3];
        let result = expect_first(vec.iter(), "Iterator should not be empty");
        assert_eq!(result.expect("expect_first should succeed"), &1);
    }

    #[test]
    fn test_expect_first_empty() {
        let vec: Vec<i32> = vec![];
        let result = expect_first(vec.iter(), "Iterator should not be empty");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DBError::Internal(_)));
    }

    #[test]
    fn test_expect_min() {
        let vec = vec![3, 1, 2];
        let result = expect_min(vec.iter(), "Iterator should not be empty");
        assert_eq!(result.expect("expect_min should succeed"), &1);
    }

    #[test]
    fn test_expect_max() {
        let vec = vec![1, 3, 2];
        let result = expect_max(vec.iter(), "Iterator should not be empty");
        assert_eq!(result.expect("expect_max should succeed"), &3);
    }
}
