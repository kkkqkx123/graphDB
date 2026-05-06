# Cache Module Improvement Roadmap

## 1. Implementation Status

### 1.1 Completed Improvements

| Improvement              | Status       | Description                                      |
| ------------------------ | ------------ | ------------------------------------------------ |
| Fine-grained Statistics  | ✅ Completed | Per-cache-type hit/miss/eviction tracking        |
| Eviction Listener        | ✅ Completed | Callback support for eviction events             |
| High Priority Pool       | ✅ Completed | Extra memory quota for id_index cache            |
| Batch Operations         | ✅ Completed | Batch get/insert/invalidate operations           |
| Memory Pressure Response | ✅ Completed | Memory pressure detection and capacity reduction |
| Cache Warmup             | ✅ Completed | Warmup configuration and stats structures        |
| Hit Rate Prediction      | ✅ Completed | Hit rate predictor for capacity planning         |

### 1.2 Pending Improvements

| Improvement     | Priority | Complexity | Status         |
| --------------- | -------- | ---------- | -------------- |
| Async API       | 🟢 Low   | Low        | ❌ Not Started |
| Adaptive Tuning | 🟢 Low   | High       | ❌ Not Started |
| Cache Tiering   | 🟢 Low   | High       | ❌ Not Started |

---

## 2. Implemented Improvements

### 2.1 Batch Operations

Added batch operations for improved throughput:

```rust
pub struct BatchGetResult<T> {
    pub results: Vec<Option<T>>,
    pub hits: usize,
    pub misses: usize,
}

pub struct BatchInsertResult {
    pub inserted: usize,
    pub total_size: usize,
}

impl RecordCache {
    pub fn get_vertices_batch(&self, keys: &[VertexCacheKey]) -> BatchGetResult<CachedVertex>;
    pub fn insert_vertices_batch(&self, entries: Vec<(VertexCacheKey, CachedVertex)>) -> BatchInsertResult;
    pub fn get_edges_batch(&self, keys: &[EdgeCacheKey]) -> BatchGetResult<CachedEdge>;
    pub fn insert_edges_batch(&self, entries: Vec<(EdgeCacheKey, CachedEdge)>) -> BatchInsertResult;
    pub fn get_id_indexes_batch(&self, keys: &[(u16, &str)]) -> BatchGetResult<u32>;
    pub fn insert_id_indexes_batch(&self, entries: Vec<(u16, String, u32)>) -> BatchInsertResult;
    pub fn invalidate_batch(&self, keys: &[CacheKeyRef<'_>]) -> usize;
}
```

**Benefits**:

- Reduced per-operation overhead
- Higher throughput for bulk operations
- Better memory tracking efficiency

### 2.2 Memory Pressure Response

Added memory pressure detection and response:

```rust
pub enum MemoryPressureLevel {
    Normal,
    Warning,
    Critical,
}

pub struct MemoryPressureConfig {
    pub enabled: bool,
    pub high_watermark: f32,  // 0.9 = 90% memory used
    pub low_watermark: f32,   // 0.7 = 70% memory used
    pub reduction_factor: f32, // 0.5 = reduce to 50%
}

impl RecordCache {
    pub fn check_memory_pressure(&self) -> MemoryPressureLevel;
    pub fn reduce_capacity(&mut self, factor: f32);
    pub fn restore_capacity(&mut self);
}
```

**Benefits**:

- Prevent OOM crashes
- Graceful degradation under pressure
- System stability

### 2.3 Cache Warmup

Added warmup configuration and statistics:

```rust
pub struct CacheWarmupConfig {
    pub enabled: bool,
    pub warmup_vertex_labels: Vec<u16>,
    pub warmup_edge_labels: Vec<u16>,
    pub max_warmup_entries: usize,
}

pub struct WarmupStats {
    pub vertices_loaded: usize,
    pub edges_loaded: usize,
    pub id_indexes_loaded: usize,
    pub total_bytes: usize,
    pub duration_ms: u64,
}
```

**Benefits**:

- Reduced cold start latency
- Consistent performance after restart
- Better user experience

### 2.4 Hit Rate Prediction

Added hit rate predictor for capacity planning:

```rust
pub struct HitRatePredictor {
    // Records access patterns
}

pub struct PredictionResult {
    pub predicted_hit_rate: f64,
    pub recommended_capacity: usize,
    pub expected_memory_usage: usize,
    pub current_hit_rate: f64,
}

impl HitRatePredictor {
    pub fn new(max_history: usize, current_capacity: usize) -> Self;
    pub fn record_access(&mut self, access: CacheAccess);
    pub fn predict_for_capacity(&self, target_capacity: usize) -> PredictionResult;
    pub fn find_optimal_capacity(&self, target_hit_rate: f64) -> Option<PredictionResult>;
}
```

**Benefits**:

- Data-driven capacity planning
- Cost optimization
- Performance prediction

---

## 3. Detailed Improvement Plans (Pending)

### 3.1 Async API (Priority: Low)

#### Problem

Current implementation only provides synchronous API, which may block threads in high-concurrency scenarios.

#### Industry Reference

**Moka**: Already supports async API via `moka::future::Cache`.

#### Proposed Solution

```rust
use moka::future::Cache as AsyncCache;

pub struct AsyncRecordCache {
    vertex_cache: AsyncCache<VertexCacheKey, CachedVertex>,
    edge_cache: AsyncCache<EdgeCacheKey, CachedEdge>,
    // ...
}

impl AsyncRecordCache {
    pub async fn get_vertex(&self, key: &VertexCacheKey) -> Option<CachedVertex> {
        self.vertex_cache.get(key).await
    }

    pub async fn insert_vertex(&self, key: VertexCacheKey, vertex: CachedVertex) {
        self.vertex_cache.insert(key, vertex).await;
    }
}
```

#### Implementation Steps

1. Create `AsyncRecordCache` struct
2. Implement async versions of all cache operations
3. Add feature flag to switch between sync/async
4. Update `SharedRecordCache` to support both modes

#### Expected Benefits

- Better thread utilization
- Higher throughput in async contexts
- Reduced thread pool pressure

---

### 3.2 Adaptive Tuning (Priority: Low)

#### Problem

Fixed cache configuration cannot adapt to changing workloads:

- OLTP workloads need different cache ratios than OLAP
- Memory pressure varies over time
- Access patterns change throughout the day

#### Industry Reference

**PostgreSQL Clock Sweep**: Dynamically adjusts buffer priority based on access patterns.

**Redis maxmemory-policy**: Can be changed at runtime.

#### Proposed Solution

```rust
pub struct AdaptiveConfig {
    pub enabled: bool,
    pub adjustment_interval: Duration,
    pub min_hit_rate: f64,
    pub max_memory_ratio_change: f32,
}

pub struct AdaptiveTuner {
    config: AdaptiveConfig,
    history: Vec<CacheStatsSnapshot>,
    current_ratios: (u32, u32, u32, u32),
}

impl AdaptiveTuner {
    pub fn analyze_and_adjust(&mut self, cache: &RecordCache) -> AdjustmentResult {
        // 1. Collect recent statistics
        // 2. Analyze hit rate trends
        // 3. Identify underperforming cache types
        // 4. Propose ratio adjustments
        // 5. Apply adjustments if beneficial
    }
}
```

#### Implementation Steps

1. Implement statistics history collection
2. Create adjustment algorithm based on hit rate analysis
3. Add runtime configuration update support
4. Implement gradual adjustment to avoid thrashing
5. Add monitoring for adjustment effectiveness

#### Expected Benefits

- Self-optimizing cache performance
- Better resource utilization
- Reduced manual tuning

---

### 3.3 Cache Tiering (Priority: Low)

#### Problem

All cache entries are treated equally, but access patterns differ:

- Hot data: Frequently accessed, should stay in memory
- Warm data: Occasionally accessed, can be moved to secondary cache
- Cold data: Rarely accessed, can be evicted

#### Industry Reference

**RocksDB**: Supports block cache + compressed cache tiering.

**Caffeine**: Implements Window TinyLFU with admission window.

#### Proposed Solution

```rust
pub struct TieredCache {
    hot_cache: Cache<K, V>,      // In-memory, fast access
    warm_cache: LruCache<K, V>,  // In-memory, larger but slower
    cold_store: Option<PathBuf>, // Optional disk-backed cache
}

pub struct TieringConfig {
    pub hot_capacity: usize,
    pub warm_capacity: usize,
    pub promotion_threshold: u64,  // Access count to promote to hot
    pub demotion_threshold: Duration, // Idle time to demote to warm
}
```

#### Implementation Steps

1. Design tiered cache architecture
2. Implement promotion/demotion logic
3. Add optional disk-backed cold cache
4. Update statistics for tier tracking
5. Add configuration for tier sizing

#### Expected Benefits

- Better memory utilization
- Higher effective cache size
- Improved hit rate for warm data

---

## 4. Metrics and Monitoring

### 4.1 Current Metrics

| Metric         | Description              |
| -------------- | ------------------------ |
| `hits`         | Total cache hits         |
| `misses`       | Total cache misses       |
| `evictions`    | Total evictions          |
| `hit_rate`     | Overall hit rate         |
| `memory_usage` | Current memory usage     |
| `entry_count`  | Number of cached entries |

### 4.2 Proposed Metrics

| Metric               | Description                     | Priority |
| -------------------- | ------------------------------- | -------- |
| `eviction_rate`      | Evictions per second            | High     |
| `average_entry_size` | Average cached entry size       | Medium   |
| `memory_efficiency`  | Useful data / total memory      | Medium   |
| `warmup_progress`    | Cache warmup completion %       | Low      |
| `tier_distribution`  | Entry distribution across tiers | Low      |
| `adjustment_history` | Adaptive tuning changes         | Low      |

---

## 5. References

- [RocksDB Block Cache](https://github.com/facebook/rocksdb/wiki/Block-Cache)
- [RocksDB SimCache](https://github.com/facebook/rocksdb/wiki/SimCache)
- [PostgreSQL pg_prewarm](https://www.postgresql.org/docs/current/pgprewarm.html)
- [Caffeine Design](https://github.com/ben-manes/caffeine/wiki/Design)
- [Moka Documentation](https://docs.rs/moka/latest/moka/)
