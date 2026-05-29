//! Cache Statistics
//!
//! Statistics tracking for cache performance monitoring.

use crate::core::stats::CacheStats;

/// Snapshot of per-cache-type statistics
#[derive(Debug, Clone, Copy)]
pub struct CacheTypeStatsSnapshot {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub count: u64,
    pub memory_bytes: u64,
    pub hit_rate: f64,
    pub eviction_rate: f64,
}

impl CacheTypeStatsSnapshot {
    pub fn from_stats(stats: &CacheStats, count: u64, memory_bytes: u64) -> Self {
        let total_requests = stats.hits() + stats.misses();
        let total_operations = total_requests + stats.evictions();
        Self {
            hits: stats.hits(),
            misses: stats.misses(),
            evictions: stats.evictions(),
            count,
            memory_bytes,
            hit_rate: stats.hit_rate(),
            eviction_rate: if total_operations > 0 {
                stats.evictions() as f64 / total_operations as f64
            } else {
                0.0
            },
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
    pub eviction_rate: f64,
    pub memory_usage: usize,
    pub max_memory: usize,
    pub uptime_seconds: u64,
    pub vertex_memory_bytes: u64,
    pub id_index_memory_bytes: u64,
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
            "  Uptime: {}s",
            self.uptime_seconds,
        )?;
        writeln!(
            f,
            "  Vertices: {} (hits: {}, misses: {}, evictions: {}, hit_rate: {:.1}%, eviction_rate: {:.1}%, memory: {})",
            self.vertex.count, self.vertex.hits, self.vertex.misses,
            self.vertex.evictions, self.vertex.hit_rate * 100.0, self.vertex.eviction_rate * 100.0,
            Self::format_bytes(self.vertex.memory_bytes as usize)
        )?;
        writeln!(
            f,
            "  IdIndexes: {} (hits: {}, misses: {}, evictions: {}, hit_rate: {:.1}%, eviction_rate: {:.1}%, memory: {})",
            self.id_index.count, self.id_index.hits, self.id_index.misses,
            self.id_index.evictions, self.id_index.hit_rate * 100.0, self.id_index.eviction_rate * 100.0,
            Self::format_bytes(self.id_index.memory_bytes as usize)
        )?;
        writeln!(
            f,
            "  Total: hits: {}, misses: {}, evictions: {}, hit_rate: {:.1}%, eviction_rate: {:.1}%",
            self.total_hits,
            self.total_misses,
            self.total_evictions,
            self.hit_rate * 100.0,
            self.eviction_rate * 100.0
        )
    }
}
