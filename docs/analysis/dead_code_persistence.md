# Dead Code Analysis: `src/storage/engine/graph_storage/persistence.rs`

Two `pub(crate)` functions in `persistence.rs` are dead code (no callers).
Both represent **partially wired features** — the implementation exists but was
never connected to the startup/compaction flow.

---

## 1. `compact_transactional` — Replace `compact_all`

**Status**: `#[allow(dead_code)]` — planned to replace `compact_all()`.

### Comparison

| Aspect | `compact_all` | `compact_transactional` |
|---|---|---|
| Timestamp source | Caller-provided (`ts`) | Self-acquired from `VersionManager` (guarantees exclusivity) |
| WAL logging | None | Writes a WAL header entry (crash marker) |
| Vertex compaction | Yes | Yes (via `CompactTarget`) |
| Edge CSR compaction | No | Yes (configurable via `compact_csr`) |
| Edge property compaction | No | Yes |
| Cache clearing | No | `cache_manager.clear_cache()` |
| Concurrent-write safety | None (caller responsible) | VM lock ensures exclusivity |
| Rollback | Not supported | `abort()` + `revert_update_timestamp()` |

### Changes Required

| Step | File(s) | Detail |
|---|---|---|
| Add `WalOpType::Compact` variant | `src/core/wal/types.rs` | Currently reuses `DeleteVertex` which is semantically wrong |
| Expose `WalWriter` from `PersistenceCoordinator` | `persistence_coordinator.rs` | `compact_transactional` requires `&mut dyn WalWriter` |
| Change `StorageAdmin::compact_all(ts)` to `compact(compact_csr, reserve_ratio)` | `storage_client.rs` + `mod.rs` | Backward compat not required per project convention |
| Replace body: call `compact_transactional` | `mod.rs` | Pass the WAL writer from context |
| Remove old `compact_all` | `persistence.rs` | After verifying no remaining callers |
| Add no-op `replay_compact` to `RecoveryApplier` | `core/wal/traits.rs` | So WAL recovery doesn't choke on the new op type |

---

## 2. `load_latest_checkpoint` — Wire up Checkpoint-based Recovery

**Status**: `#[allow(dead_code)]` — bridge function awaiting checkpoint recovery integration.

### Current Recovery Flow (Broken)

```
startup → needs_recovery()? → recover_from_wal()
                                 ├── RecoveryManager::restore_from_checkpoint() ← NO-OP
                                 └── replay_wal_entries() on empty graph
```

Checkpoints are **created** by `create_checkpoint()` (called from
`auto_checkpoint_if_needed()`) but **never loaded** during recovery.
`PersistenceCoordinator::recover()` (checkpoint + WAL replay) is defined
but has **zero callers**.

### Intended Recovery Flow

```
startup → needs_recovery()? → init_with_recovery()
                                ├── load_latest_checkpoint()
                                │     └── PersistenceCoordinator::recover()
                                │           ├── find latest checkpoint dir
                                │           ├── restore_from_checkpoint()
                                │           │     └── PropertyGraph::restore_from_checkpoint()
                                │           ├── truncate WAL to checkpoint LSN
                                │           └── return CheckpointInfo
                                └── replay_wal_entries() (only entries after checkpoint LSN)
```

### Changes Required

| Step | File(s) | Detail |
|---|---|---|
| Fix `RecoveryManager::restore_from_checkpoint` | `recovery.rs:138` | Currently a no-op; must actually load checkpoint data |
| Include schema + index_meta in checkpoint data | `persistence_coordinator.rs` or `mod.rs` | Currently only graph data is checkpointed |
| Wire up `init_with_recovery()` as actual startup entry point | `mod.rs` + `storage_client.rs` | Currently never called |
| Ensure `load_from_disk()` is not needed as separate step | `mod.rs` | After checkpoint recovery covers it |

### Design Notes

- `load_latest_checkpoint()` in `persistence.rs` is the correct bridge — it
  takes `GraphStorageContext`, extracts the `PersistenceCoordinator`, and
  calls `graph.restore_from_checkpoint()`.
- The function signature matches what `init_with_recovery()` would need.
- The data format loaded by `restore_from_checkpoint` is identical to
  `load_from_disk` (both iterate `vertices/` and `edges/` subdirs with the
  same naming conventions).
- Schema and index metadata live outside the checkpoint boundary today;
  either include them in checkpoints or always load them separately.
