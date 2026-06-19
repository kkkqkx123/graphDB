# CSR Variants: Implementation Details

## 1. MutableCsr (Multiple Variant)

**File**: `crates/graphdb-storage/src/storage/edge/mutable_csr.rs`

### Purpose
Standard multi-edge CSR for general cases where vertices can have many outgoing edges.

### Layout

```
Memory Layout:
┌─────────────────────────────┐
│  Vertex 0  │ Vertex 1 │ ...  │  Primary blocks (contiguous)
└─────────────────────────────┘
         │
         └──► Overflows (append-only at end)
              ┌────────────────┐
              │  Overflow V0   │
              │  Overflow V1   │
              │  ...           │
              └────────────────┘
```

### Data Structures

```rust
pub struct MutableCsr {
    nbr_list: Vec<Nbr>,              // All edges, flat
    adj_offsets: Vec<u32>,           // Where each vertex's edges start
    primary_capacities: Vec<u32>,    // Preallocated size per vertex
    degrees: Vec<u32>,               // Actual edge count per vertex
    overflow_starts: Vec<u32>,       // Where overflow block begins (if any)
    overflow_capacities: Vec<u32>,   // Overflow size per vertex
    edge_count: AtomicU64,           // Total edge count
    vertex_capacity: usize,
}
```

### Two-Level Storage Strategy

**Primary Block**: 
- Fixed pre-allocated space for each vertex (default: 4 edges)
- Located at `adj_offsets[v]` with size `primary_capacities[v]`
- Fast insertion if space available

**Overflow Block**:
- Created when primary fills up
- Appended to the end of `nbr_list`
- Grows dynamically via `expand_vertex_capacity()`

### Insertion Logic

```
insert_edge(src, dst, edge_id, prop_offset, ts):
  1. Check if edge already exists (duplicate detection)
  2. If primary block has space:
     → Write to primary block
  3. Else:
     → Allocate overflow block
     → Copy overflow (if exists) to new location
     → Append new edge
  4. Update degree counter
  5. Increment edge_count
```

### Fragmentation

**When does it occur?**
- Each `expand_vertex_capacity()` for a vertex allocates new space at the end of `nbr_list`
- Old overflow block becomes unreachable → internal fragmentation

**Cumulative effect**:
- Repeated expansions accumulate zombie blocks
- Memory wasted, but queries unaffected (only access via current pointer)

**Detection**:
```rust
csr.fragmentation_ratio()  // Returns waste_bytes / total_bytes
                           // e.g., 2.5 = 2.5x overhead
```

### Compaction

**Trigger**:
```rust
csr.maybe_compact(threshold: f32, ts: Timestamp, reserve_ratio: f32)
```
- Only compacts if `fragmentation_ratio() > threshold`

**Operation**:
- O(V + E) time, O(E) space
- Merges primary + overflow into flat CSR
- Removes soft-deleted edges (where `create_ts > ts`)
- Reserves `reserve_ratio` free space for future growth

**Example**:
```rust
// Compact if wasting > 2.5x memory, keep 25% reserve
csr.maybe_compact(2.5, current_ts, 0.25);
```

### Operations Complexity

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| `insert_edge` | O(1) amortized | Spills to overflow when full |
| `delete_edge` (by ID) | O(degree) | Scans primary + overflow |
| `get_edge` | O(degree) | Scans both levels |
| `edges_of` | O(degree) | Returns all valid edges |
| `compact_with_ts` | O(V + E) | Defragments storage |

---

## 2. SingleMutableCsr (Single Variant)

**File**: `crates/graphdb-storage/src/storage/edge/single_mutable_csr.rs`

### Purpose
Optimized for one-to-one relationships where each vertex has **at most one outgoing edge**.

### Use Cases
- "Spouse" relationships
- "Current employer"
- Any strict single-edge semantic

### Layout

```
Direct array indexing:
┌───┬───┬───┬───┐
│ V0│ V1│ V2│ V3│  nbr_list (one Nbr per vertex)
└───┴───┴───┴───┘
 0   1   2   3
```

### Data Structures

```rust
pub struct SingleMutableCsr {
    nbr_list: Vec<Nbr>,          // One edge per vertex (may be inactive)
    edge_count: AtomicU64,       // Count of active edges
    vertex_capacity: usize,
}
```

### Operations Complexity

| Operation | Complexity |
|-----------|-----------|
| `insert_edge` | O(1) |
| `delete_edge` | O(1) |
| `get_edge` | O(1) |
| `edges_of` | O(1) |

### ⚠️ Concurrency Limitation

**Critical**: This CSR does NOT support concurrent writes at the same timestamp.

**Behavior**:
- Each vertex can have at most 1 logically valid edge
- Newer timestamps **overwrite** older ones
- If two updates arrive with same/non-monotonic timestamp, the later one is **silently rejected**

**Example**:
```
T1: insert_edge(v0, v1, ts=100) ✓ succeeds
T2: insert_edge(v0, v2, ts=99)  ✗ rejected (99 < 100)
T3: insert_edge(v0, v3, ts=100) ✗ rejected (100 == 100, not >)
```

**Workarounds**:
1. Ensure timestamp monotonicity at upper layers (WAL, transaction log)
2. Use `MutableCsr` if concurrent writes needed
3. Design system where updates are strictly ordered

---

## 3. MultiSingleMutableCsr (MultiSingle Variant)

**File**: `crates/graphdb-storage/src/storage/edge/multi_single_mutable_csr.rs`

### Purpose
Hybrid approach: each vertex has multiple edges, but limited to a fixed capacity per vertex.

### Use Case
- Memory-constrained scenarios
- Known upper bound on edges per vertex
- Faster allocation than unbounded `MutableCsr`

### Layout

```
Fixed blocks per vertex:
┌──────────┐  ┌──────────┐  ┌──────────┐
│ Vertex 0 │  │ Vertex 1 │  │ Vertex 2 │
│(cap: K)  │  │(cap: K)  │  │(cap: K)  │
└──────────┘  └──────────┘  └──────────┘
```

### Data Structures

```rust
pub struct MultiSingleMutableCsr {
    edges: Vec<Vec<Nbr>>,        // Fixed-size vec per vertex
    vertex_capacity: usize,
    max_edges_per_vertex: usize, // Fixed limit
    edge_count: AtomicU64,
}
```

### Insertion Behavior
- Returns `false` if vertex's edge count reaches `max_edges_per_vertex`
- No overflow, strict capacity

---

## 4. LabeledMutableCsr (Labeled Variant)

**File**: `crates/graphdb-storage/src/storage/edge/labeled_mutable_csr.rs`

### Purpose
Multi-label CSR where edges from the same source-destination pair may have different labels.

### Use Case
- Multi-label graphs (e.g., "friend", "colleague", "family" on same pair)
- Efficient label-filtered traversal
- GraphQL queries with label conditions

### Layout

```
Label-grouped storage:
┌─────────────────────────────────┐
│ nbr_list (flattened by label)  │
└─────────────────────────────────┘

Per-vertex mapping:
label_ranges[v] = [
  { label: 1, offset: 0, count: 3 },
  { label: 5, offset: 3, count: 2 },
  ...
]
```

### Data Structures

```rust
pub struct LabeledMutableCsr {
    nbr_list: Vec<Nbr>,                    // All edges, flat
    label_ranges: Vec<Vec<LabelRange>>,    // Label → (offset, count) per vertex
    degrees: Vec<u32>,                     // Total edges per vertex
    edge_count: AtomicU64,
    vertex_capacity: usize,
}

struct LabelRange {
    label: LabelId,
    offset: u32,
    count: u32,
}
```

### Operations Complexity

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| `insert_edge` | O(log K) | K = distinct labels at vertex |
| `get_edge` (by label) | O(log K) | Binary search on label ranges |
| `edges_of` (all) | O(degree) | Return all label groups |

---

## 5. ImmutableCsr (Immutable Variant)

**File**: `crates/graphdb-storage/src/storage/edge/immutable_csr.rs`

### Purpose
Read-only, compact snapshot of a mutable CSR for:
- Static analysis
- Batch-loaded data
- Persistent storage format
- Fast analytical queries

### Layout

```
Flat CSR (no overflow, no fragmentation):
┌────────────────────────────────┐
│  All edges, contiguous         │  nbr_list (Box<[Nbr]>)
└────────────────────────────────┘

Offset array:
┌────────────┐
│ Offsets[V] │  adj_offsets (Box<[u32]>)
│ Last entry │  = total edge count
└────────────┘

Degrees:
┌──────────────┐
│ Degrees[V]   │  degrees (Box<[u32]>)
└──────────────┘
```

### Data Structures

```rust
pub struct ImmutableCsr {
    nbr_list: Box<[Nbr]>,       // Immutable slice, no Vec overhead
    adj_offsets: Box<[u32]>,    // Where each vertex's edges start
    degrees: Box<[u32]>,        // Edge count per vertex
    vertex_capacity: usize,
}
```

### Key Differences from Mutable CSR

| Aspect | Mutable | Immutable |
|--------|---------|-----------|
| **Storage** | Vec (growable) | Box<[T]> (fixed) |
| **Fragmentation** | Yes (overflow blocks) | No (flat layout) |
| **Mutations** | `insert_edge`, `delete_edge` | None (build-only) |
| **Build phase** | Incremental | `batch_put_edge()` then `build()` |
| **Memory** | Higher (capacity > usage) | Lower (capacity == usage) |
| **Lookup** | O(degree) | O(degree) |

### Construction

```rust
// From snapshot
let immutable = ImmutableCsr::from_snapshot(&mutable_csr, snapshot_ts);

// From scratch
let mut builder = ImmutableCsr::builder(1000);
builder.batch_put_edge(0, dst_vid, edge_id, prop_offset);
builder.batch_put_edge(0, dst_vid2, edge_id2, prop_offset2);
let immutable = builder.build();
```

### Operations

| Operation | Behavior |
|-----------|----------|
| `get_edge` | Direct array lookup, ignores timestamp (immutable snapshot) |
| `edges_of` | Returns all edges, O(degree) |
| `dump` / `load` | Serialization of flat layout |
| `insert_edge` | Returns `false` (read-only) |
| `delete_edge` | Returns `false` (read-only) |

---

## 6. None (Placeholder Variant)

**File**: `crates/graphdb-storage/src/storage/edge/csr_variant.rs` (lines 56, 81, 321-362)

### Purpose
Placeholder for relationships with **no edges stored**.

### Use Case
- Directed relationships where only the schema exists
- Zero-cost edge storage
- Placeholder in polymorphic context

### Data Structure

```rust
None { vertex_capacity: usize }  // Only stores capacity, no edges
```

### Behavior

| Operation | Result |
|-----------|--------|
| `edge_count()` | 0 |
| `insert_edge()` | `false` (rejected) |
| `delete_edge()` | `false` (rejected) |
| `get_edge()` | `None` |
| `edges_of()` | Empty vec |
| `iter()` | Empty iterator |
| Memory | `sizeof(usize)` |

### Serialization

```
dump(): [0u8, vertex_capacity (8 bytes)] // Tag 0 = None
load(): Deserializes vertex_capacity, recreates None variant
```

---

## Trait Implementation Matrix

| Trait Method | Multiple | Single | MultiSingle | Labeled | Immutable | None |
|--------------|----------|--------|-------------|---------|-----------|------|
| `vertex_capacity` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `edge_count` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ (0) |
| `dump` / `load` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| `insert_edge` | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ |
| `delete_edge` | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ |
| `get_edge` | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ |
| `edges_of` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ (empty) |
| `compact_with_ts` | ✓ | ✓ (no-op) | ✓ | ✓ | ✓ | ✓ (no-op) |
| `used_memory_size` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

---

## Comparison: When to Use Which

| Scenario | Variant | Reason |
|----------|---------|--------|
| "Friends" (multi-edge, general) | `Multiple` | Default, handles any case |
| "Spouse" (one-to-one) | `Single` | O(1) access, memory efficient |
| "Followers" (bounded multi-edge, ~1K per vertex) | `MultiSingle` | Fixed memory, faster allocation |
| "Collaborates on [project/paper/team]" (multi-label) | `Labeled` | Efficient label filtering |
| Analytical snapshot, batch-loaded data | `Immutable` | Flat, compact, read-only |
| Schema exists but no actual edges stored | `None` | Zero overhead |

