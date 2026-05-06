# Cache Module Analysis and Improvement Plan

## 1. Industry Cache Implementation Research

### 1.1 RocksDB Block Cache

RocksDB is a high-performance embedded key-value store, and its block cache implementation is a reference for many database systems.

#### Key Features

**Sharded LRU Cache**

```cpp
rocksdb::NewLRUCache(
    size_t capacity,           // Total capacity in bytes
    int num_shard_bits = 0,    // Number of shard bits (2^n shards)
    uint32_t env_id = 0,
    uint64_t high_priority_pool_ratio = 0,
    bool strict_capacity_limit = true
)
```

- **Sharding**: Uses `num_shard_bits` to determine the number of shards (2^n), reducing mutex contention
- **High Priority Pool**: Reserves a portion of capacity for high-priority entries (indexes, filters)
- **Strict Capacity Limit**: When enabled, strictly adheres to capacity limits

**Lock Contention Optimization**

- Implemented as `ShardedLRUCache` to distribute load
- For high-throughput scenarios, can bypass table cache by setting `max_open_files = -1`
- `HyperClockCache`: Experimental lock-free alternative offering up to 4.5x higher ops/sec

**Statistics Monitoring**

```cpp
BLOCK_CACHE_MISS          // Total cache misses
BLOCK_CACHE_HIT           // Total cache hits
BLOCK_CACHE_INDEX_MISS/HIT   // Index block stats
BLOCK_CACHE_FILTER_MISS/HIT  // Filter block stats
BLOCK_CACHE_DATA_MISS/HIT    // Data block stats
BLOCK_CACHE_BYTES_READ       // Bytes read from cache
BLOCK_CACHE_BYTES_WRITE      // Bytes written to cache
```

**SimCache for Capacity Planning**

```cpp
std::shared_ptr<Cache> cache = NewLRUCache(capacity);
std::shared_ptr<Cache> sim_cache = NewSimCache(cache, sim_capacity, sim_num_shard_bits);
```

Simulates different cache capacities to predict hit rates.

### 1.2 PostgreSQL Buffer Pool

PostgreSQL uses a sophisticated buffer replacement strategy.

#### Clock Sweep Algorithm

PostgreSQL replaced the simple LRU algorithm with a more sophisticated approach:

- **Usage Count**: Each buffer has a `usagecount` (0-5) that increments on access
- **Clock Sweep**: Scans buffers in a circular fashion, decrementing usage count of unpinned buffers
- **Buffer Selection**: Selects buffer with lowest usage count for replacement

**Buffer States**

```sql
SELECT * FROM pg_buffercache;
-- Returns: bufferid, relfilenode, reltablespace, reldatabase,
--          relforknumber, relblocknumber, isdirty, usagecount, pinning_backends
```

**Key Improvements Over LRU**

- More resistant to sequential scans flooding the cache
- Better handles mixed workloads (OLTP + OLAP)
- Dynamically optimizes based on access patterns

### 1.3 Redis Memory Cache

Redis provides multiple eviction policies for memory management.

#### Eviction Policies

| Policy            | Description                                   |
| ----------------- | --------------------------------------------- |
| `noeviction`      | Returns error when memory limit reached       |
| `allkeys-lru`     | Evicts least recently used keys from all keys |
| `allkeys-lfu`     | Evicts least frequently used keys             |
| `allkeys-random`  | Randomly evicts keys                          |
| `volatile-lru`    | Evicts LRU among keys with expiration set     |
| `volatile-lfu`    | Evicts LFU among keys with expiration set     |
| `volatile-random` | Randomly evicts among keys with expiration    |
| `volatile-ttl`    | Evicts keys with shortest TTL                 |

**Configuration**

```redis
maxmemory 2mb
maxmemory-policy allkeys-lru
```

### 1.4 SQLite Page Cache

SQLite uses a dedicated memory pool for page cache.

#### Fixed-Size Slot Allocator

```c
sqlite3_config(SQLITE_CONFIG_PAGECACHE, pBuf, sz, N);
```

**Advantages**

1. **Fast Allocation**: All allocations are the same size, no need to coalesce or search
2. **No Fragmentation**: Memory space equals maximum memory used
3. **Predictable Memory**: Fixed memory footprint

### 1.5 Moka Cache (Rust)

Moka is a high-performance concurrent cache library for Rust, inspired by Java Caffeine.

#### TinyLFU Policy

Moka uses TinyLFU (Least Frequently Used with tiny footprint) which combines:

- **LFU Admission**: New entries compete with existing ones based on frequency
- **LRU Eviction**: Within admitted entries, least recently used is evicted

**Architecture Components**

- **FrequencySketch**: For admission decisions
- **Deque-based LRU Queues**: For ordering
- **TimerWheel**: For expiration management

**Configuration Example**

```rust
use moka::sync::Cache;
use moka::policy::EvictionPolicy;

let cache: Cache<String, String> = Cache::builder()
    .max_capacity(100)
    .eviction_policy(EvictionPolicy::lru())
    .time_to_live(Duration::from_secs(3600))  // TTL
    .time_to_idle(Duration::from_secs(300))   // TTI
    .weigher(|_key, value: &String| -> u32 {
        value.len().try_into().unwrap_or(u32::MAX)
    })
    .build();
```

**Size-Aware Eviction**

```rust
let cache: Cache<u32, String> = Cache::builder()
    .weigher(|_key, value: &String| -> u32 {
        value.len().try_into().unwrap_or(u32::MAX)
    })
    .max_capacity(32 * 1024 * 1024)  // 32 MiB
    .build();
```
