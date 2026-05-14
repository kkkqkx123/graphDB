# PropertyGraph 多锁重构方案

## 背景

PropertyGraph 原本使用单个外部 `RwLock<PropertyGraph>` 来保护所有内部状态。这意味着任何对 schema 或 edge 的读写操作都会互相阻塞，即使是完全不相关的操作（如读取 schema 信息 vs 写入 edge 数据）。

## 重构方向：从单一外锁到细粒度内锁

将 PropertyGraph 的内部字段拆分为独立的锁保护单元：

### 已完成的改动

**结构拆分**

- `schema_ops` → `RwLock<SchemaOps>`：独立的 schema 读写锁
- `edge_ops` → `RwLock<EdgeOps>`：独立的 edge 读写锁
- `is_open` → `AtomicBool`：无锁原子变量
- `last_compacted_vertices` → `Mutex<Vec<...>>`：专用互斥锁
- `index_data_manager` → `RwLock<InMemoryIndexDataManager>`：独立的索引读写锁

**外部上下文调整**

- `GraphStorageContext.graph`：`RwLock<PropertyGraph>` → `Arc<PropertyGraph>`
- 所有调用方从 `ctx.graph.read()/write()` 改为直接 `ctx.graph.method()`

**Trait 签名调整**

- `UndoTarget`：`&mut self` → `&self`
- `CompactTarget`：`&mut self` → `&self`
- `RecoveryApplier`：`&mut self` → `&self`
- `UndoLogManager::execute_undo`：`&mut T` → `&T`
- `CompactTransaction`：`&'a mut T` → `&'a T`
- `TransactionSupport::rollback / with_rollback / execute_in_transaction`：`&mut PropertyGraph` → `&PropertyGraph`

**Bug 修复**

- `auth/user_storage.rs`：`.write()` → `.read()`（P1 误用写锁）

**问题分析**

- `docs/analysis/lock_usage_analysis.md`：全量锁分析，含 P1/P2/P3 优先级

### 存在的问题

当前编译尚未通过（11 个错误），核心问题分为两类：

#### 1. 借用到 RwLock 守卫导致的生命周期问题

原本 `scan_vertices()`、`vertex_tables()`、`edge_tables()`、`get_edge_table()` 等方法返回的数据引用自 `self`（原本在外部锁保护下是安全的）。拆分为内部锁后，这些方法需要从 `self.schema_ops.read()` 返回的守卫中借用数据，但守卫是局部变量，函数返回后即被释放。

**受影响方法**：
- `scan_vertices(label, ts) -> Option<VertexIterator<'_>>`
- `vertex_tables() -> &HashMap<LabelId, VertexTable>`
- `edge_tables() -> &HashMap<(LabelId,LabelId,LabelId), EdgeTable>`
- `get_edge_table(...) -> Option<&EdgeTable>`

**解决方案**：在锁内部收集数据，返回 owned 类型（VertexRecord 和 EdgeRecord 均已实现 Clone）。

#### 2. 签名冲突：`&self` vs `&mut self`

PropertyGraph 拆掉外部锁后，所有方法都改为 `&self`。但内部字段（`wal_manager`、`cache_manager`）的部分方法仍需要 `&mut self`。

**受影响方法**：
- `PropertyGraph::set_wal_writer(&self)` → 调用 `WalManager::set_wal_writer(&mut self)`
- `PropertyGraph::set_edge_property_cache(&self)` → 调用 `CacheManager::set_edge_property_cache(&mut self)`

**解决方案**：
- `wal_manager` 字段包装为 `Mutex<WalManager>`
- `CacheManager::edge_property_cache` 字段包装为 `Mutex<Option<Arc<EdgePropertyCache>>>`，使 `set_edge_property_cache` 可以接受 `&self`

此外，`UndoLogEntry::undo()` 仍使用 `&mut T`，需改为 `&T` 以匹配 `UndoTarget` trait 的新签名。

## 后续需要作出的修改

### 高优先级（编译阻塞）

1. **`mod.rs` - 修正守卫借用问题**
   - `scan_vertices`：内部收集 `Vec<VertexRecord>`，返回 `Option<Vec<VertexRecord>>`
   - 删除 `get_vertex_table`（未使用）
   - 删除 `vertex_tables()` 和 `edge_tables()` 的 HashMap 返回，替换为：
     - `total_vertex_count(&self) -> usize`
     - `total_edge_count(&self) -> usize`
     - `collect_all_edge_records(&self, ts) -> Vec<(LabelId, LabelId, LabelId, EdgeRecord)>`
   - 删除 `get_edge_table` 和 `get_edge_table_by_label`，替换为 `scan_edges(...)` 收集 Vec

2. **`mod.rs` - 修正 `&self` 可变性**
   - `wal_manager: WalManager` → `wal_manager: Mutex<WalManager>`
   - 所有调用点增加 `.lock()`

3. **`cache.rs` - 内部 Mutex 改造**
   - `edge_property_cache: Option<Arc<EdgePropertyCache>>` → `edge_property_cache: Mutex<Option<Arc<EdgePropertyCache>>>`
   - 所有读写该字段的方法更新
   - `set_edge_property_cache` 从 `&mut self` 改为 `&self`

4. **`undo_log.rs` - UndoLogEntry 签名修正**
   - 所有 `undo<T: UndoTarget + ?Sized>(&self, graph: &mut T, ...)` 改为 `graph: &T`

### 中优先级（调用方适配）

5. **`reader.rs` - scan_edges 适配**
   - `graph.get_edge_table(...).scan(ts)` → `graph.scan_edges(..., ts)`

6. **`persistence.rs` - vertex/edge 计数适配**
   - `graph.vertex_tables().values().map(...)` → `graph.total_vertex_count()`
   - `graph.edge_tables().values().map(...)` → `graph.total_edge_count()`

7. **`maintenance.rs` - 完整重构**
   - `get_storage_stats`：同上改为 count 方法
   - `find_dangling_edges`：改为从 `collect_all_edge_records` 获取数据，然后在锁外逐条检查

### 低优先级（清理）

8. **未使用导入清理**
   - 移除 `parking_lot::Mutex` 和 `parking_lot::RwLock` 的未使用导入（多个文件）

## 锁获取顺序约定

所有需要同时获取 `schema_ops` 和 `edge_ops` 的操作，必须**先获取 `schema_ops`，再获取 `edge_ops`**，以防止死锁。

## 性能预期

- **读-读不阻塞**：schema 的读锁不会阻塞 edge 的读锁（原本会被同一外部锁阻塞）
- **写-读不阻塞**：edge 写入不阻塞 schema 读取（vertex lookup/scan）
- **写-写互斥**：同类型的写操作仍然互斥
