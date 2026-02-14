//! 错误处理辅助函数
//!
//! 提供安全的锁操作和错误传播宏
//!
//! 注意：本项目统一使用 parking_lot 锁
//! parking_lot 的优势：
//! - 更好的性能（2-5 倍于 std::sync）
//! - 无锁污染问题（panic 时自动释放）
//! - 更小的内存占用（1 字节）
//! - 自适应自旋和公平性策略

pub use parking_lot::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// 安全地获取 parking_lot Mutex 锁
/// 
/// parking_lot 的锁不会污染，所以直接返回 guard
/// 这是一个简单的包装，保持 API 一致性
#[inline]
pub fn safe_lock<T>(mutex: &Mutex<T>) -> MutexGuard<T> {
    mutex.lock()
}

/// 安全地获取 parking_lot RwLock 读锁
#[inline]
pub fn safe_read<T>(rwlock: &RwLock<T>) -> RwLockReadGuard<T> {
    rwlock.read()
}

/// 安全地获取 parking_lot RwLock 写锁
#[inline]
pub fn safe_write<T>(rwlock: &RwLock<T>) -> RwLockWriteGuard<T> {
    rwlock.write()
}

/// 兼容层：为 std::sync::Mutex 提供相同的 API
/// 
/// 注意：这是为了兼容旧代码，新项目应直接使用 parking_lot::Mutex
pub mod compat {
    use std::sync::{self, PoisonError};
    use crate::core::error::{DBError, LockError};

    /// 安全地获取 std::sync::Mutex 锁
    /// 
    /// 将锁污染错误转换为 DBError
    pub fn safe_lock_std<T>(
        mutex: &sync::Mutex<T>,
    ) -> Result<sync::MutexGuard<T>, DBError> {
        mutex.lock().map_err(|e: PoisonError<_>| {
            DBError::Lock(LockError::MutexPoisoned {
                reason: format!("Mutex was poisoned: {}", e),
            })
        })
    }

    /// 安全地获取 std::sync::RwLock 读锁
    pub fn safe_read_std<T>(
        rwlock: &sync::RwLock<T>,
    ) -> Result<sync::RwLockReadGuard<T>, DBError> {
        rwlock.read().map_err(|e: PoisonError<_>| {
            DBError::Lock(LockError::RwLockReadPoisoned {
                reason: format!("RwLock was poisoned: {}", e),
            })
        })
    }

    /// 安全地获取 std::sync::RwLock 写锁
    pub fn safe_write_std<T>(
        rwlock: &sync::RwLock<T>,
    ) -> Result<sync::RwLockWriteGuard<T>, DBError> {
        rwlock.write().map_err(|e: PoisonError<_>| {
            DBError::Lock(LockError::RwLockWritePoisoned {
                reason: format!("RwLock was poisoned: {}", e),
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_lock_success() {
        let mutex = Mutex::new(42);
        let guard = safe_lock(&mutex);
        assert_eq!(*guard, 42);
    }

    #[test]
    fn test_safe_read_write() {
        let rwlock = RwLock::new(42);
        
        // Test read
        let guard = safe_read(&rwlock);
        assert_eq!(*guard, 42);
        drop(guard);
        
        // Test write
        let mut guard = safe_write(&rwlock);
        *guard = 100;
        drop(guard);
        
        // Verify write
        let guard = safe_read(&rwlock);
        assert_eq!(*guard, 100);
    }

    #[test]
    fn test_mutex_thread_safety() {
        use std::sync::Arc;
        use std::thread;

        let mutex = Arc::new(Mutex::new(0));
        let mut handles = vec![];

        for _ in 0..10 {
            let m = Arc::clone(&mutex);
            let handle = thread::spawn(move || {
                let mut guard = safe_lock(&m);
                *guard += 1;
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let guard = safe_lock(&mutex);
        assert_eq!(*guard, 10);
    }

    #[test]
    fn test_compat_std_mutex() {
        use std::sync::Mutex as StdMutex;
        use compat::safe_lock_std;

        let mutex = StdMutex::new(42);
        let guard = safe_lock_std(&mutex).expect("Lock should succeed");
        assert_eq!(*guard, 42);
    }
}
