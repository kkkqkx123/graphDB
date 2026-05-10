//! Cache Statistics
//!
//! Statistics tracking for cache performance monitoring.

use std::sync::atomic::{AtomicU64, Ordering};

/// Per-cache-type statistics
#[derive(Debug, Default)]
pub struct CacheTypeStats {
    pub hits: AtomicU64,
    pub misses: AtomicU64,
    pub evictions: AtomicU64,
}

impl CacheTypeStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn hits(&self) -> u64 {
        self.hits.load(Ordering::Relaxed)
    }

    pub fn misses(&self) -> u64 {
        self.misses.load(Ordering::Relaxed)
    }

    pub fn evictions(&self) -> u64 {
        self.evictions.load(Ordering::Relaxed)
    }

    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits();
        let misses = self.misses();
        let total = hits + misses;
        if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        }
    }

    pub fn reset(&self) {
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
        self.evictions.store(0, Ordering::Relaxed);
    }
}

/// Snapshot of per-cache-type statistics
#[derive(Debug, Clone, Copy)]
pub struct CacheTypeStatsSnapshot {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub count: u64,
    pub memory_bytes: u64,
    pub hit_rate: f64,
}

impl CacheTypeStatsSnapshot {
    pub fn from_stats(stats: &CacheTypeStats, count: u64, memory_bytes: u64) -> Self {
        Self {
            hits: stats.hits(),
            misses: stats.misses(),
            evictions: stats.evictions(),
            count,
            memory_bytes,
            hit_rate: stats.hit_rate(),
        }
    }
}

/// Aggregated statistics for record cache
#[derive(Debug, Clone, Copy)]
pub struct RecordCacheStats {
    pub vertex: CacheTypeStatsSnapshot,
    pub id_index: CacheTypeStatsSnapshot,
    pub total_hits: u64,
    pub total_misses: u64,
    pub total_evictions: u64,
    pub hit_rate: f64,
    pub memory_usage: usize,
    pub max_memory: usize,
}

impl RecordCacheStats {
    pub fn format_bytes(bytes: usize) -> String {
        const KB: usize = 1024;
        const MB: usize = KB * 1024;
        const GB: usize = MB * 1024;

        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }

    pub fn total_count(&self) -> u64 {
        self.vertex.count + self.id_index.count
    }
}

impl std::fmt::Display for RecordCacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Record Cache: {}/{} ({:.1}%)",
            Self::format_bytes(self.memory_usage),
            Self::format_bytes(self.max_memory),
            if self.max_memory > 0 {
                self.memory_usage as f64 / self.max_memory as f64 * 100.0
            } else {
                0.0
            }
        )?;
        writeln!(
            f,
            "  Vertices: {} (hits: {}, misses: {}, evictions: {}, hit_rate: {:.1}%)",
            self.vertex.count, self.vertex.hits, self.vertex.misses, 
            self.vertex.evictions, self.vertex.hit_rate * 100.0
        )?;
        writeln!(
            f,
            "  IdIndexes: {} (hits: {}, misses: {}, evictions: {}, hit_rate: {:.1}%)",
            self.id_index.count, self.id_index.hits, self.id_index.misses, 
            self.id_index.evictions, self.id_index.hit_rate * 100.0
        )?;
        writeln!(
            f,
            "  Total: hits: {}, misses: {}, evictions: {}, hit_rate: {:.1}%",
            self.total_hits, self.total_misses, self.total_evictions, self.hit_rate * 100.0
        )
    }
}
