# Cache Module Architecture Design

## 1. Current Storage Architecture Analysis

### 1.1 Storage Layer Hierarchy

```
┌─────────────────────────────────────────────────────────────┐
│                     Query Engine                             │
│                  (Executor, Planner)                         │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                   StorageClient Trait                        │
│         (get_vertex, get_edge, scan, insert, etc.)          │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    PropertyGraph                             │
│    ┌─────────────────┐    ┌─────────────────┐               │
│    │  VertexTables   │    │   EdgeTables    │               │
│    │  HashMap<Label, │    │ HashMap<Key,    │               │
│    │  VertexTable>   │    │ EdgeTable>      │               │
│    └────────┬────────┘    └────────┬────────┘               │
│             │                      │                         │
│             ▼                      ▼                         │
│    ┌─────────────────┐    ┌─────────────────┐               │
│    │   record_cache  │    │   record_cache  │  ← CURRENT    │
│    │  (CachedVertex, │    │  (CachedEdge)   │    USAGE      │
│    │  CachedEdge)    │    │                 │               │
│    └─────────────────┘    └─────────────────┘               │
│                                                              │
│    ┌─────────────────┐                                       │
│    │   block_cache   │  ← UNUSED                            │
│    │  (BlockId ->    │                                       │
│    │   Arc<[u8]>)    │                                       │
│    └─────────────────┘                                       │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    Table Layer                               │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ VertexTable                                           │   │
│  │  - IdIndexer<String> (external_id -> internal_id)     │   │
│  │  - ColumnStore (columnar property storage)            │   │
│  │  - VertexTimestamp (MVCC timestamps)                  │   │
│  └──────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ EdgeTable                                             │   │
│  │  - MutableCsr (out_edges, in_edges)                   │   │
│  │  - PropertyTable (edge properties)                    │   │
│  │  - edge_id_counter                                    │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    Page Layer                                │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ PageManager                                           │   │
│  │  - pages: HashMap<PageId, Page>  ← Another cache!     │   │
│  │  - MAX_PAGES_IN_MEMORY = 1024                         │   │
│  │  - Simple LRU eviction                                │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                  Container Layer                             │
│  - MmapContainer (memory-mapped files)                       │
│  - ArenaAllocator (arena-based allocation)                   │
│  - FileSharedMmap (shared memory mapping)                    │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 Current Problems

| Problem               | Description                    | Impact                      |
| --------------------- | ------------------------------ | --------------------------- |
| **BlockCache Unused** | Created but never called       | Wastes 50% of cache memory  |
| **RecordCache Bug**   | Double get() call returns None | Cache never works correctly |
| **Duplicate Caching** | PageManager has its own cache  | Inconsistent behavior       |
| **No Clear Strategy** | What to cache is unclear       | Architecture confusion      |
| **O(n) LRU**          | Vec-based LRU is inefficient   | Performance degradation     |

### 1.3 Data Access Patterns

```
Query Pattern Analysis:
─────────────────────────────────────────────────────────────

1. Point Lookup (Most Common)
   - get_vertex(space, tag, id) → VertexRecord
   - get_edge(space, src, dst, edge_type) → EdgeRecord
   - Pattern: High locality, repeated access to same vertices/edges

2. Range Scan
   - scan_vertices(space, tag) → Iterator<VertexRecord>
   - Pattern: Sequential access, low locality

3. Traversal (Graph-specific)
   - get_neighbors(vid) → Iterator<EdgeRecord>
   - Pattern: Breadth-first or depth-first, high locality within subgraph

4. Property Lookup
   - get_property(vid, prop_name) → Value
   - Pattern: Often repeated for same properties

5. Index Lookup
   - scan_vertices_by_prop(space, tag, prop, value)
   - Pattern: Random access based on property value
```

---

## 2. Cache Module Purpose and Goals

### 2.1 Core Purpose

**The cache module should accelerate data access by keeping frequently accessed data in memory, reducing the cost of:**

1. **Repeated Record Access** - Same vertex/edge accessed multiple times in a query
2. **Property Deserialization** - Avoid parsing raw bytes repeatedly
3. **ID Resolution** - Cache external_id → internal_id mappings
4. **Schema Lookups** - Cache table schemas and metadata

### 2.2 What NOT to Cache

| Data Type            | Reason                                    |
| -------------------- | ----------------------------------------- |
| **Raw Pages/Blocks** | Table layer already in memory (Vec-based) |
| **Full Table Scans** | Low locality, pollutes cache              |
| **Write Buffers**    | Handled by WAL and flush manager          |
| **Transaction Data** | Managed by transaction layer              |

### 2.3 Design Goals

1. **Correctness First** - Must not return stale or incorrect data
2. **Transparency** - Cache should be invisible to callers
3. **Configurability** - Allow tuning for different workloads
4. **Observability** - Provide statistics for monitoring
5. **Efficiency** - O(1) operations, minimal overhead

---

## 3. Proposed Architecture

### 3.1 Single Cache Layer - Record Cache

**Decision: Remove BlockCache, focus on RecordCache**

Rationale:

- Current architecture stores data in memory (Vec-based ColumnStore, CSR)
- No disk-based block storage that would benefit from block-level caching
- PageManager's cache is for persistence layer, separate concern
- Record-level caching provides the most value for query workloads

### 3.2 New Cache Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     RecordCache                              │
│                                                              │
│  ┌─────────────────────────────────────────────────────┐    │
│  │                  CacheBackend                        │    │
│  │   (Pluggable: Moka | LRU | Custom)                   │    │
│  │                                                      │    │
│  │  ┌─────────────────┐  ┌─────────────────┐           │    │
│  │  │  VertexCache    │  │   EdgeCache     │           │    │
│  │  │  (Key: VertexId │  │  (Key: EdgeKey  │           │    │
│  │  │   Value: Vertex │  │   Value: Edge   │           │    │
│  │  │   Record)       │  │   Record)       │           │    │
│  │  └─────────────────┘  └─────────────────┘           │    │
│  │                                                      │    │
│  │  ┌─────────────────────────────────────────┐        │    │
│  │  │           IdIndexCache                   │        │    │
│  │  │  (Key: (LabelId, external_id)            │        │    │
│  │  │   Value: internal_id)                    │        │    │
│  │  └─────────────────────────────────────────┘        │    │
│  └─────────────────────────────────────────────────────┘    │
│                                                              │
│  ┌─────────────────────────────────────────────────────┐    │
│  │               CacheStatistics                        │    │
│  │  - hits, misses, hit_rate                           │    │
│  │  - evictions, memory_usage                          │    │
│  │  - per_cache_type breakdown                         │    │
│  └─────────────────────────────────────────────────────┘    │
│                                                              │
│  ┌─────────────────────────────────────────────────────┐    │
│  │               InvalidationManager                    │    │
│  │  - on_insert(label, id)                             │    │
│  │  - on_update(label, id)                             │    │
│  │  - on_delete(label, id)                             │    │
│  │  - on_schema_change(label)                          │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

### 3.3 Cache Key Design

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VertexCacheKey {
    pub label_id: u16,
    pub internal_id: u32,
    pub timestamp: u64,  // MVCC timestamp for versioning
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeCacheKey {
    pub edge_label_id: u16,
    pub src_vid: u64,
    pub dst_vid: u64,
    pub edge_id: u64,
    pub timestamp: u64,  // MVCC timestamp for versioning
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdIndexCacheKey {
    pub label_id: u16,
    pub external_id: String,
}
```

### 3.4 Cache Value Design

```rust
#[derive(Debug, Clone)]
pub struct CachedVertex {
    pub internal_id: u32,
    pub external_id: String,
    pub properties: Arc<[(String, Value)]>,  // Arc for cheap cloning
    pub size_estimate: u32,  // Pre-computed for weigher
}

#[derive(Debug, Clone)]
pub struct CachedEdge {
    pub edge_id: u64,
    pub src_vid: u64,
    pub dst_vid: u64,
    pub properties: Arc<[(String, Value)]>,
    pub size_estimate: u32,
}
```

---

## 4. Integration Points

### 4.1 PropertyGraph Integration

```rust
impl PropertyGraph {
    pub fn get_vertex_by_id(
        &self,
        label: LabelId,
        external_id: &str,
        ts: Timestamp,
    ) -> Option<VertexRecord> {
        // 1. Try cache first
        if let Some(cache) = &self.record_cache {
            if let Some(internal_id) = cache.get_id_index(label, external_id) {
                if let Some(cached) = cache.get_vertex(VertexCacheKey::new(label, internal_id, ts)) {
                    return Some(cached.into());
                }
            }
        }

        // 2. Cache miss - load from table
        let table = self.vertex_tables.get(&label)?;
        let internal_id = table.get_internal_id(external_id, ts)?;
        let record = table.get_by_internal_id(internal_id, ts)?;

        // 3. Populate cache
        if let Some(cache) = &self.record_cache {
            cache.insert_id_index(label, external_id, internal_id);
            cache.insert_vertex(VertexCacheKey::new(label, internal_id, ts), record.clone().into());
        }

        Some(record)
    }

    pub fn delete_vertex(&mut self, label: LabelId, external_id: &str, ts: Timestamp) {
        // 1. Invalidate cache first
        if let Some(cache) = &self.record_cache {
            if let Some(internal_id) = cache.get_id_index(label, external_id) {
                cache.invalidate_vertex(VertexCacheKey::new(label, internal_id, ts));
                cache.invalidate_id_index(label, external_id);
            }
        }

        // 2. Perform deletion
        // ...
    }
}
```

### 4.2 Configuration Integration

```rust
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Enable caching
    pub enabled: bool,

    /// Maximum memory for all caches (bytes)
    pub max_memory: usize,

    /// Memory allocation ratio: vertex:edge:id_index
    pub memory_ratio: (u32, u32, u32),

    /// Time-to-live for cached entries
    pub ttl: Option<Duration>,

    /// Time-to-idle for cached entries
    pub tti: Option<Duration>,

    /// Number of shards per cache
    pub shard_count: usize,

    /// Eviction policy
    pub eviction_policy: EvictionPolicy,
}

#[derive(Debug, Clone, Copy)]
pub enum EvictionPolicy {
    Lru,
    Lfu,
    TinyLfu,  // Default - best hit rate
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_memory: 256 * 1024 * 1024,  // 256MB
            memory_ratio: (50, 40, 10),      // 50% vertex, 40% edge, 10% id index
            ttl: Some(Duration::from_secs(3600)),
            tti: Some(Duration::from_secs(300)),
            shard_count: 16,
            eviction_policy: EvictionPolicy::TinyLfu,
        }
    }
}
```

---

## 5. Implementation Plan

### 5.1 Phase 1: Remove BlockCache

**Action**: Delete `block_cache.rs` and related code

**Rationale**:

- Never used in practice
- No block-based storage layer to integrate with
- Wastes memory allocation

**Changes**:

```diff
// mod.rs
- mod block_cache;
- pub use block_cache::{BlockCache, BlockId, ...};
```

### 5.2 Phase 2: Rewrite RecordCache

**Action**: Complete rewrite using Moka or custom implementation

**New Structure**:

```
src/storage/cache/
├── mod.rs              # Module exports
├── config.rs           # CacheConfig, EvictionPolicy
├── keys.rs             # VertexCacheKey, EdgeCacheKey, IdIndexCacheKey
├── values.rs           # CachedVertex, CachedEdge
├── backend.rs          # CacheBackend trait
├── moka_backend.rs     # Moka-based implementation
├── record_cache.rs     # Main RecordCache struct
└── statistics.rs       # CacheStatistics
```

### 5.3 Phase 3: Integration

**Action**: Integrate with PropertyGraph

**Key Points**:

1. Cache lookup before table access
2. Cache population after successful reads
3. Cache invalidation on writes
4. Statistics exposure via StorageClient

### 5.4 Phase 4: Testing & Monitoring

**Action**: Add comprehensive tests and monitoring

**Test Coverage**:

- Basic get/insert/remove operations
- Cache eviction under memory pressure
- Concurrent access patterns
- Invalidation correctness
- Statistics accuracy

---

## 6. Backend Options

### 6.1 Option A: Moka Cache (Recommended)

**Pros**:

- Battle-tested, high performance
- TinyLFU policy for optimal hit rate
- Built-in TTL/TTI support
- Size-aware eviction (weigher)
- Concurrent access optimized

**Cons**:

- External dependency
- Less control over internals

**Implementation**:

```rust
use moka::sync::Cache;

pub struct MokaBackend<K, V> {
    cache: Cache<K, V>,
}

impl<K, V> CacheBackend<K, V> for MokaBackend<K, V>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn get(&self, key: &K) -> Option<V> {
        self.cache.get(key)
    }

    fn insert(&self, key: K, value: V) {
        self.cache.insert(key, value);
    }

    fn remove(&self, key: &K) {
        self.cache.invalidate(key);
    }
}
```

### 6.2 Option B: Custom LRU Implementation

**Pros**:

- No external dependency
- Full control over behavior
- Can optimize for specific use case

**Cons**:

- More code to maintain
- Need to implement TTL/TTI manually
- Need to handle concurrency manually

### 6.3 Recommendation

**Use Moka for production, with trait abstraction for future flexibility**

```rust
pub trait CacheBackend<K, V>: Send + Sync {
    fn get(&self, key: &K) -> Option<V>;
    fn insert(&self, key: K, value: V);
    fn remove(&self, key: &K);
    fn clear(&self);
    fn entry_count(&self) -> u64;
    fn weighted_size(&self) -> u64;
}
```

---

## 7. Summary

### Key Decisions

| Decision             | Rationale                                |
| -------------------- | ---------------------------------------- |
| Remove BlockCache    | No block-based storage layer, never used |
| Single cache layer   | Simpler architecture, less confusion     |
| Record-level caching | Most value for query workloads           |
| Moka backend         | Battle-tested, optimal hit rate          |
| TTL/TTI support      | Automatic cleanup of stale data          |
| Size-aware eviction  | Accurate memory management               |

### Expected Benefits

1. **Correctness** - Fixed double-get bug
2. **Performance** - O(1) operations, optimal hit rate
3. **Simplicity** - Single cache layer, clear purpose
4. **Flexibility** - Pluggable backend, configurable policies
5. **Observability** - Comprehensive statistics

### Migration Path

1. Remove BlockCache (immediate)
2. Rewrite RecordCache with Moka (short-term)
3. Add IdIndexCache (medium-term)
4. Add TTL/TTI support (medium-term)
5. Performance testing and tuning (ongoing)

---

## 8. Current Implementation Analysis and Improvement Plan

### 8.1 Current Implementation Status

#### ✅ Correctly Implemented

1. **Architecture Simplification** - Removed unused BlockCache, only RecordCache remains
2. **High-Performance Backend** - Using Moka library with TinyLFU eviction policy
3. **Size-aware Eviction** - Implemented weigher for size-based eviction
4. **Statistics** - Basic hit rate, memory usage statistics provided
5. **Memory Tracking** - Integrated with MemoryTracker for memory management

#### ❌ Missing Critical Features

| Feature                 | Document Suggestion                      | Current Status           | Impact                             |
| ----------------------- | ---------------------------------------- | ------------------------ | ---------------------------------- |
| **IdIndexCache**        | Cache external_id → internal_id mapping  | ❌ Not implemented       | Performance loss on every query    |
| **MVCC Timestamp**      | Cache key includes timestamp             | ❌ Not implemented       | May return wrong version data      |
| **TTL/TTI**             | Auto-cleanup of stale data               | ❌ Not configured        | Cache may contain stale data       |
| **CacheBackend trait**  | Abstract interface for multiple backends | ❌ Not implemented       | Cannot switch cache implementation |
| **Fine-grained Config** | memory_ratio, shard_count, etc.          | ⚠️ Partially implemented | Lack of flexibility                |

### 8.2 Critical Issues Analysis

#### Issue 1: Missing MVCC Version Control ⚠️ HIGH PRIORITY

**Problem**: Current cache key does not include timestamp

```rust
// Current implementation
pub struct VertexCacheKey {
    pub label_id: u16,
    pub internal_id: u32,
    // ❌ Missing timestamp
}

// Document suggestion
pub struct VertexCacheKey {
    pub label_id: u16,
    pub internal_id: u32,
    pub timestamp: u64,  // ✅ MVCC timestamp for versioning
}
```

**Impact**:

- May return wrong version data in MVCC scenarios
- Cannot correctly handle concurrent transactions

**Solution**: Add timestamp to cache key immediately

#### Issue 2: Missing IdIndexCache ⚠️ HIGH PRIORITY

**Problem**: Every query requires lookup from IdIndexer

```rust
// Current implementation - property_graph.rs:547
let table = self.vertex_tables.get(&label)?;
let internal_id = table.get_internal_id(external_id, ts)?;  // Lookup every time

// Document suggestion
if let Some(cache) = &self.record_cache {
    if let Some(internal_id) = cache.get_id_index(label, external_id) {  // ✅ Check cache first
        // ...
    }
}
```

**Impact**:

- IdIndexer uses HashMap lookup, O(1) but still has hash computation overhead
- Performance loss for frequent queries on same external_id

**Solution**: Add IdIndexCache to cache external_id → internal_id mapping

#### Issue 3: Missing TTL/TTI Configuration ⚠️ MEDIUM PRIORITY

**Problem**: Cached data never expires

```rust
// Current implementation
let vertex_cache = Cache::builder()
    .max_capacity(vertex_memory)
    .weigher(|_key: &VertexCacheKey, value: &CachedVertex| value.estimated_size())
    .build();  // ❌ No TTL/TTI

// Document suggestion
let vertex_cache = Cache::builder()
    .max_capacity(vertex_memory)
    .weigher(...)
    .time_to_live(Duration::from_secs(3600))  // ✅ TTL
    .time_to_idle(Duration::from_secs(300))   // ✅ TTI
    .build();
```

**Impact**:

- Cache may contain stale data
- Cannot auto-cleanup unused data

**Solution**: Add TTL/TTI configuration options

### 8.3 Improvement Recommendations

#### Priority Ranking

| Priority  | Improvement                              | Expected Benefit          | Implementation Difficulty |
| --------- | ---------------------------------------- | ------------------------- | ------------------------- |
| 🔴 High   | Add MVCC timestamp to cache key          | Fix correctness issue     | Low                       |
| 🔴 High   | Implement IdIndexCache                   | Improve query performance | Medium                    |
| 🟡 Medium | Add TTL/TTI configuration                | Auto-cleanup stale data   | Low                       |
| 🟢 Low    | Add CacheBackend trait                   | Provide flexibility       | Medium                    |
| 🟢 Low    | Fine-grained config (memory_ratio, etc.) | Optimize memory usage     | Low                       |

### 8.4 Implementation Roadmap

#### Phase 1: Fix Correctness Issues (1-2 days)

1. Add timestamp to VertexCacheKey and EdgeCacheKey
2. Update cache usage code in PropertyGraph
3. Add MVCC-related tests

**Changes**:

```rust
// src/storage/cache/record_cache.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VertexCacheKey {
    pub label_id: u16,
    pub internal_id: u32,
    pub timestamp: u64,  // ✅ Add MVCC timestamp
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeCacheKey {
    pub edge_label_id: u16,
    pub src_vid: u64,
    pub dst_vid: u64,
    pub edge_id: u64,
    pub timestamp: u64,  // ✅ Add MVCC timestamp
}
```

#### Phase 2: Performance Optimization (2-3 days)

1. Implement IdIndexCache
2. Add TTL/TTI configuration
3. Performance testing and benchmarking

**New Structures**:

```rust
// src/storage/cache/record_cache.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdIndexCacheKey {
    pub label_id: u16,
    pub external_id: String,
}

pub struct RecordCache {
    vertex_cache: Cache<VertexCacheKey, CachedVertex>,
    edge_cache: Cache<EdgeCacheKey, CachedEdge>,
    id_index_cache: Cache<IdIndexCacheKey, u32>,  // ✅ New: IdIndexCache
    // ...
}
```

**Configuration**:

```rust
#[derive(Debug, Clone)]
pub struct RecordCacheConfig {
    pub max_memory: usize,
    pub memory_ratio: (u32, u32, u32),  // vertex:edge:id_index
    pub ttl: Option<Duration>,
    pub tti: Option<Duration>,
}
```

#### Phase 3: Architecture Optimization (Optional, 3-5 days)

1. Add CacheBackend trait
2. Implement fine-grained configuration
3. Add more statistics metrics

### 8.5 Overall Evaluation

**Current Design Reasonableness: ⭐⭐⭐☆☆ (3/5)**

**Strengths:**

- ✅ Removed unused BlockCache
- ✅ Using high-performance Moka library
- ✅ Implemented basic caching functionality
- ✅ Has statistics support

**Weaknesses:**

- ❌ Missing MVCC version control (correctness issue)
- ❌ Missing IdIndexCache (performance issue)
- ❌ Missing TTL/TTI (stale data issue)
- ❌ Missing abstract interface (flexibility issue)

**Conclusion**: Current implementation is basically reasonable but missing critical features. Recommend improving according to the above priorities. The most important is to fix MVCC version control issue, followed by implementing IdIndexCache for performance improvement.
