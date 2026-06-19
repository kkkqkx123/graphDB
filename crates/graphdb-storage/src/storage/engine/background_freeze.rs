//! Background Delta Freezing Manager
//!
//! Periodically converts mutable CSR deltas into immutable segments to:
//! - Reduce memory fragmentation (no overflow blocks in immutable segments)
//! - Lower query latency (fewer segment scans)
//! - Improve cache locality (immutable CSRs are flatter)
//!
//! ## Design
//!
//! - Single background thread checks delta sizes at fixed intervals
//! - Freezing triggered when delta edges exceed threshold
//! - Non-blocking: freezing doesn't hold write locks on other tables
//! - Graceful shutdown: thread joins cleanly on drop
//!
//! ## Example
//!
//! ```ignore
//! let config = BackgroundFreezeConfig::default();
//! let manager = BackgroundFreezeManager::new(storage.clone(), config);
//! // Background thread starts automatically
//! // On drop, thread joins gracefully
//! ```

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use parking_lot::Mutex;

use crate::core::types::Timestamp;
use crate::core::StorageResult;

/// Configuration for background freezing behavior
#[derive(Debug, Clone)]
pub struct BackgroundFreezeConfig {
    /// Freeze when mutable delta edges exceed this count
    pub delta_edge_threshold: u64,
    /// Check interval in milliseconds
    pub check_interval_ms: u64,
    /// Enable automatic segment merging after freeze
    pub enable_segment_merge: bool,
    /// Merge segments if count >= this threshold
    pub segment_merge_threshold: u32,
}

impl Default for BackgroundFreezeConfig {
    fn default() -> Self {
        Self {
            delta_edge_threshold: 100_000,      // 100K edges
            check_interval_ms: 5 * 60 * 1000,   // 5 minutes
            enable_segment_merge: true,
            segment_merge_threshold: 3,         // >= 3 segments to merge
        }
    }
}

/// Statistics about background freezing operations
#[derive(Debug, Clone, Copy)]
pub struct FreezeStats {
    /// Total number of freeze operations completed
    pub freeze_count: u64,
    /// Total edges frozen across all operations
    pub total_frozen_edges: u64,
    /// Duration of last freeze operation in milliseconds
    pub last_freeze_duration_ms: u64,
    /// Current mutable delta edges (unfroken)
    pub current_delta_edges: u64,
}

/// Background freezing manager
///
/// Runs a background thread that periodically checks edge tables
/// and freezes deltas when they exceed the configured threshold.
pub struct BackgroundFreezeManager {
    /// Configuration
    config: Arc<BackgroundFreezeConfig>,
    /// Statistics (atomic for thread-safe reads)
    stats: Arc<Mutex<FreezeStats>>,
    /// Flag to signal shutdown
    should_stop: Arc<AtomicBool>,
    /// Background thread handle
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl BackgroundFreezeManager {
    /// Create a new background freeze manager (thread starts immediately)
    pub fn new(config: BackgroundFreezeConfig) -> Self {
        let config = Arc::new(config);
        let stats = Arc::new(Mutex::new(FreezeStats {
            freeze_count: 0,
            total_frozen_edges: 0,
            last_freeze_duration_ms: 0,
            current_delta_edges: 0,
        }));
        let should_stop = Arc::new(AtomicBool::new(false));

        let config_clone = Arc::clone(&config);
        let stats_clone = Arc::clone(&stats);
        let stop_clone = Arc::clone(&should_stop);

        let handle = thread::spawn(move || {
            Self::background_freeze_loop(&config_clone, &stats_clone, &stop_clone);
        });

        Self {
            config,
            stats,
            should_stop,
            thread_handle: Some(handle),
        }
    }

    /// Background thread main loop
    fn background_freeze_loop(
        config: &Arc<BackgroundFreezeConfig>,
        stats: &Arc<Mutex<FreezeStats>>,
        should_stop: &Arc<AtomicBool>,
    ) {
        let check_interval = Duration::from_millis(config.check_interval_ms);

        loop {
            if should_stop.load(Ordering::Relaxed) {
                log::info!("Background freeze thread shutting down");
                break;
            }

            thread::sleep(check_interval);

            // In a real implementation, this would:
            // 1. Query current storage state
            // 2. Iterate edge tables
            // 3. Check delta sizes
            // 4. Trigger freeze if threshold exceeded
            // 5. Update statistics
            //
            // For now, just log heartbeat
            log::trace!("Background freeze check cycle");
        }
    }

    /// Get current freeze statistics (snapshot)
    pub fn get_stats(&self) -> FreezeStats {
        *self.stats.lock()
    }

    /// Manually record a freeze event (for integration with storage)
    pub(crate) fn record_freeze(&self, delta_edges: u64, duration_ms: u64) {
        let mut stats = self.stats.lock();
        stats.freeze_count += 1;
        stats.total_frozen_edges += delta_edges;
        stats.last_freeze_duration_ms = duration_ms;
    }

    /// Manually record current delta size
    pub(crate) fn record_delta_size(&self, delta_edges: u64) {
        let mut stats = self.stats.lock();
        stats.current_delta_edges = delta_edges;
    }

    /// Get configuration reference
    pub fn config(&self) -> &BackgroundFreezeConfig {
        &self.config
    }
}

impl Drop for BackgroundFreezeManager {
    fn drop(&mut self) {
        // Signal shutdown
        self.should_stop.store(true, Ordering::Release);

        // Wait for thread to finish
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }

        log::debug!("Background freeze manager shut down");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation_and_drop() {
        let config = BackgroundFreezeConfig::default();
        let manager = BackgroundFreezeManager::new(config);

        let stats = manager.get_stats();
        assert_eq!(stats.freeze_count, 0);
    }

    #[test]
    fn test_record_freeze() {
        let config = BackgroundFreezeConfig::default();
        let manager = BackgroundFreezeManager::new(config);

        manager.record_freeze(50_000, 100);
        let stats = manager.get_stats();
        assert_eq!(stats.freeze_count, 1);
        assert_eq!(stats.total_frozen_edges, 50_000);
        assert_eq!(stats.last_freeze_duration_ms, 100);

        manager.record_freeze(30_000, 50);
        let stats = manager.get_stats();
        assert_eq!(stats.freeze_count, 2);
        assert_eq!(stats.total_frozen_edges, 80_000);
        assert_eq!(stats.last_freeze_duration_ms, 50);
    }

    #[test]
    fn test_graceful_shutdown() {
        let config = BackgroundFreezeConfig {
            check_interval_ms: 100,
            ..Default::default()
        };

        let manager = BackgroundFreezeManager::new(config);
        // Manager should drop cleanly without hanging
        drop(manager);
    }
}
