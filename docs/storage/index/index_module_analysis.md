# Index Storage Module Analysis

## Overview

The `src/storage/index` module is the **storage layer index data management module** for the GraphDB graph database. This document analyzes its design, implementation, and compares it with other database systems.

## Module Structure

```
src/storage/index/
├── mod.rs                    # Module entry point
├── index_data_manager.rs     # IndexDataManager trait and InMemoryIndexDataManager
├── vertex_index_manager.rs   # Vertex index management
├── edge_index_manager.rs     # Edge index management
├── index_key_codec.rs        # Index key encoding/decoding
├── index_updater.rs          # DML-triggered index updates
├── index_gc_manager.rs       # Garbage collection for tombstones
└── index_compression.rs      # Compression utilities (not integrated)
```

## Core Responsibilities

| Component                  | Responsibility                              |
| -------------------------- | ------------------------------------------- |
| `IndexDataManager` trait   | Defines interface for index CRUD operations |
| `InMemoryIndexDataManager` | In-memory implementation using BTreeMap     |
| `VertexIndexManager`       | Manages vertex property indexes             |
| `EdgeIndexManager`         | Manages edge property indexes               |
| `IndexKeyCodec`            | Serializes/deserializes index keys          |
| `IndexUpdater`             | Maintains index consistency during DML      |
| `IndexGcManager`           | Background cleanup of deleted entries       |

## Current Design Analysis

### Strengths

#### 1. Clear Layered Architecture

```
┌─────────────────────────────────────────┐
│           IndexUpdater                   │  <- DML Integration Layer
├─────────────────────────────────────────┤
│       IndexDataManager (trait)          │  <- Abstraction Layer
├─────────────────────────────────────────┤
│  VertexIndexManager │ EdgeIndexManager  │  <- Implementation Layer
├─────────────────────────────────────────┤
│           IndexKeyCodec                  │  <- Serialization Layer
└─────────────────────────────────────────┘
```

- Separation of metadata management (`metadata::IndexMetadataManager`) from data management
- Trait-based abstraction allows future alternative implementations

#### 2. MVCC Support

```rust
pub struct IndexEntry {
    pub created_ts: Timestamp,
    pub deleted_ts: Option<Timestamp>,
}

impl IndexEntry {
    pub fn is_visible_at(&self, read_ts: Timestamp) -> bool {
        self.created_ts <= read_ts
            && self.deleted_ts.map_or(true, |deleted_ts| deleted_ts > read_ts)
    }
}
```

- Multi-version concurrency control via timestamps
- Snapshot isolation for reads
- Soft deletes with tombstone markers

#### 3. Dual Index Structure

| Index Type    | Key Format                                      | Purpose                       |
| ------------- | ----------------------------------------------- | ----------------------------- |
| Forward Index | `(space_id, index_name, prop_value, entity_id)` | Fast lookup by property value |
| Reverse Index | `(space_id, entity_id, index_name)`             | Fast delete by entity ID      |

This design enables:

- O(log n) lookups via BTreeMap
- Efficient range queries
- Quick deletion without full index scan

#### 4. Incremental Garbage Collection

```rust
pub fn gc_tombstones_incremental(
    &self,
    safe_ts: Timestamp,
    batch_size: usize,
) -> Result<GcStats, StorageError>
```

- Background GC with configurable batch size
- Rate limiting to avoid impacting normal operations
- Integration with `VersionManager` for safe timestamp determination

#### 5. Transaction Support

- `IndexUpdateContext` for batch operations
- `UndoLog` for transaction rollback
- Commit/rollback semantics

### Weaknesses and Improvement Opportunities

#### 1. Memory-Only Storage

**Current Implementation:**

```rust
pub struct VertexIndexManager {
    forward_index: Arc<RwLock<BTreeMap<IndexKey, IndexEntry>>>,
    reverse_index: Arc<RwLock<BTreeMap<IndexKey, IndexEntry>>>,
}
```

**Problem:**

- All index data stored in memory
- Cannot handle datasets larger than available RAM
- Restart requires full index rebuild

**Recommendation:**

- Consider LSM-tree based storage (like RocksDB)
- Or integrate with existing KV store (redb, RocksDB)

#### 2. Delete Operation Complexity

**Current Flow:**

```
delete_vertex_indexes():
  1. Scan reverse_index to find all index_names for vertex
  2. For each index_name, scan forward_index to find matching keys
  3. Mark entries as deleted
```

**Problem:**

- Multiple lock acquisitions
- O(n) scans within each index partition
- Potential lock contention

**Recommendation:**

- Store forward key reference in reverse index
- Or use a combined index structure

#### 3. Key Parsing Complexity

**Current Implementation:**

```rust
// Manual byte offset management
let prop_value_len = u32::from_le_bytes(
    key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])
) as usize;
```

**Problem:**

- Error-prone manual offset calculations
- `unwrap_or([0; 4])` masks parsing errors
- Difficult to maintain and extend

**Recommendation:**

- Use structured serialization (bincode, protobuf)
- Define clear key format with builder pattern

#### 4. Unused Compression Module

The `index_compression.rs` defines `PrefixCompressor` and `DictionaryCompressor` but they are not integrated into the main flow.

**Recommendation:**

- Remove if not needed (reduce maintenance burden)
- Or integrate into persistence layer

#### 5. No Composite Index Support

```rust
pub fn build_vertex_index_key(
    space_id: u64,
    index_name: &str,
    prop_value: &Value,  // Single property only
    vertex_id: &Value,
) -> Result<ByteKey, StorageError>
```

**Problem:**

- Cannot create indexes on multiple properties
- Limits query optimization for multi-column filters

**Recommendation:**

- Extend key format to support multiple property values
- Add composite index support to `Index` definition

#### 6. GC Manager Type Coupling

```rust
pub struct IndexGcManager {
    index_manager: InMemoryIndexDataManager,  // Concrete type
    // ...
}
```

**Problem:**

- Violates trait abstraction principle
- Cannot swap implementations

**Recommendation:**

- Make generic: `IndexGcManager<I: IndexDataManager>`
- Or add GC methods to trait

---

## Comparative Analysis with Other Databases

### 1. RocksDB (LSM-Tree Storage)

**Architecture:**

```
┌─────────────────────────────────────────────────────┐
│                    Write Path                        │
├─────────────────────────────────────────────────────┤
│  Write → WAL (Write-Ahead Log) → MemTable (Memory)  │
│                                       ↓              │
│                              Immutable MemTable      │
│                                       ↓              │
│                              Flush to SST Files      │
│                                       ↓              │
│                           Compaction (Background)   │
└─────────────────────────────────────────────────────┘
```

**Key Features:**

- **MemTable**: In-memory sorted buffer (skiplist)
- **WAL**: Durability guarantee before acknowledgment
- **SST Files**: Sorted string tables on disk
- **Compaction**: Background merge of SST files

**Applicable to GraphDB:**
| Feature | Applicability |
|---------|---------------|
| MemTable | Already using BTreeMap (similar concept) |
| WAL | Could add for durability |
| SST Files | Could use for disk-based index storage |
| Compaction | Could use for GC instead of tombstones |

### 2. Neo4j (Native Graph Storage)

**Architecture:**

```
┌─────────────────────────────────────────────────────┐
│                  Native Graph Storage               │
├─────────────────────────────────────────────────────┤
│  Node Store ←→ Relationship Store ←→ Property Store │
│       ↓              ↓                    ↓         │
│  Index-Free Adjacency (O(1) traversal)             │
└─────────────────────────────────────────────────────┘
```

**Key Features:**

- **Index-Free Adjacency**: Direct pointers between nodes and relationships
- **Native Graph Format**: No translation layer needed
- **Property Store**: Separate storage for properties
- **Multiple Index Types**: Range, full-text, vector indexes

**Applicable to GraphDB:**
| Feature | Applicability |
|---------|---------------|
| Index-Free Adjacency | Already using CSR structure |
| Property Store | Could separate property storage |
| Multiple Index Types | Already have tag/edge indexes |

### 3. NebulaGraph (neug Reference Implementation)

**Architecture from `ref/neug/storages`:**

```
┌─────────────────────────────────────────────────────┐
│                  PropertyGraph                       │
├─────────────────────────────────────────────────────┤
│  VertexTable[]    │    EdgeTable[]                  │
│       ↓           │         ↓                        │
│  Indexer + Table  │    CSR (Mutable/Immutable)      │
│       ↓           │         ↓                        │
│  MMapContainer    │    MMapContainer                 │
└─────────────────────────────────────────────────────┘
```

**Key Components:**

#### MMapContainer (`container/mmap_container.cc`)

```cpp
class MMapContainer : public IDataContainer {
    void* mmap_data_;   // Memory-mapped data
    size_t mmap_size_;  // Size of mapped region

    void Open(const std::string& path) {
        mmap_data_ = mmap(path, mmap_size_);
        // Uses MD5 checksum for integrity
    }
};
```

- Memory-mapped file I/O for efficient access
- Supports both file-backed and anonymous mappings
- Checksum verification for data integrity

#### VertexTable (`graph/vertex_table.cc`)

```cpp
class VertexTable {
    IndexerType indexer_;      // OID -> VID mapping
    Table* table_;             // Property columns
    VertexTimestamp v_ts_;     // MVCC timestamps

    bool AddVertex(Property id, vector<Property> props,
                   vid_t& vid, timestamp_t ts);
};
```

- Separates ID indexing from property storage
- MVCC timestamps for visibility control
- Batch operations for bulk loading

#### MutableCSR (`csr/mutable_csr.cc`)

```cpp
template <typename EDATA_T>
class MutableCsr {
    IDataContainer* nbr_list_;       // Neighbor list
    IDataContainer* degree_list_;    // Degree array
    IDataContainer* cap_list_;       // Capacity array
    IDataContainer* adj_list_buffer_; // Pointer array
    SpinLock* locks_;                // Per-vertex locks

    void compact();  // Remove deleted edges
};
```

- CSR (Compressed Sparse Row) for edge storage
- Per-vertex spin locks for concurrency
- Timestamp-based edge visibility
- Compact operation for garbage collection

**Applicable to GraphDB:**
| Feature | Current GraphDB | NebulaGraph | Recommendation |
|---------|-----------------|-------------|----------------|
| Storage Backend | BTreeMap in memory | MMapContainer | Consider mmap for persistence |
| Index Structure | Forward + Reverse | Indexer + Table | Similar approach |
| Edge Storage | Separate index | CSR | CSR is more efficient |
| MVCC | Timestamp in entry | VertexTimestamp | Similar approach |
| GC | Tombstone marking | Compact operation | Consider physical deletion |

---

## Improvement Recommendations

### Short-term (Low Effort)

1. **Fix Key Parsing**
   - Replace manual byte manipulation with structured serialization
   - Add proper error handling

2. **Remove Unused Code**
   - Delete or integrate `index_compression.rs`

3. **Fix Type Coupling**
   - Make `IndexGcManager` generic over `IndexDataManager`

### Medium-term (Moderate Effort)

4. **Add Persistence Layer**

   ```
   Option A: Use existing redb
   Option B: Integrate RocksDB
   Option C: Implement mmap-based storage (like neug)
   ```

5. **Optimize Delete Operations**
   - Store forward key reference in reverse index
   - Or use combined index structure

6. **Add Composite Index Support**
   - Extend key format for multiple properties
   - Update query optimizer to use composite indexes

### Long-term (High Effort)

7. **Consider CSR for Edge Storage**
   - More memory-efficient for graph structure
   - Better cache locality for traversals

8. **Implement WAL for Durability**
   - Write-ahead log before index updates
   - Recovery mechanism on restart

---

## Summary

| Aspect                | Rating        | Notes                             |
| --------------------- | ------------- | --------------------------------- |
| Module Organization   | ✅ Good       | Clear separation of concerns      |
| MVCC Design           | ✅ Good       | Proper timestamp-based visibility |
| Interface Abstraction | ⚠️ Partial    | Some concrete type coupling       |
| Storage Strategy      | ⚠️ Limited    | Memory-only, no persistence       |
| Delete Efficiency     | ⚠️ Needs Work | Multiple scans required           |
| Code Quality          | ⚠️ Needs Work | Complex key parsing               |
| Feature Completeness  | ⚠️ Partial    | No composite indexes              |

**Overall Assessment:** The current design is **reasonable for small to medium datasets** with in-memory constraints. For production use with larger datasets, the primary improvements needed are:

1. Persistent storage backend
2. Optimized delete operations
3. Composite index support
