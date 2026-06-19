# CSR (Compressed Sparse Row) Storage Documentation

Complete documentation of CSR architecture, variants, dispatch logic, and memory management in GraphDB.

## Quick Navigation

### For Quick Answers
- **[Quick Reference](quick_reference.md)** — Code examples, API reference, common pitfalls

### For Understanding
- **[Overview](overview.md)** — What is CSR, why 6 variants, trait hierarchy
- **[Variants](variants.md)** — Deep dive into each implementation (Multiple, Single, MultiSingle, Labeled, Immutable, None)
- **[Dispatch Logic](dispatch.md)** — How CSR is selected, created, and polymorphically dispatched

### For Optimization & Maintenance
- **[Fragmentation & Compaction](fragmentation.md)** — Memory management, compaction strategies, when & how

---

## Core Concepts at a Glance

### What is CSR?
Compressed Sparse Row is a memory-efficient representation of sparse adjacency lists. Instead of storing a dense V×V matrix, CSR stores:
- Adjacency offsets (where each vertex's edges start)
- Flattened edge list (all edges packed contiguously)
- Degree array (edge count per vertex)

Space: O(V + E) instead of O(V²)

### Six Variants

| Variant | Storage | Lookups | Use Case |
|---------|---------|---------|----------|
| **Multiple** | Two-level (primary + overflow) | O(degree) | Multi-edge relationships (general) |
| **Single** | Direct array | O(1) | One-to-one relationships |
| **MultiSingle** | Fixed-size blocks | O(degree) | Bounded multi-edge |
| **Labeled** | Label-grouped | O(log K) | Multi-label edges |
| **Immutable** | Flat, read-only | O(degree) | Snapshots, batch-loaded |
| **None** | Placeholder | - | No edges stored |

### Runtime Polymorphism: CsrVariant

All 6 variants wrapped in a single enum for zero-vtable dispatch:
```rust
pub enum CsrVariant {
    Multiple(MutableCsr),
    Single(SingleMutableCsr),
    MultiSingle(MultiSingleMutableCsr),
    Labeled(LabeledMutableCsr),
    Immutable(ImmutableCsr),
    None { vertex_capacity: usize },
}
```

---

## Architecture

### Trait Hierarchy

```
CsrBase (All variants)
├─ vertex_capacity()
├─ edge_count()
├─ dump() / load()
└─ (Read & serialize operations)

MutableCsrTrait (Mutable variants)
├─ insert_edge()
├─ delete_edge() / delete_edge_by_dst() / delete_edge_by_offset()
├─ revert_delete_by_offset()
├─ get_edge()
├─ edges_of()
├─ compact_with_ts()
└─ used_memory_size()
```

### File Organization

```
crates/graphdb-storage/src/storage/edge/
├── csr_variant.rs              # Enum wrapper & dispatch
├── csr_trait.rs                # Trait definitions
├── mutable_csr.rs              # Multiple variant
├── single_mutable_csr.rs       # Single variant
├── multi_single_mutable_csr.rs # MultiSingle variant
├── labeled_mutable_csr.rs      # Labeled variant
├── immutable_csr.rs            # Immutable variant
├── fragmentation_stats.rs      # Metrics
├── edge_table.rs               # EdgeTable (combines out/in CSRs)
└── property_table.rs           # Edge properties
```

---

## Key Design Decisions

### 1. Enum-Based Dispatch (No Vtable)
✅ Inline-friendly, compiler can optimize  
✅ Type-safe at compile time via pattern matching  
✅ Preserves monomorphic code generation  

### 2. Two-Level Storage (MutableCsr)
✅ O(1) amortized insertion (no reshuffle)  
⚠️ Internal fragmentation over time  
✓ Mitigated via compaction  

### 3. Timestamp Versioning (Soft-Delete)
✅ MVCC support (multiple snapshots)  
✅ Fast deletion (mark, not remove)  
✅ Time-travel queries possible  

### 4. Single for One-to-One
✅ O(1) access, minimal memory  
⚠️ Requires monotonic timestamp ordering  

### 5. Labeled for Multi-Label
✅ O(log K) label-filtered traversal  
✅ Compact label storage  

---

## When to Use Each Variant

| Scenario | Variant | Reason |
|----------|---------|--------|
| Multi-edge relationship (friends, follows) | Multiple | Default, handles any case |
| One-to-one relationship (spouse, employer) | Single | O(1) lookup, minimal memory |
| Known bounded edges per vertex (< 1K) | MultiSingle | Fixed allocation, efficient |
| Multi-label same source-destination | Labeled | Efficient label filtering |
| Batch-loaded analytical data | Immutable | Flat, compact, read-only |
| Schema exists but no edges stored | None | Zero overhead |

---

## Common Operations

### Creation
```rust
let csr = CsrVariant::from_strategy(EdgeStrategy::Multiple, 1000, 10000)?;
```

### Query
```rust
let edge = csr.get_edge(src_vid, dst_vid, timestamp);
let neighbors = csr.edges_of(src_vid, timestamp);
```

### Mutation
```rust
csr.insert_edge(src_vid, dst_vid, edge_id, prop_offset, timestamp);
csr.delete_edge(src_vid, edge_id, timestamp);
```

### Maintenance
```rust
if csr.fragmentation_ratio() > 2.5 {
    csr.compact_with_ts(timestamp, 0.25);
}
```

---

## Critical Warnings

### ⚠️ SingleMutableCsr Concurrency
Single variant does NOT support concurrent writes at same timestamp. Ensure monotonic ordering or use Multiple variant.

### ⚠️ Fragmentation in Multiple Variant
Repeated overflow expansions create internal fragmentation. Monitor `fragmentation_ratio()` and compact when needed.

### ⚠️ Timestamp Filtering Required
All queries must pass timestamp. Omitting or using `u32::MAX` may include deleted edges.

### ⚠️ Immutable is Read-Only
ImmutableCsr rejects all write operations. Use MutableCsr for mutable workloads.

---

## Performance Checklists

### Before Serialization
- [ ] Check `fragmentation_ratio()`
- [ ] If > 1.5, call `compact_with_ts()`
- [ ] Verify dump size is reasonable

### After Bulk Deletion
- [ ] Monitor edge count
- [ ] If many soft-deletes, compact to hard-delete
- [ ] Free reclaimed memory

### High-Concurrency Systems
- [ ] Use Multiple variant (avoid Single's concurrency limitations)
- [ ] Disable frequent compaction (reduce lock contention)
- [ ] Monitor fragmentation growth rate

### OLAP / Analytics
- [ ] Convert to ImmutableCsr if snapshot needed
- [ ] Compact before export to reduce size
- [ ] Consider time-travel queries with timestamps

---

## Debugging & Diagnostics

### Check Fragmentation
```rust
println!("Fragmentation: {:.2}x", csr.fragmentation_ratio());
```

### Estimate Memory Usage
```rust
println!("Memory: {} MB", csr.used_memory_size() / 1_000_000);
```

### Inspect Edge Count
```rust
println!("Edges: {}", csr.edge_count());
```

### Iterate All Edges
```rust
for (vid, nbr) in csr.iter(timestamp) {
    println!("V{} -> {:?}", vid, nbr.neighbor);
}
```

---

## Testing

Unit tests in `csr_variant.rs`:
- ✅ Multiple, Single, MultiSingle, None, Immutable variants
- ✅ Insertion, deletion, query operations
- ✅ Timestamp visibility filtering
- ✅ Serialization round-trip
- ✅ Compaction behavior
- ✅ Edge cases (empty, full, overflow)

Run:
```bash
cargo test --lib storage::edge::csr_variant -- --nocapture
```

---

## References

- **Compressed Sparse Row (CSR)** — Standard matrix format, O(V+E) space
- **Two-Level CSR** — Overflow blocks to avoid O(n) reshuffling on growth
- **Soft-Delete** — MVCC pattern, mark with delete_ts instead of removing
- **MVCC** — Multi-Version Concurrency Control, supports time-travel queries

---

## Related Documentation

- `docs/storage/` — Overall storage architecture
- `crates/graphdb-storage/src/storage/edge/` — Implementation source
- AGENTS.md — Project conventions (no backward compatibility requirements)

