# 存储层架构改造建议

> 本文档记录了存储层架构的改进建议，涉及需要较大重构的部分。这些建议暂不执行，待后续评估后决定是否实施。

## 一、简化中间层：合并 SchemaOps 和 EdgeOps

### 当前问题

当前存在三层管理结构：
1. `SchemaOps` 管理 `HashMap<LabelId, VertexTable>`
2. `EdgeOps` 管理 `HashMap<LabelId, EdgeTable>`
3. `PropertyGraph` 持有 `SchemaOps` 和 `EdgeOps`

这种设计导致：
- 方法调用链过长：`PropertyGraph → SchemaOps → VertexTable`
- 状态分散在多个结构体中
- 事务操作需要跨多个组件协调

### 建议方案

将 `SchemaOps` 和 `EdgeOps` 的功能直接合并到 `PropertyGraph` 中：

```rust
pub struct PropertyGraph {
    // 直接管理表，不通过中间层
    vertex_tables: HashMap<LabelId, VertexTable>,
    edge_tables: HashMap<LabelId, EdgeTable>,
    
    // 其他组件保持不变
    cache_manager: CacheManager,
    wal_manager: Option<WalManager>,
    index_data_manager: SharedIndexDataManager,
    config: EngineConfig,
}
```

**优势**：
- 减少一层间接调用
- 状态集中管理
- 事务协调更简单

**风险**：
- `PropertyGraph` 文件会变大（可通过 impl 分块缓解）

---

## 二、统一事务模块

### 当前问题

事务相关代码分散在多个位置：
- `engine/transaction.rs` - 基础事务操作
- `property_graph/transaction_targets/` - undo/compact/recovery
- `graph_storage/transaction_support.rs` - 事务支持
- `graph_storage/transactional_writer.rs` - 事务写入器

### 建议方案

统一事务模块到 `engine/transaction/` 子目录：

```
src/storage/engine/transaction/
├── mod.rs           # 事务入口
├── context.rs       # 事务上下文
├── ops.rs           # 事务操作（原 transaction.rs）
├── undo.rs          # 撤销逻辑
├── redo.rs          # 重做逻辑
├── recovery.rs      # 恢复逻辑
├── compact.rs       # 压缩逻辑
└── writer.rs        # 事务写入器
```

**优势**：
- 事务逻辑集中，易于理解和维护
- 减少跨模块依赖
- 便于添加新事务特性

---

## 三、明确持久化责任链

### 当前问题

持久化逻辑分散在三个组件中：
1. `WalManager` - WAL 日志管理
2. `PropertyGraph.flush()/load_data()` - 内存刷新/加载
3. `PersistenceCoordinator` - Checkpoint/Snapshot 协调

三者之间的协调关系不够清晰，存在以下问题：
- WAL 和 flush 的触发时机不明确
- Checkpoint 和 flush 的职责重叠
- 恢复流程涉及多个组件的协调

### 建议方案

明确持久化责任链：

```
写入操作
    ↓
WAL (Write-Ahead Log) - 保证持久性
    ↓
Memory (内存) - 提供快速访问
    ↓
Flush (定期) - 将内存数据写入磁盘
    ↓
Checkpoint (定期) - 创建一致性快照
    ↓
Snapshot (手动) - 用户触发的完整备份
```

**具体改进**：
1. 将 `PropertyGraph.flush()` 重命名为 `PropertyGraph.flush_to_disk()`，明确其职责
2. 在 `PersistenceCoordinator` 中统一管理所有持久化触发逻辑
3. 添加 `PersistenceState` 枚举跟踪当前持久化状态

---

## 四、精简 GraphStorageContext

### 当前问题

`GraphStorageContext` 持有 9 个大组件：
```rust
pub struct GraphStorageContext {
    pub graph: Arc<PropertyGraph>,                    // 1
    pub schema_manager: Arc<SchemaManager>,           // 2
    pub extended_schema_manager: Arc<ExtendedSchemaManager>, // 3
    pub index_metadata_manager: Arc<IndexManager>,    // 4
    pub version_manager: Arc<VersionManager>,         // 5
    pub user_storage: Arc<UserStorage>,               // 6
    pub current_txn_context: Arc<RwLock<Option<...>>>, // 7
    pub persistence: Option<Arc<RwLock<...>>>,        // 8
    pub index_gc_manager: Option<Arc<IndexGcManager>>, // 9
    pub fulltext_storage: Option<Arc<FulltextStorage>>, // 10
    pub work_dir: Option<PathBuf>,
    pub db_path: String,
}
```

### 建议方案

**方案 A：拆分为子 Context**

```rust
pub struct GraphStorageContext {
    pub core: CoreStorageContext,      // graph, version_manager
    pub schema: SchemaContext,         // schema_manager, extended_schema_manager
    pub index: IndexContext,           // index_metadata_manager, index_gc_manager
    pub persistence: PersistenceContext, // persistence, wal_manager
    pub extensions: ExtensionContext,   // user_storage, fulltext_storage
    pub txn_state: TransactionState,    // current_txn_context
}
```

**方案 B：依赖注入模式**

```rust
pub struct GraphStorageContext {
    pub graph: Arc<PropertyGraph>,
    pub config: StorageConfig,  // 包含所有可选组件的配置
}

// 通过 graph 访问其他组件
impl GraphStorageContext {
    pub fn schema_manager(&self) -> &SchemaManager {
        self.graph.schema_manager()
    }
}
```

**推荐方案 B**，因为：
- 保持 API 简单
- 减少结构体嵌套
- 通过 PropertyGraph 作为统一入口

---

## 五、边属性存储支持压缩编码

### 当前问题

顶点使用列式存储 + 7 种压缩编码，而边属性使用行式存储且无压缩支持。

| 方面 | 顶点 (ColumnStore) | 边 (PropertyTable) |
|------|-------------------|-------------------|
| 存储方向 | 列式 | 行式 |
| 压缩 | 7 种编码类型 | 无 |
| 空值处理 | 每列专用 BitVec | 每单元格 Option<Value> |
| 内存效率 | 高（类型特定编码） | 低（每行开销） |

### 建议方案

**短期方案**：为 `PropertyTable` 添加可选的列式存储模式

```rust
pub struct PropertyTable {
    // 当前行式存储（保持兼容）
    row_groups: Vec<RowGroup>,
    
    // 新增：可选列式存储
    column_store: Option<ColumnStore>,
    
    // 配置
    storage_mode: PropertyStorageMode,
}

pub enum PropertyStorageMode {
    Row,      // 行式（默认，适合点查）
    Column,   // 列式（适合扫描）
    Hybrid,   // 混合（热数据行式，冷数据列式）
}
```

**长期方案**：考虑使用统一的存储格式

---

## 六、改进建议优先级

| 优先级 | 改进项 | 复杂度 | 收益 |
|--------|--------|--------|------|
| P0 | 统一事务模块 | 中 | 高 |
| P1 | 简化中间层 | 低 | 中 |
| P1 | 明确持久化责任链 | 中 | 高 |
| P2 | 精简 GraphStorageContext | 中 | 中 |
| P2 | 边属性存储支持压缩 | 高 | 中 |

---

## 七、已完成的局部改进

以下改进已在本次会话中完成：

### 1. 类型转换改进
- 在 `VertexRecord` 上实现 `From<&VertexRecord> for Vertex`
- 在 `EdgeRecord` 上实现 `From<&EdgeRecord> for Edge`
- 添加 `into_vertex_with_tag()` 和 `into_edge_with_type()` 方法

### 2. 编码模块激活
- 在 `Column` 上添加 `compute_stats()` 方法
- 在 `ColumnStore` 上添加 `apply_encoding_to_column()` 方法
- 在 `ColumnStore` 上添加 `auto_apply_encodings()` 方法

### 3. 缓存文档改进
- 完善 `CacheManager` 模块文档
- 明确说明顶点和边缓存的设计差异
- 添加 API 对称性说明
