# Vertex/Edge Storage Architecture Issues

> Analysis date: 2026-05-17
> Scope: `src/storage/vertex/` and `src/storage/edge/`

## Overview

This document describes architecture-level design issues in the vertex and edge storage modules. These issues require structural changes rather than localized bug fixes.

---

## 1. Column Store (Vertex) vs Row Store (Edge) Asymmetry

### Problem

Vertex storage uses a **columnar layout** (`ColumnStore`) with 7 compression encodings (Dictionary, RLE, BitPacking, FSST, ALP, Varint, Lazy), while edge property storage uses a **row-oriented layout** (`PropertyTable`) with zero compression support.

| Aspect | Vertex (ColumnStore) | Edge (PropertyTable) |
|--------|---------------------|---------------------|
| Storage orientation | Columnar | Row-oriented |
| Compression | 7 encoding types | None |
| Null handling | Dedicated `BitVec` per column | `Option<Value>` per cell |
| Schema evolution | `add_column()` per property | `add_property()` per property |
| Memory efficiency | High (type-specific encoding) | Low (row overhead per entry) |

### Impact

- Edge properties consume more memory than necessary in high-cardinality scenarios (e.g., millions of edges with repeated values)
- No compression means edge scan/iteration pays full I/O cost for all properties
- Cannot leverage type-specific optimizations (e.g., RLE for repeated edge types, BitPacking for small-range weights)

### Root Cause

`PropertyTable` was implemented as a simple `Vec<PropertyRow>` without considering columnar compression. The design decision to use row-oriented storage for edges was intentional (edges are typically accessed by full row), but the lack of any compression layer is an oversight.

### Suggested Approach

Consider a **hybrid design** where `PropertyTable` internally uses a columnar layout for individual properties but presents a row-oriented API:

```
PropertyTable (row API)
  ├── Column 1 (weight: Double) → [compression encoding]
  ├── Column 2 (since: Int)     → [compression encoding]
  └── Column 3 (metadata: String) → [compression encoding]
```

This would reuse the existing `Column` and encoding infrastructure from `vertex::column_store` while maintaining the row-level access pattern that edges require.

---

## 2. EdgeTable Persistence Gap: Critical Missing State

### Problem

`EdgeTable::flush()` persists CSR data and property data but **does not persist** two critical in-memory data structures:

- `active_vertices: HashSet<VertexId>` — tracks which vertices have edges, required by `scan()`
- `edge_id_to_src: HashMap<EdgeId, (VertexId, VertexId)>` — maps edge ID to (src, dst), required by `delete_edge_by_id()`

After `load()`, both structures are empty, causing:

- `scan()` returns zero edges
- `delete_edge_by_id()` always returns `Ok(false)` (edge not found)
- `has_edge()` and `get_edge()` still work (CSR data is intact)

### Impact

**Critical data loss.** Any edge table loaded from disk becomes partially non-functional:
- Full table scans are broken
- ID-based edge deletion is broken
- Only point lookups (get_edge by src/dst) work correctly

### Root Cause

The flush/load serialization was implemented without a complete inventory of all state fields. `active_vertices` and `edge_id_to_src` were added later for scan/by-id operations but were never included in the persistence path.

### Suggested Fix

1. Add serialization of `active_vertices` and `edge_id_to_src` to `flush()` and `load()`
2. Alternatively, rebuild these structures from CSR data on load (iterate out_csr to collect active vertices and edge_id mappings)
3. Add a test that verifies flush/load roundtrip preserves all operation capabilities

---

## 3. MVCC Implementation Divergence

### Problem

Vertex and edge modules implement MVCC timestamp tracking differently:

**Vertex MVCC** (`VertexTimestamp`):
- Separate `start_ts` / `end_ts` vectors
- Explicit `deleted` flag
- `is_valid(internal_id, ts)` checks `start <= ts < end`
- Supports `revert_remove()` → `revert_remove()` lifecycle

**Edge MVCC** (embedded in `Nbr`):
- Single `timestamp` field in `Nbr` struct
- `INVALID_TIMESTAMP` signals deletion
- `is_valid` logic is implicitly checked by CSR iterators
- No explicit end_ts / start_ts separation

### Impact

- Vertex MVCC supports version lifecycle management (creation → active → deletion → reversion)
- Edge MVCC has a simpler model: present (valid timestamp) or deleted (INVALID_TIMESTAMP)
- Edge MVCC cannot distinguish between "never existed" and "was deleted long ago"
- Edge compaction must guess valid edges based solely on timestamp presence

### Root Cause

The edge module uses a simpler MVCC model designed for CSR adjacency lists where each neighbor entry only needs a "valid or not" bit for the current version. The vertex module needs full MVCC for property versioning.

### Suggested Approach

Evaluate whether edges need full MVCC or if the current simplified model is sufficient-timestamp approach is adequate. If edge property versioning is needed in the future, `PropertyTable` entries should also carry version metadata.

---

## 4. Shared PropOffset Between Out-CSR and In-CSR

### Problem

When an edge is inserted, both `out_csr` and `in_csr` entries reference the **same** `prop_offset` value, pointing to the same row in `PropertyTable`. This creates tight coupling:

```rust
// edge_table.rs insert_edge()
self.out_csr.insert_edge(src, dst, edge_id, prop_offset, ts);
self.in_csr.insert_edge(dst, src, edge_id, prop_offset, ts); // same prop_offset
```

### Impact

- Deleting an edge requires deleting properties only once (not twice)
- Edge property updates via `update_edge_property_by_offset()` redundantly writes to both CSR entries (same property row, so no data corruption, but unnecessary)
- Cannot independently version out-edge vs in-edge properties

### Root Cause

Design optimization to avoid duplicating property data for bidirectional adjacency.

### Assessment

This is **acceptable** for the current use case. The shared offset is an optimization, not a bug. However, it should be explicitly documented and the redundancy in `update_edge_property_by_offset()` should be cleaned up.

---

## 5. No Unified Storage Abstraction

### Problem

`VertexTable` and `EdgeTable` share no common trait or interface, despite having similar operations:

| Operation | VertexTable | EdgeTable |
|-----------|-------------|-----------|
| insert | `insert()` | `insert_edge()` |
| get | `get()` / `get_by_internal_id()` | `get_edge()` |
| delete | `delete()` / `delete_by_internal_id()` | `delete_edge()` / `delete_edge_by_id()` |
| scan | `scan()` | `scan()` |
| flush | `flush()` | `flush()` |
| load | `load()` | `load()` |
| schema | `schema()` | `schema()` |

### Impact

- Storage engine (`property_graph`) must handle vertex and edge tables separately
- Cannot write generic batch operations
- Iterator implementations are duplicated for vertex vs edge
- Cache layer must use different types for vertex vs edge lookups

### Root Cause

The modules evolved independently before being integrated into the property graph engine.

### Suggested Approach

Introduce a `StorageTable` trait that both `VertexTable` and `EdgeTable` implement. At minimum, define common operations for persistence, schema, and scan. The trait can be introduced incrementally without refactoring all at once.

---

## 6. PropertyTable FreeList vs ColumnStore Growth Strategy

### Problem

`PropertyTable` uses a free list to reuse deleted slot offsets, while `ColumnStore` relies on `resize/grow without a free list mechanism:

**PropertyTable** (edge):
- Free list stores deleted offsets
- `insert()` pops from free list first, else appends
- Offset reuse is immediate

**ColumnStore** (vertex):
- No free list
- Deleted vertex data remains in column storage until `compact()`
- Space reclamation requires explicit compaction call

### Impact

- Consequence

- Edge property storage reuses offsets aggressively, which complicates debugging and version tracking
- Vertex storage never reuses internal IDs unless `compact()` is called
- Two different fragmentation behavior profiles for similar operations

### Assessment

This difference is partly justified by the usage pattern (edges have more churn), but the lack of a unified approach creates inconsistency.

---

## 7. EdgeStrategy::None Allows Invalid EdgeTable State

### Problem

`EdgeTable::new()` and `with_config()` allow creating an `EdgeTable` with `EdgeStrategy::None`, but `insert_edge()` returns an error when the strategy is None:

```rust
if self.schema.oe_strategy == EdgeStrategy::None {
    return Err(StorageError::invalid_operation("Edge strategy is None"));
}
```

### Impact

- EdgeTable can exist in a state where it can never contain edges
- Memory is allocated for CSR structures even when no edges can be stored
- The error should be caught at construction time, not operation time

### Root Cause

The `EdgeStrategy::None` variant was designed for schema completeness but should not create operational EdgeTable instances.

---

## 8. Summary

| # | Issue | Severity | Category |
|---|-------|----------|----------|
| 1 | Column vs Row asymmetry | Medium | Structural |
| 2 | Active vertices / edge_id_to_src not persisted | **Critical** | Data loss |
| 3 | MVCC implementation divergence | Low | Design choice |
| 4 | Shared prop_offset coupling | Low | Optimization |
| 5 | No unified storage abstraction | Medium | Extensibility |
| 6 | FreeList vs compaction fragmentation | Low | Consistency |
| 7 | EdgeStrategy::None creates invalid tables | Low | Robustness |

### Priority Actions

1. **P0**: Fix EdgeTable persistence (issue #2)
2. **P1**: Evaluate PropertyTable compression (issue #1)
3. **P2**: Define storage trait abstraction (issue #5)
4. **P3**: Revisit EdgeStrategy::None handling (issue #7)