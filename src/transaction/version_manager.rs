//! MVCC Version Manager
//!
//! Provides timestamp management for MVCC (Multi-Version Concurrency Control)
//! based transaction isolation.

use std::collections::HashSet;
use std::sync::atomic::{AtomicI32, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

use super::wal::types::Timestamp;

/// Default ring buffer size for timestamp tracking
const RING_BUF_SIZE: u32 = 1024 * 1024;
/// Ring buffer index mask
const RING_INDEX_MASK: u32 = RING_BUF_SIZE - 1;

/// Version manager error
#[derive(Debug, Clone, thiserror::Error)]
pub enum VersionManagerError {
    #[error("Too many concurrent transactions")]
    TooManyTransactions,

    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(Timestamp),

    #[error("Update transaction already in progress")]
    UpdateInProgress,

    #[error("Timeout waiting for transaction")]
    Timeout,
}

/// Version manager result type
pub type VersionManagerResult<T> = Result<T, VersionManagerError>;

/// Bit set for timestamp tracking
#[derive(Debug, Default)]
struct BitSet {
    data: RwLock<Vec<u64>>,
}

impl BitSet {
    fn new(size: usize) -> Self {
        let word_count = (size + 63) / 64;
        Self {
            data: RwLock::new(vec![0u64; word_count]),
        }
    }

    fn set(&self, index: u32) {
        let word = index as usize / 64;
        let bit = index as usize % 64;
        if let Ok(mut data) = self.data.write() {
            if word < data.len() {
                data[word] |= 1u64 << bit;
            }
        }
    }

    fn reset(&self, index: u32) {
        let word = index as usize / 64;
        let bit = index as usize % 64;
        if let Ok(mut data) = self.data.write() {
            if word < data.len() {
                data[word] &= !(1u64 << bit);
            }
        }
    }

    fn test(&self, index: u32) -> bool {
        let word = index as usize / 64;
        let bit = index as usize % 64;
        if let Ok(data) = self.data.read() {
            if word < data.len() {
                return (data[word] & (1u64 << bit)) != 0;
            }
        }
        false
    }

    fn atomic_reset_with_ret(&self, index: u32) -> bool {
        let word = index as usize / 64;
        let bit = index as usize % 64;
        if let Ok(mut data) = self.data.write() {
            if word < data.len() {
                let mask = 1u64 << bit;
                let was_set = (data[word] & mask) != 0;
                if was_set {
                    data[word] &= !mask;
                    return true;
                }
            }
        }
        false
    }

    fn reset_all(&self) {
        if let Ok(mut data) = self.data.write() {
            for word in data.iter_mut() {
                *word = 0;
            }
        }
    }
}

/// Version manager configuration
#[derive(Debug, Clone)]
pub struct VersionManagerConfig {
    /// Maximum concurrent read transactions
    pub max_concurrent_reads: u32,
    /// Maximum concurrent insert transactions
    pub max_concurrent_inserts: u32,
    /// Maximum concurrent update transactions (usually 1)
    pub max_concurrent_updates: u32,
    /// Thread count for update transaction blocking
    pub thread_num: i32,
    /// Wait timeout for acquiring timestamps
    pub wait_timeout: Duration,
}

impl Default for VersionManagerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_reads: 1000,
            max_concurrent_inserts: 100,
            max_concurrent_updates: 1,
            thread_num: 1,
            wait_timeout: Duration::from_secs(30),
        }
    }
}

impl VersionManagerConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_concurrent_reads(mut self, max: u32) -> Self {
        self.max_concurrent_reads = max;
        self
    }

    pub fn with_max_concurrent_inserts(mut self, max: u32) -> Self {
        self.max_concurrent_inserts = max;
        self
    }

    pub fn with_thread_num(mut self, num: i32) -> Self {
        self.thread_num = num;
        self
    }
}

/// MVCC Version Manager
///
/// Manages timestamps for read, insert, and update transactions.
/// Implements snapshot isolation through timestamp-based versioning.
pub struct VersionManager {
    /// Next write timestamp
    write_ts: AtomicU32,
    /// Current read timestamp
    read_ts: AtomicU32,
    /// Pending transaction count
    pending_reqs: AtomicI32,
    /// Pending update transaction count
    pending_update_reqs: AtomicI32,
    /// Thread count for update blocking
    thread_num: AtomicI32,
    /// Timestamp buffer for tracking completed transactions
    buffer: BitSet,
    /// Lock for timestamp updates
    lock: Mutex<()>,
    /// Configuration
    config: VersionManagerConfig,
}

impl VersionManager {
    /// Create a new version manager
    pub fn new() -> Self {
        Self::with_config(VersionManagerConfig::default())
    }

    /// Create a new version manager with configuration
    pub fn with_config(config: VersionManagerConfig) -> Self {
        let thread_num = config.thread_num;
        Self {
            write_ts: AtomicU32::new(1),
            read_ts: AtomicU32::new(0),
            pending_reqs: AtomicI32::new(0),
            pending_update_reqs: AtomicI32::new(0),
            thread_num: AtomicI32::new(thread_num),
            buffer: BitSet::new(RING_BUF_SIZE as usize),
            lock: Mutex::new(()),
            config,
        }
    }

    /// Initialize timestamps from a recovered state
    pub fn init_ts(&self, ts: Timestamp, thread_num: i32) {
        self.write_ts.store(ts + 1, Ordering::SeqCst);
        self.read_ts.store(ts, Ordering::SeqCst);
        self.thread_num.store(thread_num, Ordering::SeqCst);
    }

    /// Clear all state
    pub fn clear(&self) {
        self.write_ts.store(1, Ordering::SeqCst);
        self.read_ts.store(0, Ordering::SeqCst);
        self.pending_reqs.store(0, Ordering::SeqCst);
        self.pending_update_reqs.store(0, Ordering::SeqCst);
        self.buffer.reset_all();
    }

    /// Get current write timestamp (next timestamp to be assigned)
    pub fn write_timestamp(&self) -> Timestamp {
        self.write_ts.load(Ordering::SeqCst)
    }

    /// Get current read timestamp (last committed timestamp)
    pub fn read_timestamp(&self) -> Timestamp {
        self.read_ts.load(Ordering::SeqCst)
    }

    /// Acquire a read timestamp
    ///
    /// Returns a timestamp that represents a consistent snapshot
    /// of the database at that point in time.
    pub fn acquire_read_timestamp(&self) -> Timestamp {
        loop {
            let pr = self.pending_reqs.fetch_add(1, Ordering::SeqCst);
            if pr >= 0 {
                return self.read_ts.load(Ordering::SeqCst);
            }
            self.pending_reqs.fetch_sub(1, Ordering::SeqCst);

            thread::sleep(Duration::from_micros(100));
        }
    }

    /// Release a read timestamp
    pub fn release_read_timestamp(&self) {
        self.pending_reqs.fetch_sub(1, Ordering::SeqCst);
    }

    /// Acquire an insert timestamp
    ///
    /// Returns a unique timestamp for an insert transaction.
    pub fn acquire_insert_timestamp(&self) -> Timestamp {
        loop {
            let pr = self.pending_reqs.fetch_add(1, Ordering::SeqCst);
            if pr >= 0 {
                return self.write_ts.fetch_add(1, Ordering::SeqCst);
            }
            self.pending_reqs.fetch_sub(1, Ordering::SeqCst);

            thread::sleep(Duration::from_micros(100));
        }
    }

    /// Release an insert timestamp
    ///
    /// Updates the read timestamp if this was the next expected timestamp.
    pub fn release_insert_timestamp(&self, ts: Timestamp) {
        let _guard = self.lock.lock().unwrap();

        if ts == self.read_ts.load(Ordering::SeqCst) + 1 {
            while self.buffer.atomic_reset_with_ret((ts + 1) & RING_INDEX_MASK) {
                // Continue advancing read timestamp
            }
            self.read_ts.store(ts, Ordering::SeqCst);
        } else {
            self.buffer.set(ts & RING_INDEX_MASK);
        }

        self.pending_reqs.fetch_sub(1, Ordering::SeqCst);
    }

    /// Acquire an update timestamp
    ///
    /// Update transactions require exclusive access and will block
    /// until all other transactions complete.
    pub fn acquire_update_timestamp(&self) -> VersionManagerResult<Timestamp> {
        let mut expected = 0;
        while self
            .pending_update_reqs
            .compare_exchange_weak(0, 1, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            expected = 0;
            thread::sleep(Duration::from_micros(100));
        }

        let pr = self
            .pending_reqs
            .fetch_sub(self.thread_num.load(Ordering::SeqCst), Ordering::SeqCst);
        if pr != 0 {
            while self.pending_reqs.load(Ordering::SeqCst) != -self.thread_num.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_micros(100));
            }
        }

        Ok(self.write_ts.fetch_add(1, Ordering::SeqCst))
    }

    /// Release an update timestamp
    pub fn release_update_timestamp(&self, ts: Timestamp) {
        let _guard = self.lock.lock().unwrap();

        if ts == self.read_ts.load(Ordering::SeqCst) + 1 {
            self.read_ts.store(ts, Ordering::SeqCst);
        } else {
            self.buffer.set(ts & RING_INDEX_MASK);
        }

        self.pending_reqs
            .fetch_add(self.thread_num.load(Ordering::SeqCst), Ordering::SeqCst);
        self.pending_update_reqs.store(0, Ordering::SeqCst);
    }

    /// Revert an update timestamp (for aborted transactions)
    ///
    /// Returns true if the timestamp was successfully reverted.
    pub fn revert_update_timestamp(&self, ts: Timestamp) -> bool {
        let expected = ts + 1;
        if self
            .write_ts
            .compare_exchange(expected, ts, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            self.pending_reqs
                .fetch_add(self.thread_num.load(Ordering::SeqCst), Ordering::SeqCst);
            self.pending_update_reqs.store(0, Ordering::SeqCst);
            return true;
        }
        false
    }

    /// Get pending request count
    pub fn pending_count(&self) -> i32 {
        self.pending_reqs.load(Ordering::SeqCst)
    }

    /// Check if an update transaction is in progress
    pub fn is_update_in_progress(&self) -> bool {
        self.pending_update_reqs.load(Ordering::SeqCst) > 0
    }
}

impl Default for VersionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII guard for read timestamp
pub struct ReadTimestampGuard {
    version_manager: Arc<VersionManager>,
    timestamp: Timestamp,
}

impl ReadTimestampGuard {
    pub fn new(version_manager: Arc<VersionManager>) -> Self {
        let timestamp = version_manager.acquire_read_timestamp();
        Self {
            version_manager,
            timestamp,
        }
    }

    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }
}

impl Drop for ReadTimestampGuard {
    fn drop(&mut self) {
        self.version_manager.release_read_timestamp();
    }
}

/// RAII guard for insert timestamp
pub struct InsertTimestampGuard {
    version_manager: Arc<VersionManager>,
    timestamp: Option<Timestamp>,
}

impl InsertTimestampGuard {
    pub fn new(version_manager: Arc<VersionManager>) -> Self {
        let timestamp = version_manager.acquire_insert_timestamp();
        Self {
            version_manager,
            timestamp: Some(timestamp),
        }
    }

    pub fn timestamp(&self) -> Timestamp {
        self.timestamp.unwrap_or(0)
    }

    /// Commit and release the timestamp
    pub fn commit(mut self) {
        if let Some(ts) = self.timestamp.take() {
            self.version_manager.release_insert_timestamp(ts);
        }
    }

    /// Abort and release the timestamp
    pub fn abort(mut self) {
        if let Some(ts) = self.timestamp.take() {
            self.version_manager.release_insert_timestamp(ts);
        }
    }
}

impl Drop for InsertTimestampGuard {
    fn drop(&mut self) {
        if let Some(ts) = self.timestamp.take() {
            self.version_manager.release_insert_timestamp(ts);
        }
    }
}

/// RAII guard for update timestamp
pub struct UpdateTimestampGuard {
    version_manager: Arc<VersionManager>,
    timestamp: Option<Timestamp>,
}

impl UpdateTimestampGuard {
    pub fn new(version_manager: Arc<VersionManager>) -> VersionManagerResult<Self> {
        let timestamp = version_manager.acquire_update_timestamp()?;
        Ok(Self {
            version_manager,
            timestamp: Some(timestamp),
        })
    }

    pub fn timestamp(&self) -> Timestamp {
        self.timestamp.unwrap_or(0)
    }

    /// Commit and release the timestamp
    pub fn commit(mut self) {
        if let Some(ts) = self.timestamp.take() {
            self.version_manager.release_update_timestamp(ts);
        }
    }

    /// Abort and revert the timestamp
    pub fn abort(mut self) {
        if let Some(ts) = self.timestamp.take() {
            self.version_manager.revert_update_timestamp(ts);
        }
    }
}

impl Drop for UpdateTimestampGuard {
    fn drop(&mut self) {
        if let Some(ts) = self.timestamp.take() {
            self.version_manager.release_update_timestamp(ts);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_version_manager_basic() {
        let vm = VersionManager::new();

        let ts1 = vm.acquire_read_timestamp();
        assert_eq!(ts1, 0);
        vm.release_read_timestamp();

        let ts2 = vm.acquire_insert_timestamp();
        assert!(ts2 >= 1);
        vm.release_insert_timestamp(ts2);
    }

    #[test]
    fn test_read_timestamp_guard() {
        let vm = Arc::new(VersionManager::new());

        {
            let guard = ReadTimestampGuard::new(vm.clone());
            assert_eq!(guard.timestamp(), 0);
        }

        assert_eq!(vm.pending_count(), 0);
    }

    #[test]
    fn test_insert_timestamp_guard() {
        let vm = Arc::new(VersionManager::new());

        {
            let guard = InsertTimestampGuard::new(vm.clone());
            let ts = guard.timestamp();
            assert!(ts >= 1);
        }

        assert_eq!(vm.pending_count(), 0);
    }

    #[test]
    fn test_update_timestamp_guard() {
        let vm = Arc::new(VersionManager::new());

        {
            let guard = UpdateTimestampGuard::new(vm.clone()).expect("Failed to acquire update");
            let ts = guard.timestamp();
            assert!(ts >= 1);
        }

        assert!(!vm.is_update_in_progress());
    }

    #[test]
    fn test_concurrent_reads() {
        let vm = Arc::new(VersionManager::new());
        let mut handles = vec![];

        for _ in 0..10 {
            let vm_clone = vm.clone();
            handles.push(thread::spawn(move || {
                let guard = ReadTimestampGuard::new(vm_clone);
                thread::sleep(Duration::from_millis(10));
                guard.timestamp()
            }));
        }

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        assert!(results.iter().all(|&ts| ts == 0));
    }

    #[test]
    fn test_concurrent_inserts() {
        let vm = Arc::new(VersionManager::new());
        let mut handles = vec![];

        for _ in 0..10 {
            let vm_clone = vm.clone();
            handles.push(thread::spawn(move || {
                let guard = InsertTimestampGuard::new(vm_clone);
                let ts = guard.timestamp();
                thread::sleep(Duration::from_millis(10));
                ts
            }));
        }

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        let unique: HashSet<_> = results.into_iter().collect();
        assert_eq!(unique.len(), 10);
    }
}
