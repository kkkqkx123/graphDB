//! Freeze Statistics Collector
//!
//! Collects metrics about delta freezing operations and provides
//! configuration for freeze decision-making.
//!
//! Does NOT execute freezing — that's handled by GraphStorageContext
//! via trigger_background_freeze() method.
//!
//! ## Design
//!
//! BackgroundFreezeManager provides:
//! - Configuration for freeze thresholds (delta_edge_threshold)
//! - Decision support (should_freeze check)
//! - Statistics collection (record_freeze, record_delta_size)
//!
//! Actual freezing is triggered by:
//! - GraphStorageContext::trigger_background_freeze() during maintenance
//! - Checkpoint operations before persistence
//! - HTTP API endpoint for manual triggering
//!
//! ## Example
//!
//! ```ignore
//! let manager = BackgroundFreezeManager::new(config);
//!
//! // Check if freezing should be triggered
//! if manager.should_freeze(table.delta_edge_count()) {
//!     // Call trigger_background_freeze() externally
//!     let frozen = table.compact_and_freeze_with_auto_gc(ts, &config);
//!     manager.record_freeze(frozen, duration_ms);
//! }
//! ```

use std::sync::Arc;

use parking_lot::Mutex;

/// Configuration for freeze decision-making
#[derive(Debug, Clone)]
pub struct BackgroundFreezeConfig {
    /// Freeze when mutable delta edges exceed this count
    pub delta_edge_threshold: u64,
}

impl Default for BackgroundFreezeConfig {
    fn default() -> Self {
        Self {
            delta_edge_threshold: 100_000,  // 100K edges
        }
    }
}

/// Statistics about freeze operations
#[derive(Debug, Clone, Copy, Default)]
pub struct FreezeStats {
    /// Total number of freeze operations completed
    pub freeze_count: u64,
    /// Total edges frozen across all operations
    pub total_frozen_edges: u64,
    /// Duration of last freeze operation in milliseconds
    pub last_freeze_duration_ms: u64,
    /// Current mutable delta edges (unfrozen)
    pub current_delta_edges: u64,
}

/// Freeze decision information
#[derive(Debug, Clone)]
pub struct FreezeDecision {
    pub should_freeze: bool,
    pub current_delta_edges: u64,
    pub threshold: u64,
}

/// Freeze statistics collector and decision maker
///
/// Provides configuration, decision support, and metrics collection
/// for delta freezing operations. The actual freezing is executed
/// by GraphStorageContext via trigger_background_freeze().
pub struct BackgroundFreezeManager {
    /// Configuration
    config: Arc<BackgroundFreezeConfig>,
    /// Statistics (thread-safe for concurrent reads)
    stats: Arc<Mutex<FreezeStats>>,
}

impl BackgroundFreezeManager {
    /// Create a new freeze manager
    pub fn new(config: BackgroundFreezeConfig) -> Self {
        Self {
            config: Arc::new(config),
            stats: Arc::new(Mutex::new(FreezeStats::default())),
        }
    }

    /// Check if freezing should be triggered
    pub fn should_freeze(&self, current_delta_edges: u64) -> bool {
        current_delta_edges >= self.config.delta_edge_threshold
    }

    /// Get detailed freeze decision
    pub fn get_freeze_decision(&self, current_delta_edges: u64) -> FreezeDecision {
        FreezeDecision {
            should_freeze: self.should_freeze(current_delta_edges),
            current_delta_edges,
            threshold: self.config.delta_edge_threshold,
        }
    }

    /// Get current freeze statistics (snapshot)
    pub fn get_stats(&self) -> FreezeStats {
        *self.stats.lock()
    }

    /// Record a freeze event (called by GraphStorageContext after freezing)
    pub(crate) fn record_freeze(&self, edges_frozen: u64, duration_ms: u64) {
        let mut stats = self.stats.lock();
        stats.freeze_count += 1;
        stats.total_frozen_edges += edges_frozen;
        stats.last_freeze_duration_ms = duration_ms;
    }

    /// Record current delta size (for monitoring)
    pub(crate) fn record_delta_size(&self, delta_edges: u64) {
        let mut stats = self.stats.lock();
        stats.current_delta_edges = delta_edges;
    }

    /// Get configuration reference
    pub fn config(&self) -> &BackgroundFreezeConfig {
        &self.config
    }
}

impl Default for BackgroundFreezeManager {
    fn default() -> Self {
        Self::new(BackgroundFreezeConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_freeze_decision_below_threshold() {
        let config = BackgroundFreezeConfig {
            delta_edge_threshold: 100_000,
        };
        let manager = BackgroundFreezeManager::new(config);

        let decision = manager.get_freeze_decision(50_000);
        assert!(!decision.should_freeze);
        assert_eq!(decision.current_delta_edges, 50_000);
        assert_eq!(decision.threshold, 100_000);
    }

    #[test]
    fn test_freeze_decision_at_threshold() {
        let config = BackgroundFreezeConfig {
            delta_edge_threshold: 100_000,
        };
        let manager = BackgroundFreezeManager::new(config);

        let decision = manager.get_freeze_decision(100_000);
        assert!(decision.should_freeze);
    }

    #[test]
    fn test_freeze_decision_above_threshold() {
        let config = BackgroundFreezeConfig {
            delta_edge_threshold: 100_000,
        };
        let manager = BackgroundFreezeManager::new(config);

        let decision = manager.get_freeze_decision(150_000);
        assert!(decision.should_freeze);
    }

    #[test]
    fn test_should_freeze_method() {
        let manager = BackgroundFreezeManager::default();
        assert!(!manager.should_freeze(50_000));
        assert!(manager.should_freeze(100_000));
        assert!(manager.should_freeze(150_000));
    }

    #[test]
    fn test_record_freeze() {
        let manager = BackgroundFreezeManager::default();

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
    fn test_record_delta_size() {
        let manager = BackgroundFreezeManager::default();

        manager.record_delta_size(12_000);
        let stats = manager.get_stats();
        assert_eq!(stats.current_delta_edges, 12_000);

        manager.record_delta_size(25_000);
        let stats = manager.get_stats();
        assert_eq!(stats.current_delta_edges, 25_000);
    }

    #[test]
    fn test_manager_creation_and_stats() {
        let config = BackgroundFreezeConfig {
            delta_edge_threshold: 50_000,
        };
        let manager = BackgroundFreezeManager::new(config);

        let stats = manager.get_stats();
        assert_eq!(stats.freeze_count, 0);
        assert_eq!(stats.total_frozen_edges, 0);
        assert_eq!(stats.current_delta_edges, 0);
    }

    #[test]
    fn test_config_access() {
        let config = BackgroundFreezeConfig {
            delta_edge_threshold: 75_000,
        };
        let manager = BackgroundFreezeManager::new(config);

        assert_eq!(manager.config().delta_edge_threshold, 75_000);
    }

    #[test]
    fn test_default_config() {
        let manager = BackgroundFreezeManager::default();
        assert_eq!(
            manager.config().delta_edge_threshold,
            100_000,
            "Default threshold should be 100K edges"
        );
    }
}
