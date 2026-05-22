# Storage Module Analysis

## Module Overview

```
src/storage/
├── mod.rs                          # Module declarations + re-exports
├── storage_client.rs              # StorageReader/Writer/SchemaOps/AuthOps/Admin/Client traits
├── storage_types.rs               # PropertyId, EdgeOffset, ColumnDef, FieldDef, etc.
├── compression.rs                 # CompressionType (None/Zstd)
├── metrics.rs                     # StorageMetrics wrapper (latency/success recording)
├── test_mock.rs                   # MockStorage for testing

├── cache/                         # Moka-backed concurrent cache (vertex + ID index)
├── container/                     # mmap container: PersistentContainer + VolatileContainer
├── edge/                          # CSR edge storage (Csr/MutableCsr/EdgeTable/PropertyTable)
├── engine/                        # Storage engine core (see below)
├── extend/                        # BM25 fulltext index
├── index/                         # Primary indexes + secondary BTreeMap indexes with MVCC/GC
├── metadata/                      # SchemaManager + IndexMetadataManager
├── utils/                         # NameIndexer + persistence encoding helpers
├── vertex/                        # Columnar vertex storage (VertexTable + IdIndexer + encoding)
```

### Engine Internal Structure

```
engine/
├── data_store.rs                  # GraphDataStore (all vertex/edge tables + label maps)
├── property_graph/                # PropertyGraph (low-level graph facade)
│   ├── mod.rs                     # Struct + all public API methods
│   ├── core_ops.rs                # Vertex/edge CRUD free functions
│   ├── type_ops.rs                # Schema type DDL free functions
│   ├── flush.rs                   # Persistence load/save free functions
│   └── index_mvcc.rs              # Index MVCC update/delete free functions
├── graph_storage/                 # StorageClient trait implementation
│   ├── mod.rs                     # GraphStorage struct + all trait impls
│   ├── context.rs                 # GraphStorageContext (shared references)
│   ├── reader.rs                  # GraphStorageReader
│   ├── writer.rs                  # GraphStorageWriter
│   ├── schema_adapter.rs          # SchemaAdapterOps
│   ├── persistence.rs             # PersistenceOps (save/restore/checkpoint)
│   ├── maintenance.rs             # MaintenanceOps (stats, dangling edges)
│   ├── index_manager.rs           # IndexManagerOps (index CRUD + lookup)
│   ├── transactional_writer.rs    # TransactionalWriter (WAL-batched writes)
│   ├── transaction_support.rs     # with_rollback/execute_in_transaction helpers
│   ├── type_utils.rs              # Record-to-domain-object conversion
│   ├── user_ops.rs                # User/role management
│   └── transaction_config.rs      # DurabilityLevel, IsolationLevel
├── transaction/                   # Transaction undo/compact/recovery targets
├── persistence_coordinator.rs     # WAL → Flush → Checkpoint → Snapshot chain
├── wal_manager.rs                 # WAL manager (LSN tracking)
├── snapshot_manager.rs            # Multi-version snapshot management
├── cache_manager.rs               # CacheManager wrapper
├── config.rs                      # PropertyGraphConfig, FlushConfig
├── batch.rs                       # Batch import reader/writer
├── query.rs                       # Query helpers (scan_vertices, vertex_count)
├── sync_wrapper.rs                # SyncWrapper (push changes to external index)
└── edge_params.rs                 # Edge operation parameter types
```

## Data Flow

```
API / Query / Tests
    ↓ StorageClient trait (6 unified traits)
GraphStorage ────────────────────── adapter layer
    ↓ delegates to Ops objects
GraphStorageReader/Writer + PersistenceOps + SchemaAdapterOps + IndexManagerOps + ...
    ↓
PropertyGraph ───────────────────── low-level facade
    ↓ delegates to free functions
core_ops + type_ops + flush + index_mvcc
    ↓
GraphDataStore
    ↓
VertexTable / EdgeTable
    ↓
container (mmap) / index (BTreeMap/DashMap)
```

## Issues Identified

### P0 — Encapsulation & Access Control

#### Issue 1: PropertyGraph fields are unnecessarily public

`PropertyGraph` exposes almost all internal fields as `pub` or `pub(crate)`:

```rust
pub struct PropertyGraph {
    pub data_store: GraphDataStore,                              // pub
    pub(crate) cache_manager: CacheManager,                      // pub(crate)
    pub(crate) wal_manager: Mutex<WalManager>,                   // pub(crate)
    pub(crate) table_tracker: Arc<TableTracker>,                 // pub(crate)
    pub(crate) config: PropertyGraphConfig,                      // pub(crate)
    pub(crate) is_open: AtomicBool,                              // pub(crate)
    pub(crate) last_compacted_vertices: Mutex<...>,              // pub(crate)
    pub(crate) index_data_manager: RwLock<IndexDataManagerImpl>, // pub(crate)
}
```

`GraphDataStore` is even worse — all fields are `pub`:

```rust
pub struct GraphDataStore {
    pub vertex_tables: RwLock<HashMap<LabelId, VertexTable>>,
    pub edge_tables: RwLock<HashMap<(LabelId, LabelId, LabelId), EdgeTable>>,
    pub vertex_label_names: RwLock<HashMap<String, LabelId>>,
    pub edge_label_names: RwLock<HashMap<String, LabelId>>,
    pub vertex_label_counter: RwLock<LabelId>,
    pub edge_label_counter: RwLock<LabelId>,
}
```

This means `core_ops.rs`, `type_ops.rs`, `flush.rs`, etc. — all free functions —
directly read and write internal fields through `self.data_store.vertex_tables.write()`
etc., with zero access control. Any future refactoring of internal storage
representation (e.g., replacing `HashMap` with `BTreeMap` or `DashMap`, or changing
the edge table key structure) will ripple across all these call sites.

### P0 — Interface Bloat

#### Issue 2: GraphStorage hybrid trait impl + public methods

`GraphStorage` implements 5 traits (~80 methods) AND adds 30+ extra public methods
not declared in any trait:

- `GraphStorage::flush()`, `create_checkpoint()`, `compact_all()`
- `recover_from_wal()`, `recover_from_wal_with_config()`
- `init_with_recovery()`, `needs_recovery()`
- `is_index_gc_running()`, `start_index_gc()`, `stop_index_gc()`
- `save_data()`, `save_data_to_dir()`
- `auto_flush_if_needed()`, `auto_checkpoint_if_needed()`
- etc.

Downstream code that accepts `Box<dyn StorageClient>` cannot call these methods.
They must downcast or access `GraphStorage` directly, which defeats the purpose
of the trait abstraction. The trait is the public contract, but the real API is
much larger.

### P0 — Pattern Inconsistency

#### Issue 3: Temporary delegation objects created per call

Every trait method in `GraphStorage` creates a new temporary object:

```rust
impl StorageReader for GraphStorage {
    fn get_vertex(&self, space: &str, id: &VertexId) -> ... {
        reader::GraphStorageReader::new(&self.ctx).get_vertex(space, id)
    }
}

impl StorageWriter for GraphStorage {
    fn insert_vertex(&mut self, ...) {
        writer::GraphStorageWriter::new(&self.ctx).insert_vertex(...)
    }
}

impl StorageAdmin for GraphStorage {
    fn save_to_disk(&self) -> ... {
        persistence::PersistenceOps::new(&self.ctx).save_to_disk()
    }
    fn get_storage_stats(&self) -> ... {
        maintenance::MaintenanceOps::new(&self.ctx).get_storage_stats()
    }
}
```

`PersistenceOps`, `GraphStorageReader`, `GraphStorageWriter`, `SchemaAdapterOps`,
`IndexManagerOps`, `MaintenanceOps`, `UserOps` — all follow this pattern. None of
them hold any mutable state. They are pure facades that could instead be free
functions accepting `&GraphStorageContext`. The extra layer adds zero value while
increasing code volume and cognitive overhead.

### P1 — Architecture Ambiguity

#### Issue 4: Overlapping transaction subsystems

Two transaction-related abstractions exist with unclear boundaries:

- `engine/transaction/ops.rs`: `TransactionOps` trait — add/delete vertex/edge
  with undo support, targets for undo/compact/recovery targeting PropertyGraph.
- `engine/graph_storage/transactional_writer.rs`: `TransactionalWriter` —
  atomic inserts with WAL logging.
- `engine/graph_storage/transaction_support.rs`: `with_rollback` / `execute_in_transaction`.

Developers cannot easily determine which abstraction to use for a new operation.

#### Issue 5: Edge table indexed by (src_label, dst_label, edge_type) triple

```rust
edge_tables: RwLock<HashMap<(LabelId, LabelId, LabelId), EdgeTable>>
```

The same edge type connecting different src/dst label pairs lands in different
EdgeTables. This causes:

- **Data fragmentation**: Edge type "knows" connecting (Person→Person) and
  (Person→Organization) are stored separately.
- **High query cost**: `scan_edges_by_label()` iterates all tables and filters
  — O(n) instead of O(1).
- **Table proliferation**: Every new label combination creates a new table.

### P1 — Redundancy

#### Issue 6: SchemaManager accessible via multiple paths

```rust
// Path 1: Direct method
GraphStorage::get_schema_manager() -> Arc<SchemaManager>

// Path 2: Via trait
StorageAdmin::get_schema_manager() -> Option<Arc<SchemaManager>>

// Path 3: Via PropertyGraph → SchemaAdapterOps
// (SchemaAdapterOps holds &GraphStorageContext → accesses schema_manager field)
```

Multiple access paths make it harder to replace, wrap, or instrument `SchemaManager`.

#### Issue 7: MockStorage global wildcard re-export

```rust
#[cfg(test)]
pub use test_mock::*;
```

This floods the `crate::storage` module with all items from `test_mock.rs`.
While behind `#[cfg(test)]`, it still forces the compiler to parse the entire file
even when only some types are needed.

#### Issue 8: No edge cache under high read load

Design decision: "CSR is read-optimized, no edge cache needed." However,
hot edges read repeatedly still decompress properties from `PropertyTable` each time.
An optional edge record cache (keyed by edge identity) could improve QPS for hot edges.

### P2 — Naming

#### Issue 9: Ambiguous "encoding" module names

- `utils/encoding.rs`: Persistence file format (magic bytes, varint helpers, section headers)
- `vertex/encoding/`: Column compression algorithms (ALP, FSST, RLE, BitPacking, etc.)

Same name, completely different concerns. Confusing for new contributors.
