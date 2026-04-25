# Fulltext and Vector Index Synchronization Difference Analysis

## Overview

This document analyzes the synchronization mechanisms for fulltext and vector indexes in GraphDB, identifies key differences, and provides improvement recommendations.

## 1. Implementation Comparison

### 1.1 Architecture Overview

| Aspect | Fulltext Index | Vector Index |
|--------|----------------|--------------|
| **Storage Location** | Local filesystem | Remote Qdrant service |
| **Dependency** | `bm25-service` / `inversearch-service` | `vector-client` → `qdrant-client` |
| **Communication** | Local function call | gRPC/HTTP network call |
| **Latency** | Microseconds | Milliseconds to seconds |
| **Failure Modes** | Local I/O errors | Network timeout, connection loss, service unavailable |
| **Transaction Support** | Real commit/rollback | **No real transaction** |

### 1.2 Key Code Differences

**Fulltext Index - Local Implementation** (`src/search/adapters/bm25_adapter.rs`)

```rust
pub struct Bm25SearchEngine {
    index: Bm25Index,           // Local index object
    index_path: PathBuf,        // Local file path
}

async fn commit(&self) -> Result<(), SearchError> {
    self.index.commit()?;       // Local commit
    Ok(())
}

async fn rollback(&self) -> Result<(), SearchError> {
    Ok(())                      // Local rollback (if supported)
}
```

**Vector Index - Network Implementation** (`crates/vector-client/src/engine/qdrant/mod.rs`)

```rust
pub struct QdrantEngine {
    client: Arc<Qdrant>,        // Network client
    config: VectorClientConfig,
}

async fn create_client(conn_config: &ConnectionConfig) -> Result<Qdrant> {
    let url = conn_config.to_url();
    let client = Qdrant::from_url(&url).build()?;
    client.health_check().await?;  // Network health check
}
```

### 1.3 Synchronization Flow (After Fix)

```
┌─────────────────────────────────────────────────────────────────┐
│                        Data Change Event                         │
└─────────────────────────────┬───────────────────────────────────┘
                              │
              ┌───────────────┴───────────────┐
              ↓                               ↓
    ┌─────────────────┐             ┌─────────────────┐
    │  Fulltext Index │             │  Vector Index   │
    │  (Local)        │             │  (Remote)       │
    └────────┬────────┘             └────────┬────────┘
             │                               │
             ↓                               ↓
    ┌─────────────────┐             ┌─────────────────┐
    │ FulltextClient  │             │ VectorClient    │
    │ - SearchEngine  │             │ - VectorManager │
    │ - Local call    │             │ - Network call  │
    │ - 2 retries     │             │ - 3 retries     │
    │ - With DLQ      │             │ - With DLQ      │
    │ - Real txn      │             │ - Fake txn      │
    └─────────────────┘             └─────────────────┘
```

## 2. Issues Found and Fixed

### 2.1 Missing Retry Mechanism for Vector Index (FIXED)

**Before:**
```rust
// No retry for vector operations
self.vector_manager.upsert(&collection_name, point).await?;
```

**After:**
```rust
// With retry mechanism
let result = with_retry(
    || {
        let point_clone = point.clone();
        let collection_name_clone = collection_name.clone();
        let vm = vector_manager.clone();
        async move { vm.upsert(&collection_name_clone, point_clone).await }
    },
    &self.config.retry_config,
).await;
```

### 2.2 Missing Dead Letter Queue for Vector Index (FIXED)

**Before:**
```rust
// Failed operations were lost
Err(e) => {
    return Err(ExternalIndexError::InsertError(e.to_string()));
}
```

**After:**
```rust
// Failed operations are saved to DLQ with full operation data
Err(e) => {
    let error_msg = e.to_string();
    self.add_to_dlq(
        IndexOperation::Insert {
            key: self.index_key(),
            id: id.to_string(),
            data: data.clone(),  // Full data preserved
            payload: HashMap::new(),
        },
        &error_msg,
    );
    Err(ExternalIndexError::InsertError(error_msg))
}
```

### 2.3 Inconsistent Retry Configuration (FIXED)

**Before:** Both local and remote indexes used the same retry configuration.

**After:** Differentiated retry strategies:
- **Local (Fulltext):** 2 retries, 50ms initial delay, 2s max delay
- **Remote (Vector):** 3 retries, 100ms initial delay, 10s max delay

```rust
fn default_local_retry_config() -> RetryConfig {
    RetryConfig::new(2, Duration::from_millis(50), Duration::from_secs(2))
}

fn default_remote_retry_config() -> RetryConfig {
    RetryConfig::new(3, Duration::from_millis(100), Duration::from_secs(10))
}
```

### 2.4 Missing Circuit Breaker (ADDED)

Added circuit breaker module (`src/sync/circuit_breaker.rs`) for protecting remote service calls:

```rust
pub struct CircuitBreakerConfig {
    pub failure_threshold: u64,     // Default: 5
    pub recovery_timeout: Duration, // Default: 30s
    pub success_threshold: u64,     // Default: 3
    pub failure_window: Duration,   // Default: 60s
}
```

## 3. Remaining Issues

### 3.1 Architecture Duplication

Two independent coordinators still exist:
- `SyncCoordinator` handles fulltext index
- `VectorSyncCoordinator` handles vector index

**Recommendation:** Consider merging into a unified coordinator in a future refactor.

### 3.2 Inconsistent Transaction Semantics

**Fulltext Index** (`src/sync/external_index/fulltext_client.rs`):
```rust
async fn commit(&self) -> IndexResult<()> {
    self.search_engine.commit().await...
}

async fn rollback(&self) -> IndexResult<()> {
    self.search_engine.rollback().await...
}
```

**Vector Index** (`src/sync/external_index/vector_client.rs`):
```rust
async fn commit(&self) -> IndexResult<()> {
    debug!("VectorClient commit: no-op for remote vector store");
    Ok(())
}

async fn rollback(&self) -> IndexResult<()> {
    debug!("VectorClient rollback: no-op for remote vector store");
    Ok(())
}
```

**Issue:** When transaction rolls back, fulltext index can rollback but vector index operations cannot be undone, leading to potential data inconsistency.

**Recommendation:** Implement compensation transactions or document this limitation clearly.

## 4. New Features Added

### 4.1 VectorClientConfig

```rust
pub struct VectorClientConfig {
    pub retry_config: RetryConfig,
    pub operation_timeout: Duration,
    pub use_dead_letter_queue: bool,
}

impl VectorClientConfig {
    pub fn with_retry_config(mut self, retry_config: RetryConfig) -> Self;
    pub fn with_operation_timeout(mut self, timeout: Duration) -> Self;
    pub fn with_dead_letter_queue(mut self, use_dlq: bool) -> Self;
}
```

### 4.2 Circuit Breaker

```rust
pub struct CircuitBreaker {
    // ...
}

pub enum CircuitState {
    Closed,    // Normal operation
    Open,      // Failing, requests blocked
    HalfOpen,  // Testing recovery
}

// Usage
let result = with_circuit_breaker(&circuit_breaker, || async {
    some_remote_operation().await
}).await;
```

## 5. Files Modified

| File | Changes |
|------|---------|
| `src/sync/external_index/vector_client.rs` | Added retry, DLQ support, VectorClientConfig |
| `src/sync/external_index/mod.rs` | Export VectorClientConfig |
| `src/sync/coordinator/coordinator.rs` | Differentiated retry configs for local/remote |
| `src/sync/circuit_breaker.rs` | New file - circuit breaker implementation |
| `src/sync/mod.rs` | Export circuit breaker module |

## 6. Summary

| Aspect | Status |
|--------|--------|
| **Retry for Vector Index** | ✅ Fixed |
| **DLQ for Vector Index** | ✅ Fixed |
| **Differentiated Retry Config** | ✅ Fixed |
| **Circuit Breaker** | ✅ Added |
| **Unified Coordinator** | ⏳ Future work |
| **Compensation Transactions** | ⏳ Future work |
