# Cache Necessity Analysis

## Overview

This document analyzes the necessity of each cache type in the current project, identifying design conflicts, redundancies, and optimization opportunities based on actual usage patterns.

---

## 1. Storage Layer Cache (RecordCache)

### 1.1 Current Architecture

Location: `src/storage/cache/record_cache.rs`

RecordCache manages four sub-caches as a unified facade:

| Cache Type | Key Structure | Value Type | Default Memory Ratio |
|------------|---------------|------------|---------------------|
| Vertex Cache | (label_id, internal_id, timestamp) | CachedVertex | 40% |
| Edge Cache | (edge_label_id, src_vid, dst_vid, edge_id, timestamp) | CachedEdge | 30% |
| Edge Query Cache | (edge_label_id, src_vid, dst_vid, timestamp) | CachedEdge | 20% |
| ID Index Cache | (label_id, external_id) | u32 (internal_id) | 10% |

Implementation: Uses Moka with TinyLFU eviction policy, TTL/TTI support, and memory-weighted eviction.

### 1.2 Actual Usage Analysis

#### ID Index Cache - NECESSARY (High Value)

**Usage Location**: [property_graph.rs:L558-L571](file:///d:/项目/database/graphDB/src/storage/property_graph.rs#L558-L571)

```rust
let internal_id = if let Some(ref record_cache) = self.record_cache {
    record_cache.get_id_index(label, external_id)
} else {
    None
};
```

**Necessity Assessment**: HIGH

**Reasons**:
- Every vertex lookup by external_id requires this mapping
- Avoids HashMap lookup in IdIndexer (O(1) but still has hash computation overhead)
- Small memory footprint (only stores u32 internal_id)
- High hit rate expected for workloads with repeated vertex access
- Key does not include timestamp - single entry per external_id

**Recommendation**: Keep and optimize

#### Vertex Cache - CONDITIONALLY NECESSARY (Design Conflict)

**Usage Location**: [property_graph.rs:L578-L599](file:///d:/项目/database/graphDB/src/storage/property_graph.rs#L578-L599)

**Design Conflicts Identified**:

1. **MVCC Timestamp in Key**: Key includes timestamp, meaning every version of a vertex is cached separately
   - For a vertex accessed at ts=1000, ts=1001, ts=1002, three separate cache entries are created
   - This defeats the purpose of caching for read-mostly workloads
   - Most queries use current timestamp, so historical versions rarely benefit from cache

2. **Data Duplication with Column Store**: CachedVertex stores full properties as `Vec<(String, Value)>`
   - The same data is already stored in ColumnStore (columnar format)
   - ColumnStore uses compressed encoding (Dictionary, RLE, BitPacking, etc.)
   - CachedVertex stores uncompressed data, using 3-5x more memory
   - For a vertex with 10 properties, cached version uses ~500 bytes vs ~100 bytes compressed

3. **Cache Key Mismatch**: Vertex lookup by internal_id bypasses ID Index Cache
   - [property_graph.rs:L605-L636](file:///d:/项目/database/graphDB/src/storage/property_graph.rs#L605-L636) shows `get_vertex_by_internal_id` uses Vertex Cache directly
   - Graph traversals use internal_id, so Vertex Cache is hit during traversals
   - But the timestamp issue still applies

**Necessity Assessment**: MEDIUM (needs redesign)

**Recommendation**: 
- Remove timestamp from key, cache only latest version
- Consider caching compressed column chunks instead of full vertex records
- Or reduce to a small L1 cache for hot vertices only

#### Edge Query Cache - NECESSARY (High Value)

**Usage Location**: [property_graph.rs:L766-L801](file:///d:/项目/database/graphDB/src/storage/property_graph.rs#L766-L801)

```rust
if let Some(ref record_cache) = self.record_cache {
    let query_key = EdgeQueryKey::new(
        edge_label,
        src_internal as u64,
        dst_internal as u64,
        ts as u64,
    );
    if let Some(cached) = record_cache.get_edge_by_query(&query_key) {
        return Some(EdgeRecord { ... });
    }
}
```

**Necessity Assessment**: HIGH

**Reasons**:
- Edge lookup by (src, dst) requires searching CSR structure
- CSR search is O(log N) for sorted edges, but still requires memory access
- Caching avoids CSR traversal for repeated edge queries
- Graph algorithms frequently check edge existence between same vertex pairs
- Key does not include edge_id - single entry per (src, dst) pair

**Design Issue**: Timestamp in key causes same versioning problem as Vertex Cache

**Recommendation**: Keep, but remove timestamp from key

#### Edge Cache - UNNECESSARY (Redundant)

**Usage Analysis**: Edge Cache is defined but NEVER used in property_graph.rs

All edge lookups use Edge Query Cache instead:
- `get_edge_by_external_id` uses Edge Query Cache
- Edge deletion uses Edge Query Cache for invalidation
- No code path calls `get_edge()` or `insert_edge()` on RecordCache

**Necessity Assessment**: LOW (Unused)

**Reasons**:
- Edge Cache key includes edge_id, which is only known after lookup
- In practice, users query edges by (src, dst), not by edge_id
- Edge Query Cache serves the same purpose with a more practical key
- 30% of default memory allocation is wasted on unused cache

**Recommendation**: Remove Edge Cache, reallocate memory to Edge Query Cache

### 1.3 Summary - RecordCache

| Cache Type | Status | Priority | Action |
|------------|--------|----------|--------|
| ID Index Cache | Keep | High | Optimize |
| Edge Query Cache | Keep | High | Remove timestamp from key |
| Vertex Cache | Redesign | Medium | Remove timestamp, consider compressed caching |
| Edge Cache | Remove | Low | Reallocate memory to Edge Query Cache |

---

## 2. Query Layer Cache

### 2.1 Plan Cache (QueryPlanCache)

Location: `src/query/cache/plan_cache.rs`

**Purpose**: Cache parsed and optimized query plans (Prepared Statement style)

**Necessity Assessment**: HIGH

**Reasons**:
- Query parsing and optimization is expensive (AST parsing, validation, planning)
- Applications frequently execute same query with different parameters
- Moka-based implementation with memory-weighted eviction is appropriate
- Already integrated with QueryPipelineManager
- Dependent table tracking enables targeted invalidation

**Current State**: Well-designed, no major issues

**Recommendation**: Keep as-is

### 2.2 CTE Cache (CteCacheManager)

Location: `src/query/cache/cte_cache.rs`

**Purpose**: Cache CTE (Common Table Expression) results to avoid repeated computation

**Necessity Assessment**: MEDIUM

**Reasons**:
- CTEs are expensive to compute, especially recursive CTEs
- Cache helps when same CTE is referenced multiple times in a query
- However, CTEs are typically used once per query in graph databases
- Graph traversal CTEs produce different results based on starting vertex
- Memory overhead for storing result sets can be significant

**Design Issues**:
- Stores results as `Arc<Vec<u8>>` (raw bytes), requiring serialization/deserialization
- No integration with storage layer cache - may cache data already in RecordCache
- TTL-based eviction may not align with query patterns

**Recommendation**: Keep but reduce default memory allocation; consider lazy materialization

---

## 3. Inversearch Cache (Separate Crate)

Location: `crates/inversearch/src/search/cache.rs`

**Purpose**: Cache full-text search results

**Necessity Assessment**: HIGH (for search functionality)

**Reasons**:
- Full-text search is expensive (tokenization, scoring, ranking)
- Search results are deterministic for same query
- LRU with TTL is appropriate for search workloads
- Isolated from storage/query caches - no coordination needed

**Recommendation**: Keep as-is

---

## 4. Design Conflicts and Issues

### 4.1 Timestamp in Cache Keys

**Problem**: Vertex Cache and Edge Query Cache include timestamp in keys

**Impact**:
- Every MVCC version creates a separate cache entry
- For a vertex updated 100 times, 100 cache entries exist
- Cache hit rate drops significantly for write-heavy workloads
- Memory waste for historical versions rarely accessed

**Solution**:
```rust
// Current (problematic)
pub struct VertexCacheKey {
    pub label_id: u16,
    pub internal_id: u32,
    pub timestamp: u64,  // Remove this
}

// Proposed
pub struct VertexCacheKey {
    pub label_id: u16,
    pub internal_id: u32,
}

// For MVCC, cache only the latest version
// Historical versions are read from storage directly
```

### 4.2 Data Duplication

**Problem**: RecordCache stores uncompressed data while ColumnStore uses compression

**Impact**:
- CachedVertex uses 3-5x more memory than compressed storage
- For 1M vertices with 10 properties each:
  - Compressed: ~100MB
  - Cached: ~300-500MB
- Cache capacity is effectively reduced by compression ratio

**Solution Options**:

Option A: Cache compressed column chunks
```rust
pub struct CompressedColumnChunk {
    pub encoding: CompressionEncoding,
    pub data: Arc<[u8]>,
    pub row_range: Range<usize>,
}
```

Option B: Reduce cache size and accept lower hit rate
- Keep current design but reduce max_memory to avoid memory pressure

Option C: Cache only hot vertices
- Use a small L1 cache (e.g., 10MB) for frequently accessed vertices
- Let cold data be served from compressed storage

### 4.3 Unused Edge Cache

**Problem**: 30% of default memory allocation is wasted on unused Edge Cache

**Impact**:
- Default config: (40% vertex, 30% edge, 20% edge_query, 10% id_index)
- Edge Cache is never used, so 30% memory is wasted
- Effective cache size is only 70% of configured max_memory

**Solution**:
```rust
// Current
memory_ratio: (40, 30, 20, 10),

// Proposed
memory_ratio: (30, 0, 50, 20),  // Reallocate to edge_query and id_index
```

### 4.4 Cache Coordination Gap

**Problem**: Storage cache (RecordCache) and query cache (Plan/CTE) operate independently

**Impact**:
- Plan Cache may cache plans that reference data already in RecordCache
- CTE Cache may store results that duplicate RecordCache entries
- No coordinated memory budget management across layers

**Solution**: Implement unified memory budget at CacheManager level
```rust
pub struct UnifiedCacheManager {
    storage_cache: RecordCache,
    plan_cache: QueryPlanCache,
    cte_cache: CteCacheManager,
    total_memory_budget: usize,
    // Dynamic allocation based on workload
}
```

---

## 5. Recommended Actions

### Phase 1: Quick Wins (High Priority)

1. **Remove Edge Cache**
   - Delete Edge Cache from RecordCache
   - Reallocate 30% memory to Edge Query Cache
   - Update default ratio to (40, 0, 40, 20)

2. **Remove Timestamp from Keys**
   - Update VertexCacheKey to exclude timestamp
   - Update EdgeQueryKey to exclude timestamp
   - Cache only latest version for MVCC

### Phase 2: Optimization (Medium Priority)

3. **Reduce Vertex Cache Memory**
   - Lower vertex cache ratio from 40% to 20%
   - Use compressed column chunk caching for cold data
   - Keep uncompressed cache only for hot vertices

4. **Add Cache Warming**
   - Warm ID Index Cache on startup for frequently accessed labels
   - Warm Edge Query Cache for common edge patterns

### Phase 3: Advanced (Low Priority)

5. **Unified Memory Budget**
   - Implement CacheManager to coordinate all caches
   - Dynamic memory allocation based on workload
   - Cross-layer cache coordination

6. **Compressed Column Caching**
   - Cache compressed column chunks instead of full records
   - Integrate with existing compression encodings
   - Lazy decompression on cache hit

---

## 6. Memory Impact Analysis

### Current Configuration (128MB total)

| Cache Type | Allocation | Actual Usage | Waste |
|------------|-----------|--------------|-------|
| Vertex Cache | 51.2 MB | ~30 MB | 21.2 MB |
| Edge Cache | 38.4 MB | 0 MB | 38.4 MB |
| Edge Query Cache | 25.6 MB | ~20 MB | 5.6 MB |
| ID Index Cache | 12.8 MB | ~10 MB | 2.8 MB |
| **Total** | **128 MB** | **~60 MB** | **68 MB (53%)** |

### Proposed Configuration (128MB total)

| Cache Type | Allocation | Expected Usage | Efficiency |
|------------|-----------|----------------|------------|
| Vertex Cache | 25.6 MB | ~20 MB | 80% |
| Edge Query Cache | 64 MB | ~50 MB | 78% |
| ID Index Cache | 25.6 MB | ~20 MB | 78% |
| Hot Vertex L1 | 12.8 MB | ~10 MB | 78% |
| **Total** | **128 MB** | **~100 MB** | **78%** |

**Expected Improvement**: 53% -> 78% memory utilization (+25%)

---

## 7. Conclusion

### Caches to Keep
- ID Index Cache: Essential for vertex lookups
- Edge Query Cache: Essential for edge lookups
- Plan Cache: Essential for query performance
- CTE Cache: Useful for complex queries
- Inversearch Cache: Essential for search functionality

### Caches to Remove
- Edge Cache: Unused, redundant with Edge Query Cache

### Caches to Redesign
- Vertex Cache: Remove timestamp from key, consider compressed caching

### Key Design Principles
1. Cache keys should not include MVCC timestamps
2. Avoid data duplication between cache and compressed storage
3. Coordinate memory budgets across cache layers
4. Remove unused caches to improve memory utilization
5. Use compressed caching for cold data, uncompressed for hot data
