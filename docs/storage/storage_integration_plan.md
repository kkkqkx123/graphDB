# Storage Module Integration Plan

## 1. Overview

This document details the integration plan for the storage module, addressing the gaps between the transaction layer and storage layer identified in the architecture analysis.

### 1.1 Current Status

| Component | Implementation Status | Integration Status |
|-----------|----------------------|-------------------|
| CSR Edge Storage | ✅ Complete | ✅ Integrated |
| Column Vertex Storage | ✅ Complete | ✅ Integrated |
| MVCC Version Manager | ✅ Complete | ❌ Not Integrated |
| WAL Module | ✅ Complete | ❌ Not Integrated |
| Undo Log | ✅ Complete | ❌ Not Integrated |
| mmap Container | ✅ Complete | ❌ Not Used |

### 1.2 Core Issues

1. **Transaction-Storage Disconnect**: `InsertTransaction` and `UpdateTransaction` use `InsertTarget`/`UndoTarget` traits, but `PropertyGraph` doesn't implement them
2. **No WAL Integration**: Data changes don't persist to WAL, preventing crash recovery
3. **No Persistence**: `PropertyGraph` is purely in-memory, data lost on restart
4. **Incomplete Entity Adapters**: `VertexStorage`/`EdgeStorage` use hardcoded timestamps

---

## 2. File Changes Summary

### 2.1 Files to Modify

| File | Changes |
|------|---------|
| `src/storage/property_graph.rs` | Implement `InsertTarget`, `UndoTarget` traits; add WAL support |
| `src/storage/entity/vertex_storage.rs` | Fix timestamp acquisition from VersionManager |
| `src/storage/entity/edge_storage.rs` | Fix timestamp acquisition from VersionManager |
| `src/storage/graph_storage.rs` | Add WAL writer initialization |
| `src/storage/shared_state.rs` | Add WAL writer reference |
| `src/storage/vertex/vertex_table.rs` | Add persistence methods |
| `src/storage/edge/edge_table.rs` | Add persistence methods |
| `src/storage/mod.rs` | Export new types |

### 2.2 Files to Create

| File | Purpose |
|------|---------|
| `src/storage/iterator/vertex_iter.rs` | CSR-optimized vertex iterator |
| `src/storage/iterator/edge_iter.rs` | CSR-optimized edge iterator |

### 2.3 Files to Delete

None. All existing files are needed.

---

## 3. Phase 1: Transaction Trait Implementation (P0)

### 3.1 Objective

Make `PropertyGraph` implement `InsertTarget` and `UndoTarget` traits to enable transaction operations.

### 3.2 Implementation Details

#### 3.2.1 InsertTarget Implementation

```rust
// In src/storage/property_graph.rs

use crate::transaction::insert_transaction::{InsertTarget, InsertTransactionResult};
use crate::transaction::wal::types::{LabelId, VertexId, EdgeId, Timestamp};

impl InsertTarget for PropertyGraph {
    fn add_vertex(
        &mut self,
        label: LabelId,
        oid: &[u8],
        properties: &[(String, Vec<u8>)],
        ts: Timestamp,
    ) -> InsertTransactionResult<VertexId> {
        let external_id = std::str::from_utf8(oid)
            .map_err(|e| InsertTransactionError::SerializationError(e.to_string()))?;
        
        let props: Vec<(String, Value)> = properties
            .iter()
            .map(|(k, v)| {
                let value = deserialize_value(v)?;
                Ok((k.clone(), value))
            })
            .collect::<Result<_, _>>()?;
        
        let internal_id = self.insert_vertex(label, external_id, &props, ts)
            .map_err(|e| InsertTransactionError::SchemaError(e.to_string()))?;
        
        Ok(internal_id as VertexId)
    }

    fn add_edge(
        &mut self,
        src_label: LabelId,
        src_vid: VertexId,
        dst_label: LabelId,
        dst_vid: VertexId,
        edge_label: LabelId,
        properties: &[(String, Vec<u8>)],
        ts: Timestamp,
    ) -> InsertTransactionResult<EdgeId> {
        // Similar implementation
    }
}
```

#### 3.2.2 UndoTarget Implementation

```rust
// In src/storage/property_graph.rs

use crate::transaction::undo_log::{UndoTarget, UndoLogResult, PropertyValue};

impl UndoTarget for PropertyGraph {
    fn delete_vertex_type(&mut self, label: LabelId) -> UndoLogResult<()> {
        self.vertex_tables.remove(&label);
        Ok(())
    }

    fn delete_edge_type(&mut self, src_label: LabelId, dst_label: LabelId, edge_label: LabelId) -> UndoLogResult<()> {
        self.edge_tables.remove(&(src_label, dst_label, edge_label));
        Ok(())
    }

    fn delete_vertex(&mut self, label: LabelId, vid: VertexId, ts: Timestamp) -> UndoLogResult<()> {
        if let Some(table) = self.vertex_tables.get_mut(&label) {
            table.delete_by_internal_id(vid as u32, ts)
                .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        }
        Ok(())
    }

    fn undo_update_vertex_property(
        &mut self,
        label: LabelId,
        vid: VertexId,
        col_id: ColumnId,
        value: PropertyValue,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        if let Some(table) = self.vertex_tables.get_mut(&label) {
            let value = property_value_to_value(value);
            table.update_property_by_col_id(vid as u32, col_id, &value, ts)
                .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        }
        Ok(())
    }
    
    // ... other methods
}
```

### 3.3 Test Requirements

- Unit tests for each trait method
- Integration test: InsertTransaction -> PropertyGraph -> verify data
- Integration test: UpdateTransaction abort -> verify rollback

---

## 4. Phase 2: WAL Integration (P0)

### 4.1 Objective

Integrate WAL writer into PropertyGraph for durability guarantees.

### 4.2 Implementation Details

#### 4.2.1 PropertyGraph WAL Support

```rust
// In src/storage/property_graph.rs

use std::sync::Arc;
use parking_lot::RwLock;
use crate::transaction::wal::writer::WalWriter;

pub struct PropertyGraph {
    vertex_tables: HashMap<LabelId, VertexTable>,
    edge_tables: HashMap<(LabelId, LabelId, LabelId), EdgeTable>,
    // ... existing fields
    
    // NEW: WAL writer for durability
    wal_writer: Option<Arc<RwLock<Box<dyn WalWriter>>>>,
    wal_enabled: bool,
}

impl PropertyGraph {
    pub fn with_wal(mut self, wal_writer: Arc<RwLock<Box<dyn WalWriter>>>) -> Self {
        self.wal_writer = Some(wal_writer);
        self.wal_enabled = true;
        self
    }

    pub fn set_wal_writer(&mut self, wal_writer: Arc<RwLock<Box<dyn WalWriter>>>) {
        self.wal_writer = Some(wal_writer);
        self.wal_enabled = true;
    }

    fn write_wal(&self, op_type: WalOpType, data: &[u8]) -> StorageResult<()> {
        if !self.wal_enabled {
            return Ok(());
        }
        
        if let Some(wal) = &self.wal_writer {
            let mut writer = wal.write();
            writer.append(data)
                .map_err(|e| StorageError::WalError(e.to_string()))?;
        }
        Ok(())
    }
}
```

#### 4.2.2 GraphStorage WAL Initialization

```rust
// In src/storage/graph_storage.rs

use crate::transaction::wal::writer::LocalWalWriter;

impl GraphStorage {
    pub fn new_with_path(path: PathBuf) -> StorageResult<Self> {
        let config = PropertyGraphConfig {
            work_dir: path.clone(),
            ..Default::default()
        };
        
        // Initialize WAL
        let wal_dir = path.join("wal");
        let wal_writer = Arc::new(RwLock::new(
            Box::new(LocalWalWriter::new(wal_dir.to_str().unwrap(), 0)) as Box<dyn WalWriter>
        ));
        
        let mut graph = PropertyGraph::with_config(config);
        graph.set_wal_writer(wal_writer.clone());
        
        let graph = Arc::new(RwLock::new(graph));
        // ... rest of initialization
    }
}
```

### 4.3 Test Requirements

- Test WAL write on vertex insert
- Test WAL write on edge insert
- Test WAL sync to disk
- Test crash recovery from WAL

---

## 5. Phase 3: Entity Adapter Fixes (P1)

### 5.1 Objective

Fix entity adapters to properly integrate with VersionManager for MVCC.

### 5.2 Implementation Details

#### 5.2.1 VertexStorage Fix

```rust
// In src/storage/entity/vertex_storage.rs

impl VertexStorage {
    fn get_read_timestamp(&self) -> Timestamp {
        // FIXED: Get from VersionManager
        self.version_manager.get_current_read_timestamp()
    }

    fn get_write_timestamp(&self) -> Timestamp {
        // FIXED: Acquire from VersionManager
        self.version_manager.acquire_insert_timestamp()
            .unwrap_or(INVALID_TIMESTAMP - 1)
    }
}
```

#### 5.2.2 EdgeStorage Fix

```rust
// In src/storage/entity/edge_storage.rs

impl EdgeStorage {
    fn get_read_timestamp(&self) -> Timestamp {
        self.version_manager.get_current_read_timestamp()
    }

    fn get_write_timestamp(&self) -> Timestamp {
        self.version_manager.acquire_insert_timestamp()
            .unwrap_or(INVALID_TIMESTAMP - 1)
    }
}
```

### 5.3 Test Requirements

- Test MVCC snapshot isolation
- Test concurrent read transactions
- Test read-write conflict handling

---

## 6. Phase 4: Persistence Implementation (P1)

### 6.1 Objective

Implement data persistence using mmap containers.

### 6.2 Implementation Details

#### 6.2.1 VertexTable Persistence

```rust
// In src/storage/vertex/vertex_table.rs

use crate::storage::container::MmapContainer;

impl VertexTable {
    pub fn flush(&self) -> StorageResult<()> {
        // Write columns to mmap container
        // Write timestamps to mmap container
        // Write id_indexer to mmap container
        Ok(())
    }

    pub fn load(&mut self, path: &Path) -> StorageResult<()> {
        // Load from mmap container
        Ok(())
    }
}
```

#### 6.2.2 PropertyGraph Persistence

```rust
// In src/storage/property_graph.rs

impl PropertyGraph {
    pub fn flush(&self) -> StorageResult<()> {
        for table in self.vertex_tables.values() {
            table.flush()?;
        }
        for table in self.edge_tables.values() {
            table.flush()?;
        }
        
        // Sync WAL
        if let Some(wal) = &self.wal_writer {
            wal.read().sync()
                .map_err(|e| StorageError::WalError(e.to_string()))?;
        }
        
        Ok(())
    }

    pub fn checkpoint(&mut self) -> StorageResult<()> {
        // 1. Flush all data
        self.flush()?;
        
        // 2. Create checkpoint marker in WAL
        // 3. Clear old WAL entries
        
        Ok(())
    }
}
```

### 6.3 Test Requirements

- Test flush to disk
- Test load from disk
- Test restart recovery

---

## 7. Phase 5: CSR Iterators (P2)

### 7.1 Objective

Implement optimized iterators for CSR edge traversal.

### 7.2 New Files

#### 7.2.1 edge_iter.rs

```rust
// src/storage/iterator/edge_iter.rs

use crate::storage::edge::{MutableCsr, EdgeRecord, VertexId, Timestamp};

pub struct CsrEdgeIterator<'a> {
    csr: &'a MutableCsr,
    current_vertex: usize,
    current_edge_idx: usize,
    ts: Timestamp,
}

impl<'a> Iterator for CsrEdgeIterator<'a> {
    type Item = EdgeRecord;

    fn next(&mut self) -> Option<Self::Item> {
        // O(d) complexity traversal
        loop {
            let edges = self.csr.edges_of(self.current_vertex as VertexId);
            
            if self.current_edge_idx < edges.len() {
                let edge = edges.nth(self.current_edge_idx)?;
                self.current_edge_idx += 1;
                
                // Check MVCC visibility
                if edge.is_visible(self.ts) {
                    return Some(edge);
                }
            } else {
                self.current_vertex += 1;
                self.current_edge_idx = 0;
                
                if self.current_vertex >= self.csr.vertex_capacity() {
                    return None;
                }
            }
        }
    }
}
```

#### 7.2.2 vertex_iter.rs

```rust
// src/storage/iterator/vertex_iter.rs

use crate::storage::vertex::{VertexTable, VertexRecord, Timestamp};

pub struct VertexIterator<'a> {
    table: &'a VertexTable,
    current_idx: u32,
    ts: Timestamp,
}

impl<'a> Iterator for VertexIterator<'a> {
    type Item = VertexRecord;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_idx < self.table.len() as u32 {
            if let Some(record) = self.table.get_by_internal_id(self.current_idx, self.ts) {
                self.current_idx += 1;
                return Some(record);
            }
            self.current_idx += 1;
        }
        None
    }
}
```

### 7.3 Test Requirements

- Test O(d) edge traversal complexity
- Test MVCC visibility filtering
- Benchmark vs. full scan

---

## 8. Implementation Order

```
Week 1: Phase 1 (Transaction Traits)
├── Day 1-2: InsertTarget implementation
├── Day 3-4: UndoTarget implementation
└── Day 5: Integration tests

Week 2: Phase 2 (WAL Integration)
├── Day 1-2: PropertyGraph WAL support
├── Day 3: GraphStorage initialization
└── Day 4-5: Recovery tests

Week 3: Phase 3-4 (Entity Adapters + Persistence)
├── Day 1-2: VertexStorage/EdgeStorage fixes
├── Day 3-4: Persistence implementation
└── Day 5: Integration tests

Week 4: Phase 5 (CSR Iterators)
├── Day 1-2: Edge iterator
├── Day 3: Vertex iterator
├── Day 4: Performance benchmarks
└── Day 5: Documentation
```

---

## 9. Success Criteria

| Criterion | Verification Method |
|-----------|-------------------|
| Transaction operations work | InsertTransaction can insert vertices/edges |
| Rollback works | UpdateTransaction abort restores state |
| WAL durability | Crash recovery restores data |
| MVCC isolation | Concurrent reads see consistent snapshots |
| Persistence | Data survives restart |
| Performance | Edge traversal is O(d), not O(E) |
