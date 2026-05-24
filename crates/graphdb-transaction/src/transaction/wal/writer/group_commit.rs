//! Group commit manager for batching WAL writes

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};

use crate::transaction::wal::types::{WalError, WalOpType, WalResult};

/// Pending write for group commit
pub(crate) struct PendingWrite {
    pub(crate) op_type: WalOpType,
    pub(crate) timestamp: u32,
    pub(crate) data: Vec<u8>,
    result: Arc<Mutex<WalResult<bool>>>,
    notified: Arc<Condvar>,
}

/// Group commit manager for batching multiple writes
pub struct GroupCommitManager {
    pending_writes: Mutex<VecDeque<PendingWrite>>,
    batch_size_limit: usize,
    commit_delay_us: u64,
    is_leader: AtomicBool,
}

impl GroupCommitManager {
    pub fn new(batch_size_limit: usize, commit_delay_us: u64) -> Self {
        Self {
            pending_writes: Mutex::new(VecDeque::new()),
            batch_size_limit,
            commit_delay_us,
            is_leader: AtomicBool::new(false),
        }
    }

    pub fn submit(&self, op_type: WalOpType, timestamp: u32, data: &[u8]) -> WalResult<bool> {
        let result = Arc::new(Mutex::new(Ok(false)));
        let notified = Arc::new(Condvar::new());

        let pending = PendingWrite {
            op_type,
            timestamp,
            data: data.to_vec(),
            result: result.clone(),
            notified: notified.clone(),
        };

        {
            let mut queue = self
                .pending_writes
                .lock()
                .map_err(|_| WalError::IoError("Failed to lock pending writes".to_string()))?;
            queue.push_back(pending);
        }

        let mut result_guard = result
            .lock()
            .map_err(|_| WalError::IoError("Failed to lock result".to_string()))?;

        loop {
            if let Ok(true) = &*result_guard {
                return Ok(true);
            }
            if let Err(e) = &*result_guard {
                return Err(e.clone());
            }

            result_guard = notified
                .wait_timeout(
                    result_guard,
                    std::time::Duration::from_micros(self.commit_delay_us),
                )
                .map_err(|_| WalError::IoError("Wait timeout error".to_string()))?
                .0;
        }
    }

    pub fn process_batch<F>(&self, write_fn: F) -> WalResult<()>
    where
        F: FnOnce(&[(WalOpType, u32, &[u8])]) -> WalResult<bool>,
    {
        if let Some(batch) = self.collect_batch() {
            if batch.is_empty() {
                return Ok(());
            }

            let entries: Vec<(WalOpType, u32, &[u8])> = batch
                .iter()
                .map(|p| (p.op_type, p.timestamp, p.data.as_slice()))
                .collect();

            let success = write_fn(&entries);

            match success {
                Ok(_) => Self::notify_results(batch, true),
                Err(e) => Self::notify_error(batch, e),
            }
        }
        Ok(())
    }

    pub(crate) fn collect_batch(&self) -> Option<Vec<PendingWrite>> {
        let mut queue = self.pending_writes.lock().ok()?;

        if queue.is_empty() {
            return None;
        }

        let batch_size = queue.len().min(self.batch_size_limit);
        let batch: Vec<PendingWrite> = queue.drain(..batch_size).collect();
        Some(batch)
    }

    pub fn is_leader(&self) -> bool {
        self.is_leader.load(Ordering::SeqCst)
    }

    pub fn set_leader(&self, is_leader: bool) {
        self.is_leader.store(is_leader, Ordering::SeqCst);
    }

    pub fn has_pending(&self) -> bool {
        self.pending_writes
            .lock()
            .map(|q| !q.is_empty())
            .unwrap_or(false)
    }

    fn notify_results(batch: Vec<PendingWrite>, success: bool) {
        for pending in batch {
            if let Ok(mut result) = pending.result.lock() {
                *result = Ok(success);
            }
            pending.notified.notify_all();
        }
    }

    fn notify_error(batch: Vec<PendingWrite>, error: WalError) {
        for pending in batch {
            if let Ok(mut result) = pending.result.lock() {
                *result = Err(error.clone());
            }
            pending.notified.notify_all();
        }
    }
}

impl Default for GroupCommitManager {
    fn default() -> Self {
        Self::new(1024, 100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_commit_manager() {
        let manager = GroupCommitManager::new(10, 100);
        assert!(!manager.has_pending());
    }
}
