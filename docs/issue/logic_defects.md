# Vertex/Edge Storage Logic Defects

> Analysis date: 2026-05-17
> Scope: `src/storage/vertex/` and `src/storage/edge/`

## Overview

This document catalogs localized logic issues, bugs, and code quality problems in the vertex and edge storage modules. Each defect includes the exact location, root cause, impact, and suggested fix.

---

## 1. VertexTable::get_external_id() Missing Timestamp Check

### Location

[src/storage/vertex/vertex_table.rs#L358](file:///d:/%E9%A1%B9%E7%9B%AE/database/graphDB/src/storage/vertex/vertex_table.rs#L358)

### Code

```rust
pub fn get_external_id(&self, internal_id: u32) -> Option<String> {
    self.id_indexer.get_key(internal_id).cloned()
}
```

### Problem

The method returns external IDs for **deleted** vertices. Unlike `get()` and `get_by_internal_id()`, it does not verify MVCC timestamp validity. If a vertex was deleted, its `IdIndexer` entry may still exist (marked as inactive via timestamp), but `get_external_id()` will still return the ID.

### Impact

Callers that use `get_external_id()` (e.g., `transaction.rs` to resolve vertex IDs for edge insertion) could receive IDs of deleted vertices, leading to incorrect edge linking.

### Root Cause

The method was added as a simple lookup without considering MVCC semantics. The corresponding `get_internal_id()` method (line 345) correctly checks timestamps.

### Fix Suggestion

Add a timestamp parameter and validity check:

```rust
pub fn get_external_id(&self, internal_id: u32, ts: Timestamp) -> Option<String> {
    if !self.is_open || !self.timestamps.is_valid(internal_id, ts) {
        return None;
    }
    self.id_indexer.get_key(internal_id).cloned()
}
```

Update all callers to provide a timestamp.

---

## 2. PropertyTable::set_property_by_id() Missing Negative prop_id Validation

### Location

[src/storage/edge/property_table.rs#L194-L206](file:///d:/%E9%A1%B9%E7%9B%AE/database/graphDB/src/storage/edge/property_table.rs#L194-L206)

### Code

```rust
pub fn set_property_by_id(
    &mut self,
    offset: u32,
    prop_id: i32,
    value: Option<Value>,
) -> StorageResult<()> {
    let col_idx = prop_id as usize;  // <-- negative i32 wraps to large usize
    ...
    if col_idx >= self.schema.len() {
        return Err(StorageError::column_not_found(...));
    }
    ...
}
```

### Problem

`prop_id` is `i32`, and `as usize` on a negative value wraps to a very large positive number. The subsequent bounds check (`col_idx >= self.schema.len()`) catches the error, but produces a misleading error message (column_not_found instead of invalid_prop_id).

### Impact

- No crash or UB (bounds check catches it), but the error message is misleading
- Makes debugging harder if a negative prop_id propagates from upper layers

### Root Cause

`prop_id` should be `u32` (or the conversion should explicitly check for negative values).

### Fix Suggestion

```rust
pub fn set_property_by_id(
    &mut self,
    offset: u32,
    prop_id: i32,
    value: Option<Value>,
) -> StorageResult<()> {
    if prop_id < 0 {
        return Err(StorageError::invalid_parameter(format!(
            "prop_id cannot be negative: {}", prop_id
        )));
    }
    let col_idx = prop_id as usize;
    ...
}
```

Same fix should be applied to `get_property_by_property_by_id()` at line 221.

---

## 3. EdgeTable::insert_edge() Does Not Roll Back edge_id_counter on Failure

### Location

[src/storage/edge/edge_table.rs#L125-L170](file:///d:/%E9%A1%B9%E7%9B%AE/database/graphDB/src/storage/edge/edge_table.rs#L125-L170)

### Code

```rust
pub fn insert_edge_id = self.edge_id_counter.fetch_add(1, Ordering::Relaxed);

// ... later checks may return Err ...

if self.schema.oe_strategy == EdgeStrategy::Single
    && self.out_csr.has_edge(src, dst, ts) {
        self.properties.delete(prop_offset);
        return Err(StorageError::edge_already_exists(...));
    }

if !self.out_csr.insert_edge(src, dst, edge_id, prop_offset, ts) {
    self.properties.delete(prop_offset);
    return Err(StorageError::edge_already_exists(...));
}
```

### Problem

`edge_id_counter` is atomically incremented before validation checks. If any check fails and the function returns `Err`, the edge ID is **consumed but never used**. This creates gaps in the edge ID sequence.

### Impact

- Edge ID sequence will have holes under concurrent or error-prone workloads
- Edge IDs cannot be used as a reliable count of inserted edges
- Monitoring/reporting that depends on `edge_id_counter` will show inflated values

### Root Cause

Premature increment of the counter before validation.

### Fix Suggestion

Move `fetch_add` to just before the successful return, or accept the ID gap as intentional (document it). If ID gaps are acceptable, at minimum ensure the property data is cleaned up on all error paths (currently done, but counter still moves forward).

---

## 4. EdgeTable::compact_properties() Only Scans Out-CSR

### Location

[src/storage/edge/edge_table.rs#L900-L910](file:///d:/%E9%A1%B9%E7%9B%AE/database/graphDB/src/storage/edge/edge_table.rs#L900-L910)

### Code

```rust
pub fn compact_properties(&mut self, ts: Timestamp) {
    let mut valid_offsets = std::collections::HashSet::new();

    for (_, nbr) in self.out_csr.iter(ts) {
        if nbr.prop_offset > 0 {
            valid_offsets.insert(nbr.prop_offset);
        }
    }

    self.properties.compact(&valid_offsets);
}
```

### Problem

Only `out_csr` is scanned to build the set of valid property offsets. While out_csr and in_csr share the same prop_offset values (so out_csr theoretically covers all offsets), scanning both would be more robust against future changes where in_csr might have different offsets.

### Impact

- Currently no correctness issue due to shared prop_offset design
- Fragile if the design changes to allow independent in/out offsets
- Compact may remove properties that in_csr references (if they diverge)

### Fix Suggestion

```rust
for (src, nbr) in self.out_csr.iter(ts) {
    if nbr.prop_offset > 0 { valid_offsets.insert(nbr.prop_offset); }
}
if self.schema.ie_strategy != EdgeStrategy::None {
    for (dst, nbr) in self.in_csr.iter(ts) {
        if nbr.prop_offset > 0 { valid_offsets.insert(nbr.prop_offset); }
    }
}
```

---

## 5. VertexTable::insert() Does Not Validate Properties Against Schema

### Location

[src/storage/vertex/vertex_table.rs#L100](file:///d:/%E9%A1%B9%E7%9B%AE/database/graphDB/src/storage/vertex/vertex_table.rs#L100)

### Code

```rust
pub fn insert(
    &mut self,
    external_id: &str,
    properties: &[(String, Value)],
    ts: Timestamp,
) -> StorageResult<u32> {
    ...
    self.columns.set(internal_id as usize, properties)?;
    ...
}
```

### Problem

Properties are passed directly to `ColumnStore::set()` with no validation against `self.schema`. If a caller passes:
- Unknown property names → silently ignored by ColumnStore
- Wrong data types → ColumnStore may panic or store corrupted data
- Missing required (non-nullable) properties → ColumnStore stores None instead of erroring

### Impact

Silent data corruption or unexpected behavior when upper layers pass invalid property data.

### Root Cause

No defensive validation at the `VertexTable` API boundary.

### Fix Suggestion

```rust
for (name, value) in properties {
    let prop_def = self.schema.properties.iter()
        .find(|p| p.name == *name)
        .ok_or_else(|| StorageError::column_not_found(name.clone()))?;

    if value.data_type() != prop_def.data_type {
        return Err(StorageError::type_mismatch(
            prop_def.data_type.clone(),
            value.data_type(),
        ));
    }
}
```

Same validation should be added to `EdgeTable::insert_edge()` against `self.schema.properties`.

---

## 6. EdgeTable::update_edge_property_by_offset() Redundant Double Write

### Location

[src/storage/edge/edge_table.rs#L598-L615](file:///d:/%E9%A1%B9%E7%9B%AE/database/graphDB/src/storage/edge/edge_table.rs#L598-L615)

### Code

```rust
pub fn update_edge_property_by_offset(&mut self, params: ...) -> StorageResult<bool> {
    if let Some(nbr) = self.out_csr.get_edge(params.src, params.dst, params.ts) {
        self.properties
            .set_property_by_id(nbr.prop_offset, params.col_id, Some(params.value.clone()))?;

        if self.schema.ie_strategy != EdgeStrategy::None {
            if let Some(_ie_nbr) = self.in_csr.get_edge(params.dst, params.src, params.ts) {
                self.properties
                    .set_property_by_id(_ie_nbr.prop_offset, params.col_id, Some(params.value.clone()))?;
            }
        }
        return Ok(true);
    }
    Ok(false)
}
```

### Problem

Because both `out_csr` and `in_csr` entries in the same `prop_offset` (same row in PropertyTable), the update writes the same value **twice** to the same row. The second write is redundant.

### Impact

- No data corruption (same value overwritten)
- Double the expected write operations, double the work for PropertyTable lookup and assignment
 - Wasted CPU cycles on every edge property update

### Root Cause

The developer assumed out_csr and in_csr might have different prop_offsets, but in reality they share the same value.

### Fix Suggestion

Remove the redundant in_csr write entirely, or add a debug assertion:

```rust
debug_assert_eq!(nbr.prop_offset, _ie_nbr.prop_offset,
    "out_csr and in_csr should share the same prop_offset");
```

---

## 7. Various Cleanup Items

### 7.1 Unused Import in EdgeTable

[src/storage/edge/edge_table.rs#L14](file:///d:/%E9%A1%B9%E7%9B%AE/database/graphDB/src/storage/edge/edge_table.rs#L14)

```rust
use crate::core::DataType;  // Only used by add_property(), but import is always present
```

`DataType` is only used by `add_property()` method, not by core EdgeTable operations.

### 7.2 SpinLock in MutableCsr

[src/storage/edge/mutable_csr.rs](file:///d:/%E9%A1%B9%E7%9B%AE/database/graphDB/src/storage/edge/mutable_csr.rs)

`MutableCsr` implements per-vertex spin locks, but `EdgeTable` itself is not designed for concurrent access. The spin locks add complexity is unused in practice since edge operations require external synchronization (via `RwLock` in `EdgeOps`).

### 7.3 `Nbr` vs `ImmutableNbr` Duplication

[src/storage/edge/mod.rs](file:///d:/%E9%A1%B9%E7%9B%AE/database/graphDB/src/storage/edge/mod.rs#L119-L146)

`Nbr` and `ImmutableNbr` are nearly identical structs (differing only in timestamp field). Consider merging them with an optional timestamp or using a generic.

---

## 8. Summary

| # | Defect | File | Severity |
|---|--------|------|----------|
| 1 | `get_external_id()` no ts check | vertex_table.rs#L358 | **High** |
| 2 | Negative prop_id not validated | property_table.rs#L194 | Medium |
| 3 | edge_id_counter not rolled back on failure | edge_table.rs#L125 | Low |
| 4 | compact_properties only scans out_csr | edge_table.rs#L900 | Low |
| 5 | insert() no schema validation | vertex_table.rs#L100 | **High** |
| 6 | Redundant double write in update | edge_table.rs#L598 | Low |
| 7 | Various cleanup items | multiple | Low |

### Priority Fixes

1. **P0**: Defect #1 — add timestamp check to `get_external_id()` (data correctness)
2. **P1**: Defect #5 — add schema validation to `insert()` and `insert_edge()` (data quality)
3. **P2**: Defect #2 — add negative prop_id check (defensive coding)
4. **P3**: Defect #4 — scan both CSRs in compact (robustness)