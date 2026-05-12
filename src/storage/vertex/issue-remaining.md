# Remaining Issues and Modification Directions

Status as of: 2026-05-12

---

## Legend

| Icon | Meaning |
|------|---------|
| ✅ Resolved | Issue has been addressed |
| ⚠️ Partially Resolved | Mitigation in place, but not fully solved |
| ❌ Unresolved | No fix implemented |

---

## Issue Status Summary

| # | Issue | Status | Current State |
|---|-------|--------|---------------|
| 1 | ID type (external vs internal) | ✅ Resolved | Index stores external ID (`Value`). Native methods exist but unused in main path |
| 2 | Compact + index sync | ⚠️ Partially | `compact_all` in `GraphStorage` coordinates GC, but `CompactTransaction` path is not wired |
| 3 | Index update atomicity | ⚠️ Partially | `insert_vertex` rollback fixed (tracks all inserted tags). `IndexUpdateContext`/UndoLog tier exists but unused in `GraphStorage` DML |
| 4 | WAL/MVCC timestamp sync | ✅ Resolved | `VersionManager` provides unified `AtomicU32` counter |
| 5 | Checkpoint cross-component consistency | ❌ Unresolved | Vertex/Edge/Index persisted separately, no global atomic snapshot |
| 6 | Multi-lock deadlock risk | ❌ Unresolved | `graph.write()` + `index_data_manager.read()` potential lock inversion |
| 7 | IndexScanIterator | ✅ Resolved | Implemented in `index_scan_iter.rs` |
| 8 | Integration test coverage | ✅ Resolved | `test_vertex_table_and_index_integration` added |

---

## Remaining Issues

### 1. Compact does not clean index tombstones via CompactTransaction

**Problem**: `PropertyGraph` (as `CompactTarget`) does not own the index manager. When `CompactTransaction` is used, index GC is not triggered.

**Current mitigation**: `GraphStorage::compact_all()` coordinates both phases, but `CompactTransaction` is bypassed.

**Directions**:

- **Option A: Extend CompactTarget trait** — Add an optional `gc_index` callback to `CompactTarget`, implemented by `PropertyGraph` by forwarding to a registered listener.

    ```rust
    pub trait CompactTarget: Send + Sync {
        fn compact(&mut self, ...) -> CompactTransactionResult<()>;
        fn gc_index(&self, safe_ts: Timestamp) -> CompactTransactionResult<()> {
            Ok(())  // default no-op
        }
    }
    ```

    Then `CompactTransaction::commit()` calls `self.graph.gc_index(self.timestamp)?;`.

- **Option B: Move index GC into PropertyGraph** — Add an `Arc<dyn Fn(Timestamp)>` callback field to `PropertyGraph`, set by `GraphStorage` at initialization. `CompactTarget::compact()` invokes the callback after compacting vertex tables.

- **Option C: Expose `IndexGcManager` at the service layer** — Schedule periodic GC on a timer/cron independent of compact. Already partially exists as `IndexGcManager` infrastructure.

---

### 2. Main DML path lacks atomicity for index updates

**Problem**: `GraphStorage::insert_vertex`, `update_vertex`, `delete_vertex` update vertex data then update indexes, but use manual rollback (or none) instead of the existing `IndexUpdateContext`/`IndexUndoLog`.

**Directions**:

- **Option A: Integrate IndexUpdateContext into GraphStorage DML methods** — Replace direct `update_vertex_indexes`/`delete_vertex_indexes` calls with `IndexUpdateContext::commit()` that provides proper undo logging.

    ```rust
    fn insert_vertex(&mut self, ...) -> Result<Value, StorageError> {
        let mut ctx = IndexUpdateContext::new(&index_data_manager, ...);
        // ... update vertex table ...
        ctx.add_vertex_update(vertex_id, tags);
        ctx.commit()?;  // atomic: records undo entries first, then applies
        // On error, ctx.rollback() is called via Drop
    }
    ```

- **Option B: Use transactional wrappers** — Add a lightweight `with_atomic` helper that captures a snapshot of index state before the operation and restores on failure.

- **Option C: Status quo** — The multi-tag rollback fix already prevents data corruption. Accept the rare edge case of zombie entries cleaned by next compact. Low priority if `IndexGcManager` is enabled.

---

### 3. Multi-lock deadlock risk

**Problem**: `GraphStorage` methods hold `graph.write()` while acquiring `index_data_manager.read()` (via `update_vertex_indexes`). If another thread acquires the locks in reverse order, deadlock occurs.

---

**Direction**: **Eliminate the second lock by merging index access into PropertyGraph.**

Move `InMemoryIndexDataManager` (or a reference to it) into `PropertyGraph`, so index operations are protected under the same `graph.write()` lock. This eliminates the multi-lock scenario entirely.

```rust
pub struct GraphStorage {
    graph: Arc<RwLock<PropertyGraph>>,  // now owns index_data_manager internally
    // index_data_manager: Arc<RwLock<InMemoryIndexDataManager>>,  // REMOVED
    ...
}
```

`PropertyGraph` gains methods like:
```rust
impl PropertyGraph {
    pub fn insert_vertex_and_index(&mut self, ...) -> StorageResult<()> {
        self.insert_vertex(...)?;
        self.index_data_manager.update_vertex_indexes(...)?;
        Ok(())
    }
}
```

**Trade-off**: Coarser lock granularity, reduced concurrent read throughput on index. Acceptable for single-node deployment.

---

### 4. Checkpoint cross-component atomicity

**Problem**: Vertex tables, edge tables, WAL, and index data are flushed independently. A crash between flushes produces an inconsistent state.

---

**Directions**:

- **Option A: Global manifest + two-phase flush** — Maintain a `MANIFEST` file that records the LSN and component states. Flush order: (1) flush all components to temporary files, (2) atomically update MANIFEST, (3) rename temp files to final.

    ```
    data/
    ├── MANIFEST              # atomically rewritten on each checkpoint
    ├── checkpoint_42/
    │   ├── vertices/
    │   ├── edges/
    │   └── index/
    └── ...
    ```

- **Option B: Copy-on-write (CoW) checkpoint** — Each checkpoint creates a full new directory (`checkpoint_<id>/`). Only after all components are written does the root pointer update atomically. `PersistenceCoordinator` already uses the `checkpoint_<id>` directory pattern — complete the support for atomic root pointer swap.

- **Option C: WAL-only durability with in-memory state** — Accept that in-memory state is the primary store. The WAL is the source of truth for recovery; checkpoints are optimization hints. Add WAL replay verification on startup.

---

### 5. Predicate pushdown not utilizing index scans

**Problem**: `VertexFilterIterator` performs full table scan + predicate evaluation. `IndexScanIterator` exists but is not wired into the query engine's scan path.

---

**Direction**: **Integrate IndexScanIterator into the query plan selection.**

Add a scan strategy layer that chooses between:
- `IndexScanIterator` (when predicate matches an indexed property)
- `VertexFilterIterator` (fallback full scan)

```rust
pub enum ScanStrategy<'a> {
    Index(IndexScanIterator<'a>),
    FullScan(VertexFilterIterator<'a>),
}
```

Connect this to the query planner in the `query` module.

---

### 6. Cross-label scan lacks optimization

**Problem**: `VertexScanIterator` iterates all labels. No index-based routing.

---

**Directions**:

- **Option A**: When a predicate references a specific tag, only scan the relevant `VertexTable` for that label.
- **Option B**: Maintain a global vertex ID index across all labels for O(1) lookup-by-ID without label iteration.

Low priority for single-node use cases.

---

## Summary

| Priority | Issue | Effort | Direction |
|----------|-------|--------|-----------|
| High | Deadlock risk (lock inversion) | 2-3 days | Merge index into PropertyGraph |
| High | Checkpoint atomicity | 3-5 days | Global MANIFEST + two-phase flush |
| Medium | DML atomicity (IndexUpdateContext) | 2-4 days | Integrate context into GraphStorage DML |
| Medium | Compact + index GC wiring | 1-2 days | Extend CompactTarget trait |
| Low | Predicate pushdown | 3-5 days | ScanStrategy enum + query planner integration |
| Low | Cross-label scan | 2-3 days | Tag-aware label routing |
