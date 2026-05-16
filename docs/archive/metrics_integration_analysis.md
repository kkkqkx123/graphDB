# Metrics Integration Analysis

## Overview

Analysis of how each module in the `src` directory integrates with the `core::stats` metrics module, and evaluation of integration completeness.

## Metrics Module Architecture

The metrics core is located at `src/core/stats/`, consisting of:

| Component                  | File                   | Responsibility                                                        |
| -------------------------- | ---------------------- | --------------------------------------------------------------------- |
| **StatsManager**           | `manager.rs`           | Unified metrics manager, provides all `record_*` methods              |
| **QueryMetrics**           | `metrics.rs`           | Lightweight query metrics (microsecond precision, returned to client) |
| **QueryProfile**           | `profile.rs`           | Detailed query profile (millisecond precision, for monitoring)        |
| **ErrorStatsManager**      | `error_stats.rs`       | Error statistics                                                      |
| **LatencyHistogram**       | `latency_histogram.rs` | Latency percentile calculations                                       |
| **AggregatedStatsManager** | `aggregated_stats.rs`  | Aggregated query statistics                                           |
| **SlowQueryLogger**        | `slow_query_logger.rs` | Slow query logging                                                    |

StatsManager defines **7 categories, 40+** `MetricType` enum values covering query, storage, search, transaction, sync, and index dimensions.

## Module-by-Module Integration Details

### 1. `src/query/` — Query Module ⭐ Most Complete

**Integration**: `QueryPipelineManager` holds `Arc<StatsManager>`, instruments the full lifecycle in `execute_query_with_profile`.

**Integrated Metrics**:

- `NumQueries` / `NumActiveQueries` — query counting ✅
- `QueryParseTimeUs` / `QueryValidateTimeUs` / `QueryPlanTimeUs` / `QueryOptimizeTimeUs` / `QueryExecuteTimeUs` / `QueryTotalTimeUs` — per-stage timing ✅
- `QueryPlanNodeCount` / `QueryResultRowCount` — plan nodes and result rows ✅
- `NumMatchQueries` / `NumCreateQueries` / `NumUpdateQueries` / `NumDeleteQueries` / `NumInsertQueries` / `NumGoQueries` / `NumFetchQueries` / `NumLookupQueries` / `NumShowQueries` — query type classification ✅
- `record_query_profile` — full query profile recording ✅
- `record_query_metrics` — lightweight metrics recording ✅

**Gaps**:

- ❌ `record_failed_query` is never called. Errors use `record_query_profile` with failed status instead
- ❌ Query cache (PlanCacheStats, CTE cache) has its own independent stats system, **not integrated with StatsManager**

### 2. `src/storage/` — Storage Module ⭐ Decorator Pattern

**Integration**: `MetricsStorage` is a generic decorator wrapping `StorageClient`, instrumenting before/after each read/write operation.

**Integrated Metrics**:

- `StorageReadOps` / `StorageWriteOps` — operation counting ✅
- `StorageReadLatencyUs` / `StorageWriteLatencyUs` — latency tracking ✅
- `StorageErrors` — error counting ✅

**Gaps**:

- ❌ `record_storage_cache_hit` is never called
- ❌ `record_index_scan` / `record_index_write` are never called

### 3. `src/search/` — Search Module ⭐ Most Complete

**Integration**: `MetricsSearchEngine` decorator wrapping `SearchEngine`, `IndexCache` records cache metrics, `FulltextIndexManager` injects StatsManager via `set_stats_manager`.

**Integrated Metrics**:

- `NumSearchQueries` / `SearchLatencyMs` / `NumSearchErrors` — search operations ✅
- `NumIndexOperations` / `IndexLatencyMs` / `NumIndexErrors` — index operations ✅
- `NumDeleteOperations` / `DeleteLatencyMs` / `NumDeleteErrors` — delete operations ✅
- `SearchResultCount` — search result count ✅
- `SearchCacheHitCount` / `SearchCacheMissCount` — cache hit/miss ✅
- `SearchError*` classified errors (IndexNotFound / EngineError / IoError / Serialization / Internal) ✅

**Assessment**: The search module has the most complete integration — all operations have latency + success/failure dual-dimension monitoring, with fine-grained error classification.

### 4. `src/transaction/` — Transaction Module ⚠️ Partial

**Integration**: `TransactionStats` holds `Option<Arc<StatsManager>>`, constructed via `with_stats_manager`. TransactionManager accepts StatsManager via `with_stats_manager`.

**Integrated Metrics**:

- `TxnBeginCount` / `TxnActiveCount` ✅
- `TxnCommitCount` ✅
- `TxnRollbackCount` ✅
- `TxnConflictCount` ✅

**Issues**:

- ⚠️ `TransactionManager::new()` and `with_version_config()` do NOT integrate StatsManager — only `with_stats_manager()` does
- ⚠️ In `api/mod.rs`, TransactionManager is created with `TransactionManager::new(txn_config)` NOT `with_stats_manager`, so **Server mode transaction metrics are effectively empty**
- ⚠️ In `embedded/database.rs`, same issue — `TransactionManager::new(txn_manager_config)` without StatsManager

### 5. `src/sync/` — Sync Module ⚠️ Partial

**Integration**: `SyncCoordinator` holds `Option<Arc<StatsManager>>`, injected via `with_stats_manager`.

**Integrated Metrics**:

- `SyncOperations` / `SyncLatencyMs` — recorded in `on_change` method ✅

**Gaps**:

- ❌ `record_sync_error` is never called
- ❌ `set_sync_queue_depth` is never called
- ⚠️ `stats_manager` is `Option` type — if `with_stats_manager` is not called, no metrics are recorded

### 6. `src/api/` — API Module ⭐ Good

**Server Mode** (`graph_service.rs`):

- ✅ Creates StatsManager with SlowQueryLogger enabled
- ✅ StatsManager shared as singleton to QueryApi
- ✅ Injects StatsManager into FulltextIndexManager in `start_service_with_config`
- ✅ HTTP statistics handlers expose REST API for querying metrics

**Embedded Mode** (`database.rs`):

- ✅ Creates shared StatsManager
- ✅ Passes to QueryApi

**Issues**:

- ⚠️ Server mode: TransactionManager not wired with StatsManager
- ⚠️ Embedded mode: TransactionManager not wired with StatsManager

### 7. `src/core/session_stats.rs` — Session Statistics

✅ `SessionStatistics` is an independent session-level counter (last_changes, total_changes, last_insert_id, etc.), **designed not to** integrate with StatsManager — different abstraction level.

## StatsManager API Usage Completeness Matrix

| StatsManager Method          | Defined At     | Called?                          | Missing From                                              |
| ---------------------------- | -------------- | -------------------------------- | --------------------------------------------------------- |
| `record_query_metrics`       | manager.rs:583 | ✅ query_pipeline_manager        | —                                                         |
| `record_query_profile`       | manager.rs:219 | ✅ query_pipeline_manager        | —                                                         |
| `record_failed_query`        | manager.rs:569 | ❌ **Never called**              | query_pipeline_manager uses record_query_profile directly |
| `record_aggregated_query`    | manager.rs:664 | ✅ internal write_slow_query_log | —                                                         |
| `record_storage_read`        | manager.rs:820 | ✅ storage/metrics.rs            | —                                                         |
| `record_storage_write`       | manager.rs:829 | ✅ storage/metrics.rs            | —                                                         |
| `record_storage_cache_hit`   | manager.rs:838 | ❌ **Never called**              | storage layer                                             |
| `record_search`              | manager.rs:740 | ✅ search/metrics.rs             | —                                                         |
| `record_index_operation`     | manager.rs:763 | ✅ search/metrics.rs             | —                                                         |
| `record_delete_operation`    | manager.rs:781 | ✅ search/metrics.rs             | —                                                         |
| `record_search_result_count` | manager.rs:799 | ✅ search/metrics.rs             | —                                                         |
| `record_cache_hit`           | manager.rs:806 | ✅ search/index_cache.rs         | —                                                         |
| `record_search_error`        | manager.rs:925 | ✅ search/metrics.rs             | —                                                         |
| `record_index_error`         | manager.rs:934 | ✅ search/metrics.rs             | —                                                         |
| `record_delete_error`        | manager.rs:943 | ✅ search/metrics.rs             | —                                                         |
| `record_txn_begin`           | manager.rs:849 | ✅ transaction/types.rs          | —                                                         |
| `record_txn_commit`          | manager.rs:855 | ✅ transaction/types.rs          | —                                                         |
| `record_txn_rollback`        | manager.rs:861 | ✅ transaction/types.rs          | —                                                         |
| `record_txn_conflict`        | manager.rs:867 | ✅ transaction/types.rs          | —                                                         |
| `record_sync_operation`      | manager.rs:871 | ✅ coordinator.rs                | —                                                         |
| `record_sync_error`          | manager.rs:879 | ❌ **Never called**              | sync layer                                                |
| `set_sync_queue_depth`       | manager.rs:883 | ❌ **Never called**              | sync layer                                                |
| `record_index_scan`          | manager.rs:887 | ❌ **Never called**              | storage/query layer                                       |
| `record_index_write`         | manager.rs:892 | ❌ **Never called**              | storage/query layer                                       |
| `set_index_memory_usage`     | manager.rs:896 | ❌ **Never called**              | index layer                                               |

## Summary: Integration Completeness Assessment

### ✅ Fully Integrated Modules

1. **Search** — all operations with latency + success/failure + error classification + cache hit
2. **Query Pipeline** — full lifecycle timing + query type classification + profile recording

### ⚠️ Partially Integrated Modules

3. **Storage** — read/write operations integrated, but cache hit / index scan / index write not reported
4. **Transaction** — metrics defined completely, but **StatsManager not passed to TransactionManager** in Server/Embedded modes
5. **Sync** — sync operation latency recorded, but sync error / queue depth not reported

### ❌ Completely Missing Integration

6. **Query Cache** — PlanCacheStats / CTE Cache have their own stats system, **completely independent from StatsManager**
7. **Index Metrics** — `record_index_scan` / `record_index_write` / `set_index_memory_usage` **defined but never called by any code**

## Key Issues

| #   | Issue                                                                            | Impact                                     | Affected Files                       |
| --- | -------------------------------------------------------------------------------- | ------------------------------------------ | ------------------------------------ |
| 1   | TransactionManager not wired with StatsManager                                   | Transaction metrics always zero            | `api/mod.rs`, `embedded/database.rs` |
| 2   | `record_storage_cache_hit` not called                                            | Cannot monitor storage cache efficiency    | —                                    |
| 3   | `record_sync_error` / `set_sync_queue_depth` not called                          | Sync errors/queue depth invisible          | —                                    |
| 4   | `record_index_scan` / `record_index_write` / `set_index_memory_usage` not called | Index operation metrics completely missing | —                                    |
| 5   | `record_failed_query` not called                                                 | Query failures not using dedicated path    | `query_pipeline_manager.rs`          |
| 6   | Query cache not integrated with StatsManager                                     | Cache hit rate invisible                   | `query/cache/stats.rs`               |
