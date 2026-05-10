//! Hit Rate Predictor
//!
//! Predicts cache hit rate for capacity planning.
//! Used for determining optimal cache size based on access patterns.

use std::collections::HashMap;
use std::time::Instant;

/// Record of a cache access event
#[derive(Debug, Clone)]
pub struct CacheAccess {
    /// Type of cache accessed
    pub cache_type: CacheAccessType,
    /// Hash of the key accessed
    pub key_hash: u64,
    /// Size of the value in bytes
    pub size: usize,
    /// When the access occurred
    pub timestamp: Instant,
}

/// Type of cache being accessed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheAccessType {
    /// Vertex record cache
    Vertex,
    /// ID index cache (external_id -> internal_id)
    IdIndex,
}

/// Result of a hit rate prediction
#[derive(Debug, Clone)]
pub struct PredictionResult {
    /// Predicted hit rate at the target capacity
    pub predicted_hit_rate: f64,
    /// The target capacity used for prediction
    pub recommended_capacity: usize,
    /// Expected memory usage at target capacity
    pub expected_memory_usage: usize,
    /// Current hit rate
    pub current_hit_rate: f64,
}

/// Predictor for cache hit rate based on access history
pub struct HitRatePredictor {
    access_history: Vec<CacheAccess>,
    max_history: usize,
    current_capacity: usize,
}

impl HitRatePredictor {
    /// Create a new predictor
    ///
    /// # Arguments
    /// * `max_history` - Maximum number of access records to keep
    /// * `current_capacity` - Current cache capacity in bytes
    pub fn new(max_history: usize, current_capacity: usize) -> Self {
        Self {
            access_history: Vec::with_capacity(max_history),
            max_history,
            current_capacity,
        }
    }

    /// Record a cache access
    pub fn record_access(&mut self, access: CacheAccess) {
        if self.access_history.len() >= self.max_history {
            self.access_history.remove(0);
        }
        self.access_history.push(access);
    }

    /// Predict hit rate for a given target capacity
    pub fn predict_for_capacity(&self, target_capacity: usize) -> PredictionResult {
        let mut simulated_cache_size = 0usize;
        let mut hits = 0usize;
        let mut misses = 0usize;
        let mut entries: HashMap<u64, (usize, Instant)> = HashMap::new();

        for access in &self.access_history {
            if let std::collections::hash_map::Entry::Vacant(e) = entries.entry(access.key_hash) {
                misses += 1;
                if simulated_cache_size + access.size <= target_capacity {
                    e.insert((access.size, access.timestamp));
                    simulated_cache_size += access.size;
                }
            } else {
                hits += 1;
            }
        }

        let total = hits + misses;
        let predicted_hit_rate = if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        };

        let current_hits = self.access_history.iter().filter(|a| entries.contains_key(&a.key_hash)).count();
        let current_hit_rate = if total > 0 {
            current_hits as f64 / total as f64
        } else {
            0.0
        };

        PredictionResult {
            predicted_hit_rate,
            recommended_capacity: target_capacity,
            expected_memory_usage: simulated_cache_size,
            current_hit_rate,
        }
    }

    /// Find the optimal capacity to achieve a target hit rate
    pub fn find_optimal_capacity(&self, target_hit_rate: f64) -> Option<PredictionResult> {
        if self.access_history.is_empty() {
            return None;
        }

        let total_access_size: usize = self.access_history.iter().map(|a| a.size).sum();
        let min_capacity = self.current_capacity / 4;
        let max_capacity = total_access_size;

        for capacity in (min_capacity..=max_capacity).step_by(1024 * 1024) {
            let result = self.predict_for_capacity(capacity);
            if result.predicted_hit_rate >= target_hit_rate {
                return Some(result);
            }
        }

        self.predict_for_capacity(max_capacity).into()
    }

    /// Get the number of recorded accesses
    pub fn access_count(&self) -> usize {
        self.access_history.len()
    }

    /// Clear access history
    pub fn clear_history(&mut self) {
        self.access_history.clear();
    }
}
