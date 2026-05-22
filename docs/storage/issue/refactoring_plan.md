# Storage Module Refactoring Plan

## Priority Overview

| Priority | Issue | Effort | Risk | Dependencies |
|----------|-------|--------|------|-------------|
| **P0** | 1. Encapsulation | Medium | Medium | — |
| **P0** | 2. Interface bloat | Medium | Low | 1 |
| **P0** | 3. Delegate pattern | Low | Low | 1 |
| **P1** | 4. Transaction overlap | Large | High | 1 |
| **P1** | 5. Edge table key | Large | High | 1 |
| **P1** | 6. Schema paths | Low | Low | 1 |
| **P1** | 7. Mock re-export | Low | Low | — |
| **P1** | 8. Edge cache | Medium | Medium | 1 |
| **P2** | 9. Encoding naming | Low | Low | — |

Recommended order: 3 → 1 → 2 → 7 → 9 → 6 → 8 → 5 → 4

---

## Plan 1: Fix delegate pattern (P0, Low Effort)

**Problem**: Every GraphStorage trait method creates a temporary Ops object.

**Solution**: Convert `GraphStorageReader`, `GraphStorageWriter`, `PersistenceOps`,
`SchemaAdapterOps`, etc. from structs to free functions (or associated functions on
`GraphStorageContext`).

```rust
// Before
impl StorageReader for GraphStorage {
    fn get_vertex(&self, ...) -> ... {
        reader::GraphStorageReader::new(&self.ctx).get_vertex(...)
    }
}

// After — GraphStorageReader struct removed, functions moved to mod-level
impl StorageReader for GraphStorage {
    fn get_vertex(&self, space: &str, id: &VertexId) -> ... {
        reader::get_vertex(&self.ctx, space, id)
    }
}
```

**Files affected**: `engine/graph_storage/reader.rs`, `writer.rs`, `persistence.rs`,
`maintenance.rs`, `index_manager.rs`, `schema_adapter.rs`, `user_ops.rs`,
`type_utils.rs`, and `mod.rs`

**Steps**:
1. Replace `pub struct FooOps { ctx: &'a GraphStorageContext }` with `pub(crate) fn foo(...)`
2. Update all call sites in `mod.rs` trait impls
3. Delete the now-empty struct definitions
4. Run clippy + tests

---

## Plan 2: Encapsulate PropertyGraph & GraphDataStore (P0, Medium Effort)

**Problem**: Internal fields are `pub` allowing unrestricted access.

**Solution**: Make all fields `pub(crate)` on `PropertyGraph` private, add accessor
methods, and convert free functions in `core_ops`/`type_ops`/`flush`/`index_mvcc`
into `impl PropertyGraph` methods.

```rust
// Phase 1 — GraphDataStore: add accessors
impl GraphDataStore {
    pub(crate) fn vertex_table(&self, label: LabelId) -> ... { ... }
    pub(crate) fn vertex_table_mut(&self, label: LabelId) -> ... { ... }
    pub(crate) fn edge_table(&self, key: &EdgeTableKey) -> ... { ... }
    pub(crate) fn insert_vertex_table(&self, label: LabelId, table: VertexTable) { ... }
    pub(crate) fn remove_vertex_table(&self, label: LabelId) { ... }
    // ... etc
}

// Phase 2 — PropertyGraph: make fields private, promote free functions to methods
impl PropertyGraph {
    // Before: core_ops::insert_vertex(self, ...)
    // After:
    fn insert_vertex_impl(&self, ...) -> ... { ... }
}
```

**Files affected**:
- `engine/data_store.rs` — add accessors, make fields private
- `engine/property_graph/` — `core_ops.rs`, `type_ops.rs`, `flush.rs`, `index_mvcc.rs`
- `engine/property_graph/mod.rs` — make fields private

**Edge table key change**: Introduce a newtype for the triple to improve readability
and make future changes easier:

```rust
#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub(crate) struct EdgeTableKey {
    pub src_label: LabelId,
    pub dst_label: LabelId,
    pub edge_label: LabelId,
}
```

---

## Plan 3: Reduce GraphStorage public surface (P0, Medium Effort)

**Problem**: 30+ public methods not in any trait; trait is not the real contract.

**Solution**: 
1. Move maintenance/admin methods to `StorageAdmin` trait (extend it)
2. Move persistence/GC methods to `StorageAdmin` trait (extend it)
3. Add `WalRecoveryOps`, `PersistenceOps` traits (optional)
4. Or: accept the concrete type is sometimes needed and create a cleaner split

**Option A (Recommended) — Extend StorageAdmin**:

```rust
// Extend StorageAdmin with missing methods
pub trait StorageAdmin: Send + Sync + std::fmt::Debug {
    // existing...
    fn load_from_disk(&mut self) -> Result<(), StorageError>;
    fn save_to_disk(&self) -> Result<(), StorageError>;
    
    // additions:
    fn flush(&self) -> StorageResult<()>;
    fn create_checkpoint(&self) -> StorageResult<Option<CheckpointStats>>;
    fn compact_all(&self, ts: Timestamp) -> StorageResult<()>;
    fn recover_from_wal(&self) -> StorageResult<RecoveryStats>;
    fn needs_recovery(&self) -> bool;
    fn is_index_gc_running(&self) -> bool;
    fn get_storage_stats(&self) -> StorageStats;
    fn auto_flush_if_needed(&self) -> StorageResult<bool>;
    fn auto_checkpoint_if_needed(&self) -> StorageResult<Option<CheckpointStats>>;
}
```

This makes the trait the real contract. Downstream code using `Box<dyn StorageAdmin>`
or `Box<dyn StorageClient>` can call all important methods.

**Option B — Create new trait(s)**:

```rust
pub trait StoragePersistence: Send + Sync + std::fmt::Debug {
    fn flush(&self) -> StorageResult<()>;
    fn create_checkpoint(&self) -> StorageResult<Option<CheckpointStats>>;
    fn save_to_disk(&self) -> StorageResult<()>;
    fn load_from_disk(&mut self) -> StorageResult<()>;
    fn compact_all(&self, ts: Timestamp) -> StorageResult<()>;
}

pub trait StorageRecovery: Send + Sync + std::fmt::Debug {
    fn needs_recovery(&self) -> bool;
    fn recover_from_wal(&self) -> StorageResult<RecoveryStats>;
    fn init_with_recovery(&self) -> StorageResult<Option<RecoveryStats>>;
}
```

**Steps**:
1. Audit all `GraphStorage` public methods
2. Categorize: persistence, maintenance, GC, other
3. Add to existing traits or create new traits
4. Update `GraphStorage` trait impls
5. Update downstream callers (mainly `api/` and `tests/`)
6. Remove redundant methods (or keep them as aliases)

---

## Plan 4: Fix MockStorage re-export (P1, Low Effort)

**Problem**: Wildcard `pub use test_mock::*` floods namespace.

**Solution**: Explicit re-exports.

```rust
// Before
#[cfg(test)]
pub use test_mock::*;

// After
#[cfg(test)]
pub use test_mock::MockStorage;
```

**Files affected**: `mod.rs`

---

## Plan 5: Rename utils/encoding.rs (P2, Low Effort)

**Problem**: Two "encoding" modules with different purposes.

**Solution**: Rename `utils/encoding.rs` to `utils/persistence_format.rs` (or similar).

```rust
// utils/mod.rs
pub mod persistence_format;  // was: encoding

// Also rename the file
// utils/encoding.rs → utils/persistence_format.rs
```

**Files affected**: `utils/mod.rs`, `utils/encoding.rs`, all files that `use` it.

---

## Plan 6: Consolidate SchemaManager access paths (P1, Low Effort)

**Problem**: SchemaManager accessible from 3+ paths.

**Solution**: Choose one canonical path and deprecate others.

**Recommended canonical path**: `GraphStorage::get_schema_manager() -> Arc<SchemaManager>`

- Remove `StorageAdmin::get_schema_manager()` from trait (or keep as default that returns `None`)
- Remove `SchemaAdapterOps`'s independent reference; make it use `ctx.schema_manager` only
- Ensure only `GraphStorage.get_schema_manager()` is the public API

---

## Plan 7: Edge table key redesign (P1, Large Effort, High Impact)

**Problem**: `(src_label, dst_label, edge_label)` triple causes fragmentation.

**Option A — Single key by edge_label**:

```rust
edge_tables: RwLock<HashMap<LabelId, EdgeTable>>,
// Each EdgeTable stores all edges of that type regardless of src/dst label
```

Requires adding `src_label` / `dst_label` columns to the CSR or a separate index.

**Option B — Two-level map**:

```rust
edge_tables: RwLock<HashMap<LabelId, HashMap<(LabelId, LabelId), EdgeTable>>>,
// Level 1: edge_label → Level 2: (src_label, dst_label) → EdgeTable
```

Makes `scan_edges_by_label()` O(1) but keeps label-pair partitioning for physical storage.

**Option C — Current but with newtype**:

```rust
#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub struct EdgeTableKey {
    pub edge_type: LabelId,
    pub src_label: LabelId,
    pub dst_label: LabelId,
}
edge_tables: RwLock<HashMap<EdgeTableKey, EdgeTable>>,
```

Lowest risk but doesn't fix fragmentation.

**Recommendation**: Start with Option C (newtype, no behavioral change), then evaluate
if Option A is needed based on real-world data patterns.

---

## Plan 8: Consolidate transaction subsystems (P1, Large Effort, High Risk)

**Problem**: `TransactionOps` trait + `TransactionalWriter` + `transaction_support`
have overlapping responsibilities.

**Solution**: Merge into a unified transaction abstraction.

1. `TransactionalWriter` should be the single point for WAL-batched writes
2. `TransactionOps` should use `TransactionalWriter` internally
3. Remove the duplicate code paths

```rust
// Unified approach: TransactionalWriter becomes the core
pub(crate) struct TransactionalWriter<'a> {
    graph: &'a PropertyGraph,
    wal: &'a WalManager,
    index_updater: &'a IndexUpdater,
}

impl TransactionalWriter<'_> {
    pub fn insert_vertex(&self, ...) -> StorageResult<...>;
    pub fn insert_edge(&self, ...) -> StorageResult<...>;
    pub fn delete_vertex(&self, ...) -> StorageResult<...>;
    pub fn delete_edge(&self, ...) -> StorageResult<...>;
    // With WAL + index maintenance built-in
}
```

Then `TransactionOps` trait impls delegate to `TransactionalWriter`.

---

## Plan 9: Add optional edge cache (P1, Medium Effort)

**Problem**: Repeated hot edge reads always hit PropertyTable decompression.

**Solution**: Add optional `EdgeRecordCache` to `RecordCache`.

```rust
// cache/types.rs
pub enum CacheType {
    Vertex,
    IdIndex,
    Edge,  // new
}

// cache/record_cache.rs — add
pub struct RecordCache {
    vertex_cache: moka::sync::Cache<VertexCacheKey, CachedVertex>,
    id_index_cache: moka::sync::Cache<String, InternalVertexId>,
    edge_cache: moka::sync::Cache<EdgeCacheKey, CachedEdgeRecord>,  // new
}
```

Edge cache key: `(EdgeCacheKey { edge_label, src_id, dst_id, rank })`
Only enabled when `cache_memory` config allows it. Should be optional
(behind a config flag) to avoid memory overhead for workloads without hot edges.

---

## Summary

| Order | Plan | Before | After | Key Metric |
|-------|------|--------|-------|------------|
| 1 | 3. Delegate → free functions | 7 structs, 100+ `::new()` calls | Free functions | Lines of code -15% |
| 2 | 1. Encapsulate internals | 8 pub fields in PG, 6 in GDS | All private + accessors | Rustc visibility checks |
| 3 | 2. Reduce public surface | 30+ untrait methods | Added to traits | trait == real contract |
| 4 | 4. Fix mock re-export | Wildcard `pub use test_mock::*` | Explicit re-exports | Cleaner module interface |
| 5 | 5. Rename encoding | Confusing dual encoding | `persistence_format` + `compression` | Developer comprehension |
| 6 | 6. Schema paths | 3+ access paths | 1 canonical path | Maintenance cost |
| 7 | 8. Edge cache | 0 cache for edges | Optional config-driven cache | Hot read QPS |
| 8 | 5. Edge table key | Raw triple tuple | Newtype or simplified key | Query efficiency |
| 9 | 4. Transaction merge | Two overlapping subsystems | Unified `TransactionalWriter` | Code clarity |
