## 节点存储架构分析

### 一、整体架构

节点存储采用**分层架构**，从上到下分为：

```
GraphStorage (顶层入口)
    └── PropertyGraph (属性图管理)
            └── VertexTable (顶点表，按Label分组)
                    ├── IdIndexer<String> (外部ID ↔ 内部ID映射)
                    ├── ColumnStore (列式属性存储)
                    └── VertexTimestamp (MVCC时间戳追踪)
```

### 二、核心组件详解

#### 1. [VertexTable](file:///d:/项目/database/graphDB/src/storage/vertex/vertex_table.rs#L44-L56)

```rust
pub struct VertexTable {
    label: LabelId,           // 标签ID
    label_name: String,       // 标签名称
    schema: VertexSchema,     // 模式定义
    id_indexer: IdIndexer<String>,  // ID映射
    columns: ColumnStore,     // 列式存储
    timestamps: VertexTimestamp,    // MVCC时间戳
    config: VertexTableConfig,
    is_open: bool,
}
```

**职责**：管理同一Label下所有顶点的存储，是节点存储的核心单元。

#### 2. [IdIndexer<K>](file:///d:/项目/database/graphDB/src/storage/vertex/id_indexer.rs#L13-L18)

```rust
pub struct IdIndexer<K> {
    keys: Vec<K>,                    // 内部ID → 外部ID
    key_to_index: HashMap<K, u32>,   // 外部ID → 内部ID
    capacity: usize,
}
```

**设计**：

- 双向映射：外部ID(String) ↔ 内部ID(u32)
- O(1) 查找复杂度
- 内部ID是连续的 u32，便于数组索引

#### 3. [ColumnStore](file:///d:/项目/database/graphDB/src/storage/vertex/column_store.rs#L19-L24)

```rust
pub struct Column {
    name: String,
    data_type: DataType,
    nullable: bool,
    data: Vec<u8>,           // 紧凑二进制存储
    offsets: Vec<usize>,     // 变长类型偏移量
    null_bitmap: Option<Vec<bool>>,
}
```

**设计**：

- 列式存储，每列独立存储同类型数据
- 定长类型：直接按偏移量存取
- 变长类型(String)：使用 offsets 数组 + 长度前缀

#### 4. [VertexTimestamp](file:///d:/项目/database/graphDB/src/storage/vertex/vertex_timestamp.rs#L11-L16)

```rust
pub struct VertexTimestamp {
    start_ts: Vec<Timestamp>,  // 创建时间戳
    end_ts: Vec<Timestamp>,    // 删除时间戳
    deleted: Vec<bool>,        // 删除标记
}
```

**设计**：MVCC实现，通过时间戳判断记录可见性。

---

### 三、数据流分析

#### 插入流程

```
insert_vertex(external_id, properties, ts)
    ↓
IdIndexer.insert(external_id) → internal_id
    ↓
VertexTimestamp.insert(internal_id, ts)
    ↓
ColumnStore.set(internal_id, properties)
```

#### 查询流程

```
get_vertex(external_id, ts)
    ↓
IdIndexer.get_index(external_id) → internal_id
    ↓
VertexTimestamp.is_valid(internal_id, ts) → 检查可见性
    ↓
ColumnStore.get(internal_id) → properties
```

---

### 四、设计评估

#### ✅ 优点

| 方面           | 说明                                                    |
| -------------- | ------------------------------------------------------- |
| **列式存储**   | 适合OLAP场景，同类型数据连续存储，压缩率高，CPU缓存友好 |
| **MVCC支持**   | 通过时间戳实现事务隔离，支持并发读写                    |
| **双向ID映射** | 内部ID连续，便于数组索引；外部ID灵活，支持字符串/整数   |
| **模块化设计** | 各组件职责清晰，易于测试和维护                          |
| **缓存集成**   | PropertyGraph层集成了RecordCache，支持热点数据缓存      |

#### ⚠️ 潜在问题

| 问题                | 详细分析                                                            | 建议改进                             |
| ------------------- | ------------------------------------------------------------------- | ------------------------------------ |
| **内存开销大**      | IdIndexer使用`String`作为外部ID，每个字符串独立分配堆内存           | 考虑使用字符串池或`Arc<str>`减少分配 |
| **变长存储效率**    | ColumnStore对String类型使用`offsets: Vec<usize>`，每行8字节额外开销 | 可考虑使用字典编码或前缀压缩         |
| **容量固定**        | `IdIndexer`有固定容量限制，超出返回`CapacityExceeded`错误           | 实现动态扩容机制                     |
| **删除空洞**        | 删除顶点后，内部ID不回收，造成数组空洞                              | `compact()`方法存在但需手动调用      |
| **无压缩**          | 列数据直接存储原始字节，未利用列式存储的压缩优势                    | 可集成RLE、Delta编码等               |
| **null_bitmap效率** | 使用`Vec<bool>`，每个空值标记占1字节                                | 应使用位图，1bit标记一个空值         |

#### 🔍 代码示例问题

[IdIndexer 容量限制](file:///d:/项目/database/graphDB/src/storage/vertex/id_indexer.rs#L42-L45):

```rust
pub fn insert(&mut self, key: K) -> StorageResult<u32> {
    if self.keys.len() >= self.capacity {
        return Err(StorageError::CapacityExceeded);  // 硬性限制
    }
    // ...
}
```

[ColumnStore null_bitmap](file:///d:/项目/database/graphDB/src/storage/vertex/column_store.rs#L21):

```rust
null_bitmap: Option<Vec<bool>>,  // 应使用 BitVec 或 [u8] 位图
```

---

### 五、与业界对比

| 特性     | 本项目      | Neo4j      | NebulaGraph     |
| -------- | ----------- | ---------- | --------------- |
| 存储模型 | 列式存储    | 原生图存储 | KV存储(RocksDB) |
| ID映射   | 内存HashMap | 直接偏移量 | Hash映射        |
| MVCC     | 时间戳数组  | 事务日志   | RocksDB快照     |
| 压缩     | 无          | 页级压缩   | RocksDB压缩     |

---

### 六、总结

**现有设计整体合理**，采用了列式存储+MVCC的经典组合，适合单机场景。

**主要改进建议**：

1. **内存优化**：使用字符串池、位图等减少内存占用
2. **压缩支持**：为列数据添加压缩（特别是字符串列）
3. **动态扩容**：移除固定容量限制，实现自动扩容
4. **垃圾回收**：实现自动的compact机制，回收删除空洞

对于个人使用和小规模应用场景，当前设计已经足够。如果需要支持更大规模数据，建议优先解决内存开销和压缩问题。
