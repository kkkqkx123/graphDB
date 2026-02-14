//! 错误处理辅助函数
//!
//! 提供安全的锁操作和错误传播宏

use crate::core::error::DBError;
use std::sync::{Mutex, RwLock};

/// 安全地获取 Mutex 锁，提供有意义的错误信息
pub fn safe_lock<T>(mutex: &Mutex<T>) -> Result<std::sync::MutexGuard<T>, DBError> {
    mutex.lock().map_err(|e| {
        crate::core::error::LockError::MutexPoisoned {
            reason: format!("Mutex is poisoned: {:?}", e),
        }
        .into()
    })
}

/// 安全地获取 RwLock 读锁
pub fn safe_read<T>(rwlock: &RwLock<T>) -> Result<std::sync::RwLockReadGuard<T>, DBError> {
    rwlock.read().map_err(|e| {
        crate::core::error::LockError::RwLockReadPoisoned {
            reason: format!("RwLock read lock is poisoned: {:?}", e),
        }
        .into()
    })
}

/// 安全地获取 RwLock 写锁
pub fn safe_write<T>(rwlock: &RwLock<T>) -> Result<std::sync::RwLockWriteGuard<T>, DBError> {
    rwlock.write().map_err(|e| {
        crate::core::error::LockError::RwLockWritePoisoned {
            reason: format!("RwLock write lock is poisoned: {:?}", e),
        }
        .into()
    })
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

        let result = std::panic::catch_unwind(|| {
            let _guard = mutex.lock().expect("mutex.lock should succeed");
            panic!("Intentional panic to poison the lock");
        });

        assert!(result.is_err());

        let result = safe_lock(&mutex);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DBError::Lock(_)));
    }
}
