# CSR Architecture Overview

## What is CSR?

**CSR** (Compressed Sparse Row) is a column-oriented graph edge storage format used throughout GraphDB for efficient space utilization and fast neighbor traversal.

Instead of storing edges as a dense adjacency matrix, CSR stores them as:
- **Adjacency offset array**: where each vertex's edges start in the flattened edge list
- **Flattened edge list**: all edges for all vertices packed contiguously
- **Degree array**: how many edges each vertex has

This reduces memory usage from O(V²) to O(V + E).

## Current CSR Variants

GraphDB supports **6 CSR variants**, selected per relationship type:

| Variant | Type | Use Case | Complexity | Key Feature |
|---------|------|----------|-----------|-------------|
| `Multiple` | Mutable | Multi-edge relationships (general case) | O(degree) lookup | Two-level (primary + overflow) |
| `Single` | Mutable | One-to-one relationships | O(1) lookup | Direct array indexing |
| `MultiSingle` | Mutable | Multi-edge with limited capacity | O(degree) lookup | Fixed-size blocks per vertex |
| `Labeled` | Mutable | Multi-label edges | O(log K) label lookup | Label-grouped storage |
| `Immutable` | Read-only | Snapshots & batch-loaded data | O(degree) lookup | Flat layout, no fragmentation |
| `None` | Placeholder | No edges stored | - | Zero memory overhead |

## Selection via EdgeStrategy

The `EdgeStrategy` enum controls which variant is created for each direction (outgoing/incoming):

```rust
pub enum EdgeStrategy {
    None,      // No edges stored
    Single,    // Use SingleMutableCsr (one edge per vertex)
    #[default]
    Multiple,  // Use MutableCsr (multi-edge)
}
```

Relationships are created with both outgoing (`oe_strategy`) and incoming (`ie_strategy`) strategies:

```rust
pub struct EdgeSchema {
    pub oe_strategy: EdgeStrategy,  // How to store outgoing edges
    pub ie_strategy: EdgeStrategy,  // How to store incoming edges
    // ...
}
```

## Trait Hierarchy

### CsrBase (Fundamental)
Shared interface for all CSR types:
- `vertex_capacity()` - max vertices
- `edge_count()` - total edges
- `dump()` / `load()` - serialization

### MutableCsrTrait (Edit Operations)
Extended by all mutable variants:
- `insert_edge()` - add edge
- `delete_edge()` - remove by ID
- `delete_edge_by_dst()` - remove by destination
- `delete_edge_by_offset()` - remove by position
- `revert_delete_by_offset()` - undo deletion
- `get_edge()` - lookup
- `edges_of()` - get all neighbors
- `compact_with_ts()` - defragmentation
- `used_memory_size()` - memory estimation

### ImmutableCsr
Read-only interface (no mutations):
- `get_edge()` - lookup
- `edges_of()` - get all neighbors
- `dump()` / `load()` - serialization

## Runtime Polymorphism: CsrVariant

All 6 variants are wrapped in a **single enum** `CsrVariant` for runtime dispatch:

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

This design:
- ✅ Avoids `dyn` trait objects (no vtable overhead)
- ✅ Enables inline branching (compiler can optimize)
- ✅ Preserves type safety at compile time
- ✅ Allows runtime selection per relationship

## Timestamp & Versioning

All mutable CSRs support **versioned edges** via timestamps:

```rust
pub struct Nbr {
    pub neighbor: VertexId,
    pub edge_id: EdgeId,
    pub prop_offset: u32,
    pub create_ts: Timestamp,      // When edge was created
    pub delete_ts: Timestamp,      // When edge was deleted (u32::MAX = active)
}
```

Queries filter edges by timestamp:
```rust
csr.edges_of(src_vid, ts)  // Only edges valid at timestamp ts
```

This enables:
- Time-travel queries ("what was the graph at time T?")
- MVCC-based concurrency control
- Edge deletion via soft-delete (mark with delete_ts)

## Fragmentation Management

**Mutable CSR** (Multiple variant) uses two-level storage:
- **Primary block**: fixed-size slot per vertex
- **Overflow block**: append-only expansion when primary fills

Over time, repeated expansions create **internal fragmentation** (zombie blocks in `nbr_list`).

Recovery via `compact_with_ts()`:
- Merges primary + overflow into flat CSR
- Removes soft-deleted edges
- Reclaims all wasted space
- O(V + E) cost, requires exclusive write access

See [Fragmentation & Compaction](fragmentation.md) for details.

## File Organization

```
crates/graphdb-storage/src/storage/edge/
├── csr_trait.rs              # Trait definitions (CsrBase, MutableCsrTrait)
├── csr_variant.rs            # Enum wrapper, dispatch logic
├── mutable_csr.rs            # Multiple variant (two-level with overflow)
├── single_mutable_csr.rs     # Single variant (O(1) direct array)
├── multi_single_mutable_csr.rs  # MultiSingle variant
├── labeled_mutable_csr.rs    # Labeled variant (label-grouped)
├── immutable_csr.rs          # Immutable variant (flat, read-only)
├── csr.rs                    # (legacy, may be merged)
├── fragmentation_stats.rs    # Metrics reporting
├── edge_table.rs             # EdgeTable combines out/in CSRs + properties
└── property_table.rs         # Edge property storage
```

## Next Steps

- [Variant Details](variants.md) - deep dive into each CSR implementation
- [Dispatch Logic](dispatch.md) - how CSR is selected and created
- [Fragmentation & Compaction](fragmentation.md) - memory management
