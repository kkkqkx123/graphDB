# Lock Usage Analysis

Analyzed: 2026-05-14

## Overview

This document analyzes the usage of `Mutex` and `RwLock` across the project. Three families of locking primitives are used:

| Lock Family | Usage Scope | Key Characteristic |
|---|---|---|
| `parking_lot::Mutex` / `RwLock` | `src/` core code (dominant) | No poisoning, lightweight, can't be used across `.await` |
| `tokio::sync::Mutex` / `RwLock` | `crates/inversearch/`, `src/sync/` | Async-aware, used with `.await`, supports `blocking_*` |
| `std::sync::Mutex` / `RwLock` | `crates/`, `graphdb-cli/`, WAL module | Standard library, has poisoning |

---

## Good Practices Observed

### No sync locks held across `.await`
No `parking_lot` or `std::sync` lock is held across an `.await` point. All such locks are released before entering async contexts.

### Consistent lock ordering
- `src/query/cache/invalidation.rs` — `DependencyTracker` always acquires `table_to_keys` then `key_to_tables` in that order
- `src/storage/cache/edge_property_cache.rs` — always acquires `cache` then `access_tracker` in that order
- No circular lock dependencies were identified

### Condvar usage is correct
`src/transaction/version_manager.rs` uses `Mutex` + `Condvar` correctly: the lock is only held while checking/waiting on the condition and released before actual work.

### `try_lock` in `Drop`
`src/sync/batch/processor.rs:130` correctly uses `try_lock()` in `Drop` implemention (destructors cannot be async).

---

## Issues Found

### P1: Read operations using write lock in `auth/user_storage.rs`

**File**: `src/api/server/auth/user_storage.rs`
**Lines**: 108, 113

```rust
// ❌ Write lock acquired for a read-only operation
self.users.write().get(username)
```

Using `.write()` for a read-only lookup blocks all concurrent readers unnecessarily. This is a pessimistic locking mistake.

**Fix**: Replace with `.read()`:
```rust
self.users.read().get(username)
```

---

### P1: `PropertyGraph` single RwLock is global throughput bottleneck

**File**: `src/storage/engine/graph_storage/context.rs`
**Line**: 27

```rust
pub graph: Arc<RwLock<PropertyGraph>>
```

All DML operations (insert, update, delete) require `graph.write()`, which blocks all concurrent readers. As the single most contended lock in the system, this is the primary throughput bottleneck for concurrent workloads.

**Possible mitigations**:
- Split `PropertyGraph` into finer-grained locks (per-tag, per-edge-type)
- Introduce分段锁 (sharded locking)
- Use `dashmap` for certain sub-structures
- Adopt an MVCC approach to allow reads during writes

---

### P2: Silent poisoning handling in `group_commit.rs`

**File**: `src/transaction/wal/writer/group_commit.rs`
**Lines**: 51, 57, 103, 124

```rust
// ❌ Poison silently ignored, operation returns None
self.pending_writes.lock().ok()?
```

When the `std::sync::Mutex` is poisoned, `.ok()` converts the error to `None`, and the `?` causes early return. This means a poisoned lock silently drops batch writes with no error reporting.

**Fix**: Either use `parking_lot::Mutex` (no poisoning) or log/propagate the error instead of silently returning `None`.

---

### P3: LRU cache forces write lock for reads

**File**: `crates/inversearch/src/search/cache.rs`
**Lines**: 78-96

```rust
// ❌ Cache lookup requires write lock to update LRU order
self.store.write()
```

The `tokio::sync::RwLock` protecting the LRU cache must be acquired as `write()` even for cache hits, because LRU needs to update access order. This is a known trade-off with LRU + RwLock.

**Alternatives**:
- Use `crossbeam` epoch-based reclamation
- Implement a lock-free LRU (e.g., `deadqueue` or custom sharded approach)
- Accept the trade-off and document it

---

### P3: `try_lock` silent fallback in non-critical paths

Multiple locations use `try_lock()` and silently skip the operation if contention is detected:

| File | Line | Impact |
|---|---|---|
| `src/sync/batch/processor.rs` | 130 | Background task not aborted on drop |
| `crates/inversearch/src/search/cache.rs` | 126,150,162,171 | Cache maintenance skipped |
| `crates/inversearch/src/compress/cache.rs` | 24,29 | Compression cache missed |

These are acceptable as best-effort fallbacks, but adding a `tracing::warn!()` on fallback would help debug contention issues.

---

## Lock Statistics

### By module (approximate distinct lock instances)

| Module | Count | Primary Type |
|---|---|---|
| `src/storage/` | ~10 | `parking_lot::RwLock` |
| `src/transaction/` | ~8 | `parking_lot::RwLock` / `std::sync::Mutex` |
| `src/query/` | ~50+ | `parking_lot::RwLock` (primarily `storage: Arc<RwLock<S>>` pattern) |
| `src/api/` | ~20 | `parking_lot::RwLock` |
| `src/core/` | ~5 | `parking_lot::RwLock` / `Mutex` |
| `src/sync/` | ~3 | `tokio::sync::Mutex` |
| `crates/inversearch/` | ~15 | `tokio::sync::RwLock` / `parking_lot::RwLock` |
| `crates/bm25/` | ~3 | `tokio::sync::RwLock` |
| `graphdb-cli/` | ~5 | `std::sync::Mutex` |
| **Total** | **~120+** | |

### Read vs Write lock patterns in executors

Most data-access executors (SELECT/LOOKUP/GO etc.) use `storage.read()`, while data-modification executors (INSERT/UPDATE/DELETE) use `storage.write()`. This is a correct pattern.

---

## Recommendations by Priority

| Priority | Action | Benefit |
|---|---|---|
| **P1** | Fix `auth/user_storage.rs` — `.write()` → `.read()` for read ops | Immediate concurrency improvement |
| **P1** | Architect `PropertyGraph` to reduce single RwLock contention | Major throughput improvement for concurrent workloads |
| **P2** | Fix `group_commit.rs` poison handling | Reliability |
| **P3** | Add `tracing::warn!()` on `try_lock` fallbacks | Debuggability |
| **P3** | Document LRU cache write-lock trade-off in `inversearch` | Maintainability |
