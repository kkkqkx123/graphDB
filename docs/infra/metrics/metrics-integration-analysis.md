# Metrics Integration Analysis (2026-05-16)

## Overview

Analysis of how the metrics system (`src/core/stats/`) integrates with each module, and which modules are missing required implementations.

## Metrics Infrastructure

All located in `src/core/stats/`:

| File | Purpose |
|------|---------|
| `manager.rs` | `StatsManager` (central hub), `MetricType` enum (31 variants), `MetricValue` (atomic counter), 3-tier granularity (global/space/index) |
| `metrics.rs` | `QueryMetrics` — lightweight query timing for client response |
| `profile.rs` | `QueryProfile`, `StageMetrics`, `ExecutorStat` — detailed query profiling |
| `latency_histogram.rs` | `LatencyHistogram` — P50/P95/P99 percentile calculations |
| `error_stats.rs` | `ErrorStatsManager`, `ErrorType`, `QueryPhase` — error tracking |
| `aggregated_stats.rs` | `AggregatedStatsManager` — query pattern aggregation |
| `slow_query_logger.rs` | `SlowQueryLogger`, `SlowQueryConfig` — slow query logging |
| `utils.rs` | Duration formatting, cache hit rate, etc. |

### Record Methods on StatsManager

**Generic counters (3 tiers):**
- `add_value(MetricType)` / `add_value_with_amount(MetricType, u64)` / `dec_value(MetricType)` — global
- `add_space_metric(&str, MetricType)` / `dec_space_metric(&str, MetricType)` — space-level
- `add_index_metric(&str, MetricType)` / `dec_index_metric(&str, MetricType)` — index-level

**High-level recorders:**
- `record_query_metrics(&QueryMetrics)` — records query timing breakdown + latency histogram
- `record_query_profile(QueryProfile)` — full profile with slow query detection
- `record_aggregated_query(profile, is_slow)` — pattern aggregation
- `record_search(space_id, index_name, latency_ms, success)` — 3-tier search op
- `record_index_operation(space_id, index_name, latency_ms, success)` — 3-tier index op
- `record_delete_operation(space_id, index_name, latency_ms, success)` — 3-tier delete op
- `record_search_result_count(space_id, count)` — result count tracking
- `record_cache_hit(space_id, bool)` — cache hit/miss tracking
- `record_search_error(...)` / `record_index_error(...)` / `record_delete_error(...)` — classified errors
- `record_error(ErrorType, QueryPhase)` — generic query error
- `record_failed_query(profile, error_info)` — failed query profile

## Module Integration Status

### Modules WITH Metrics Integration

| Module | Level | Key Files | What Is Measured |
|--------|-------|-----------|------------------|
| `search/` | Full | `search/metrics.rs`, `search/index_cache.rs`, `search/manager.rs` | Search/index/delete ops latency + success/failure, cache hit/miss, classified errors via `MetricsSearchEngine` decorator pattern |
| `query/` | Full | `query/query_pipeline_manager.rs` | Query timing per phase (parse, validate, plan, optimize, execute, total), plan node count, result row count, profiles, slow query logging |
| `api/` | Partial | `api/server/graph_service.rs`, `api/server/http/handlers/statistics.rs` | Auth failures, active query decrement; statistics API reads all metrics (read-only) |

### Modules WITHOUT Metrics Integration

| Module | Directory | Uses StatsManager? | Notes |
|--------|-----------|--------------------|-------|
| `storage/` | `src/storage/` (17 subdirs) | No | No metrics of any kind |
| `transaction/` | `src/transaction/` (17 files) | No | Has independent `TransactionMonitor` not connected to `StatsManager` |
| `sync/` | `src/sync/` (9 entries) | No | No metrics of any kind |
| `storage/index/` | `src/storage/index/` (under storage) | No | No metrics of any kind |
| `utils/` | `src/utils/` | No | Utility helpers, low priority |
| `common/` | `src/common/` | No | Shared types, low priority |
| `config/` | `src/config/mod.rs` | Has `MonitoringConfig` struct | Config struct exists but never wired to anything |

### Crate-Level (outside `src/`)

| Crate | Status |
|-------|--------|
| `crates/bm25` | Docs reference `StorageMetrics` / `MetricsCollector` / `OperationTimer` — **do not exist in current code** |
| `crates/inversearch` | Same as above — referenced in docs but not implemented |

## Critical Issues

### 1. Multiple StatsManager Instances

There are **three separate `StatsManager` instances**:

1. **Server StatsManager** — created in `graph_service.rs:141-143`
   - Records: auth failures, active query decrement
   - Read by: statistics API handlers (via `state.server.get_stats_manager()`)
2. **Pipeline StatsManager** — created inside each `QueryApi` at `query_api.rs:29,42,59,77`
   - Records: query metrics and profiles
   - **NOT the same instance as server StatsManager**
3. **HttpServer StatsManager** — created in `server.rs:44` for admin HTTP operations

**Consequence:** Query metrics (`NumQueries`, `NumMatchQueries`, etc.) recorded by `QueryPipelineManager` go to instance #2, but the statistics API reads from instance #1. **All query-type metrics displayed in the statistics API will always be 0.**

### 2. Defined But Never Recorded Metrics

These `MetricType` variants are defined and exposed in the statistics API but **never incremented** in production code (only in tests):

| MetricType | Defined at | API Endpoint | Status |
|------------|------------|--------------|--------|
| `NumQueries` | `manager.rs:27` | `/statistics/queries`, `/statistics/database` | Never recorded |
| `NumActiveQueries` | `manager.rs:28` | `/statistics/database` | Only decremented (lines 415, 478), never incremented |
| `NumMatchQueries` | `manager.rs:38` | `/statistics/queries` | Never recorded |
| `NumCreateQueries` | `manager.rs:39` | `/statistics/queries` | Never recorded |
| `NumUpdateQueries` | `manager.rs:40` | `/statistics/queries` | Never recorded |
| `NumDeleteQueries` | `manager.rs:41` | `/statistics/queries` | Never recorded |
| `NumInsertQueries` | `manager.rs:42` | `/statistics/queries` | Never recorded |
| `NumGoQueries` | `manager.rs:43` | `/statistics/queries` | Never recorded |
| `NumFetchQueries` | `manager.rs:44` | `/statistics/queries` | Never recorded |
| `NumLookupQueries` | `manager.rs:45` | `/statistics/queries` | Never recorded |
| `NumShowQueries` | `manager.rs:46` | `/statistics/queries` | Never recorded |

## Modules Requiring Implementation

### Storage (`src/storage/`)

Suggested new `MetricType` variants:
- `StorageReadOps` — cumulative read operation count
- `StorageWriteOps` — cumulative write operation count
- `StorageReadLatencyUs` — total read latency in microseconds
- `StorageWriteLatencyUs` — total write latency in microseconds
- `StorageErrors` — storage error count
- `StorageCacheHitRate` — page/block cache hit rate

Implementation approach: Create a `MetricsStorage` decorator (similar to `MetricsSearchEngine` in `search/metrics.rs`) wrapping the storage engine.

### Transaction (`src/transaction/`)

Suggested new `MetricType` variants:
- `TxnBeginCount` — transaction begin count
- `TxnCommitCount` — transaction commit count
- `TxnRollbackCount` — transaction rollback count
- `TxnActiveCount` — currently active transactions
- `TxnConflictCount` — transaction conflict/retry count

Implementation approach: Either replace `TransactionMonitor` with `StatsManager`, or have `TransactionMonitor` hold a reference to `StatsManager` and forward metrics.

### Sync (`src/sync/`)

Suggested new `MetricType` variants:
- `SyncOperations` — sync operation count
- `SyncLatencyMs` — sync operation latency
- `SyncErrors` — sync error count
- `SyncQueueDepth` — current sync queue depth

### Index (`src/storage/index/`)

Suggested new `MetricType` variants:
- `IndexScanCount` — index scan count
- `IndexLookupLatencyUs` — index lookup latency
- `IndexMemoryUsage` — index memory consumption
- `IndexWriteOps` — index write operation count

## Priority Recommendations

| Priority | Item | Effort | Impact |
|----------|------|--------|--------|
| **P0** | Fix `StatsManager` singleton — share one instance across server, pipeline, HTTP handlers | Small | High — all query metrics broken |
| **P0** | Record `NumQueries` / `NumActiveQueries` increment in `query_pipeline_manager.rs` | Small | High — basic query counting missing |
| **P0** | Record query type counters (`NumMatchQueries`, `NumGoQueries`, etc.) in appropriate pipeline handlers | Medium | High — per-type query visibility |
| **P1** | Add `storage/` metrics via decorator pattern | Medium | High — storage is performance bottleneck |
| **P1** | Add `transaction/` metrics integration | Medium | Medium — transaction throughput visibility |
| **P2** | Add `sync/` metrics | Medium | Low — sync module maturity unknown |
| **P2** | Add `storage/index/` metrics | Medium | Low — index metrics partially covered by search module |
| **P3** | Implement `StorageMetrics` / `MetricsCollector` in `crates/bm25` and `crates/inversearch` | Large | Medium — per design docs |

## Key File References

| File | Lines | Content |
|------|-------|---------|
| `src/core/stats/manager.rs` | 25-66 | `MetricType` enum (31 variants) |
| `src/core/stats/manager.rs` | 136-148 | `StatsManager` struct |
| `src/core/stats/manager.rs` | 378-442 | Generic counter methods (3 tiers) |
| `src/core/stats/manager.rs` | 551-586 | `record_query_metrics` |
| `src/core/stats/manager.rs` | 708-834 | Search/index/delete recorders |
| `src/search/metrics.rs` | 59-193 | `MetricsSearchEngine` decorator |
| `src/search/manager.rs` | 262-284 | `set_stats_manager` injection |
| `src/search/index_cache.rs` | 37-43 | Cache hit/miss recording |
| `src/query/query_pipeline_manager.rs` | 325-484 | Query metrics + profile recording |
| `src/api/server/graph_service.rs` | 138-143 | Server StatsManager creation |
| `src/api/server/graph_service.rs` | 164-196 | Auth failure recording |
| `src/api/core/query_api.rs` | 29, 42, 59, 77 | Pipeline StatsManager creation (separate instances) |
| `src/api/server/http/handlers/statistics.rs` | 62-394 | Statistics API handlers (read-only) |
| `src/transaction/monitor.rs` | 1-151 | Independent TransactionMonitor |
