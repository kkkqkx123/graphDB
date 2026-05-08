//! Hit Rate Predictor
//!
//! Predicts cache hit rate for capacity planning.

use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct CacheAccess {
    pub cache_type: CacheAccessType,
    pub key_hash: u64,
    pub size: usize,
    pub timestamp: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheAccessType {
    Vertex,
    Edge,
    EdgeQuery,
    IdIndex,
}

#[derive(Debug, Clone)]
pub struct PredictionResult {
    pub predicted_hit_rate: f64,
    pub recommended_capacity: usize,
    pub expected_memory_usage: usize,
    pub current_hit_rate: f64,
}

pub struct HitRatePredictor {
    access_history: Vec<CacheAccess>,
    max_history: usize,
    current_capacity: usize,
}

impl HitRatePredictor {
    pub fn new(max_history: usize, current_capacity: usize) -> Self {
        Self {
            access_history: Vec::with_capacity(max_history),
            max_history,
            current_capacity,
        }
    }

    pub fn record_access(&mut self, access: CacheAccess) {
        if self.access_history.len() >= self.max_history {
            self.access_history.remove(0);
        }
        self.access_history.push(access);
    }

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

    pub fn access_count(&self) -> usize {
        self.access_history.len()
    }

    pub fn clear_history(&mut self) {
        self.access_history.clear();
    }
}
