# GraphDB 存储与事务架构迁移方案

## 1. 概述

本文档基于 `neug_storage_migration_analysis.md` 分析结果，详细规划 `src/transaction` 和 `src/storage` 目录的迁移改造方案。

### 1.1 迁移目标

| 目标 | 描述 |
|------|------|
| 完全移除 redb | redb 作为通用 KV 存储，不适合图数据结构 |
| 采用 CSR 边存储 | O(d) 复杂度的边遍历，而非 O(E) 全表扫描 |
| 采用列式顶点存储 | 消除属性名重复存储，提升内存效率 |
| 实现 MVCC 时间戳 | 显式的时间戳控制，支持快照隔离 |
| 实现 WAL 持久化 | 自定义 WAL 文件，支持崩溃恢复 |
| 实现 Undo Log | 完整的事务回滚支持 |

### 1.2 当前架构分析

#### 事务模块 (src/transaction/)

| 文件 | 当前功能 | 迁移后状态 |
|------|----------|------------|
| `mod.rs` | 模块入口，创建 TransactionManager | **重写** - 移除 redb 依赖 |
| `context.rs` | TransactionContext，绑定 redb 事务 | **重写** - 采用 MVCC 时间戳 |
| `manager.rs` | TransactionManager，管理事务生命周期 | **重写** - 集成 VersionManager |
| `types.rs` | 事务类型定义 | **保留** - 扩展新类型 |
| `cleaner.rs` | 过期事务清理 | **保留** - 适配新架构 |
| `monitor.rs` | 事务监控指标 | **保留** - 无需修改 |
| `index_buffer.rs` | 索引更新缓冲 | **保留** - 适配新架构 |

#### 存储模块 (src/storage/)

| 目录/文件 | 当前功能 | 迁移后状态 |
|-----------|----------|------------|
| `engine/redb_storage.rs` | RedbStorage 主入口 | **删除** - 替换为 PropertyGraph |
| `engine/redb_types.rs` | redb 表定义 | **删除** - 替换为 CSR/列存储 |
| `engine/runtime_context.rs` | 运行时上下文 | **保留** - 适配新架构 |
| `entity/vertex_storage.rs` | 顶点存储 | **重写** - 采用列存储 |
| `entity/edge_storage.rs` | 边存储 | **重写** - 采用 CSR |
| `entity/user_storage.rs` | 用户存储 | **保留** - 独立模块 |
| `operations/redb/` | redb 读写操作 | **删除** - 替换为原生实现 |
| `operations/traits/` | 读写接口定义 | **保留** - 适配新接口 |
| `operations/rollback.rs` | 回滚执行器 | **重写** - 基于 Undo Log |
| `metadata/` | Schema 和索引元数据 | **保留** - 适配新存储 |
| `index/` | 索引数据管理 | **保留** - 适配新存储 |
| `iterator/` | 存储迭代器 | **重写** - CSR 迭代器 |
| `shared_state.rs` | 共享状态 | **重写** - 移除 redb 依赖 |

---

## 2. 新架构设计

### 2.1 目录结构规划

```
src/
├── transaction/
│   ├── mod.rs                    # 模块入口
│   ├── types.rs                  # 事务类型定义 (保留)
│   ├── version_manager.rs        # [新增] MVCC 时间戳管理
│   ├── read_transaction.rs       # [新增] 只读快照事务
│   ├── insert_transaction.rs     # [新增] 插入事务
│   ├── update_transaction.rs     # [新增] 更新事务
│   ├── compact_transaction.rs    # [新增] 压缩事务
│   ├── undo_log.rs               # [新增] Undo Log 实现
│   ├── wal/
│   │   ├── mod.rs                # WAL 模块入口
│   │   ├── writer.rs             # WAL 写入器
│   │   ├── parser.rs             # WAL 解析器
│   │   └── types.rs              # WAL 类型定义
│   ├── context.rs                # 事务上下文 (重写)
│   ├── manager.rs                # 事务管理器 (重写)
│   ├── cleaner.rs                # 事务清理 (保留)
│   ├── monitor.rs                # 事务监控 (保留)
│   └── index_buffer.rs           # 索引缓冲 (保留)
│
├── storage/
│   ├── mod.rs                    # 模块入口
│   ├── property_graph.rs         # [新增] PropertyGraph 主入口
│   ├── container/
│   │   ├── mod.rs                # 容器模块入口
│   │   ├── mmap_container.rs     # [新增] mmap 容器
│   │   ├── arena_allocator.rs    # [新增] Arena 分配器
│   │   └── types.rs              # 容器类型定义
│   ├── vertex/
│   │   ├── mod.rs                # 顶点存储模块入口
│   │   ├── vertex_table.rs       # [新增] 顶点表 (列存储)
│   │   ├── id_indexer.rs         # [新增] ID 索引器
│   │   ├── column_store.rs       # [新增] 列存储
│   │   └── vertex_timestamp.rs   # [新增] 顶点时间戳
│   ├── edge/
│   │   ├── mod.rs                # 边存储模块入口
│   │   ├── csr.rs                # [新增] CSR 边存储
│   │   ├── mutable_csr.rs        # [新增] 可变 CSR
│   │   ├── edge_table.rs         # [新增] 边表
│   │   └── property_table.rs     # [新增] 边属性表
│   ├── schema/
│   │   ├── mod.rs                # Schema 模块入口
│   │   ├── schema_manager.rs     # Schema 管理 (保留并扩展)
│   │   └── type_registry.rs      # [新增] 类型注册表
│   ├── index/
│   │   └── ...                   # 索引模块 (保留并适配)
│   ├── operations/
│   │   ├── mod.rs                # 操作模块入口
│   │   ├── traits.rs             # 操作接口 (保留)
│   │   └── rollback.rs           # 回滚操作 (重写)
│   ├── iterator/
│   │   ├── mod.rs                # 迭代器模块入口
│   │   ├── vertex_iter.rs        # [新增] 顶点迭代器
│   │   └── edge_iter.rs          # [新增] CSR 边迭代器
│   └── shared_state.rs           # 共享状态 (重写)
```

### 2.2 核心组件设计

#### 2.2.1 VersionManager (MVCC 时间戳管理)

```rust
pub struct VersionManager {
    write_ts: AtomicU32,      // 下一个写时间戳
    read_ts: AtomicU32,       // 当前读时间戳
    pending_reqs: AtomicI32,  // 待处理事务数
    pending_update_reqs: AtomicI32,
    timestamp_buffer: BitSet, // 时间戳环形缓冲
}

impl VersionManager {
    pub fn acquire_read_timestamp(&self) -> Timestamp;
    pub fn release_read_timestamp(&self, ts: Timestamp);
    pub fn acquire_insert_timestamp(&self) -> Timestamp;
    pub fn release_insert_timestamp(&self, ts: Timestamp);
    pub fn acquire_update_timestamp(&self) -> Timestamp;
    pub fn release_update_timestamp(&self, ts: Timestamp);
}
```

#### 2.2.2 WAL (Write-Ahead Log)

```rust
pub struct WalHeader {
    length: u32,
    op_type: WalOpType,
    timestamp: u32,
}

pub struct WalWriter {
    fd: File,
    wal_path: PathBuf,
    file_size: usize,
    file_used: usize,
}

pub enum WalOpType {
    InsertVertex = 0,
    InsertEdge = 1,
    CreateVertexType = 2,
    CreateEdgeType = 3,
    AddVertexProp = 4,
    AddEdgeProp = 5,
    UpdateVertexProp = 6,
    UpdateEdgeProp = 7,
    DeleteVertex = 8,
    DeleteEdge = 9,
}
```

#### 2.2.3 Undo Log

```rust
pub trait UndoLog: Send + Sync {
    fn undo(&self, graph: &mut PropertyGraph, ts: Timestamp) -> Result<(), StorageError>;
}

pub struct InsertVertexUndo {
    label: LabelId,
    vid: VertexId,
}

pub struct InsertEdgeUndo {
    src_label: LabelId,
    dst_label: LabelId,
    edge_label: LabelId,
    src_vid: VertexId,
    dst_vid: VertexId,
    oe_offset: i32,
    ie_offset: i32,
}

pub struct UpdateVertexPropUndo {
    label: LabelId,
    vid: VertexId,
    col_id: i32,
    old_value: Value,
}
```

#### 2.2.4 VertexTable (列存储)

```rust
pub struct VertexTable {
    label: LabelId,
    id_indexer: IdIndexer<String>,  // 外部ID -> 内部ID 映射
    columns: Vec<Column>,            // 列存储
    timestamps: VertexTimestamp,     // MVCC 时间戳
}

pub struct Column {
    col_id: i32,
    data_type: DataType,
    nullable: bool,
    data: Vec<u8>,                   // 列数据
    null_bitmap: Option<BitSet>,     // NULL 位图
}

pub struct VertexTimestamp {
    start_ts: Vec<u32>,              // 创建时间戳
    end_ts: Vec<u32>,                // 删除时间戳
}
```

#### 2.2.5 CSR Edge Storage

```rust
pub struct Csr<Edge> {
    offsets: Vec<u32>,               // 顶点偏移数组
    edges: Vec<Edge>,                // 边数组
}

pub struct MutableCsr {
    offsets: Vec<u32>,
    edges: Vec<EdgeRecord>,
    capacity: usize,
}

pub struct EdgeTable {
    label: LabelId,
    out_csr: MutableCsr,             // 出边 CSR
    in_csr: MutableCsr,              // 入边 CSR
    properties: PropertyTable,       // 边属性表
}

pub struct EdgeRecord {
    dst: VertexId,
    edge_id: EdgeId,
    prop_offset: u32,                // 属性偏移
    timestamp: u32,                  // MVCC 时间戳
}
```

---

## 3. 分阶段迁移方案

### 阶段一：基础设施层 (Phase 1)

**目标**：建立新架构的基础设施，不破坏现有功能

**预计工作量**：2-3 周

#### 3.1.1 新增文件

| 文件 | 描述 |
|------|------|
| `src/storage/container/mod.rs` | 容器模块入口 |
| `src/storage/container/mmap_container.rs` | mmap 容器实现 |
| `src/storage/container/arena_allocator.rs` | Arena 内存分配器 |
| `src/storage/container/types.rs` | 容器类型定义 |
| `src/transaction/wal/mod.rs` | WAL 模块入口 |
| `src/transaction/wal/writer.rs` | WAL 写入器 |
| `src/transaction/wal/parser.rs` | WAL 解析器 |
| `src/transaction/wal/types.rs` | WAL 类型定义 |
| `src/transaction/version_manager.rs` | MVCC 时间戳管理 |
| `src/transaction/undo_log.rs` | Undo Log 实现 |

#### 3.1.2 实现细节

**mmap_container.rs**:
```rust
pub struct MmapContainer {
    fd: File,
    mmap: MmapMut,
    size: usize,
    capacity: usize,
}

impl MmapContainer {
    pub fn create(path: &Path, capacity: usize) -> Result<Self, StorageError>;
    pub fn open(path: &Path) -> Result<Self, StorageError>;
    pub fn write(&mut self, offset: usize, data: &[u8]) -> Result<(), StorageError>;
    pub fn read(&self, offset: usize, len: usize) -> &[u8];
    pub fn sync(&self) -> Result<(), StorageError>;
    pub fn resize(&mut self, new_capacity: usize) -> Result<(), StorageError>;
}
```

**version_manager.rs**:
```rust
pub struct VersionManager {
    write_ts: AtomicU32,
    read_ts: AtomicU32,
    pending_reqs: AtomicI32,
    pending_update_reqs: AtomicI32,
    buffer: RwLock<BitSet>,
    config: VersionManagerConfig,
}

pub struct VersionManagerConfig {
    pub max_concurrent_reads: usize,
    pub max_concurrent_inserts: usize,
}
```

#### 3.1.3 测试要点

- mmap 容器的读写正确性
- mmap 容器的扩展和收缩
- WAL 写入和解析的正确性
- WAL 崩溃恢复测试
- VersionManager 并发时间戳分配
- Undo Log 执行的正确性

---

### 阶段二：存储层核心 (Phase 2)

**目标**：实现 CSR 边存储和列式顶点存储

**预计工作量**：3-4 周

#### 3.2.1 新增文件

| 文件 | 描述 |
|------|------|
| `src/storage/vertex/mod.rs` | 顶点存储模块入口 |
| `src/storage/vertex/vertex_table.rs` | 顶点表实现 |
| `src/storage/vertex/id_indexer.rs` | ID 索引器 |
| `src/storage/vertex/column_store.rs` | 列存储实现 |
| `src/storage/vertex/vertex_timestamp.rs` | 顶点时间戳 |
| `src/storage/edge/mod.rs` | 边存储模块入口 |
| `src/storage/edge/csr.rs` | CSR 实现 |
| `src/storage/edge/mutable_csr.rs` | 可变 CSR |
| `src/storage/edge/edge_table.rs` | 边表实现 |
| `src/storage/edge/property_table.rs` | 边属性表 |
| `src/storage/property_graph.rs` | PropertyGraph 主入口 |

#### 3.2.2 实现细节

**vertex_table.rs**:
```rust
pub struct VertexTable {
    label: LabelId,
    label_name: String,
    id_indexer: IdIndexer<String>,
    columns: Vec<ColumnStore>,
    timestamps: VertexTimestamp,
    schema: Schema,
}

impl VertexTable {
    pub fn new(label: LabelId, label_name: String, schema: Schema) -> Self;
    pub fn insert(&mut self, vid: VertexId, properties: &[(String, Value)]) -> Result<i32, StorageError>;
    pub fn get(&self, internal_id: i32) -> Option<VertexRecord>;
    pub fn get_property(&self, internal_id: i32, col_id: i32) -> Option<Value>;
    pub fn update_property(&mut self, internal_id: i32, col_id: i32, value: Value) -> Result<(), StorageError>;
    pub fn delete(&mut self, internal_id: i32, ts: Timestamp) -> Result<(), StorageError>;
    pub fn scan(&self, ts: Timestamp) -> VertexIterator;
}
```

**csr.rs**:
```rust
pub struct Csr<Edge> {
    offsets: Vec<u32>,
    edges: Vec<Edge>,
}

impl<Edge> Csr<Edge> {
    pub fn new() -> Self;
    pub fn edges_of(&self, vid: VertexId) -> &[Edge];
    pub fn degree(&self, vid: VertexId) -> usize;
    pub fn iter(&self) -> CsrIterator<Edge>;
}

pub struct MutableCsr {
    offsets: Vec<u32>,
    edges: Vec<EdgeRecord>,
    deleted_edges: HashSet<EdgeId>,
}

impl MutableCsr {
    pub fn new(vertex_capacity: usize) -> Self;
    pub fn insert_edge(&mut self, src: VertexId, edge: EdgeRecord) -> Result<(), StorageError>;
    pub fn delete_edge(&mut self, src: VertexId, edge_id: EdgeId) -> Result<(), StorageError>;
    pub fn edges_of(&self, vid: VertexId) -> EdgeIterator;
    pub fn degree(&self, vid: VertexId) -> usize;
    pub fn compact(&mut self) -> Csr<EdgeRecord>;
}
```

**property_graph.rs**:
```rust
pub struct PropertyGraph {
    vertex_tables: HashMap<LabelId, VertexTable>,
    edge_tables: HashMap<LabelId, EdgeTable>,
    schema_manager: SchemaManager,
    version_manager: Arc<VersionManager>,
    wal_writer: Arc<RwLock<WalWriter>>,
}

impl PropertyGraph {
    pub fn open(path: &Path) -> Result<Self, StorageError>;
    pub fn create_vertex_type(&mut self, name: &str, schema: Schema) -> Result<LabelId, StorageError>;
    pub fn create_edge_type(&mut self, name: &str, schema: Schema) -> Result<LabelId, StorageError>;
    pub fn get_vertex_table(&self, label: LabelId) -> Option<&VertexTable>;
    pub fn get_edge_table(&self, label: LabelId) -> Option<&EdgeTable>;
}
```

#### 3.2.3 需要修改的文件

| 文件 | 修改内容 |
|------|----------|
| `src/storage/mod.rs` | 添加新模块导出 |
| `src/storage/metadata/schema.rs` | 扩展 Schema 支持列存储 |

#### 3.2.4 测试要点

- VertexTable 的 CRUD 操作
- 列存储的读写性能
- CSR 边插入和删除
- CSR 边遍历正确性
- MutableCsr 的 compact 操作
- PropertyGraph 的整体功能

---

### 阶段三：事务层重构 (Phase 3)

**目标**：实现新的事务类型，集成 MVCC 和 WAL

**预计工作量**：2-3 周

#### 3.3.1 新增文件

| 文件 | 描述 |
|------|------|
| `src/transaction/read_transaction.rs` | 只读快照事务 |
| `src/transaction/insert_transaction.rs` | 插入事务 |
| `src/transaction/update_transaction.rs` | 更新事务 |
| `src/transaction/compact_transaction.rs` | 压缩事务 |

#### 3.3.2 重写文件

| 文件 | 重写内容 |
|------|----------|
| `src/transaction/context.rs` | 移除 redb 依赖，采用 MVCC 时间戳 |
| `src/transaction/manager.rs` | 集成 VersionManager，管理不同类型事务 |
| `src/transaction/mod.rs` | 更新模块导出 |

#### 3.3.3 实现细节

**read_transaction.rs**:
```rust
pub struct ReadTransaction<'a> {
    graph: &'a PropertyGraph,
    version_manager: &'a VersionManager,
    timestamp: Timestamp,
}

impl<'a> ReadTransaction<'a> {
    pub fn new(graph: &'a PropertyGraph, vm: &'a VersionManager) -> Result<Self, StorageError> {
        let timestamp = vm.acquire_read_timestamp();
        Ok(Self { graph, version_manager: vm, timestamp })
    }

    pub fn get_vertex(&self, label: LabelId, vid: VertexId) -> Option<VertexRecord>;
    pub fn get_edge(&self, label: LabelId, src: VertexId, dst: VertexId) -> Option<EdgeRecord>;
    pub fn scan_vertices(&self, label: LabelId) -> VertexIterator;
    pub fn scan_edges(&self, label: LabelId, src: VertexId) -> EdgeIterator;
}

impl<'a> Drop for ReadTransaction<'a> {
    fn drop(&mut self) {
        self.version_manager.release_read_timestamp(self.timestamp);
    }
}
```

**insert_transaction.rs**:
```rust
pub struct InsertTransaction<'a> {
    graph: &'a mut PropertyGraph,
    version_manager: &'a VersionManager,
    wal_writer: &'a mut WalWriter,
    timestamp: Timestamp,
    wal_buffer: Vec<u8>,
}

impl<'a> InsertTransaction<'a> {
    pub fn new(
        graph: &'a mut PropertyGraph,
        vm: &'a VersionManager,
        wal: &'a mut WalWriter,
    ) -> Result<Self, StorageError> {
        let timestamp = vm.acquire_insert_timestamp();
        Ok(Self { graph, version_manager: vm, wal_writer: wal, timestamp, wal_buffer: Vec::new() })
    }

    pub fn insert_vertex(&mut self, label: LabelId, properties: Vec<(String, Value)>) -> Result<VertexId, StorageError>;
    pub fn insert_edge(&mut self, label: LabelId, src: VertexId, dst: VertexId, properties: Vec<(String, Value)>) -> Result<EdgeId, StorageError>;

    pub fn commit(mut self) -> Result<(), StorageError> {
        self.wal_writer.append(&self.wal_buffer)?;
        self.version_manager.release_insert_timestamp(self.timestamp);
        Ok(())
    }

    pub fn abort(mut self) -> Result<(), StorageError> {
        self.version_manager.release_insert_timestamp(self.timestamp);
        Ok(())
    }
}
```

**update_transaction.rs**:
```rust
pub struct UpdateTransaction<'a> {
    graph: &'a mut PropertyGraph,
    version_manager: &'a VersionManager,
    wal_writer: &'a mut WalWriter,
    timestamp: Timestamp,
    undo_logs: Vec<Box<dyn UndoLog>>,
    wal_buffer: Vec<u8>,
}

impl<'a> UpdateTransaction<'a> {
    pub fn new(
        graph: &'a mut PropertyGraph,
        vm: &'a VersionManager,
        wal: &'a mut WalWriter,
    ) -> Result<Self, StorageError> {
        let timestamp = vm.acquire_update_timestamp();
        Ok(Self { graph, version_manager: vm, wal_writer: wal, timestamp, undo_logs: Vec::new(), wal_buffer: Vec::new() })
    }

    pub fn create_vertex_type(&mut self, name: &str, schema: Schema) -> Result<LabelId, StorageError>;
    pub fn create_edge_type(&mut self, name: &str, schema: Schema) -> Result<LabelId, StorageError>;
    pub fn add_vertex_property(&mut self, label: LabelId, prop: PropertyDef) -> Result<(), StorageError>;
    pub fn update_vertex(&mut self, label: LabelId, vid: VertexId, properties: Vec<(String, Value)>) -> Result<(), StorageError>;
    pub fn delete_vertex(&mut self, label: LabelId, vid: VertexId) -> Result<(), StorageError>;

    pub fn commit(mut self) -> Result<(), StorageError> {
        self.wal_writer.append(&self.wal_buffer)?;
        self.version_manager.release_update_timestamp(self.timestamp);
        Ok(())
    }

    pub fn abort(mut self) -> Result<(), StorageError> {
        while let Some(undo) = self.undo_logs.pop() {
            undo.undo(&mut self.graph, self.timestamp)?;
        }
        self.version_manager.release_update_timestamp(self.timestamp);
        Ok(())
    }
}
```

#### 3.3.4 测试要点

- ReadTransaction 的快照隔离
- InsertTransaction 的并发插入
- UpdateTransaction 的 DDL 操作
- UpdateTransaction 的回滚正确性
- WAL 持久化和恢复
- 事务超时和清理

---

### 阶段四：接口适配层 (Phase 4)

**目标**：适配现有 API 接口，保持向后兼容

**预计工作量**：1-2 周

#### 3.4.1 需要修改的文件

| 文件 | 修改内容 |
|------|----------|
| `src/storage/entity/vertex_storage.rs` | 重写为 VertexTable 的适配层 |
| `src/storage/entity/edge_storage.rs` | 重写为 EdgeTable 的适配层 |
| `src/storage/operations/traits/reader.rs` | 适配新存储接口 |
| `src/storage/operations/traits/writer.rs` | 适配新存储接口 |
| `src/storage/operations/rollback.rs` | 基于 Undo Log 重写 |
| `src/storage/iterator/storage_iter.rs` | 重写为 CSR 迭代器 |
| `src/storage/shared_state.rs` | 移除 redb 依赖 |
| `src/storage/engine/mod.rs` | 移除 redb_storage 导出 |

#### 3.4.2 需要删除的文件

| 文件 | 删除原因 |
|------|----------|
| `src/storage/engine/redb_storage.rs` | 替换为 PropertyGraph |
| `src/storage/engine/redb_types.rs` | 替换为 CSR/列存储类型 |
| `src/storage/operations/redb/reader.rs` | 替换为原生实现 |
| `src/storage/operations/redb/writer.rs` | 替换为原生实现 |
| `src/storage/operations/redb/mod.rs` | 模块已删除 |

#### 3.4.3 适配层实现

**vertex_storage.rs (适配层)**:
```rust
pub struct VertexStorage {
    graph: Arc<RwLock<PropertyGraph>>,
    version_manager: Arc<VersionManager>,
}

impl VertexStorage {
    pub fn get_vertex(&self, space: &str, id: &Value) -> Result<Option<Vertex>, StorageError> {
        let graph = self.graph.read();
        let txn = ReadTransaction::new(&graph, &self.version_manager)?;
        let label = self.get_label_id(space)?;
        let internal_id = graph.get_vertex_table(label)?.id_indexer().get(id)?;
        Ok(txn.get_vertex(label, internal_id).map(|r| r.to_vertex()))
    }

    pub fn insert_vertex(&self, space: &str, vertex: Vertex) -> Result<Value, StorageError> {
        let mut graph = self.graph.write();
        let mut wal = self.wal_writer.write();
        let mut txn = InsertTransaction::new(&mut graph, &self.version_manager, &mut wal)?;
        let label = self.get_label_id(space)?;
        let properties = vertex.to_properties();
        let vid = txn.insert_vertex(label, properties)?;
        txn.commit()?;
        Ok(vid)
    }
}
```

#### 3.4.4 测试要点

- 现有 API 的兼容性测试
- 性能回归测试
- 数据迁移测试

---

### 阶段五：清理与优化 (Phase 5)

**目标**：移除 redb 依赖，优化性能

**预计工作量**：1 周

#### 3.5.1 需要删除的文件/目录

| 文件/目录 | 删除原因 |
|-----------|----------|
| `src/storage/engine/redb_storage.rs` | 已替换 |
| `src/storage/engine/redb_types.rs` | 已替换 |
| `src/storage/operations/redb/` | 已替换 |

#### 3.5.2 Cargo.toml 修改

```toml
[dependencies]
# 移除
# redb = "2.x"

# 新增
memmap2 = "0.9"          # mmap 支持
bit-set = "0.8"          # BitSet 实现
```

#### 3.5.3 性能优化

- CSR 边遍历优化
- 列存储压缩
- mmap 预读优化
- WAL 批量写入

---

## 4. 代码保留/修改/删除汇总

### 4.1 事务模块 (src/transaction/)

| 文件 | 操作 | 说明 |
|------|------|------|
| `mod.rs` | **重写** | 移除 redb 依赖，导出新事务类型 |
| `context.rs` | **重写** | 采用 MVCC 时间戳，移除 redb 事务 |
| `manager.rs` | **重写** | 集成 VersionManager |
| `types.rs` | **保留+扩展** | 保留现有类型，扩展新类型 |
| `cleaner.rs` | **保留+适配** | 适配新事务类型 |
| `monitor.rs` | **保留** | 无需修改 |
| `index_buffer.rs` | **保留+适配** | 适配新架构 |
| `version_manager.rs` | **新增** | MVCC 时间戳管理 |
| `read_transaction.rs` | **新增** | 只读快照事务 |
| `insert_transaction.rs` | **新增** | 插入事务 |
| `update_transaction.rs` | **新增** | 更新事务 |
| `compact_transaction.rs` | **新增** | 压缩事务 |
| `undo_log.rs` | **新增** | Undo Log 实现 |
| `wal/mod.rs` | **新增** | WAL 模块 |
| `wal/writer.rs` | **新增** | WAL 写入器 |
| `wal/parser.rs` | **新增** | WAL 解析器 |
| `wal/types.rs` | **新增** | WAL 类型定义 |

### 4.2 存储模块 (src/storage/)

| 文件/目录 | 操作 | 说明 |
|-----------|------|------|
| `mod.rs` | **修改** | 更新模块导出 |
| `engine/redb_storage.rs` | **删除** | 替换为 PropertyGraph |
| `engine/redb_types.rs` | **删除** | 替换为 CSR/列存储类型 |
| `engine/runtime_context.rs` | **保留+适配** | 适配新架构 |
| `engine/mod.rs` | **修改** | 移除 redb 导出 |
| `entity/vertex_storage.rs` | **重写** | 采用列存储 |
| `entity/edge_storage.rs` | **重写** | 采用 CSR |
| `entity/user_storage.rs` | **保留** | 独立模块 |
| `operations/redb/` | **删除** | 替换为原生实现 |
| `operations/traits/` | **保留+适配** | 适配新接口 |
| `operations/rollback.rs` | **重写** | 基于 Undo Log |
| `operations/write_txn_executor.rs` | **删除** | 不再需要 |
| `metadata/` | **保留+适配** | 适配新存储 |
| `index/` | **保留+适配** | 适配新存储 |
| `iterator/` | **重写** | CSR 迭代器 |
| `extend/` | **保留+适配** | 适配新存储 |
| `monitoring/` | **保留** | 无需修改 |
| `api/` | **保留+适配** | 适配新接口 |
| `shared_state.rs` | **重写** | 移除 redb 依赖 |
| `test_mock.rs` | **重写** | 适配新架构 |
| `container/` | **新增** | mmap 容器模块 |
| `vertex/` | **新增** | 顶点存储模块 |
| `edge/` | **新增** | 边存储模块 |
| `property_graph.rs` | **新增** | PropertyGraph 主入口 |

---

## 5. 风险与缓解措施

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 数据迁移复杂 | 高 | 提供迁移工具，支持增量迁移 |
| API 兼容性 | 中 | 保持接口不变，内部重写 |
| 性能回归 | 中 | 分阶段性能测试，及时优化 |
| 并发正确性 | 高 | 完善的并发测试，压力测试 |
| 崩溃恢复 | 高 | WAL 恢复测试，混沌测试 |

---

## 6. 测试策略

### 6.1 单元测试

- 每个新组件必须有单元测试
- 测试覆盖率 > 80%

### 6.2 集成测试

- 事务正确性测试
- 并发事务测试
- 崩溃恢复测试

### 6.3 性能测试

- CSR 边遍历性能
- 列存储读写性能
- 并发事务吞吐量

### 6.4 压力测试

- 长时间运行测试
- 大数据量测试
- 高并发测试

---

## 7. 时间线

| 阶段 | 内容 | 预计时间 |
|------|------|----------|
| Phase 1 | 基础设施层 | 第 1-3 周 |
| Phase 2 | 存储层核心 | 第 4-7 周 |
| Phase 3 | 事务层重构 | 第 8-10 周 |
| Phase 4 | 接口适配层 | 第 11-12 周 |
| Phase 5 | 清理与优化 | 第 13 周 |
| **总计** | | **约 13 周** |

---

## 8. 附录

### 8.1 NeuG 参考文件

- `transaction/version_manager.cc` - MVCC 时间戳管理
- `transaction/wal/wal.cc` - WAL 实现
- `transaction/undo_log.cc` - Undo Log 实现
- `storage/vertex_table/` - 顶点表实现
- `storage/edge_table/` - 边表实现
- `storage/csr/` - CSR 实现

### 8.2 相关文档

- `docs/storage/neug_storage_migration_analysis.md` - 迁移分析文档
- `docs/archive/unsafe.md` - unsafe 使用记录
- `docs/archive/dynamic.md` - 动态分发记录
