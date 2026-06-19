# Fragmentation & Compaction in MutableCsr

## Problem: Why Fragmentation?

### Two-Level Storage Design

MutableCsr uses a two-level approach to avoid O(n) reshuffling:

```
Initial State (primary blocks contiguous):
┌─────────────────────────────────┐
│V0: [E0, E1] │ V1: [E2] │ V2: []│  Primary blocks
└─────────────────────────────────┘

After V0 fills up:
┌─────────────────────────────────┐
│V0: [E0, E1] │ V1: [E2] │ V2: []│  Primary
└─────────────────────────────────┘
                    └──→ [E3] (overflow appended)

After V0 expands (fills again):
┌─────────────────────────────────┐
│V0: [E0, E1] │ V1: [E2] │ V2: []│  Primary (unchanged)
└─────────────────────────────────┘
     └────────────────────→ [E3] ← zombie (unreachable)
                    └──→ [E3, E4, E5] (new overflow)
```

### Root Cause

Each vertex expansion allocates new space at **end of `nbr_list`**:
1. Old overflow block address becomes unreachable
2. Pointer at `overflow_starts[v]` updated to new location
3. Old space never reclaimed → internal fragmentation

### Cumulative Effect

After many vertex expansions:
- `nbr_list` contains both live and dead edges
- Serialization dumps **entire** list including zombie blocks
- Queries unaffected (always use current pointers)
- Memory wasted but correctness preserved

---

## Measuring Fragmentation

### Fragmentation Ratio

**Definition**:
```
fragmentation_ratio = total_capacity / used_edges
```

**Examples**:
- `1.0`: No wasted space (perfectly packed)
- `2.0`: 50% wasted (2x the space needed)
- `5.0`: 80% wasted (very fragmented)

**Location**: `MutableCsr::fragmentation_ratio()`

```rust
pub fn fragmentation_ratio(&self) -> f32 {
    let used = self.edge_count.load(Ordering::Relaxed) as f32;
    if used == 0.0 {
        return 1.0;
    }
    self.nbr_list.len() as f32 / used
}
```

### Diagnostics

Query fragmentation status:

```rust
let ratio = csr.fragmentation_ratio();
if ratio > 2.5 {
    println!("High fragmentation: {:.2}x", ratio);
}
```

---

## Compaction: Recovery

### Purpose

Merge primary + overflow blocks into **flat CSR** layout:
- ✅ Removes all zombie blocks
- ✅ Merges soft-deleted edges (soft-delete with timestamp)
- ✅ Restores `fragmentation_ratio()` to ~1.0
- ✅ Reduces serialization size

### Method Signature

```rust
pub fn compact_with_ts(&mut self, ts: Timestamp, reserve_ratio: f32) -> usize
```

**Parameters**:
- `ts`: Timestamp threshold
  - Removes edges with `create_ts > ts` (not yet valid)
  - Soft-deleted edges (with `delete_ts < u32::MAX`) are removed
- `reserve_ratio`: Reserve fraction for future growth
  - `0.25` = reserve 25% extra capacity
  - Reduces need for immediate re-expansion

**Returns**: Number of edges removed

### Algorithm

```
compact_with_ts(ts, reserve_ratio):
  1. Allocate new nbr_list with reserved capacity
  2. For each vertex v:
     a. Iterate primary block [offset[v], offset[v] + degree[v])
     b. Iterate overflow block (if exists)
     c. Filter: keep only edges where create_ts <= ts AND delete_ts == u32::MAX
     d. Append valid edges to new list
  3. Rebuild offsets and degrees for flat layout
  4. Clear overflow pointers (no more needed)
  5. Replace nbr_list with compacted version
  6. Return count of removed edges
```

### Complexity

- **Time**: O(V + E) — visit all vertices and edges once
- **Space**: O(E) — allocate new edge list
- **Lock**: Exclusive write access required (not concurrent)

### Example

```rust
// Before compaction
let ratio = csr.fragmentation_ratio();  // 3.2x

// Compact: remove old edges (ts > 1000), keep 25% reserve
let removed = csr.compact_with_ts(1000, 0.25);
println!("Removed {} edges", removed);

// After compaction
let ratio = csr.fragmentation_ratio();  // ~1.25x (25% reserve)
```

---

## When to Compact

### Automatic: maybe_compact()

**Location**: `CsrVariant::maybe_compact()`

```rust
pub fn maybe_compact(&mut self, threshold: f32, ts: Timestamp, reserve_ratio: f32) {
    if let CsrVariant::Multiple(csr) = self {
        if csr.should_compact(threshold) {
            csr.compact_with_ts(ts, reserve_ratio);
        }
    }
}
```

**Pattern**:
- Query fragmentation ratio
- If > threshold, invoke `compact_with_ts()`
- Threshold typically `2.5` (50% waste)

### Scenarios

| Scenario | When | Action |
|----------|------|--------|
| **High-throughput writes** | Rare | Monitor ratio, compact during off-peak |
| **Batch deletion** | After massive delete | Compact to reclaim space |
| **Before serialization** | Snapshot time | Compact to reduce disk size |
| **Periodic maintenance** | Scheduled task | e.g., hourly if ratio > 3.0 |
| **Memory pressure** | OOM near threshold | Emergency compact |

### Production Heuristic

```rust
// Check periodically
if csr.fragmentation_ratio() > 2.5 {
    // Compact during off-peak hours
    csr.compact_with_ts(current_ts, 0.25);
}

// Before persistent snapshot
if csr.fragmentation_ratio() > 1.5 {
    csr.compact_with_ts(snapshot_ts, 0.1);
}
```

---

## Trade-offs

### Costs of NOT Compacting

| Impact | Effect |
|--------|--------|
| **Disk usage** | Serialized snapshots bloated by 2-5x |
| **Network** | Large transfers of fragmented CSR |
| **Cache efficiency** | Dead edges waste CPU cache lines |
| **Query latency** | Slight overhead scanning dead blocks |

### Costs of Compacting

| Impact | Effect |
|--------|--------|
| **CPU time** | O(V + E) full scan and rewrite |
| **Lock duration** | Exclusive write access (blocks other writers) |
| **Memory peak** | Temporarily 2x space during rewrite |
| **Latency spike** | Queries blocked during compaction |

### Recommendation

- **High-concurrency OLTP**: Rarely compact (throughput cost too high)
- **OLAP / Analytics**: Compact before snapshot export (size matters)
- **Batch loads**: Compact after bulk insertions (avoid initial overflow)
- **Retention-heavy workloads**: Compact monthly if soft-delete ratio > 50%

---

## Compaction in Other Variants

### MutableCsr (Multiple)
- ✅ Full compaction supported
- ✅ Removes soft-deleted edges

### SingleMutableCsr
- ⚠️ No-op (returns 0)
- Rationale: O(1) direct access, no overflow, no fragmentation

### MultiSingleMutableCsr
- ⚠️ Minimal compaction (fixed-size blocks)
- Rationale: Blocks pre-sized, limited expansion

### LabeledMutableCsr
- ✅ Full compaction supported (but more complex)
- Rebuilds label ranges after flattening

### ImmutableCsr
- ⚠️ No-op (read-only snapshot already flat)
- Rationale: Immutable data, no mutations possible

### None
- ⚠️ No-op (zero edges)
- Rationale: No edge storage

---

## Advanced: Manual Control

### Check Before Deciding

```rust
let ratio = csr.fragmentation_ratio();
let edge_count = csr.edge_count();
let capacity_bytes = csr.used_memory_size();

if ratio > 2.0 && edge_count > 10000 {
    println!("Fragmentation Alert:");
    println!("  Ratio: {:.2}x", ratio);
    println!("  Edges: {}", edge_count);
    println!("  Memory: {} MB", capacity_bytes / 1_000_000);
}
```

### Controlled Compaction

```rust
// Option 1: Compact with strict reserve
csr.compact_with_ts(current_ts, 0.1);  // 10% reserve, tight packing

// Option 2: Compact with generous reserve for growth
csr.compact_with_ts(current_ts, 0.5);  // 50% reserve, faster expansion

// Option 3: Compact old snapshots only
csr.compact_with_ts(snapshot_ts, 0.05);  // Minimal reserve, removes old data
```

### Monitoring

```rust
pub struct FragmentationMetrics {
    pub fragmentation_ratio: f32,
    pub total_edges: u64,
    pub used_bytes: usize,
    pub wasted_bytes: usize,
}

fn collect_metrics(csr: &MutableCsr) -> FragmentationMetrics {
    let ratio = csr.fragmentation_ratio();
    let edges = csr.edge_count();
    let used = csr.used_memory_size();
    let wasted = (used as f32 * (1.0 - 1.0 / ratio)) as usize;

    FragmentationMetrics {
        fragmentation_ratio: ratio,
        total_edges: edges,
        used_bytes: used,
        wasted_bytes: wasted,
    }
}
```

---

## Soft-Delete Semantics

### Create & Delete Timestamps

```rust
pub struct Nbr {
    pub neighbor: VertexId,
    pub edge_id: EdgeId,
    pub prop_offset: u32,
    pub create_ts: Timestamp,      // When added
    pub delete_ts: Timestamp,      // When soft-deleted (u32::MAX = active)
}
```

### Visibility Window

Edge is visible at timestamp `T` if:
```rust
create_ts <= T && T < delete_ts
```

### Soft-Delete Process

1. **Delete operation**: Set `delete_ts = current_ts`
2. **Query**: Filters out edges where `delete_ts <= query_ts`
3. **Compaction**: Removes edges where `delete_ts < current_ts` (hard-delete)

**Benefits**:
- ✅ Fast deletion (no reallocation)
- ✅ MVCC support (multiple snapshots see different state)
- ✅ Time-travel queries (query past state)
- ✅ Undo capability (revert can reset `delete_ts`)

---

## Serialization & Fragmentation

### dump() includes fragmentation

```rust
fn dump(&self) -> Vec<u8> {
    let mut result = vec![1u8];  // Tag for Multiple
    result.extend(self.nbr_list.len().to_le_bytes());  // Entire list including zombies!
    result.extend_from_slice(/* ... serialized edges ... */);
    result
}
```

**Impact**: 
- Fragmented CSR serializes at 2-5x size
- Network transfer cost increases
- Disk storage inflated

**Mitigation**: 
- Compact before serialization if `ratio > 1.5`

### load() reconstructs fragmented state

```rust
fn load(&mut self, data: &[u8]) -> StorageResult<()> {
    // Deserializes entire nbr_list (including zombies)
    // Restores exact fragmentation state from snapshot
}
```

**Note**: Deserialized CSR may already be fragmented. Consider re-compacting if ratio > 2.0.

---

## Future Optimizations

### Lazy Compaction
- Mark blocks for compaction but defer actual work
- Batch compactions during idle time
- Reduce user-facing latency spikes

### Incremental Compaction
- Compact one vertex at a time
- Amortize O(V + E) cost over many operations
- Maintain query latency SLOs

### Adaptive Thresholds
- Monitor workload patterns
- Auto-tune compaction threshold
- Trigger early if write rate drops

