# GraphDB Storage Engine Analysis

## Overview

This document analyzes the current storage implementation of GraphDB, comparing it with industry-standard databases like RocksDB, SQLite, Neo4j, and TiKV, and provides recommendations for improvement in memory management and persistence.

## Current Storage Architecture

### Core Components

```
PropertyGraph
├── VertexTables (HashMap<LabelId, VertexTable>)
│   ├── IdIndexer (external ID -> internal ID mapping)
│   ├── ColumnStore (columnar property storage)
│   └── VertexTimestamp (MVCC timestamps)
├── EdgeTables (HashMap<(LabelId, LabelId, LabelId), EdgeTable>)
│   ├── MutableCsr (outgoing edges)
│   ├── MutableCsr (incoming edges)
│   └── PropertyTable (edge properties)
└── SchemaManager (metadata management)
```

### Memory Usage Analysis

#### 1. VertexTable Memory Layout

| Component       | Data Structure                | Memory Characteristics                              |
| --------------- | ----------------------------- | --------------------------------------------------- |
| IdIndexer       | `Vec<K> + HashMap<K, u32>`    | Double storage: keys stored in both Vec and HashMap |
| ColumnStore     | `Vec<u8> + Option<Vec<bool>>` | Raw bytes + null bitmap, grows dynamically          |
| VertexTimestamp | `Vec<Timestamp>`              | Linear growth with vertex count                     |

**Issues:**

- No memory limit configuration
- No compression for sparse data
- Null bitmap uses `Vec<bool>` (1 byte per element) instead of bit-level packing
- String storage is inline with length prefix, causing fragmentation

#### 2. EdgeTable Memory Layout

| Component     | Data Structure                       | Memory Characteristics                                  |
| ------------- | ------------------------------------ | ------------------------------------------------------- |
| MutableCsr    | `Vec<Vec<Nbr>>`                      | Nested Vec, each adjacency list independently allocated |
| DeletedEdges  | `Mutex<HashSet<(VertexId, EdgeId)>>` | Grows unbounded without cleanup                         |
| PropertyTable | Similar to ColumnStore               | Same issues as vertex ColumnStore                       |

**Issues:**

- `Vec<Vec<Nbr>>` has poor cache locality
- Deleted edges set grows without bounds
- No edge compaction mechanism
- No memory pooling

#### 3. Schema Manager Memory

- All schema data kept in memory
- No lazy loading of schema components
- Index metadata fully loaded regardless of usage

### Persistence Analysis

#### Current Implementation

```rust
pub fn flush(&self) -> StorageResult<()> {
    // 1. Write version file
    // 2. Flush each vertex table
    // 3. Flush each edge table
    // 4. Sync WAL
}
```

**Issues:**

1. **Full Flush**: All data written on every flush, no incremental persistence
2. **No Compaction**: Deleted data not removed during flush
3. **Synchronous I/O**: Blocking writes, no async support
4. **No Compression**: Raw bytes written directly to disk
5. **No Checksum**: Data integrity not verified

## Comparison with Other Databases

### RocksDB (LSM-Tree Architecture)

**Key Features:**

- **MemTable**: In-memory sorted buffer with configurable size
- **SST Files**: Immutable sorted files on disk
- **Block Cache**: LRU cache for hot data blocks
- **Write Buffer Manager**: Unified memory management for memtables
- **Compaction**: Background merge of SST files

**Memory Management:**

```cpp
// RocksDB approach
WriteBufferManager write_buffer_manager(100 * 1024 * 1024, block_cache);
// Limits total memtable memory to 100MB
// Integrates with block cache for unified memory budget
```

**Lessons for GraphDB:**

1. Implement memory budget with soft/hard limits
2. Add block cache for frequently accessed data
3. Use Write Buffer Manager pattern for memory control
4. Implement background flush when memory threshold exceeded

### SQLite (B-Tree Architecture)

**Key Features:**

- **Page Cache**: Configurable cache size (default 2000 pages)
- **WAL Mode**: Write-ahead logging for durability
- **Page-based Storage**: Fixed 4096-byte pages
- **Memory-mapped I/O**: Optional mmap for read operations

**Memory Management:**

```sql
PRAGMA cache_size = -64000;  -- 64MB cache
PRAGMA mmap_size = 268435456;  -- 256MB mmap
```

**Lessons for GraphDB:**

1. Implement page-based caching
2. Consider mmap for read-heavy workloads
3. Add configurable cache size limits
4. Use WAL for durability with periodic checkpoints

### Neo4j (Native Graph Database)

**Key Features:**

- **Page Cache**: Separate caches for nodes, relationships, properties
- **Transaction Log**: Circular log with checkpointing
- **Store Files**: Separate files for different entity types
- **Memory Mapping**: Optional for large datasets

**Architecture:**

```
┌─────────────────────────────────────────┐
│              Page Cache                  │
├─────────────┬─────────────┬─────────────┤
│ Node Store  │ Rel Store   │ Property    │
│ (8 bytes/   │ (34 bytes/  │ Store       │
│  node)      │  rel)       │ (variable)  │
└─────────────┴─────────────┴─────────────┘
```

**Lessons for GraphDB:**

1. Fixed-size records for vertices/edges
2. Separate stores for different data types
3. Page cache with configurable size
4. Checkpoint-based recovery

### TiKV (Distributed KV Store)

**Key Features:**

- **RocksDB-based**: Uses RocksDB as local storage engine
- **Raft Consensus**: Distributed consistency
- **Region-based Sharding**: Data partitioned into regions
- **Snapshot Isolation**: MVCC implementation

**Architecture:**

```
┌─────────────────────────────────────────┐
│           Placement Driver               │
└─────────────────────────────────────────┘
                    │
    ┌───────────────┼───────────────┐
    ▼               ▼               ▼
┌─────────┐   ┌─────────┐   ┌─────────┐
│ Store 1 │   │ Store 2 │   │ Store 3 │
│ (RocksDB)│   │ (RocksDB)│   │ (RocksDB)│
└─────────┘   └─────────┘   └─────────┘
```

**Lessons for GraphDB:**

1. Use proven storage engine (RocksDB) as foundation
2. Implement Engine trait for storage abstraction
3. Batch writes for efficiency
4. Snapshot-based reads

### neug (Reference Implementation)

**Key Features:**

- **Memory Levels**: InMemory, SyncToFile, HugePagePreferred
- **MMap Container**: Memory-mapped file I/O
- **CSR Format**: Compressed Sparse Row for edges
- **Immutable/Mutable CSR**: Separate implementations

**Memory Management:**

```cpp
enum MemoryLevel {
    kInMemory,        // Pure in-memory
    kSyncToFile,      // Sync to disk
    kHugePagePreferred  // Use huge pages if available
};
```

**Lessons for GraphDB:**

1. Implement memory level configuration
2. Use mmap for efficient file I/O
3. Separate immutable and mutable data structures
4. Support huge pages for large datasets

## Improvement Recommendations

### 1. Memory Management

#### 1.1 Implement Memory Budget

```rust
pub struct MemoryConfig {
    pub max_vertex_memory: usize,      // Max memory for vertex data
    pub max_edge_memory: usize,        // Max memory for edge data
    pub max_cache_memory: usize,       // Max memory for block cache
    pub eviction_policy: EvictionPolicy,
}

pub enum EvictionPolicy {
    LRU,           // Least recently used
    LFU,           // Least frequently used
    FIFO,          // First in first out
}
```

#### 1.2 Add Block Cache

```rust
pub struct BlockCache {
    cache: LruCache<BlockId, Arc<[u8]>>,
    memory_usage: AtomicUsize,
    max_memory: usize,
}

impl BlockCache {
    pub fn get(&self, block_id: BlockId) -> Option<Arc<[u8]>> {
        self.cache.get(&block_id).cloned()
    }

    pub fn insert(&self, block_id: BlockId, data: Vec<u8>) {
        let size = data.len();
        if self.memory_usage.load(Ordering::Relaxed) + size > self.max_memory {
            self.evict(size);
        }
        self.cache.put(block_id, Arc::from(data));
    }
}
```

#### 1.3 Optimize Data Structures

**Null Bitmap Optimization:**

```rust
// Current: Vec<bool> - 1 byte per element
pub struct NullBitmap {
    bitmap: Vec<bool>,
}

// Proposed: BitVec - 1 bit per element
pub struct NullBitmap {
    bitmap: Vec<u64>,  // 64 bits per u64
}

impl NullBitmap {
    pub fn is_null(&self, idx: usize) -> bool {
        let word = idx / 64;
        let bit = idx % 64;
        (self.bitmap[word] >> bit) & 1 == 1
    }
}
```

**CSR Optimization:**

```rust
// Current: Vec<Vec<Nbr>> - poor cache locality
pub struct MutableCsr {
    adj_lists: Vec<Vec<Nbr>>,
}

// Proposed: Flat CSR with offsets
pub struct FlatCsr {
    offsets: Vec<usize>,    // Offset into edges for each vertex
    edges: Vec<Nbr>,        // Contiguous edge array
    degrees: Vec<usize>,    // Degree of each vertex
}
```

### 2. Persistence Improvements

#### 2.1 Incremental Persistence

```rust
pub struct PersistenceManager {
    dirty_pages: HashSet<PageId>,
    last_flush: Instant,
    flush_interval: Duration,
}

impl PersistenceManager {
    pub fn mark_dirty(&mut self, page_id: PageId) {
        self.dirty_pages.insert(page_id);
    }

    pub fn flush_dirty(&mut self) -> StorageResult<()> {
        let dirty: Vec<_> = self.dirty_pages.drain().collect();
        for page_id in dirty {
            self.flush_page(page_id)?;
        }
        Ok(())
    }
}
```

#### 2.2 Checkpoint Mechanism Enhancement

```rust
pub struct CheckpointManager {
    checkpoint_dir: PathBuf,
    temp_dir: PathBuf,
    last_checkpoint: Option<CheckpointInfo>,
}

impl CheckpointManager {
    pub fn create_checkpoint(&mut self) -> StorageResult<CheckpointInfo> {
        // 1. Block new writes
        // 2. Flush all dirty data
        // 3. Create atomic snapshot
        // 4. Update checkpoint metadata
        // 5. Resume writes
    }

    pub fn restore_checkpoint(&self, checkpoint_id: u64) -> StorageResult<()> {
        // 1. Verify checkpoint exists
        // 2. Load checkpoint data
        // 3. Replay WAL from checkpoint
    }
}
```

#### 2.3 Data Compression

```rust
pub enum CompressionType {
    None,
    Snappy,
    Zstd,
    LZ4,
}

pub struct CompressedBlock {
    compression: CompressionType,
    uncompressed_size: usize,
    data: Vec<u8>,
}
```

### 3. Storage Format Improvements

#### 3.1 Page-Based Storage

```rust
pub const PAGE_SIZE: usize = 4096;

pub struct Page {
    id: PageId,
    data: [u8; PAGE_SIZE],
    checksum: u32,
    page_type: PageType,
}

pub enum PageType {
    VertexData,
    EdgeData,
    Schema,
    Index,
    Free,
}
```

#### 3.2 Fixed-Size Records

```rust
// Vertex record: 8 bytes (internal ID) + 8 bytes (timestamp)
pub const VERTEX_RECORD_SIZE: usize = 16;

// Edge record: 8 bytes (src) + 8 bytes (dst) + 8 bytes (edge_id) + 8 bytes (timestamp)
pub const EDGE_RECORD_SIZE: usize = 32;
```

### 4. Configuration Options

```rust
pub struct StorageConfig {
    // Memory settings
    pub memory: MemoryConfig,

    // Persistence settings
    pub persistence: PersistenceConfig,

    // Cache settings
    pub cache: CacheConfig,

    // WAL settings
    pub wal: WalConfig,
}

pub struct PersistenceConfig {
    pub flush_interval: Duration,
    pub checkpoint_interval: Duration,
    pub compression: CompressionType,
    pub sync_on_write: bool,
}

pub struct CacheConfig {
    pub block_cache_size: usize,
    pub metadata_cache_size: usize,
    pub cache_shards: usize,
}

pub struct WalConfig {
    pub enabled: bool,
    pub max_size: usize,
    pub sync_interval: Duration,
}
```

## Implementation Priority

### Phase 1: Memory Management (High Priority)

1. Implement memory budget configuration
2. Add block cache for hot data
3. Optimize null bitmap to bit-level packing
4. Add memory usage monitoring

### Phase 2: Persistence Enhancement (High Priority)

1. Implement incremental persistence
2. Add data compression support
3. Enhance checkpoint mechanism
4. Add checksum verification

### Phase 3: Storage Format (Medium Priority)

1. Migrate to page-based storage
2. Implement fixed-size records
3. Add storage format versioning
4. Implement data migration tools

### Phase 4: Advanced Features (Low Priority)

1. Implement mmap support
2. Add huge page support
3. Implement memory-mapped CSR
4. Add async I/O support

## Conclusion

The current GraphDB storage implementation provides basic functionality but lacks several critical features for production use:

1. **Memory Management**: No memory limits, no eviction policies, inefficient data structures
2. **Persistence**: Full flush only, no compression, no incremental persistence
3. **Performance**: Poor cache locality, no block cache, synchronous I/O only

By implementing the recommendations in this document, GraphDB can achieve:

- Predictable memory usage with configurable limits
- Efficient persistence with incremental writes and compression
- Better performance through caching and optimized data structures
- Production-ready reliability with checksums and recovery mechanisms
