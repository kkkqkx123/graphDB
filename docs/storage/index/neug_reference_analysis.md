# Neug Storage Reference Analysis

This document analyzes the storage implementation in `ref/neug/storages` directory for reference when improving GraphDB's index storage.

## Directory Structure

```
ref/neug/storages/
├── container/          # Memory-mapped container implementations
│   ├── mmap_container.cc
│   ├── file_mmap_container.cc
│   ├── anon_mmap_container.cc
│   └── container_utils.cc
├── csr/                # Compressed Sparse Row implementations
│   ├── mutable_csr.cc
│   ├── immutable_csr.cc
│   └── generic_view_utils.cc
├── graph/              # Graph data structures
│   ├── property_graph.cc
│   ├── vertex_table.cc
│   ├── edge_table.cc
│   ├── schema.cc
│   └── vertex_timestamp.cc
└── loader/             # Data loading utilities
    ├── csv_property_graph_loader.cc
    └── loader_utils.cc
```

## Core Components

### 1. MMapContainer (Memory-Mapped Storage)

**File:** `container/mmap_container.cc`

```cpp
class MMapContainer : public IDataContainer {
    void* mmap_data_;    // Pointer to mapped memory
    size_t mmap_size_;   // Size of mapped region
    std::string path_;   // File path (empty for anonymous)

    // Open existing file
    void Open(const std::string& path);

    // Create anonymous mapping (in-memory)
    void Resize(size_t size);

    // Dump to file with MD5 checksum
    void Dump(const std::string& path);

    // Check if data changed
    bool IsDirty();
};
```

**Key Design Points:**

| Feature     | Implementation                              |
| ----------- | ------------------------------------------- |
| File Format | `[FileHeader][Data...]` with MD5 checksum   |
| Memory Mode | Supports file-backed and anonymous mappings |
| Integrity   | MD5 checksum in header                      |
| Dirty Check | Compare current MD5 with stored checksum    |

**Applicable to GraphDB:**

- Could use mmap for index persistence
- MD5 checksum for data integrity
- Support both memory-only and file-backed modes

### 2. VertexTable (Vertex Storage)

**File:** `graph/vertex_table.cc`

```cpp
class VertexTable {
    IndexerType indexer_;              // OID (external ID) -> VID (internal ID)
    Table* table_;                     // Columnar property storage
    VertexTimestamp v_ts_;             // MVCC timestamp tracker
    std::shared_ptr<VertexSchema> vertex_schema_;

    // Core operations
    bool AddVertex(Property id, vector<Property> props,
                   vid_t& vid, timestamp_t ts, bool insert_safe);
    bool get_index(const Property& oid, vid_t& lid, timestamp_t ts);
    Property GetOid(vid_t lid, timestamp_t ts);
    bool IsValidLid(vid_t lid, timestamp_t ts);

    // Batch operations
    void insert_vertices(shared_ptr<IRecordBatchSupplier> supplier);
    void BatchDeleteVertices(const vector<vid_t>& vids);
};
```

**Key Design Points:**

| Component         | Purpose                             |
| ----------------- | ----------------------------------- |
| `IndexerType`     | Hash map for OID → VID lookup       |
| `Table`           | Columnar storage for properties     |
| `VertexTimestamp` | Track vertex validity per timestamp |

**MVCC Implementation:**

```cpp
bool VertexTable::get_index(const Property& oid, vid_t& lid,
                            timestamp_t ts) const {
    auto res = indexer_.get_index(oid, lid);
    if (res && !v_ts_.IsVertexValid(lid, ts)) {
        return false;  // Vertex deleted at this timestamp
    }
    return res;
}
```

**Applicable to GraphDB:**

- Separate indexer from property storage
- Columnar storage for properties (better cache locality)
- Timestamp-based visibility check

### 3. MutableCSR (Edge Storage)

**File:** `csr/mutable_csr.cc`

```cpp
template <typename EDATA_T>
class MutableCsr {
    IDataContainer* nbr_list_;        // Contiguous neighbor array
    IDataContainer* degree_list_;     // Degree per vertex
    IDataContainer* cap_list_;        // Capacity per vertex
    IDataContainer* adj_list_buffer_; // Pointer array to neighbors
    SpinLock* locks_;                 // Per-vertex locks
    atomic<uint64_t> edge_num_;       // Total edge count

    // Core operations
    void batch_put_edges(vector<vid_t> src, vector<vid_t> dst,
                        vector<EDATA_T> data);
    void compact();  // Remove deleted edges
};
```

**CSR Structure:**

```
Vertex 0: [nbr_0, nbr_1, nbr_2, ...]  degree=3, capacity=5
Vertex 1: [nbr_3, nbr_4]               degree=2, capacity=2
Vertex 2: [nbr_5, nbr_6, nbr_7]        degree=3, capacity=4
...

adj_list_buffer_: [ptr0, ptr1, ptr2, ...]  // Pointers to each vertex's neighbors
degree_list_:     [3, 2, 3, ...]           // Actual edge count
cap_list_:        [5, 2, 4, ...]           // Allocated capacity
nbr_list_:        [all neighbors contiguous] // Actual neighbor data
```

**Key Design Points:**

| Feature       | Implementation                    |
| ------------- | --------------------------------- |
| Memory Layout | Contiguous neighbor storage       |
| Concurrency   | Per-vertex spin locks             |
| MVCC          | Timestamp in each neighbor entry  |
| GC            | `compact()` removes deleted edges |

**Compact Operation:**

```cpp
void MutableCsr::compact() {
    for each vertex:
        read_ptr = write_ptr = start of neighbors
        while read_ptr != end:
            if read_ptr->timestamp != INVALID_TIMESTAMP:
                *write_ptr = *read_ptr  // Keep valid edge
                write_ptr++
            else:
                removed++  // Skip deleted edge
        degree[vertex] -= removed
}
```

**Applicable to GraphDB:**

- CSR is more memory-efficient than separate index entries
- Better cache locality for graph traversals
- Compact operation for physical deletion (vs. tombstones)

### 4. PropertyGraph (Top-Level Graph)

**File:** `graph/property_graph.cc`

```cpp
class PropertyGraph {
    vector<VertexTable> vertex_tables_;     // Per label
    map<size_t, EdgeTable> edge_tables_;    // Key: (src_label, dst_label, edge_label)
    Schema schema_;

    // Schema management
    Status CreateVertexType(const CreateVertexTypeParam& config);
    Status CreateEdgeType(const CreateEdgeTypeParam& config);

    // Data operations
    Status BatchAddVertices(label_t v_label, shared_ptr<IRecordBatchSupplier> supplier);
    Status BatchAddEdges(label_t src_v_label, label_t dst_v_label, label_t e_label,
                        shared_ptr<IRecordBatchSupplier> supplier);
};
```

**Edge Table Indexing:**

```cpp
size_t index = schema_.generate_edge_label(src_label, dst_label, edge_label);
// Combines three labels into single index
```

**Applicable to GraphDB:**

- Per-label vertex tables (similar to tag-based separation)
- Edge tables keyed by (src_label, dst_label, edge_label)
- Schema-driven storage

## Comparison: GraphDB vs Neug

| Aspect            | GraphDB                    | Neug                                  |
| ----------------- | -------------------------- | ------------------------------------- |
| **Index Storage** | BTreeMap in memory         | MMapContainer (file-backed)           |
| **Vertex Index**  | Forward + Reverse BTreeMap | Indexer (hash map) + Table (columnar) |
| **Edge Storage**  | Forward + Reverse BTreeMap | CSR (Compressed Sparse Row)           |
| **MVCC**          | Timestamp in IndexEntry    | VertexTimestamp + edge timestamp      |
| **GC Strategy**   | Tombstone + incremental GC | Compact operation                     |
| **Persistence**   | Flush to binary files      | Memory-mapped files                   |
| **Concurrency**   | RwLock on whole index      | Per-vertex spin locks                 |

## Recommendations for GraphDB

### 1. Consider CSR for Edge Storage

**Current GraphDB:**

```rust
forward_index: BTreeMap<IndexKey, IndexEntry>  // Per-edge entries
reverse_index: BTreeMap<IndexKey, IndexEntry>  // For deletion
```

**Neug CSR:**

```cpp
nbr_list_: contiguous array of neighbors
degree_list_: degree per vertex
```

**Benefits:**

- 50-70% memory reduction for dense graphs
- Better cache locality for traversals
- Simpler deletion (just update degree)

### 2. Add Memory-Mapped Persistence

**Current GraphDB:**

```rust
fn flush(&self, path: &Path) {
    // Manual binary write
}
```

**Neug:**

```cpp
void Open(const std::string& path) {
    mmap_data_ = mmap(path, size);
    // Direct memory access to file
}
```

**Benefits:**

- Instant startup (no load time)
- OS handles paging
- Automatic persistence

### 3. Per-Vertex Locking

**Current GraphDB:**

```rust
Arc<RwLock<BTreeMap<IndexKey, IndexEntry>>>  // Single lock for whole index
```

**Neug:**

```cpp
SpinLock* locks_;  // One lock per vertex
```

**Benefits:**

- Higher concurrency for multi-threaded writes
- Reduced lock contention

### 4. Columnar Property Storage

**Current GraphDB:** Properties stored with index entries

**Neug:**

```cpp
Table* table_;  // Columnar storage for properties
```

**Benefits:**

- Better compression for repeated values
- Efficient scans on single property
- Better cache utilization

## Implementation Priority

| Priority | Feature                   | Effort | Impact |
| -------- | ------------------------- | ------ | ------ |
| High     | Memory-mapped persistence | Medium | High   |
| High     | Per-vertex locking        | Low    | Medium |
| Medium   | CSR for edge storage      | High   | High   |
| Low      | Columnar property storage | High   | Medium |

## Code References

| Component    | Neug File                     | GraphDB Equivalent        |
| ------------ | ----------------------------- | ------------------------- |
| Container    | `container/mmap_container.cc` | N/A (could add)           |
| Vertex Table | `graph/vertex_table.cc`       | `vertex_index_manager.rs` |
| Edge Table   | `graph/edge_table.cc`         | `edge_index_manager.rs`   |
| CSR          | `csr/mutable_csr.cc`          | N/A (could add)           |
| Schema       | `graph/schema.cc`             | `metadata/schema.rs`      |
