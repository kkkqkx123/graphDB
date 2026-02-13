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

/// 从 Arc 中获取可变引用
pub fn expect_arc_mut<T>(
    arc: &mut std::sync::Arc<T>,
) -> Result<&mut T, DBError> {
    std::sync::Arc::get_mut(arc).ok_or_else(|| {
        DBError::Internal("Arc does not have unique ownership".to_string())
    })
}

#[macro_export]
macro_rules! db_return_if_err {
    ($result:expr) => {
        if let Err(e) = $result {
            return Err(e.into());
        }
    };
}

#[macro_export]
macro_rules! db_assert {
    ($condition:expr, $message:expr) => {
        if !$condition {
            Err(DBError::Internal($message.to_string()))
        } else {
            Ok(())
        }
    };
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

    #[test]
    fn test_db_assert_macro() {
        let result = db_assert!(true, "This should pass");
        assert!(result.is_ok());

        let result = db_assert!(false, "This should fail");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DBError::Internal(msg) if msg == "This should fail"));
    }

    #[test]
    fn test_db_return_if_err_macro() {
        fn test_func() -> Result<i32, DBError> {
            let value = 42;
            db_return_if_err!(Ok::<i32, DBError>(value));
            Ok(value)
        }

        assert_eq!(test_func().expect("Expected test function to return 42"), 42);

        fn test_func_err() -> Result<i32, DBError> {
            let result: Result<i32, DBError> = Err(DBError::Internal("test error".to_string()));
            db_return_if_err!(result);
            Ok(0)
        }

        assert!(test_func_err().is_err());
    }
}
