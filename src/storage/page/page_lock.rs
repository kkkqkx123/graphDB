//! Page Lock Manager
//!
//! Provides page-level locking for transaction isolation.
//! Supports read-write locks with lock upgrade capability.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use parking_lot::{Mutex, RwLock};

use crate::transaction::types::TransactionId;

/// Page lock identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PageLockId {
    pub table_type: u8,
    pub label_id: u16,
    pub page_number: u64,
}

impl PageLockId {
    pub fn new(table_type: u8, label_id: u16, page_number: u64) -> Self {
        Self {
            table_type,
            label_id,
            page_number,
        }
    }

    pub fn vertex(label_id: u16, page_number: u64) -> Self {
        Self::new(1, label_id, page_number)
    }

    pub fn edge(label_id: u16, page_number: u64) -> Self {
        Self::new(2, label_id, page_number)
    }

    pub fn schema(page_number: u64) -> Self {
        Self::new(4, 0, page_number)
    }
}

/// Lock mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockMode {
    Shared,
    Exclusive,
}

/// Lock request result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockResult {
    Granted,
    Waiting,
    Timeout,
    Deadlock,
}

/// Lock entry tracking holders
#[derive(Debug, Default)]
struct LockEntry {
    shared_holders: Vec<TransactionId>,
    exclusive_holder: Option<TransactionId>,
    waiting_transactions: Vec<(TransactionId, LockMode, Instant)>,
}

impl LockEntry {
    fn is_locked(&self) -> bool {
        !self.shared_holders.is_empty() || self.exclusive_holder.is_some()
    }

    fn is_exclusive_locked(&self) -> bool {
        self.exclusive_holder.is_some()
    }

    fn holds_lock(&self, txn_id: TransactionId) -> Option<LockMode> {
        if self.exclusive_holder == Some(txn_id) {
            return Some(LockMode::Exclusive);
        }
        if self.shared_holders.contains(&txn_id) {
            return Some(LockMode::Shared);
        }
        None
    }
}

/// Page lock manager configuration
#[derive(Debug, Clone)]
pub struct PageLockConfig {
    pub lock_timeout: Duration,
    pub deadlock_detection_interval: Duration,
    pub max_waiters: usize,
}

impl Default for PageLockConfig {
    fn default() -> Self {
        Self {
            lock_timeout: Duration::from_secs(30),
            deadlock_detection_interval: Duration::from_secs(1),
            max_waiters: 100,
        }
    }
}

/// Page lock manager for transaction isolation
pub struct PageLockManager {
    locks: RwLock<HashMap<PageLockId, LockEntry>>,
    config: PageLockConfig,
    stats: Mutex<LockStats>,
}

#[derive(Debug, Default, Clone)]
pub struct LockStats {
    pub total_locks: u64,
    pub shared_locks: u64,
    pub exclusive_locks: u64,
    pub lock_waits: u64,
    pub lock_timeouts: u64,
    pub deadlocks_detected: u64,
    pub lock_upgrades: u64,
}

impl PageLockManager {
    pub fn new() -> Self {
        Self::with_config(PageLockConfig::default())
    }

    pub fn with_config(config: PageLockConfig) -> Self {
        Self {
            locks: RwLock::new(HashMap::new()),
            config,
            stats: Mutex::new(LockStats::default()),
        }
    }

    /// Acquire a lock on a page
    pub fn acquire_lock(
        &self,
        page_id: PageLockId,
        txn_id: TransactionId,
        mode: LockMode,
    ) -> LockResult {
        self.acquire_lock_with_timeout(page_id, txn_id, mode, self.config.lock_timeout)
    }

    /// Acquire a lock with custom timeout
    pub fn acquire_lock_with_timeout(
        &self,
        page_id: PageLockId,
        txn_id: TransactionId,
        mode: LockMode,
        timeout: Duration,
    ) -> LockResult {
        let start = Instant::now();

        loop {
            {
                let mut locks = self.locks.write();
                let entry = locks.entry(page_id).or_default();

                if let Some(current_mode) = entry.holds_lock(txn_id) {
                    match (current_mode, mode) {
                        (LockMode::Exclusive, _) => return LockResult::Granted,
                        (LockMode::Shared, LockMode::Shared) => return LockResult::Granted,
                        (LockMode::Shared, LockMode::Exclusive) => {
                            if entry.shared_holders.len() == 1 {
                                entry.shared_holders.clear();
                                entry.exclusive_holder = Some(txn_id);
                                self.stats.lock().lock_upgrades += 1;
                                return LockResult::Granted;
                            }
                        }
                    }
                }

                let can_grant = match mode {
                    LockMode::Shared => !entry.is_exclusive_locked(),
                    LockMode::Exclusive => !entry.is_locked(),
                };

                if can_grant {
                    match mode {
                        LockMode::Shared => {
                            entry.shared_holders.push(txn_id);
                            self.stats.lock().shared_locks += 1;
                        }
                        LockMode::Exclusive => {
                            entry.exclusive_holder = Some(txn_id);
                            self.stats.lock().exclusive_locks += 1;
                        }
                    }
                    self.stats.lock().total_locks += 1;
                    return LockResult::Granted;
                }

                if entry.waiting_transactions.len() >= self.config.max_waiters {
                    return LockResult::Deadlock;
                }

                if entry.waiting_transactions.iter().any(|(id, _, _)| *id == txn_id) {
                    let elapsed = start.elapsed();
                    if elapsed >= timeout {
                        self.stats.lock().lock_timeouts += 1;
                        return LockResult::Timeout;
                    }
                    self.stats.lock().lock_waits += 1;
                } else {
                    entry.waiting_transactions.push((txn_id, mode, Instant::now()));
                    self.stats.lock().lock_waits += 1;
                }
            }

            std::thread::sleep(Duration::from_micros(100));

            if start.elapsed() >= timeout {
                self.stats.lock().lock_timeouts += 1;
                return LockResult::Timeout;
            }
        }
    }

    /// Release a lock held by a transaction
    pub fn release_lock(&self, page_id: PageLockId, txn_id: TransactionId) -> bool {
        let mut locks = self.locks.write();

        if let Some(entry) = locks.get_mut(&page_id) {
            let was_holder = if entry.exclusive_holder == Some(txn_id) {
                entry.exclusive_holder = None;
                true
            } else if let Some(pos) = entry.shared_holders.iter().position(|&id| id == txn_id) {
                entry.shared_holders.remove(pos);
                true
            } else {
                false
            };

            entry.waiting_transactions.retain(|(id, _, _)| *id != txn_id);

            if !entry.is_locked() && entry.waiting_transactions.is_empty() {
                locks.remove(&page_id);
            }

            return was_holder;
        }

        false
    }

    /// Release all locks held by a transaction
    pub fn release_all_locks(&self, txn_id: TransactionId) -> usize {
        let mut locks = self.locks.write();
        let mut released = 0;

        let pages_to_clean: Vec<PageLockId> = locks
            .iter()
            .filter_map(|(page_id, entry)| {
                if entry.holds_lock(txn_id).is_some() {
                    Some(*page_id)
                } else {
                    None
                }
            })
            .collect();

        for page_id in pages_to_clean {
            if let Some(entry) = locks.get_mut(&page_id) {
                if entry.exclusive_holder == Some(txn_id) {
                    entry.exclusive_holder = None;
                    released += 1;
                } else if let Some(pos) = entry.shared_holders.iter().position(|&id| id == txn_id) {
                    entry.shared_holders.remove(pos);
                    released += 1;
                }

                entry.waiting_transactions.retain(|(id, _, _)| *id != txn_id);

                if !entry.is_locked() && entry.waiting_transactions.is_empty() {
                    locks.remove(&page_id);
                }
            }
        }

        released
    }

    /// Check if a page is locked
    pub fn is_locked(&self, page_id: &PageLockId) -> bool {
        let locks = self.locks.read();
        locks.get(page_id).map(|e| e.is_locked()).unwrap_or(false)
    }

    /// Check if a page is exclusively locked
    pub fn is_exclusive_locked(&self, page_id: &PageLockId) -> bool {
        let locks = self.locks.read();
        locks
            .get(page_id)
            .map(|e| e.is_exclusive_locked())
            .unwrap_or(false)
    }

    /// Get the lock mode held by a transaction
    pub fn get_lock_mode(&self, page_id: &PageLockId, txn_id: TransactionId) -> Option<LockMode> {
        let locks = self.locks.read();
        locks.get(page_id).and_then(|e| e.holds_lock(txn_id))
    }

    /// Get lock statistics
    pub fn stats(&self) -> LockStats {
        self.stats.lock().clone()
    }

    /// Get the number of locked pages
    pub fn locked_page_count(&self) -> usize {
        self.locks.read().len()
    }

    /// Clear all locks (use with caution)
    pub fn clear(&self) {
        self.locks.write().clear();
    }
}

impl Default for PageLockManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_lock() {
        let manager = PageLockManager::new();
        let page_id = PageLockId::vertex(1, 0);

        let result = manager.acquire_lock(page_id, 1, LockMode::Shared);
        assert_eq!(result, LockResult::Granted);

        let result = manager.acquire_lock(page_id, 2, LockMode::Shared);
        assert_eq!(result, LockResult::Granted);

        assert!(manager.is_locked(&page_id));
        assert!(!manager.is_exclusive_locked(&page_id));
    }

    #[test]
    fn test_exclusive_lock() {
        let manager = PageLockManager::new();
        let page_id = PageLockId::vertex(1, 0);

        let result = manager.acquire_lock(page_id, 1, LockMode::Exclusive);
        assert_eq!(result, LockResult::Granted);

        assert!(manager.is_exclusive_locked(&page_id));
    }

    #[test]
    fn test_lock_conflict() {
        let config = PageLockConfig {
            lock_timeout: Duration::from_millis(100),
            ..Default::default()
        };
        let manager = PageLockManager::with_config(config);
        let page_id = PageLockId::vertex(1, 0);

        let result = manager.acquire_lock(page_id, 1, LockMode::Shared);
        assert_eq!(result, LockResult::Granted);

        let result = manager.acquire_lock_with_timeout(
            page_id,
            2,
            LockMode::Exclusive,
            Duration::from_millis(50),
        );
        assert_eq!(result, LockResult::Timeout);
    }

    #[test]
    fn test_lock_upgrade() {
        let manager = PageLockManager::new();
        let page_id = PageLockId::vertex(1, 0);

        let result = manager.acquire_lock(page_id, 1, LockMode::Shared);
        assert_eq!(result, LockResult::Granted);

        let result = manager.acquire_lock(page_id, 1, LockMode::Exclusive);
        assert_eq!(result, LockResult::Granted);

        assert!(manager.is_exclusive_locked(&page_id));
    }

    #[test]
    fn test_release_lock() {
        let manager = PageLockManager::new();
        let page_id = PageLockId::vertex(1, 0);

        manager.acquire_lock(page_id, 1, LockMode::Exclusive);
        assert!(manager.is_locked(&page_id));

        let released = manager.release_lock(page_id, 1);
        assert!(released);
        assert!(!manager.is_locked(&page_id));
    }

    #[test]
    fn test_release_all_locks() {
        let manager = PageLockManager::new();

        manager.acquire_lock(PageLockId::vertex(1, 0), 1, LockMode::Exclusive);
        manager.acquire_lock(PageLockId::vertex(1, 1), 1, LockMode::Shared);
        manager.acquire_lock(PageLockId::edge(2, 0), 1, LockMode::Exclusive);

        assert_eq!(manager.locked_page_count(), 3);

        let released = manager.release_all_locks(1);
        assert_eq!(released, 3);
        assert_eq!(manager.locked_page_count(), 0);
    }

    #[test]
    fn test_lock_stats() {
        let manager = PageLockManager::new();
        let page_id = PageLockId::vertex(1, 0);

        manager.acquire_lock(page_id, 1, LockMode::Shared);
        manager.acquire_lock(page_id, 2, LockMode::Shared);
        manager.acquire_lock(PageLockId::edge(2, 0), 1, LockMode::Exclusive);

        let stats = manager.stats();
        assert_eq!(stats.shared_locks, 2);
        assert_eq!(stats.exclusive_locks, 1);
        assert_eq!(stats.total_locks, 3);
    }
}
