# CSR-Aware Index Improvement Plan

## 1. Background

The current `src/storage/index` module uses a generic BTreeMap-based index implementation that is decoupled from the CSR (Compressed Sparse Row) storage layer. This design creates several inefficiencies:

1. **Storage Model Mismatch**: CSR uses contiguous arrays with offsets, while indexes use BTreeMap key-value pairs
2. **Loose Integration**: Indexes and CSR data are maintained separately, requiring manual consistency management
3. **Inconsistent MVCC**: Different timestamp mechanisms between CSR and indexes
4. **Type Conversion Overhead**: Indexes use `Value` type while CSR uses native `VertexId/EdgeId`

## 2. Index Classification

### 2.1 Primary Indexes (CSR-Aware)

Primary indexes are tightly coupled with CSR storage structure and provide fast access to data by internal IDs.

| Index         | Description                                 | MVCC | GC Required |
| ------------- | ------------------------------------------- | ---- | ----------- |
| `EdgeIdIndex` | Maps `edge_id -> (src, dst, prop_offset)`   | No   | No          |
| `DegreeIndex` | Maps `vertex_id -> (out_degree, in_degree)` | No   | No          |

**Characteristics**:

- Native ID types (u64) for maximum performance
- No MVCC overhead (always consistent with CSR)
- Automatically maintained during DML operations
- No tombstone GC required

### 2.2 Secondary Indexes (Property Indexes)

Secondary indexes support complex property-based queries and use MVCC for snapshot isolation.

| Index                | Description                | MVCC | GC Required |
| -------------------- | -------------------------- | ---- | ----------- |
| `VertexIndexManager` | Index on vertex properties | Yes  | Yes         |
| `EdgeIndexManager`   | Index on edge properties   | Yes  | Yes         |

**Characteristics**:

- Support MVCC for snapshot isolation
- BTreeMap-based for range queries
- Support tombstone GC for deleted entries
- Managed by `IndexGcManager`

### 2.3 Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    Index Architecture                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────────────────┐    ┌──────────────────────┐   │
│  │   Primary Indexes        │    │   Secondary Indexes  │   │
│  │   (CSR-Aware)            │    │   (Property)         │   │
│  ├──────────────────────────┤    ├──────────────────────┤   │
│  │ • EdgeIdIndex            │    │ • VertexIndexManager │   │
│  │ • DegreeIndex            │    │ • EdgeIndexManager   │   │
│  │                          │    │                      │   │
│  │ • Native ID types (u64)  │    │ • Value type         │   │
│  │ • No MVCC                │    │ • MVCC support       │   │
│  │ • No GC required         │    │ • GC required        │   │
│  └──────────────────────────┘    └──────────────────────┘   │
│           │                              │                   │
│           │                              ▼                   │
│           │                    ┌──────────────────────┐     │
│           │                    │   IndexGcManager     │     │
│           │                    │   (Secondary only)   │     │
│           │                    └──────────────────────┘     │
│           ▼                              │                   │
│  ┌──────────────────────────────────────────────────────┐   │
│  │                  IndexDataManager                     │   │
│  │                  (Unified Interface)                  │   │
│  └──────────────────────────────────────────────────────┘   │
│                              │                               │
│                              ▼                               │
│  ┌──────────────────────────────────────────────────────┐   │
│  │              EdgeStorage / VertexStorage              │   │
│  │              (Entity Layer)                           │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## 3. Improvement Goals

- Unify ID types between index and CSR layers
- Align MVCC timestamp mechanisms
- Add CSR-aware index structures for better performance
- Integrate indexes directly with EdgeTable/VertexTable
- Clear separation between Primary and Secondary indexes

## 4. Phase 1: Type and MVCC Unification (Completed)

### 3.1 Unified ID Types

**Problem**: Index keys serialize `Value` type, while CSR uses native `u64` for vertex/edge IDs.

**Solution**: Use native `VertexId` and `EdgeId` types in index keys.

**Files to Modify**:

- `src/storage/index/index_key_codec.rs`
- `src/storage/index/vertex_index_manager.rs`
- `src/storage/index/edge_index_manager.rs`
- `src/storage/index/index_data_manager.rs`

**Changes**:

```rust
// Before
pub fn build_vertex_index_key(
    space_id: u64,
    index_name: &str,
    prop_value: &Value,
    vertex_id: &Value,  // Uses Value type
) -> Result<ByteKey, StorageError>

// After
pub fn build_vertex_index_key(
    space_id: u64,
    index_name: &str,
    prop_value: &Value,
    vertex_id: VertexId,  // Uses native VertexId (u64)
) -> Result<ByteKey, StorageError>
```

### 3.2 Unified MVCC Timestamps

**Problem**: CSR uses `Nbr.timestamp` while indexes use `IndexEntry { created_ts, deleted_ts }`.

**Solution**: Create a shared timestamp source and ensure consistent visibility checks.

**Files to Modify**:

- `src/storage/index/index_data_manager.rs`
- `src/storage/edge/mutable_csr.rs`
- `src/storage/vertex/vertex_table.rs`

**Changes**:

1. Add shared `TimestampSource` trait
2. Ensure both CSR and indexes use the same `VersionManager`
3. Add consistency validation in transaction commit

### 3.3 Implementation Tasks

| Task                            | File                      | Description                              |
| ------------------------------- | ------------------------- | ---------------------------------------- |
| Update `IndexKeyCodec`          | `index_key_codec.rs`      | Use `VertexId/EdgeId` instead of `Value` |
| Update `VertexIndexManager`     | `vertex_index_manager.rs` | Accept `VertexId` parameters             |
| Update `EdgeIndexManager`       | `edge_index_manager.rs`   | Accept `VertexId/EdgeId` parameters      |
| Update `IndexDataManager` trait | `index_data_manager.rs`   | Update trait signatures                  |
| Update callers                  | `entity/*.rs`             | Adapt to new API                         |

## 5. Phase 2: CSR-Aware Index Structures (Completed)

### 4.1 Edge ID Index

**Purpose**: Fast lookup of edge by `edge_id` to get `(src, dst)` pair.

**Design**:

```rust
/// CSR-aware edge ID index
pub struct EdgeIdIndex {
    /// Maps edge_id -> (src_vid, dst_vid, prop_offset)
    index: DashMap<EdgeId, (VertexId, VertexId, u32)>,
}

impl EdgeIdIndex {
    pub fn insert(&self, edge_id: EdgeId, src: VertexId, dst: VertexId, prop_offset: u32);
    pub fn get(&self, edge_id: EdgeId) -> Option<(VertexId, VertexId, u32)>;
    pub fn remove(&self, edge_id: EdgeId);
}
```

**Integration**: Add to `EdgeTable` as a built-in index.

### 4.2 Vertex Degree Index

**Purpose**: Fast degree queries and degree-based filtering.

**Design**:

```rust
/// Vertex degree index for CSR
pub struct DegreeIndex {
    /// Maps vertex_id -> (out_degree, in_degree)
    degrees: DashMap<VertexId, (u32, u32)>,
}
```

### 4.3 Property Offset Index

**Purpose**: Fast property lookup by `prop_offset`.

**Design**:

```rust
/// Property offset index for edge properties
pub struct PropertyOffsetIndex {
    /// Maps prop_offset -> property values
    properties: Vec<HashMap<String, Value>>,
}
```

### 4.4 Implementation Tasks

| Task                 | File               | Description                 |
| -------------------- | ------------------ | --------------------------- |
| Create `EdgeIdIndex` | `edge_id_index.rs` | New file for edge ID index  |
| Create `DegreeIndex` | `degree_index.rs`  | New file for degree index   |
| Add to `EdgeTable`   | `edge_table.rs`    | Integrate CSR-aware indexes |
| Update `mod.rs`      | `index/mod.rs`     | Export new types            |

## 6. Phase 3: Deep Integration (Completed)

### 5.1 Index Integration in EdgeTable

**Current State**: Indexes maintained separately in `entity/edge_storage.rs`.

**Target State**: Indexes integrated into `EdgeTable` with automatic maintenance.

**Design**:

```rust
pub struct EdgeTable {
    label: LabelId,
    label_name: String,
    src_label: LabelId,
    dst_label: LabelId,
    schema: EdgeSchema,
    out_csr: MutableCsr,
    in_csr: MutableCsr,
    properties: PropertyTable,
    edge_id_counter: AtomicU64,
    config: EdgeTableConfig,
    is_open: bool,

    // New: CSR-aware indexes
    edge_id_index: EdgeIdIndex,
    property_indexes: HashMap<String, PropertyIndex>,
}

impl EdgeTable {
    pub fn insert_edge(...) -> StorageResult<EdgeId> {
        // ... existing logic ...

        // Auto-update indexes
        self.edge_id_index.insert(edge_id, src, dst, prop_offset);
        for (name, index) in &self.property_indexes {
            index.insert(edge_id, prop_offset, &property_values);
        }

        Ok(edge_id)
    }
}
```

### 5.2 Index Integration in VertexTable

**Design**: Similar integration for vertex property indexes.

### 5.3 Consistency Guarantees

1. **Atomic Updates**: Index updates happen within the same write lock as CSR updates
2. **Rollback Support**: Index changes can be rolled back via undo log
3. **Crash Recovery**: Indexes rebuilt from CSR on startup if needed

### 5.4 Implementation Tasks

| Task                                | File              | Description                 |
| ----------------------------------- | ----------------- | --------------------------- |
| Add `EdgeIdIndex` to `EdgeTable`    | `edge_table.rs`   | Integrate edge ID index     |
| Add property indexes to `EdgeTable` | `edge_table.rs`   | Integrate property indexes  |
| Add indexes to `VertexTable`        | `vertex_table.rs` | Similar integration         |
| Update `entity/*.rs`                | `entity/*.rs`     | Remove manual index updates |
| Add consistency checks              | `edge_table.rs`   | Validate index consistency  |

## 6. Migration Path

### 6.1 Backward Compatibility

- Phase 1 changes are API-compatible (internal type changes)
- Phase 2 adds new optional indexes
- Phase 3 changes internal architecture but maintains external API

### 6.2 Testing Strategy

1. **Unit Tests**: Test each new index type independently
2. **Integration Tests**: Test index-CSR consistency
3. **Performance Tests**: Benchmark query improvements
4. **Migration Tests**: Test upgrade from old format

## 7. Success Metrics

| Metric                | Current                  | Target                     |
| --------------------- | ------------------------ | -------------------------- |
| Edge lookup by ID     | O(n) scan                | O(1) hash lookup           |
| Index update overhead | Separate BTreeMap update | Integrated with CSR update |
| Type conversion       | Value serialization      | Native u64                 |
| MVCC consistency      | Manual sync              | Automatic sync             |

## 8. Timeline

| Phase                          | Duration | Priority |
| ------------------------------ | -------- | -------- |
| Phase 1: Type/MVCC Unification | 2-3 days | High     |
| Phase 2: CSR-Aware Indexes     | 3-5 days | Medium   |
| Phase 3: Deep Integration      | 5-7 days | Medium   |

## 9. References

- [Index Module Analysis](./index_module_analysis.md)
- [Index Improvement Plan](../index_improvement_plan.md)
- [MVCC GC Research](../mvcc_gc_research.md)
- [Neug Reference Analysis](./neug_reference_analysis.md)
